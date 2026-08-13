// MCP engine session manager for the OCaml Tethers MCP engine.
//
// Uses tools/call with name "tethers.validate" for Tether validation and
// "tethers.evaluate" for Tethers request evaluation via a retained session.

use crate::child_process::{ChildConfig, ChildError, SupervisedChild};
use serde_json::Value;
use std::fmt;
use std::path::Path;
use std::time::Duration;

const ENGINE_INITIALIZE_ID: u64 = 1;
const VALIDATION_REQUEST_BASE_ID: u64 = 100;
const DEFAULT_ENGINE_READ_TIMEOUT: Duration = Duration::from_secs(10);

/// Typed wire classification from the OCaml Tethers planner.
///
/// Classifies known planner statuses and preserves an unknown string
/// status for host-level semantic rejection.
#[derive(Debug)]
pub enum PlannerResponseWire {
    Matched(Value),
    NotMatched(Value),
    Error(Value),
    Unknown { status: String, response: Value },
}

/// Errors from engine session operations.
#[derive(Debug)]
pub enum EngineError {
    Child(ChildError),
    ProtocolError(String),
    InitializeFailed(String),
    ValidationFailed {
        tether_index: usize,
        tether_id: String,
        tether_version: String,
        error_code: String,
        message: String,
    },
    EvaluationFailed {
        evaluation_id: String,
        message: String,
    },
    SerializeFailed(String),
    Interrupted,
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Child(e) => write!(f, "engine child error: {e}"),
            Self::ProtocolError(msg) => write!(f, "engine protocol error: {msg}"),
            Self::InitializeFailed(msg) => write!(f, "engine initialize failed: {msg}"),
            Self::ValidationFailed {
                tether_index,
                tether_id,
                tether_version,
                error_code,
                message,
            } => write!(
                f,
                "validation failed for tether {tether_index} ({tether_id} v{tether_version}): [{error_code}] {message}"
            ),
            Self::EvaluationFailed {
                evaluation_id,
                message,
            } => write!(f, "evaluation failed for {evaluation_id}: {message}"),
            Self::SerializeFailed(msg) => write!(f, "serialization failed: {msg}"),
            Self::Interrupted => write!(f, "engine session interrupted"),
        }
    }
}

impl std::error::Error for EngineError {}

impl From<ChildError> for EngineError {
    fn from(e: ChildError) -> Self {
        match e {
            ChildError::Interrupted => EngineError::Interrupted,
            other => EngineError::Child(other),
        }
    }
}

/// Retained MCP engine session using tools/call for Tether validation and
/// evaluation.
pub struct EngineSession {
    child: SupervisedChild,
    next_request_id: u64,
    read_timeout: Duration,
}

