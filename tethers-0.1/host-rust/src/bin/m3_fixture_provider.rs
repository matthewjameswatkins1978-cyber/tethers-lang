//! Native credential-free MCP fixture used only by M3 conformance tests.

use serde_json::{json, Value};
use std::io::{self, BufRead};

fn main() {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let mode = arguments
        .windows(2)
        .find(|pair| pair[0] == "--mode")
        .map(|pair| pair[1].as_str())
        .unwrap_or("valid");
    eprintln!("fixture stderr is intentionally untrusted: M3_SECRET_CANARY");
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let Ok(request) = serde_json::from_str::<Value>(&line) else {
            println!("{{malformed");
            continue;
        };
        let Some(id) = request.get("id").cloned() else {
            continue;
        };
        let method = request.get("method").and_then(Value::as_str).unwrap_or("");
        if mode == "hang" && method == "initialize" {
            std::thread::sleep(std::time::Duration::from_secs(60));
            continue;
        }
        if mode == "oversized" && method == "tools/list" {
            println!("{}", "x".repeat(2 * 1024 * 1024));
            continue;
        }
        if mode == "malformed" && method == "tools/list" {
            println!("{{malformed");
            continue;
        }
        let response = match method {
            "initialize" => json!({
                "jsonrpc":"2.0","id":id,"result":{
                    "protocolVersion":"2025-11-25",
                    "capabilities":{"tools":{"listChanged":false}},
                    "serverInfo":{"name":"tethers-stdio-fixture","version":"0.1.0"}
                }
            }),
            "tools/list" => json!({
                "jsonrpc":"2.0","id":id,"result":{"tools":[{
                    "name":"fixture_ping",
                    "inputSchema":{"type":"object","additionalProperties":true}
                }]}
            }),
            "tools/call" => {
                let invalid = request
                    .pointer("/params/arguments/__tethers_invalid")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                json!({
                    "jsonrpc":"2.0","id":id,"result":{
                        "content":[],
                        "structuredContent":{
                            "ambient_secret_present":std::env::var_os("TETHERS_TEST_AMBIENT_SECRET").is_some(),
                            "arguments":arguments,
                            "working_directory":std::env::current_dir().ok().map(|path| path.to_string_lossy().into_owned()),
                            "environment_names":std::env::vars_os().map(|(name, _)| name.to_string_lossy().into_owned()).collect::<Vec<_>>()
                        },
                        "isError":invalid
                    }
                })
            }
            _ => {
                json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":"method not found"}})
            }
        };
        println!("{response}");
    }
}
