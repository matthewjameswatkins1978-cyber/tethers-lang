use clap::Parser;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tethers_reference_host::cli::{Cli, PlugCommand};

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
    std::env::temp_dir().join(format!("tethers-j24l2-{name}-{}", uuid::Uuid::new_v4()))
}

fn run(args: Vec<OsString>) -> (i32, serde_json::Value) {
    let output = Command::new(host_binary())
        .args(args)
        .output()
        .expect("reference host process");
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "",
        "stderr must be empty"
    );
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 envelope");
    assert_eq!(
        stdout.lines().count(),
        1,
        "expected one JSON line: {stdout}"
    );
    let envelope: serde_json::Value = serde_json::from_str(stdout.trim()).expect("JSON envelope");
    let process_code = output.status.code().expect("process exit code");
    assert_eq!(process_code, envelope["exit_code"].as_i64().unwrap() as i32);
    (process_code, envelope)
}

fn write_package(root: &Path, name: &str) -> PathBuf {
    let provider_bytes =
        std::fs::read(env!("CARGO_BIN_EXE_m3_fixture_provider")).expect("read provider binary");
    let package = root.join(name);
    std::fs::write(
        &package,
        tethers_reference_host::test_fixture_package::build_fixture_package(&provider_bytes)
            .expect("deterministic package"),
    )
    .expect("package bytes");
    package
}

fn wrap_args(args: &[&str]) -> Vec<OsString> {
    args.iter().map(|s| OsString::from(*s)).collect()
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn conformance_snapshot(root: &Path) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    if !root.is_dir() {
        return out;
    }
    let mut entries: Vec<_> = fs::read_dir(root)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();
    for entry in entries {
        let filename = entry.file_name().unwrap().to_string_lossy().to_string();
        if filename.starts_with('.') {
            continue;
        }
        if entry.is_file() {
            let content = fs::read(&entry).expect("read conformance entry");
            out.insert(filename, sha256(&content));
        }
    }
    out
}

// -------------------------------------------------------------------
// Clap parse tests
// -------------------------------------------------------------------

fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(std::iter::once("tethers-reference-host").chain(args.iter().copied()))
}

#[test]
fn j24l2_clap_valid_install_parses() {
    let cli = parse(&[
        "plug",
        "install",
        "--host-data-root",
        "C:\\host",
        "--request",
        "C:\\req.json",
    ])
    .unwrap();
    match cli.command.unwrap() {
        tethers_reference_host::cli::Command::Plug {
            command:
                PlugCommand::Install {
                    host_data_root,
                    request,
                },
        } => {
            assert_eq!(host_data_root, PathBuf::from("C:\\host"));
            assert_eq!(request, PathBuf::from("C:\\req.json"));
        }
        _ => panic!("expected Install"),
    }
}

#[test]
fn j24l2_clap_reordered_options_parse() {
    let cli = parse(&[
        "plug",
        "install",
        "--request",
        "C:\\req.json",
        "--host-data-root",
        "C:\\host",
    ])
    .unwrap();
    match cli.command.unwrap() {
        tethers_reference_host::cli::Command::Plug {
            command: PlugCommand::Install { .. },
        } => {}
        _ => panic!(),
    }
}

#[test]
fn j24l2_clap_missing_host_data_root_rejected() {
    assert!(parse(&["plug", "install", "--request", "C:\\req.json"]).is_err());
}

#[test]
fn j24l2_clap_missing_request_rejected() {
    assert!(parse(&["plug", "install", "--host-data-root", "C:\\host"]).is_err());
}

#[test]
fn j24l2_clap_duplicate_host_data_root_rejected() {
    assert!(parse(&[
        "plug",
        "install",
        "--host-data-root",
        "C:\\a",
        "--host-data-root",
        "C:\\b",
        "--request",
        "C:\\req.json",
    ])
    .is_err());
}

#[test]
fn j24l2_clap_duplicate_request_rejected() {
    assert!(parse(&[
        "plug",
        "install",
        "--host-data-root",
        "C:\\host",
        "--request",
        "C:\\a.json",
        "--request",
        "C:\\b.json",
    ])
    .is_err());
}

#[test]
fn j24l2_clap_unknown_option_rejected() {
    assert!(parse(&[
        "plug",
        "install",
        "--host-data-root",
        "C:\\host",
        "--request",
        "C:\\req.json",
        "--unknown",
    ])
    .is_err());
}

#[test]
fn j24l2_clap_no_package_candidate_option_accepted() {
    assert!(parse(&[
        "plug",
        "install",
        "--host-data-root",
        "C:\\host",
        "--request",
        "C:\\req.json",
        "--package",
        "C:\\pkg.tetherplug",
    ])
    .is_err());
}

