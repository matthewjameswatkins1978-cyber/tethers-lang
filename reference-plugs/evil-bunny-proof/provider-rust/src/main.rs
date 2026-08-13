//! The Evil Bunny — a safe, deterministic adversarial MCP stdio provider.
//!
//! This binary is a protocol test fixture for the P6 adversarial-provider
//! proof ("The Evil Bunny Test").  It can lie, hang, crash, emit malformed
//! protocol, or advertise false contracts at the Tethers/MCP boundary.  It
//! deliberately does NOT damage files, escape the filesystem, touch
//! credentials, attack the network, spawn uncontrolled processes, persist
//! itself, or perform any destructive action.  Process supervision is NOT a
//! security sandbox; this fixture never claims otherwise.
//!
//! One binary, many deterministic modes, selected by the launch argument
//! `--mode <mode>`.  Each mode violates exactly one primary protocol contract
//! wherever practical so the experiment log stays readable.  stdout carries
//! only protocol lines; diagnostics go to stderr.

use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

const PROTOCOL_VERSION: &str = "2025-11-25";
const SERVER_NAME: &str = "tethers-evil-bunny-provider";
const TOOL_NAME: &str = "evil_probe";

fn input_schema() -> Value {
    serde_json::from_str(
        r#"{"type":"object","properties":{"message":{"type":"string"}},"required":["message"],"additionalProperties":false}"#,
    )
    .expect("static input schema is valid JSON")
}

fn output_schema() -> Value {
    serde_json::from_str(
        r#"{"type":"object","properties":{"echo":{"type":"string"}},"required":["echo"],"additionalProperties":false}"#,
    )
    .expect("static output schema is valid JSON")
}

fn argument_value<'a>(arguments: &'a [String], name: &str) -> Option<&'a str> {
    arguments
        .windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].as_str())
}

fn mode_of(arguments: &[String]) -> String {
    argument_value(arguments, "--mode")
        .unwrap_or("good")
        .to_owned()
}

/// A protocol write decision.  `Silent` lets the host observe a missing
/// response.
enum Output {
    Respond(Value),
    Silent,
}

fn main() -> std::process::ExitCode {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let mode = mode_of(&arguments);

    match mode.as_str() {
        "early-death" => {
            eprintln!("evil-bunny: dying before any protocol exchange");
            return std::process::ExitCode::from(7);
        }
        "malformed-stdout" => {
            eprintln!("evil-bunny: emitting a malformed stdout line");
            let stdout = io::stdout();
            let mut out = stdout.lock();
            let _ = writeln!(out, "not json");
            let _ = out.flush();
            return std::process::ExitCode::from(0);
        }
        _ => {}
    }

    if let Err(error) = run_mcp_loop(&mode) {
        eprintln!("evil-bunny: stdio failure: {error}");
        return std::process::ExitCode::from(1);
    }

    if mode == "shutdown-refusal" {
        eprintln!("evil-bunny: refusing graceful shutdown");
        loop {
            std::thread::sleep(std::time::Duration::from_secs(3600));
        }
    }
    std::process::ExitCode::SUCCESS
}

fn run_mcp_loop(mode: &str) -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let Ok(request) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let Some(id) = request.get("id").cloned() else {
            continue; // notification: no response on stdout
        };
        let method = request.get("method").and_then(Value::as_str).unwrap_or("");
        match dispatch(mode, method, &id) {
            Output::Respond(response) => {
                writeln!(out, "{response}")?;
                out.flush()?;
            }
            Output::Silent => {}
        }
    }
    Ok(())
}

fn dispatch(mode: &str, method: &str, id: &Value) -> Output {
    if mode == "silent" {
        return Output::Silent;
    }
    match method {
        "initialize" => initialize_response(mode, id),
        "tools/list" => tools_list_response(mode, id),
        "ping" => Output::Respond(response_with_id(mode, id, Value::Null)),
        _ => Output::Respond(json!({
            "jsonrpc":"2.0",
            "id": id,
            "error":{"code":-32601,"message":"method not found"}
        })),
    }
}

/// The wrong-response-id mode binds an otherwise plausible response to a
/// different request id.  Other modes echo the request id.
fn response_id(mode: &str, id: &Value) -> Value {
    if mode == "wrong-response-id" {
        id.as_i64()
            .map(|number| json!(number + 1000))
            .unwrap_or_else(|| json!("definitely-not-the-request-id"))
    } else {
        id.clone()
    }
}

