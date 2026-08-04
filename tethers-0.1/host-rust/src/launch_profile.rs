//! Exact M3 quarantine test-launch preparation.
//!
//! The `supervised` profile is process ownership and launch hygiene. It is
//! deliberately and permanently represented as not isolated.

use crate::candidate::CandidateRecord;
use crate::child_process::{ChildConfig, ChildError, SupervisedChild};
use crate::conformance::{ConformanceDisposition, ConformanceEvidence};
use crate::current_trust::{CurrentTrustAuthority, PublisherDeveloperTrustAuthority};
use crate::enablement::EnablementRecord;
use crate::installed::{InstallationApprovalRecord, InstalledPlugRecord};
use crate::m3_store::{
    canonical, reject_reparse, sha256, verify_chain, M3Error, Result, StoreRoot,
};
use crate::operational_scope::OperationalScope;
use crate::package::PayloadEvidence;
use crate::trust::{DeveloperApprovalStore, PackageTrustEvidence, PublisherTrustStore};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

pub const SUPERVISED_PROFILE_LABEL: &str = "supervised";
pub const SUPERVISED_PROFILE_LIMITATION: &str =
    "process supervision only; not isolated or hostile-code-safe";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LaunchProfileEvidence {
    pub profile_format_version: u32,
    pub profile_label: String,
    pub isolated: bool,
    pub limitation: String,
    pub candidate_id: String,
    pub semantic_package_digest: String,
    pub executable_digest: String,
    pub executable_relative_path: String,
    pub arguments: Vec<String>,
    pub working_directory_relative_path: String,
    pub environment_names: Vec<String>,
    pub environment_digest: String,
    pub max_processes: u32,
    pub process_memory_limit_bytes: u64,
    pub protocol_line_limit_bytes: u64,
    pub stderr_tail_limit_bytes: u64,
    pub wall_time_limit_ms: u64,
    pub profile_evidence_digest: String,
}

impl LaunchProfileEvidence {
    fn covered_bytes(&self) -> Result<Vec<u8>> {
        let mut copy = self.clone();
        copy.profile_evidence_digest.clear();
        canonical(&copy)
    }

    pub fn validate(&self) -> Result<()> {
        if self.profile_format_version != 1
            || self.profile_label != SUPERVISED_PROFILE_LABEL
            || self.isolated
            || self.limitation != SUPERVISED_PROFILE_LIMITATION
            || self.profile_evidence_digest != sha256(&self.covered_bytes()?)
        {
            return Err(M3Error::new(
                "launch_profile_invalid",
                "invalid supervised profile evidence",
            ));
        }
        Ok(())
    }

    pub fn require_for_candidate(&self, candidate: &CandidateRecord) -> Result<()> {
        self.validate()?;
        let executable = candidate
            .payloads
            .iter()
            .find(|payload| payload.path == candidate.launch_path)
            .ok_or_else(|| M3Error::new("launch_candidate_mismatch", "launch payload is absent"))?;
        if self.candidate_id != candidate.candidate_id
            || self.semantic_package_digest != candidate.semantic_package_digest
            || self.executable_relative_path != candidate.launch_path
            || self.executable_digest != executable.sha256
            || self.arguments != candidate.launch_arguments
            || self.working_directory_relative_path != candidate.provider_working_directory
            || self.profile_label != SUPERVISED_PROFILE_LABEL
            || self.isolated
            || self.limitation != SUPERVISED_PROFILE_LIMITATION
        {
            return Err(M3Error::new(
                "launch_candidate_mismatch",
                "launch evidence is not bound to this exact candidate",
            ));
        }
        Ok(())
    }
}

pub struct LaunchProfileEvidenceStore {
    root: StoreRoot,
}

impl LaunchProfileEvidenceStore {
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