impl EngineSession {
    /// Launch the engine and perform MCP initialize handshake.
    pub fn launch(engine_path: &Path, working_dir: &Path) -> Result<Self, EngineError> {
        let config = ChildConfig {
            command: engine_path.to_string_lossy().into_owned(),
            args: Vec::new(),
            current_dir: Some(working_dir.to_path_buf()),
            ..ChildConfig::default()
        };

        let mut child = SupervisedChild::launch(config)?;

        // MCP initialize.
        let init_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": ENGINE_INITIALIZE_ID,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {
                    "name": "tethers-reference-host",
                    "version": "0.2.0"
                }
            }
        });

        Self::write_json(&mut child, &init_request)?;
        let init_response = Self::read_json(
            &mut child,
            ENGINE_INITIALIZE_ID,
            "initialize",
            DEFAULT_ENGINE_READ_TIMEOUT,
        )?;

        let version = init_response
            .get("protocolVersion")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                EngineError::InitializeFailed(
                    "initialize response missing protocolVersion".to_owned(),
                )
            })?;
        if version != "2025-11-25" {
            return Err(EngineError::InitializeFailed(format!(
                "engine selected incompatible protocol version: {version}"
            )));
        }

        // Verify engine advertises tools capability (needed for tools/call).
        if !init_response
            .pointer("/capabilities/tools")
            .is_some_and(|v| v.is_object())
        {
            return Err(EngineError::InitializeFailed(
                "engine did not advertise tools capability".to_owned(),
            ));
        }

        // Send initialized notification.
        let init_notify = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {}
        });
        Self::write_json(&mut child, &init_notify)?;

        Ok(Self {
            child,
            next_request_id: VALIDATION_REQUEST_BASE_ID,
            read_timeout: Duration::from_secs(10),
        })
    }

    /// Validate one Tether source via tools/call.
    pub fn validate_tether(
        &mut self,
        index: usize,
        id: &str,
        version: &str,
        source: &str,
    ) -> Result<(), EngineError> {
        let request_id = self.next_request_id;
        self.next_request_id += 1;

        // Use tools/call with name "tethers.validate".
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "tools/call",
            "params": {
                "name": "tethers.validate",
                "arguments": {
                    "source": source
                }
            }
        });

        Self::write_json(&mut self.child, &request)?;
        let result = Self::read_json(&mut self.child, request_id, "tools/call", self.read_timeout)?;

        // Parse result.structuredContent.valid.
        let structured = result.get("structuredContent").ok_or_else(|| {
            EngineError::ProtocolError("tools/call result missing structuredContent".to_owned())
        })?;

        let valid = structured
            .get("valid")
            .and_then(Value::as_bool)
            .ok_or_else(|| {
                EngineError::ProtocolError("structuredContent missing valid field".to_owned())
            })?;

        if valid {
            Ok(())
        } else {
            let error_code = structured
                .pointer("/error/code")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_owned();
            let message = structured
                .pointer("/error/message")
                .and_then(Value::as_str)
                .unwrap_or("validation failed")
                .to_owned();
            Err(EngineError::ValidationFailed {
                tether_index: index,
                tether_id: id.to_owned(),
                tether_version: version.to_owned(),
                error_code,
                message,
            })
        }
    }

    /// Evaluate a fully-formed Tethers 0.1 request through the retained engine.
    ///
    /// Sends `tools/call` with `tethers.evaluate` and the complete request
    /// envelope.  Returns a typed wire classification; a Tethers response
    /// with `status: "error"` is valid planner data and is classified as
    /// `PlannerResponseWire::Error`, not an engine transport failure.
    pub fn evaluate_tether(
        &mut self,
        evaluation_id: &str,
        request_envelope: &Value,
    ) -> Result<PlannerResponseWire, EngineError> {
        let request_id = self.next_request_id;
        self.next_request_id += 1;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "tools/call",
            "params": {
                "name": "tethers.evaluate",
                "arguments": {
                    "request": request_envelope
                }
            }
        });

        Self::write_json(&mut self.child, &request)?;
        let result = Self::read_json(&mut self.child, request_id, "tools/call", self.read_timeout)?;

        let structured = result.get("structuredContent").cloned().ok_or_else(|| {
            EngineError::ProtocolError("tools/call result missing structuredContent".to_owned())
        })?;

        classify_wire_response(evaluation_id, structured)
    }

    /// List available tools via tools/list.
    pub fn list_tools(&mut self) -> Result<Vec<Value>, EngineError> {
        let request_id = self.next_request_id;
        self.next_request_id += 1;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "tools/list",
            "params": {}
        });

        Self::write_json(&mut self.child, &request)?;
        let result = Self::read_json(&mut self.child, request_id, "tools/list", self.read_timeout)?;

        let tools = result
            .get("tools")
            .and_then(Value::as_array)
            .cloned()
            .ok_or_else(|| {
                EngineError::ProtocolError("tools/list result missing tools array".to_owned())
            })?;

        Ok(tools)
    }

    pub fn stderr_tail(&self) -> String {
        self.child.stderr_tail()
    }

    pub fn shutdown(self) {
        self.child.shutdown();
    }

    // --- Private helpers ---

    fn write_json(child: &mut SupervisedChild, msg: &Value) -> Result<(), EngineError> {
        let line =
            serde_json::to_string(msg).map_err(|e| EngineError::SerializeFailed(e.to_string()))?;
        child.write_line(&line).map_err(EngineError::from)
    }

    fn read_json(
        child: &mut SupervisedChild,
        expected_id: u64,
        method: &str,
        timeout: Duration,
    ) -> Result<Value, EngineError> {
        let line = child.read_protocol_line(timeout)?;
        let line = line.trim();
        if line.is_empty() {
            return Err(EngineError::ProtocolError(
                "empty engine response".to_owned(),
            ));
        }

        let response: Value = serde_json::from_str(line)
            .map_err(|e| EngineError::ProtocolError(format!("malformed engine JSON: {e}")))?;

        let obj = response.as_object().ok_or_else(|| {
            EngineError::ProtocolError("engine response not a JSON object".to_owned())
        })?;

        if obj.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
            return Err(EngineError::ProtocolError(
                "engine response missing jsonrpc 2.0".to_owned(),
            ));
        }

        if let Some(error) = obj.get("error") {
            let msg = error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("unknown error");
            return Err(EngineError::ProtocolError(format!(
                "engine returned error for {method}: {msg}"
            )));
        }

        match obj.get("id") {
            Some(Value::Number(n)) if n.as_u64() == Some(expected_id) => {}
            Some(other) => {
                return Err(EngineError::ProtocolError(format!(
                    "engine response id mismatch: expected {expected_id}, got {other}"
                )));
            }
            None => {
                return Err(EngineError::ProtocolError(
                    "engine response missing id".to_owned(),
                ));
            }
        }

        Ok(obj
            .get("result")
            .cloned()
            .unwrap_or(Value::Object(Default::default())))
    }
}

