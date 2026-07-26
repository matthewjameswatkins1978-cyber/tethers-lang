//! J09's host-owned, redacted replay identity model.
//!
//! Persistence is intentionally behind the target-specific backend.  This
//! module owns canonical proofs and the immutable chain validation rules, so a
//! filesystem implementation cannot invent a transition or persist arguments.

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayError {
    InvalidIdentifier,
    BindingMismatch,
    InvalidChain,
    PersistenceUnavailable,
}

impl fmt::Display for ReplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for ReplayError {}

fn digest(value: &Value) -> Result<String, ReplayError> {
    let bytes = serde_json_canonicalizer::to_vec(value).map_err(|_| ReplayError::InvalidChain)?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

fn identifier(value: &str) -> Result<&str, ReplayError> {
    if value.is_empty() || value.chars().any(char::is_whitespace) {
        Err(ReplayError::InvalidIdentifier)
    } else {
        Ok(value)
    }
}

/// Digest of the exact planner tuple; it is the only logical-key material that
/// reaches durable storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicalExecutionKey(String);

impl LogicalExecutionKey {
    pub fn derive(
        anchor_event_id: &str,
        evaluation_id: &str,
        action_id: &str,
    ) -> Result<Self, ReplayError> {
        Ok(Self(digest(&json!({
            "format": "tethers-logical-execution-v1",
            "anchor_event_id": identifier(anchor_event_id)?,
            "evaluation_id": identifier(evaluation_id)?,
            "action_id": identifier(action_id)?,
        }))?))
    }

    pub fn as_digest(&self) -> &str {
        &self.0
    }
}

/// Complete redacted proof bound to a claim.  Raw arguments cannot be supplied
/// to this type; callers must provide their canonical digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionBinding {
    pub evaluation_id: String,
    pub action_id: String,
    pub capability_name: String,
    pub capability_version: u32,
    pub manifest_digest: String,
    pub provider_identity: String,
    pub argument_digest: String,
}

impl ExecutionBinding {
    pub fn digest(&self) -> Result<String, ReplayError> {
        digest(&json!({
            "evaluation_id": self.evaluation_id,
            "action_id": self.action_id,
            "capability_name": self.capability_name,
            "capability_version": self.capability_version,
            "manifest_digest": self.manifest_digest,
            "provider_identity": self.provider_identity,
            "argument_digest": self.argument_digest,
        }))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayState {
    ClaimedNoState,
    IntentRecorded,
    InvocationArmed,
    Succeeded,
    Failed,
    Uncertain,
}

impl ReplayState {
    fn text(self) -> &'static str {
        match self {
            Self::ClaimedNoState => "claimed_no_state",
            Self::IntentRecorded => "intent_recorded",
            Self::InvocationArmed => "invocation_armed",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Uncertain => "uncertain",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Claim {
    pub logical_key: LogicalExecutionKey,
    pub execution_id: String,
    pub binding: ExecutionBinding,
    pub claim_digest: String,
}

impl Claim {
    pub fn new(
        logical_key: LogicalExecutionKey,
        execution_id: String,
        binding: ExecutionBinding,
    ) -> Result<Self, ReplayError> {
        let claim_digest = digest(
            &json!({"record_kind":"identity_claim","ledger_format_version":1,
            "logical_key_digest":logical_key.as_digest(),"execution_id":execution_id,
            "binding_digest":binding.digest()?}),
        )?;
        Ok(Self {
            logical_key,
            execution_id,
            binding,
            claim_digest,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Generation {
    pub number: u8,
    pub state: ReplayState,
    pub predecessor_digest: String,
    pub record_digest: String,
}

pub fn validate_chain(
    claim: &Claim,
    generations: &[Generation],
) -> Result<ReplayState, ReplayError> {
    if generations.len() > 3 {
        return Err(ReplayError::InvalidChain);
    }
    let mut predecessor = claim.claim_digest.clone();
    let mut expected = 0u8;
    let mut state = ReplayState::ClaimedNoState;
    for generation in generations {
        if generation.number != expected || generation.predecessor_digest != predecessor {
            return Err(ReplayError::InvalidChain);
        }
        let allowed = match expected {
            0 => generation.state == ReplayState::IntentRecorded,
            1 => generation.state == ReplayState::InvocationArmed,
            2 => matches!(
                generation.state,
                ReplayState::Succeeded | ReplayState::Failed | ReplayState::Uncertain
            ),
            _ => false,
        };
        if !allowed {
            return Err(ReplayError::InvalidChain);
        }
        predecessor = generation.record_digest.clone();
        state = generation.state;
        expected += 1;
    }
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sibling_actions_are_distinct() {
        assert_ne!(
            LogicalExecutionKey::derive("event", "eval", "a1")
                .unwrap()
                .as_digest(),
            LogicalExecutionKey::derive("event", "eval", "a2")
                .unwrap()
                .as_digest()
        );
    }
    #[test]
    fn chain_cannot_skip_armed() {
        let binding = ExecutionBinding {
            evaluation_id: "eval".into(),
            action_id: "a".into(),
            capability_name: "cap".into(),
            capability_version: 1,
            manifest_digest: "sha256:x".into(),
            provider_identity: "p".into(),
            argument_digest: "sha256:y".into(),
        };
        let claim = Claim::new(
            LogicalExecutionKey::derive("event", "eval", "a").unwrap(),
            "exec_x".into(),
            binding,
        )
        .unwrap();
        assert!(validate_chain(
            &claim,
            &[Generation {
                number: 0,
                state: ReplayState::Succeeded,
                predecessor_digest: claim.claim_digest.clone(),
                record_digest: "d".into()
            }]
        )
        .is_err());
    }
}
