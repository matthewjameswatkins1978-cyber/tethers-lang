// result_anchor.rs — Result Anchor event envelope
//
// The host creates exactly one Result Anchor after an executor call produces
// a known outcome.  No Result Anchor is created when the Action was never
// attempted (Ask, Deny, Unavailable, identity mismatch, or intent-write
// failure).
//
// ---------------------------------------------------------------------------
// Envelope
// ---------------------------------------------------------------------------
//
// Every Result Anchor contains:
//
//   event_id       — stable: "<evaluation_id>/<action_id>/result"
//   event_name     — "capability.succeeded" or "capability.failed"
//   producer       — "tethers-reference-host"
//   correlation_id — the original input event ID
//   causation_id   — the original input event ID
//   generation     — 1
//   occurred_at    — host-supplied Unix-millisecond timestamp
//   facts          — evaluation_id, action_id, capability (name/version),
//                    manifest digest, provider identity, and either a
//                    validated result or structured error
//
// Failure classification:
//   - Executor error:           error code "provider_error"
//   - Output-validation failure: error code "result_validation_failed"

use serde::Serialize;

// ---------------------------------------------------------------------------
// Result Anchor kind — captures the known outcome
// ---------------------------------------------------------------------------

/// The known outcome of a completed executor call.
pub enum ResultAnchorKind {
    /// Valid successful output.
    Succeeded(serde_json::Value),
    /// Executor returned an error.
    ProviderError(String),
    /// Executor returned output that failed validation.
    ResultValidationFailed(String),
    /// A provider invocation may have begun but no trustworthy final evidence
    /// was observed before the host deadline.
    Uncertain { code: String, message: String },
    /// J06's redacted known-failure representation.
    Failed { code: String, message: String },
}

// ---------------------------------------------------------------------------
// Result Anchor envelope
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct ResultAnchor {
    pub event_id: String,
    pub event_name: String,
    pub producer: String,
    pub correlation_id: String,
    pub causation_id: String,
    pub generation: u32,
    pub occurred_at: u64,
    pub facts: ResultAnchorFacts,
}

/// Capability identity nested inside the Facts.
///
/// Uses the canonical `capability.name` / `capability.version` path
/// so a follow-up Tether can inspect `capability.name` directly.
#[derive(Debug, Serialize)]
pub struct ResultAnchorCapability {
    /// Resolved capability name, taken from DispatchReadyAction.
    pub name: String,
    /// Resolved capability version, taken from DispatchReadyAction.
    pub version: u32,
}

#[derive(Debug, Serialize)]
pub struct ResultAnchorFacts {
    pub evaluation_id: String,
    pub action_id: String,

    /// Resolved capability identity (nested).
    pub capability: ResultAnchorCapability,

    /// Verified manifest digest, taken from DispatchReadyAction.
    pub manifest_digest: String,
    /// Resolved provider identity, taken from DispatchReadyAction.
    pub provider_identity: String,

    /// Present for `capability.succeeded`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,

    /// Present for `capability.failed`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResultAnchorError>,
}

#[derive(Debug, Serialize)]
pub struct ResultAnchorError {
    pub code: String,
    pub message: String,
}

// ---------------------------------------------------------------------------
// Constructor
// ---------------------------------------------------------------------------

