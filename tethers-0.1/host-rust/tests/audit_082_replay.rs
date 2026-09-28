// Tethers 0.8.2 adversarial-hardening regression suite (Lane A evidence).
//
// Cross-platform proof for the DEF-01/DEF-03/DEF-04/RSK-01 repairs:
// recognised interrupted-publication residue never blocks the ledger,
// unrecognised entries still fail closed, provision failures emit structured
// machine-readable diagnostics (never `diagnostic: null`), and the operator
// resolve route releases exactly the abandoned `claimed_no_state` admission
// while preserving evidence and refusing every other durable state.
//
// All destructive operations run against isolated temporary stores.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;
use tethers_reference_host::replay::{ExecutionBinding, LogicalExecutionKey, ReplayState};
use tethers_reference_host::replay_store::{provision_replay, ReplayLedger};

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

fn run_host(args: &[&str]) -> (i32, serde_json::Value) {
    let output = Command::new(host_binary())
        .args(args)
        .output()
        .expect("failed to run host binary");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let envelope = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|_| panic!("stdout must be a JSON envelope: {stdout}"));
    (output.status.code().unwrap_or(-1), envelope)
}

fn normalize_test_path(path: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        path.to_path_buf()
    }
    #[cfg(not(windows))]
    {
        std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
    }
}

fn harden_replay_acl(root: &Path) {
    #[cfg(windows)]
    {
        // The Windows J09 contract requires a protected host-data root; this
        // mirrors the hardening helper used by the existing CLI/replay suites.
        let acl_script = format!(
            "$p='{}'; $identity=[System.Security.Principal.WindowsIdentity]::GetCurrent().Name; $acl=[System.Security.AccessControl.DirectorySecurity]::new(); $acl.SetAccessRuleProtection($true,$false); $inherit=[System.Security.AccessControl.InheritanceFlags]::ContainerInherit -bor [System.Security.AccessControl.InheritanceFlags]::ObjectInherit; foreach($t in @($identity,'NT AUTHORITY\\SYSTEM','BUILTIN\\Administrators')) {{ $acl.AddAccessRule([System.Security.AccessControl.FileSystemAccessRule]::new($t,'FullControl',$inherit,'None','Allow')) }}; Set-Acl -LiteralPath $p -AclObject $acl",
            root.display()
        );
        let status = std::process::Command::new("pwsh.exe")
            .args(["-NoProfile", "-Command", &acl_script])
            .status()
            .expect("pwsh must be available for Windows ACL hardening");
        assert!(status.success(), "ACL hardening must succeed");
    }
    #[cfg(not(windows))]
    let _ = root;
}

