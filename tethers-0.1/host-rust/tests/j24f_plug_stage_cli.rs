//! J24F compiled-binary evidence for the public Plug stage adapter.

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tethers_reference_host::pdf_tools;
use uuid::Uuid;

fn host_binary() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_tethers-reference-host")
        .or_else(|| std::env::var_os("CARGO_BIN_EXE_tethers_reference_host"))
        .map(PathBuf::from)
        .or_else(|| {
            std::env::current_exe().ok().and_then(|path| {
                path.parent()?
                    .parent()
                    .map(|dir| dir.join("tethers-reference-host.exe"))
            })
        })
        .expect("compiled reference host binary")
}

fn temp_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("tethers-j24f-{name}-{}", Uuid::new_v4()))
}

fn write_package(root: &Path, name: &str, provider_bytes: &[u8]) -> PathBuf {
    let package = root.join(name);
    fs::write(
        &package,
        pdf_tools::build_reference_package(provider_bytes).expect("deterministic package"),
    )
    .expect("package bytes");
    package
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn snapshot(root: &Path) -> BTreeMap<String, String> {
    fn visit(root: &Path, path: &Path, output: &mut BTreeMap<String, String>) {
        let mut entries = fs::read_dir(path)
            .expect("snapshot directory")
            .map(|entry| entry.expect("snapshot entry").path())
            .collect::<Vec<_>>();
        entries.sort();
        for entry in entries {
            let relative = entry
                .strip_prefix(root)
                .expect("snapshot relative path")
                .to_string_lossy()
                .replace('\\', "/");
            let metadata = fs::symlink_metadata(&entry).expect("snapshot metadata");
            if metadata.is_dir() {
                output.insert(format!("{relative}/"), "<directory>".into());
                visit(root, &entry, output);
            } else {
                output.insert(relative, sha256(&fs::read(&entry).expect("snapshot file")));
            }
        }
    }

    let mut output = BTreeMap::new();
    if root.is_dir() {
        visit(root, root, &mut output);
    }
    output
}

fn run(args: Vec<OsString>) -> (i32, Value) {
    let output = Command::new(host_binary())
        .args(args)
        .output()
        .expect("reference host process");
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 envelope");
    assert_eq!(
        stdout.lines().count(),
        1,
        "expected one JSON line: {stdout}"
    );
    let envelope: Value = serde_json::from_str(stdout.trim()).expect("JSON envelope");
    let process_code = output.status.code().expect("process exit code");
    assert_eq!(process_code, envelope["exit_code"].as_i64().unwrap() as i32);
    (process_code, envelope)
}

fn stage(host: &Path, package: &Path, equal_signs: bool) -> (i32, Value) {
    let host_arg = if equal_signs {
        OsString::from(format!("--host-data-root={}", host.display()))
    } else {
        OsString::from("--host-data-root")
    };
    let package_arg = if equal_signs {
        OsString::from(format!("--package={}", package.display()))
    } else {
        OsString::from("--package")
    };
    let mut args = vec![OsString::from("plug"), OsString::from("stage"), host_arg];
    if !equal_signs {
        args.push(host.as_os_str().to_owned());
    }
    args.push(package_arg);
    if !equal_signs {
        args.push(package.as_os_str().to_owned());
    }
    run(args)
}

fn assert_keys(value: &Value, expected: &[&str]) {
    let actual = value
        .as_object()
        .expect("JSON object")
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected.iter().copied().collect());
}

fn assert_no_lifecycle_paths(root: &Path) {
    for name in [
        "install",
        "installed-records",
        "enablements",
        "trust",
        "conformance",
        "approvals",
    ] {
        assert!(!root.join(name).exists(), "forbidden path exists: {name}");
    }
}

