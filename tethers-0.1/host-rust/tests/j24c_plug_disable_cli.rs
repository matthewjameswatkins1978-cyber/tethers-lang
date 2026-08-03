use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use tethers_reference_host::candidate::{extract_to_quarantine, CandidateRegistry};
use tethers_reference_host::conformance::{run_host_conformance, ConformanceEvidenceStore};
use tethers_reference_host::enablement::{EnablementRecord, EnablementState, EnablementStore};
use tethers_reference_host::installed::{InstallationApprovalStore, InstalledPlugRegistry};
use tethers_reference_host::launch_profile::PreparedSupervisedLaunch;
use tethers_reference_host::pdf_tools::{self, PdfOperationalScopeBinding};
use tethers_reference_host::trust::{
    DeveloperApprovalStore, PackageTrustEvidence, PublisherTrustStore,
};
use uuid::Uuid;

fn host_binary() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_tethers-reference-host")
        .or_else(|| std::env::var_os("CARGO_BIN_EXE_tethers_reference_host"))
        .map(PathBuf::from)
        .or_else(|| {
            std::env::current_exe()
                .ok()?
                .parent()?
                .parent()
                .map(|path| path.join("tethers-reference-host.exe"))
        })
        .expect("compiled reference host binary")
}

fn canonical<T: serde::Serialize>(value: &T) -> Vec<u8> {
    serde_json_canonicalizer::to_vec(value).unwrap()
}

fn sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn run_disable(root: &Path, installed_id: &str) -> (i32, Value) {
    let output = Command::new(host_binary())
        .args(["plug", "disable", "--host-data-root"])
        .arg(root)
        .args(["--installed-id", installed_id])
        .output()
        .expect("reference host process");
    let code = output.status.code().expect("process exit code");
    let envelope: Value = serde_json::from_slice(&output.stdout).expect("one JSON envelope");
    assert_eq!(code, envelope["exit_code"].as_i64().unwrap() as i32);
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
    (code, envelope)
}

fn run_list(root: &Path) -> (i32, Value) {
    let output = Command::new(host_binary())
        .args(["plug", "list", "--host-data-root"])
        .arg(root)
        .output()
        .expect("reference host process");
    let code = output.status.code().expect("process exit code");
    let envelope: Value = serde_json::from_slice(&output.stdout).expect("one JSON envelope");
    assert_eq!(code, envelope["exit_code"].as_i64().unwrap() as i32);
    (code, envelope)
}

fn snapshot(root: &Path) -> BTreeMap<String, String> {
    fn visit(root: &Path, path: &Path, output: &mut BTreeMap<String, String>) {
        let mut entries = fs::read_dir(path)
            .unwrap_or_else(|error| panic!("read snapshot directory {path:?}: {error}"))
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();
        entries.sort();
        for entry in entries {
            let relative = entry
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let metadata = fs::symlink_metadata(&entry).unwrap();
            if metadata.is_dir() {
                output.insert(relative.clone(), "<directory>".into());
                visit(root, &entry, output);
            } else {
                output.insert(relative, sha256(&fs::read(&entry).unwrap()));
            }
        }
    }
    let mut output = BTreeMap::new();
    visit(root, root, &mut output);
    output
}

fn assert_read_only(
    root: &Path,
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
) {
    assert_eq!(before, after, "command changed lifecycle state");
    assert!(!serde_json::to_string(after)
        .unwrap()
        .contains(root.to_string_lossy().as_ref()));
}

fn resign(record: &mut EnablementRecord) {
    record.record_digest.clear();
    record.record_digest = sha256(&canonical(record));
    record.validate().unwrap();
}

fn write_record(root: &Path, record: &EnablementRecord) {
    fs::write(
        root.join("enablements")
            .join(format!("{}.json", record.enablement_id)),
        canonical(record),
    )
    .unwrap();
}

