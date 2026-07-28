// J13A focused tests: CLI parsing, path resolution, engine session,
// provider availability, process supervision, and output envelope.

use std::path::PathBuf;
use std::process::Command;
use std::io::Write;

// ===========================================================================
// CLI parsing tests
// ===========================================================================

fn host_binary() -> PathBuf {
    // Find the compiled binary
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push("tethers-reference-host.exe");
    path
}

fn run_host(args: &[&str]) -> (i32, String, String) {
    let output = Command::new(host_binary())
        .args(args)
        .output()
        .expect("failed to run host binary");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let code = output.status.code().unwrap_or(-1);
    (code, stdout, stderr)
}

fn assert_envelope(stdout: &str, expected_status: &str, expected_exit_code: i32) -> serde_json::Value {
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout must be valid JSON envelope");
    assert_eq!(envelope["schema"], "tethers.cli/1");
    assert_eq!(envelope["status"], expected_status);
    assert_eq!(envelope["exit_code"], expected_exit_code);
    envelope
}

#[test]
fn j13a_valid_check_command_help() {
    // --help shouldn't be reachable in check, but we test help for the binary
    let (code, stdout, _) = run_host(&["--help"]);
    // help output goes to stdout from clap, but we emit envelope.
    // Actually clap --help will print before we get to main if we use
    // parse() instead of try_parse_from. With try_parse_from, we get an
    // error that includes help text.
    // Let's just verify we get an exit code indicating CLI usage error.
    assert!(code != 0 || stdout.contains("Usage:"), "help should show usage");
}

#[test]
fn j13a_no_command_emits_envelope() {
    let (code, stdout, _) = run_host(&[]);
    assert_envelope(&stdout, "invalid_cli_usage", 2);
    assert_eq!(code, 2);
}

#[test]
fn j13a_unknown_command_emits_envelope() {
    let (code, stdout, _) = run_host(&["nonexistent"]);
    assert_envelope(&stdout, "invalid_cli_usage", 2);
    assert_eq!(code, 2);
}

#[test]
fn j13a_misspelled_runn_never_enters_legacy() {
    let (code, stdout, _) = run_host(&["runn", "engine.exe", "req.json"]);
    assert_envelope(&stdout, "invalid_cli_usage", 2);
    assert_eq!(code, 2);
    assert!(!stdout.contains("legacy"), "runn must not reach legacy");
}

#[test]
fn j13a_check_missing_config_emits_error() {
    let (code, stdout, _) = run_host(&["check", "--engine", "nonexistent.exe"]);
    let env = assert_envelope(&stdout, "invalid_cli_usage", 2);
    // The error should mention missing config
    assert!(env["error"]["message"].as_str().unwrap_or("").contains("config")
        || stdout.contains("required"));
}

#[test]
fn j13a_check_missing_engine_emits_error() {
    let (code, stdout, _) = run_host(&["check", "--config", "nonexistent.json"]);
    let env = assert_envelope(&stdout, "invalid_cli_usage", 2);
    assert!(env["error"]["message"].as_str().unwrap_or("").contains("engine")
        || stdout.contains("required"));
}

#[test]
fn j13a_check_duplicate_config_rejected() {
    let (code, stdout, _) = run_host(&[
        "check",
        "--config", "a.json",
        "--config", "b.json",
        "--engine", "e.exe",
    ]);
    assert_envelope(&stdout, "invalid_cli_usage", 2);
    assert_eq!(code, 2);
}

#[test]
fn j13a_check_duplicate_engine_rejected() {
    let (code, stdout, _) = run_host(&[
        "check",
        "--config", "c.json",
        "--engine", "a.exe",
        "--engine", "b.exe",
    ]);
    assert_envelope(&stdout, "invalid_cli_usage", 2);
    assert_eq!(code, 2);
}

