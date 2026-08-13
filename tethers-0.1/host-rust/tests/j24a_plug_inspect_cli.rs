//! J24A public, read-only Plug inspection CLI evidence.

use std::fs;
use std::process::Command;

use serde_json::Value;
use tethers_reference_host::test_fixture_package;

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

fn package_path() -> (std::path::PathBuf, Vec<u8>) {
    let provider = fs::read(env!("CARGO_BIN_EXE_m3_fixture_provider"))
        .expect("compiled fixture provider binary");
    let bytes =
        test_fixture_package::build_fixture_package(&provider).expect("deterministic package");
    let root = std::env::temp_dir().join(format!("tethers-j24a-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&root).expect("test directory");
    let package = root.join("fixture.tetherplug");
    fs::write(&package, &bytes).expect("package bytes");
    (package, bytes)
}

#[test]
fn public_inspect_emits_complete_read_only_success_envelope() {
    let (package, original_bytes) = package_path();
    let before_entries = fs::read_dir(package.parent().unwrap())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<Vec<_>>();
    let output = Command::new(host_binary())
        .args(["plug", "inspect", "--package"])
        .arg(&package)
        .output()
        .expect("reference host process");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 1);
    let envelope: Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(envelope["schema"], "tethers.cli/1");
    assert_eq!(envelope["command"], "plug inspect");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["exit_code"], 0);
    assert!(envelope["error"].is_null());
    let inspection = &envelope["data"]["inspection"];
    assert_eq!(inspection["package"]["package_id"], "tethers.fixture");
    assert_eq!(inspection["package"]["package_version"], "1.0.0");
    assert_eq!(inspection["provider_id"], "tethers-stdio-fixture");
    assert_eq!(inspection["capabilities"][0]["name"], "fixture.ping");
    assert_eq!(inspection["capabilities"][0]["version"], 1);
    let evidence_digest = inspection["inspection_evidence_digest"].as_str().unwrap();
    assert_eq!(evidence_digest.len(), 71);
    assert!(evidence_digest.starts_with("sha256:"));
    assert!(evidence_digest[7..]
        .bytes()
        .all(|byte| byte.is_ascii_hexdigit()));
    assert!(inspection.get("archive_path").is_none());
    assert_eq!(fs::read(&package).unwrap(), original_bytes);
    let after_entries = fs::read_dir(package.parent().unwrap())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<Vec<_>>();
    assert_eq!(before_entries, after_entries);
    fs::remove_dir_all(package.parent().unwrap()).unwrap();
}

#[test]
fn malformed_and_package_failures_preserve_cli_mapping() {
    let invalid = Command::new(host_binary())
        .args(["plug", "inspect", "--package", "not-a-package.zip"])
        .output()
        .unwrap();
    assert_eq!(invalid.status.code(), Some(3));
    let invalid_json: Value = serde_json::from_slice(&invalid.stdout).unwrap();
    assert_eq!(invalid_json["status"], "invalid_data");
    assert_eq!(invalid_json["error"]["code"], "invalid_archive");

    let missing = Command::new(host_binary())
        .args(["plug", "inspect", "--package", "missing.tetherplug"])
        .output()
        .unwrap();
    assert_eq!(missing.status.code(), Some(4));
    let missing_json: Value = serde_json::from_slice(&missing.stdout).unwrap();
    assert_eq!(missing_json["status"], "unavailable");
    assert_eq!(missing_json["error"]["code"], "archive_read");
}

#[test]
fn malformed_command_shapes_are_rejected() {
    for args in [
        vec!["plug"],
        vec!["plug", "inspect"],
        vec!["plug", "inspect", "--package", "a", "--package", "b"],
        vec!["plug", "inspect", "--unknown", "a"],
        vec!["plug", "inspect", "--package", "a", "extra"],
    ] {
        let output = Command::new(host_binary()).args(args).output().unwrap();
        assert_eq!(output.status.code(), Some(2));
    }
}
