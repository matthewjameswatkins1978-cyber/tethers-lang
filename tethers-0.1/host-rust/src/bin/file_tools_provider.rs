//! Credential-free native reference provider for the M4 File Tools Plug.
//!
//! In installed operational mode, scope is delivered through
//! `TETHERS_OPERATIONAL_SCOPE_JSON`. During host conformance the provider
//! falls back to the TEMP scratch directory.

use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use tethers_reference_host::file_tools::{self, FileScope};

const OPERATIONAL_SCOPE_JSON_ENV: &str = "TETHERS_OPERATIONAL_SCOPE_JSON";
const TETHERS_CONFORMANCE_ENV: &str = "TETHERS_CONFORMANCE";
const TEMP_ENV: &str = "TEMP";

fn response(id: &Value, result: Value) -> Value {
    json!({"jsonrpc":"2.0","id":id,"result":result})
}
fn error(id: &Value, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})
}

fn is_conformance_mode(value: Option<&str>) -> bool {
    value == Some("1")
}

fn required_path<'a>(scope: &'a Value, field: &str) -> Result<&'a str, String> {
    match scope.get(field).and_then(Value::as_str) {
        Some(value) if !value.is_empty() => Ok(value),
        Some(_) => Err(format!("{field} must not be empty")),
        None => Err(format!("{field} is required and must be a string")),
    }
}

fn conformance_scope(temp: Option<&str>) -> Result<FileScope, String> {
    let temp = temp
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "TEMP is required during host conformance".to_owned())?;
    let root = PathBuf::from(temp);
    FileScope::new(&root, &root, &root).map_err(|error| error.to_string())
}

fn scope_from_configuration(
    conformance: bool,
    operational_scope_json: Option<&str>,
    temp: Option<&str>,
) -> Result<FileScope, String> {
    let json = match operational_scope_json.filter(|value| !value.is_empty()) {
        Some(json) => json,
        None if conformance => return conformance_scope(temp),
        None => {
            return Err(
                "TETHERS_OPERATIONAL_SCOPE_JSON is required in installed operational mode".into(),
            )
        }
    };
    let scope: Value =
        serde_json::from_str(json).map_err(|_| "malformed operational scope JSON".to_owned())?;
    let query_root = PathBuf::from(required_path(&scope, "query_root")?);
    let source_root = PathBuf::from(required_path(&scope, "move_source_root")?);
    let destination_root = PathBuf::from(required_path(&scope, "move_destination_root")?);
    let max_content_bytes = scope
        .get("max_content_bytes")
        .and_then(Value::as_u64)
        .ok_or_else(|| "max_content_bytes is required and must be an integer".to_owned())?;

    FileScope::new(&query_root, &source_root, &destination_root)
        .map_err(|error| error.to_string())?
        .with_max_content_bytes(max_content_bytes)
        .map_err(|error| error.to_string())
}

fn resolve_scope() -> Result<FileScope, String> {
    let scope = match std::env::var_os(OPERATIONAL_SCOPE_JSON_ENV) {
        Some(value) => Some(
            value
                .into_string()
                .map_err(|_| "TETHERS_OPERATIONAL_SCOPE_JSON must be valid UTF-8".to_owned())?,
        ),
        None => None,
    };
    scope_from_configuration(
        is_conformance_mode(std::env::var(TETHERS_CONFORMANCE_ENV).ok().as_deref()),
        scope.as_deref(),
        std::env::var(TEMP_ENV).ok().as_deref(),
    )
}

