//! Native MCP provider for the Agent Essentials coding slice.
//!
//! Git operations are structured and repository-rooted. Process execution is
//! argv-only and is independently bounded by the host-supplied scope. Named
//! verification checks are configured by scope and cannot accept command text.

use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use tethers_reference_host::agent_coding;

const SCOPE_ENV: &str = "TETHERS_OPERATIONAL_SCOPE_JSON";
const CONFORMANCE_ENV: &str = "TETHERS_CONFORMANCE";
const TEMP_ENV: &str = "TEMP";

fn response(id: &Value, result: Value) -> Value {
    json!({"jsonrpc":"2.0","id":id,"result":result})
}

fn error(id: &Value, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})
}

fn scope() -> Result<agent_coding::CodingScope, String> {
    let value = match std::env::var(SCOPE_ENV) {
        Ok(value) => value,
        Err(_) if std::env::var(CONFORMANCE_ENV).ok().as_deref() == Some("1") => {
            let temp = std::env::var(TEMP_ENV)
                .map_err(|_| "TEMP is required during conformance".to_owned())?;
            json!({
                "repository_root": temp,
                "process_cwd_root": temp,
                "allowed_programs": ["git"],
                "max_runtime_ms": 5000,
                "max_output_bytes": 65536,
                "allowed_environment_keys": [],
                "verification_checks": {}
            })
            .to_string()
        }
        Err(_) => {
            return Err(format!(
                "{SCOPE_ENV} is required in installed operational mode"
            ))
        }
    };
    let parsed: Value = serde_json::from_str(&value)
        .map_err(|error| format!("malformed operational scope JSON: {error}"))?;
    agent_coding::CodingScope::from_json(&parsed).map_err(|error| error.to_string())
}

fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","properties":properties,"required":required,"additionalProperties":false})
}

fn output_object(properties: Value, required: &[&str]) -> Value {
    object(properties, required)
}

fn tool(name: &str, input: Value, output: Value) -> Value {
    json!({"name":name,"inputSchema":input,"outputSchema":output})
}

fn relative_path() -> Value {
    json!({"type":"string","minLength":1,"pattern":"^(?!/)(?!.*\\\\)(?!.*(^|/)\\.\\.?(/|$))[^:]+$"})
}

fn nonempty_text() -> Value {
    json!({"type":"string","minLength":1})
}

fn bounded_bytes() -> Value {
    json!({"type":"integer","minimum":1,"maximum":16777216})
}

fn process_output() -> Value {
    output_object(
        json!({
            "program": nonempty_text(),
            "cwd": nonempty_text(),
            "exit_code":{"type":["integer","null"]},
            "stdout":{"type":"string"},
            "stderr":{"type":"string"},
            "stdout_utf8":{"type":"boolean"},
            "stderr_utf8":{"type":"boolean"},
            "stdout_truncated":{"type":"boolean"},
            "stderr_truncated":{"type":"boolean"},
            "timed_out":{"type":"boolean"},
            "duration_ms":{"type":"integer","minimum":0}
        }),
        &[
            "program",
            "cwd",
            "exit_code",
            "stdout",
            "stderr",
            "stdout_utf8",
            "stderr_utf8",
            "stdout_truncated",
            "stderr_truncated",
            "timed_out",
            "duration_ms",
        ],
    )
}

fn verification_output() -> Value {
    output_object(
        json!({
            "check": nonempty_text(),
            "passed":{"type":"boolean"},
            "exit_code":{"type":["integer","null"]},
            "duration_ms":{"type":"integer","minimum":0},
            "stdout":{"type":"string"},
            "stderr":{"type":"string"},
            "stdout_utf8":{"type":"boolean"},
            "stderr_utf8":{"type":"boolean"},
            "stdout_truncated":{"type":"boolean"},
            "stderr_truncated":{"type":"boolean"},
            "timed_out":{"type":"boolean"}
        }),
        &[
            "check",
            "passed",
            "exit_code",
            "duration_ms",
            "stdout",
            "stderr",
            "stdout_utf8",
            "stderr_utf8",
            "stdout_truncated",
            "stderr_truncated",
            "timed_out",
        ],
    )
}

fn status_output() -> Value {
    output_object(
        json!({
            "branch":{"type":["string","null"]},
            "clean":{"type":"boolean"},
            "entries":{"type":"array","items":output_object(json!({
                "index_status":{"type":"string","minLength":1,"maxLength":1},
                "worktree_status":{"type":"string","minLength":1,"maxLength":1},
                "path":{"type":"string","minLength":1}
            }), &["index_status","worktree_status","path"])}
        }),
        &["branch", "clean", "entries"],
    )
}

