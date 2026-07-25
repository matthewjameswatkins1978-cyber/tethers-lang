//! Host-owned, in-memory exact one-shot Ask approvals.
//!
//! This module owns the proof and lifecycle only.  It deliberately cannot
//! create a dispatch permission; the production seam re-runs effective policy
//! and asks `policy` for the sole post-consumption Allow proof.

use crate::policy::ProposedAction;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub const APPROVAL_FORMAT_VERSION: &str = "1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalProof {
    pub approval_format_version: String,
    pub evaluation_id: String,
    pub plan_id: String,
    pub action_id: String,
    pub capability_name: String,
    pub capability_version: u32,
    pub argument_digest: String,
    pub manifest_digest: String,
    pub provider_identity: String,
    pub approval_binding_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalProofError {
    MissingField(&'static str),
}

impl std::fmt::Display for ApprovalProofError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "missing approval proof field: {self:?}")
    }
}
impl std::error::Error for ApprovalProofError {}

impl ApprovalProof {
    pub fn from_action(action: &ProposedAction) -> Result<Self, ApprovalProofError> {
        let manifest_digest = action
            .manifest_digest
            .as_deref()
            .filter(|value| !value.is_empty())
            .ok_or(ApprovalProofError::MissingField("manifest_digest"))?;
        let capability_version =
            action
                .bridge_capability_version
                .ok_or(ApprovalProofError::MissingField(
                    "bridge_capability_version",
                ))?;
        let provider_identity = action
            .bridge_provider_identity
            .as_deref()
            .filter(|value| !value.is_empty())
            .ok_or(ApprovalProofError::MissingField("bridge_provider_identity"))?;
        Ok(Self::new(
            action.evaluation_id.clone(),
            action.plan_id.clone(),
            action.action_id.clone(),
            action.capability_name.clone(),
            capability_version,
            &action.arguments,
            manifest_digest.to_owned(),
            provider_identity.to_owned(),
        ))
    }

    pub fn new(
        evaluation_id: String,
        plan_id: String,
        action_id: String,
        capability_name: String,
        capability_version: u32,
        arguments: &serde_json::Value,
        manifest_digest: String,
        provider_identity: String,
    ) -> Self {
        let mut proof = Self {
            approval_format_version: APPROVAL_FORMAT_VERSION.to_owned(),
            evaluation_id,
            plan_id,
            action_id,
            capability_name,
            capability_version,
            argument_digest: digest(arguments),
            manifest_digest,
            provider_identity,
            approval_binding_digest: String::new(),
        };
        proof.approval_binding_digest = proof.binding_digest();
        proof
    }

    fn binding_digest(&self) -> String {
        digest(&json!({
            "approval_format_version": self.approval_format_version,
            "evaluation_id": self.evaluation_id,
            "plan_id": self.plan_id,
            "action_id": self.action_id,
            "capability_name": self.capability_name,
            "capability_version": self.capability_version,
            "argument_digest": self.argument_digest,
            "manifest_digest": self.manifest_digest,
            "provider_identity": self.provider_identity,
        }))
    }

    /// Both the visible fields and the independently derived binding digest
    /// must match.  Never treat a digest match as a substitute for the fields.
    pub fn exactly_matches(&self, other: &Self) -> bool {
        self.approval_format_version == other.approval_format_version
            && self.evaluation_id == other.evaluation_id
            && self.plan_id == other.plan_id
            && self.action_id == other.action_id
            && self.capability_name == other.capability_name
            && self.capability_version == other.capability_version
            && self.argument_digest == other.argument_digest
            && self.manifest_digest == other.manifest_digest
            && self.provider_identity == other.provider_identity
            && self.approval_binding_digest == other.approval_binding_digest
            && self.approval_binding_digest == self.binding_digest()
            && other.approval_binding_digest == other.binding_digest()
    }
}

fn digest(value: &serde_json::Value) -> String {
    let bytes = serde_json_canonicalizer::to_vec(value).expect("canonical JSON value");
    format!("sha256:{:x}", Sha256::digest(bytes))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalState {
    Pending,
    Approved,
    Denied,
    Cancelled,
    Invalidated,
    Consumed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalRecord {
    pub approval_id: String,
    pub proof: ApprovalProof,
    pub state: ApprovalState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalError {
    Missing,
    Pending,
    Denied,
    Cancelled,
    Invalidated,
    Consumed,
    ProofMismatch,
}

impl std::fmt::Display for ApprovalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "approval state error: {self:?}")
    }
}
impl std::error::Error for ApprovalError {}

impl ApprovalError {
    fn from_state(state: ApprovalState) -> Self {
        match state {
            ApprovalState::Pending => Self::Pending,
            ApprovalState::Approved => Self::Pending,
            ApprovalState::Denied => Self::Denied,
            ApprovalState::Cancelled => Self::Cancelled,
            ApprovalState::Invalidated => Self::Invalidated,
            ApprovalState::Consumed => Self::Consumed,
        }
    }
}