    fn record_identity(evidence: &LaunchProfileEvidence) -> Result<String> {
        let digest = &evidence.profile_evidence_digest;
        let suffix = digest.strip_prefix("sha256:").ok_or_else(|| {
            M3Error::new(
                "launch_profile_store_invalid",
                "invalid profile evidence digest",
            )
        })?;
        if suffix.len() != 64
            || !suffix
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            return Err(M3Error::new(
                "launch_profile_store_invalid",
                "invalid profile evidence digest",
            ));
        }
        Ok(suffix.to_owned())
    }

    pub fn create(&self, evidence: &LaunchProfileEvidence) -> Result<()> {
        evidence.validate()?;
        let id = Self::record_identity(evidence)?;
        self.root.create_json(&id, evidence)?;
        Ok(())
    }

    pub fn load_all(&self) -> Result<Vec<LaunchProfileEvidence>> {
        let mut records = Vec::new();
        let mut seen = BTreeSet::new();
        for path in self.root.entries()? {
            if path.extension().and_then(|value| value.to_str()) == Some("tmp") {
                return Err(M3Error::new(
                    "launch_profile_store_invalid",
                    "torn launch-profile evidence",
                ));
            }
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                return Err(M3Error::new(
                    "launch_profile_store_invalid",
                    "unexpected launch-profile store entry",
                ));
            }
            let record: LaunchProfileEvidence = self.root.read(&path)?;
            record.validate()?;
            let id = Self::record_identity(&record)?;
            if path.file_stem().and_then(|value| value.to_str()) != Some(&id) {
                return Err(M3Error::new(
                    "launch_profile_store_invalid",
                    "launch-profile filename mismatch",
                ));
            }
            if !seen.insert(record.profile_evidence_digest.clone()) {
                return Err(M3Error::new(
                    "launch_profile_store_invalid",
                    "duplicate launch-profile evidence",
                ));
            }
            records.push(record);
        }
        records.sort_by(|a, b| a.profile_evidence_digest.cmp(&b.profile_evidence_digest));
        Ok(records)
    }
}

pub struct PreparedSupervisedLaunch {
    pub evidence: LaunchProfileEvidence,
    executable: PathBuf,
    working_directory: PathBuf,
    environment: BTreeMap<String, String>,
    scratch_directory: PathBuf,
}

