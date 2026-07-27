//! J09's host-owned, redacted replay identity and immutable-record model.
//!
//! Persistence is intentionally behind the target-specific backend. This
//! module owns canonical proofs and the exact chain graph, so filesystem code
//! cannot invent a transition or persist raw arguments.

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fmt;
use uuid::{Uuid, Variant, Version};

const SHA256_PREFIX: &str = "sha256:";
const IDENTITY_CLAIM_KIND: &str = "identity_claim";
const REPLAY_GENERATION_KIND: &str = "replay_generation";
const LEDGER_FORMAT_VERSION: u32 = 1;
const GENERATION_RECORD_VERSION: u32 = 1;

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

fn canonical_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, ReplayError> {
    serde_json_canonicalizer::to_vec(value).map_err(|_| ReplayError::InvalidChain)
}

fn canonical_digest<T: Serialize>(value: &T) -> Result<String, ReplayError> {
    Ok(digest_bytes(&canonical_bytes(value)?))
}

/// Digest one successfully durable, redacted J06 outcome entry using the same
/// canonical JSON and SHA-256 vocabulary as replay records.
pub fn durable_outcome_digest<T: Serialize>(value: &T) -> Result<String, ReplayError> {
    canonical_digest(value)
}

fn digest_bytes(bytes: &[u8]) -> String {
    format!("{SHA256_PREFIX}{:x}", Sha256::digest(bytes))
}

fn parse_canonical<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, ReplayError> {
    let value: Value = serde_json::from_slice(bytes).map_err(|_| ReplayError::InvalidChain)?;
    if canonical_bytes(&value)? != bytes {
        return Err(ReplayError::InvalidChain);
    }
    serde_json::from_value(value).map_err(|_| ReplayError::InvalidChain)
}

fn identifier(value: &str) -> Result<&str, ReplayError> {
    if value.is_empty() || value.chars().any(char::is_whitespace) {
        Err(ReplayError::InvalidIdentifier)
    } else {
        Ok(value)
    }
}

fn validate_digest(value: &str) -> Result<(), ReplayError> {
    let Some(hex) = value.strip_prefix(SHA256_PREFIX) else {
        return Err(ReplayError::InvalidChain);
    };
    if hex.len() != 64
        || !hex
            .as_bytes()
            .iter()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(ReplayError::InvalidChain);
    }
    Ok(())
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
        Ok(Self(canonical_digest(&json!({
            "format": "tethers-logical-execution-v1",
            "anchor_event_id": identifier(anchor_event_id)?,
            "evaluation_id": identifier(evaluation_id)?,
            "action_id": identifier(action_id)?,
        }))?))
    }

    pub fn from_digest(value: String) -> Result<Self, ReplayError> {
        validate_digest(&value)?;
        Ok(Self(value))
    }

    pub fn as_digest(&self) -> &str {
        &self.0
    }

    pub fn filename_digest(&self) -> &str {
        // The private constructor and `from_digest` establish this once.
        self.0
            .strip_prefix(SHA256_PREFIX)
            .expect("validated logical-key digest")
    }
}

/// Complete redacted proof bound to a claim. Raw arguments cannot be supplied
/// to this type; callers provide only their already-canonical digest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
    fn validate(&self) -> Result<(), ReplayError> {
        identifier(&self.evaluation_id)?;
        identifier(&self.action_id)?;
        identifier(&self.capability_name)?;
        identifier(&self.provider_identity)?;
        if self.capability_version == 0 {
            return Err(ReplayError::InvalidChain);
        }
        validate_digest(&self.manifest_digest)?;
        validate_digest(&self.argument_digest)?;
        Ok(())
    }

    pub fn digest(&self) -> Result<String, ReplayError> {
        self.validate()?;
        canonical_digest(self)
    }
}

/// Opaque host-created identity. Its constructor does not accept caller data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionId(String);

impl ExecutionId {
    pub fn generate() -> Self {
        Self(format!("exec_{}", Uuid::new_v4().hyphenated()))
    }