fn tools() -> Value {
    let path = relative_path();
    let bytes = bounded_bytes();
    let command_args = json!({
        "program": nonempty_text(),
        "args":{"type":"array","maxItems":256,"items":{"type":"string","maxLength":16384}},
        "cwd":{"type":"string","minLength":1,"pattern":"^(?:\\.|(?!/)(?!.*\\\\)(?!.*(^|/)\\.\\.?(/|$))[^:]+$"},
        "timeout_ms":{"type":"integer","minimum":1,"maximum":120000},
        "max_output_bytes":bytes.clone(),
        "environment":{"type":"object","additionalProperties":{"type":"string","maxLength":16384}}
    });
    let diff_output = output_object(
        json!({
            "staged":{"type":"boolean"},
            "path":{"type":["string","null"]},
            "diff":{"type":"string"},
            "utf8":{"type":"boolean"},
            "truncated":{"type":"boolean"}
        }),
        &["staged", "path", "diff", "utf8", "truncated"],
    );
    let log_output = output_object(
        json!({
            "commits":{"type":"array","items":output_object(json!({
                "commit":{"type":"string"},"author":{"type":"string"},
                "timestamp":{"type":"string"},"subject":{"type":"string"}
            }), &["commit","author","timestamp","subject"])}
        }),
        &["commits"],
    );
    let show_output = output_object(
        json!({
            "revision":{"type":"string"},"content":{"type":"string"},
            "utf8":{"type":"boolean"},"truncated":{"type":"boolean"}
        }),
        &["revision", "content", "utf8", "truncated"],
    );
    json!({"tools":[
        tool("git_status", object(json!({}), &[]), status_output()),
        tool("git_diff", object(json!({"staged":{"type":"boolean"},"max_bytes":bytes.clone(),"path":path.clone()}), &["staged","max_bytes"]), diff_output),
        tool("git_log", object(json!({"max_count":{"type":"integer","minimum":1,"maximum":1000}}), &["max_count"]), log_output),
        tool("git_show", object(json!({"revision":nonempty_text(),"max_bytes":bytes.clone()}), &["revision","max_bytes"]), show_output),
        tool("git_branch_list", object(json!({}), &[]), output_object(json!({"branches":{"type":"array","items":output_object(json!({"name":{"type":"string"},"commit":{"type":"string"},"upstream":{"type":["string","null"]}}), &["name","commit","upstream"] )}}), &["branches"])),
        tool("git_branch_current", object(json!({}), &[]), output_object(json!({"branch":{"type":["string","null"]}}), &["branch"])),
        tool("git_add", object(json!({"paths":{"type":"array","minItems":1,"maxItems":256,"items":path.clone()}}), &["paths"]), output_object(json!({"added_paths":{"type":"array","items":{"type":"string"}}}), &["added_paths"])),
        tool("git_branch_create", object(json!({"branch":nonempty_text(),"start_point":nonempty_text()}), &["branch"]), output_object(json!({"created":{"type":"string"},"start_point":{"type":["string","null"]}}), &["created","start_point"])),
        tool("git_checkout", object(json!({"branch":nonempty_text()}), &["branch"]), output_object(json!({"branch":{"type":"string"},"checked_out":{"type":"boolean"}}), &["branch","checked_out"])),
        tool("git_commit", object(json!({"message":nonempty_text()}), &["message"]), output_object(json!({"committed":{"type":"boolean"},"commit":{"type":"string"}}), &["committed","commit"])),
        tool("process_execute", object(command_args, &["program","args"]), process_output()),
        tool("verification_run", object(json!({"check":nonempty_text()}), &["check"]), verification_output())
    ]})
}

fn call(
    scope: &agent_coding::CodingScope,
    name: &str,
    arguments: &Value,
) -> Result<Value, agent_coding::CodingError> {
    match name {
        agent_coding::GIT_STATUS => agent_coding::git_status(scope, arguments),
        agent_coding::GIT_DIFF => agent_coding::git_diff(scope, arguments),
        agent_coding::GIT_LOG => agent_coding::git_log(scope, arguments),
        agent_coding::GIT_SHOW => agent_coding::git_show(scope, arguments),
        agent_coding::GIT_BRANCH_LIST => agent_coding::git_branch_list(scope, arguments),
        agent_coding::GIT_BRANCH_CURRENT => agent_coding::git_branch_current(scope, arguments),
        agent_coding::GIT_ADD => agent_coding::git_add(scope, arguments),
        agent_coding::GIT_BRANCH_CREATE => agent_coding::git_branch_create(scope, arguments),
        agent_coding::GIT_CHECKOUT => agent_coding::git_checkout(scope, arguments),
        agent_coding::GIT_COMMIT => agent_coding::git_commit(scope, arguments),
        agent_coding::PROCESS_EXECUTE => agent_coding::process_execute(scope, arguments),
        agent_coding::VERIFICATION_RUN => agent_coding::verification_run(scope, arguments),
        _ => Err(agent_coding::CodingError {
            code: "unknown_operation",
            message: "operation is not part of the reviewed Agent Essentials coding contract"
                .into(),
        }),
    }
}

fn main() {
    let scope = match scope() {
        Ok(scope) => scope,
        Err(message) => {
            eprintln!("agent coding provider configuration refused: {message}");
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
                    json!({"protocolVersion":"2025-11-25","capabilities":{"tools":{}},"serverInfo":{"name":"tethers-agent-coding","version":"0.1.0"}}),
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
                call(&scope, name, &arguments)
            }
            "tools/list" | "tools/call" => Err(agent_coding::CodingError {
                code: "not_initialized",
                message: "MCP session is not initialized".into(),
            }),
            _ => Err(agent_coding::CodingError {
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