/// Launch an already-installed provider. Package launch arguments are treated
/// as a declaration: only the reviewed scope placeholders may be materialised
/// from the current host-owned operational binding.
pub fn launch_installed_provider(
    record: &InstalledPlugRecord,
    installed_directory: &Path,
    trust: &PackageTrustEvidence,
    publisher_trust: &PublisherTrustStore,
    developer_approvals: &DeveloperApprovalStore,
    conformance: &ConformanceEvidence,
    approval: &InstallationApprovalRecord,
    enablement: &EnablementRecord,
    scope: &OperationalScope,
) -> std::result::Result<SupervisedChild, ChildError> {
    record.validate().map_err(map_installed_error)?;
    trust
        .revalidate_current(
            &record.package_id,
            publisher_trust,
            developer_approvals,
            crate::m3_store::unix_ms().map_err(map_installed_error)?,
        )
        .map_err(map_installed_error)?;
    conformance.validate().map_err(map_installed_error)?;
    if conformance.disposition != ConformanceDisposition::Passed
        || conformance.evidence_digest != record.conformance_evidence_digest
        || conformance.semantic_package_digest != record.semantic_package_digest
        || conformance.capabilities != record.capability_manifests
    {
        return Err(ChildError::LaunchFailed {
            command: record.launch_path.clone(),
            message: "current conformance evidence is not pinned to installed state".into(),
        });
    }
    approval.validate().map_err(map_installed_error)?;
    if approval.candidate_id.is_empty()
        || approval.package_id != record.package_id
        || approval.semantic_package_digest != record.semantic_package_digest
        || approval.conformance_evidence_digest != conformance.evidence_digest
        || approval.trust_evidence.evidence_digest != trust.evidence_digest
    {
        return Err(ChildError::LaunchFailed {
            command: record.launch_path.clone(),
            message: "installation approval pins are stale".into(),
        });
    }
    if enablement.installed_id != record.installed_id
        || enablement.state != crate::enablement::EnablementState::Enabled
        || enablement.semantic_package_digest != record.semantic_package_digest
        || enablement.conformance_evidence_digest != conformance.evidence_digest
        || enablement.operational_scope_digest != scope.integrity_digest()
    {
        return Err(ChildError::LaunchFailed {
            command: record.launch_path.clone(),
            message: "enablement pins are stale".into(),
        });
    }
    enablement.validate().map_err(map_installed_error)?;
    if enablement.operational_scope != *scope {
        return Err(ChildError::LaunchFailed {
            command: record.launch_path.clone(),
            message: "enablement scope does not match supplied scope".into(),
        });
    }
    if scope.installed_id() != record.installed_id
        || !enablement.capabilities.iter().any(|binding| {
            binding.name == scope.capability_name() && binding.version == scope.capability_version()
        })
    {
        return Err(ChildError::LaunchFailed {
            command: record.launch_path.clone(),
            message: "enabled capability binding does not match scope".into(),
        });
    }
    scope.validate().map_err(map_installed_error)?;
    if !installed_directory.is_absolute() {
        return Err(ChildError::LaunchFailed {
            command: record.launch_path.clone(),
            message: "installed directory must be absolute".into(),
        });
    }
    verify_chain(installed_directory).map_err(map_installed_error)?;
    let executable = installed_directory.join(&record.launch_path);
    let working_directory = installed_directory.join(&record.provider_working_directory);
    revalidate_installed_files(record, installed_directory).map_err(map_installed_error)?;
    let executable = fs::canonicalize(&executable).map_err(|error| ChildError::LaunchFailed {
        command: record.launch_path.clone(),
        message: error.to_string(),
    })?;
    let working_directory =
        fs::canonicalize(&working_directory).map_err(|error| ChildError::LaunchFailed {
            command: record.launch_path.clone(),
            message: error.to_string(),
        })?;
    if !working_directory.starts_with(installed_directory) || !working_directory.is_dir() {
        return Err(ChildError::LaunchFailed {
            command: record.launch_path.clone(),
            message: "installed working directory escaped".into(),
        });
    }
    let mut args = record.launch_arguments.clone();
    match scope {
        OperationalScope::FileTools(s) => {
            let replacements = [
                (
                    "__TETHERS_FILE_QUERY_ROOT__",
                    s.query_root.to_string_lossy().into_owned(),
                ),
                (
                    "__TETHERS_FILE_SOURCE_ROOT__",
                    s.move_source_root.to_string_lossy().into_owned(),
                ),
                (
                    "__TETHERS_FILE_DESTINATION_ROOT__",
                    s.move_destination_root.to_string_lossy().into_owned(),
                ),
            ];
            for arg in &mut args {
                for (placeholder, value) in &replacements {
                    if arg == placeholder {
                        *arg = value.clone();
                    }
                }
            }
        }
        OperationalScope::Pdf(s) => {
            let replacement = s.query_root.to_string_lossy().into_owned();
            for arg in &mut args {
                if arg == "__TETHERS_PDF_QUERY_ROOT__" {
                    *arg = replacement.clone();
                }
            }
        }
    }
    if args.iter().any(|arg| arg.contains("__TETHERS_")) {
        return Err(ChildError::LaunchFailed {
            command: executable.to_string_lossy().into_owned(),
            message: "unreviewed launch placeholder".into(),
        });
    }
    let system_root = std::env::var("SystemRoot").map_err(|_| ChildError::LaunchFailed {
        command: executable.to_string_lossy().into_owned(),
        message: "SystemRoot unavailable".into(),
    })?;
    let scratch = installed_directory.join(".operational-scratch");
    fs::create_dir_all(&scratch).map_err(|error| ChildError::LaunchFailed {
        command: executable.to_string_lossy().into_owned(),
        message: error.to_string(),
    })?;
    let mut environment = BTreeMap::new();
    environment.insert("SystemRoot".into(), system_root.clone());
    environment.insert("WINDIR".into(), system_root);
    environment.insert("TEMP".into(), scratch.to_string_lossy().into_owned());
    environment.insert("TMP".into(), scratch.to_string_lossy().into_owned());
    environment.insert("TETHERS_CONFORMANCE".into(), "0".into());
    match scope {
        OperationalScope::FileTools(s) => {
            environment.insert(
                "TETHERS_FILE_QUERY_ROOT".into(),
                s.query_root.to_string_lossy().into_owned(),
            );
            environment.insert(
                "TETHERS_FILE_SOURCE_ROOT".into(),
                s.move_source_root.to_string_lossy().into_owned(),
            );
            environment.insert(
                "TETHERS_FILE_DESTINATION_ROOT".into(),
                s.move_destination_root.to_string_lossy().into_owned(),
            );
        }
        OperationalScope::Pdf(s) => {
            environment.insert(
                "TETHERS_PDF_QUERY_ROOT".into(),
                s.query_root.to_string_lossy().into_owned(),
            );
            environment.insert("TETHERS_PDF_MAX_BYTES".into(), s.max_bytes.to_string());
        }
    }
    let mut config = ChildConfig::production(executable.to_string_lossy().into_owned(), args);
    config.current_dir = Some(working_directory);
    config.clear_environment = true;
    config.environment = environment;
    config.assign_before_execution = true;
    config.max_processes = 8;
    config.process_memory_limit_bytes = 256 * 1024 * 1024;
    config.max_protocol_line_bytes = 1024 * 1024;
    config.stderr_tail_bytes = 16 * 1024;
    config.graceful_close_timeout = Duration::from_secs(1);
    SupervisedChild::launch(config)
}