fn main() {
    let scope = match resolve_scope() {
        Ok(scope) => scope,
        Err(error) => {
            eprintln!("file-tools provider configuration refused: {error}");
            std::process::exit(2);
        }
    };
    let stdin = io::stdin();
    let mut initialized = false;
    let mut client_initialized = false;
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let request: Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(_) => {
                eprintln!("malformed JSON request");
                continue;
            }
        };
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let method = request.get("method").and_then(Value::as_str).unwrap_or("");
        let result = match method {
            "initialize" => {
                initialized = true;
                Ok(
                    json!({"protocolVersion":"2025-11-25","capabilities":{"tools":{}},"serverInfo":{"name":"tethers-file-tools","version":"1.0.0"}}),
                )
            }
            "notifications/initialized" => {
                if initialized {
                    client_initialized = true;
                }
                Ok(Value::Null)
            }
            "tools/list" if client_initialized => Ok(json!({"tools":[
                {"name":file_tools::METADATA_OPERATION,"inputSchema":file_tools::metadata_input_schema(),"outputSchema":file_tools::metadata_output_schema()},
                {"name":file_tools::METADATA_V2_OPERATION,"inputSchema":file_tools::metadata_v2_input_schema(),"outputSchema":file_tools::metadata_v2_output_schema()},
                {"name":file_tools::MOVE_OPERATION,"inputSchema":file_tools::move_input_schema(),"outputSchema":file_tools::move_output_schema()}
            ]})),
            "tools/call" if client_initialized => {
                let params = request.get("params").cloned().unwrap_or(Value::Null);
                let name = params.get("name").and_then(Value::as_str).unwrap_or("");
                let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                match name {
                    file_tools::METADATA_OPERATION => file_tools::metadata(&scope, &arguments),
                    file_tools::METADATA_V2_OPERATION => {
                        file_tools::metadata_v2(&scope, &arguments)
                    }
                    file_tools::MOVE_OPERATION => file_tools::move_file(&scope, &arguments),
                    _ => Err(file_tools::FileToolsError {
                        code: "unknown_operation",
                        message: "operation is not part of the reviewed File Tools contract".into(),
                    }),
                }
            }
            "tools/list" | "tools/call" => Err(file_tools::FileToolsError {
                code: "not_initialized",
                message: "MCP session is not initialized".into(),
            }),
            _ => Err(file_tools::FileToolsError {
                code: "method_not_found",
                message: "unsupported MCP method".into(),
            }),
        };
        if method == "notifications/initialized" {
            continue;
        }
        let output = match result {
            Ok(value) => response(&id, if value.is_null() { json!({}) } else { value }),
            Err(failure) => error(
                &id,
                if failure.code == "unknown_operation" || failure.code == "method_not_found" {
                    -32601
                } else {
                    -32602
                },
                &failure.message,
            ),
        };
        println!(
            "{}",
            serde_json::to_string(&output).expect("provider response is serializable")
        );
        io::stdout().flush().ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope_json(root: &std::path::Path, max_content_bytes: Value) -> String {
        json!({
            "query_root": root,
            "move_source_root": root,
            "move_destination_root": root,
            "max_content_bytes": max_content_bytes,
        })
        .to_string()
    }

    #[test]
    fn valid_generic_scope_loads_exact_roots() {
        let root = std::env::temp_dir();
        let scope =
            scope_from_configuration(false, Some(&scope_json(&root, json!(4096))), None).unwrap();
        let canonical = std::fs::canonicalize(root).unwrap();
        assert_eq!(scope.query_root, canonical);
        assert_eq!(scope.source_root, canonical);
        assert_eq!(scope.destination_root, canonical);
    }

    #[test]
    fn valid_smaller_max_content_bytes_is_applied_exactly() {
        let root = std::env::temp_dir();
        let scope =
            scope_from_configuration(false, Some(&scope_json(&root, json!(4096))), None).unwrap();
        assert_eq!(scope.max_content_bytes, 4096);
    }

    #[test]
    fn missing_scope_refuses_in_normal_mode() {
        assert!(scope_from_configuration(false, None, None).is_err());
    }

    #[test]
    fn malformed_scope_refuses() {
        assert!(scope_from_configuration(false, Some("{"), None).is_err());
    }

    #[test]
    fn missing_query_root_refuses() {
        let root = std::env::temp_dir();
        let scope = json!({
            "move_source_root": root,
            "move_destination_root": root,
            "max_content_bytes": 4096,
        })
        .to_string();
        assert!(scope_from_configuration(false, Some(&scope), None).is_err());
    }

    #[test]
    fn missing_move_source_root_refuses() {
        let root = std::env::temp_dir();
        let scope = json!({
            "query_root": root,
            "move_destination_root": root,
            "max_content_bytes": 4096,
        })
        .to_string();
        assert!(scope_from_configuration(false, Some(&scope), None).is_err());
    }

    #[test]
    fn missing_move_destination_root_refuses() {
        let root = std::env::temp_dir();
        let scope = json!({
            "query_root": root,
            "move_source_root": root,
            "max_content_bytes": 4096,
        })
        .to_string();
        assert!(scope_from_configuration(false, Some(&scope), None).is_err());
    }

    #[test]
    fn missing_max_content_bytes_refuses() {
        let root = std::env::temp_dir();
        let scope = json!({
            "query_root": root,
            "move_source_root": root,
            "move_destination_root": root,
        })
        .to_string();
        assert!(scope_from_configuration(false, Some(&scope), None).is_err());
    }

    #[test]
    fn wrong_field_type_refuses() {
        let root = std::env::temp_dir();
        let scope = scope_json(&root, json!("4096"));
        assert!(scope_from_configuration(false, Some(&scope), None).is_err());
    }

    #[test]
    fn invalid_max_content_bytes_refuses() {
        let root = std::env::temp_dir();
        let scope = scope_json(&root, json!(file_tools::MAX_CONTENT_BYTES + 1));
        assert!(scope_from_configuration(false, Some(&scope), None).is_err());
    }

    #[test]
    fn conformance_unset_does_not_activate_fallback() {
        assert!(!is_conformance_mode(None));
        assert!(scope_from_configuration(is_conformance_mode(None), None, None).is_err());
    }

    #[test]
    fn conformance_zero_does_not_activate_fallback() {
        assert!(!is_conformance_mode(Some("0")));
        assert!(scope_from_configuration(is_conformance_mode(Some("0")), None, None).is_err());
    }

    #[test]
    fn other_conformance_values_do_not_activate_fallback() {
        assert!(!is_conformance_mode(Some("true")));
        assert!(scope_from_configuration(is_conformance_mode(Some("true")), None, None).is_err());
    }

    #[test]
    fn conformance_one_activates_fallback() {
        let temp = std::env::temp_dir();
        let scope =
            scope_from_configuration(is_conformance_mode(Some("1")), None, temp.to_str()).unwrap();
        let canonical = std::fs::canonicalize(temp).unwrap();
        assert_eq!(scope.query_root, canonical);
        assert_eq!(scope.max_content_bytes, file_tools::MAX_CONTENT_BYTES);
    }
}