#[test]
fn first_stage_emits_allowlisted_created_candidate_and_only_candidate_state() {
    let root = temp_dir("created");
    let host = root.join("host-data");
    fs::create_dir_all(&host).unwrap();
    let package = write_package(&root, "pdf-tools.tetherplug", b"non-executable-provider");

    let (code, envelope) = stage(&host, &package, false);
    assert_eq!(code, 0);
    assert_eq!(envelope["schema"], "tethers.cli/1");
    assert_eq!(envelope["command"], "plug stage");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["exit_code"], 0);
    assert!(envelope["error"].is_null());
    assert_keys(&envelope["data"], &["candidate"]);

    let candidate = &envelope["data"]["candidate"];
    assert_keys(
        candidate,
        &[
            "candidate_id",
            "disposition",
            "state",
            "package_id",
            "package_version",
            "semantic_package_digest",
            "raw_archive_digest",
            "provider_id",
            "provider_version",
            "platform",
            "capabilities",
            "created_unix_ms",
        ],
    );
    assert_eq!(candidate["disposition"], "created");
    assert_eq!(candidate["state"], "quarantined_installation_candidate");
    assert_eq!(candidate["package_id"], "tethers.pdf-tools");
    assert_eq!(candidate["package_version"], "1.0.0");
    assert_eq!(candidate["provider_id"], "tethers-pdf-provider");
    assert_eq!(candidate["provider_version"], "1.0.0");
    assert_keys(&candidate["platform"], &["os", "architecture"]);
    assert_eq!(candidate["platform"]["os"], "windows");
    assert_eq!(candidate["platform"]["architecture"], "x86_64");
    let capabilities = candidate["capabilities"].as_array().unwrap();
    assert_eq!(capabilities.len(), 1);
    assert_keys(
        &capabilities[0],
        &["name", "version", "manifest_digest", "operation"],
    );
    assert_eq!(capabilities[0]["name"], "pdf.inspect");
    assert_eq!(capabilities[0]["version"], 1);
    assert_eq!(capabilities[0]["operation"], "pdf_inspect");

    for forbidden in [
        "quarantine_relative_path",
        "source_size_bytes",
        "plug_json",
        "payloads",
        "signature_files",
        "signatures_present",
        "launch_path",
        "launch_arguments",
        "provider_working_directory",
        "capability_operation_namespace",
        "inspection_report_format_version",
        "inspection_evidence_digest",
        "record_digest",
    ] {
        assert!(
            !envelope.to_string().contains(forbidden),
            "exposed {forbidden}"
        );
    }

    let candidate_files = fs::read_dir(host.join("candidates")).unwrap().count();
    let quarantine_dirs = fs::read_dir(host.join("quarantine"))
        .unwrap()
        .filter(|entry| entry.as_ref().unwrap().path().is_dir())
        .count();
    assert_eq!(candidate_files, 1);
    assert_eq!(quarantine_dirs, 1);
    assert_no_lifecycle_paths(&host);
    assert!(snapshot(&host)
        .keys()
        .any(|path| path.starts_with("quarantine/")));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn exact_replay_emits_existing_same_id_and_changes_no_host_bytes() {
    let root = temp_dir("replay");
    let host = root.join("host-data");
    fs::create_dir_all(&host).unwrap();
    let package = write_package(&root, "pdf-tools.tetherplug", b"non-executable-replay");

    let (_, first) = stage(&host, &package, false);
    let first_id = first["data"]["candidate"]["candidate_id"].clone();
    let before = snapshot(&host);
    let (code, second) = stage(&host, &package, true);
    assert_eq!(code, 0);
    assert_eq!(second["data"]["candidate"]["disposition"], "existing");
    assert_eq!(second["data"]["candidate"]["candidate_id"], first_id);
    assert_eq!(before, snapshot(&host));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn service_failures_preserve_codes_and_do_not_create_or_change_candidate_state() {
    let root = temp_dir("failures");
    let host = root.join("host-data");
    fs::create_dir_all(&host).unwrap();

    let malformed = root.join("malformed.tetherplug");
    fs::write(&malformed, b"not a ZIP archive").unwrap();
    let before = snapshot(&host);
    let (code, malformed_result) = stage(&host, &malformed, false);
    assert_eq!(code, 3);
    assert_eq!(malformed_result["status"], "invalid_data");
    assert_eq!(malformed_result["error"]["code"], "invalid_archive");
    assert_eq!(before, snapshot(&host));

    let missing = root.join("missing.tetherplug");
    let (code, missing_result) = stage(&host, &missing, false);
    assert_eq!(code, 4);
    assert_eq!(missing_result["status"], "unavailable");
    assert_eq!(missing_result["error"]["code"], "archive_read");
    assert_eq!(before, snapshot(&host));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn semantic_conflict_and_corrupt_record_change_no_candidate_bytes() {
    let root = temp_dir("unchanged-errors");
    let host = root.join("host-data");
    fs::create_dir_all(&host).unwrap();
    let package_a = write_package(&root, "a.tetherplug", b"provider-bytes-A");
    let package_b = write_package(&root, "b.tetherplug", b"provider-bytes-B");

    stage(&host, &package_a, false);
    let before_conflict = snapshot(&host);
    let (code, conflict) = stage(&host, &package_b, false);
    assert_eq!(code, 3);
    assert_eq!(conflict["status"], "invalid_data");
    assert_eq!(conflict["error"]["code"], "semantic_conflict");
    assert_eq!(before_conflict, snapshot(&host));

    let record = fs::read_dir(host.join("candidates"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    fs::write(&record, b"corrupted candidate record").unwrap();
    let before_corrupt = snapshot(&host);
    let (code, corrupt) = stage(&host, &package_a, false);
    assert_eq!(code, 3);
    assert_eq!(corrupt["status"], "invalid_data");
    assert_eq!(corrupt["error"]["code"], "record_invalid");
    assert_eq!(before_corrupt, snapshot(&host));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn relative_arguments_and_malformed_shapes_are_cli_usage_errors() {
    let root = temp_dir("usage");
    fs::create_dir_all(&root).unwrap();
    let package = write_package(&root, "package.tetherplug", b"provider-bytes");
    let absolute_host = root.join("host-data");
    fs::create_dir_all(&absolute_host).unwrap();

    let (code, host_error) = run(vec![
        "plug".into(),
        "stage".into(),
        "--host-data-root".into(),
        "relative-host".into(),
        "--package".into(),
        package.as_os_str().to_owned(),
    ]);
    assert_eq!(code, 2);
    assert_eq!(host_error["error"]["code"], "invalid_cli_usage");
    assert_eq!(host_error["error"]["field"], "/host-data-root");
    assert!(snapshot(&absolute_host).is_empty());

    let (code, package_error) = run(vec![
        "plug".into(),
        "stage".into(),
        "--host-data-root".into(),
        absolute_host.as_os_str().to_owned(),
        "--package".into(),
        "relative-package.tetherplug".into(),
    ]);
    assert_eq!(code, 2);
    assert_eq!(package_error["error"]["code"], "invalid_cli_usage");
    assert_eq!(package_error["error"]["field"], "/package");
    assert!(snapshot(&absolute_host).is_empty());

    for args in [
        vec!["plug", "stage"],
        vec!["plug", "stage", "--host-data-root", "a"],
        vec!["plug", "stage", "--package", "a"],
        vec![
            "plug",
            "stage",
            "--host-data-root",
            "a",
            "--host-data-root",
            "b",
            "--package",
            "c",
        ],
        vec![
            "plug",
            "stage",
            "--host-data-root",
            "a",
            "--package",
            "b",
            "extra",
        ],
        vec![
            "plug",
            "stage",
            "--host-data-root",
            "a",
            "--package",
            "b",
            "--unknown",
        ],
    ] {
        let (code, _) = run(args.into_iter().map(OsString::from).collect());
        assert_eq!(code, 2);
    }

    fs::remove_dir_all(root).unwrap();
}

#[cfg(windows)]
#[test]
fn junction_backed_package_is_rejected_without_candidate_state() {
    let root = temp_dir("junction");
    let host = root.join("host-data");
    let target = root.join("target");
    let junction = root.join("junction");
    fs::create_dir_all(&host).unwrap();
    fs::create_dir_all(&target).unwrap();
    let target_package = write_package(&target, "package.tetherplug", b"junction-provider");
    let status = Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(&junction)
        .arg(&target)
        .status()
        .unwrap();
    assert!(status.success());
    assert!(junction.join(target_package.file_name().unwrap()).is_file());

    let before = snapshot(&host);
    let (code, envelope) = stage(
        &host,
        &junction.join(target_package.file_name().unwrap()),
        false,
    );
    assert_eq!(code, 3);
    assert_eq!(envelope["status"], "invalid_data");
    assert_eq!(envelope["error"]["code"], "unsafe_destination");
    assert_eq!(before, snapshot(&host));

    fs::remove_dir(&junction).unwrap();
    fs::remove_dir_all(root).unwrap();
}
