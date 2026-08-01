// MCP engine session manager for the OCaml Tethers MCP engine.
//
// Uses tools/call with name "tethers.validate" for Tether validation and
// "tethers.evaluate" for Tethers request evaluation via a retained session.

use crate::child_process::{ChildConfig, ChildError, SupervisedChild};
use serde_json::Value;
use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

const ENGINE_INITIALIZE_ID: u64 = 1;
const VALIDATION_REQUEST_BASE_ID: u64 = 100;

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
    pub fn launch(engine_path: &PathBuf, working_dir: &PathBuf) -> Result<Self, EngineError> {
        let config = ChildConfig {
            command: engine_path.to_string_lossy().into_owned(),
            args: Vec::new(),
            current_dir: Some(working_dir.clone()),
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
        let init_response = Self::read_json(&mut child, ENGINE_INITIALIZE_ID, "initialize")?;

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
        let result = Self::read_json(&mut self.child, request_id, "tools/call")?;

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
    /// envelope.  Returns the Tethers planner response from
    /// `result.structuredContent`.  A Tethers response with `status: "error"`
    /// is valid planner data and is returned normally; only MCP transport
    /// errors are treated as failures.
    pub fn evaluate_tether(
        &mut self,
        evaluation_id: &str,
        request_envelope: &Value,
    ) -> Result<Value, EngineError> {
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
        let result = Self::read_json(&mut self.child, request_id, "tools/call")?;

        // Extract structuredContent.  The Tethers planner response lives here.
        // A `status: "error"` inside structuredContent is planner data, not an
        // MCP transport failure.
        let structured = result.get("structuredContent").cloned().ok_or_else(|| {
            EngineError::ProtocolError("tools/call result missing structuredContent".to_owned())
        })?;

        // Validate that we got a recognizable Tethers response with a status field.
        match structured.get("status").and_then(Value::as_str) {
            Some(_) => Ok(structured),
            None => Err(EngineError::EvaluationFailed {
                evaluation_id: evaluation_id.to_owned(),
                message: "Tethers response missing status field".to_owned(),
            }),
        }
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
    ) -> Result<Value, EngineError> {
        let line = child.read_protocol_line(Duration::from_secs(10))?;
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

#[cfg(test)]
mod tests {
    use super::*;

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
        let request = serde_json::json!({
            "protocol_version": "0.1",
            "language_version": "0.1",
            "evaluation_id": "eval_j13b_real_001",
            "tether": {
                "id": "test.tether",
                "version": "1.0.0",
                "source": VALID_TETHER
            },
            "event": {
                "id": "evt_j13b_real_001",
                "name": "coding.task_completed",
                "data": {"project": "tethers"}
            },
            "facts": {"project.type": "software"},
            "capabilities": [{
                "name": "lantern.task.record",
                "version": "1.0.0",
                "inputs": {"project": "string"},
                "effects": ["lantern.write"],
                "reversibility": "compensatable"
            }]
        });
        let response = session
            .evaluate_tether("eval_j13b_real_001", &request)
            .expect("real retained tethers.evaluate call");
        assert_eq!(response["evaluation_id"], "eval_j13b_real_001");
        assert_eq!(response["event_id"], "evt_j13b_real_001");
        assert_eq!(response["status"], "matched");

        let mut second_request = request;
        second_request["evaluation_id"] = Value::String("eval_j13b_real_002".to_owned());
        second_request["event"]["id"] = Value::String("evt_j13b_real_002".to_owned());
        let second = session
            .evaluate_tether("eval_j13b_real_002", &second_request)
            .expect("second real retained tethers.evaluate call");
        assert_eq!(second["evaluation_id"], "eval_j13b_real_002");
        assert_eq!(second["event_id"], "evt_j13b_real_002");
        assert_eq!(second["status"], "matched");
        session.shutdown();
    }
}
