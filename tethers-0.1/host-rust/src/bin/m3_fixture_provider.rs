//! Native credential-free MCP fixture used only by M3 conformance tests.

use serde_json::{json, Value};
use std::io::{self, BufRead};
use std::process::Command;

fn argument_value<'a>(arguments: &'a [String], name: &str) -> Option<&'a str> {
    arguments
        .windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].as_str())
}

fn unrelated_inheritable_handle_accessible(arguments: &[String]) -> Option<bool> {
    let raw = argument_value(arguments, "--unrelated-inheritable-handle")?;
    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::HANDLE;
        use windows_sys::Win32::System::Threading::WaitForSingleObject;
        let Ok(raw) = raw.parse::<isize>() else {
            return Some(false);
        };
        // Use WaitForSingleObject to test whether the raw handle value maps
        // to a waitable handle object.  The parent signals the event before
        // launch, so WAIT_OBJECT_0 proves the test-handle was inherited
        // (a colliding file-handle or registry-key would return WAIT_FAILED).
        Some(unsafe { WaitForSingleObject(raw as HANDLE, 0) } != 0xFFFF_FFFF)
    }
    #[cfg(not(windows))]
    {
        let _ = raw;
        Some(false)
    }
}

fn main() {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let mode = arguments
        .windows(2)
        .find(|pair| pair[0] == "--mode")
        .map(|pair| pair[1].as_str())
        .unwrap_or("valid");
    if let Some(marker) = argument_value(&arguments, "--provider-marker") {
        let _ = std::fs::write(marker, b"provider process created");
    }
    let handle_canary_accessible = unrelated_inheritable_handle_accessible(&arguments);
    let startup_child = if mode.starts_with("spawn-child") {
        let child = Command::new("cmd")
            .args(["/c", "ping", "-t", "127.0.0.1"])
            .spawn()
            .ok();
        if let Some(child) = &child {
            if let Some(scratch) = std::env::var_os("TEMP") {
                let _ = std::fs::write(
                    std::path::Path::new(&scratch).join("m3-startup-child.pid"),
                    child.id().to_string(),
                );
            }
        }
        child
    } else {
        None
    };
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
        if (mode == "malformed" || mode == "spawn-child-malformed") && method == "tools/list" {
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
            "tools/list" if mode == "paginated" && request.pointer("/params/cursor").is_none() => {
                json!({
                    "jsonrpc":"2.0","id":id,"result":{"tools":[],"nextCursor":"page-2"}
                })
            }
            "tools/list" => {
                let input_schema = if mode == "wrong-schema" {
                    json!({"type":"object","additionalProperties":true})
                } else {
                    json!({
                        "type":"object",
                        "properties":{"message":{"type":"string"}},
                        "required":["message"],
                        "additionalProperties":false
                    })
                };
                json!({
                "jsonrpc":"2.0","id":id,"result":{"tools":[{
                    "name":"fixture_ping",
                    "inputSchema":input_schema
                }]}
                })
            }
            "tools/call" => {
                let invalid = request
                    .pointer("/params/arguments/__tethers_invalid")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let message = request
                    .pointer("/params/arguments/message")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let structured_content = if mode == "wrong-output" {
                    json!({"unexpected":message})
                } else {
                    json!({"echo":message})
                };
                json!({
                    "jsonrpc":"2.0","id":id,"result":{
                        "content":[],
                        "structuredContent":structured_content,
                        "tethersFixtureEvidence":{
                            "ambient_secret_present":std::env::var_os("TETHERS_TEST_AMBIENT_SECRET").is_some(),
                            "startup_child_pid":startup_child.as_ref().map(std::process::Child::id),
                            "arguments":arguments,
                            "working_directory":std::env::current_dir().ok().map(|path| path.to_string_lossy().into_owned()),
                            "environment_names":std::env::vars_os().map(|(name, _)| name.to_string_lossy().into_owned()).collect::<Vec<_>>(),
                            "unrelated_inheritable_handle_canary_requested":handle_canary_accessible.is_some(),
                            "unrelated_inheritable_handle_accessible":handle_canary_accessible
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