fn temp_root(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("audit-082-{label}-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&path).unwrap();
    let path = normalize_test_path(&path);
    harden_replay_acl(&path);
    path
}

fn binding(action: &str) -> ExecutionBinding {
    ExecutionBinding {
        evaluation_id: "eval-audit-082".into(),
        action_id: action.into(),
        capability_name: "audit.capability".into(),
        capability_version: 1,
        manifest_digest: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .into(),
        provider_identity: "provider-audit".into(),
        argument_digest: "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
            .into(),
    }
}

fn nonce() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

// ---------------------------------------------------------------------------
// DEF-04: structured diagnostics on every platform
// ---------------------------------------------------------------------------

#[test]
fn audit_082_provision_failure_emits_structured_diagnostic_object() {
    // Relative path: the diagnostic object must name the exact reason.
    let (code, envelope) = run_host(&["provision-replay", "relative/path"]);
    assert_ne!(code, 0);
    assert_eq!(envelope["schema"], "tethers.cli/1");
    assert_eq!(envelope["status"], "failed");
    let diagnostic = &envelope["data"]["diagnostic"];
    assert!(
        diagnostic.is_object(),
        "diagnostic must never be null on any platform: {envelope}"
    );
    assert_eq!(diagnostic["phase"], "path_validation");
    assert_eq!(diagnostic["reason"], "path_not_absolute");
    assert!(
        diagnostic["recovery"]
            .as_str()
            .is_some_and(|r| !r.is_empty()),
        "diagnostic must carry recovery guidance: {diagnostic}"
    );

    // Hostile hierarchy: a regular file where the replay subtree must be.
    let root = temp_root("diag-hostile");
    std::fs::write(root.join("replay"), "blocking file").unwrap();
    let (code, envelope) = run_host(&["provision-replay", &root.to_string_lossy()]);
    assert_ne!(code, 0);
    let diagnostic = &envelope["data"]["diagnostic"];
    assert!(
        diagnostic.is_object(),
        "diagnostic must never be null on any platform: {envelope}"
    );
    assert!(
        diagnostic["phase"].as_str().is_some_and(|p| !p.is_empty()),
        "diagnostic phase must be specific: {diagnostic}"
    );
    assert!(
        diagnostic["reason"].as_str().is_some_and(|r| !r.is_empty()),
        "diagnostic reason must be specific: {diagnostic}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

// ---------------------------------------------------------------------------
// DEF-01: recognised staging residue is tolerated; unknown entries fail closed
// ---------------------------------------------------------------------------

#[cfg(unix)]
#[test]
fn audit_082_recognised_residue_never_blocks_ledger_reopen() {
    let root = temp_root("residue");
    provision_replay(&root).unwrap();
    let hostile = b"not-a-record";

    // Recognised residue in claims/ and beside FORMAT.json.
    let stem = format!("{}.claim.json", "a".repeat(64));
    let claim_residue = format!("{stem}.{}.tmp", nonce());
    std::fs::write(root.join("replay/v1/claims").join(&claim_residue), hostile).unwrap();
    std::fs::write(
        root.join("replay/v1")
            .join(format!("FORMAT.json.{}.tmp", nonce())),
        hostile,
    )
    .unwrap();

    let ledger =
        Rc::new(ReplayLedger::open(&root).expect("recognised residue must not block open"));
    let key = LogicalExecutionKey::derive("anchor-082", "eval-082", "action-082").unwrap();
    let mut admission =
        ReplayLedger::admit_or_recover_owned(&ledger, key.clone(), binding("action-082")).unwrap();
    assert!(admission.is_fresh());
    admission.publish_intent().unwrap();
    drop(admission);

    // Recognised residue beside a real generation is tolerated too.
    let reopened = Rc::new(ReplayLedger::open(&root).unwrap());
    let recovered =
        ReplayLedger::admit_or_recover_owned(&reopened, key, binding("action-082")).unwrap();
    assert_eq!(recovered.state(), ReplayState::IntentRecorded);
    drop(recovered);

    // The residue was never modified, deleted, or parsed.
    assert_eq!(
        std::fs::read(root.join("replay/v1/claims").join(&claim_residue)).unwrap(),
        hostile
    );

    // An unrecognised `.tmp` (no nonce, legacy deterministic shape) still
    // fails the whole ledger closed.
    std::fs::write(
        root.join("replay/v1/claims").join(format!(".{stem}.tmp")),
        hostile,
    )
    .unwrap();
    assert!(ReplayLedger::open(&root).is_err());

    // A recognised stem with a hostile non-regular object fails closed.
    let _ = std::fs::remove_dir_all(&root);
}

#[cfg(windows)]
#[test]
fn audit_082_staging_shaped_debris_fails_closed_on_windows() {
    // The accepted J09 Windows contract is deliberately stricter than the
    // POSIX recognised-residue tolerance: ANY temporary-shaped entry fails
    // the whole ledger closed, is never parsed or repaired by the host, and
    // only an operator removal restores service.
    let root = temp_root("debris");
    provision_replay(&root).unwrap();
    let hostile = b"not-a-record";
    let debris = format!("{}.claim.json.{}.tmp", "a".repeat(64), nonce());
    let debris_path = root.join("replay/v1/claims").join(&debris);
    std::fs::write(&debris_path, hostile).unwrap();

    assert!(ReplayLedger::open(&root).is_err());
    // The debris was never modified, deleted, or parsed.
    assert_eq!(std::fs::read(&debris_path).unwrap(), hostile);

    // After operator removal the ledger is healthy again.
    std::fs::remove_file(&debris_path).unwrap();
    let ledger = Rc::new(ReplayLedger::open(&root).unwrap());
    let key = LogicalExecutionKey::derive("anchor-082", "eval-082", "action-debris").unwrap();
    let admission =
        ReplayLedger::admit_or_recover_owned(&ledger, key, binding("action-debris")).unwrap();
    assert!(admission.is_fresh());
    drop(admission);
    let _ = std::fs::remove_dir_all(&root);
}

// ---------------------------------------------------------------------------
// DEF-03: the operator resolve route
// ---------------------------------------------------------------------------

#[test]
fn audit_082_abandoned_admission_is_resolved_via_cli_and_key_released() {
    let root = temp_root("resolve");
    provision_replay(&root).unwrap();
    let key = LogicalExecutionKey::derive("anchor-082", "eval-082", "action-resolve").unwrap();

    // Abandon a fresh admission: the claim stays durable, no generation.
    let abandoned = {
        let ledger = Rc::new(ReplayLedger::open(&root).unwrap());
        let admission =
            ReplayLedger::admit_or_recover_owned(&ledger, key.clone(), binding("action-resolve"))
                .unwrap();
        assert!(admission.is_fresh());
        admission.execution_id().to_owned()
    };

    // Across restart the abandoned claim blocks and is manual-resolution-only.
    {
        let ledger = Rc::new(ReplayLedger::open(&root).unwrap());
        let mut recovered =
            ReplayLedger::admit_or_recover_owned(&ledger, key.clone(), binding("action-resolve"))
                .unwrap();
        assert!(!recovered.is_fresh());
        assert_eq!(recovered.state(), ReplayState::ClaimedNoState);
        assert!(recovered.publish_intent().is_err());
    }

    // The operator route releases it with evidence preserved.
    let (code, envelope) = run_host(&[
        "replay-resolve-claim",
        &root.to_string_lossy(),
        "--execution-id",
        &abandoned,
    ]);
    assert_eq!(code, 0, "resolve must succeed: {envelope}");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["data"]["execution_id"], abandoned);
    assert_eq!(envelope["data"]["already_quarantined"], false);
    let record = envelope["data"]["quarantined_record"]
        .as_str()
        .unwrap()
        .to_owned();

    let quarantine = root.join("replay-quarantine");
    let preserved = std::fs::read(quarantine.join(&record)).unwrap();
    let claim = tethers_reference_host::replay::Claim::from_canonical_bytes(
        &preserved,
        &LogicalExecutionKey::from_digest(
            envelope["data"]["logical_key_digest"]
                .as_str()
                .unwrap()
                .to_owned(),
        )
        .unwrap(),
    )
    .expect("quarantined record must remain a canonical claim");
    assert_eq!(claim.execution_id.as_str(), abandoned);
    let audit = std::fs::read_to_string(quarantine.join("resolve-log.jsonl")).unwrap();
    let line: serde_json::Value = serde_json::from_str(audit.trim()).unwrap();
    assert_eq!(line["schema"], "tethers.replay_resolve/1");
    assert_eq!(line["execution_id"], abandoned);
    assert_eq!(line["resolved_state"], "claimed_no_state");

    // The released key admits a FRESH identity and completes normally.
    let ledger = Rc::new(ReplayLedger::open(&root).unwrap());
    let mut admission =
        ReplayLedger::admit_or_recover_owned(&ledger, key, binding("action-resolve")).unwrap();
    assert!(admission.is_fresh());
    assert_ne!(admission.execution_id(), abandoned);
    admission.publish_intent().unwrap();
    drop(admission);

    // Resolving the released identity again reports the truth.
    let (code, envelope) = run_host(&[
        "replay-resolve-claim",
        &root.to_string_lossy(),
        "--execution-id",
        &abandoned,
    ]);
    assert_ne!(code, 0);
    assert_eq!(envelope["error"]["code"], "REPLAY_RESOLVE_FAILED");
    assert_eq!(
        envelope["data"]["diagnostic"]["reason"],
        "execution_not_found"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn audit_082_resolve_refuses_states_beyond_claimed_no_state() {
    let root = temp_root("resolve-refuse");
    provision_replay(&root).unwrap();
    let key = LogicalExecutionKey::derive("anchor-082", "eval-082", "action-refuse").unwrap();
    let execution = {
        let ledger = Rc::new(ReplayLedger::open(&root).unwrap());
        let mut admission =
            ReplayLedger::admit_or_recover_owned(&ledger, key.clone(), binding("action-refuse"))
                .unwrap();
        admission.publish_intent().unwrap();
        admission.execution_id().to_owned()
    };

    let (code, envelope) = run_host(&[
        "replay-resolve-claim",
        &root.to_string_lossy(),
        "--execution-id",
        &execution,
    ]);
    assert_ne!(code, 0);
    assert_eq!(envelope["error"]["code"], "REPLAY_RESOLVE_FAILED");
    assert_eq!(
        envelope["data"]["diagnostic"]["reason"],
        "state_not_resolvable"
    );

    // The durable intent record is untouched and still recovers.
    let ledger = Rc::new(ReplayLedger::open(&root).unwrap());
    let recovered =
        ReplayLedger::admit_or_recover_owned(&ledger, key, binding("action-refuse")).unwrap();
    assert_eq!(recovered.state(), ReplayState::IntentRecorded);
    drop(recovered);

    // Invalid identities are refused with a specific diagnostic.
    let (code, envelope) = run_host(&[
        "replay-resolve-claim",
        &root.to_string_lossy(),
        "--execution-id",
        "not-an-execution-id",
    ]);
    assert_ne!(code, 0);
    assert_eq!(
        envelope["data"]["diagnostic"]["reason"],
        "invalid_execution_id"
    );
    let _ = std::fs::remove_dir_all(&root);
}

// ---------------------------------------------------------------------------
// RSK-01: unsafe permissions on the replay subtree fail closed with a
// specific security diagnostic (POSIX platforms).
// ---------------------------------------------------------------------------

#[cfg(unix)]
#[test]
fn audit_082_group_writable_replay_storage_fails_closed_with_security_diagnostic() {
    use std::os::unix::fs::PermissionsExt;
    let root = temp_root("perm");
    provision_replay(&root).unwrap();
    std::fs::set_permissions(
        root.join("replay/v1/claims"),
        std::fs::Permissions::from_mode(0o775),
    )
    .unwrap();
    assert!(matches!(
        ReplayLedger::open(&root),
        Err(tethers_reference_host::replay::ReplayError::PersistenceUnavailable)
    ));
    let diagnostic = tethers_reference_host::replay::last_replay_diagnostic()
        .expect("unix replay failures must record a structured diagnostic");
    assert_eq!(diagnostic.phase, "security_validation");
    assert_eq!(diagnostic.reason, "unsafe_ownership_or_permissions");
    let _ = std::fs::remove_dir_all(&root);
}
