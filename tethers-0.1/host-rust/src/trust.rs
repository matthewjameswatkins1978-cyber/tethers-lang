//! M3 detached Ed25519 evidence, publisher trust, and unsigned developer approval.
//!
//! Signature validity, current publisher trust, and developer approval remain
//! separate facts. None grants installation, enablement, policy, or invocation.

use crate::candidate::CandidateRecord;
use crate::installation_trust::ExactCandidateTrustRecord;
use crate::m3_store::{canonical, sha256, strict_json, unix_ms, M3Error, Result, StoreRoot};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ed25519_dalek::pkcs8::DecodePublicKey;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use uuid::Uuid;

const SIGNATURE_DOMAIN: &str = "tethers.tetherplug.signature.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SignatureEnvelope {
    pub signature_format_version: String,
    pub algorithm: String,
    pub key_id: String,
    pub semantic_package_digest: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VerifiedSignatureEvidence {
    pub evidence_format_version: u32,
    pub key_id: String,
    pub semantic_package_digest: String,
    pub envelope_digest: String,
    pub signing_input_digest: String,
}

pub fn signing_input(semantic_package_digest: &str) -> Vec<u8> {
    format!("{SIGNATURE_DOMAIN}\n{semantic_package_digest}\n").into_bytes()
}

pub fn key_id_from_spki(der_spki: &[u8]) -> Result<String> {
    VerifyingKey::from_public_key_der(der_spki)
        .map_err(|_| M3Error::new("signature_key_invalid", "invalid RFC 8410 Ed25519 SPKI"))?;
    Ok(sha256(der_spki))
}

pub fn verify_ed25519(der_spki: &[u8], message: &[u8], signature: &[u8]) -> Result<()> {
    let key = VerifyingKey::from_public_key_der(der_spki)
        .map_err(|_| M3Error::new("signature_key_invalid", "invalid RFC 8410 Ed25519 SPKI"))?;
    let signature = Signature::from_slice(signature).map_err(|_| {
        M3Error::new(
            "signature_invalid",
            "signature must contain exactly 64 bytes",
        )
    })?;
    key.verify(message, &signature)
        .map_err(|_| M3Error::new("signature_invalid", "Ed25519 verification failed"))
}

pub fn verify_signature_envelope(
    envelope_bytes: &[u8],
    expected_semantic_digest: &str,
    der_spki: &[u8],
) -> Result<VerifiedSignatureEvidence> {
    let envelope: SignatureEnvelope = strict_json(envelope_bytes)?;
    if envelope.signature_format_version != "1" {
        return Err(M3Error::new(
            "signature_profile",
            "unsupported signature format",
        ));
    }
    if envelope.algorithm != "ed25519" {
        return Err(M3Error::new(
            "signature_profile",
            "unsupported signature algorithm",
        ));
    }
    if envelope.semantic_package_digest != expected_semantic_digest {
        return Err(M3Error::new(
            "signature_digest_mismatch",
            "semantic digest differs",
        ));
    }
    let expected_key_id = key_id_from_spki(der_spki)?;
    if envelope.key_id != expected_key_id {
        return Err(M3Error::new(
            "signature_key_mismatch",
            "key identity differs",
        ));
    }
    if envelope.signature.contains('=') {
        return Err(M3Error::new(
            "signature_encoding",
            "base64url padding is forbidden",
        ));
    }
    let decoded = URL_SAFE_NO_PAD
        .decode(envelope.signature.as_bytes())
        .map_err(|_| M3Error::new("signature_encoding", "malformed unpadded base64url"))?;
    if decoded.len() != 64 {
        return Err(M3Error::new(
            "signature_invalid",
            "signature must contain exactly 64 bytes",
        ));
    }
    let input = signing_input(expected_semantic_digest);
    verify_ed25519(der_spki, &input, &decoded)?;
    Ok(VerifiedSignatureEvidence {
        evidence_format_version: 1,
        key_id: expected_key_id,
        semantic_package_digest: expected_semantic_digest.to_owned(),
        envelope_digest: sha256(envelope_bytes),
        signing_input_digest: sha256(&input),
    })
}

