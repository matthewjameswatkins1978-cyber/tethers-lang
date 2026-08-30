use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

fn run(args: &[&str], input: &str) -> (i32, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tethers"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("portable CLI should start");
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(input.as_bytes())
        .expect("request should be written");
    let output = child.wait_with_output().expect("portable CLI should exit");
    (
        output.status.code().expect("CLI should return a code"),
        String::from_utf8(output.stdout).expect("stdout should be UTF-8"),
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
    )
}

fn assert_decision(args: &[&str], input: &str, expected: &str) {
    let (code, stdout, stderr) = run(args, input);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stderr.is_empty(), "diagnostics leaked to stderr: {stderr}");
    let response: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON stdout");
    assert_eq!(response["decision"], expected);
}

#[test]
fn stdin_returns_allow_ask_and_deny() {
    assert_decision(
        &["evaluate"],
        r#"{"action":{"name":"a","version":1},"context":{},"policy":{"default":"deny","rules":[{"name":"a","version":1,"decision":"allow"}]}}"#,
        "ALLOW",
    );
    assert_decision(
        &["evaluate"],
        r#"{"action":{"name":"b","version":1},"context":{},"policy":{"default":"deny","rules":[{"name":"b","version":1,"decision":"ask"}]}}"#,
        "ASK",
    );
    assert_decision(
        &["evaluate"],
        r#"{"action":{"name":"c","version":1},"context":{},"policy":{"default":"deny","rules":[{"name":"c","version":1,"decision":"deny"}]}}"#,
        "DENY",
    );
}

#[test]
fn malformed_and_missing_policy_fail_closed_as_json() {
    let (code, stdout, stderr) = run(&["evaluate"], "{");
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stderr.is_empty());
    let response: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(response["decision"], "DENY");
    assert!(response["error"]
        .as_str()
        .unwrap()
        .starts_with("invalid request JSON:"));

    let (code, stdout, stderr) = run(
        &["evaluate"],
        r#"{"action":{"name":"a","version":1},"context":{}}"#,
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stderr.is_empty());
    let response: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(response["decision"], "DENY");
    assert_eq!(response["error"], "missing policy");
}

#[test]
fn input_and_policy_files_are_supported() {
    let root =
        std::env::temp_dir().join(format!("tethers-portable-cli-test-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let input_path = root.join("request.json");
    let policy_path = root.join("policy.json");
    fs::write(
        &input_path,
        r#"{"action":{"name":"file.action","version":1},"context":{}}"#,
    )
    .unwrap();
    fs::write(
        &policy_path,
        r#"{"default":"deny","rules":[{"name":"file.action","version":1,"decision":"allow"}]}"#,
    )
    .unwrap();

    let input = input_path.to_str().unwrap();
    let policy = policy_path.to_str().unwrap();
    assert_decision(
        &["evaluate", "--input", input, "--policy", policy],
        "",
        "ALLOW",
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn unknown_fields_fail_closed_without_non_json_output() {
    let (code, stdout, stderr) = run(
        &["evaluate"],
        r#"{"action":{"name":"a","version":1},"context":{},"policy":{"default":"allow","rules":[],"when":true}}"#,
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stderr.is_empty());
    let response: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(response["decision"], "DENY");
    assert!(response["error"]
        .as_str()
        .unwrap()
        .contains("unknown field"));
}
