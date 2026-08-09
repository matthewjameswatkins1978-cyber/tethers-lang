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

fn response(id: &Value, result: Value) -> Value {
    json!({"jsonrpc":"2.0","id":id,"result":result})
}
fn error(id: &Value, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})
}

fn resolve_root(field: &str) -> PathBuf {
    let json = match std::env::var(OPERATIONAL_SCOPE_JSON_ENV) {
        Ok(v) if !v.is_empty() => v,
        _ => return std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };
    let value: Value = match serde_json::from_str(&json) {
        Ok(v) => v,
        Err(_) => return std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };
    match value.get(field).and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => PathBuf::from(s),
        _ => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    }
}

fn main() {
    let root = resolve_root("query_root");
    let source = resolve_root("move_source_root");
    let destination = resolve_root("move_destination_root");
    let scope = match FileScope::new(&root, &PathBuf::from(&source), &PathBuf::from(&destination)) {
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