pub fn verify_candidate_signatures(
    candidate: &CandidateRecord,
    quarantine_root: &Path,
    trust_store: &PublisherTrustStore,
    now_unix_ms: u64,
) -> Result<Vec<PackageTrustEvidence>> {
    candidate
        .validate()
        .map_err(|error| M3Error::new("candidate_invalid", error.message))?;
    let root = fs::canonicalize(quarantine_root)
        .map_err(|error| M3Error::new("signature_io", error.to_string()))?;
    let directory = fs::canonicalize(root.join(&candidate.quarantine_relative_path))
        .map_err(|error| M3Error::new("signature_io", error.to_string()))?;
    if !directory.starts_with(&root) {
        return Err(M3Error::new(
            "signature_io",
            "candidate escaped quarantine root",
        ));
    }
    if candidate.signature_files.is_empty() {
        return Err(M3Error::new(
            "signature_absent",
            "candidate has no detached signature",
        ));
    }
    let mut keys = BTreeSet::new();
    let mut evidence = Vec::new();
    for file in &candidate.signature_files {
        let filename = signature_filename_key_hex(&file.path)?;
        let bytes = fs::read(directory.join(&file.path))
            .map_err(|error| M3Error::new("signature_io", error.to_string()))?;
        if bytes.len() as u64 != file.size_bytes || sha256(&bytes) != file.sha256 {
            return Err(M3Error::new(
                "signature_drift",
                "detached signature changed",
            ));
        }
        let envelope: SignatureEnvelope = strict_json(&bytes)?;
        if envelope.key_id.strip_prefix("sha256:") != Some(filename) {
            return Err(M3Error::new(
                "signature_filename",
                "filename and key identity differ",
            ));
        }
        if !keys.insert(envelope.key_id.clone()) {
            return Err(M3Error::new(
                "signature_duplicate",
                "duplicate authority from one key",
            ));
        }
        let publisher =
            trust_store.require_trusted(&envelope.key_id, &candidate.package_id, now_unix_ms)?;
        let verified = verify_signature_envelope(
            &bytes,
            &candidate.semantic_package_digest,
            &publisher.der_spki()?,
        )?;
        evidence.push(PackageTrustEvidence::signed(&verified, &publisher)?);
    }
    Ok(evidence)
}

