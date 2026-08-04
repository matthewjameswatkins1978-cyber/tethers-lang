//! Exact-candidate installation trust authority.
//!
//! J24I adds one immutable trust record pinned to candidate ID and candidate-record
//! digest, persisted through the audited StoreRoot authority. It deliberately refuses
//! current-authority revalidation until the future locked executor supplies the exact
//! trust store.

use crate::candidate::CandidateRecord;
use crate::installation_request::{
    InstallationRequest, InstallationTargetState, InstallationTrustScope,
    INSTALLATION_REQUEST_SCHEMA,
};
use crate::m3_store::{canonical, sha256, unix_ms, M3Error, Result, StoreRoot};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExactCandidateTrustRecord {
    pub schema_version: u32,
    pub candidate_id: String,
    pub candidate_record_digest: String,
    pub package_id: String,
    pub package_version: String,
    pub semantic_package_digest: String,
    pub raw_archive_digest: String,
    pub provider_id: String,
    pub provider_version: String,
    pub request_schema: String,
    pub trust_scope: String,
    pub approving_authority: String,
    pub created_unix_ms: u64,
    pub record_digest: String,
}

impl ExactCandidateTrustRecord {
    fn covered_bytes(&self) -> Result<Vec<u8>> {
        let mut copy = self.clone();
        copy.record_digest.clear();
        canonical(&copy)
    }

    fn validate(&self) -> Result<()> {
        let valid_digest = |d: &str| {
            d.strip_prefix("sha256:")
                .map(|hex| {
                    hex.len() == 64
                        && hex
                            .bytes()
                            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
                })
                .unwrap_or(false)
        };
        if self.schema_version != 1
            || self.candidate_id.is_empty()
            || self.candidate_id != self.candidate_id.to_lowercase()
            || !valid_digest(&self.candidate_record_digest)
            || !valid_digest(&self.semantic_package_digest)
            || !valid_digest(&self.raw_archive_digest)
            || !valid_digest(&self.record_digest)
            || self.package_id.is_empty()
            || self.package_version.is_empty()
            || self.provider_id.is_empty()
            || self.provider_version.is_empty()
            || self.approving_authority.is_empty()
            || self.request_schema != INSTALLATION_REQUEST_SCHEMA
            || self.trust_scope != "exact_candidate"
            || self.record_digest != sha256(&self.covered_bytes()?)
        {
            return Err(M3Error::new(
                "installation_trust_invalid",
                "invalid exact-candidate trust record",
            ));
        }
        Ok(())
    }

    pub fn require_for_candidate(&self, candidate: &CandidateRecord) -> Result<()> {
        candidate
            .validate()
            .map_err(|error| M3Error::new("candidate_invalid", error.message))?;
        if self.candidate_id != candidate.candidate_id
            || self.candidate_record_digest != candidate.record_digest
            || self.package_id != candidate.package_id
            || self.package_version != candidate.package_version
            || self.semantic_package_digest != candidate.semantic_package_digest
            || self.raw_archive_digest != candidate.raw_archive_digest
            || self.provider_id != candidate.provider_id
            || self.provider_version != candidate.provider_version
        {
            return Err(M3Error::new(
                "installation_trust_candidate_mismatch",
                "exact-candidate trust is not bound to this candidate",
            ));
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct ExactCandidateTrustStore {
    root: StoreRoot,
}

impl ExactCandidateTrustStore {
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

    pub fn create(
        &self,
        candidate: &CandidateRecord,
        request: &InstallationRequest,
        approving_authority: &str,
    ) -> Result<ExactCandidateTrustRecord> {
        candidate
            .validate()
            .map_err(|error| M3Error::new("candidate_invalid", error.message))?;

        if request.schema != INSTALLATION_REQUEST_SCHEMA {
            return Err(M3Error::new(
                "installation_trust_request_invalid",
                "installation request is not valid for exact-candidate trust",
            ));
        }

        if request.candidate_id != candidate.candidate_id {
            return Err(M3Error::new(
                "installation_trust_request_invalid",
                "installation request is not valid for exact-candidate trust",
            ));
        }

        if !matches!(request.trust.scope, InstallationTrustScope::ExactCandidate) {
            return Err(M3Error::new(
                "installation_trust_request_invalid",
                "installation request is not valid for exact-candidate trust",
            ));
        }

        if !request.conformance.allow_non_isolated_supervised_execution {
            return Err(M3Error::new(
                "installation_trust_request_invalid",
                "installation request is not valid for exact-candidate trust",
            ));
        }

        if !matches!(
            request.installation.target_state,
            InstallationTargetState::Disabled
        ) {
            return Err(M3Error::new(
                "installation_trust_request_invalid",
                "installation request is not valid for exact-candidate trust",
            ));
        }

        if approving_authority.is_empty() {
            return Err(M3Error::new(
                "installation_trust_invalid",
                "approving authority is required",
            ));
        }

        let mut record = ExactCandidateTrustRecord {
            schema_version: 1,
            candidate_id: candidate.candidate_id.clone(),
            candidate_record_digest: candidate.record_digest.clone(),
            package_id: candidate.package_id.clone(),
            package_version: candidate.package_version.clone(),
            semantic_package_digest: candidate.semantic_package_digest.clone(),
            raw_archive_digest: candidate.raw_archive_digest.clone(),
            provider_id: candidate.provider_id.clone(),
            provider_version: candidate.provider_version.clone(),
            request_schema: INSTALLATION_REQUEST_SCHEMA.to_owned(),
            trust_scope: "exact_candidate".to_owned(),
            approving_authority: approving_authority.to_owned(),
            created_unix_ms: unix_ms()?,
            record_digest: String::new(),
        };
        record.record_digest = sha256(&record.covered_bytes()?);
        record.validate()?;
        self.root.create_json(&record.candidate_id, &record)?;
        Ok(record)
    }

    fn validated_view(&self) -> Result<Vec<ExactCandidateTrustRecord>> {
        let mut records = Vec::new();
        let mut ids = HashSet::new();
        for path in self.root.entries()? {
            if path.extension().and_then(|value| value.to_str()) == Some("tmp") {
                return Err(M3Error::new(
                    "installation_trust_invalid",
                    "torn exact-candidate trust record",
                ));
            }
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                return Err(M3Error::new(
                    "installation_trust_invalid",
                    "unexpected exact-candidate trust entry",
                ));
            }
            let record: ExactCandidateTrustRecord = self.root.read(&path)?;
            record.validate()?;
            if path.file_stem().and_then(|value| value.to_str()) != Some(&record.candidate_id) {
                return Err(M3Error::new(
                    "installation_trust_invalid",
                    "exact-candidate trust filename mismatch",
                ));
            }
            if !ids.insert(record.candidate_id.clone()) {
                return Err(M3Error::new(
                    "installation_trust_invalid",
                    "duplicate exact-candidate trust evidence",
                ));
            }
            records.push(record);
        }
        records.sort_by(|a, b| a.candidate_id.cmp(&b.candidate_id));
        Ok(records)
    }

    pub fn find(&self, candidate_id: &str) -> Result<Option<ExactCandidateTrustRecord>> {
        let records = self.validated_view()?;
        Ok(records
            .into_iter()
            .find(|record| record.candidate_id == candidate_id))
    }

    pub fn load_all(&self) -> Result<Vec<ExactCandidateTrustRecord>> {
        self.validated_view()
    }
}