fn response_with_id(mode: &str, id: &Value, result: Value) -> Value {
    json!({"jsonrpc":"2.0","id":response_id(mode, id),"result":result})
}

fn initialize_response(mode: &str, id: &Value) -> Output {
    let server_name = if mode == "identity-liar" {
        "tethers-impostor-provider"
    } else {
        SERVER_NAME
    };
    let protocol_version = if mode == "protocol-liar" {
        "2024-11-05"
    } else {
        PROTOCOL_VERSION
    };
    let result = json!({
        "protocolVersion": protocol_version,
        "capabilities":{"tools":{}},
        "serverInfo":{"name":server_name,"version":env!("CARGO_PKG_VERSION")}
    });
    Output::Respond(response_with_id(mode, id, result))
}

/// The operation advertised in `tools/list`.  `wrong-name` renames the
/// operation; the schema-liar modes keep the reviewed name but advertise a
/// false schema; `output-schema-omitted` advertises only `inputSchema`.
fn tool_definition(mode: &str) -> Value {
    let name = if mode == "wrong-name" {
        "evil_probe_v2"
    } else {
        TOOL_NAME
    };
    let input = if mode == "input-schema-liar" {
        json!({"type":"object","properties":{"message":{"type":"string"}},"required":["different"],"additionalProperties":false})
    } else {
        input_schema()
    };
    let output = if mode == "output-schema-liar" {
        json!({"type":"object","properties":{"echo":{"type":"string"}},"required":["different"],"additionalProperties":false})
    } else {
        output_schema()
    };
    let mut tool = json!({
        "name": name,
        "description": "Evil Bunny probe operation",
        "inputSchema": input
    });
    if mode != "output-schema-omitted" {
        tool["outputSchema"] = output;
    }
    tool
}

fn tools_list_response(mode: &str, id: &Value) -> Output {
    match mode {
        "missing-operation" => Output::Respond(response_with_id(mode, id, json!({"tools": []}))),
        "surprise-operation" => {
            let mut tools = vec![tool_definition(mode)];
            tools.push(json!({
                "name": "surprise_tool",
                "description": "an undeclared extra operation",
                "inputSchema": {"type":"object"},
                "outputSchema": {"type":"object"}
            }));
            Output::Respond(response_with_id(mode, id, json!({"tools": tools})))
        }
        _ => Output::Respond(response_with_id(
            mode,
            id,
            json!({"tools": [tool_definition(mode)]}),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(mode: &str, id: i64, method: &str) -> Output {
        dispatch(mode, method, &json!(id))
    }

    #[test]
    fn good_bunny_advertises_exact_reviewed_discovery() {
        match request("good", 2, "tools/list") {
            Output::Respond(response) => {
                let tool = response["result"]["tools"][0].clone();
                assert_eq!(tool["name"], TOOL_NAME);
                assert_eq!(tool["inputSchema"], input_schema());
                assert_eq!(tool["outputSchema"], output_schema());
            }
            other => panic!(
                "expected a response, got {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }

    #[test]
    fn good_bunny_identity_and_protocol_are_reviewed_values() {
        match request("good", 1, "initialize") {
            Output::Respond(response) => {
                assert_eq!(response["result"]["protocolVersion"], PROTOCOL_VERSION);
                assert_eq!(response["result"]["serverInfo"]["name"], SERVER_NAME);
                assert_eq!(
                    response["result"]["serverInfo"]["version"],
                    env!("CARGO_PKG_VERSION")
                );
                assert_eq!(response["id"], 1);
            }
            other => panic!(
                "expected a response, got {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }

    #[test]
    fn identity_liar_claims_a_different_server_name() {
        match request("identity-liar", 1, "initialize") {
            Output::Respond(response) => {
                assert_ne!(response["result"]["serverInfo"]["name"], SERVER_NAME);
            }
            other => panic!(
                "expected a response, got {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }

    #[test]
    fn output_schema_omitted_advertises_no_output_schema() {
        match request("output-schema-omitted", 2, "tools/list") {
            Output::Respond(response) => {
                let tool = response["result"]["tools"][0].clone();
                assert!(tool.get("outputSchema").is_none());
            }
            other => panic!(
                "expected a response, got {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }

    #[test]
    fn wrong_response_id_binds_to_a_different_id() {
        match request("wrong-response-id", 1, "initialize") {
            Output::Respond(response) => {
                assert_eq!(response["id"], 1001);
            }
            other => panic!(
                "expected a response, got {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }
}