/// This store is intentionally process-local.  A new instance has no records,
/// which is the required restart expiry semantics for 0.2.
#[derive(Debug, Default)]
pub struct ApprovalStore {
    records: HashMap<String, ApprovalRecord>,
    next_identity: u64,
}

impl ApprovalStore {
    pub fn request(&mut self, proof: ApprovalProof) -> ApprovalRecord {
        self.next_identity += 1;
        let record = ApprovalRecord {
            approval_id: format!("approval-{}", self.next_identity),
            proof,
            state: ApprovalState::Pending,
        };
        self.records
            .insert(record.approval_id.clone(), record.clone());
        record
    }

    pub fn record(&self, approval_id: &str) -> Result<&ApprovalRecord, ApprovalError> {
        self.records.get(approval_id).ok_or(ApprovalError::Missing)
    }

    pub fn pending_matching(&self, proof: &ApprovalProof) -> Option<&ApprovalRecord> {
        self.records.values().find(|record| {
            record.state == ApprovalState::Pending && record.proof.exactly_matches(proof)
        })
    }

    pub fn decide(
        &mut self,
        approval_id: &str,
        next: ApprovalState,
    ) -> Result<ApprovalRecord, ApprovalError> {
        if !matches!(
            next,
            ApprovalState::Approved | ApprovalState::Denied | ApprovalState::Cancelled
        ) {
            return Err(ApprovalError::Pending);
        }
        self.transition(approval_id, ApprovalState::Pending, next)
    }

    pub fn invalidate_live(
        &mut self,
        approval_id: &str,
    ) -> Result<Option<ApprovalRecord>, ApprovalError> {
        let state = self.record(approval_id)?.state;
        match state {
            ApprovalState::Pending | ApprovalState::Approved => self
                .transition(approval_id, state, ApprovalState::Invalidated)
                .map(Some),
            _ => Ok(None),
        }
    }

    /// Single mutable transition is the atomic consume boundary.  Callers
    /// cannot receive a dispatch Allow until this has completed.
    pub fn consume(
        &mut self,
        approval_id: &str,
        fresh_proof: &ApprovalProof,
    ) -> Result<ApprovalRecord, ApprovalError> {
        let record = self
            .records
            .get(approval_id)
            .ok_or(ApprovalError::Missing)?;
        if !record.proof.exactly_matches(fresh_proof) {
            return Err(ApprovalError::ProofMismatch);
        }
        self.transition(
            approval_id,
            ApprovalState::Approved,
            ApprovalState::Consumed,
        )
    }

    fn transition(
        &mut self,
        approval_id: &str,
        expected: ApprovalState,
        next: ApprovalState,
    ) -> Result<ApprovalRecord, ApprovalError> {
        let record = self
            .records
            .get_mut(approval_id)
            .ok_or(ApprovalError::Missing)?;
        if record.state != expected {
            return Err(ApprovalError::from_state(record.state));
        }
        record.state = next;
        Ok(record.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn proof() -> ApprovalProof {
        ApprovalProof::new(
            "eval".into(),
            "plan".into(),
            "action".into(),
            "fixture.ask".into(),
            1,
            &json!({"value": "safe"}),
            "sha256:manifest".into(),
            "fixture".into(),
        )
    }

    #[test]
    fn deterministic_proof_includes_every_field_and_excludes_arguments() {
        let a = proof();
        assert_eq!(a, proof());
        assert!(!format!("{a:?}").contains("safe"));
        let mut changed = proof();
        changed.plan_id = "other".into();
        changed.approval_binding_digest = changed.binding_digest();
        assert!(!a.exactly_matches(&changed));
    }

    #[test]
    fn records_are_distinct_and_terminal_records_are_not_pending() {
        let mut store = ApprovalStore::default();
        let first = store.request(proof());
        store
            .decide(&first.approval_id, ApprovalState::Cancelled)
            .unwrap();
        let second = store.request(proof());
        assert_ne!(first.approval_id, second.approval_id);
        assert_eq!(
            store.record(&first.approval_id).unwrap().state,
            ApprovalState::Cancelled
        );
        assert_eq!(
            store.record(&second.approval_id).unwrap().state,
            ApprovalState::Pending
        );
    }

    #[test]
    fn consume_is_one_shot_and_requires_full_proof() {
        let mut store = ApprovalStore::default();
        let record = store.request(proof());
        store
            .decide(&record.approval_id, ApprovalState::Approved)
            .unwrap();
        assert_eq!(
            store.consume(&record.approval_id, &proof()).unwrap().state,
            ApprovalState::Consumed
        );
        assert_eq!(
            store.consume(&record.approval_id, &proof()),
            Err(ApprovalError::Consumed)
        );
    }
}
