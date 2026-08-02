//! Explicit M4 enablement authority.
//!
//! Installation remains historical material.  Only this host-owned record can
//! make one exact installed identity available, and it never creates policy or
//! per-call approval.  Disablement is a durable tombstone-like transition.

use crate::installed::InstalledPlugRecord;
use crate::m3_store::{canonical, sha256, unix_ms, M3Error, Result, StoreRoot};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EnablementRecord {
    pub schema_version: u32,
    pub enablement_id: String,
    pub installed_id: String,
    pub package_id: String,
    pub semantic_package_digest: String,
    pub provider_id: String,
    pub provider_version: String,
    pub conformance_evidence_digest: String,
    pub installation_approval_id: String,
    pub capabilities: Vec<EnabledCapability>,
    pub state: EnablementState,
    pub authority: String,
    pub changed_unix_ms: u64,
    pub record_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EnabledCapability {
    pub name: String,
    pub version: u32,
    pub manifest_digest: String,
    pub provider_operation_name: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EnablementState {
    Enabled,
    Disabled,
}

impl EnablementRecord {
    fn covered_bytes(&self) -> Result<Vec<u8>> {
        let mut copy = self.clone();
        copy.record_digest.clear();
        canonical(&copy)
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1
            || Uuid::parse_str(&self.enablement_id).is_err()
            || self.installed_id.is_empty()
            || self.package_id.is_empty()
            || self.provider_id.is_empty()
            || self.provider_version.is_empty()
            || self.semantic_package_digest.len() != 71
            || self.conformance_evidence_digest.len() != 71
            || self.installation_approval_id.is_empty()
            || self.capabilities.is_empty()
            || self.authority.is_empty()
            || self.record_digest != sha256(&self.covered_bytes()?)
        {
            return Err(M3Error::new(
                "enablement_invalid",
                "invalid enablement record",
            ));
        }
        let mut identities = BTreeSet::new();
        if self
            .capabilities
            .iter()
            .any(|cap| cap.version == 0 || !identities.insert((cap.name.clone(), cap.version)))
        {
            return Err(M3Error::new(
                "enablement_invalid",
                "duplicate or invalid capability binding",
            ));
        }
        Ok(())
    }
}

pub struct EnablementStore {
    root: StoreRoot,
}

impl EnablementStore {
    pub fn open(path: &Path) -> Result<Self> {
        Ok(Self {
            root: StoreRoot::open(path)?,
        })
    }

    pub fn enable(
        &self,
        installed: &InstalledPlugRecord,
        authority: &str,
    ) -> Result<EnablementRecord> {
        installed.validate()?;
        if installed.state != "present_disabled" || authority.is_empty() {
            return Err(M3Error::new(
                "enablement_refused",
                "only an installed-disabled Plug may be explicitly enabled",
            ));
        }
        if self.is_available(&installed.installed_id)? {
            return Err(M3Error::new(
                "enablement_conflict",
                "installed Plug is already enabled",
            ));
        }
        let mut record = EnablementRecord {
            schema_version: 1,
            enablement_id: Uuid::new_v4().to_string(),
            installed_id: installed.installed_id.clone(),
            package_id: installed.package_id.clone(),
            semantic_package_digest: installed.semantic_package_digest.clone(),
            provider_id: installed.provider_id.clone(),
            provider_version: installed.provider_version.clone(),
            conformance_evidence_digest: installed.conformance_evidence_digest.clone(),
            installation_approval_id: installed.installation_approval_id.clone(),
            capabilities: installed
                .disabled_bindings
                .iter()
                .map(|binding| EnabledCapability {
                    name: binding.capability_name.clone(),
                    version: binding.capability_version,
                    manifest_digest: binding.manifest_digest.clone(),
                    provider_operation_name: binding.provider_operation_name.clone(),
                })
                .collect(),
            state: EnablementState::Enabled,
            authority: authority.to_owned(),
            changed_unix_ms: unix_ms()?,
            record_digest: String::new(),
        };
        record.record_digest = sha256(&record.covered_bytes()?);
        record.validate()?;
        self.root.create_json(&record.enablement_id, &record)?;
        Ok(record)
    }

    pub fn disable(
        &self,
        installed: &InstalledPlugRecord,
        authority: &str,
    ) -> Result<EnablementRecord> {
        installed.validate()?;
        if authority.is_empty() {
            return Err(M3Error::new(
                "enablement_refused",
                "disablement requires host authority",
            ));
        }
        let mut record = self
            .load_all()?
            .into_iter()
            .rev()
            .find(|record| {
                record.installed_id == installed.installed_id
                    && record.state == EnablementState::Enabled
            })
            .ok_or_else(|| M3Error::new("enablement_refused", "installed Plug is not enabled"))?;
        record.enablement_id = Uuid::new_v4().to_string();
        record.state = EnablementState::Disabled;
        record.authority = authority.to_owned();
        record.changed_unix_ms = unix_ms()?;
        record.record_digest = sha256(&record.covered_bytes()?);
        // The enabled record remains immutable historical evidence.  A separate
        // disabled record is the fail-closed current transition.
        self.root.create_json(&record.enablement_id, &record)?;
        Ok(record)
    }

    pub fn load_all(&self) -> Result<Vec<EnablementRecord>> {
        let mut records = Vec::new();
        let mut ids = BTreeSet::new();
        for path in self.root.entries()? {
            if path.extension().and_then(|v| v.to_str()) == Some("tmp") {
                return Err(M3Error::new("enablement_invalid", "torn enablement record"));
            }
            if path.extension().and_then(|v| v.to_str()) != Some("json") {
                return Err(M3Error::new(
                    "enablement_invalid",
                    "unexpected enablement entry",
                ));
            }
            let record: EnablementRecord = self.root.read(&path)?;
            record.validate()?;
            if path.file_stem().and_then(|v| v.to_str()) != Some(&record.enablement_id)
                || !ids.insert(record.enablement_id.clone())
            {
                return Err(M3Error::new(
                    "enablement_invalid",
                    "enablement identity mismatch",
                ));
            }
            records.push(record);
        }
        Ok(records)
    }

    pub fn is_available(&self, installed_id: &str) -> Result<bool> {
        Ok(self
            .load_all()?
            .into_iter()
            .filter(|record| record.installed_id == installed_id)
            .max_by_key(|record| record.changed_unix_ms)
            .is_some_and(|record| record.state == EnablementState::Enabled))
    }

    pub fn active_provider_identities(&self) -> Result<Vec<String>> {
        let mut providers = BTreeSet::new();
        for record in self
            .load_all()?
            .into_iter()
            .filter(|record| record.state == EnablementState::Enabled)
        {
            if self.is_available(&record.installed_id)? {
                providers.insert(record.provider_id);
            }
        }
        Ok(providers.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::installed::{DisabledBindingRecord, InstalledPlugRecord};
    use crate::trust::{PackageTrustEvidence, TrustModeEvidence};
    use std::fs;

    #[test]
    fn enablement_is_explicit_and_disable_removes_availability() {
        let digest = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let mut trust = PackageTrustEvidence {
            evidence_format_version: 1,
            semantic_package_digest: digest.into(),
            mode: TrustModeEvidence::UnsignedDeveloper {
                approval_id: "approval".into(),
                approval_record_digest: digest.into(),
                visibly_unsigned: true,
            },
            evidence_digest: String::new(),
        };
        let mut trust_covered = trust.clone();
        trust_covered.evidence_digest.clear();
        trust.evidence_digest =
            crate::m3_store::sha256(&crate::m3_store::canonical(&trust_covered).unwrap());
        let mut installed = InstalledPlugRecord {
            schema_version: 1,
            installed_id: Uuid::new_v4().to_string(),
            state: "present_disabled".into(),
            package_id: "tethers.file-tools".into(),
            package_version: "1.0.0".into(),
            semantic_package_digest: digest.into(),
            source_candidate_id: "candidate".into(),
            installation_relative_path: "plug".into(),
            raw_archive_digest: digest.into(),
            plug_json: crate::package::PayloadEvidence {
                path: "plug.json".into(),
                sha256: digest.into(),
                size_bytes: 1,
                role: "package_descriptor".into(),
            },
            payloads: Vec::new(),
            signature_files: Vec::new(),
            capability_manifests: Vec::new(),
            trust_evidence: trust,
            installation_approval_id: "approval".into(),
            installation_approval_digest: digest.into(),
            conformance_evidence_id: "conformance".into(),
            conformance_evidence_digest: digest.into(),
            provider_id: "tethers-file-tools".into(),
            provider_version: "1.0.0".into(),
            launch_path: "provider/file_tools_provider.exe".into(),
            launch_arguments: Vec::new(),
            provider_working_directory: "provider".into(),
            launch_profile_label: "supervised".into(),
            socket_major: 1,
            mcp_protocol_version: "2025-11-25".into(),
            platform: "windows".into(),
            architecture: "x86_64".into(),
            disabled_bindings: vec![DisabledBindingRecord {
                state: "disabled".into(),
                capability_name: "file.move".into(),
                capability_version: 1,
                manifest_digest: digest.into(),
                provider_operation_name: "file_move".into(),
            }],
            created_unix_ms: 1,
            record_digest: String::new(),
        };
        let mut covered = installed.clone();
        covered.record_digest.clear();
        installed.record_digest =
            crate::m3_store::sha256(&crate::m3_store::canonical(&covered).unwrap());
        let root = std::env::temp_dir().join(format!("tethers-m4-enablement-{}", Uuid::new_v4()));
        let store = EnablementStore::open(&root).unwrap();
        assert!(!store.is_available(&installed.installed_id).unwrap());
        store.enable(&installed, "Matthew").unwrap();
        assert!(store.is_available(&installed.installed_id).unwrap());
        store.disable(&installed, "Matthew").unwrap();
        assert!(!store.is_available(&installed.installed_id).unwrap());
        fs::remove_dir_all(root).unwrap();
    }
}
