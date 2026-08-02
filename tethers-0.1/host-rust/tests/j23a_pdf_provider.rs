//! J23A: stdio protocol evidence for the native PDF inspection provider.
//!
//! Every assertion comes from the real compiled binary over stdin/stdout, so
//! the MCP surface rather than an in-process shortcut is what is proved.

use serde_json::{json, Value};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdout, Command, Stdio};
use tethers_reference_host::pdf_tools::{inspect_input_schema, inspect_output_schema};
use uuid::Uuid;

/// One retained provider session. A single `BufReader` is kept for the whole
/// session because a fresh reader per request could discard buffered bytes
/// belonging to a later response.
struct Session {
    child: Child,
    reader: BufReader<ChildStdout>,
}

impl Session {
    fn start(query_root: &Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_pdf_tools_provider"))
            .args(["--query-root", query_root.to_str().unwrap()])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("pdf provider binary starts");
        let reader = BufReader::new(child.stdout.take().expect("stdout is piped"));
        Self { child, reader }
    }

    fn send(&mut self, message: Value) {
        let stdin = self.child.stdin.as_mut().expect("stdin is piped");
        writeln!(stdin, "{}", serde_json::to_string(&message).unwrap()).unwrap();
        stdin.flush().unwrap();
    }

    fn request(&mut self, message: Value) -> Value {
        self.send(message);
        let mut line = String::new();
        self.reader.read_line(&mut line).unwrap();
        serde_json::from_str(&line).expect("provider emits one JSON line per request")
    }

    fn initialize(&mut self) -> Value {
        let initialize = self.request(
            json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25"}}),
        );
        self.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
        initialize
    }

    /// Closes stdin and returns the provider exit status.
    fn finish(mut self) -> std::process::ExitStatus {
        drop(self.child.stdin.take());
        self.child.wait().unwrap()
    }
}

fn temp_root(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("tethers-j23a-{label}-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    root
}

fn minimal_pdf_bytes(page_count: u32) -> Vec<u8> {
    let mut pdf = String::from("%PDF-1.4\n%binary comment\n");
    for index in 1..=page_count {
        pdf.push_str(&format!(
            "{} 0 obj\n<< /Type /Page /Parent 99 0 R >>\nendobj\n",
            index + 1
        ));
    }
    pdf.push_str("trailer\n<< /Root 99 0 R >>\nstartxref\n0\n%%EOF\n");
    pdf.into_bytes()
}

#[test]
fn provider_initializes_and_lists_only_pdf_inspect() {
    let root = temp_root("list");
    let mut session = Session::start(&root);

    let initialize = session.initialize();
    assert_eq!(initialize["result"]["protocolVersion"], "2025-11-25");
    assert_eq!(
        initialize["result"]["serverInfo"]["name"],
        "tethers-pdf-provider"
    );
    assert_eq!(initialize["result"]["serverInfo"]["version"], "1.0.0");

    let tools = session.request(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}));
    let listed = tools["result"]["tools"].as_array().unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0]["name"], "pdf_inspect");
    assert_eq!(listed[0]["inputSchema"], inspect_input_schema());
    assert_eq!(listed[0]["outputSchema"], inspect_output_schema());

    assert!(session.finish().success());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn valid_pdf_call_returns_complete_inspection_fields() {
    let root = temp_root("inspect");
    let pdf = minimal_pdf_bytes(3);
    fs::write(root.join("sample.pdf"), &pdf).unwrap();

    let mut session = Session::start(&root);
    session.initialize();
    let call = session.request(
        json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"pdf_inspect","arguments":{"path":"sample.pdf"}}}),
    );
    let result = &call["result"];
    assert_eq!(result["path"], "sample.pdf");
    assert_eq!(result["size_bytes"], pdf.len() as u64);
    let digest = result["sha256"].as_str().unwrap();
    assert!(digest.starts_with("sha256:"));
    assert_eq!(digest.len(), "sha256:".len() + 64);
    assert!(digest["sha256:".len()..]
        .chars()
        .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)));
    assert_eq!(result["is_pdf"], true);
    assert_eq!(result["pdf_version"], "1.4");
    assert_eq!(result["page_count"], 3);

    assert!(session.finish().success());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn binary_non_utf8_body_is_accepted_over_stdio() {
    let root = temp_root("binary");
    let mut pdf = b"%PDF-1.7\n".to_vec();
    pdf.extend((0x80u8..0xFFu8).cycle().take(4096));
    pdf.extend(b"\n/Type /Page\n%%EOF\n");
    fs::write(root.join("binary.pdf"), &pdf).unwrap();

    let mut session = Session::start(&root);
    session.initialize();
    let call = session.request(
        json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"pdf_inspect","arguments":{"path":"binary.pdf"}}}),
    );
    assert_eq!(call["result"]["is_pdf"], true);
    assert_eq!(call["result"]["pdf_version"], "1.7");
    assert_eq!(call["result"]["page_count"], 1);
    assert_eq!(call["result"]["size_bytes"], pdf.len() as u64);

    assert!(session.finish().success());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn unknown_operation_and_unsupported_method_are_method_not_found() {
    let root = temp_root("unknown");
    let mut session = Session::start(&root);
    session.initialize();

    let unknown_tool = session.request(
        json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"pdf_extract","arguments":{}}}),
    );
    assert_eq!(unknown_tool["error"]["code"], -32601);

    let unknown_method =
        session.request(json!({"jsonrpc":"2.0","id":3,"method":"resources/list","params":{}}));
    assert_eq!(unknown_method["error"]["code"], -32601);

    assert!(session.finish().success());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn tools_list_before_initialization_is_refused() {
    let root = temp_root("uninitialized");
    let mut session = Session::start(&root);
    let tools = session.request(json!({"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}));
    assert_eq!(tools["error"]["code"], -32602);
    assert!(tools.get("result").is_none());
    assert!(session.finish().success());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn invalid_arguments_and_directory_paths_are_refused() {
    let root = temp_root("invalid");
    fs::create_dir_all(root.join("folder")).unwrap();
    let mut session = Session::start(&root);
    session.initialize();

    for (id, arguments) in [
        (2, json!({})),
        (3, json!({"path": 7})),
        (4, json!({"path": "sample.pdf", "extra": true})),
        (5, json!({"path": "../escape.pdf"})),
        (6, json!({"path": "absent.pdf"})),
        (7, json!({"path": "folder"})),
    ] {
        let refused = session.request(
            json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":"pdf_inspect","arguments":arguments}}),
        );
        assert_eq!(refused["error"]["code"], -32602, "arguments case {id}");
        assert!(refused.get("result").is_none(), "arguments case {id}");
    }

    assert!(session.finish().success());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn an_unusable_query_root_refuses_to_start_the_request_loop() {
    let absent = std::env::temp_dir().join(format!("tethers-j23a-absent-{}", Uuid::new_v4()));
    let refused = Command::new(env!("CARGO_BIN_EXE_pdf_tools_provider"))
        .args(["--query-root", absent.to_str().unwrap()])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    assert!(!refused.status.success());
    assert!(refused.stdout.is_empty());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("configuration refused"));

    let without_root = Command::new(env!("CARGO_BIN_EXE_pdf_tools_provider"))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    assert!(!without_root.status.success());
    assert!(without_root.stdout.is_empty());
}
