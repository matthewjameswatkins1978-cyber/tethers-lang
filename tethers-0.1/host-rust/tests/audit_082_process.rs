// Tethers 0.8.2 adversarial-hardening regression suite (Lane B evidence).
//
// RSK-04 proof: a `tethers exec` timeout terminates the whole spawned POSIX
// process tree (process-group SIGKILL), not merely the immediate child, and
// the machine-readable result reports its supervision contract truthfully.
// The pre-0.8.2 POSIX path killed only the direct child and self-reported
// "child_process_only"; the descendant-survival assertion below fails
// against that implementation.

use std::path::PathBuf;
use std::process::Command;

fn host_binary() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_tethers-reference-host")
        .or_else(|| std::env::var_os("CARGO_BIN_EXE_tethers_reference_host"))
        .map(PathBuf::from)
        .or_else(|| {
            std::env::current_exe().ok().and_then(|path| {
                path.parent()?.parent().map(|dir| {
                    dir.join(if cfg!(windows) {
                        "tethers-reference-host.exe"
                    } else {
                        "tethers-reference-host"
                    })
                })
            })
        })
        .expect("compiled reference host binary")
}

#[cfg(unix)]
#[test]
fn audit_082_exec_timeout_terminates_whole_posix_tree() {
    let base = std::env::temp_dir().join(format!("audit-082-exec-tree-{}", uuid::Uuid::new_v4()));
    let workspace = base.join("workspace");
    let state = base.join("state");
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::create_dir_all(&state).unwrap();
    let pidfile = workspace.join("descendant.pid");
    let script = format!("sleep 60 & echo $! > '{}'; wait", pidfile.display());

    let output = Command::new(host_binary())
        .args([
            "exec",
            "--program",
            "sh",
            "--arg",
            "-c",
            "--arg",
            &script,
            "--timeout-ms",
            "2000",
        ])
        .current_dir(&workspace)
        .env("TETHERS_HOST_DATA_ROOT", &state)
        .output()
        .expect("run host binary");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout must be a JSON envelope");
    assert_eq!(envelope["status"], "ok", "exec envelope: {stdout}");
    assert_eq!(envelope["data"]["timed_out"], true);
    assert_eq!(envelope["data"]["supervision"], "process_group_sigkill");
    assert_eq!(envelope["data"]["resource_limits"], "none_applied");

    // The deliberately spawned descendant must not outlive the timeout.
    let descendant: i32 = std::fs::read_to_string(&pidfile)
        .expect("descendant pidfile")
        .trim()
        .parse()
        .expect("descendant pid");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    // SAFETY: pure liveness probe.
    while unsafe { libc::kill(descendant, 0) } == 0 {
        assert!(
            std::time::Instant::now() < deadline,
            "descendant {descendant} survived the exec timeout: the process tree was not terminated"
        );
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    let _ = std::fs::remove_dir_all(&base);
}

#[cfg(windows)]
#[test]
fn audit_082_exec_timeout_reports_truthful_supervision_on_windows() {
    let base = std::env::temp_dir().join(format!("audit-082-exec-tree-{}", uuid::Uuid::new_v4()));
    let workspace = base.join("workspace");
    let state = base.join("state");
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::create_dir_all(&state).unwrap();

    let output = Command::new(host_binary())
        .args([
            "exec",
            "--program",
            "cmd.exe",
            "--arg",
            "/c",
            "--arg",
            "ping -n 30 127.0.0.1 > NUL",
            "--timeout-ms",
            "2000",
        ])
        .current_dir(&workspace)
        .env("TETHERS_HOST_DATA_ROOT", &state)
        .output()
        .expect("run host binary");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout must be a JSON envelope");
    assert_eq!(envelope["status"], "ok", "exec envelope: {stdout}");
    assert_eq!(envelope["data"]["timed_out"], true);
    assert_eq!(
        envelope["data"]["supervision"],
        "process_tree_best_effort_taskkill"
    );
    assert_eq!(envelope["data"]["resource_limits"], "none_applied");
    let _ = std::fs::remove_dir_all(&base);
}
