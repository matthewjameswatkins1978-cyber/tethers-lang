//! Explicit M3 installation approval and immutable present-disabled state.
//!
//! This module does not expose enablement, operational resolution, provider
//! invocation, credentials, policy, replay, Trail, or Anchor admission.

use crate::candidate::CandidateRecord;
use crate::conformance::{current_suite_digest, ConformanceDisposition, ConformanceEvidence};
use crate::current_trust::{CurrentTrustAuthority, PublisherDeveloperTrustAuthority};
use crate::installation_publication_intent::InstallationPublicationIntent;
use crate::installation_recovery::InstallationRecoverySnapshot;
use crate::launch_profile::{revalidate_candidate, LaunchProfileEvidence};
use crate::m3_store::{
    canonical, reject_reparse, sha256, strict_json, unix_ms, verify_chain, M3Error, Result,
    StoreRoot,
};
use crate::manifest;
use crate::package::{CapabilityEvidence, PayloadEvidence};
use crate::trust::{DeveloperApprovalStore, PackageTrustEvidence, PublisherTrustStore};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path};
use uuid::Uuid;

#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use windows_sys::Win32::Storage::FileSystem::{
    GetFileAttributesW, SetFileAttributesW, FILE_ATTRIBUTE_READONLY, INVALID_FILE_ATTRIBUTES,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ReviewedCapability {
    pub capability_name: String,
    pub capability_version: u32,
    pub manifest_digest: String,
    pub provider_operation_name: String,
    pub effects: Vec<String>,
    pub permission_scope: Value,
    pub permission_scope_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InstallationApprovalRecord {
    pub schema_version: u32,
    pub approval_id: String,
    pub candidate_id: String,
    pub package_id: String,
    pub package_version: String,
    pub semantic_package_digest: String,
    pub raw_archive_digest: String,
    pub source_size_bytes: u64,
    pub payloads: Vec<PayloadEvidence>,
    pub reviewed_capabilities: Vec<ReviewedCapability>,
    pub trust_evidence: PackageTrustEvidence,
    pub provider_id: String,
    pub provider_version: String,
    pub launch_path: String,
    pub launch_arguments: Vec<String>,
    pub provider_working_directory: String,
    pub launch_profile_label: String,
    pub launch_profile_limitation: String,
    pub launch_profile_evidence_digest: String,
    pub conformance_evidence_id: String,
    pub conformance_evidence_digest: String,
    pub approving_authority: String,
    pub approved_unix_ms: u64,
    pub record_digest: String,
}

impl InstallationApprovalRecord {
    fn covered_bytes(&self) -> Result<Vec<u8>> {
        let mut copy = self.clone();
        copy.record_digest.clear();
        canonical(&copy)
    }

    pub(crate) fn require_for_recovery(
        &self,
        candidate: &CandidateRecord,
        quarantine: &Path,
        trust: &PackageTrustEvidence,
        launch: &LaunchProfileEvidence,
        conformance: &ConformanceEvidence,
    ) -> Result<()> {
        self.validate()?;
        if self.candidate_id != candidate.candidate_id
            || self.package_id != candidate.package_id
            || self.package_version != candidate.package_version
            || self.semantic_package_digest != candidate.semantic_package_digest
            || self.raw_archive_digest != candidate.raw_archive_digest
            || self.source_size_bytes != candidate.source_size_bytes
            || self.payloads != candidate.payloads
            || self.provider_id != candidate.provider_id
            || self.provider_version != candidate.provider_version
            || self.launch_path != candidate.launch_path
            || self.launch_arguments != candidate.launch_arguments
            || self.provider_working_directory != candidate.provider_working_directory
            || self.launch_profile_label != launch.profile_label
            || self.launch_profile_limitation != launch.limitation
            || self.launch_profile_evidence_digest != launch.profile_evidence_digest
            || self.trust_evidence != *trust
            || self.conformance_evidence_id != conformance.evidence_id
            || self.conformance_evidence_digest != conformance.evidence_digest
        {
            return Err(M3Error::new(
                "install_approval_stale",
                "approval pins drifted",
            ));
        }
        let reviewed = reviewed_capabilities(candidate, quarantine)?;
        if reviewed != self.reviewed_capabilities {
            return Err(M3Error::new(
                "install_approval_stale",
                "reviewed capabilities drifted",
            ));
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        self.trust_evidence.validate()?;
        if self.schema_version != 1
            || Uuid::parse_str(&self.approval_id).is_err()
            || self.approving_authority.is_empty()
            || self.reviewed_capabilities.is_empty()
            || self.launch_profile_label != "supervised"
            || !self.launch_profile_limitation.contains("not isolated")
            || self.record_digest != sha256(&self.covered_bytes()?)
        {
            return Err(M3Error::new(
                "install_approval_invalid",
                "invalid installation approval",
            ));
        }
        Ok(())
    }
}

pub(crate) fn reviewed_capabilities(
    candidate: &CandidateRecord,
    quarantine: &Path,
) -> Result<Vec<ReviewedCapability>> {
    let mut reviewed = Vec::new();
    for capability in &candidate.capabilities {
        let path = quarantine.join(&capability.manifest_path);
        reject_reparse(&path)?;
        let bytes = fs::read(&path)
            .map_err(|error| M3Error::new("install_review_io", error.to_string()))?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| M3Error::new("install_review_invalid", "manifest is not UTF-8"))?;
        let verified = manifest::verify_manifest(text)
            .map_err(|error| M3Error::new("install_review_invalid", error.message))?;
        if verified.verified_digest() != capability.manifest_digest
            || verified.capability_name() != capability.name
            || verified.capability_version() != capability.version
        {
            return Err(M3Error::new(
                "install_review_invalid",
                "manifest evidence drifted",
            ));
        }
        let value: Value = strict_json(&bytes)?;
        let permission_scope = value
            .get("permission_scope")
            .cloned()
            .ok_or_else(|| M3Error::new("install_review_invalid", "permission scope is absent"))?;
        reviewed.push(ReviewedCapability {
            capability_name: capability.name.clone(),
            capability_version: capability.version,
            manifest_digest: capability.manifest_digest.clone(),
            provider_operation_name: capability.operation.clone(),
            effects: verified.manifest().effects.clone(),
            permission_scope_digest: sha256(&canonical(&permission_scope)?),
            permission_scope,
        });
    }
    Ok(reviewed)
}

pub struct InstallationApprovalStore {
    root: StoreRoot,
}

impl InstallationApprovalStore {
    pub fn open(path: &Path) -> Result<Self> {
        Ok(Self {
            root: StoreRoot::open(path)?,
        })
    }

    pub fn open_existing(path: &Path) -> Result<Self> {
        Ok(Self {
            root: StoreRoot::open_existing(path)?,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn approve(
        &self,
        candidate: &CandidateRecord,
        quarantine_root: &Path,
        trust: &PackageTrustEvidence,
        publisher_trust: &PublisherTrustStore,
        developer_approvals: &DeveloperApprovalStore,
        launch: &LaunchProfileEvidence,
        conformance: &ConformanceEvidence,
        approving_authority: &str,
    ) -> Result<InstallationApprovalRecord> {
        let authority = PublisherDeveloperTrustAuthority::new(publisher_trust, developer_approvals);
        self.approve_with_authority(
            candidate,
            quarantine_root,
            trust,
            &authority,
            launch,
            conformance,
            approving_authority,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn approve_with_authority(
        &self,
        candidate: &CandidateRecord,
        quarantine_root: &Path,
        trust: &PackageTrustEvidence,
        authority: &dyn CurrentTrustAuthority,
        launch: &LaunchProfileEvidence,
        conformance: &ConformanceEvidence,
        approving_authority: &str,
    ) -> Result<InstallationApprovalRecord> {
        let quarantine = revalidate_candidate(candidate, quarantine_root)?;
        trust.require_for_candidate(candidate)?;
        launch.require_for_candidate(candidate)?;
        authority.revalidate_current(candidate, trust, unix_ms()?)?;
        conformance.require_current(candidate, trust, launch, &current_suite_digest()?)?;
        if conformance.disposition != ConformanceDisposition::Passed {
            return Err(M3Error::new(
                "install_approval_refused",
                "conformance has not passed",
            ));
        }
        if self
            .load_all()?
            .iter()
            .any(|existing| existing.candidate_id == candidate.candidate_id)
        {
            return Err(M3Error::new(
                "install_approval_conflict",
                "candidate already approved",
            ));
        }
        let mut record = InstallationApprovalRecord {
            schema_version: 1,
            approval_id: Uuid::new_v4().to_string(),
            candidate_id: candidate.candidate_id.clone(),
            package_id: candidate.package_id.clone(),
            package_version: candidate.package_version.clone(),
            semantic_package_digest: candidate.semantic_package_digest.clone(),
            raw_archive_digest: candidate.raw_archive_digest.clone(),
            source_size_bytes: candidate.source_size_bytes,
            payloads: candidate.payloads.clone(),
            reviewed_capabilities: reviewed_capabilities(candidate, &quarantine)?,
            trust_evidence: trust.clone(),
            provider_id: candidate.provider_id.clone(),
            provider_version: candidate.provider_version.clone(),
            launch_path: candidate.launch_path.clone(),
            launch_arguments: candidate.launch_arguments.clone(),
            provider_working_directory: candidate.provider_working_directory.clone(),
            launch_profile_label: launch.profile_label.clone(),
            launch_profile_limitation: launch.limitation.clone(),
            launch_profile_evidence_digest: launch.profile_evidence_digest.clone(),
            conformance_evidence_id: conformance.evidence_id.clone(),
            conformance_evidence_digest: conformance.evidence_digest.clone(),
            approving_authority: approving_authority.into(),
            approved_unix_ms: unix_ms()?,
            record_digest: String::new(),
        };
        record.record_digest = sha256(&record.covered_bytes()?);
        record.validate()?;
        self.root.create_json(&record.approval_id, &record)?;
        Ok(record)
    }

    pub fn load_all(&self) -> Result<Vec<InstallationApprovalRecord>> {
        let mut records = Vec::new();
        let mut identities = BTreeSet::new();
        for path in self.root.entries()? {
            if path.extension().and_then(|value| value.to_str()) == Some("tmp") {
                return Err(M3Error::new(
                    "install_approval_invalid",
                    "torn approval record",
                ));
            }
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                return Err(M3Error::new(
                    "install_approval_invalid",
                    "unexpected approval entry",
                ));
            }
            let record: InstallationApprovalRecord = self.root.read(&path)?;
            record.validate()?;
            if path.file_stem().and_then(|value| value.to_str()) != Some(&record.approval_id)
                || !identities.insert(record.approval_id.clone())
            {
                return Err(M3Error::new(
                    "install_approval_invalid",
                    "duplicate or mismatched approval identity",
                ));
            }
            records.push(record);
        }
        Ok(records)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DisabledBindingRecord {
    pub state: String,
    pub capability_name: String,
    pub capability_version: u32,
    pub manifest_digest: String,
    pub provider_operation_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InstalledPlugRecord {
    pub schema_version: u32,
    pub installed_id: String,
    pub state: String,
    pub package_id: String,
    pub package_version: String,
    pub semantic_package_digest: String,
    pub source_candidate_id: String,
    pub installation_relative_path: String,
    pub raw_archive_digest: String,
    pub plug_json: PayloadEvidence,
    pub payloads: Vec<PayloadEvidence>,
    pub signature_files: Vec<PayloadEvidence>,
    pub capability_manifests: Vec<CapabilityEvidence>,
    pub trust_evidence: PackageTrustEvidence,
    pub installation_approval_id: String,
    pub installation_approval_digest: String,
    pub conformance_evidence_id: String,
    pub conformance_evidence_digest: String,
    pub provider_id: String,
    pub provider_version: String,
    pub launch_path: String,
    pub launch_arguments: Vec<String>,
    pub provider_working_directory: String,
    pub launch_profile_label: String,
    pub socket_major: u32,
    pub mcp_protocol_version: String,
    pub platform: String,
    pub architecture: String,
    pub disabled_bindings: Vec<DisabledBindingRecord>,
    pub created_unix_ms: u64,
    pub record_digest: String,
}

impl InstalledPlugRecord {
    fn covered_bytes(&self) -> Result<Vec<u8>> {
        let mut copy = self.clone();
        copy.record_digest.clear();
        canonical(&copy)
    }

    pub(crate) fn require_for_recovery(
        &self,
        intent: &InstallationPublicationIntent,
        candidate: &CandidateRecord,
        trust: &PackageTrustEvidence,
        launch: &LaunchProfileEvidence,
        conformance: &ConformanceEvidence,
        approval: &InstallationApprovalRecord,
    ) -> Result<()> {
        self.validate()?;
        if self.installed_id != intent.transaction_id
            || self.source_candidate_id != intent.candidate_id
            || self.source_candidate_id != candidate.candidate_id
            || self.installation_relative_path != intent.destination_relative_path
        {
            return Err(M3Error::new(
                "installed_record_invalid",
                "recovery identity mismatch",
            ));
        }
        if self.package_id != candidate.package_id
            || self.package_version != candidate.package_version
            || self.semantic_package_digest != candidate.semantic_package_digest
            || self.raw_archive_digest != candidate.raw_archive_digest
            || self.plug_json != candidate.plug_json
            || self.payloads != candidate.payloads
            || self.signature_files != candidate.signature_files
            || self.capability_manifests != candidate.capabilities
            || self.trust_evidence != *trust
            || self.installation_approval_id != approval.approval_id
            || self.installation_approval_digest != approval.record_digest
            || self.conformance_evidence_id != conformance.evidence_id
            || self.conformance_evidence_digest != conformance.evidence_digest
            || self.provider_id != candidate.provider_id
            || self.provider_version != candidate.provider_version
            || self.launch_path != candidate.launch_path
            || self.launch_arguments != candidate.launch_arguments
            || self.provider_working_directory != candidate.provider_working_directory
            || self.launch_profile_label != launch.profile_label
            || self.platform != candidate.selected_platform.os
            || self.architecture != candidate.selected_platform.architecture
        {
            return Err(M3Error::new(
                "installed_record_invalid",
                "recovery chain mismatch",
            ));
        }
        let expected_bindings: Vec<DisabledBindingRecord> = candidate
            .capabilities
            .iter()
            .map(|capability| DisabledBindingRecord {
                state: "disabled".into(),
                capability_name: capability.name.clone(),
                capability_version: capability.version,
                manifest_digest: capability.manifest_digest.clone(),
                provider_operation_name: capability.operation.clone(),
            })
            .collect();
        if self.disabled_bindings != expected_bindings {
            return Err(M3Error::new(
                "installed_record_invalid",
                "disabled bindings mismatch",
            ));
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        self.trust_evidence.validate()?;
        if self.schema_version != 1
            || Uuid::parse_str(&self.installed_id).is_err()
            || self.state != "present_disabled"
            || self.socket_major != 1
            || self.mcp_protocol_version != "2025-11-25"
            || self.platform != "windows"
            || self.architecture != "x86_64"
            || self.launch_profile_label != "supervised"
            || self.plug_json.path != "plug.json"
            || self.disabled_bindings.is_empty()
            || self
                .disabled_bindings
                .iter()
                .any(|binding| binding.state != "disabled")
            || self.record_digest != sha256(&self.covered_bytes()?)
        {
            return Err(M3Error::new(
                "installed_record_invalid",
                "invalid installed Plug record",
            ));
        }
        Ok(())
    }

    /// M3 records intentionally have no active resolver representation.
    pub fn active_binding_count(&self) -> usize {
        0
    }
}

#[cfg(windows)]
fn wide(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().chain(Some(0)).collect()
}

fn mark_read_only(path: &Path) -> Result<()> {
    let mut permissions = fs::metadata(path)
        .map_err(|error| M3Error::new("install_io", error.to_string()))?
        .permissions();
    permissions.set_readonly(true);
    fs::set_permissions(path, permissions)
        .map_err(|error| M3Error::new("install_io", error.to_string()))?;
    #[cfg(windows)]
    {
        let path_w = wide(path);
        // SAFETY: path_w is a live nul-terminated UTF-16 buffer for both calls.
        let attributes = unsafe { GetFileAttributesW(path_w.as_ptr()) };
        if attributes == INVALID_FILE_ATTRIBUTES
            || unsafe { SetFileAttributesW(path_w.as_ptr(), attributes | FILE_ATTRIBUTE_READONLY) }
                == 0
        {
            return Err(M3Error::new(
                "install_io",
                "failed to set read-only attributes",
            ));
        }
    }
    if !fs::metadata(path)
        .map_err(|error| M3Error::new("install_io", error.to_string()))?
        .permissions()
        .readonly()
    {
        return Err(M3Error::new(
            "install_io",
            "installed payload remained writable",
        ));
    }
    Ok(())
}

fn expected_files(candidate: &CandidateRecord) -> BTreeMap<String, PayloadEvidence> {
    std::iter::once(&candidate.plug_json)
        .chain(candidate.payloads.iter())
        .chain(candidate.signature_files.iter())
        .map(|evidence| (evidence.path.clone(), evidence.clone()))
        .collect()
}

fn copy_files(
    source: &Path,
    staging: &Path,
    expected: &BTreeMap<String, PayloadEvidence>,
) -> Result<()> {
    for (relative, evidence) in expected {
        let source_path = source.join(relative);
        reject_reparse(&source_path)?;
        let destination = staging.join(relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| M3Error::new("install_io", error.to_string()))?;
            verify_chain(parent)?;
        }
        let bytes = fs::read(&source_path)
            .map_err(|error| M3Error::new("install_io", error.to_string()))?;
        if bytes.len() as u64 != evidence.size_bytes || sha256(&bytes) != evidence.sha256 {
            return Err(M3Error::new(
                "install_drift",
                "source payload changed during install",
            ));
        }
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&destination)
            .map_err(|error| M3Error::new("install_io", error.to_string()))?;
        output
            .write_all(&bytes)
            .map_err(|error| M3Error::new("install_io", error.to_string()))?;
        output
            .sync_all()
            .map_err(|error| M3Error::new("install_io", error.to_string()))?;
        mark_read_only(&destination)?;
    }
    Ok(())
}

fn collect_installed_files(
    root: &Path,
    directory: &Path,
    files: &mut BTreeSet<String>,
) -> Result<()> {
    for entry in fs::read_dir(directory)
        .map_err(|error| M3Error::new("installed_record_invalid", error.to_string()))?
    {
        let entry =
            entry.map_err(|error| M3Error::new("installed_record_invalid", error.to_string()))?;
        let path = entry.path();
        reject_reparse(&path)?;
        let kind = entry
            .file_type()
            .map_err(|error| M3Error::new("installed_record_invalid", error.to_string()))?;
        if kind.is_dir() {
            collect_installed_files(root, &path, files)?;
        } else if kind.is_file() {
            files.insert(
                path.strip_prefix(root)
                    .map_err(|_| {
                        M3Error::new("installed_record_invalid", "installed path escaped")
                    })?
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        } else {
            return Err(M3Error::new(
                "installed_record_invalid",
                "non-ordinary installed entry",
            ));
        }
    }
    Ok(())
}

pub struct InstalledPlugRegistry {
    install_root: StoreRoot,
    record_root: StoreRoot,
}

impl InstalledPlugRegistry {
    pub fn open(install_root: &Path, record_root: &Path) -> Result<Self> {
        let install_root = StoreRoot::open(install_root)?;
        let record_root = StoreRoot::open(record_root)?;
        if install_root.path() == record_root.path() {
            return Err(M3Error::new(
                "installed_store_invalid",
                "install and record roots must differ",
            ));
        }
        Ok(Self {
            install_root,
            record_root,
        })
    }

    pub fn open_existing(install_root: &Path, record_root: &Path) -> Result<Self> {
        let install_root = StoreRoot::open_existing(install_root)?;
        let record_root = StoreRoot::open_existing(record_root)?;
        if install_root.path() == record_root.path() {
            return Err(M3Error::new(
                "installed_store_invalid",
                "install and record roots must differ",
            ));
        }
        Ok(Self {
            install_root,
            record_root,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn install_disabled(
        &self,
        candidate: &CandidateRecord,
        quarantine_root: &Path,
        trust: &PackageTrustEvidence,
        publisher_trust: &PublisherTrustStore,
        developer_approvals: &DeveloperApprovalStore,
        launch: &LaunchProfileEvidence,
        conformance: &ConformanceEvidence,
        approval: &InstallationApprovalRecord,
    ) -> Result<InstalledPlugRecord> {
        let authority = PublisherDeveloperTrustAuthority::new(publisher_trust, developer_approvals);
        self.install_disabled_with_authority(
            candidate,
            quarantine_root,
            trust,
            &authority,
            launch,
            conformance,
            approval,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn install_disabled_with_authority(
        &self,
        candidate: &CandidateRecord,
        quarantine_root: &Path,
        trust: &PackageTrustEvidence,
        authority: &dyn CurrentTrustAuthority,
        launch: &LaunchProfileEvidence,
        conformance: &ConformanceEvidence,
        approval: &InstallationApprovalRecord,
    ) -> Result<InstalledPlugRecord> {
        let source = revalidate_candidate(candidate, quarantine_root)?;
        trust.require_for_candidate(candidate)?;
        launch.require_for_candidate(candidate)?;
        authority.revalidate_current(candidate, trust, unix_ms()?)?;
        conformance.require_current(candidate, trust, launch, &current_suite_digest()?)?;
        approval.validate()?;
        if approval.candidate_id != candidate.candidate_id
            || approval.semantic_package_digest != candidate.semantic_package_digest
            || approval.trust_evidence.evidence_digest != trust.evidence_digest
            || approval.launch_profile_evidence_digest != launch.profile_evidence_digest
            || approval.conformance_evidence_digest != conformance.evidence_digest
        {
            return Err(M3Error::new(
                "install_approval_stale",
                "approval pins drifted",
            ));
        }
        for existing in self.load_all()? {
            if existing.package_id == candidate.package_id
                && existing.package_version == candidate.package_version
            {
                return Err(M3Error::new(
                    "installed_conflict",
                    "package release already installed",
                ));
            }
        }
        let installed_id = Uuid::new_v4().to_string();
        let staging = self
            .install_root
            .path()
            .join(format!(".staging-{installed_id}"));
        let destination = self
            .install_root
            .path()
            .join(format!("plug-{installed_id}"));
        fs::create_dir(&staging).map_err(|error| M3Error::new("install_io", error.to_string()))?;
        verify_chain(&staging)?;
        let result = copy_files(&source, &staging, &expected_files(candidate));
        if let Err(error) = result {
            let _ = fs::remove_dir_all(&staging);
            return Err(error);
        }
        let final_revalidation = (|| -> Result<()> {
            revalidate_candidate(candidate, quarantine_root)?;
            trust.require_for_candidate(candidate)?;
            launch.require_for_candidate(candidate)?;
            authority.revalidate_current(candidate, trust, unix_ms()?)?;
            conformance.require_current(candidate, trust, launch, &current_suite_digest()?)?;
            Ok(())
        })();
        if let Err(error) = final_revalidation {
            let _ = fs::remove_dir_all(&staging);
            return Err(error);
        }
        verify_chain(self.install_root.path())?;
        if destination.exists() {
            let _ = fs::remove_dir_all(&staging);
            return Err(M3Error::new(
                "installed_conflict",
                "installation target exists",
            ));
        }
        fs::rename(&staging, &destination)
            .map_err(|error| M3Error::new("install_io", error.to_string()))?;
        let relative = destination
            .strip_prefix(self.install_root.path())
            .map_err(|_| M3Error::new("installed_store_invalid", "installation escaped root"))?
            .to_string_lossy()
            .replace('\\', "/");
        let disabled_bindings = candidate
            .capabilities
            .iter()
            .map(|capability| DisabledBindingRecord {
                state: "disabled".into(),
                capability_name: capability.name.clone(),
                capability_version: capability.version,
                manifest_digest: capability.manifest_digest.clone(),
                provider_operation_name: capability.operation.clone(),
            })
            .collect();
        let mut record = InstalledPlugRecord {
            schema_version: 1,
            installed_id,
            state: "present_disabled".into(),
            package_id: candidate.package_id.clone(),
            package_version: candidate.package_version.clone(),
            semantic_package_digest: candidate.semantic_package_digest.clone(),
            source_candidate_id: candidate.candidate_id.clone(),
            installation_relative_path: relative,
            raw_archive_digest: candidate.raw_archive_digest.clone(),
            plug_json: candidate.plug_json.clone(),
            payloads: candidate.payloads.clone(),
            signature_files: candidate.signature_files.clone(),
            capability_manifests: candidate.capabilities.clone(),
            trust_evidence: trust.clone(),
            installation_approval_id: approval.approval_id.clone(),
            installation_approval_digest: approval.record_digest.clone(),
            conformance_evidence_id: conformance.evidence_id.clone(),
            conformance_evidence_digest: conformance.evidence_digest.clone(),
            provider_id: candidate.provider_id.clone(),
            provider_version: candidate.provider_version.clone(),
            launch_path: candidate.launch_path.clone(),
            launch_arguments: candidate.launch_arguments.clone(),
            provider_working_directory: candidate.provider_working_directory.clone(),
            launch_profile_label: launch.profile_label.clone(),
            socket_major: 1,
            mcp_protocol_version: "2025-11-25".into(),
            platform: candidate.selected_platform.os.clone(),
            architecture: candidate.selected_platform.architecture.clone(),
            disabled_bindings,
            created_unix_ms: unix_ms()?,
            record_digest: String::new(),
        };
        record.record_digest = sha256(&record.covered_bytes()?);
        record.validate()?;
        self.record_root
            .create_json(&record.installed_id, &record)?;
        Ok(record)
    }

    pub fn load_all(&self) -> Result<Vec<InstalledPlugRecord>> {
        let mut records = Vec::new();
        let mut identities = BTreeSet::new();
        let mut releases = BTreeMap::new();
        for path in self.record_root.entries()? {
            if path.extension().and_then(|value| value.to_str()) == Some("tmp") {
                return Err(M3Error::new(
                    "installed_record_invalid",
                    "torn installed record",
                ));
            }
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                return Err(M3Error::new(
                    "installed_record_invalid",
                    "unexpected registry entry",
                ));
            }
            let record: InstalledPlugRecord = self.record_root.read(&path)?;
            record.validate()?;
            if path.file_stem().and_then(|value| value.to_str()) != Some(&record.installed_id)
                || !identities.insert(record.installed_id.clone())
            {
                return Err(M3Error::new(
                    "installed_record_invalid",
                    "duplicate or mismatched installed identity",
                ));
            }
            let release = (record.package_id.clone(), record.package_version.clone());
            if releases
                .insert(release, record.semantic_package_digest.clone())
                .is_some()
            {
                return Err(M3Error::new(
                    "installed_conflict",
                    "duplicate installed release",
                ));
            }
            let directory = self
                .install_root
                .path()
                .join(&record.installation_relative_path);
            verify_chain(&directory)?;
            let candidate_files = std::iter::once(&record.plug_json)
                .chain(record.payloads.iter())
                .chain(record.signature_files.iter())
                .map(|payload| (payload.path.clone(), payload.clone()))
                .collect::<BTreeMap<_, _>>();
            let mut actual = BTreeSet::new();
            collect_installed_files(&directory, &directory, &mut actual)?;
            if actual != candidate_files.keys().cloned().collect() {
                return Err(M3Error::new(
                    "installed_record_invalid",
                    "installed exact file set drifted",
                ));
            }
            for (relative, evidence) in candidate_files {
                let file = directory.join(relative);
                reject_reparse(&file)?;
                let metadata = fs::metadata(&file)
                    .map_err(|error| M3Error::new("installed_record_invalid", error.to_string()))?;
                let bytes = fs::read(&file)
                    .map_err(|error| M3Error::new("installed_record_invalid", error.to_string()))?;
                if !metadata.permissions().readonly()
                    || bytes.len() as u64 != evidence.size_bytes
                    || sha256(&bytes) != evidence.sha256
                {
                    return Err(M3Error::new(
                        "installed_record_invalid",
                        "installed payload drifted",
                    ));
                }
            }
            records.push(record);
        }
        Ok(records)
    }

    pub fn installation_directory(
        &self,
        record: &InstalledPlugRecord,
    ) -> Result<std::path::PathBuf> {
        record.validate()?;
        let directory = self
            .install_root
            .path()
            .join(&record.installation_relative_path);
        verify_chain(&directory)?;
        let directory = fs::canonicalize(&directory)
            .map_err(|error| M3Error::new("installed_record_invalid", error.to_string()))?;
        if !directory.starts_with(self.install_root.path()) {
            return Err(M3Error::new(
                "installed_record_invalid",
                "installation escaped root",
            ));
        }
        Ok(directory)
    }

    pub(crate) fn observe_installation_recovery(
        &self,
        intent: &InstallationPublicationIntent,
    ) -> Result<InstallationRecoverySnapshot> {
        intent.validate().map_err(|_| intent_invalid())?;

        let install_root = self.install_root.path();
        let record_root = self.record_root.path();

        require_existing_recovery_root(install_root)?;
        require_existing_recovery_root(record_root)?;

        let staging_path = install_root.join(format!(".staging-{}", intent.transaction_id));
        let destination_path = install_root.join(&intent.destination_relative_path);
        let record_path = record_root.join(format!("{}.json", intent.transaction_id));

        let staging_present = observe_directory(&staging_path)?;
        let destination_present = observe_directory(&destination_path)?;
        let installed_record = observe_record(&record_path)?;

        Ok(InstallationRecoverySnapshot {
            staging_present,
            destination_present,
            installed_record,
        })
    }

    pub(crate) fn verify_installation_recovery_destination(
        &self,
        intent: &InstallationPublicationIntent,
    ) -> Result<()> {
        intent.validate().map_err(|_| intent_invalid())?;

        let install_root = self.install_root.path();
        require_existing_recovery_root(install_root)?;

        let destination = install_root.join(&intent.destination_relative_path);
        require_existing_recovery_destination(&destination)?;

        let expected = recovery_expected_files(intent)?;
        let mut actual = BTreeSet::new();
        collect_recovery_files(&destination, &destination, &mut actual)?;

        if actual != expected.keys().cloned().collect::<BTreeSet<_>>() {
            return Err(recovery_conflict());
        }

        for (relative, evidence) in expected {
            let file = destination.join(&relative);
            reject_reparse(&file).map_err(map_recovery_path_error)?;
            let metadata = fs::symlink_metadata(&file).map_err(|_| recovery_io())?;
            if !metadata.is_file() {
                return Err(recovery_conflict());
            }
            let bytes = fs::read(&file).map_err(|_| recovery_io())?;
            if !metadata.permissions().readonly()
                || bytes.len() as u64 != evidence.size_bytes
                || sha256(&bytes) != evidence.sha256
            {
                return Err(recovery_conflict());
            }
        }

        Ok(())
    }

    pub(crate) fn audit_installation_recovery_destinations(
        &self,
        intent: Option<&InstallationPublicationIntent>,
    ) -> Result<()> {
        if let Some(intent) = intent {
            intent.validate().map_err(|_| intent_invalid())?;
        }

        let install_path = self.install_root.path();
        let record_path = self.record_root.path();
        require_existing_recovery_root(install_path)?;
        require_existing_recovery_root(record_path)?;

        let records = self.load_all().map_err(|_| recovery_conflict())?;

        let mut destination_set = BTreeSet::new();
        for record in &records {
            let uuid = Uuid::parse_str(&record.installed_id).map_err(|_| recovery_conflict())?;
            if uuid.to_string() != record.installed_id {
                return Err(recovery_conflict());
            }
            let expected_dest = format!("plug-{}", record.installed_id);
            if record.installation_relative_path != expected_dest {
                return Err(recovery_conflict());
            }
            if !destination_set.insert(&record.installation_relative_path) {
                return Err(recovery_conflict());
            }
        }

        if let Some(intent) = intent {
            for record in &records {
                if record.installation_relative_path == intent.destination_relative_path {
                    if *record != intent.installed_record {
                        return Err(recovery_conflict());
                    }
                }
            }
        }

        let entries = fs::read_dir(install_path).map_err(|_| recovery_io())?;
        for entry_result in entries {
            let entry = entry_result.map_err(|_| recovery_io())?;
            let entry_path = entry.path();
            let file_name = entry.file_name();
            let name_str = match file_name.to_str() {
                Some(s) => s,
                None => return Err(recovery_conflict()),
            };

            if !name_str.starts_with("plug-") {
                continue;
            }

            reject_reparse(&entry_path).map_err(|e| {
                if e.code == "unsafe_store_path" {
                    e
                } else {
                    recovery_io()
                }
            })?;

            let uuid_str = &name_str[5..];
            let uuid = Uuid::parse_str(uuid_str).map_err(|_| destination_untracked())?;
            if uuid.to_string() != uuid_str {
                return Err(destination_untracked());
            }

            let accounted = records
                .iter()
                .any(|r| r.installation_relative_path == name_str)
                || intent.map_or(false, |i| i.destination_relative_path == name_str);

            if !accounted {
                return Err(destination_untracked());
            }

            let metadata = fs::symlink_metadata(&entry_path).map_err(|_| recovery_io())?;
            if !metadata.is_dir() {
                return Err(recovery_conflict());
            }
        }

        Ok(())
    }
}

fn intent_invalid() -> M3Error {
    M3Error::new(
        "installation_intent_invalid",
        "installation publication intent is invalid",
    )
}

fn destination_untracked() -> M3Error {
    M3Error::new(
        "installation_destination_untracked",
        "installed destination is not tracked by a validated record or current publication intent",
    )
}

fn recovery_conflict() -> M3Error {
    M3Error::new(
        "installation_recovery_conflict",
        "installation recovery state conflicts with publication intent",
    )
}

fn recovery_io() -> M3Error {
    M3Error::new(
        "installation_recovery_io",
        "installation recovery state could not be observed",
    )
}

fn map_recovery_path_error(error: M3Error) -> M3Error {
    if error.code == "unsafe_store_path" {
        error
    } else {
        recovery_io()
    }
}

fn require_existing_recovery_root(path: &Path) -> Result<()> {
    verify_chain(path).map_err(map_recovery_path_error)?;
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if !metadata.is_dir() {
                return Err(recovery_io());
            }
            reject_reparse(path).map_err(map_recovery_path_error)
        }
        Err(_) => Err(recovery_io()),
    }
}

fn require_existing_recovery_destination(path: &Path) -> Result<()> {
    verify_chain(path).map_err(map_recovery_path_error)?;
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if !metadata.is_dir() {
                return Err(recovery_conflict());
            }
            reject_reparse(path).map_err(map_recovery_path_error)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(recovery_conflict()),
        Err(_) => Err(recovery_io()),
    }
}

fn recovery_expected_files(
    intent: &InstallationPublicationIntent,
) -> Result<BTreeMap<String, PayloadEvidence>> {
    let mut expected = BTreeMap::new();
    for evidence in std::iter::once(&intent.installed_record.plug_json)
        .chain(intent.installed_record.payloads.iter())
        .chain(intent.installed_record.signature_files.iter())
    {
        let normalized = recovery_expected_path(&evidence.path)?;
        if expected.insert(normalized, evidence.clone()).is_some() {
            return Err(recovery_conflict());
        }
    }
    Ok(expected)
}

fn recovery_expected_path(path: &str) -> Result<String> {
    if path.is_empty()
        || path.starts_with('/')
        || path.starts_with('\\')
        || path.ends_with('/')
        || path.ends_with('\\')
        || path.contains("\\")
        || path.contains("//")
    {
        return Err(recovery_conflict());
    }
    let parsed = Path::new(path);
    if parsed.is_absolute() {
        return Err(recovery_conflict());
    }
    for component in parsed.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(recovery_conflict());
        }
    }
    Ok(path.replace('\\', "/"))
}

fn collect_recovery_files(
    root: &Path,
    directory: &Path,
    files: &mut BTreeSet<String>,
) -> Result<()> {
    for entry in fs::read_dir(directory).map_err(|_| recovery_io())? {
        let entry = entry.map_err(|_| recovery_io())?;
        let path = entry.path();
        reject_reparse(&path).map_err(map_recovery_path_error)?;
        let kind = entry.file_type().map_err(|_| recovery_io())?;
        if kind.is_dir() {
            collect_recovery_files(root, &path, files)?;
        } else if kind.is_file() {
            files.insert(
                path.strip_prefix(root)
                    .map_err(|_| recovery_io())?
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        } else {
            return Err(recovery_conflict());
        }
    }
    Ok(())
}

fn observe_directory(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(meta) => {
            match reject_reparse(path) {
                Ok(()) => {}
                Err(e) if e.code == "unsafe_store_path" => return Err(e),
                Err(_) => return Err(recovery_io()),
            }
            if !meta.is_dir() {
                return Err(recovery_conflict());
            }
            Ok(true)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err(recovery_io()),
    }
}

fn observe_record(path: &Path) -> Result<Option<InstalledPlugRecord>> {
    match fs::symlink_metadata(path) {
        Ok(meta) => {
            match reject_reparse(path) {
                Ok(()) => {}
                Err(e) if e.code == "unsafe_store_path" => return Err(e),
                Err(_) => return Err(recovery_io()),
            }
            if !meta.is_file() {
                return Err(recovery_conflict());
            }
            let bytes = fs::read(path).map_err(|_| recovery_io())?;
            let record: InstalledPlugRecord =
                strict_json(&bytes).map_err(|_| recovery_conflict())?;
            Ok(Some(record))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(recovery_io()),
    }
}
