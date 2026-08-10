//! P2A primary public proof: plug pack → plug inspect end-to-end via real CLI.

use serde_json::{json, Value};
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

fn source_dir() -> (std::path::PathBuf, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!("tethers-p2a-int-{}", uuid::Uuid::new_v4()));
    let source = root.join("example-text-tools");
    fs::create_dir_all(source.join("provider")).unwrap();
    fs::create_dir_all(source.join("manifests")).unwrap();

    let plug_json = json!({
        "package_format_version": "1",
        "package_id": "example.text-tools",
        "package_version": "0.1.0",
        "display_name": "Example Text Tools",
        "description": "Example text processing Plug for integration testing",
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
            "provider_id": "example.text-provider",
            "provider_version": "0.1.0",
            "launch": {
                "path": "provider/example-text-provider.exe",
                "arguments": ["--serve"]
            },
            "working_directory": "provider",
            "capability_operation_namespace": "text"
        },
        "capabilities": [{
            "capability_name": "text.inspect",
            "capability_version": 1,
            "manifest_path": "manifests/text-inspect.json",
            "provider_operation_name": "inspect"
        }],
        "payloads": [
            {"path": "manifests/text-inspect.json", "role": "capability_manifest"},
            {"path": "provider/example-text-provider.exe", "role": "provider_executable"}
        ]
    });

    let manifest_json = json!({
        "manifest_format_version": "1.0",
        "capability_name": "text.inspect",
        "capability_version": 1,
        "title": "Inspect Text",
        "description": "Inspect text content deterministically",
        "input_schema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}
            },
            "required": ["path"],
            "additionalProperties": false
        },
        "output_schema": {
            "type": "object",
            "properties": {
                "size_bytes": {"type": "integer"},
                "encoding": {"type": "string"}
            },
            "required": ["size_bytes", "encoding"]
        },
        "effects": ["filesystem.read"],
        "permission_scope": {
            "kind": "path_prefix",
            "allowed_prefixes": ["projects/"]
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
            "identity": "example-text-provider",
            "display_name": "Example Text Provider",
            "identity_source": "host_configuration",
            "description": "Fixture provider for integration testing"
        },
        "binding": {
            "kind": "mcp",
            "server_name": "example-text-provider",
            "tool_name": "inspect",
            "adapter": null
        }
    });

    fs::write(
        source.join("plug.json"),
        serde_json::to_vec(&plug_json).unwrap(),
    )
    .unwrap();
    fs::write(
        source.join("manifests/text-inspect.json"),
        serde_json::to_vec(&manifest_json).unwrap(),
    )
    .unwrap();
    fs::write(
        source.join("provider/example-text-provider.exe"),
        b"deterministic fixture -- example text provider",
    )
    .unwrap();

    (root, source)
}