fn classify_wire_response(
    evaluation_id: &str,
    response: Value,
) -> Result<PlannerResponseWire, EngineError> {
    match response.get("status").and_then(Value::as_str) {
        Some("matched") => Ok(PlannerResponseWire::Matched(response)),
        Some("not_matched") => Ok(PlannerResponseWire::NotMatched(response)),
        Some("error") => Ok(PlannerResponseWire::Error(response)),
        Some(other) => Ok(PlannerResponseWire::Unknown {
            status: other.to_owned(),
            response,
        }),
        None => Err(EngineError::EvaluationFailed {
            evaluation_id: evaluation_id.to_owned(),
            message: "Tethers response missing status field".to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const VALID_TETHER: &str = "tether \"Test tether\"\n\nanchor\n    coding.task_completed\n\nwhen\n    project.type is \"software\"\n\ndo\n    lantern.task.record\n        project: anchor.project\n";

    fn engine_binary_path() -> Option<PathBuf> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.pop();
        path.push("engine-ocaml");
        path.push("_build");
        path.push("default");
        path.push("bin");
        path.push("tethers_mcp_main.exe");
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    fn require_engine() -> (PathBuf, PathBuf) {
        let ep = engine_binary_path()
            .expect("engine binary not found; build with opam exec -- dune build");
        let wd = ep.parent().unwrap().to_path_buf();
        (ep, wd)
    }

    #[test]
    fn j13a_real_engine_valid_tether_via_tools_call() {
        let (engine_path, working_dir) = require_engine();
        let mut session = EngineSession::launch(&engine_path, &working_dir).expect("engine launch");
        let result = session.validate_tether(0, "test.tether", "1.0.0", VALID_TETHER);
        assert!(
            result.is_ok(),
            "valid tether should pass: {:?}",
            result.err()
        );
        session.shutdown();
    }

    #[test]
    fn j13a_real_engine_invalid_tether_via_tools_call() {
        let (engine_path, working_dir) = require_engine();
        let mut session = EngineSession::launch(&engine_path, &working_dir).expect("engine launch");
        let result = session.validate_tether(0, "bad.tether", "1.0.0", "garbage syntax {{{");
        match result {
            Err(EngineError::ValidationFailed { .. }) => {}
            Ok(()) => panic!("invalid tether should fail"),
            Err(e) => panic!("expected ValidationFailed, got {e:?}"),
        }
        session.shutdown();
    }

    #[test]
    fn j13a_engine_one_retained_session_multiple_tethers() {
        let (engine_path, working_dir) = require_engine();
        let mut session = EngineSession::launch(&engine_path, &working_dir).expect("engine launch");
        assert!(session
            .validate_tether(0, "t1", "1.0.0", VALID_TETHER)
            .is_ok());
        assert!(session
            .validate_tether(1, "t2", "1.0.0", VALID_TETHER)
            .is_ok());
        session.shutdown();
    }

    #[test]
    fn j13a_engine_no_tethers_evaluate_sent() {
        let (engine_path, working_dir) = require_engine();
        let mut session = EngineSession::launch(&engine_path, &working_dir).expect("engine launch");
        assert!(session
            .validate_tether(0, "t", "1.0.0", VALID_TETHER)
            .is_ok());
        session.shutdown();
    }

    #[test]
    fn j13b_retained_engine_uses_arguments_request_for_multiple_evaluations() {
        let (engine_path, working_dir) = require_engine();
        let mut session = EngineSession::launch(&engine_path, &working_dir).expect("engine launch");
        let request = core9b_valid_request("eval_j13b_real_001", "fixture.start");
        let wire = session
            .evaluate_tether("eval_j13b_real_001", &request)
            .expect("real retained tethers.evaluate call");
        let PlannerResponseWire::Matched(response) = wire else {
            panic!("expected Matched wire, got {wire:?}");
        };
        assert_eq!(response["evaluation_id"], "eval_j13b_real_001");
        assert_eq!(response["event_id"], "evt_eval_j13b_real_001");
        assert_eq!(response["status"], "matched");

        let second_request = core9b_valid_request("eval_j13b_real_002", "fixture.start");
        let wire = session
            .evaluate_tether("eval_j13b_real_002", &second_request)
            .expect("second real retained tethers.evaluate call");
        let PlannerResponseWire::Matched(second) = wire else {
            panic!("expected Matched wire, got {wire:?}");
        };
        assert_eq!(second["evaluation_id"], "eval_j13b_real_002");
        assert_eq!(second["event_id"], "evt_eval_j13b_real_002");
        assert_eq!(second["status"], "matched");
        session.shutdown();
    }

    #[test]
    fn j13b_wire_missing_or_non_string_status_is_engine_error() {
        let missing = serde_json::json!({"protocol_version": "0.1", "evaluation_id": "eval-1"});
        let result = classify_wire_response("eval-1", missing);
        assert!(matches!(result, Err(EngineError::EvaluationFailed { .. })));

        let non_string = serde_json::json!({"status": 42, "protocol_version": "0.1"});
        let result = classify_wire_response("eval-1", non_string);
        assert!(matches!(result, Err(EngineError::EvaluationFailed { .. })));

        let unknown_string = serde_json::json!({"status": "completed", "evaluation_id": "eval-1"});
        let result = classify_wire_response("eval-1", unknown_string);
        assert!(matches!(result, Ok(PlannerResponseWire::Unknown { .. })));
    }

    // =====================================================================
    // CORE-9B cross-language rehearsal tests (T4–T14)
    // =====================================================================

    const CORE_REHEARSAL_TETHER: &str = "tether \"core rehearsal\"\n\nanchor\n    fixture.start\n\nwhen\n\ndo\n    notify\n        message: anchor.message\n";

    /// Build a valid CORE-8B extended request matching the OCaml wire test
    /// fixture.  Capabilities include bridge metadata for the real
    /// fixture-ping provider.
    fn core9b_valid_request(evaluation_id: &str, event_name: &str) -> Value {
        serde_json::json!({
            "protocol_version": "0.1",
            "language_version": "0.1",
            "evaluation_id": evaluation_id,
            "tether": {
                "id": "core-rehearsal",
                "version": "1",
                "source": CORE_REHEARSAL_TETHER
            },
            "event": {
                "id": format!("evt_{evaluation_id}"),
                "name": event_name,
                "data": { "message": "Hello Core" }
            },
            "facts": {},
            "capabilities": [{
                "name": "fixture.ping",
                "version": "1.0.0",
                "inputs": {"message": "string"},
                "effects": ["fixture.test"],
                "reversibility": "compensatable",
                "manifest_digest": "sha256:01fed7a4b877dd82abe91a1b6cfcd476b02e4c115489e70cbb285b8bf2d32d8b",
                "bridge_capability_version": 1,
                "bridge_provider_identity": "tethers-stdio-fixture"
            }],
            "core_environment": {
                "program_id": "program.core9b",
                "core_version": "1",
                "capabilities": [{
                    "source_name": "notify",
                    "capability_id": "cap.semantic.notify",
                    "contract_digest": "CORE-CONTRACT-9B",
                    "runtime_name": "fixture.ping"
                }],
                "input_facts": []
            }
        })
    }

    /// Build a request without core_environment for legacy evaluation.
    fn core9b_legacy_request(evaluation_id: &str, event_name: &str) -> Value {
        serde_json::json!({
            "protocol_version": "0.1",
            "language_version": "0.1",
            "evaluation_id": evaluation_id,
            "tether": {
                "id": "core-rehearsal",
                "version": "1",
                "source": CORE_REHEARSAL_TETHER
            },
            "event": {
                "id": format!("evt_{evaluation_id}"),
                "name": event_name,
                "data": { "message": "Hello Core" }
            },
            "facts": {},
            "capabilities": [{
                "name": "fixture.ping",
                "version": "1.0.0",
                "inputs": {"message": "string"},
                "effects": ["fixture.test"],
                "reversibility": "compensatable"
            }]
        })
    }

    // T4: MCP tools/list contains tethers.validate and tethers.evaluate
    //     only.  No legacy or rehearsal tools remain.
    #[test]
    fn core9b_t4_mcp_tools_list_contains_two() {
        let (engine_path, working_dir) = require_engine();
        let mut session = EngineSession::launch(&engine_path, &working_dir).expect("engine launch");
        let tools = session.list_tools().expect("tools/list");
        let names: Vec<&str> = tools
            .iter()
            .filter_map(|t| t.get("name").and_then(Value::as_str))
            .collect();
        assert!(
            names.contains(&"tethers.validate"),
            "tethers.validate missing: {names:?}"
        );
        assert!(
            names.contains(&"tethers.evaluate"),
            "tethers.evaluate missing: {names:?}"
        );
        assert_eq!(names.len(), 2, "expected exactly 2 tools, got {names:?}");
        session.shutdown();
    }

    // T5: tethers.evaluate now uses the Core pipeline.  A request with
    //     core_environment produces Matched with program_digest.
    #[test]
    fn core9b_t5_evaluate_is_core() {
        let (engine_path, working_dir) = require_engine();
        let mut session = EngineSession::launch(&engine_path, &working_dir).expect("engine launch");
        let request = core9b_valid_request("eval_t5_core", "fixture.start");
        let wire = session
            .evaluate_tether("eval_t5_core", &request)
            .expect("evaluate_tether");
        let PlannerResponseWire::Matched(response) = wire else {
            panic!("T5: expected Matched, got {wire:?}");
        };
        assert_eq!(response["status"], "matched");
        assert_eq!(response["evaluation_id"], "eval_t5_core");
        // program_digest must be top-level
        let pd = response
            .get("program_digest")
            .and_then(Value::as_str)
            .expect("program_digest missing");
        assert!(pd.starts_with("sha256:"));
        session.shutdown();
    }

    // T6: tethers.evaluate with core_environment reaches Core pipeline
    //     and returns Matched with program_digest.
    #[test]
    fn core9b_t6_new_evaluate_reaches_core() {
        let (engine_path, working_dir) = require_engine();
        let mut session = EngineSession::launch(&engine_path, &working_dir).expect("engine launch");
        let request = core9b_valid_request("eval_t6_core", "fixture.start");
        let wire = session
            .evaluate_tether("eval_t6_core", &request)
            .expect("evaluate_tether");
        let PlannerResponseWire::Matched(response) = wire else {
            panic!("T6: expected Matched, got {wire:?}");
        };
        assert_eq!(response["status"], "matched");
        assert_eq!(response["evaluation_id"], "eval_t6_core");
        assert_eq!(response["event_id"], "evt_eval_t6_core");
        // program_digest must be a top-level sibling of plan
        let pd = response
            .get("program_digest")
            .and_then(Value::as_str)
            .expect("program_digest missing from top level");
        assert!(
            pd.starts_with("sha256:"),
            "program_digest must start with sha256:"
        );
        assert_eq!(pd.len(), 71, "program_digest must be sha256: + 64 hex");
        // plan must NOT contain program_digest
        let plan = response.get("plan").expect("plan missing");
        assert!(
            plan.get("program_digest").is_none(),
            "program_digest must NOT be inside plan"
        );
        session.shutdown();
    }

    // T7: tethers.evaluate with core_environment is the single production
    //     route.  Verify it works with a valid extended request.
    #[test]
    fn core9b_t7_core_evaluate_works() {
        let (engine_path, working_dir) = require_engine();
        let mut session = EngineSession::launch(&engine_path, &working_dir).expect("engine launch");
        let request = core9b_valid_request("eval_t7_core_method", "fixture.start");
        let wire = session
            .evaluate_tether("eval_t7_core_method", &request)
            .expect("evaluate_tether");
        match wire {
            PlannerResponseWire::Matched(response) => {
                assert_eq!(response["status"], "matched");
                assert_eq!(response["evaluation_id"], "eval_t7_core_method");
            }
            other => panic!("T7: expected Matched from core evaluate, got {other:?}"),
        }
        session.shutdown();
    }

    // T8: tethers.evaluate with core_environment returns Matched for a
    //     valid extended request.
    #[test]
    fn core9b_t8_core_evaluate_returns_matched() {
        let (engine_path, working_dir) = require_engine();
        let mut session = EngineSession::launch(&engine_path, &working_dir).expect("engine launch");
        let request = core9b_valid_request("eval_t8_core_method", "fixture.start");
        let wire = session
            .evaluate_tether("eval_t8_core_method", &request)
            .expect("evaluate_tether");
        match wire {
            PlannerResponseWire::Matched(response) => {
                assert_eq!(response["status"], "matched");
                assert_eq!(response["evaluation_id"], "eval_t8_core_method");
            }
            other => panic!("T8: expected Matched from core evaluate, got {other:?}"),
        }
        session.shutdown();
    }

    // T9: Core request without core_environment fails with
    //     missing_core_environment through the public tethers.evaluate.
    #[test]
    fn core9b_t9_no_core_environment_fails() {
        let (engine_path, working_dir) = require_engine();
        let mut session = EngineSession::launch(&engine_path, &working_dir).expect("engine launch");
        let request = core9b_legacy_request("eval_t9_no_core", "fixture.start");
        let wire = session
            .evaluate_tether("eval_t9_no_core", &request)
            .expect("evaluate_tether");
        match wire {
            PlannerResponseWire::Error(response) => {
                let code = response
                    .pointer("/error/code")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                assert_eq!(
                    code, "missing_core_environment",
                    "expected missing_core_environment error, got {code}"
                );
            }
            other => panic!("T9: expected Error for missing core_environment, got {other:?}"),
        }
        session.shutdown();
    }

    // T10: Identity separation — source_name, capability_id, contract_digest
    //      are Core identities; runtime_name, capability are runtime
    //      identities.  No derivation between them.
    #[test]
    fn core9b_t10_identity_separation() {
        let (engine_path, working_dir) = require_engine();
        let mut session = EngineSession::launch(&engine_path, &working_dir).expect("engine launch");
        let request = core9b_valid_request("eval_t10_identity", "fixture.start");
        let wire = session
            .evaluate_tether("eval_t10_identity", &request)
            .expect("evaluate_tether");
        let PlannerResponseWire::Matched(response) = wire else {
            panic!("T10: expected Matched, got {wire:?}");
        };
        let plan = response.get("plan").expect("plan missing");
        let actions = plan
            .get("actions")
            .and_then(Value::as_array)
            .expect("actions missing");
        assert_eq!(actions.len(), 1);
        let action = &actions[0];
        // Runtime identity: the capability the provider will execute
        assert_eq!(
            action.get("capability").and_then(Value::as_str),
            Some("fixture.ping"),
            "runtime capability must be fixture.ping"
        );
        // The request capabilities must have the Core semantic identity
        let caps = request
            .get("capabilities")
            .and_then(Value::as_array)
            .expect("capabilities");
        assert_eq!(caps.len(), 1);
        let cap = &caps[0];
        assert_eq!(
            cap.get("name").and_then(Value::as_str),
            Some("fixture.ping"),
            "runtime capability name must be fixture.ping"
        );
        // Core binding in core_environment
        let core_caps = request
            .pointer("/core_environment/capabilities")
            .and_then(Value::as_array)
            .expect("core_environment.capabilities");
        assert_eq!(core_caps.len(), 1);
        let cc = &core_caps[0];
        assert_eq!(
            cc.get("source_name").and_then(Value::as_str),
            Some("notify"),
            "Core source_name must be notify"
        );
        assert_eq!(
            cc.get("capability_id").and_then(Value::as_str),
            Some("cap.semantic.notify"),
            "Core capability_id must be cap.semantic.notify"
        );
        assert_eq!(
            cc.get("contract_digest").and_then(Value::as_str),
            Some("CORE-CONTRACT-9B"),
            "Core contract_digest must be CORE-CONTRACT-9B"
        );
        assert_eq!(
            cc.get("runtime_name").and_then(Value::as_str),
            Some("fixture.ping"),
            "runtime_name must be fixture.ping"
        );
        session.shutdown();
    }

    // T11: Bridge metadata separation — core_environment must NOT contain
    //      manifest_digest, bridge_capability_version, or
    //      bridge_provider_identity.  The top-level runtime capability
    //      MUST contain them.
    #[test]
    fn core9b_t11_bridge_metadata_separation() {
        let (engine_path, working_dir) = require_engine();
        let mut session = EngineSession::launch(&engine_path, &working_dir).expect("engine launch");
        let request = core9b_valid_request("eval_t11_bridge", "fixture.start");
        // core_environment must NOT contain bridge metadata
        let core_env = request.get("core_environment").expect("core_environment");
        assert!(
            core_env.get("manifest_digest").is_none(),
            "core_environment must not contain manifest_digest"
        );
        assert!(
            core_env.get("bridge_capability_version").is_none(),
            "core_environment must not contain bridge_capability_version"
        );
        assert!(
            core_env.get("bridge_provider_identity").is_none(),
            "core_environment must not contain bridge_provider_identity"
        );
        // Top-level runtime capability MUST contain bridge metadata
        let caps = request
            .get("capabilities")
            .and_then(Value::as_array)
            .expect("capabilities");
        assert_eq!(caps.len(), 1);
        let cap = &caps[0];
        assert_eq!(
            cap.get("manifest_digest").and_then(Value::as_str),
            Some("sha256:01fed7a4b877dd82abe91a1b6cfcd476b02e4c115489e70cbb285b8bf2d32d8b"),
            "runtime capability must have manifest_digest"
        );
        assert_eq!(
            cap.get("bridge_capability_version").and_then(Value::as_i64),
            Some(1),
            "runtime capability must have bridge_capability_version = 1"
        );
        assert_eq!(
            cap.get("bridge_provider_identity").and_then(Value::as_str),
            Some("tethers-stdio-fixture"),
            "runtime capability must have bridge_provider_identity"
        );
        // Send through core and verify it still matches
        let wire = session
            .evaluate_tether("eval_t11_bridge", &request)
            .expect("evaluate_tether");
        let PlannerResponseWire::Matched(_) = wire else {
            panic!("T11: expected Matched, got {wire:?}");
        };
        session.shutdown();
    }

    // T12: Real cross-language E2E — Rust request → real OCaml MCP binary
    //      → tethers.evaluate → Tethers_core_wire → CORE-8B → canonical
    //      Core → Runtime Plan.
    //
    //      This is the mandatory proof that the public production route
    //      delivers a canonical Core plan.
    #[test]
    fn core9b_t12_real_cross_language_e2e() {
        let (engine_path, working_dir) = require_engine();
        let mut session = EngineSession::launch(&engine_path, &working_dir).expect("engine launch");
        let request = core9b_valid_request("eval_core9b_001", "fixture.start");
        let wire = session
            .evaluate_tether("eval_core9b_001", &request)
            .expect("evaluate_tether E2E");
        let PlannerResponseWire::Matched(response) = wire else {
            panic!("T12: expected Matched, got {wire:?}");
        };

        // Correlation fields
        assert_eq!(response["protocol_version"], "0.1");
        assert_eq!(response["evaluation_id"], "eval_core9b_001");
        assert_eq!(response["event_id"], "evt_eval_core9b_001");
        assert_eq!(response["tether_id"], "core-rehearsal");
        assert_eq!(response["tether_version"], "1");

        // Plan structure
        let plan = response.get("plan").expect("plan missing");
        let plan_id = plan.get("id").and_then(Value::as_str).expect("plan.id");
        assert_eq!(
            plan_id, "eval_core9b_001/plan",
            "plan.id must be eval_id/plan"
        );

        // program_digest at top level (sibling of plan), NOT inside plan
        let pd = response
            .get("program_digest")
            .and_then(Value::as_str)
            .expect("program_digest missing from top level");
        assert!(
            pd.starts_with("sha256:"),
            "program_digest must start with sha256:"
        );
        assert_eq!(
            pd.len(),
            71,
            "program_digest must be sha256: + 64 hex chars"
        );
        assert!(
            plan.get("program_digest").is_none(),
            "program_digest must NOT be inside plan"
        );

        // Actions
        let actions = plan
            .get("actions")
            .and_then(Value::as_array)
            .expect("actions");
        assert_eq!(actions.len(), 1, "expected exactly one action");
        let action = &actions[0];
        assert_eq!(
            action.get("capability").and_then(Value::as_str),
            Some("fixture.ping"),
            "action capability must be fixture.ping"
        );
        let args = action.get("arguments").expect("action arguments");
        assert_eq!(
            args.get("message").and_then(Value::as_str),
            Some("Hello Core"),
            "action arguments.message must be Hello Core"
        );
        assert_eq!(
            action.get("idempotency_key").and_then(Value::as_str),
            Some("eval_core9b_001/action_1"),
            "idempotency_key must be eval_id/action_1"
        );

        // Effects
        let effects = action
            .get("effects")
            .and_then(Value::as_array)
            .expect("effects");
        assert!(
            effects.iter().any(|e| e.as_str() == Some("fixture.test")),
            "effects must contain fixture.test"
        );

        // Bridge metadata in the plan action
        assert_eq!(
            action.get("manifest_digest").and_then(Value::as_str),
            Some("sha256:01fed7a4b877dd82abe91a1b6cfcd476b02e4c115489e70cbb285b8bf2d32d8b"),
            "action manifest_digest must match fixture-ping"
        );
        assert_eq!(
            action
                .get("bridge_capability_version")
                .and_then(Value::as_i64),
            Some(1),
            "action bridge_capability_version must be 1"
        );
        assert_eq!(
            action
                .get("bridge_provider_identity")
                .and_then(Value::as_str),
            Some("tethers-stdio-fixture"),
            "action bridge_provider_identity must be tethers-stdio-fixture"
        );

        // Trail is empty (compatibility scaffolding)
        let trail = response.get("trail").expect("trail");
        assert!(
            trail.is_array() && trail.as_array().unwrap().is_empty(),
            "trail must be empty array"
        );

        // No Action is dispatched — this is a planner-only boundary
        session.shutdown();
    }

    // T13: Wrong event through the real Core flow — expect NotMatched.
    #[test]
    fn core9b_t13_wrong_event_not_matched() {
        let (engine_path, working_dir) = require_engine();
        let mut session = EngineSession::launch(&engine_path, &working_dir).expect("engine launch");
        let request = core9b_valid_request("eval_t13_wrong", "fixture.other");
        let wire = session
            .evaluate_tether("eval_t13_wrong", &request)
            .expect("evaluate_tether wrong event");
        match wire {
            PlannerResponseWire::NotMatched(response) => {
                assert_eq!(response["status"], "not_matched");
                assert_eq!(response["evaluation_id"], "eval_t13_wrong");
                assert_eq!(response["event_id"], "evt_eval_t13_wrong");
            }
            other => panic!("T13: expected NotMatched, got {other:?}"),
        }
        session.shutdown();
    }

    // T14: Occurrence identity — same semantic program, different
    //      evaluation_id produces same ProgramDigest but different
    //      plan.id and idempotency keys.
    #[test]
    fn core9b_t14_occurrence_identity() {
        let (engine_path, working_dir) = require_engine();
        let mut session = EngineSession::launch(&engine_path, &working_dir).expect("engine launch");

        let req1 = core9b_valid_request("eval_core9b_001", "fixture.start");
        let wire1 = session
            .evaluate_tether("eval_core9b_001", &req1)
            .expect("first evaluation");
        let PlannerResponseWire::Matched(resp1) = wire1 else {
            panic!("T14: expected Matched for first eval, got {wire1:?}")
        };

        let req2 = core9b_valid_request("eval_core9b_002", "fixture.start");
        let wire2 = session
            .evaluate_tether("eval_core9b_002", &req2)
            .expect("second evaluation");
        let PlannerResponseWire::Matched(resp2) = wire2 else {
            panic!("T14: expected Matched for second eval, got {wire2:?}")
        };

        let pd1 = resp1
            .get("program_digest")
            .and_then(Value::as_str)
            .expect("first program_digest");
        let pd2 = resp2
            .get("program_digest")
            .and_then(Value::as_str)
            .expect("second program_digest");
        assert_eq!(pd1, pd2, "same program must produce same program_digest");

        let pid1 = resp1
            .pointer("/plan/id")
            .and_then(Value::as_str)
            .expect("first plan.id");
        let pid2 = resp2
            .pointer("/plan/id")
            .and_then(Value::as_str)
            .expect("second plan.id");
        assert_ne!(
            pid1, pid2,
            "different evaluations must produce different plan.id"
        );
        assert_eq!(pid1, "eval_core9b_001/plan");
        assert_eq!(pid2, "eval_core9b_002/plan");

        let ik1 = resp1
            .pointer("/plan/actions/0/idempotency_key")
            .and_then(Value::as_str)
            .expect("first idempotency_key");
        let ik2 = resp2
            .pointer("/plan/actions/0/idempotency_key")
            .and_then(Value::as_str)
            .expect("second idempotency_key");
        assert_ne!(
            ik1, ik2,
            "different evaluations must produce different idempotency keys"
        );
        assert_eq!(ik1, "eval_core9b_001/action_1");
        assert_eq!(ik2, "eval_core9b_002/action_1");

        session.shutdown();
    }
}
