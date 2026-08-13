use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use serde_json::Value;

#[test]
fn stdout_is_mcp_protocol_only() {
    let temp = std::env::temp_dir().join(format!(
        "text-stats-provider-stdout-test-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&temp).expect("create temp dir");

    let mut child = Command::new(env!("CARGO_BIN_EXE_TETHERS_TEXT_STATS_PROVIDER"))
        .env("TETHERS_CONFORMANCE", "1")
        .env("TEMP", &temp)
        .env("TMP", &temp)
        .env_remove("TETHERS_OPERATIONAL_SCOPE_JSON")
        .env_remove("TETHERS_OPERATIONAL_SCOPE_DIGEST")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn provider");

    let mut stdin = child.stdin.take().expect("child stdin");
    let mut stdout = BufReader::new(child.stdout.take().expect("child stdout"));

    fn send(stdin: &mut std::process::ChildStdin, message: &str) {
        writeln!(stdin, "{message}").expect("write to provider stdin");
        stdin.flush().expect("flush provider stdin");
    }

    fn read_json(stdout: &mut BufReader<std::process::ChildStdout>) -> Value {
        let mut line = String::new();
        stdout.read_line(&mut line).expect("read provider stdout");
        assert!(!line.is_empty(), "provider stdout closed early");
        serde_json::from_str(line.trim())
            .expect("every provider stdout line must be exactly one JSON object")
    }

    send(
        &mut stdin,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"text-stats-test","version":"1.0.0"}}}"#,
    );
    let init = read_json(&mut stdout);
    assert_eq!(init["jsonrpc"], "2.0");
    assert_eq!(init["id"], 1);
    assert_eq!(init["result"]["protocolVersion"], "2025-11-25");
    assert_eq!(
        init["result"]["serverInfo"]["name"],
        "tethers-text-stats-provider"
    );

    send(
        &mut stdin,
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
    );

    send(
        &mut stdin,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
    );
    let tools = read_json(&mut stdout);
    assert_eq!(tools["id"], 2);
    assert_eq!(tools["result"]["tools"][0]["name"], "text_stats");
    assert_eq!(
        tools["result"]["tools"][0]["inputSchema"]["required"][0],
        "path"
    );
    assert_eq!(
        tools["result"]["tools"][0]["outputSchema"]["required"],
        serde_json::json!([
            "path",
            "size_bytes",
            "sha256",
            "line_count",
            "word_count",
            "character_count"
        ])
    );

    send(
        &mut stdin,
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"text_stats","arguments":{}}}"#,
    );
    let missing = read_json(&mut stdout);
    assert_eq!(missing["id"], 3);
    assert_eq!(missing["error"]["code"], -32602);

    send(
        &mut stdin,
        r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"text_stats","arguments":{"path":"../escape.txt"}}}"#,
    );
    let traversal = read_json(&mut stdout);
    assert_eq!(traversal["id"], 4);
    assert_eq!(traversal["result"]["isError"], true);

    child.kill().expect("kill provider");
    child.wait().expect("wait provider");
    let _ = std::fs::remove_dir_all(&temp);
}