fn map_installed_error(error: M3Error) -> ChildError {
    ChildError::LaunchFailed {
        command: "installed-provider".into(),
        message: format!("{}: {}", error.code, error.message),
    }
}
fn revalidate_installed_files(record: &InstalledPlugRecord, directory: &Path) -> Result<()> {
    let expected = std::iter::once(&record.plug_json)
        .chain(record.payloads.iter())
        .chain(record.signature_files.iter())
        .map(|e| e.path.clone())
        .collect::<BTreeSet<_>>();
    let mut actual = BTreeSet::new();
    collect_files(directory, directory, &mut actual)?;
    if actual != expected {
        return Err(M3Error::new(
            "installed_drift",
            "installed file set changed",
        ));
    }
    for evidence in std::iter::once(&record.plug_json)
        .chain(record.payloads.iter())
        .chain(record.signature_files.iter())
    {
        let path = directory.join(&evidence.path);
        reject_reparse(&path)?;
        let bytes = fs::read(&path).map_err(|e| M3Error::new("installed_drift", e.to_string()))?;
        if sha256(&bytes) != evidence.sha256 || bytes.len() as u64 != evidence.size_bytes {
            return Err(M3Error::new(
                "installed_drift",
                "installed payload digest changed",
            ));
        }
    }
    Ok(())
}

fn map_candidate_error(error: crate::package::PackageError) -> M3Error {
    M3Error::new(
        "candidate_invalid",
        format!("{}: {}", error.code, error.message),
    )
}

fn expected_files(record: &CandidateRecord) -> Result<BTreeMap<String, PayloadEvidence>> {
    let mut expected = BTreeMap::new();
    for evidence in std::iter::once(&record.plug_json)
        .chain(record.payloads.iter())
        .chain(record.signature_files.iter())
    {
        if expected
            .insert(evidence.path.clone(), evidence.clone())
            .is_some()
        {
            return Err(M3Error::new("candidate_invalid", "duplicate file evidence"));
        }
    }
    Ok(expected)
}

fn collect_files(root: &Path, directory: &Path, output: &mut BTreeSet<String>) -> Result<()> {
    for entry in
        fs::read_dir(directory).map_err(|error| M3Error::new("launch_io", error.to_string()))?
    {
        let entry = entry.map_err(|error| M3Error::new("launch_io", error.to_string()))?;
        let path = entry.path();
        reject_reparse(&path)?;
        let kind = entry
            .file_type()
            .map_err(|error| M3Error::new("launch_io", error.to_string()))?;
        if kind.is_dir() {
            collect_files(root, &path, output)?;
        } else if kind.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| M3Error::new("candidate_drift", "candidate path escaped"))?
                .to_string_lossy()
                .replace('\\', "/");
            output.insert(relative);
        } else {
            return Err(M3Error::new(
                "candidate_drift",
                "non-ordinary candidate file",
            ));
        }
    }
    Ok(())
}