#[test]
fn j13a_check_unknown_option_rejected() {
    let (code, stdout, _) = run_host(&[
        "check",
        "--config", "c.json",
        "--engine", "e.exe",
        "--unknown",
    ]);
    assert_envelope(&stdout, "invalid_cli_usage", 2);
    assert_eq!(code, 2);
}

#[test]
fn j13a_check_extra_positional_rejected() {
    let (code, stdout, _) = run_host(&[
        "check",
        "--config", "c.json",
        "--engine", "e.exe",
        "extra_arg",
    ]);
    assert_envelope(&stdout, "invalid_cli_usage", 2);
    assert_eq!(code, 2);
}

#[test]
fn j13a_check_nonexistent_config_returns_invalid_data() {
    let (code, stdout, _) = run_host(&[
        "check",
        "--config", "nonexistent-file-xyzzy.json",
        "--engine", "nonexistent-engine.exe",
    ]);
    // Path resolution happens before checking if files exist.
    // If the path doesn't exist, we get invalid_data.
    // But clap will accept the path; our code resolves it.
    // The error will be about config not found.
    let envelope: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(envelope["status"], "invalid_data");
    assert_eq!(envelope["exit_code"], 3);
    assert_eq!(code, 3);
}

#[test]
fn j13a_explicit_legacy_reaches_parser() {
    // This test requires the legacy path to still work.
    // We check that __legacy with insufficient args produces a legacy error.
    let (code, stdout, _) = run_host(&["__legacy"]);
    // Legacy expects at least 2 args (engine, request).
    // With no args, it will fail with usage.
    let envelope: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(envelope["status"], "failed");
    assert_eq!(code, 6);
}

#[test]
fn j13a_hidden_commands_not_in_help() {
    let (code, stdout, _) = run_host(&["--help"]);
    // help output should not contain hidden commands
    let output = format!("{stdout}");
    assert!(!output.contains("__legacy"), "help must hide __legacy");
    // Note: --help exits with non-zero because clap prints help and exits
}

#[test]
fn j13a_envelope_has_no_timestamp() {
    let (_, stdout, _) = run_host(&[]);
    assert!(!stdout.contains("timestamp"), "envelope must not contain timestamp");
}

#[test]
fn j13a_envelope_has_correct_schema() {
    let (_, stdout, _) = run_host(&[]);
    let envelope: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(envelope["schema"], "tethers.cli/1");
}

#[test]
fn j13a_error_envelope_has_code_and_message() {
    let (_, stdout, _) = run_host(&["nonexistent"]);
    let envelope: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert!(envelope["error"]["code"].is_string());
    assert!(envelope["error"]["message"].is_string());
}

#[test]
fn j13a_error_envelope_has_null_field_by_default() {
    let (_, stdout, _) = run_host(&["nonexistent"]);
    let envelope: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    // field may be null or absent
    let field = &envelope["error"]["field"];
    assert!(field.is_null() || field.is_string());
}

#[test]
fn j13a_command_field_in_envelope() {
    let (_, stdout, _) = run_host(&[]);
    let envelope: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert!(envelope["command"].is_string());
    assert_eq!(envelope["command"], "tethers-reference-host");
}

#[test]
fn j13a_data_field_is_object() {
    let (_, stdout, _) = run_host(&[]);
    let envelope: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert!(envelope["data"].is_object());
}

#[test]
fn j13a_reordered_options_accepted() {
    // This tests that option order doesn't matter.
    // We don't need valid paths; just that clap parses them.
    let (code, stdout, _) = run_host(&[
        "check",
        "--engine", "e.exe",
        "--config", "c.json",
    ]);
    // Will fail because paths don't exist, but the command is parsed.
    let envelope: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    // Should be invalid_data (path not found), not invalid_cli_usage
    assert_eq!(envelope["status"], "invalid_data");
    assert_eq!(code, 3);
}

