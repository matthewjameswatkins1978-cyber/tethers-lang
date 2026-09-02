//! Native MCP provider for the Agent Essentials workspace slice.
//!
//! The provider accepts only the reviewed operation set and receives its
//! operational roots from the host through TETHERS_OPERATIONAL_SCOPE_JSON.

use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use tethers_reference_host::agent_workspace;
use tethers_reference_host::file_tools::FileScope;

const SCOPE_ENV: &str = "TETHERS_OPERATIONAL_SCOPE_JSON";
const CONFORMANCE_ENV: &str = "TETHERS_CONFORMANCE";
const TEMP_ENV: &str = "TEMP";

fn response(id: &Value, result: Value) -> Value {
    json!({"jsonrpc":"2.0","id":id,"result":result})
}
fn error(id: &Value, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})
}

fn scope() -> Result<FileScope, String> {
    let value = match std::env::var(SCOPE_ENV) {
        Ok(value) => value,
        Err(_) if std::env::var(CONFORMANCE_ENV).ok().as_deref() == Some("1") => {
            let temp = std::env::var(TEMP_ENV)
                .map_err(|_| "TEMP is required during conformance".to_owned())?;
            json!({"query_root":temp,"move_source_root":temp,"move_destination_root":temp,"max_content_bytes":65536}).to_string()
        }
        Err(_) => {
            return Err(format!(
                "{SCOPE_ENV} is required in installed operational mode"
            ))
        }
    };
    let parsed: Value = serde_json::from_str(&value)
        .map_err(|e| format!("malformed operational scope JSON: {e}"))?;
    let scope_object = parsed
        .as_object()
        .ok_or_else(|| "operational scope must be a JSON object".to_owned())?;
    if scope_object.keys().any(|key| {
        ![
            "query_root",
            "move_source_root",
            "move_destination_root",
            "max_content_bytes",
        ]
        .contains(&key.as_str())
    }) {
        return Err("operational scope contains an unknown field".to_owned());
    }
    let root = |name: &str| {
        parsed
            .get(name)
            .and_then(Value::as_str)
            .ok_or_else(|| format!("{name} is required and must be a string"))
            .map(PathBuf::from)
    };
    let query = root("query_root")?;
    let source = root("move_source_root")?;
    let destination = root("move_destination_root")?;
    let limit = parsed
        .get("max_content_bytes")
        .and_then(Value::as_u64)
        .ok_or_else(|| "max_content_bytes is required and must be an integer".to_owned())?;
    FileScope::new(&query, &source, &destination)
        .map_err(|e| e.to_string())?
        .with_max_content_bytes(limit)
        .map_err(|e| e.to_string())
}

fn tool(name: &str, input: Value, output: Value) -> Value {
    json!({"name":name,"inputSchema":input,"outputSchema":output})
}
fn path_schema() -> Value {
    json!({"type":"string","minLength":1,"pattern":"^(?!/)(?!.*\\\\)(?!.*(^|/)\\.\\.?(/|$))[^:]+$"})
}
fn root_path_schema() -> Value {
    json!({"type":"string","minLength":1,"pattern":"^(?:\\.|(?!/)(?!.*\\\\)(?!.*(^|/)\\.\\.?(/|$))[^:]+$"})
}
fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","properties":properties,"required":required,"additionalProperties":false})
}
fn output_object(properties: Value, required: &[&str]) -> Value {
    object(properties, required)
}

fn sha256_schema() -> Value {
    json!({"type":"string","pattern":"^sha256:[a-f0-9]{64}$"})
}

fn entry_schema() -> Value {
    output_object(
        json!({
            "path":{"type":"string"},
            "kind":{"type":"string","enum":["file","directory","other"]},
            "size_bytes":{"type":["integer","null"],"minimum":0}
        }),
        &["path", "kind", "size_bytes"],
    )
}

fn match_schema() -> Value {
    output_object(
        json!({
            "line":{"type":"integer","minimum":1},
            "column":{"type":"integer","minimum":1},
            "text":{"type":"string"}
        }),
        &["line", "column", "text"],
    )
}

fn manifest_entry_schema() -> Value {
    json!({
        "oneOf":[
            output_object(
                json!({"path":{"type":"string"},"type":{"const":"directory"}}),
                &["path","type"],
            ),
            output_object(
                json!({"path":{"type":"string"},"type":{"const":"file"},"sha256":sha256_schema()}),
                &["path","type","sha256"],
            )
        ]
    })
}