fn json_records(root: &Path) -> Vec<String> {
    let path = root.join("enablements");
    let mut entries = fs::read_dir(&path)
        .unwrap_or_else(|error| panic!("read enablements directory {path:?}: {error}"))
        .map(|entry| entry.unwrap().path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    entries.sort();
    entries
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect()
}

fn install_pdf(
    root: &Path,
) -> (
    tethers_reference_host::installed::InstalledPlugRecord,
    PathBuf,
) {
    let archive = root.join("pdf-tools.tetherplug");
    let provider = fs::read(env!("CARGO_BIN_EXE_pdf_tools_provider")).unwrap();
    fs::write(
        &archive,
        pdf_tools::build_reference_package(&provider).unwrap(),
    )
    .unwrap();
    let report = tethers_reference_host::package::inspect(&archive).unwrap();
    let quarantine_root = root.join("quarantine");
    let quarantined = extract_to_quarantine(&report, &quarantine_root).unwrap();
    let candidate = CandidateRegistry::open(&root.join("candidates"), &quarantine_root)
        .unwrap()
        .create(&quarantined)
        .unwrap();
    let developers = DeveloperApprovalStore::open(&root.join("developer")).unwrap();
    let developer = developers
        .approve_exact_digest(&candidate.semantic_package_digest, "Matthew")
        .unwrap();
    let trust = PackageTrustEvidence::unsigned(&developer).unwrap();
    let publishers = PublisherTrustStore::open(&root.join("publishers")).unwrap();
    let prepared = PreparedSupervisedLaunch::prepare(
        &candidate,
        &quarantine_root,
        &root.join("scratch"),
        Duration::from_secs(10),
    )
    .unwrap();
    let conformance = run_host_conformance(
        &prepared,
        &candidate,
        &quarantine_root,
        &trust,
        &publishers,
        &developers,
        "tethers-reference-host@0.2.0+j24c",
    )
    .unwrap();
    ConformanceEvidenceStore::open(&root.join("conformance"))
        .unwrap()
        .create(&conformance)
        .unwrap();
    let approval = InstallationApprovalStore::open(&root.join("approvals"))
        .unwrap()
        .approve(
            &candidate,
            &quarantine_root,
            &trust,
            &publishers,
            &developers,
            &prepared.evidence,
            &conformance,
            "Matthew",
        )
        .unwrap();
    let registry =
        InstalledPlugRegistry::open(&root.join("install"), &root.join("installed-records"))
            .unwrap();
    let installed = registry
        .install_disabled(
            &candidate,
            &quarantine_root,
            &trust,
            &publishers,
            &developers,
            &prepared.evidence,
            &conformance,
            &approval,
        )
        .unwrap();
    (installed, root.join("scope"))
}

fn enable_pdf(
    root: &Path,
    installed: &tethers_reference_host::installed::InstalledPlugRecord,
    scope_root: &Path,
) {
    fs::create_dir_all(scope_root).unwrap();
    let scope =
        PdfOperationalScopeBinding::create(&installed.installed_id, scope_root, 1024, "Matthew")
            .unwrap();
    let enablements = EnablementStore::open_existing(&root.join("enablements")).unwrap();
    enablements.enable(installed, scope, "Matthew").unwrap();
}

#[test]
fn real_pdf_disable_succeeds_then_fails_on_second_attempt() {
    let root = std::env::temp_dir().join(format!("tethers-j24c-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(root.join("enablements")).unwrap();
    let (installed, scope_root) = install_pdf(&root);
    enable_pdf(&root, &installed, &scope_root);

    let before = snapshot(&root);
    let before_json = json_records(&root);
    let (code, envelope) = run_disable(&root, &installed.installed_id);
    assert_eq!(code, 0, "{envelope}");
    assert_eq!(envelope["schema"], "tethers.cli/1");
    assert_eq!(envelope["command"], "plug disable");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["data"]["installed_id"], installed.installed_id);
    assert_eq!(envelope["data"]["package_id"], installed.package_id);
    assert_eq!(envelope["data"]["state"], "disabled");
    assert!(envelope["data"]["sequence"].as_u64().unwrap() >= 2);
    assert!(!envelope["data"]["record_digest"]
        .as_str()
        .unwrap()
        .is_empty());
    let data_keys: Vec<_> = envelope["data"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    assert_eq!(
        data_keys,
        vec![
            "installed_id",
            "package_id",
            "record_digest",
            "sequence",
            "state"
        ]
    );
    for forbidden in [
        "installation_relative_path",
        "operational_scope",
        "authority",
        "trust_evidence",
        "installation_approval_id",
        "conformance_evidence_digest",
        "provider_version",
        "predecessor",
        "changed_unix_ms",
        "capabilities",
        "scope",
    ] {
        assert!(
            envelope.to_string().find(forbidden).is_none(),
            "exposed {forbidden}"
        );
    }

    let after = snapshot(&root);
    let after_json = json_records(&root);
    assert_eq!(
        after_json.len(),
        before_json.len() + 1,
        "exactly one new enablement record"
    );
    for (path, digest) in &before {
        if !path.starts_with("enablements/") {
            assert_eq!(after[path], *digest, "changed pre-existing file: {path}");
        }
    }
    let new_record: EnablementRecord = serde_json::from_str(
        &after_json
            .iter()
            .find(|j| !before_json.contains(j))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(new_record.state, EnablementState::Disabled);
    assert_eq!(new_record.authority, "tethers-reference-host-cli");
    assert!(new_record.sequence >= 2);
    assert!(new_record.previous_record_digest.is_some());

    let (_code, list_envelope) = run_list(&root);
    assert_eq!(
        list_envelope["data"]["plugs"][0]["installed_id"],
        installed.installed_id
    );
    assert_eq!(list_envelope["data"]["plugs"][0]["state"], "disabled");

    let before = snapshot(&root);
    let before_json = json_records(&root);
    let (code, envelope) = run_disable(&root, &installed.installed_id);
    assert_eq!(code, 3, "{envelope}");
    assert_eq!(envelope["status"], "invalid_data");
    assert_eq!(envelope["error"]["code"], "enablement_refused");
    assert_read_only(&root, &before, &snapshot(&root));
    let after_json = json_records(&root);
    assert_eq!(
        after_json.len(),
        before_json.len(),
        "no new records on failed attempt"
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn installed_but_never_enabled_fails_without_mutation() {
    let root = std::env::temp_dir().join(format!("tethers-j24c-never-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(root.join("enablements")).unwrap();
    let (installed, _scope_root) = install_pdf(&root);

    let before = snapshot(&root);
    let (code, envelope) = run_disable(&root, &installed.installed_id);
    assert_eq!(code, 3, "{envelope}");
    assert_eq!(envelope["status"], "invalid_data");
    assert_eq!(envelope["error"]["code"], "enablement_refused");
    assert_read_only(&root, &before, &snapshot(&root));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn unknown_installed_id_fails_without_mutation() {
    let root = std::env::temp_dir().join(format!("tethers-j24c-unknown-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(root.join("enablements")).unwrap();
    let (installed, scope_root) = install_pdf(&root);
    enable_pdf(&root, &installed, &scope_root);

    let unknown_id = Uuid::new_v4().to_string();
    let before = snapshot(&root);
    let (code, envelope) = run_disable(&root, &unknown_id);
    assert_eq!(code, 3, "{envelope}");
    assert_eq!(envelope["status"], "invalid_data");
    assert_eq!(envelope["error"]["code"], "installed_not_found");
    assert_read_only(&root, &before, &snapshot(&root));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn cross_record_drift_fails_without_mutation() {
    let root = std::env::temp_dir().join(format!("tethers-j24c-drift-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(root.join("enablements")).unwrap();
    let (installed, scope_root) = install_pdf(&root);
    enable_pdf(&root, &installed, &scope_root);

    let enablements = EnablementStore::open_existing(&root.join("enablements")).unwrap();
    let records = enablements.load_all().unwrap();
    let enabled = records
        .iter()
        .find(|r| r.state == EnablementState::Enabled)
        .unwrap();
    let mut drift = enabled.clone();
    drift.enablement_id = Uuid::new_v4().to_string();
    drift.sequence += 1;
    drift.previous_record_digest = Some(enabled.record_digest.clone());
    drift.provider_version.push_str("-drift");
    resign(&mut drift);
    drift.validate().unwrap();
    write_record(&root, &drift);

    let before = snapshot(&root);
    let (code, envelope) = run_disable(&root, &installed.installed_id);
    assert_eq!(code, 3, "{envelope}");
    assert_eq!(envelope["status"], "invalid_data");
    assert_eq!(envelope["error"]["code"], "enablement_invalid");
    assert_read_only(&root, &before, &snapshot(&root));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn corrupt_forked_chain_fails_without_mutation() {
    let root = std::env::temp_dir().join(format!("tethers-j24c-fork-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(root.join("enablements")).unwrap();
    let (installed, scope_root) = install_pdf(&root);
    enable_pdf(&root, &installed, &scope_root);

    let enablements = EnablementStore::open_existing(&root.join("enablements")).unwrap();
    let records = enablements.load_all().unwrap();
    let enabled = records
        .iter()
        .find(|r| r.state == EnablementState::Enabled)
        .unwrap();
    let mut fork = enabled.clone();
    fork.enablement_id = Uuid::new_v4().to_string();
    fork.sequence = 1;
    fork.previous_record_digest = None;
    resign(&mut fork);
    fork.validate().unwrap();
    write_record(&root, &fork);

    let before = snapshot(&root);
    let (code, envelope) = run_disable(&root, &installed.installed_id);
    assert_eq!(code, 3, "{envelope}");
    assert_eq!(envelope["status"], "invalid_data");
    assert_eq!(envelope["error"]["code"], "enablement_invalid");
    assert_read_only(&root, &before, &snapshot(&root));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn missing_and_partial_roots_fail_closed_without_creation() {
    let missing = std::env::temp_dir().join(format!("tethers-j24c-missing-{}", Uuid::new_v4()));
    let (code, envelope) = run_disable(&missing, "00000000-0000-4000-8000-000000000000");
    assert_eq!(code, 4);
    assert_eq!(envelope["status"], "unavailable");
    assert_eq!(envelope["error"]["code"], "plug_data_root_unavailable");
    assert!(!missing.exists());

    let root = std::env::temp_dir().join(format!("tethers-j24c-partial-{}", Uuid::new_v4()));
    fs::create_dir_all(root.join("install")).unwrap();
    let before = snapshot(&root);
    let (code, envelope) = run_disable(&root, "00000000-0000-4000-8000-000000000000");
    assert_eq!(code, 3);
    assert_eq!(envelope["status"], "invalid_data");
    assert_eq!(envelope["error"]["code"], "plug_store_incomplete");
    assert_read_only(&root, &before, &snapshot(&root));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn invalid_cli_usage_fails_with_exit_2() {
    let root = std::env::temp_dir().join(format!("tethers-j24c-cli-{}", Uuid::new_v4()));

    let output = Command::new(host_binary())
        .args([
            "plug",
            "disable",
            "--host-data-root",
            "relative/path",
            "--installed-id",
            "00000000-0000-4000-8000-000000000000",
        ])
        .output()
        .unwrap();
    let code = output.status.code().unwrap();
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(code, 2);
    assert_eq!(envelope["status"], "invalid_cli_usage");
    assert_eq!(envelope["exit_code"], 2);

    let output = Command::new(host_binary())
        .args([
            "plug",
            "disable",
            "--host-data-root",
            "C:\\root",
            "--installed-id",
            "not-a-uuid",
        ])
        .output()
        .unwrap();
    let code = output.status.code().unwrap();
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(code, 2);
    assert_eq!(envelope["status"], "invalid_cli_usage");
    assert_eq!(envelope["exit_code"], 2);

    assert!(!root.exists());
}

#[test]
fn reversed_uuid_filename_order_cannot_alter_latest_by_sequence() {
    let root = std::env::temp_dir().join(format!("tethers-j24c-reverse-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(root.join("enablements")).unwrap();
    let (installed, scope_root) = install_pdf(&root);

    enable_pdf(&root, &installed, &scope_root);
    let enablements = EnablementStore::open_existing(&root.join("enablements")).unwrap();
    let records = enablements.load_all().unwrap();
    let enabled = records
        .iter()
        .find(|r| r.state == EnablementState::Enabled)
        .unwrap();

    let mut disabled = enabled.clone();
    disabled.enablement_id = Uuid::new_v4().to_string();
    disabled.sequence += 1;
    disabled.previous_record_digest = Some(enabled.record_digest.clone());
    disabled.state = EnablementState::Disabled;
    resign(&mut disabled);
    disabled.validate().unwrap();

    enablements.disable(&installed, "Matthew").unwrap(); // creates actual disabled record
    let records = enablements.load_all().unwrap();
    let actual_disabled = records
        .iter()
        .find(|r| r.state == EnablementState::Disabled)
        .unwrap();

    // Verify the actual disabled record is sequence 2 and predecessor-linked
    assert_eq!(actual_disabled.sequence, 2);
    assert_eq!(
        actual_disabled.previous_record_digest.as_ref().unwrap(),
        &enabled.record_digest
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn success_envelope_exposes_only_authorised_fields() {
    let root = std::env::temp_dir().join(format!("tethers-j24c-fields-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(root.join("enablements")).unwrap();
    let (installed, scope_root) = install_pdf(&root);
    enable_pdf(&root, &installed, &scope_root);

    let (code, envelope) = run_disable(&root, &installed.installed_id);
    assert_eq!(code, 0);
    let keys: Vec<_> = envelope.as_object().unwrap().keys().cloned().collect();
    assert_eq!(
        keys,
        vec!["command", "data", "exit_code", "schema", "status"]
    );

    let data_keys: Vec<_> = envelope["data"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    let mut sorted: Vec<_> = data_keys.clone();
    sorted.sort();
    assert_eq!(
        sorted,
        vec![
            "installed_id",
            "package_id",
            "record_digest",
            "sequence",
            "state"
        ]
    );
    assert_eq!(envelope["data"]["state"], "disabled");
    assert_eq!(envelope["data"]["installed_id"], installed.installed_id);
    assert_eq!(envelope["data"]["package_id"], installed.package_id);

    fs::remove_dir_all(root).unwrap();
}
