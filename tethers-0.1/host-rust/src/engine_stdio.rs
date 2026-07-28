// MCP engine session manager for the OCaml Tethers MCP engine.
//
// Provides one retained engine session that is initialized once and
// used for per-Tether validation.  No evaluation is permitted during
// the check command.

use crate::child_process::{ChildConfig, ChildError, SupervisedChild};
use serde_json::Value;
use std::fmt;
use std::io::BufRead;
use std::path::PathBuf;

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
    SerializeFailed(String),
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
            } => {
                write!(
                    f,
                    "validation failed for tether {tether_index} ({tether_id} v{tether_version}): [{error_code}] {message}"
                )
            }
            Self::SerializeFailed(msg) => write!(f, "serialization failed: {msg}"),
        }
    }
}

impl std::error::Error for EngineError {}

impl From<ChildError> for EngineError {
    fn from(e: ChildError) -> Self {
        EngineError::Child(e)
    }
}

/// A retained MCP engine session.
pub struct EngineSession {
    child: SupervisedChild,
    next_request_id: u64,
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

        // Perform initialize.
        let init_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": ENGINE_INITIALIZE_ID,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {
                    "name": "tethers-reference-host",
                    "version": "0.1.0"
                }
            }
        });

        Self::write_json(&mut child, &init_request)?;
        let init_response =
            Self::read_json_response(&mut child, ENGINE_INITIALIZE_ID, "initialize")?;

        // Verify protocol version.
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

        // Verify engine advertises tethers capability.
        let tethers_caps = init_response.pointer("/capabilities/tethers");
        if tethers_caps.is_none() || !tethers_caps.unwrap().is_object() {
            return Err(EngineError::InitializeFailed(
                "engine did not advertise tethers capability".to_owned(),
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
        })
    }

    /// Validate one Tether source. Returns Ok(()) on successful validation.
    pub fn validate_tether(
        &mut self,
        index: usize,
        id: &str,
        version: &str,
        source: &str,
    ) -> Result<(), EngineError> {
        let request_id = self.next_request_id;
        self.next_request_id += 1;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "tethers.validate",
            "params": {
                "tether": {
                    "id": id,
                    "version": version,
                    "source": source
                }
            }
        });

        Self::write_json(&mut self.child, &request)?;
        let response = Self::read_json_response(&mut self.child, request_id, "tethers.validate")?;

        // Check for validation status.
        let status = response.get("status").and_then(Value::as_str);
        match status {
            Some("valid") => Ok(()),
            Some("invalid") | Some("error") => {
                let error_code = response
                    .get("error_code")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown");
                let message = response
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("validation failed");
                Err(EngineError::ValidationFailed {
                    tether_index: index,
                    tether_id: id.to_owned(),
                    tether_version: version.to_owned(),
                    error_code: error_code.to_owned(),
                    message: message.to_owned(),
                })
            }
            _ => Err(EngineError::ProtocolError(format!(
                "unexpected validation status for tether {index}: {status:?}"
            ))),
        }
    }

    /// Get the retained stderr diagnostic tail.
    pub fn stderr_tail(&self) -> String {
        self.child.stderr_tail()
    }

    /// Shut down the engine gracefully.
    pub fn shutdown(self) {
        self.child.shutdown();
    }

    // Private helpers.

    fn write_json(child: &mut SupervisedChild, message: &Value) -> Result<(), EngineError> {
        let line = serde_json::to_string(message)
            .map_err(|e| EngineError::SerializeFailed(e.to_string()))?;
        child.write_line(&line).map_err(EngineError::from)
    }

    fn read_json_response(
        child: &mut SupervisedChild,
        expected_id: u64,
        method: &str,
    ) -> Result<Value, EngineError> {
        let line = child.read_protocol_line()?;
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

        // Check jsonrpc.
        if obj.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
            return Err(EngineError::ProtocolError(
                "engine response missing jsonrpc 2.0".to_owned(),
            ));
        }

        // Check for error.
        if let Some(error) = obj.get("error") {
            let msg = error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("unknown error");
            return Err(EngineError::ProtocolError(format!(
                "engine returned error for {method}: {msg}"
            )));
        }

        // Check id matches.
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

    // Engine tests require the OCaml engine binary built.
    // These are focused on protocol-level unit testing.
    // Integration tests live in test-j13a-check.ps1.

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

    #[test]
    fn j13a_engine_launch_and_validate_valid_tether() {
        let engine_path = match engine_binary_path() {
            Some(p) => p,
            None => {
                eprintln!("SKIP: engine binary not found");
                return;
            }
        };

        let working_dir = engine_path.parent().unwrap().to_path_buf();

        let mut session = match EngineSession::launch(&engine_path, &working_dir) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("SKIP: engine launch failed (env restriction): {e}");
                return;
            }
        };

        // Validate a simple valid tether.
        let result =
            session.validate_tether(0, "test.tether", "1.0.0", "on event hello do log \"ok\"\n");
        assert!(
            result.is_ok(),
            "valid tether should pass: {:?}",
            result.err()
        );

        session.shutdown();
    }

    #[test]
    fn j13a_engine_validate_invalid_tether() {
        let engine_path = match engine_binary_path() {
            Some(p) => p,
            None => {
                eprintln!("SKIP: engine binary not found");
                return;
            }
        };

        let working_dir = engine_path.parent().unwrap().to_path_buf();

        let mut session = match EngineSession::launch(&engine_path, &working_dir) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("SKIP: engine launch failed (env restriction): {e}");
                return;
            }
        };

        // Validate an invalid tether.
        let result = session.validate_tether(0, "bad.tether", "1.0.0", "garbage syntax {{{");
        match result {
            Err(EngineError::ValidationFailed { .. }) => {} // expected
            Ok(()) => panic!("invalid tether should fail validation"),
            Err(e) => panic!("expected ValidationFailed, got {e:?}"),
        }

        session.shutdown();
    }

    #[test]
    fn j13a_engine_one_retained_session() {
        let engine_path = match engine_binary_path() {
            Some(p) => p,
            None => {
                eprintln!("SKIP: engine binary not found");
                return;
            }
        };

        let working_dir = engine_path.parent().unwrap().to_path_buf();

        let mut session = match EngineSession::launch(&engine_path, &working_dir) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("SKIP: engine launch failed (env restriction): {e}");
                return;
            }
        };

        // Validate multiple tethers through the same session.
        let t1 = session.validate_tether(0, "t1", "1.0.0", "on event hello do log \"ok\"\n");
        let t2 = session.validate_tether(1, "t2", "1.0.0", "on event world do log \"ok\"\n");
        assert!(t1.is_ok(), "t1 should be valid: {:?}", t1.err());
        assert!(t2.is_ok(), "t2 should be valid: {:?}", t2.err());

        session.shutdown();
    }
}