fn tools() -> Value {
    let path = path_schema();
    let root_path = root_path_schema();
    json!({"tools":[
        tool("filesystem_read", object(json!({"path":path,"max_bytes":{"type":"integer","minimum":1,"maximum":65536}}), &["path","max_bytes"]), output_object(json!({"path":{"type":"string"},"content":{"type":"string"},"bytes_read":{"type":"integer","minimum":0}}), &["path","content","bytes_read"])),
        tool("filesystem_list", object(json!({"path":root_path}), &[]), output_object(json!({"path":{"type":"string"},"entries":{"type":"array","items":entry_schema()}}), &["path","entries"])),
        tool("filesystem_stat", object(json!({"path":root_path}), &["path"]), entry_schema()),
        tool("text_search", object(json!({"path":path,"query":{"type":"string","minLength":1},"mode":{"type":"string","enum":["literal","regex"]},"max_matches":{"type":"integer","minimum":1,"maximum":10000}}), &["path","query","mode","max_matches"]), output_object(json!({"path":{"type":"string"},"mode":{"type":"string","enum":["literal","regex"]},"matches":{"type":"array","items":match_schema()},"truncated":{"type":"boolean"}}), &["path","mode","matches","truncated"])),
        tool("text_read_range", object(json!({"path":path,"start_line":{"type":"integer","minimum":1},"end_line":{"type":"integer","minimum":1}}), &["path","start_line","end_line"]), output_object(json!({"path":{"type":"string"},"start_line":{"type":"integer","minimum":1},"end_line":{"type":"integer","minimum":1},"content":{"type":"string"},"line_count":{"type":"integer","minimum":0}}), &["path","start_line","end_line","content","line_count"])),
        tool("text_replace_exact", object(json!({"path":path,"old_text":{"type":"string","minLength":1},"new_text":{"type":"string"},"expected_matches":{"type":"integer","minimum":0,"maximum":10000}}), &["path","old_text","new_text","expected_matches"]), output_object(json!({"path":{"type":"string"},"changed":{"type":"boolean"},"changed_count":{"type":"integer","minimum":0},"bytes_written":{"type":"integer","minimum":0}}), &["path","changed","changed_count","bytes_written"])),
        tool("text_compare", object(json!({"left_path":path,"right_path":path}), &["left_path","right_path"]), output_object(json!({"equal":{"type":"boolean"},"left_path":{"type":"string"},"right_path":{"type":"string"},"left_sha256":sha256_schema(),"right_sha256":sha256_schema()}), &["equal","left_path","right_path","left_sha256","right_sha256"])),
        tool("patch_apply", object(json!({"patch":{"type":"string","minLength":1},"expected_base_sha256":{"type":"string","pattern":"^sha256:[a-f0-9]{64}$"}}), &["patch"]), output_object(json!({"changed_files":{"type":"array","items":{"type":"string"}},"changed_hunks":{"type":"integer","minimum":1},"bytes_written":{"type":"integer","minimum":0}}), &["changed_files","changed_hunks","bytes_written"])),
        tool("hash_sha256", object(json!({"path":path,"text":{"type":"string"}}), &[]), output_object(json!({"sha256":sha256_schema(),"bytes":{"type":"integer","minimum":0}}), &["sha256","bytes"])),
        tool("hash_verify", object(json!({"path":path,"sha256":sha256_schema()}), &["path","sha256"]), output_object(json!({"path":{"type":"string"},"expected_sha256":sha256_schema(),"actual_sha256":sha256_schema(),"verified":{"type":"boolean"}}), &["path","expected_sha256","actual_sha256","verified"])),
        tool("hash_directory_manifest", object(json!({"path":root_path}), &[]), output_object(json!({"path":{"type":"string"},"entries":{"type":"array","items":manifest_entry_schema()},"entry_count":{"type":"integer","minimum":0}}), &["path","entries","entry_count"]))
    ]})
}

fn call(
    scope: &FileScope,
    name: &str,
    arguments: &Value,
) -> Result<Value, agent_workspace::WorkspaceError> {
    match name {
        agent_workspace::FILESYSTEM_READ => agent_workspace::read(scope, arguments),
        agent_workspace::FILESYSTEM_LIST => agent_workspace::list(scope, arguments),
        agent_workspace::FILESYSTEM_STAT => agent_workspace::stat(scope, arguments),
        agent_workspace::TEXT_SEARCH => agent_workspace::search(scope, arguments),
        agent_workspace::TEXT_READ_RANGE => agent_workspace::read_range(scope, arguments),
        agent_workspace::TEXT_REPLACE_EXACT => agent_workspace::replace_exact(scope, arguments),
        agent_workspace::TEXT_COMPARE => agent_workspace::compare(scope, arguments),
        agent_workspace::PATCH_APPLY => agent_workspace::patch_apply(scope, arguments),
        agent_workspace::HASH_SHA256 => agent_workspace::sha256(scope, arguments),
        agent_workspace::HASH_VERIFY => agent_workspace::verify(scope, arguments),
        agent_workspace::HASH_DIRECTORY_MANIFEST => {
            agent_workspace::directory_manifest(scope, arguments)
        }
        _ => Err(agent_workspace::WorkspaceError {
            code: "unknown_operation",
            message: "operation is not part of the reviewed Agent Essentials workspace contract"
                .into(),
        }),
    }
}

fn main() {
    let scope = match scope() {
        Ok(scope) => scope,
        Err(message) => {
            eprintln!("agent workspace provider configuration refused: {message}");
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
                    json!({"protocolVersion":"2025-11-25","capabilities":{"tools":{}},"serverInfo":{"name":"tethers-agent-workspace","version":"0.1.0"}}),
                )
            }
            "notifications/initialized" => {
                if initialized {
                    client_initialized = true;
                }
                Ok(Value::Null)
            }
            "tools/list" if client_initialized => Ok(tools()),
            "tools/call" if client_initialized => {
                let params = request.get("params").cloned().unwrap_or(Value::Null);
                let name = params.get("name").and_then(Value::as_str).unwrap_or("");
                let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                call(&scope, name, &arguments).map_err(|e| e)
            }
            "tools/list" | "tools/call" => Err(agent_workspace::WorkspaceError {
                code: "not_initialized",
                message: "MCP session is not initialized".into(),
            }),
            _ => Err(agent_workspace::WorkspaceError {
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