    pub fn parse(value: String) -> Result<Self, ReplayError> {
        let Some(uuid_text) = value.strip_prefix("exec_") else {
            return Err(ReplayError::InvalidChain);
        };
        let uuid = Uuid::parse_str(uuid_text).map_err(|_| ReplayError::InvalidChain)?;
        if uuid.get_variant() != Variant::RFC4122
            || uuid.get_version() != Some(Version::Random)
            || uuid.hyphenated().to_string() != uuid_text
        {
            return Err(ReplayError::InvalidChain);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn digest(&self) -> String {
        digest_bytes(self.0.as_bytes())
    }

    pub fn filename_digest(&self) -> String {
        self.digest()
            .strip_prefix(SHA256_PREFIX)
            .expect("execution digest always carries prefix")
            .to_owned()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayState {
    ClaimedNoState,
    IntentRecorded,
    InvocationArmed,
    Succeeded,
    Failed,
    Uncertain,
}

#[derive(Serialize)]
struct ClaimUnsigned<'a> {
    record_kind: &'static str,
    ledger_format_version: u32,
    logical_key_digest: &'a str,
    execution_id: &'a str,
    execution_id_digest: &'a str,
    binding: &'a ExecutionBinding,
    binding_digest: &'a str,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClaimRecord {
    record_kind: String,
    ledger_format_version: u32,
    logical_key_digest: String,
    execution_id: String,
    execution_id_digest: String,
    binding: ExecutionBinding,
    binding_digest: String,
    claim_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claim {
    pub logical_key: LogicalExecutionKey,
    pub execution_id: ExecutionId,
    pub binding: ExecutionBinding,
    pub binding_digest: String,
    pub claim_digest: String,
}

impl Claim {
    pub fn new(
        logical_key: LogicalExecutionKey,
        execution_id: ExecutionId,
        binding: ExecutionBinding,
    ) -> Result<Self, ReplayError> {
        let binding_digest = binding.digest()?;
        let execution_id_digest = execution_id.digest();
        let claim_digest = canonical_digest(&ClaimUnsigned {
            record_kind: IDENTITY_CLAIM_KIND,
            ledger_format_version: LEDGER_FORMAT_VERSION,
            logical_key_digest: logical_key.as_digest(),
            execution_id: execution_id.as_str(),
            execution_id_digest: &execution_id_digest,
            binding: &binding,
            binding_digest: &binding_digest,
        })?;
        Ok(Self {
            logical_key,
            execution_id,
            binding,
            binding_digest,
            claim_digest,
        })
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ReplayError> {
        let execution_id_digest = self.execution_id.digest();
        canonical_bytes(&json!({
            "record_kind": IDENTITY_CLAIM_KIND,
            "ledger_format_version": LEDGER_FORMAT_VERSION,
            "logical_key_digest": self.logical_key.as_digest(),
            "execution_id": self.execution_id.as_str(),
            "execution_id_digest": execution_id_digest,
            "binding": self.binding,
            "binding_digest": self.binding_digest,
            "claim_digest": self.claim_digest,
        }))
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        expected_logical_key: &LogicalExecutionKey,
    ) -> Result<Self, ReplayError> {
        let record: ClaimRecord = parse_canonical(bytes)?;
        if record.record_kind != IDENTITY_CLAIM_KIND
            || record.ledger_format_version != LEDGER_FORMAT_VERSION
            || record.logical_key_digest != expected_logical_key.as_digest()
        {
            return Err(ReplayError::InvalidChain);
        }
        let execution_id = ExecutionId::parse(record.execution_id)?;
        let binding_digest = record.binding.digest()?;
        if record.execution_id_digest != execution_id.digest()
            || record.binding_digest != binding_digest
        {
            return Err(ReplayError::InvalidChain);
        }
        let expected_claim_digest = canonical_digest(&ClaimUnsigned {
            record_kind: IDENTITY_CLAIM_KIND,
            ledger_format_version: LEDGER_FORMAT_VERSION,
            logical_key_digest: expected_logical_key.as_digest(),
            execution_id: execution_id.as_str(),
            execution_id_digest: &record.execution_id_digest,
            binding: &record.binding,
            binding_digest: &record.binding_digest,
        })?;
        if record.claim_digest != expected_claim_digest {
            return Err(ReplayError::InvalidChain);
        }
        Ok(Self {
            logical_key: expected_logical_key.clone(),
            execution_id,
            binding: record.binding,
            binding_digest: record.binding_digest,
            claim_digest: record.claim_digest,
        })
    }

    pub fn require_binding(&self, expected: &ExecutionBinding) -> Result<(), ReplayError> {
        if &self.binding != expected || self.binding_digest != expected.digest()? {
            Err(ReplayError::BindingMismatch)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TerminalStateData {
    durable_outcome_digest: String,
}

#[derive(Serialize)]
struct GenerationUnsigned<'a> {
    record_kind: &'static str,
    ledger_format_version: u32,
    record_version: u32,
    logical_key_digest: &'a str,
    execution_id_digest: &'a str,
    generation: u64,
    state: ReplayState,
    predecessor_digest: &'a str,
    state_data: &'a Value,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GenerationRecord {
    record_kind: String,
    ledger_format_version: u32,
    record_version: u32,
    logical_key_digest: String,
    execution_id_digest: String,
    generation: u64,
    state: ReplayState,
    predecessor_digest: String,
    state_data: Value,
    record_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Generation {
    pub logical_key_digest: String,
    pub execution_id_digest: String,
    pub number: u64,
    pub state: ReplayState,
    pub predecessor_digest: String,
    pub durable_outcome_digest: Option<String>,
    pub record_digest: String,
}

impl Generation {
    pub fn intent(claim: &Claim) -> Result<Self, ReplayError> {
        Self::new(
            claim,
            0,
            ReplayState::IntentRecorded,
            &claim.claim_digest,
            None,
        )
    }

    pub fn armed(claim: &Claim, predecessor: &Generation) -> Result<Self, ReplayError> {
        Self::new(
            claim,
            1,
            ReplayState::InvocationArmed,
            &predecessor.record_digest,
            None,
        )
    }

    pub fn terminal(
        claim: &Claim,
        predecessor: &Generation,
        state: ReplayState,
        durable_outcome_digest: String,
    ) -> Result<Self, ReplayError> {
        Self::new(
            claim,
            2,
            state,
            &predecessor.record_digest,
            Some(durable_outcome_digest),
        )
    }

    fn new(
        claim: &Claim,
        number: u64,
        state: ReplayState,
        predecessor_digest: &str,
        durable_outcome_digest: Option<String>,
    ) -> Result<Self, ReplayError> {
        let state_data = state_data(state, durable_outcome_digest.as_deref())?;
        validate_digest(predecessor_digest)?;
        let execution_id_digest = claim.execution_id.digest();
        let record_digest = canonical_digest(&GenerationUnsigned {
            record_kind: REPLAY_GENERATION_KIND,
            ledger_format_version: LEDGER_FORMAT_VERSION,
            record_version: GENERATION_RECORD_VERSION,
            logical_key_digest: claim.logical_key.as_digest(),
            execution_id_digest: &execution_id_digest,
            generation: number,
            state,
            predecessor_digest,
            state_data: &state_data,
        })?;
        Ok(Self {
            logical_key_digest: claim.logical_key.as_digest().to_owned(),
            execution_id_digest,
            number,
            state,
            predecessor_digest: predecessor_digest.to_owned(),
            durable_outcome_digest,
            record_digest,
        })
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ReplayError> {
        let state_data = state_data(self.state, self.durable_outcome_digest.as_deref())?;
        canonical_bytes(&json!({
            "record_kind": REPLAY_GENERATION_KIND,
            "ledger_format_version": LEDGER_FORMAT_VERSION,
            "record_version": GENERATION_RECORD_VERSION,
            "logical_key_digest": self.logical_key_digest,
            "execution_id_digest": self.execution_id_digest,
            "generation": self.number,
            "state": self.state,
            "predecessor_digest": self.predecessor_digest,
            "state_data": state_data,
            "record_digest": self.record_digest,
        }))
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, ReplayError> {
        let record: GenerationRecord = parse_canonical(bytes)?;
        if record.record_kind != REPLAY_GENERATION_KIND
            || record.ledger_format_version != LEDGER_FORMAT_VERSION
            || record.record_version != GENERATION_RECORD_VERSION
            || record.generation > 2
        {
            return Err(ReplayError::InvalidChain);
        }
        validate_digest(&record.logical_key_digest)?;
        validate_digest(&record.execution_id_digest)?;
        validate_digest(&record.predecessor_digest)?;
        let durable_outcome_digest = parse_state_data(record.state, record.state_data)?;
        let state_data = state_data(record.state, durable_outcome_digest.as_deref())?;
        let expected_digest = canonical_digest(&GenerationUnsigned {
            record_kind: REPLAY_GENERATION_KIND,
            ledger_format_version: LEDGER_FORMAT_VERSION,
            record_version: GENERATION_RECORD_VERSION,
            logical_key_digest: &record.logical_key_digest,
            execution_id_digest: &record.execution_id_digest,
            generation: record.generation,
            state: record.state,
            predecessor_digest: &record.predecessor_digest,
            state_data: &state_data,
        })?;
        if record.record_digest != expected_digest {
            return Err(ReplayError::InvalidChain);
        }
        Ok(Self {
            logical_key_digest: record.logical_key_digest,
            execution_id_digest: record.execution_id_digest,
            number: record.generation,
            state: record.state,
            predecessor_digest: record.predecessor_digest,
            durable_outcome_digest,
            record_digest: record.record_digest,
        })
    }
}

fn state_data(state: ReplayState, outcome_digest: Option<&str>) -> Result<Value, ReplayError> {
    match (state, outcome_digest) {
        (ReplayState::IntentRecorded | ReplayState::InvocationArmed, None) => Ok(json!({})),
        (ReplayState::Succeeded | ReplayState::Failed | ReplayState::Uncertain, Some(digest)) => {
            validate_digest(digest)?;
            Ok(json!({"durable_outcome_digest": digest}))
        }
        _ => Err(ReplayError::InvalidChain),
    }
}

fn parse_state_data(state: ReplayState, value: Value) -> Result<Option<String>, ReplayError> {
    match state {
        ReplayState::IntentRecorded | ReplayState::InvocationArmed => {
            if value == json!({}) {
                Ok(None)
            } else {
                Err(ReplayError::InvalidChain)
            }
        }
        ReplayState::Succeeded | ReplayState::Failed | ReplayState::Uncertain => {
            let data: TerminalStateData =
                serde_json::from_value(value).map_err(|_| ReplayError::InvalidChain)?;
            validate_digest(&data.durable_outcome_digest)?;
            Ok(Some(data.durable_outcome_digest))
        }
        ReplayState::ClaimedNoState => Err(ReplayError::InvalidChain),
    }
}

pub fn validate_chain(
    claim: &Claim,
    generations: &[Generation],
) -> Result<ReplayState, ReplayError> {
    if generations.len() > 3 {
        return Err(ReplayError::InvalidChain);
    }
    let mut predecessor = claim.claim_digest.clone();
    let mut state = ReplayState::ClaimedNoState;
    for (expected, generation) in generations.iter().enumerate() {
        if generation.number != expected as u64
            || generation.logical_key_digest != claim.logical_key.as_digest()
            || generation.execution_id_digest != claim.execution_id.digest()
            || generation.predecessor_digest != predecessor
        {
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
        predecessor.clone_from(&generation.record_digest);
        state = generation.state;
    }
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(label: &str) -> String {
        digest_bytes(label.as_bytes())
    }

    fn binding(action: &str) -> ExecutionBinding {
        ExecutionBinding {
            evaluation_id: "eval".into(),
            action_id: action.into(),
            capability_name: "cap".into(),
            capability_version: 1,
            manifest_digest: digest("manifest"),
            provider_identity: "provider".into(),
            argument_digest: digest("arguments"),
        }
    }

    fn claim() -> Claim {
        Claim::new(
            LogicalExecutionKey::derive("event", "eval", "a").unwrap(),
            ExecutionId::generate(),
            binding("a"),
        )
        .unwrap()
    }

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
    fn different_evaluations_are_distinct() {
        assert_ne!(
            LogicalExecutionKey::derive("event", "eval-1", "action")
                .unwrap()
                .as_digest(),
            LogicalExecutionKey::derive("event", "eval-2", "action")
                .unwrap()
                .as_digest()
        );
    }

    #[test]
    fn claim_round_trip_is_exact_canonical_and_redacted() {
        let claim = claim();
        let bytes = claim.canonical_bytes().unwrap();
        let recovered = Claim::from_canonical_bytes(&bytes, &claim.logical_key).unwrap();
        assert_eq!(recovered, claim);
        assert!(!String::from_utf8(bytes).unwrap().contains("raw_argument"));
    }

    #[test]
    fn substituted_execution_identity_is_rejected() {
        let claim = claim();
        let bytes = claim.canonical_bytes().unwrap();
        let mut value: Value = serde_json::from_slice(&bytes).unwrap();
        value.as_object_mut().unwrap().insert(
            "execution_id".into(),
            Value::String("exec_00000000-0000-4000-8000-000000000000".into()),
        );
        let bytes = serde_json_canonicalizer::to_vec(&value).unwrap();
        assert!(Claim::from_canonical_bytes(&bytes, &claim.logical_key).is_err());
    }

    #[test]
    fn non_canonical_or_unknown_claim_is_rejected() {
        let claim = claim();
        let canonical = String::from_utf8(claim.canonical_bytes().unwrap()).unwrap();
        let spaced = canonical.replacen('{', "{ ", 1);
        assert!(Claim::from_canonical_bytes(spaced.as_bytes(), &claim.logical_key).is_err());
        let unknown = canonical.replacen('{', "{\"unknown\":true,", 1);
        assert!(Claim::from_canonical_bytes(unknown.as_bytes(), &claim.logical_key).is_err());
    }

    #[test]
    fn chain_cannot_skip_armed() {
        let claim = claim();
        let invalid = Generation::terminal(
            &claim,
            &Generation::intent(&claim).unwrap(),
            ReplayState::Succeeded,
            digest("outcome"),
        )
        .unwrap();
        assert!(validate_chain(&claim, &[invalid]).is_err());
    }

    #[test]
    fn generation_three_is_not_representable_or_parseable() {
        let claim = claim();
        let generation = Generation::intent(&claim).unwrap();
        let bytes = generation.canonical_bytes().unwrap();
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        let mut value = value.as_object().unwrap().clone();
        value.insert("generation".into(), json!(3));
        let bytes = serde_json_canonicalizer::to_vec(&Value::Object(value)).unwrap();
        assert!(Generation::from_canonical_bytes(&bytes).is_err());
    }
}