impl ResultAnchor {
    /// Build a Result Anchor from a known outcome.
    ///
    /// `original_event_id` is the original input request event ID,
    /// used as both `correlation_id` and `causation_id`.
    pub fn new(
        kind: ResultAnchorKind,
        evaluation_id: &str,
        action_id: &str,
        capability_name: &str,
        capability_version: u32,
        manifest_digest: &str,
        provider_identity: &str,
        occurred_at: u64,
        original_event_id: &str,
    ) -> Self {
        let (event_name, result, error) = match kind {
            ResultAnchorKind::Succeeded(value) => {
                ("capability.succeeded".to_string(), Some(value), None)
            }
            ResultAnchorKind::ProviderError(message) => (
                "capability.failed".to_string(),
                None,
                Some(ResultAnchorError {
                    code: "provider_error".to_string(),
                    message,
                }),
            ),
            ResultAnchorKind::ResultValidationFailed(message) => (
                "capability.failed".to_string(),
                None,
                Some(ResultAnchorError {
                    code: "result_validation_failed".to_string(),
                    message,
                }),
            ),
            ResultAnchorKind::Uncertain { code, message } => (
                "capability.uncertain".to_string(),
                None,
                Some(ResultAnchorError { code, message }),
            ),
            ResultAnchorKind::Failed { code, message } => (
                "capability.failed".to_string(),
                None,
                Some(ResultAnchorError { code, message }),
            ),
        };

        ResultAnchor {
            event_id: format!("{evaluation_id}/{action_id}/result"),
            event_name,
            producer: "tethers-reference-host".to_string(),
            correlation_id: original_event_id.to_string(),
            causation_id: original_event_id.to_string(),
            generation: 1,
            occurred_at,
            facts: ResultAnchorFacts {
                evaluation_id: evaluation_id.to_string(),
                action_id: action_id.to_string(),
                capability: ResultAnchorCapability {
                    name: capability_name.to_string(),
                    version: capability_version,
                },
                manifest_digest: manifest_digest.to_string(),
                provider_identity: provider_identity.to_string(),
                result,
                error,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn succeeded_anchor_has_correct_identities_and_nested_capability() {
        let anchor = ResultAnchor::new(
            ResultAnchorKind::Succeeded(json!({"status": "recorded"})),
            "eval_001",
            "action_1",
            "lantern.task.record",
            1,
            "sha256:abc123",
            "lantern-local",
            1720000000000,
            "evt_input_001",
        );

        assert_eq!(anchor.event_id, "eval_001/action_1/result");
        assert_eq!(anchor.event_name, "capability.succeeded");
        assert_eq!(anchor.producer, "tethers-reference-host");
        assert_eq!(anchor.correlation_id, "evt_input_001");
        assert_eq!(anchor.causation_id, "evt_input_001");
        assert_eq!(anchor.generation, 1);
        assert_eq!(anchor.occurred_at, 1720000000000);

        assert_eq!(anchor.facts.evaluation_id, "eval_001");
        assert_eq!(anchor.facts.action_id, "action_1");
        assert_eq!(anchor.facts.capability.name, "lantern.task.record");
        assert_eq!(anchor.facts.capability.version, 1);
        assert_eq!(anchor.facts.manifest_digest, "sha256:abc123");
        assert_eq!(anchor.facts.provider_identity, "lantern-local");

        assert!(anchor.facts.result.is_some());
        assert_eq!(anchor.facts.result.unwrap(), json!({"status": "recorded"}));
        assert!(anchor.facts.error.is_none());
    }

    #[test]
    fn serialized_succeeded_uses_nested_capability_name_path() {
        let anchor = ResultAnchor::new(
            ResultAnchorKind::Succeeded(json!({"status": "recorded"})),
            "eval_001",
            "action_1",
            "lantern.task.record",
            1,
            "sha256:abc123",
            "lantern-local",
            1720000000000,
            "evt_input_001",
        );

        let value = serde_json::to_value(&anchor).unwrap();
        let facts = value.get("facts").expect("missing facts");
        assert_eq!(
            facts
                .get("capability")
                .and_then(|c| c.get("name"))
                .and_then(|v| v.as_str()),
            Some("lantern.task.record")
        );
        assert_eq!(
            facts
                .get("capability")
                .and_then(|c| c.get("version"))
                .and_then(|v| v.as_u64()),
            Some(1)
        );
        // Flat fields must not exist
        assert!(
            facts.get("capability_name").is_none(),
            "flat capability_name must be absent"
        );
        assert!(
            facts.get("capability_version").is_none(),
            "flat capability_version must be absent"
        );
        // error field must be absent in success anchor
        assert!(
            facts
                .as_object()
                .map(|m| !m.contains_key("error"))
                .unwrap_or(false),
            "error key should be absent in success anchor"
        );
    }

    #[test]
    fn provider_error_anchor_has_error_code_and_nested_capability() {
        let anchor = ResultAnchor::new(
            ResultAnchorKind::ProviderError("executor failed as requested".to_string()),
            "eval_001",
            "action_1",
            "lantern.task.record",
            1,
            "sha256:abc123",
            "lantern-local",
            1720000000000,
            "evt_input_001",
        );

        assert_eq!(anchor.event_name, "capability.failed");
        assert_eq!(anchor.facts.capability.name, "lantern.task.record");
        assert_eq!(anchor.facts.capability.version, 1);
        assert!(anchor.facts.result.is_none());
        assert!(anchor.facts.error.is_some());
        let error = anchor.facts.error.unwrap();
        assert_eq!(error.code, "provider_error");
        assert_eq!(error.message, "executor failed as requested");
    }

    #[test]
    fn result_validation_failed_anchor_has_error_code_and_no_result() {
        let anchor = ResultAnchor::new(
            ResultAnchorKind::ResultValidationFailed(
                "output validation failed: missing required field 'status'".to_string(),
            ),
            "eval_001",
            "action_1",
            "lantern.task.record",
            1,
            "sha256:abc123",
            "lantern-local",
            1720000000000,
            "evt_input_001",
        );

        assert_eq!(anchor.event_name, "capability.failed");
        assert!(anchor.facts.result.is_none());
        assert!(anchor.facts.error.is_some());
        let error = anchor.facts.error.unwrap();
        assert_eq!(error.code, "result_validation_failed");
        assert!(
            error.message.contains("output validation failed"),
            "expected validation message, got: {}",
            error.message
        );
    }

    #[test]
    fn serialized_failed_anchor_omits_result_field() {
        let anchor = ResultAnchor::new(
            ResultAnchorKind::ProviderError("boom".to_string()),
            "eval_001",
            "action_1",
            "lantern.task.record",
            1,
            "sha256:abc123",
            "lantern-local",
            1720000000000,
            "evt_input_001",
        );

        let value = serde_json::to_value(&anchor).unwrap();
        // result field must be absent, not null
        assert!(
            value
                .get("facts")
                .and_then(|f| f.as_object())
                .map(|m| !m.contains_key("result"))
                .unwrap_or(false),
            "result key should be absent in failure anchor"
        );
    }
}
