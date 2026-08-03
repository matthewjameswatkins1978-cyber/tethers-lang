//! J23C2: generated PDF package passes host conformance.
//!
//! Part A proves the `pdf_tools_provider` binary resolves the reviewed
//! `--query-root` placeholder only under host conformance, while every normal
//! and refused path behaves exactly as before. Part B builds the real provider
//! package and runs it through the existing generic M3 host conformance flow.

#![cfg(windows)]

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use serde_json::Value;
use tethers_reference_host::candidate::{extract_to_quarantine, CandidateRegistry};
use tethers_reference_host::conformance::{
    run_host_conformance, CaseDisposition, ConformanceDisposition, ConformanceEvidence,
};
use tethers_reference_host::launch_profile::PreparedSupervisedLaunch;
use tethers_reference_host::package;
use tethers_reference_host::pdf_tools;
use tethers_reference_host::trust::{
    DeveloperApprovalStore, PackageTrustEvidence, PublisherTrustStore,
};
use uuid::Uuid;

const PDF_QUERY_ROOT_PLACEHOLDER: &str = "__TETHERS_PDF_QUERY_ROOT__";
const TETHERS_CONFORMANCE: &str = "TETHERS_CONFORMANCE";
const TEMP: &str = "TEMP";

fn provider_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_pdf_tools_provider"))
}

fn temp_root(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("tethers-j23c2-{label}-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    root
}

/// Spawn the provider with stdin/stdout piped and the given extra environment.
fn spawn_session(
    query_root: &str,
    env: &[(&str, &str)],
) -> (std::process::Child, BufReader<std::process::ChildStdout>) {
    let mut command = provider_bin();
    command.args(["--query-root", query_root]);
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::null());
    for (key, value) in env {
        command.env(key, value);
    }
    let mut child = command.spawn().expect("pdf provider starts");
    let reader = BufReader::new(child.stdout.take().expect("stdout piped"));
    (child, reader)
}

fn send(session: &mut (std::process::Child, BufReader<std::process::ChildStdout>), message: Value) {
    let stdin = session.0.stdin.as_mut().expect("stdin piped");
    writeln!(stdin, "{}", serde_json::to_string(&message).unwrap()).unwrap();
    stdin.flush().unwrap();
}

fn request(
    session: &mut (std::process::Child, BufReader<std::process::ChildStdout>),
    message: Value,
) -> Value {
    send(session, message);
    let mut line = String::new();
    session.1.read_line(&mut line).unwrap();
    serde_json::from_str(&line).expect("provider emits one JSON line per request")
}

// -- Part A: provider startup proofs --

#[test]
fn normal_explicit_absolute_query_root_starts() {
    let root = temp_root("normal");
    let mut session = spawn_session(root.to_str().unwrap(), &[]);
    let initialize = request(
        &mut session,
        serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25"}}),
    );
    assert_eq!(initialize["result"]["protocolVersion"], "2025-11-25");
    assert_eq!(
        initialize["result"]["serverInfo"]["name"],
        "tethers-pdf-provider"
    );
    send(
        &mut session,
        serde_json::json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
    );
    let tools = request(
        &mut session,
        serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
    );
    assert_eq!(tools["result"]["tools"].as_array().unwrap().len(), 1);
    assert_eq!(tools["result"]["tools"][0]["name"], "pdf_inspect");
    drop(session.0.stdin.take());
    assert!(session.0.wait().unwrap().success());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn placeholder_without_conformance_is_refused() {
    let refused = provider_bin()
        .args(["--query-root", PDF_QUERY_ROOT_PLACEHOLDER])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_remove(TETHERS_CONFORMANCE)
        .output()
        .unwrap();
    assert!(!refused.status.success());
    assert!(refused.stdout.is_empty());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("configuration refused"));
}

#[test]
fn placeholder_with_conformance_zero_is_refused() {
    let refused = provider_bin()
        .args(["--query-root", PDF_QUERY_ROOT_PLACEHOLDER])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env(TETHERS_CONFORMANCE, "0")
        .output()
        .unwrap();
    assert!(!refused.status.success());
    assert!(refused.stdout.is_empty());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("configuration refused"));
}

#[test]
fn unknown_pdf_placeholder_is_refused() {
    let refused = provider_bin()
        .args(["--query-root", "__TETHERS_PDF_OTHER__"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    assert!(!refused.status.success());
    assert!(refused.stdout.is_empty());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("configuration refused"));
}

#[test]
fn placeholder_with_conformance_and_valid_temp_starts_and_passes_protocol() {
    let scratch = temp_root("conf-scratch");
    let mut session = spawn_session(
        PDF_QUERY_ROOT_PLACEHOLDER,
        &[
            ("TETHERS_CONFORMANCE", "1"),
            (TEMP, scratch.to_str().unwrap()),
        ],
    );
    let initialize = request(
        &mut session,
        serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25"}}),
    );
    assert_eq!(initialize["result"]["protocolVersion"], "2025-11-25");
    assert_eq!(
        initialize["result"]["serverInfo"]["name"],
        "tethers-pdf-provider"
    );
    send(
        &mut session,
        serde_json::json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
    );
    let tools = request(
        &mut session,
        serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
    );
    let listed = tools["result"]["tools"].as_array().unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0]["name"], "pdf_inspect");
    drop(session.0.stdin.take());
    assert!(session.0.wait().unwrap().success());
    fs::remove_dir_all(scratch).unwrap();
}