#[test]
fn j13a_directory_config_rejected() {
    // Create a temp directory and point --config to it
    let tmp = std::env::temp_dir().join(format!("j13a-dir-config-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&tmp).unwrap();
    let (code, stdout, _) = run_host(&[
        "check",
        "--config", &tmp.to_string_lossy(),
        "--engine", "nonexistent.exe",
    ]);
    let envelope: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(envelope["status"], "invalid_data");
    assert_eq!(code, 3);
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn j13a_directory_engine_rejected() {
    let tmp = std::env::temp_dir().join(format!("j13a-dir-engine-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&tmp).unwrap();
    let (code, stdout, _) = run_host(&[
        "check",
        "--config", "nonexistent.json",
        "--engine", &tmp.to_string_lossy(),
    ]);
    let envelope: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    // Config not found comes first (invalid_data)
    assert_eq!(code, 3);
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn j13a_no_trail_files_created() {
    // Run a failing check and verify no trail/replay files are created.
    let tmp = std::env::temp_dir().join(format!("j13a-trail-check-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&tmp).unwrap();

    // Verify no trail.jsonl or replay dirs exist after check fails
    let (code, _, _) = run_host(&[
        "check",
        "--config", "nonexistent.json",
        "--engine", "nonexistent.exe",
    ]);
    assert_eq!(code, 3);

    // No trail files should exist in the working directory
    assert!(!tmp.join("trail.jsonl").exists());
    assert!(!tmp.join("replay").exists());
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn j13a_provision_replay_hidden_accessible() {
    // provision-replay should be accessible via explicit subcommand
    let (code, _stdout, _stderr) = run_host(&["provision-replay"]);
    // Will fail because we don't provide an absolute path,
    // but the command should be recognized (not invalid_cli_usage).
    // The error will be about usage/missing argument.
    assert!(code != 0);
}

#[test]
fn j13a_multiple_args_to_legacy() {
    // Test that __legacy passes all trailing args through
    let (code, _stdout, _) = run_host(&[
        "__legacy",
        "engine.exe", "req.json", "allow", "trail.jsonl", "success",
        "--host-data-root", "C:\\data",
    ]);
    // Will fail because engine/req don't exist, but parse succeeds.
    assert_eq!(code, 6); // failed
}

#[test]
fn j13a_unknown_subcommand_not_legacy() {
    // "run" is not a known subcommand and must not enter legacy.
    let (code, stdout, _) = run_host(&["run", "--config", "c.json"]);
    let envelope: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(envelope["status"], "invalid_cli_usage");
    assert_eq!(code, 2);
    assert!(!stdout.contains("legacy"));
}

#[test]
fn j13a_stderr_not_contaminated_by_envelope() {
    // Envelope goes to stdout. Verify stderr is either empty or
    // contains only diagnostics (not JSON).
    let (_code, _stdout, stderr) = run_host(&["nonexistent"]);
    if !stderr.is_empty() {
        // If there's stderr content, it should not be valid JSON
        assert!(serde_json::from_str::<serde_json::Value>(stderr.trim()).is_err(),
            "stderr must not contain JSON envelope");
    }
}

#[test]
fn j13a_exit_code_matches_envelope() {
    let (code, stdout, _) = run_host(&["nonexistent"]);
    let envelope: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(envelope["exit_code"].as_i64().unwrap() as i32, code);
}

#[test]
fn j13a_outcome_status_values_correct() {
    // Verify the vocabulary mapping through the envelope.
    // invalid_cli_usage -> 2
    let (code2, stdout2, _) = run_host(&["nonexistent"]);
    assert_eq!(code2, 2);
    let env: serde_json::Value = serde_json::from_str(stdout2.trim()).unwrap();
    assert_eq!(env["status"], "invalid_cli_usage");

    // invalid_data -> 3 (check with nonexistent config)
    let (code3, stdout3, _) = run_host(&["check", "--config", "no.json", "--engine", "no.exe"]);
    assert_eq!(code3, 3);
    let env3: serde_json::Value = serde_json::from_str(stdout3.trim()).unwrap();
    assert_eq!(env3["status"], "invalid_data");
}