fn signature_filename_key_hex(path: &str) -> Result<&str> {
    let filename = path
        .strip_prefix("signatures/ed25519-")
        .and_then(|value| value.strip_suffix(".json"))
        .ok_or_else(|| M3Error::new("signature_filename", "malformed signature filename"))?;
    if filename.len() != 64
        || !filename
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(M3Error::new(
            "signature_filename",
            "malformed signature key suffix",
        ));
    }
    Ok(filename)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PublisherTrustState {
    Trusted,
    Disabled,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublisherKeyRecord {
    pub schema_version: u32,
    pub record_id: String,
    pub predecessor_record_digest: Option<String>,
    pub key_id: String,
    pub der_spki_base64url: String,
    pub publisher_identity: String,
    pub namespace_restriction: Option<String>,
    pub trust_state: PublisherTrustState,
    pub created_unix_ms: u64,
    pub updated_unix_ms: u64,
    pub approving_authority: String,
    pub expires_unix_ms: Option<u64>,
    pub revocation_reason: Option<String>,
    pub revoked_unix_ms: Option<u64>,
    pub record_digest: String,
}

impl PublisherKeyRecord {
    fn covered_bytes(&self) -> Result<Vec<u8>> {
        let mut copy = self.clone();
        copy.record_digest.clear();
        canonical(&copy)
    }

    fn validate(&self) -> Result<()> {
        let der = URL_SAFE_NO_PAD
            .decode(self.der_spki_base64url.as_bytes())
            .map_err(|_| M3Error::new("trust_record_invalid", "invalid SPKI encoding"))?;
        if self.schema_version != 1
            || Uuid::parse_str(&self.record_id).is_err()
            || self.key_id != key_id_from_spki(&der)?
            || self.publisher_identity.is_empty()
            || self.approving_authority.is_empty()
            || self.record_digest != sha256(&self.covered_bytes()?)
            || (self.trust_state == PublisherTrustState::Revoked
                && (self.revocation_reason.as_deref().unwrap_or("").is_empty()
                    || self.revoked_unix_ms.is_none()))
            || (self.trust_state != PublisherTrustState::Revoked
                && (self.revocation_reason.is_some() || self.revoked_unix_ms.is_some()))
        {
            return Err(M3Error::new(
                "trust_record_invalid",
                "invalid publisher key record",
            ));
        }
        Ok(())
    }

    pub fn der_spki(&self) -> Result<Vec<u8>> {
        URL_SAFE_NO_PAD
            .decode(self.der_spki_base64url.as_bytes())
            .map_err(|_| M3Error::new("trust_record_invalid", "invalid SPKI encoding"))
    }
}

pub struct PublisherTrustStore {
    root: StoreRoot,
}

impl PublisherTrustStore {
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

    pub fn current(&self) -> Result<BTreeMap<String, PublisherKeyRecord>> {
        let mut all = BTreeMap::<String, PublisherKeyRecord>::new();
        for path in self.root.entries()? {
            if path.extension().and_then(|value| value.to_str()) == Some("tmp") {
                return Err(M3Error::new("trust_store_invalid", "torn trust record"));
            }
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                return Err(M3Error::new(
                    "trust_store_invalid",
                    "unexpected trust-store entry",
                ));
            }
            let record: PublisherKeyRecord = self.root.read(&path)?;
            record.validate()?;
            if path.file_stem().and_then(|value| value.to_str()) != Some(&record.record_id) {
                return Err(M3Error::new(
                    "trust_store_invalid",
                    "record filename mismatch",
                ));
            }
            if all.insert(record.record_digest.clone(), record).is_some() {
                return Err(M3Error::new(
                    "trust_store_invalid",
                    "duplicate trust evidence",
                ));
            }
        }
        let predecessors = all
            .values()
            .filter_map(|record| record.predecessor_record_digest.clone())
            .collect::<BTreeSet<_>>();
        for predecessor in &predecessors {
            if !all.contains_key(predecessor) {
                return Err(M3Error::new(
                    "trust_store_invalid",
                    "missing trust predecessor",
                ));
            }
        }
        let mut current = BTreeMap::new();
        for record in all
            .values()
            .filter(|record| !predecessors.contains(&record.record_digest))
        {
            if current
                .insert(record.key_id.clone(), record.clone())
                .is_some()
            {
                return Err(M3Error::new(
                    "trust_store_invalid",
                    "conflicting trust heads",
                ));
            }
        }
        for record in all.values() {
            if let Some(predecessor) = &record.predecessor_record_digest {
                let previous = all.get(predecessor).expect("predecessor existence checked");
                if previous.key_id != record.key_id
                    || previous.der_spki_base64url != record.der_spki_base64url
                    || previous.publisher_identity != record.publisher_identity
                    || previous.namespace_restriction != record.namespace_restriction
                    || record.updated_unix_ms < previous.updated_unix_ms
                {
                    return Err(M3Error::new(
                        "trust_store_invalid",
                        "invalid trust transition",
                    ));
                }
            }
        }
        Ok(current)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn append(
        &self,
        der_spki: &[u8],
        publisher_identity: &str,
        namespace_restriction: Option<String>,
        trust_state: PublisherTrustState,
        approving_authority: &str,
        expires_unix_ms: Option<u64>,
        revocation_reason: Option<String>,
    ) -> Result<PublisherKeyRecord> {
        let key_id = key_id_from_spki(der_spki)?;
        let current = self.current()?;
        let previous = current.get(&key_id);
        let now = unix_ms()?;
        if let Some(previous) = previous {
            if previous.publisher_identity != publisher_identity
                || previous.namespace_restriction != namespace_restriction
                || previous.der_spki()? != der_spki
            {
                return Err(M3Error::new(
                    "trust_conflict",
                    "publisher key mapping conflict",
                ));
            }
            if previous.trust_state == PublisherTrustState::Revoked
                && trust_state != PublisherTrustState::Revoked
            {
                return Err(M3Error::new(
                    "trust_conflict",
                    "a revoked signing key cannot be restored",
                ));
            }
        }
        let mut record = PublisherKeyRecord {
            schema_version: 1,
            record_id: Uuid::new_v4().to_string(),
            predecessor_record_digest: previous.map(|record| record.record_digest.clone()),
            key_id,
            der_spki_base64url: URL_SAFE_NO_PAD.encode(der_spki),
            publisher_identity: publisher_identity.to_owned(),
            namespace_restriction,
            trust_state,
            created_unix_ms: previous.map_or(now, |record| record.created_unix_ms),
            updated_unix_ms: now,
            approving_authority: approving_authority.to_owned(),
            expires_unix_ms,
            revocation_reason: if trust_state == PublisherTrustState::Revoked {
                revocation_reason
            } else {
                None
            },
            revoked_unix_ms: (trust_state == PublisherTrustState::Revoked).then_some(now),
            record_digest: String::new(),
        };
        record.record_digest = sha256(&record.covered_bytes()?);
        record.validate()?;
        self.root.create_json(&record.record_id, &record)?;
        Ok(record)
    }

    pub fn require_trusted(
        &self,
        key_id: &str,
        package_id: &str,
        now_unix_ms: u64,
    ) -> Result<PublisherKeyRecord> {
        let record = self
            .current()?
            .remove(key_id)
            .ok_or_else(|| M3Error::new("trust_unknown", "signing key is not host-known"))?;
        if record.trust_state != PublisherTrustState::Trusted {
            return Err(M3Error::new(
                "trust_not_current",
                "signing key is disabled or revoked",
            ));
        }
        if record
            .expires_unix_ms
            .is_some_and(|expiry| expiry <= now_unix_ms)
        {
            return Err(M3Error::new(
                "trust_expired",
                "signing key trust has expired",
            ));
        }
        if record
            .namespace_restriction
            .as_ref()
            .is_some_and(|prefix| !package_id.starts_with(prefix))
        {
            return Err(M3Error::new(
                "trust_namespace",
                "package is outside trusted namespace",
            ));
        }
        Ok(record)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DeveloperApprovalRecord {
    pub schema_version: u32,
    pub approval_id: String,
    pub semantic_package_digest: String,
    pub visibly_unsigned: bool,
    pub approving_authority: String,
    pub created_unix_ms: u64,
    pub record_digest: String,
}

impl DeveloperApprovalRecord {
    fn covered_bytes(&self) -> Result<Vec<u8>> {
        let mut copy = self.clone();
        copy.record_digest.clear();
        canonical(&copy)
    }

    fn validate(&self) -> Result<()> {
        let semantic_digest =
            self.semantic_package_digest
                .strip_prefix("sha256:")
                .filter(|digest| {
                    digest.len() == 64
                        && digest
                            .bytes()
                            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
                });
        if self.schema_version != 1
            || Uuid::parse_str(&self.approval_id).is_err()
            || !self.visibly_unsigned
            || self.approving_authority.is_empty()
            || semantic_digest.is_none()
            || self.record_digest != sha256(&self.covered_bytes()?)
        {
            return Err(M3Error::new(
                "developer_approval_invalid",
                "invalid developer approval",
            ));
        }
        Ok(())
    }
}

pub struct DeveloperApprovalStore {
    root: StoreRoot,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum TrustModeEvidence {
    SignedPublisher {
        key_id: String,
        publisher_identity: String,
        signature_evidence_digest: String,
        trust_record_digest: String,
    },
    UnsignedDeveloper {
        approval_id: String,
        approval_record_digest: String,
        visibly_unsigned: bool,
    },
    ExactCandidate {
        candidate_id: String,
        candidate_record_digest: String,
        installation_trust_record_digest: String,
        approving_authority: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PackageTrustEvidence {
    pub evidence_format_version: u32,
    pub semantic_package_digest: String,
    pub mode: TrustModeEvidence,
    pub evidence_digest: String,
}

impl PackageTrustEvidence {
    pub fn require_for_candidate(&self, candidate: &CandidateRecord) -> Result<()> {
        self.validate()?;
        if self.semantic_package_digest != candidate.semantic_package_digest {
            return Err(M3Error::new(
                "trust_candidate_mismatch",
                "trust evidence is not bound to this candidate semantic digest",
            ));
        }
        if let TrustModeEvidence::ExactCandidate {
            candidate_id,
            candidate_record_digest,
            ..
        } = &self.mode
        {
            candidate
                .validate()
                .map_err(|error| M3Error::new("candidate_invalid", error.message))?;
            if *candidate_id != candidate.candidate_id
                || *candidate_record_digest != candidate.record_digest
            {
                return Err(M3Error::new(
                    "trust_candidate_mismatch",
                    "trust evidence is not bound to this candidate semantic digest",
                ));
            }
        }
        Ok(())
    }

    fn covered_bytes(&self) -> Result<Vec<u8>> {
        let mut copy = self.clone();
        copy.evidence_digest.clear();
        canonical(&copy)
    }

    pub fn signed(
        signature: &VerifiedSignatureEvidence,
        publisher: &PublisherKeyRecord,
    ) -> Result<Self> {
        if signature.key_id != publisher.key_id
            || publisher.trust_state != PublisherTrustState::Trusted
        {
            return Err(M3Error::new(
                "trust_evidence_invalid",
                "signature and current publisher trust differ",
            ));
        }
        let mut evidence = Self {
            evidence_format_version: 1,
            semantic_package_digest: signature.semantic_package_digest.clone(),
            mode: TrustModeEvidence::SignedPublisher {
                key_id: publisher.key_id.clone(),
                publisher_identity: publisher.publisher_identity.clone(),
                signature_evidence_digest: signature.envelope_digest.clone(),
                trust_record_digest: publisher.record_digest.clone(),
            },
            evidence_digest: String::new(),
        };
        evidence.evidence_digest = sha256(&evidence.covered_bytes()?);
        Ok(evidence)
    }

    pub fn unsigned(approval: &DeveloperApprovalRecord) -> Result<Self> {
        let mut evidence = Self {
            evidence_format_version: 1,
            semantic_package_digest: approval.semantic_package_digest.clone(),
            mode: TrustModeEvidence::UnsignedDeveloper {
                approval_id: approval.approval_id.clone(),
                approval_record_digest: approval.record_digest.clone(),
                visibly_unsigned: true,
            },
            evidence_digest: String::new(),
        };
        evidence.evidence_digest = sha256(&evidence.covered_bytes()?);
        Ok(evidence)
    }

    pub fn exact_candidate(record: &ExactCandidateTrustRecord) -> Result<Self> {
        record.validate()?;
        let mut evidence = Self {
            evidence_format_version: 1,
            semantic_package_digest: record.semantic_package_digest.clone(),
            mode: TrustModeEvidence::ExactCandidate {
                candidate_id: record.candidate_id.clone(),
                candidate_record_digest: record.candidate_record_digest.clone(),
                installation_trust_record_digest: record.record_digest.clone(),
                approving_authority: record.approving_authority.clone(),
            },
            evidence_digest: String::new(),
        };
        evidence.evidence_digest = sha256(&evidence.covered_bytes()?);
        evidence.validate()?;
        Ok(evidence)
    }

    pub fn validate(&self) -> Result<()> {
        if self.evidence_format_version != 1
            || self.evidence_digest != sha256(&self.covered_bytes()?)
            || matches!(
                self.mode,
                TrustModeEvidence::UnsignedDeveloper {
                    visibly_unsigned: false,
                    ..
                }
            )
        {
            return Err(M3Error::new(
                "trust_evidence_invalid",
                "invalid trust evidence",
            ));
        }
        if let TrustModeEvidence::ExactCandidate {
            candidate_id,
            candidate_record_digest,
            installation_trust_record_digest,
            approving_authority,
        } = &self.mode
        {
            if !is_valid_candidate_id(candidate_id)
                || !is_sha256_digest(candidate_record_digest)
                || !is_sha256_digest(installation_trust_record_digest)
                || approving_authority.is_empty()
                || !is_sha256_digest(&self.semantic_package_digest)
            {
                return Err(M3Error::new(
                    "trust_evidence_invalid",
                    "invalid trust evidence",
                ));
            }
        }
        Ok(())
    }

    pub fn revalidate_current(
        &self,
        package_id: &str,
        trust_store: &PublisherTrustStore,
        developer_store: &DeveloperApprovalStore,
        now_unix_ms: u64,
    ) -> Result<()> {
        self.validate()?;
        match &self.mode {
            TrustModeEvidence::SignedPublisher {
                key_id,
                publisher_identity,
                ..
            } => {
                let current = trust_store.require_trusted(key_id, package_id, now_unix_ms)?;
                if current.publisher_identity != *publisher_identity {
                    return Err(M3Error::new("trust_drift", "publisher mapping changed"));
                }
            }
            TrustModeEvidence::UnsignedDeveloper {
                approval_id,
                approval_record_digest,
                visibly_unsigned,
            } => {
                let approval = developer_store
                    .find(&self.semantic_package_digest)?
                    .ok_or_else(|| M3Error::new("trust_drift", "developer approval is absent"))?;
                if !*visibly_unsigned
                    || approval.approval_id != *approval_id
                    || approval.record_digest != *approval_record_digest
                {
                    return Err(M3Error::new(
                        "trust_drift",
                        "developer approval evidence changed",
                    ));
                }
            }
            TrustModeEvidence::ExactCandidate { .. } => {
                return Err(M3Error::new(
                    "trust_exact_candidate_authority_required",
                    "exact-candidate trust requires current installation-trust authority",
                ));
            }
        }
        Ok(())
    }
}

fn is_valid_candidate_id(value: &str) -> bool {
    Uuid::parse_str(value)
        .map(|parsed| parsed.hyphenated().to_string() == value)
        .unwrap_or(false)
}

fn is_sha256_digest(value: &str) -> bool {
    value
        .strip_prefix("sha256:")
        .map(|hex| {
            hex.len() == 64
                && hex
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
        .unwrap_or(false)
}

impl DeveloperApprovalStore {
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

    pub fn approve_exact_digest(
        &self,
        semantic_package_digest: &str,
        approving_authority: &str,
    ) -> Result<DeveloperApprovalRecord> {
        if self.find(semantic_package_digest)?.is_some() {
            return Err(M3Error::new(
                "developer_approval_conflict",
                "digest already approved",
            ));
        }
        let mut record = DeveloperApprovalRecord {
            schema_version: 1,
            approval_id: Uuid::new_v4().to_string(),
            semantic_package_digest: semantic_package_digest.to_owned(),
            visibly_unsigned: true,
            approving_authority: approving_authority.to_owned(),
            created_unix_ms: unix_ms()?,
            record_digest: String::new(),
        };
        record.record_digest = sha256(&record.covered_bytes()?);
        record.validate()?;
        self.root.create_json(&record.approval_id, &record)?;
        Ok(record)
    }

    pub fn find(&self, digest: &str) -> Result<Option<DeveloperApprovalRecord>> {
        let mut found = None;
        for path in self.root.entries()? {
            if path.extension().and_then(|value| value.to_str()) == Some("tmp") {
                return Err(M3Error::new("developer_approval_invalid", "torn approval"));
            }
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                return Err(M3Error::new(
                    "developer_approval_invalid",
                    "unexpected approval entry",
                ));
            }
            let record: DeveloperApprovalRecord = self.root.read(&path)?;
            record.validate()?;
            if path.file_stem().and_then(|value| value.to_str()) != Some(&record.approval_id) {
                return Err(M3Error::new(
                    "developer_approval_invalid",
                    "approval filename mismatch",
                ));
            }
            if record.semantic_package_digest == digest && found.replace(record).is_some() {
                return Err(M3Error::new(
                    "developer_approval_invalid",
                    "duplicate digest approval",
                ));
            }
        }
        Ok(found)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::pkcs8::EncodePublicKey;
    use ed25519_dalek::{Signer, SigningKey};
    use std::fs;

    fn root(name: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("tethers-m3-trust-{name}-{}", Uuid::new_v4()));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn signing_key() -> SigningKey {
        SigningKey::from_bytes(&[7u8; 32])
    }

    fn spki(key: &SigningKey) -> Vec<u8> {
        key.verifying_key()
            .to_public_key_der()
            .unwrap()
            .as_bytes()
            .to_vec()
    }

    #[test]
    fn rfc8032_empty_message_vector_is_accepted() {
        let public =
            hex::decode("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a")
                .unwrap();
        let signature = hex::decode("e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b").unwrap();
        let mut der = hex::decode("302a300506032b6570032100").unwrap();
        der.extend(public);
        verify_ed25519(&der, b"", &signature).unwrap();
    }

    #[test]
    fn strict_envelope_binds_final_newline_digest_and_key() {
        let key = signing_key();
        let der = spki(&key);
        let digest = format!("sha256:{}", "1".repeat(64));
        let signature = key.sign(&signing_input(&digest));
        let envelope = SignatureEnvelope {
            signature_format_version: "1".into(),
            algorithm: "ed25519".into(),
            key_id: key_id_from_spki(&der).unwrap(),
            semantic_package_digest: digest.clone(),
            signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
        };
        let bytes = serde_json::to_vec(&envelope).unwrap();
        verify_signature_envelope(&bytes, &digest, &der).unwrap();

        let without_final_newline = format!("{SIGNATURE_DOMAIN}\n{digest}");
        let wrong = key.sign(without_final_newline.as_bytes());
        let mut wrong_envelope = envelope.clone();
        wrong_envelope.signature = URL_SAFE_NO_PAD.encode(wrong.to_bytes());
        assert_eq!(
            verify_signature_envelope(&serde_json::to_vec(&wrong_envelope).unwrap(), &digest, &der)
                .unwrap_err()
                .code,
            "signature_invalid"
        );
        let duplicate = format!(
            "{{\"signature_format_version\":\"1\",\"algorithm\":\"ed25519\",\"key_id\":\"{}\",\"key_id\":\"{}\",\"semantic_package_digest\":\"{}\",\"signature\":\"{}\"}}",
            envelope.key_id, envelope.key_id, digest, envelope.signature
        );
        assert_eq!(
            verify_signature_envelope(duplicate.as_bytes(), &digest, &der)
                .unwrap_err()
                .code,
            "record_invalid"
        );
        let mut padded = envelope;
        padded.signature.push('=');
        assert_eq!(
            verify_signature_envelope(&serde_json::to_vec(&padded).unwrap(), &digest, &der)
                .unwrap_err()
                .code,
            "signature_encoding"
        );
    }

    #[test]
    fn signature_refusal_matrix_is_typed_and_fail_closed() {
        let key = signing_key();
        let der = spki(&key);
        let digest = format!("sha256:{}", "4".repeat(64));
        let signature = key.sign(&signing_input(&digest));
        let envelope = SignatureEnvelope {
            signature_format_version: "1".into(),
            algorithm: "ed25519".into(),
            key_id: key_id_from_spki(&der).unwrap(),
            semantic_package_digest: digest.clone(),
            signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
        };
        let bytes = serde_json::to_vec(&envelope).unwrap();
        assert_eq!(
            verify_signature_envelope(&bytes, &format!("sha256:{}", "5".repeat(64)), &der)
                .unwrap_err()
                .code,
            "signature_digest_mismatch"
        );
        let other = signing_key().to_bytes().map(|byte| byte ^ 0x55);
        let other = SigningKey::from_bytes(&other);
        assert_eq!(
            verify_signature_envelope(&bytes, &digest, &spki(&other))
                .unwrap_err()
                .code,
            "signature_key_mismatch"
        );
        let mut malformed = envelope.clone();
        malformed.signature = URL_SAFE_NO_PAD.encode([0u8; 63]);
        assert_eq!(
            verify_signature_envelope(&serde_json::to_vec(&malformed).unwrap(), &digest, &der)
                .unwrap_err()
                .code,
            "signature_invalid"
        );
        let mut altered = signature.to_bytes();
        altered[0] ^= 1;
        malformed.signature = URL_SAFE_NO_PAD.encode(altered);
        assert_eq!(
            verify_signature_envelope(&serde_json::to_vec(&malformed).unwrap(), &digest, &der)
                .unwrap_err()
                .code,
            "signature_invalid"
        );
        let unknown = serde_json::json!({
            "signature_format_version":"1","algorithm":"ed25519","key_id":envelope.key_id,
            "semantic_package_digest":digest,"signature":envelope.signature,"authority":true
        });
        assert_eq!(
            verify_signature_envelope(&serde_json::to_vec(&unknown).unwrap(), &digest, &der)
                .unwrap_err()
                .code,
            "record_invalid"
        );
        assert_eq!(
            signature_filename_key_hex("signatures/ED25519-bad.json")
                .unwrap_err()
                .code,
            "signature_filename"
        );
    }

    #[test]
    fn unknown_disabled_expired_and_conflicting_trust_are_refused() {
        let root = root("state-refusals");
        let key = signing_key();
        let der = spki(&key);
        let store = PublisherTrustStore::open(&root).unwrap();
        let key_id = key_id_from_spki(&der).unwrap();
        assert_eq!(
            store
                .require_trusted(&key_id, "tethers.example", 0)
                .unwrap_err()
                .code,
            "trust_unknown"
        );
        store
            .append(
                &der,
                "publisher:host",
                None,
                PublisherTrustState::Disabled,
                "Matthew",
                None,
                None,
            )
            .unwrap();
        assert_eq!(
            store
                .require_trusted(&key_id, "tethers.example", 0)
                .unwrap_err()
                .code,
            "trust_not_current"
        );
        store
            .append(
                &der,
                "publisher:host",
                None,
                PublisherTrustState::Trusted,
                "Matthew",
                Some(10),
                None,
            )
            .unwrap();
        assert_eq!(
            store
                .require_trusted(&key_id, "tethers.example", 10)
                .unwrap_err()
                .code,
            "trust_expired"
        );
        assert_eq!(
            store
                .append(
                    &der,
                    "publisher:changed",
                    None,
                    PublisherTrustState::Trusted,
                    "Matthew",
                    None,
                    None,
                )
                .unwrap_err()
                .code,
            "trust_conflict"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn trust_transitions_restart_and_revocation_fail_closed() {
        let root = root("transition");
        let key = signing_key();
        let der = spki(&key);
        let store = PublisherTrustStore::open(&root).unwrap();
        let trusted = store
            .append(
                &der,
                "publisher:host-owned",
                Some("tethers.".into()),
                PublisherTrustState::Trusted,
                "Matthew",
                None,
                None,
            )
            .unwrap();
        assert_ne!(trusted.publisher_identity, "package presentation text");
        store
            .require_trusted(&trusted.key_id, "tethers.example", u64::MAX - 1)
            .unwrap();
        store
            .append(
                &der,
                "publisher:host-owned",
                Some("tethers.".into()),
                PublisherTrustState::Revoked,
                "Matthew",
                None,
                Some("key retired".into()),
            )
            .unwrap();
        let reopened = PublisherTrustStore::open(&root).unwrap();
        assert_eq!(
            reopened
                .require_trusted(&trusted.key_id, "tethers.example", 0)
                .unwrap_err()
                .code,
            "trust_not_current"
        );
        assert_eq!(
            reopened
                .append(
                    &der,
                    "publisher:host-owned",
                    Some("tethers.".into()),
                    PublisherTrustState::Trusted,
                    "Matthew",
                    None,
                    None,
                )
                .unwrap_err()
                .code,
            "trust_conflict"
        );
        fs::write(root.join(".torn.tmp"), b"{}").unwrap();
        assert_eq!(reopened.current().unwrap_err().code, "trust_store_invalid");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn developer_approval_is_exact_digest_only() {
        let root = root("developer");
        let store = DeveloperApprovalStore::open(&root).unwrap();
        let digest = format!("sha256:{}", "2".repeat(64));
        let record = store.approve_exact_digest(&digest, "Matthew").unwrap();
        assert!(record.visibly_unsigned);
        assert!(store.find(&digest).unwrap().is_some());
        assert!(store
            .find(&format!("sha256:{}", "3".repeat(64)))
            .unwrap()
            .is_none());
        assert_eq!(
            store
                .approve_exact_digest("sha256:ABC", "Matthew")
                .unwrap_err()
                .code,
            "developer_approval_invalid"
        );
        let _ = fs::remove_dir_all(root);
    }
}