#[test]
fn conformance_mode_with_missing_temp_is_refused() {
    let refused = provider_bin()
        .args(["--query-root", PDF_QUERY_ROOT_PLACEHOLDER])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env(TETHERS_CONFORMANCE, "1")
        .env_remove(TEMP)
        .output()
        .unwrap();
    assert!(!refused.status.success());
    assert!(refused.stdout.is_empty());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("configuration refused"));
}

#[test]
fn conformance_mode_with_invalid_or_relative_temp_is_refused() {
    let relative = temp_root("conf-relative");
    let relative_temp = relative.file_name().unwrap().to_string_lossy().into_owned();
    let refused_relative = provider_bin()
        .args(["--query-root", PDF_QUERY_ROOT_PLACEHOLDER])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env(TETHERS_CONFORMANCE, "1")
        .env(TEMP, &relative_temp)
        .output()
        .unwrap();
    assert!(!refused_relative.status.success());
    assert!(refused_relative.stdout.is_empty());
    assert!(String::from_utf8_lossy(&refused_relative.stderr).contains("configuration refused"));

    let absent = std::env::temp_dir().join(format!("tethers-j23c2-absent-{}", Uuid::new_v4()));
    let refused_absent = provider_bin()
        .args(["--query-root", PDF_QUERY_ROOT_PLACEHOLDER])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env(TETHERS_CONFORMANCE, "1")
        .env(TEMP, absent.to_str().unwrap())
        .output()
        .unwrap();
    assert!(!refused_absent.status.success());
    assert!(refused_absent.stdout.is_empty());
    assert!(String::from_utf8_lossy(&refused_absent.stderr).contains("configuration refused"));
    fs::remove_dir_all(relative).unwrap();
}

// -- Part B: complete package conformance --

fn assert_case_passed(evidence: &ConformanceEvidence, case_id: &str) {
    let case = evidence
        .cases
        .iter()
        .find(|case| case.case_id == case_id)
        .unwrap_or_else(|| panic!("conformance case {case_id} is missing"));
    assert_eq!(case.disposition, CaseDisposition::Passed, "case {case_id}");
}

#[test]
fn generated_pdf_package_passes_host_conformance() {
    let base = temp_root("conformance");
    let archive = base.join("pdf-tools.tetherplug");
    let provider_bytes =
        fs::read(env!("CARGO_BIN_EXE_pdf_tools_provider")).expect("compiled provider");
    fs::write(
        &archive,
        pdf_tools::build_reference_package(&provider_bytes).unwrap(),
    )
    .unwrap();

    let report = package::inspect(&archive).unwrap();
    let quarantined = extract_to_quarantine(&report, &base.join("quarantine")).unwrap();
    let candidates =
        CandidateRegistry::open(&base.join("candidates"), &base.join("quarantine")).unwrap();
    let candidate = candidates.create(&quarantined).unwrap();
    let developers = DeveloperApprovalStore::open(&base.join("developer")).unwrap();
    let developer = developers
        .approve_exact_digest(&candidate.semantic_package_digest, "Matthew")
        .unwrap();
    let trust = PackageTrustEvidence::unsigned(&developer).unwrap();
    let publishers = PublisherTrustStore::open(&base.join("publishers")).unwrap();
    let prepared = PreparedSupervisedLaunch::prepare(
        &candidate,
        &base.join("quarantine"),
        &base.join("scratch"),
        Duration::from_secs(5),
    )
    .unwrap();
    let conformance = run_host_conformance(
        &prepared,
        &candidate,
        &base.join("quarantine"),
        &trust,
        &publishers,
        &developers,
        "tethers-reference-host@0.2.0+j23c2",
    )
    .unwrap();

    // Package identity.
    assert_eq!(conformance.package_id, "tethers.pdf-tools");
    assert_eq!(conformance.package_version, "1.0.0");
    assert_eq!(conformance.provider_id, "tethers-pdf-provider");

    // Declared launch arguments keep the exact placeholder untouched.
    let expected_arguments = vec![
        "--query-root".to_string(),
        PDF_QUERY_ROOT_PLACEHOLDER.to_string(),
    ];
    assert_eq!(candidate.launch_arguments, expected_arguments);
    assert_eq!(prepared.evidence.arguments, expected_arguments);

    // Conformance disposition and bounded retry/stderr behaviour.
    assert_eq!(conformance.disposition, ConformanceDisposition::Passed);
    assert_eq!(conformance.retry_count, 0);
    assert!(!conformance.raw_stderr_persisted);

    // Required individual cases passed.
    assert_case_passed(&conformance, "provider_identity");
    assert_case_passed(&conformance, "mcp_initialize_protocol_pin");
    assert_case_passed(&conformance, "complete_discovery_exact_operations");
    assert_case_passed(&conformance, "bounded_shutdown_process_cleanup");

    // No conformance_session failure exists.
    assert!(
        conformance
            .cases
            .iter()
            .find(|case| case.case_id == "conformance_session")
            .is_none(),
        "unexpected conformance_session failure: {:?}",
        conformance.cases
    );

    // Capability evidence remains pdf.inspect@1 with the frozen manifest digest.
    assert_eq!(conformance.capabilities.len(), 1);
    assert_eq!(conformance.capabilities[0].name, "pdf.inspect");
    assert_eq!(conformance.capabilities[0].version, 1);
    assert_eq!(
        conformance.capabilities[0].manifest_digest,
        "sha256:26da081128608859c1259da7ddd784d343241504cb47339ca54a9b5979b6297c"
    );

    // Clean up prepared scratch and test directories; do not install or enable.
    prepared.cleanup_scratch().unwrap();
    fs::remove_dir_all(&base).unwrap();
}