pub fn revalidate_candidate(record: &CandidateRecord, quarantine_root: &Path) -> Result<PathBuf> {
    record.validate().map_err(map_candidate_error)?;
    verify_chain(quarantine_root)?;
    let root = fs::canonicalize(quarantine_root)
        .map_err(|error| M3Error::new("launch_io", error.to_string()))?;
    let directory = root.join(&record.quarantine_relative_path);
    verify_chain(&directory)?;
    let directory = fs::canonicalize(&directory)
        .map_err(|error| M3Error::new("candidate_drift", error.to_string()))?;
    if !directory.starts_with(&root) || !directory.is_dir() {
        return Err(M3Error::new(
            "candidate_drift",
            "candidate is outside quarantine root",
        ));
    }
    let expected = expected_files(record)?;
    let mut actual = BTreeSet::new();
    collect_files(&directory, &directory, &mut actual)?;
    if actual != expected.keys().cloned().collect() {
        return Err(M3Error::new(
            "candidate_drift",
            "candidate exact file set changed",
        ));
    }
    for (relative, evidence) in expected {
        let path = directory.join(&relative);
        reject_reparse(&path)?;
        let metadata = fs::metadata(&path)
            .map_err(|error| M3Error::new("candidate_drift", error.to_string()))?;
        let bytes =
            fs::read(&path).map_err(|error| M3Error::new("candidate_drift", error.to_string()))?;
        if !metadata.is_file()
            || !metadata.permissions().readonly()
            || bytes.len() as u64 != evidence.size_bytes
            || sha256(&bytes) != evidence.sha256
        {
            return Err(M3Error::new(
                "candidate_drift",
                "candidate file integrity changed",
            ));
        }
    }
    let descriptor = fs::read(directory.join("plug.json"))
        .map_err(|error| M3Error::new("candidate_drift", error.to_string()))?;
    if crate::package::semantic_digest_for_plug_json(&descriptor)
        .map_err(|error| M3Error::new("candidate_drift", error.message))?
        != record.semantic_package_digest
    {
        return Err(M3Error::new(
            "candidate_drift",
            "plug.json semantic evidence changed",
        ));
    }
    Ok(directory)
}

fn approved_environment(scratch: &Path) -> Result<BTreeMap<String, String>> {
    let system_root = std::env::var("SystemRoot")
        .map_err(|_| M3Error::new("launch_environment", "SystemRoot is unavailable"))?;
    let mut environment = BTreeMap::new();
    environment.insert("SystemRoot".into(), system_root.clone());
    environment.insert("WINDIR".into(), system_root);
    let scratch = scratch.to_string_lossy().into_owned();
    environment.insert("TEMP".into(), scratch.clone());
    environment.insert("TMP".into(), scratch);
    environment.insert("TETHERS_CONFORMANCE".into(), "1".into());
    Ok(environment)
}

impl PreparedSupervisedLaunch {
    pub(crate) fn revalidate_current_trust(
        &self,
        candidate: &CandidateRecord,
        trust: &PackageTrustEvidence,
        publisher_trust: &PublisherTrustStore,
        developer_approvals: &DeveloperApprovalStore,
    ) -> Result<()> {
        let authority = PublisherDeveloperTrustAuthority::new(publisher_trust, developer_approvals);
        self.revalidate_current_trust_with(candidate, trust, &authority)
    }

    pub(crate) fn revalidate_current_trust_with(
        &self,
        candidate: &CandidateRecord,
        trust: &PackageTrustEvidence,
        authority: &dyn CurrentTrustAuthority,
    ) -> Result<()> {
        self.evidence.require_for_candidate(candidate)?;
        trust.require_for_candidate(candidate)?;
        authority.revalidate_current(candidate, trust, crate::m3_store::unix_ms()?)
    }

