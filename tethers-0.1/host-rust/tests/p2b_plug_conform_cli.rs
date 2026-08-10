//! P2B primary public proof: plug conform via real CLI.

use serde_json::Value;
use std::fs;
use std::process::Command;

fn host_binary() -> std::path::PathBuf {
    std::env::var_os("CARGO_BIN_EXE_tethers-reference-host")
        .or_else(|| std::env::var_os("CARGO_BIN_EXE_tethers_reference_host"))
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::current_exe().ok().and_then(|path| {
                path.parent()?
                    .parent()
                    .map(|dir| dir.join("tethers-reference-host.exe"))
            })
        })
        .expect("compiled reference host binary")
}

fn fixture_provider_binary() -> std::path::PathBuf {
    std::env::var_os("CARGO_BIN_EXE_m3_fixture_provider")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::current_exe().ok().and_then(|path| {
                path.parent()?
                    .parent()
                    .map(|dir| dir.join("m3_fixture_provider.exe"))
            })
        })
        .expect("compiled m3 fixture provider binary")
}

fn source_dir(label: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "tethers-p2b-{}-{}",
        label,
        uuid::Uuid::new_v4()
    ));
    let source = root.join("example-conformance-fixture");
    fs::create_dir_all(source.join("provider")).unwrap();
    fs::create_dir_all(source.join("manifests")).unwrap();

    let fixture_exe = fixture_provider_binary();
    fs::copy(&fixture_exe, source.join("provider/tethers-stdio-fixture.exe")).unwrap();

    let plug_json = serde_json::json!({
        "package_format_version": "1",
        "package_id": "example.conformance-fixture",
        "package_version": "0.1.0",
        "display_name": "Conformance Fixture Provider",
        "description": "Test plug for P2B conformance proof",
        "publisher": "tethers-example",
        "licence": "MIT",
        "socket_major": 1,
        "protocol_bindings": [{
            "protocol": "MCP",
            "version": "2025-11-25",
            "transport": "stdio"
        }],
        "platforms": [{
            "os": "windows",
            "architecture": "x86_64"
        }],
        "provider": {
            "provider_id": "tethers-stdio-fixture",
            "provider_version": "0.1.0",
            "launch": {
                "path": "provider/tethers-stdio-fixture.exe",
                "arguments": ["--mode", "valid"]
            },
            "working_directory": "provider",
            "capability_operation_namespace": "fixture"
        },
        "capabilities": [{
            "capability_name": "fixture.ping",
            "capability_version": 1,
            "manifest_path": "manifests/fixture-ping.json",
            "provider_operation_name": "fixture_ping"
        }],
        "payloads": [
            {"path": "manifests/fixture-ping.json", "role": "capability_manifest"},
            {"path": "provider/tethers-stdio-fixture.exe", "role": "provider_executable"}
        ]
    });

    let manifest_json = serde_json::json!({
        "manifest_format_version": "1.0",
        "capability_name": "fixture.ping",
        "capability_version": 1,
        "title": "Fixture Ping",
        "description": "Simple ping fixture for conformance testing",
        "input_schema": {
            "type": "object",
            "properties": {
                "message": {"type": "string"}
            },
            "required": ["message"],
            "additionalProperties": false
        },
        "output_schema": {
            "type": "object",
            "properties": {
                "echo": {"type": "string"}
            },
            "required": ["echo"],
            "additionalProperties": false
        },
        "effects": ["fixture.echo"],
        "permission_scope": {
            "kind": "path_prefix",
            "allowed_prefixes": ["test/"]
        },
        "reversibility": "reversible",
        "determinism": "deterministic",
        "idempotency": {
            "mechanism": "none"
        },
        "confirmation_policy": {
            "standing_permitted": true,
            "per_call_required": false
        },
        "timeout_ms": 5000,
        "retry_policy": {
            "max_retries": 0,
            "backoff_ms": 500,
            "allowed_on": ["outcome_unknown"],
            "requires_idempotency_proof": false
        },
        "provider": {
            "identity": "tethers-stdio-fixture",
            "display_name": "Tethers M3 STDIO Fixture",
            "identity_source": "host_configuration",
            "description": "STDIO fixture for M3 conformance tests"
        },
        "binding": {
            "kind": "mcp",
            "server_name": "tethers-stdio-fixture",
            "tool_name": "fixture_ping",
            "adapter": null
        }
    });

    fs::write(
        source.join("plug.json"),
        serde_json::to_vec(&plug_json).unwrap(),
    )
    .unwrap();
    fs::write(
        source.join("manifests/fixture-ping.json"),
        serde_json::to_vec(&manifest_json).unwrap(),
    )
    .unwrap();

    (root, source)
}

fn pack(source: &std::path::Path, output: &std::path::Path) -> Value {
    let cmd = Command::new(host_binary())
        .args(["plug", "pack", "--source"])
        .arg(source)
        .args(["--output"])
        .arg(output)
        .output()
        .expect("plug pack process");
    assert_eq!(
        cmd.status.code(),
        Some(0),
        "plug pack must succeed, stderr: {}",
        String::from_utf8_lossy(&cmd.stderr)
    );
    serde_json::from_slice(&cmd.stdout).unwrap()
}

fn parse_output(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|_| {
            panic!(
                "failed to parse CLI output: {}",
                String::from_utf8_lossy(&output.stdout)
            )
        })
}

// ── Positive conform test ──

