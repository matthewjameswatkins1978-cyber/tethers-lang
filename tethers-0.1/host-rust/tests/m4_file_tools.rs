#![cfg(windows)]

use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use tethers_reference_host::file_tools::{build_reference_package, manifest_with_digest, metadata_manifest_without_digest, move_manifest_without_digest};
use uuid::Uuid;

fn request(child: &mut std::process::Child, message: Value) -> Value {
    let stdin = child.stdin.as_mut().unwrap();
    writeln!(stdin, "{}", serde_json::to_string(&message).unwrap()).unwrap();
    stdin.flush().unwrap();
    let stdout = child.stdout.as_mut().unwrap();
    let mut line = String::new();
    BufReader::new(stdout).read_line(&mut line).unwrap();
    serde_json::from_str(&line).unwrap()
}

#[test]
fn native_file_tools_provider_performs_query_and_non_overwriting_move() {
    let root = std::env::temp_dir().join(format!("tethers-m4-provider-{}", Uuid::new_v4()));
    let query = root.join("query");
    let source = root.join("source");
    let destination = root.join("destination");
    fs::create_dir_all(&query).unwrap();
    fs::create_dir_all(&source).unwrap();
    fs::create_dir_all(&destination).unwrap();
    fs::write(query.join("hello.txt"), b"hello").unwrap();
    fs::write(source.join("move.txt"), b"move").unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_file_tools_provider"))
        .args(["--query-root", query.to_str().unwrap(), "--source-root", source.to_str().unwrap(), "--destination-root", destination.to_str().unwrap()])
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().unwrap();
    let initialize = request(&mut child, serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25"}}));
    assert_eq!(initialize["result"]["protocolVersion"], "2025-11-25");
    let stdin = child.stdin.as_mut().unwrap();
    writeln!(stdin, "{{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}}").unwrap();
    stdin.flush().unwrap();
    let tools = request(&mut child, serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}));
    assert_eq!(tools["result"]["tools"].as_array().unwrap().len(), 2);
    let metadata = request(&mut child, serde_json::json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"file_metadata","arguments":{"path":"hello.txt","include_content":true}}}));
    assert_eq!(metadata["result"]["content"], "hello");
    fs::write(destination.join("move.txt"), b"existing").unwrap();
    let moved = request(&mut child, serde_json::json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"file_move","arguments":{"source_path":"move.txt","destination_path":"move.txt"}}}));
    assert_eq!(moved["error"]["code"], -32602);
    assert!(source.join("move.txt").exists());
    drop(child.stdin.take());
    let _ = child.wait();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn m4_contract_and_package_digest_material_is_stable() {
    let metadata = manifest_with_digest(metadata_manifest_without_digest()).unwrap();
    let movement = manifest_with_digest(move_manifest_without_digest()).unwrap();
    assert_eq!(metadata["capability_name"], "file.metadata");
    assert_eq!(movement["capability_name"], "file.move");
    assert_eq!(build_reference_package(b"provider"), build_reference_package(b"provider"));
}