#[test]
fn p2a_public_pack_and_inspect_roundtrip() {
    let (root, source) = source_dir();
    let output = root.join("example-text-tools.tetherplug");

    let plug_before = fs::read(source.join("plug.json")).unwrap();
    let manifest_before = fs::read(source.join("manifests/text-inspect.json")).unwrap();

    let pack_cmd = Command::new(host_binary())
        .args(["plug", "pack", "--source"])
        .arg(&source)
        .args(["--output"])
        .arg(&output)
        .output()
        .expect("plug pack process");

    assert_eq!(
        pack_cmd.status.code(),
        Some(0),
        "plug pack exit code 0 expected, stderr: {}",
        String::from_utf8_lossy(&pack_cmd.stderr)
    );
    let pack_stdout = String::from_utf8(pack_cmd.stdout).unwrap();
    let pack_env: Value = serde_json::from_str(pack_stdout.trim()).unwrap();

    assert_eq!(pack_env["schema"], "tethers.cli/1");
    assert_eq!(pack_env["command"], "plug pack");
    assert_eq!(pack_env["status"], "ok");
    assert_eq!(pack_env["exit_code"], 0);
    assert!(pack_env["error"].is_null());
    let data = &pack_env["data"];
    assert_eq!(data["package_id"], "example.text-tools");
    assert_eq!(data["package_version"], "0.1.0");
    assert_eq!(data["provider_id"], "example.text-provider");
    assert_eq!(data["capability_count"], 1);
    assert!(!data["semantic_package_digest"].as_str().unwrap().is_empty());
    assert!(data["semantic_package_digest"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));
    assert!(!data["raw_archive_digest"].as_str().unwrap().is_empty());
    assert!(data["raw_archive_size"].as_u64().unwrap() > 0);

    assert!(
        output.exists(),
        "output .tetherplug must exist after success"
    );

    let inspect_cmd = Command::new(host_binary())
        .args(["plug", "inspect", "--package"])
        .arg(&output)
        .output()
        .expect("plug inspect process");

    assert_eq!(
        inspect_cmd.status.code(),
        Some(0),
        "plug inspect exit code 0 expected, stderr: {}",
        String::from_utf8_lossy(&inspect_cmd.stderr)
    );
    let inspect_stdout = String::from_utf8(inspect_cmd.stdout).unwrap();
    let inspect_env: Value = serde_json::from_str(inspect_stdout.trim()).unwrap();

    assert_eq!(inspect_env["schema"], "tethers.cli/1");
    assert_eq!(inspect_env["command"], "plug inspect");
    assert_eq!(inspect_env["status"], "ok");
    assert_eq!(inspect_env["exit_code"], 0);
    let inspection = &inspect_env["data"]["inspection"];
    assert_eq!(inspection["package"]["package_id"], "example.text-tools");
    assert_eq!(inspection["package"]["package_version"], "0.1.0");
    assert_eq!(inspection["provider_id"], "example.text-provider");
    assert_eq!(inspection["capabilities"][0]["name"], "text.inspect");
    assert_eq!(inspection["capabilities"][0]["version"], 1);

    let manifest_digest = inspection["capabilities"][0]["manifest_digest"]
        .as_str()
        .unwrap();
    assert!(manifest_digest.starts_with("sha256:"));
    assert_eq!(manifest_digest.len(), 71);

    let payload_index = &inspection["payloads"];
    assert!(payload_index.as_array().unwrap().len() >= 2);
    let manifest_payload = payload_index
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["role"] == "capability_manifest")
        .unwrap();
    assert!(manifest_payload["sha256"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));
    assert!(manifest_payload["size_bytes"].as_u64().unwrap() > 0);

    let packed_zip = fs::read(&output).unwrap();
    use std::io::Read;
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&packed_zip)).unwrap();
    let packed_plug_json = {
        let mut entry = archive.by_name("plug.json").unwrap();
        let mut buf = String::new();
        entry.read_to_string(&mut buf).unwrap();
        buf
    };
    let packed_plug: Value = serde_json::from_str(&packed_plug_json).unwrap();
    assert!(
        packed_plug.get("payload_index").is_some(),
        "packed plug.json must have payload_index"
    );
    assert!(
        packed_plug.get("payloads").is_none(),
        "packed plug.json must NOT have author-side payloads"
    );

    let packed_manifest = {
        let mut entry = archive.by_name("manifests/text-inspect.json").unwrap();
        let mut buf = String::new();
        entry.read_to_string(&mut buf).unwrap();
        buf
    };
    let packed_manifest_val: Value = serde_json::from_str(&packed_manifest).unwrap();
    let packed_manifest_digest = packed_manifest_val["digest"].as_str().unwrap();
    assert_eq!(packed_manifest_digest, manifest_digest);
    assert_eq!(packed_manifest_digest.len(), 71);
    assert!(packed_manifest_digest.starts_with("sha256:"));

    let plug_after = fs::read(source.join("plug.json")).unwrap();
    let manifest_after = fs::read(source.join("manifests/text-inspect.json")).unwrap();
    assert_eq!(
        plug_before, plug_after,
        "source plug.json must be unchanged"
    );
    assert_eq!(
        manifest_before, manifest_after,
        "source manifest must be unchanged"
    );

    assert_eq!(
        pack_env["data"]["semantic_package_digest"],
        inspection["package"]["semantic_digest"]
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn p2a_pack_is_byte_identical_across_two_runs() {
    let (root, source) = source_dir();
    let first = root.join("first.tetherplug");
    let second = root.join("second.tetherplug");

    let pack1 = Command::new(host_binary())
        .args(["plug", "pack", "--source"])
        .arg(&source)
        .args(["--output"])
        .arg(&first)
        .output()
        .unwrap();
    assert_eq!(pack1.status.code(), Some(0));

    let pack2 = Command::new(host_binary())
        .args(["plug", "pack", "--source"])
        .arg(&source)
        .args(["--output"])
        .arg(&second)
        .output()
        .unwrap();
    assert_eq!(pack2.status.code(), Some(0));

    let a = fs::read(&first).unwrap();
    let b = fs::read(&second).unwrap();
    assert_eq!(a, b, "two packs must be byte-for-byte identical");

    let env1: Value = serde_json::from_slice(&pack1.stdout).unwrap();
    let env2: Value = serde_json::from_slice(&pack2.stdout).unwrap();
    assert_eq!(
        env1["data"]["raw_archive_digest"],
        env2["data"]["raw_archive_digest"]
    );
    assert_eq!(
        env1["data"]["semantic_package_digest"],
        env2["data"]["semantic_package_digest"]
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn p2a_refuses_relative_source_cli() {
    let output = Command::new(host_binary())
        .args([
            "plug",
            "pack",
            "--source",
            "relative",
            "--output",
            "C:\\out.tetherplug",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let env: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(env["status"], "invalid_cli_usage");
}

#[test]
fn p2a_refuses_relative_output_cli() {
    let dir = std::env::temp_dir().join(format!("tethers-p2a-rel-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(host_binary())
        .args([
            "plug",
            "pack",
            "--source",
            dir.to_str().unwrap(),
            "--output",
            "relative.tetherplug",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn p2a_refuses_wrong_extension_cli() {
    let dir = std::env::temp_dir().join(format!("tethers-p2a-ext-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(host_binary())
        .args([
            "plug",
            "pack",
            "--source",
            dir.to_str().unwrap(),
            "--output",
            dir.join("out.zip").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn p2a_refuses_existing_output_cli() {
    let (root, source) = source_dir();
    let output = root.join("exists.tetherplug");
    fs::write(&output, b"pre-existing").unwrap();

    let cmd = Command::new(host_binary())
        .args(["plug", "pack", "--source"])
        .arg(&source)
        .args(["--output"])
        .arg(&output)
        .output()
        .unwrap();
    assert_eq!(cmd.status.code(), Some(3));
    let env: Value = serde_json::from_slice(&cmd.stdout).unwrap();
    assert_eq!(env["error"]["code"], "output_exists");
    assert_eq!(fs::read(&output).unwrap(), b"pre-existing");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn p2a_no_output_left_on_failure_cli() {
    let (root, source) = source_dir();
    let output = root.join("no-final.tetherplug");
    let _ = fs::remove_file(source.join("provider/example-text-provider.exe"));

    let cmd = Command::new(host_binary())
        .args(["plug", "pack", "--source"])
        .arg(&source)
        .args(["--output"])
        .arg(&output)
        .output()
        .unwrap();
    assert_ne!(cmd.status.code(), Some(0));
    assert!(!output.exists(), "no .tetherplug must remain after failure");

    let _ = fs::remove_dir_all(&root);
}
