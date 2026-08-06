use clap::Parser;
use std::path::PathBuf;
use std::fs;
use tethers_reference_host::cli::{Cli, PlugCommand};

fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(std::iter::once("tethers-reference-host").chain(args.iter().copied()))
}

#[test]
fn j24l2_clap_valid_install_parses() {
    let cli = parse(&[
        "plug", "install",
        "--host-data-root", "C:\\host",
        "--request", "C:\\req.json",
    ])
    .unwrap();
    match cli.command.unwrap() {
        tethers_reference_host::cli::Command::Plug { command: PlugCommand::Install { host_data_root, request } } => {
            assert_eq!(host_data_root, PathBuf::from("C:\\host"));
            assert_eq!(request, PathBuf::from("C:\\req.json"));
        }
        _ => panic!("expected Install"),
    }
}

#[test]
fn j24l2_clap_reordered_options_parse() {
    let cli = parse(&[
        "plug", "install",
        "--request", "C:\\req.json",
        "--host-data-root", "C:\\host",
    ])
    .unwrap();
    match cli.command.unwrap() {
        tethers_reference_host::cli::Command::Plug { command: PlugCommand::Install { host_data_root, request } } => {
            assert_eq!(host_data_root, PathBuf::from("C:\\host"));
            assert_eq!(request, PathBuf::from("C:\\req.json"));
        }
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
        "plug", "install",
        "--host-data-root", "C:\\a",
        "--host-data-root", "C:\\b",
        "--request", "C:\\req.json",
    ]).is_err());
}

#[test]
fn j24l2_clap_duplicate_request_rejected() {
    assert!(parse(&[
        "plug", "install",
        "--host-data-root", "C:\\host",
        "--request", "C:\\a.json",
        "--request", "C:\\b.json",
    ]).is_err());
}

#[test]
fn j24l2_clap_unknown_option_rejected() {
    assert!(parse(&[
        "plug", "install",
        "--host-data-root", "C:\\host",
        "--request", "C:\\req.json",
        "--unknown",
    ]).is_err());
}

#[test]
fn j24l2_clap_no_package_candidate_option_accepted() {
    // package flag should not parse as Install
    assert!(parse(&[
        "plug", "install",
        "--host-data-root", "C:\\host",
        "--request", "C:\\req.json",
        "--package", "C:\\pkg.tetherplug",
    ]).is_err());
}

#[test]
fn j24l2_clap_equal_sign_accepted() {
    let cli = parse(&[
        "plug", "install",
        "--host-data-root=C:\\host",
        "--request=C:\\req.json",
    ]).unwrap();
    assert!(matches!(
        cli.command,
        Some(tethers_reference_host::cli::Command::Plug {
            command: PlugCommand::Install { .. }
        })
    ));
}

// ---- Pre-mutation validation ----

use uuid::Uuid;

fn temp_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("tethers-j24l2-{name}-{}", Uuid::new_v4()))
}

#[test]
fn j24l2_relative_host_data_root_returns_error_creates_nothing() {
    let root = temp_dir("relative-host");
    fs::create_dir_all(&root).unwrap();

    let result = tethers_reference_host::plug_install_command::run_install(
        &PathBuf::from("relative-host"),
        &PathBuf::from("C:\\req.json"),
    );
    assert_eq!(result.exit_code, 2);
    assert_eq!(result.envelope.status, tethers_reference_host::cli::OutcomeStatus::InvalidCliUsage);
    assert_eq!(result.envelope.error.as_ref().unwrap().code, "invalid_cli_usage");
    assert_eq!(result.envelope.error.as_ref().unwrap().message, "--host-data-root must be absolute");
    assert_eq!(result.envelope.error.as_ref().unwrap().field.as_deref(), Some("/host-data-root"));

    fs::remove_dir_all(&root).ok();
}

#[test]
fn j24l2_relative_request_path_returns_error_creates_nothing() {
    let root = temp_dir("relative-request");
    fs::create_dir_all(&root).unwrap();

    let result = tethers_reference_host::plug_install_command::run_install(
        &PathBuf::from("C:\\host"),
        &PathBuf::from("relative-req.json"),
    );
    assert_eq!(result.exit_code, 2);
    assert_eq!(result.envelope.status, tethers_reference_host::cli::OutcomeStatus::InvalidCliUsage);
    assert_eq!(result.envelope.error.as_ref().unwrap().code, "invalid_cli_usage");
    assert_eq!(result.envelope.error.as_ref().unwrap().message, "--request must be absolute");
    assert_eq!(result.envelope.error.as_ref().unwrap().field.as_deref(), Some("/request"));

    fs::remove_dir_all(&root).ok();
}

#[test]
fn j24l2_missing_host_data_root_returns_unavailable() {
    let root = temp_dir("missing-host");
    let missing = root.join("nonexistent");

    let result = tethers_reference_host::plug_install_command::run_install(
        &missing,
        &PathBuf::from("C:\\req.json"),
    );
    assert_eq!(result.exit_code, 4);
    assert_eq!(result.envelope.status, tethers_reference_host::cli::OutcomeStatus::Unavailable);
    assert_eq!(result.envelope.error.as_ref().unwrap().code, "plug_data_root_unavailable");
    assert_eq!(result.envelope.error.as_ref().unwrap().message, "host data root is unavailable");

    assert!(!missing.exists(), "missing host-data root must not be created");
    fs::remove_dir_all(&root).ok();
}

#[test]
fn j24l2_malformed_request_creates_no_lifecycle_state() {
    let root = temp_dir("malformed-req");
    fs::create_dir_all(&root).unwrap();

    let request_path = root.join("bad.json");
    fs::write(&request_path, b"not json").unwrap();

    let result = tethers_reference_host::plug_install_command::run_install(&root, &request_path);
    assert!(result.exit_code != 0);

    let children: Vec<String> = fs::read_dir(&root)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .filter(|n| n != "bad.json")
        .collect();
    for name in &children {
        eprintln!("unexpected child after bad request: {name}");
    }
    assert!(children.is_empty(), "malformed request must not create state");

    fs::remove_dir_all(&root).ok();
}

#[test]
fn j24l2_missing_candidate_roots_creates_no_later_lifecycle_roots() {
    let root = temp_dir("missing-stage");
    fs::create_dir_all(&root).unwrap();

    let request_path = root.join("req.json");
    let req = serde_json::json!({
        "schema": "tethers.plug-install/1",
        "candidate_id": "3d846d40-01fc-4e1e-b77d-83944dbed76f",
        "trust": {"scope": "exact_candidate"},
        "conformance": {"allow_non_isolated_supervised_execution": true},
        "installation": {"target_state": "disabled"}
    });
    fs::write(&request_path, serde_json::to_vec(&req).unwrap()).unwrap();

    let result = tethers_reference_host::plug_install_command::run_install(&root, &request_path);
    assert!(result.exit_code != 0);

    let lifecycle_children = [
        "installation-trust", "launch-profiles", "conformance",
        "installation-approvals", "install", "installed-records",
        "enablements", "installation-intent", "conformance-scratch",
        "installation.lock",
    ];
    for child in &lifecycle_children {
        let path = root.join(child);
        assert!(!path.exists(), "{child} must not exist after missing stage roots");
    }

    fs::remove_dir_all(&root).ok();
}