#[test]
fn j24l2_clap_equal_sign_accepted() {
    let cli = parse(&[
        "plug",
        "install",
        "--host-data-root=C:\\host",
        "--request=C:\\req.json",
    ])
    .unwrap();
    assert!(matches!(
        cli.command,
        Some(tethers_reference_host::cli::Command::Plug {
            command: PlugCommand::Install { .. }
        })
    ));
}

// -------------------------------------------------------------------
// Windows E2E: fresh install, complete, re-install
// -------------------------------------------------------------------

#[cfg(windows)]
#[test]
fn j24l2_e2e_fresh_install_and_reinstall() {
    let root = temp_dir("e2e");
    let host = root.join("host-data");
    fs::create_dir_all(&host).unwrap();

    let package = write_package(&root, "fixture.tetherplug");

    // 1. Stage the package
    let (code, envelope) = run(wrap_args(&[
        "plug",
        "stage",
        "--host-data-root",
        &host.display().to_string(),
        "--package",
        &package.display().to_string(),
    ]));
    assert_eq!(code, 0, "stage must succeed: {envelope}");
    assert_eq!(envelope["status"], "ok");
    let candidate_id = envelope["data"]["candidate"]["candidate_id"]
        .as_str()
        .expect("candidate_id")
        .to_string();
    assert!(!candidate_id.is_empty());
    assert_eq!(
        envelope["data"]["candidate"]["state"],
        "quarantined_installation_candidate"
    );

    // 2. Write the J24G request
    let request_path = root.join("install-request.json");
    let request = serde_json::json!({
        "schema": "tethers.plug-install/1",
        "candidate_id": &candidate_id,
        "trust": {"scope": "exact_candidate"},
        "conformance": {"allow_non_isolated_supervised_execution": true},
        "installation": {"target_state": "disabled"}
    });
    fs::write(&request_path, serde_json::to_vec(&request).unwrap()).unwrap();

    // 3. Install
    let (code, envelope) = run(wrap_args(&[
        "plug",
        "install",
        "--host-data-root",
        &host.display().to_string(),
        "--request",
        &request_path.display().to_string(),
    ]));
    assert_eq!(code, 0, "install must succeed: {envelope}");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["data"]["result"], "complete");
    assert_eq!(envelope["data"]["candidate_id"], candidate_id);
    assert_eq!(envelope["data"]["step_count"], 4);

    let steps = envelope["data"]["steps"].as_array().expect("steps array");
    assert_eq!(steps.len(), 4, "must have exactly 4 retained steps");

    // Verify step 0: create_exact_candidate_trust
    assert_eq!(steps[0]["before_action"], "create_exact_candidate_trust");
    assert_eq!(steps[0]["after_action"], "run_supervised_conformance");
    assert_eq!(steps[0]["outcome"], "advanced");
    assert_eq!(steps[0]["executed_action"], "create_exact_candidate_trust");

    // Verify step 1: run_supervised_conformance
    assert_eq!(steps[1]["before_action"], "run_supervised_conformance");
    assert_eq!(steps[1]["after_action"], "create_installation_approval");
    assert_eq!(steps[1]["outcome"], "advanced");
    assert_eq!(steps[1]["executed_action"], "run_supervised_conformance");

    // Verify step 2: create_installation_approval
    assert_eq!(steps[2]["before_action"], "create_installation_approval");
    assert_eq!(steps[2]["after_action"], "publish_disabled_installation");
    assert_eq!(steps[2]["outcome"], "advanced");
    assert_eq!(steps[2]["executed_action"], "create_installation_approval");

    // Verify step 3: publish_disabled_installation
    assert_eq!(steps[3]["before_action"], "publish_disabled_installation");
    assert_eq!(steps[3]["after_action"], "complete");
    assert_eq!(steps[3]["outcome"], "advanced");
    assert_eq!(steps[3]["executed_action"], "publish_disabled_installation");

    let installed_id = envelope["data"]["installed_id"]
        .as_str()
        .expect("installed_id");
    assert!(!installed_id.is_empty());
    let installed_record_digest = envelope["data"]["installed_record_digest"]
        .as_str()
        .expect("installed_record_digest");
    assert!(installed_record_digest.starts_with("sha256:"));

    // 4. Prove one disabled installed record + one final destination
    let installed_records_dir = host.join("installed-records");
    assert!(
        installed_records_dir.is_dir(),
        "installed-records must exist"
    );
    let records_before: Vec<_> = fs::read_dir(&installed_records_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(
        records_before.len(),
        1,
        "must have exactly 1 installed record"
    );
    let record_digest = records_before.into_iter().next().unwrap();
    assert!(record_digest.ends_with(".json"), "record must be JSON");

    let install_dir = host.join("install");
    assert!(install_dir.is_dir(), "install must exist");
    let destinations_before: Vec<_> = fs::read_dir(&install_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(
        destinations_before.len(),
        1,
        "must have exactly 1 destination"
    );

    // 5. Prove no publication intent remains
    let intent_dir = host.join("installation-intent");
    let intent_current = intent_dir.join("current.json");
    assert!(
        !intent_current.exists(),
        "no pending publication intent ({})",
        intent_current.display()
    );

    // 6. Prove conformance scratch is clean
    let scratch_dir = host.join("conformance-scratch");
    assert!(
        !scratch_dir.exists() || {
            let scratch_entries = fs::read_dir(&scratch_dir)
                .unwrap()
                .map(|rd| rd.unwrap().file_name().to_string_lossy().to_string())
                .collect::<Vec<_>>();
            let filtered: Vec<_> = scratch_entries
                .into_iter()
                .filter(|n| n != "." && n != "..")
                .collect();
            filtered.is_empty()
        },
        "conformance scratch must be clean"
    );

    // 7. Prove enablements/ exists and is empty
    let enablements_dir = host.join("enablements");
    assert!(enablements_dir.is_dir(), "enablements must exist");
    let enablements_entries: Vec<_> = fs::read_dir(&enablements_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .filter(|n| n != "." && n != "..")
        .collect();
    assert!(enablements_entries.is_empty(), "enablements must be empty");

    // 8. plug list reports exactly one disabled Plug
    let (code, envelope) = run(wrap_args(&[
        "plug",
        "list",
        "--host-data-root",
        &host.display().to_string(),
    ]));
    assert_eq!(code, 0, "plug list must succeed: {envelope}");
    assert_eq!(envelope["status"], "ok");
    let plug_list = envelope["data"]["plugs"].as_array().expect("plugs array");
    assert_eq!(plug_list.len(), 1, "must have exactly 1 plug");
    assert_eq!(plug_list[0]["installed_id"], installed_id);
    assert_eq!(plug_list[0]["state"], "disabled");

    // 9. Snapshot installed records, destination state, and conformance
    let record_snapshot = {
        let mut entries: Vec<_> = fs::read_dir(&installed_records_dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        entries.sort();
        entries
    };
    let destination_snapshot = {
        let mut entries: Vec<_> = fs::read_dir(&install_dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        entries.sort();
        entries
    };
    let conformance_dir = host.join("conformance");
    let conformance_before = conformance_snapshot(&conformance_dir);

    // 10. Invoke plug install again
    let (code, envelope) = run(wrap_args(&[
        "plug",
        "install",
        "--host-data-root",
        &host.display().to_string(),
        "--request",
        &request_path.display().to_string(),
    ]));
    assert_eq!(code, 0, "second install must succeed: {envelope}");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["data"]["result"], "complete");
    assert_eq!(envelope["data"]["step_count"], 1);

    let steps2 = envelope["data"]["steps"].as_array().expect("steps array");
    assert_eq!(steps2.len(), 1, "must have 1 step on re-install");
    assert_eq!(steps2[0]["before_action"], "complete");
    assert_eq!(steps2[0]["after_action"], "complete");
    assert_eq!(steps2[0]["outcome"], "already_complete");

    // 11. Prove record count and destination unchanged
    let records_after: Vec<_> = fs::read_dir(&installed_records_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(records_after, record_snapshot, "records must be unchanged");

    let destinations_after: Vec<_> = fs::read_dir(&install_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(
        destinations_after, destination_snapshot,
        "destinations must be unchanged"
    );

    // 12. Prove no conformance retry — exact byte-level snapshot equality
    let conformance_after = conformance_snapshot(&conformance_dir);
    assert_eq!(
        conformance_after, conformance_before,
        "conformance store must be unchanged: no retry, no added/removed/changed evidence"
    );

    // 13. Prove enablements is still empty
    let enablements_after: Vec<_> = fs::read_dir(&enablements_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .filter(|n| n != "." && n != "..")
        .collect();
    assert!(
        enablements_after.is_empty(),
        "enablements must remain empty"
    );

    // 14. Prove no intent was created on second install
    assert!(!intent_current.exists(), "no intent after re-install");

    // Cleanup
    fs::remove_dir_all(root).unwrap();
}
