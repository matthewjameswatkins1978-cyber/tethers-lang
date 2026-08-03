use serde_json::Value;
use std::fs;
use std::process::Command;

fn host_binary() -> std::path::PathBuf {
    std::env::var_os("CARGO_BIN_EXE_tethers_reference_host")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::current_exe()
                .ok()?
                .parent()?
                .parent()
                .map(|path| path.join("tethers-reference-host.exe"))
        })
        .expect("compiled reference host binary")
}

fn run(root: &std::path::Path) -> (i32, Value) {
    let output = Command::new(host_binary())
        .args(["plug", "list", "--host-data-root"])
        .arg(root)
        .output()
        .expect("reference host process");
    (
        output.status.code().unwrap(),
        serde_json::from_slice(&output.stdout).unwrap(),
    )
}

#[test]
fn empty_root_is_successful_and_read_only() {
    let root = std::env::temp_dir().join(format!("tethers-j24b-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let before = fs::read_dir(&root).unwrap().count();
    let (code, envelope) = run(&root);
    assert_eq!(code, 0);
    assert_eq!(envelope["command"], "plug list");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["exit_code"], 0);
    assert_eq!(envelope["data"]["count"], 0);
    assert!(envelope["data"]["plugs"].as_array().unwrap().is_empty());
    assert_eq!(fs::read_dir(&root).unwrap().count(), before);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn missing_and_partial_roots_fail_closed_without_creation() {
    let missing =
        std::env::temp_dir().join(format!("tethers-j24b-missing-{}", uuid::Uuid::new_v4()));
    let (code, envelope) = run(&missing);
    assert_eq!(code, 4);
    assert_eq!(envelope["status"], "unavailable");
    assert_eq!(envelope["error"]["code"], "plug_data_root_unavailable");
    assert!(!missing.exists());

    let root = std::env::temp_dir().join(format!("tethers-j24b-partial-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(root.join("install")).unwrap();
    let (code, envelope) = run(&root);
    assert_eq!(code, 3);
    assert_eq!(envelope["status"], "invalid_data");
    assert_eq!(envelope["error"]["code"], "plug_store_incomplete");
    assert!(!root.join("installed-records").exists());
    fs::remove_dir_all(root).unwrap();
}
