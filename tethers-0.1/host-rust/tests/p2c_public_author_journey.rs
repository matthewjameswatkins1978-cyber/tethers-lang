//! P2C end-to-end public author journey: pack → inspect → conform via real CLI.

use serde_json::{json, Value};
use std::fs;
use std::path::Path;
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

fn parse_env(output: &std::process::Output) -> Value {
    let stdout = String::from_utf8(output.stdout.clone())
        .unwrap()
        .trim()
        .to_owned();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "expected exactly one JSON envelope line");
    serde_json::from_str(lines[0]).unwrap()
}

fn build_source_tree(author_root: &Path, marker_path: &Path) -> std::path::PathBuf {
    let source = author_root.join("public-author-proof");
    fs::create_dir_all(source.join("provider")).unwrap();
    fs::create_dir_all(source.join("manifests")).unwrap();

    let fixture_exe = fixture_provider_binary();
    fs::copy(
        &fixture_exe,
        source.join("provider/tethers-stdio-fixture.exe"),
    )
    .unwrap();

    let plug_json = json!({
        "package_format_version": "1",
        "package_id": "example.public-author-proof",
        "package_version": "0.1.0",
        "display_name": "Public Author Proof Plug",
        "description": "P2C end-to-end public author journey proof plug",
        "publisher": "tethers-p2c",
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
                "arguments": [
                    "--mode", "valid",
                    "--provider-marker", marker_path.to_str().unwrap()
                ]
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

    let manifest_json = json!({
        "manifest_format_version": "1.0",
        "capability_name": "fixture.ping",
        "capability_version": 1,
        "title": "Fixture Ping",
        "description": "P2C fixture ping capability for conformance testing",
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
            "display_name": "Tethers STDIO Fixture",
            "identity_source": "host_configuration",
            "description": "STDIO fixture provider for P2C author proof"
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

    source
}

#[test]
fn p2c_public_author_journey() {
    let test_root = std::env::temp_dir().join(format!("tethers-p2c-{}", uuid::Uuid::new_v4()));
    let author_root = test_root.join("author");
    let conform_temp = test_root.join("conform-temp");
    fs::create_dir_all(&author_root).unwrap();
    fs::create_dir_all(&conform_temp).unwrap();

    let marker = test_root.join("provider-marker.txt");

    let source = build_source_tree(&author_root, &marker);
    let package = test_root.join("public-author-proof.tetherplug");

    // ── Capture source bytes before journey ──
    let plug_before = fs::read(source.join("plug.json")).unwrap();
    let manifest_before = fs::read(source.join("manifests/fixture-ping.json")).unwrap();
    let provider_before = fs::read(source.join("provider/tethers-stdio-fixture.exe")).unwrap();

    // ── STEP 1: Real public pack ──
    assert!(
        !marker.exists(),
        "provider marker must not exist before any conform"
    );

    let pack_cmd = Command::new(host_binary())
        .args(["plug", "pack", "--source"])
        .arg(&source)
        .args(["--output"])
        .arg(&package)
        .output()
        .expect("plug pack");

    assert_eq!(
        pack_cmd.status.code(),
        Some(0),
        "pack exit 0, stderr: {}",
        String::from_utf8_lossy(&pack_cmd.stderr)
    );
    let pack_env = parse_env(&pack_cmd);
    assert_eq!(pack_env["schema"], "tethers.cli/1");
    assert_eq!(pack_env["command"], "plug pack");
    assert_eq!(pack_env["status"], "ok");
    assert_eq!(pack_env["exit_code"], 0);
    assert!(pack_env["error"].is_null());

    let pack_data = &pack_env["data"];
    assert_eq!(pack_data["package_id"], "example.public-author-proof");
    assert_eq!(pack_data["package_version"], "0.1.0");
    assert_eq!(pack_data["provider_id"], "tethers-stdio-fixture");
    assert_eq!(pack_data["capability_count"], 1);

    let pack_semantic = pack_data["semantic_package_digest"].as_str().unwrap();
    assert!(!pack_semantic.is_empty());
    assert!(pack_semantic.starts_with("sha256:"));
    assert_eq!(pack_semantic.len(), 71);

    let raw_archive_digest = pack_data["raw_archive_digest"].as_str().unwrap();
    assert!(!raw_archive_digest.is_empty());
    assert!(raw_archive_digest.starts_with("sha256:"));

    let raw_archive_size = pack_data["raw_archive_size"].as_u64().unwrap();
    assert!(raw_archive_size > 0);

    assert!(
        package.exists(),
        "package .tetherplug must exist after pack"
    );

    let package_after_pack = fs::read(&package).unwrap();

    // ── STEP 2: Real public inspect ──
    let inspect_cmd = Command::new(host_binary())
        .args(["plug", "inspect", "--package"])
        .arg(&package)
        .output()
        .expect("plug inspect");

    assert_eq!(
        inspect_cmd.status.code(),
        Some(0),
        "inspect exit 0, stderr: {}",
        String::from_utf8_lossy(&inspect_cmd.stderr)
    );
    let inspect_env = parse_env(&inspect_cmd);
    assert_eq!(inspect_env["schema"], "tethers.cli/1");
    assert_eq!(inspect_env["command"], "plug inspect");
    assert_eq!(inspect_env["status"], "ok");
    assert_eq!(inspect_env["exit_code"], 0);

    let inspection = &inspect_env["data"]["inspection"];
    assert_eq!(
        inspection["package"]["package_id"],
        "example.public-author-proof"
    );
    assert_eq!(inspection["package"]["package_version"], "0.1.0");
    assert_eq!(inspection["provider_id"], "tethers-stdio-fixture");
    assert_eq!(inspection["capabilities"][0]["name"], "fixture.ping");
    assert_eq!(inspection["capabilities"][0]["version"], 1);

    let manifest_digest = inspection["capabilities"][0]["manifest_digest"]
        .as_str()
        .unwrap();
    assert!(
        manifest_digest.starts_with("sha256:"),
        "manifest digest must exist after inspect: {manifest_digest}"
    );
    assert_eq!(manifest_digest.len(), 71);

    let evidence_digest = inspection["inspection_evidence_digest"].as_str().unwrap();
    assert!(evidence_digest.starts_with("sha256:"));
    assert_eq!(evidence_digest.len(), 71);

    let inspect_semantic = inspection["package"]["semantic_digest"].as_str().unwrap();
    assert_eq!(
        pack_semantic, inspect_semantic,
        "digest continuity: pack semantic_digest == inspect semantic_digest"
    );

    let package_after_inspect = fs::read(&package).unwrap();
    assert_eq!(
        package_after_pack, package_after_inspect,
        "inspect must not mutate package bytes"
    );

    // ── STEP 3: Execution safety gate (no approval) ──
    assert!(
        !marker.exists(),
        "provider marker must not exist before conform without approval"
    );

    let no_approval_cmd = Command::new(host_binary())
        .args(["plug", "conform", "--package"])
        .arg(package.to_str().unwrap())
        .env("TEMP", &conform_temp)
        .env("TMP", &conform_temp)
        .output()
        .expect("plug conform without approval");

    assert_eq!(
        no_approval_cmd.status.code(),
        Some(5),
        "conform without approval must exit 5"
    );
    let no_approval_env = parse_env(&no_approval_cmd);
    assert_eq!(no_approval_env["schema"], "tethers.cli/1");
    assert_eq!(no_approval_env["command"], "plug conform");
    assert_eq!(no_approval_env["status"], "approval_required");
    assert_eq!(no_approval_env["exit_code"], 5);
    assert_eq!(
        no_approval_env["error"]["code"],
        "conformance_execution_approval_required"
    );

    assert!(
        !marker.exists(),
        "provider must not be executed without approval"
    );

    // ── STEP 4: Real approved public conform ──
    let approved_cmd = Command::new(host_binary())
        .args([
            "plug",
            "conform",
            "--package",
            package.to_str().unwrap(),
            "--allow-non-isolated-supervised-execution",
        ])
        .env("TEMP", &conform_temp)
        .env("TMP", &conform_temp)
        .output()
        .expect("plug conform with approval");

    assert_eq!(
        approved_cmd.status.code(),
        Some(0),
        "approved conform exit 0, stderr: {}",
        String::from_utf8_lossy(&approved_cmd.stderr)
    );
    let approved_env = parse_env(&approved_cmd);
    assert_eq!(approved_env["schema"], "tethers.cli/1");
    assert_eq!(approved_env["command"], "plug conform");
    assert_eq!(approved_env["status"], "ok");
    assert_eq!(approved_env["exit_code"], 0);

    let conform_data = &approved_env["data"];
    assert_eq!(conform_data["package_id"], "example.public-author-proof");
    assert_eq!(conform_data["package_version"], "0.1.0");
    assert_eq!(conform_data["provider_id"], "tethers-stdio-fixture");
    assert_eq!(conform_data["provider_version"], "0.1.0");

    let conform_semantic = conform_data["semantic_package_digest"].as_str().unwrap();
    assert_eq!(
        pack_semantic, conform_semantic,
        "digest continuity: pack == inspect == conform semantic_package_digest"
    );

    let disposition = conform_data["conformance"]["disposition"].as_str().unwrap();
    assert_eq!(disposition, "passed");

    let suite_version = conform_data["conformance"]["suite_version"]
        .as_str()
        .unwrap();
    assert!(!suite_version.is_empty());

    let suite_digest = conform_data["conformance"]["suite_digest"]
        .as_str()
        .unwrap();
    assert!(!suite_digest.is_empty());
    assert!(suite_digest.starts_with("sha256:"));

    let launch = &conform_data["launch_profile"];
    assert_eq!(launch["isolated"], false);
    let limitation = launch["limitation"].as_str().unwrap();
    assert!(
        limitation.contains("not isolated"),
        "limitation must state non-isolation: {limitation}"
    );

    assert_eq!(conform_data["retry_count"].as_u64().unwrap(), 0);
    assert_eq!(conform_data["raw_stderr_persisted"], false);

    let evidence_id = conform_data["conformance_evidence_id"].as_str().unwrap();
    assert!(!evidence_id.is_empty());

    let evidence_digest_field = conform_data["conformance_evidence_digest"]
        .as_str()
        .unwrap();
    assert!(!evidence_digest_field.is_empty());
    assert!(evidence_digest_field.starts_with("sha256:"));

    // ── Provider execution proof ──
    assert!(
        marker.exists(),
        "provider marker MUST exist after approved conform"
    );

    // ── Cleanup proof ──
    let entries: Vec<_> = fs::read_dir(&conform_temp)
        .map(|dir| dir.filter_map(|e| e.ok()).collect::<Vec<_>>())
        .unwrap_or_default();
    let remaining_conform_dirs: Vec<_> = entries
        .iter()
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("tethers-p2b-conform-")
        })
        .collect();
    assert!(
        remaining_conform_dirs.is_empty(),
        "no tethers-p2b-conform-* directories must remain after conform"
    );

    // ── Public output hygiene ──
    let approved_stdout = String::from_utf8_lossy(&approved_cmd.stdout);
    assert!(!approved_stdout.contains("M3_SECRET_CANARY"));
    assert!(!approved_stdout.contains("quarantine"));
    assert!(!approved_stdout.contains("installation-trust"));
    assert!(!approved_stdout.contains("conformance-scratch"));

    // ── Immutability proof ──
    let plug_after = fs::read(source.join("plug.json")).unwrap();
    let manifest_after = fs::read(source.join("manifests/fixture-ping.json")).unwrap();
    let provider_after = fs::read(source.join("provider/tethers-stdio-fixture.exe")).unwrap();

    assert_eq!(
        plug_before, plug_after,
        "source plug.json must be immutable"
    );
    assert_eq!(
        manifest_before, manifest_after,
        "source manifest must be immutable"
    );
    assert_eq!(
        provider_before, provider_after,
        "source provider executable must be immutable"
    );

    let package_after_journey = fs::read(&package).unwrap();
    assert_eq!(
        package_after_pack, package_after_journey,
        "package bytes must be unchanged since immediately after pack"
    );

    let _ = fs::remove_dir_all(&test_root);
}
