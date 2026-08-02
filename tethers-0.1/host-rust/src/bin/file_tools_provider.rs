//! Credential-free native reference provider for the M4 File Tools Plug.

use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use tethers_reference_host::file_tools::{self, FileScope};

fn argument(name: &str) -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(value) = args.next() {
        if value == name {
            return args.next();
        }
    }
    None
}

fn response(id: &Value, result: Value) -> Value {
    json!({"jsonrpc":"2.0","id":id,"result":result})
}
fn error(id: &Value, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})
}

fn main() {
    let fallback = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let root = argument("--query-root")
        .or_else(|| argument("--provider-root"))
        .filter(|value| !value.starts_with("__TETHERS_FILE_"))
        .map(PathBuf::from)
        .unwrap_or_else(|| fallback.clone());
    let source = argument("--source-root")
        .filter(|value| !value.starts_with("__TETHERS_FILE_"))
        .map(PathBuf::from)
        .unwrap_or_else(|| root.clone());
    let destination = argument("--destination-root")
        .filter(|value| !value.starts_with("__TETHERS_FILE_"))
        .map(PathBuf::from)
        .unwrap_or_else(|| root.clone());
    let scope = match FileScope::new(&root, &PathBuf::from(source), &PathBuf::from(destination)) {
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