    pub fn prepare(
        record: &CandidateRecord,
        quarantine_root: &Path,
        scratch_root: &Path,
        wall_time_limit: Duration,
    ) -> Result<Self> {
        let candidate = revalidate_candidate(record, quarantine_root)?;
        let executable = candidate.join(&record.launch_path);
        reject_reparse(&executable)?;
        if !executable.is_absolute() || !executable.is_file() {
            return Err(M3Error::new(
                "launch_path_invalid",
                "launch executable is not an absolute file",
            ));
        }
        let launch_payload = record
            .payloads
            .iter()
            .find(|payload| payload.path == record.launch_path)
            .ok_or_else(|| M3Error::new("launch_path_invalid", "launch payload is not indexed"))?;
        if launch_payload.role != "provider_executable" {
            return Err(M3Error::new(
                "launch_path_invalid",
                "interpreter-backed launch is deferred",
            ));
        }
        let working_directory = candidate.join(&record.provider_working_directory);
        verify_chain(&working_directory)?;
        let working_directory = fs::canonicalize(&working_directory)
            .map_err(|error| M3Error::new("launch_path_invalid", error.to_string()))?;
        if !working_directory.starts_with(&candidate) || !working_directory.is_dir() {
            return Err(M3Error::new(
                "launch_path_invalid",
                "working directory escaped candidate",
            ));
        }
        let scratch_root = StoreRoot::open(scratch_root)?;
        let scratch_directory = scratch_root
            .path()
            .join(format!("session-{}", Uuid::new_v4()));
        fs::create_dir(&scratch_directory)
            .map_err(|error| M3Error::new("launch_io", error.to_string()))?;
        verify_chain(&scratch_directory)?;
        let environment = approved_environment(&scratch_directory)?;
        let environment_digest = sha256(&canonical(&environment)?);
        let mut evidence = LaunchProfileEvidence {
            profile_format_version: 1,
            profile_label: SUPERVISED_PROFILE_LABEL.into(),
            isolated: false,
            limitation: SUPERVISED_PROFILE_LIMITATION.into(),
            candidate_id: record.candidate_id.clone(),
            semantic_package_digest: record.semantic_package_digest.clone(),
            executable_digest: launch_payload.sha256.clone(),
            executable_relative_path: record.launch_path.clone(),
            arguments: record.launch_arguments.clone(),
            working_directory_relative_path: record.provider_working_directory.clone(),
            environment_names: environment.keys().cloned().collect(),
            environment_digest,
            max_processes: 8,
            process_memory_limit_bytes: 256 * 1024 * 1024,
            protocol_line_limit_bytes: 1024 * 1024,
            stderr_tail_limit_bytes: 16 * 1024,
            wall_time_limit_ms: wall_time_limit.as_millis() as u64,
            profile_evidence_digest: String::new(),
        };
        evidence.profile_evidence_digest = sha256(&evidence.covered_bytes()?);
        evidence.validate()?;
        Ok(Self {
            evidence,
            executable,
            working_directory,
            environment,
            scratch_directory,
        })
    }

    pub(crate) fn launch_for_candidate(
        &self,
        candidate: &CandidateRecord,
        trust: &PackageTrustEvidence,
        publisher_trust: &PublisherTrustStore,
        developer_approvals: &DeveloperApprovalStore,
    ) -> std::result::Result<SupervisedChild, ChildError> {
        let authority = PublisherDeveloperTrustAuthority::new(publisher_trust, developer_approvals);
        self.launch_for_candidate_with(candidate, trust, &authority)
    }

    pub(crate) fn launch_for_candidate_with(
        &self,
        candidate: &CandidateRecord,
        trust: &PackageTrustEvidence,
        authority: &dyn CurrentTrustAuthority,
    ) -> std::result::Result<SupervisedChild, ChildError> {
        self.revalidate_current_trust_with(candidate, trust, authority)
            .map_err(|error| ChildError::LaunchFailed {
                command: self.executable.to_string_lossy().into_owned(),
                message: format!("{}: {}", error.code, error.message),
            })?;
        let mut config = ChildConfig::production(
            self.executable.to_string_lossy().into_owned(),
            self.evidence.arguments.clone(),
        );
        config.current_dir = Some(self.working_directory.clone());
        config.clear_environment = true;
        config.environment = self.environment.clone();
        config.max_processes = self.evidence.max_processes;
        config.process_memory_limit_bytes = self.evidence.process_memory_limit_bytes as usize;
        config.assign_before_execution = true;
        config.max_protocol_line_bytes = self.evidence.protocol_line_limit_bytes as usize;
        config.stderr_tail_bytes = self.evidence.stderr_tail_limit_bytes as usize;
        config.graceful_close_timeout = Duration::from_secs(1);
        SupervisedChild::launch(config)
    }

    pub(crate) fn working_directory(&self) -> &Path {
        &self.working_directory
    }

    pub fn scratch_directory(&self) -> &Path {
        &self.scratch_directory
    }

    pub fn cleanup_scratch(self) -> Result<()> {
        verify_chain(&self.scratch_directory)?;
        fs::remove_dir_all(&self.scratch_directory)
            .map_err(|error| M3Error::new("launch_io", error.to_string()))
    }
}