#[test]
fn p2b_public_conform_valid_passes() {
    let (root, source) = source_dir("valid");
    let output = root.join("fixture.tetherplug");
    pack(&source, &output);

    let cmd = Command::new(host_binary())
        .args([
            "plug",
            "conform",
            "--package",
            output.to_str().unwrap(),
            "--allow-non-isolated-supervised-execution",
        ])
        .output()
        .expect("plug conform process");

    let env = parse_output(&cmd);
    assert_eq!(env["schema"], "tethers.cli/1");
    assert_eq!(env["command"], "plug conform");
    assert_eq!(env["status"], "ok");
    assert_eq!(env["exit_code"], 0);

    let data = &env["data"];
    assert_eq!(
        data["package_id"],
        "example.conformance-fixture"
    );
    assert_eq!(data["package_version"], "0.1.0");
    assert_eq!(data["provider_id"], "tethers-stdio-fixture");
    assert_eq!(data["provider_version"], "0.1.0");
    assert!(!data["semantic_package_digest"]
        .as_str()
        .unwrap()
        .is_empty());
    assert!(data["semantic_package_digest"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));

    let conformance = &data["conformance"];
    assert_eq!(conformance["disposition"], "passed");
    assert!(conformance["suite_version"]
        .as_str()
        .unwrap()
        .starts_with("m3-generic-"));
    assert!(conformance["suite_digest"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));

    let cases = conformance["cases"].as_array().unwrap();
    assert!(cases
        .iter()
        .any(|c| c["case_id"] == "bounded_valid_fixture_call"));
    assert!(cases
        .iter()
        .any(|c| c["case_id"] == "invalid_fixture_call_refused"));
    assert!(cases
        .iter()
        .any(|c| c["case_id"] == "bounded_shutdown_process_cleanup"));

    let launch = &data["launch_profile"];
    assert_eq!(launch["isolated"], false);
    assert!(launch["limitation"]
        .as_str()
        .unwrap()
        .contains("not isolated"));
    assert!(data["retry_count"].as_u64().unwrap() == 0);
    assert!(data["raw_stderr_persisted"] == false);

    assert!(!data["conformance_evidence_id"]
        .as_str()
        .unwrap()
        .is_empty());
    assert!(data["conformance_evidence_digest"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));

    let stdout = String::from_utf8_lossy(&cmd.stdout);
    assert!(!stdout.contains("M3_SECRET_CANARY"));

    let _ = fs::remove_dir_all(&root);
}

// ── Approval required test ──

#[test]
fn p2b_conform_requires_explicit_approval() {
    let (root, source) = source_dir("approval");
    let output = root.join("fixture.tetherplug");
    pack(&source, &output);

    let cmd = Command::new(host_binary())
        .args(["plug", "conform", "--package", output.to_str().unwrap()])
        .output()
        .expect("plug conform without approval");

    let env = parse_output(&cmd);
    assert_eq!(env["schema"], "tethers.cli/1");
    assert_eq!(env["command"], "plug conform");
    assert_eq!(env["status"], "approval_required");
    assert_eq!(env["exit_code"], 5);
    assert_eq!(
        env["error"]["code"],
        "conformance_execution_approval_required"
    );
    assert!(
        env["error"]["message"]
            .as_str()
            .unwrap()
            .contains("not isolation")
    );

    let _ = fs::remove_dir_all(&root);
}

// ── Negative tests ──

#[test]
fn p2b_conform_rejects_relative_package_path() {
    let output = Command::new(host_binary())
        .args([
            "plug",
            "conform",
            "--package",
            "relative.tetherplug",
            "--allow-non-isolated-supervised-execution",
        ])
        .output()
        .unwrap();
    let env = parse_output(&output);
    assert_eq!(env["status"], "invalid_cli_usage");
    assert_eq!(env["exit_code"], 2);
}

#[test]
fn p2b_conform_missing_package_returns_unavailable() {
    let missing = std::env::temp_dir()
        .join(format!("tethers-p2b-missing-{}", uuid::Uuid::new_v4()))
        .join("nonexistent.tetherplug");

    let output = Command::new(host_binary())
        .args([
            "plug",
            "conform",
            "--package",
            missing.to_str().unwrap(),
            "--allow-non-isolated-supervised-execution",
        ])
        .output()
        .unwrap();
    let env = parse_output(&output);
    assert_eq!(env["status"], "unavailable");
    assert_eq!(env["exit_code"], 4);
}

#[test]
fn p2b_conform_malformed_package_returns_invalid_data() {
    let dir = std::env::temp_dir().join(format!("tethers-p2b-malformed-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    let bad = dir.join("bad.tetherplug");
    fs::write(&bad, b"not a valid zip archive -- just some bytes").unwrap();

    let output = Command::new(host_binary())
        .args([
            "plug",
            "conform",
            "--package",
            bad.to_str().unwrap(),
            "--allow-non-isolated-supervised-execution",
        ])
        .output()
        .unwrap();
    let env = parse_output(&output);
    assert_eq!(env["status"], "invalid_data");
    assert_eq!(env["exit_code"], 3);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn p2b_conform_stderr_not_in_public_output() {
    let (root, source) = source_dir("stderr-canary");
    let output = root.join("fixture.tetherplug");
    pack(&source, &output);

    let cmd = Command::new(host_binary())
        .args([
            "plug",
            "conform",
            "--package",
            output.to_str().unwrap(),
            "--allow-non-isolated-supervised-execution",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&cmd.stdout);
    assert!(!stdout.contains("M3_SECRET_CANARY"));
    assert!(!stdout.contains("fixture stderr"));

    let _ = fs::remove_dir_all(&root);
}

// ── Regression: P2A pack and J24A inspect still pass ──

#[test]
fn p2b_regression_plug_pack_help_contains_pack() {
    let host = host_binary();
    let output = Command::new(&host)
        .args(["plug", "pack", "--help"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("pack") || stdout.contains("--source"));

    let output = Command::new(&host)
        .args(["plug", "inspect", "--help"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("inspect") || stdout.contains("--package"));
}
