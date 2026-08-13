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
use tethers_reference_host::operational_scope::OperationalScopeEvidence;
use tethers_reference_host::test_fixture_package;

fn make_scope(installed_id: &str, root: &Path, max_bytes: u64) -> OperationalScopeEvidence {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "query_root": {"type": "string", "x-tethers-path": "canonical-directory"},
            "max_bytes": {"type": "integer", "minimum": 1, "maximum": 67108864}
        },
        "required": ["query_root", "max_bytes"],
        "additionalProperties": false
    });
    let schema_bytes = serde_json_canonicalizer::to_vec(&schema).unwrap();
    use sha2::{Digest, Sha256};
    let schema_digest = format!("sha256:{:x}", Sha256::digest(schema_bytes));
    OperationalScopeEvidence::create(
        installed_id,
        "tethers.fixture",
        "tethers-stdio-fixture",
        &schema_digest,
        &serde_json::json!({"query_root": root.to_string_lossy(), "max_bytes": max_bytes}),
        "Matthew",
    )
    .unwrap()
}
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

fn sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn run_enable(root: &Path, installed_id: &str, scope: &Path) -> (i32, Value) {
    let output = Command::new(host_binary())
        .args(["plug", "enable", "--host-data-root"])
        .arg(root)
        .args(["--installed-id", installed_id])
        .args(["--scope"])
        .arg(scope)
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
    let archive = root.join("fixture.tetherplug");
    let provider = fs::read(env!("CARGO_BIN_EXE_m3_fixture_provider")).unwrap();
    fs::write(
        &archive,
        test_fixture_package::build_fixture_package(&provider).unwrap(),
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
        "tethers-reference-host@0.2.0+j24d",
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
    let scope = make_scope(&installed.installed_id, scope_root, 1024);
    let enablements = EnablementStore::open_existing(&root.join("enablements")).unwrap();
    enablements.enable(installed, scope, "Matthew").unwrap();
}

fn write_scope_file(root: &Path, query_root: &str, max_bytes: u64) -> PathBuf {
    let path = root.join("scope.json");
    let content = serde_json::json!({
        "schema": "tethers.plug-scope/1",
        "scope": {
            "query_root": query_root,
            "max_bytes": max_bytes
        }
    });
    fs::write(&path, serde_json::to_vec(&content).unwrap()).unwrap();
    path
}

#[test]
fn never_enabled_pdf_plug_is_enabled_from_valid_scope_file() {
    let root = std::env::temp_dir().join(format!("tethers-j24d-never-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(root.join("enablements")).unwrap();
    let (installed, scope_root) = install_pdf(&root);
    fs::create_dir_all(&scope_root).unwrap();
    let scope_file = write_scope_file(&root, scope_root.to_str().unwrap(), 1024);

    let before = snapshot(&root);
    let before_json = json_records(&root);
    let (code, envelope) = run_enable(&root, &installed.installed_id, &scope_file);
    assert_eq!(code, 0, "{envelope}");
    assert_eq!(envelope["schema"], "tethers.cli/1");
    assert_eq!(envelope["command"], "plug enable");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["data"]["installed_id"], installed.installed_id);
    assert_eq!(envelope["data"]["package_id"], installed.package_id);
    assert_eq!(envelope["data"]["state"], "enabled");
    assert!(envelope["data"]["sequence"].as_u64().unwrap() >= 1);
    assert!(!envelope["data"]["record_digest"]
        .as_str()
        .unwrap()
        .is_empty());
    assert!(!envelope["data"]["scope_digest"]
        .as_str()
        .unwrap()
        .is_empty());

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
    assert_eq!(new_record.state, EnablementState::Enabled);
    assert_eq!(new_record.authority, "tethers-reference-host-cli");
    assert!(new_record.sequence >= 1);
    assert!(!new_record.operational_scope_digest.is_empty());

    let (_code, list_envelope) = run_list(&root);
    assert_eq!(
        list_envelope["data"]["plugs"][0]["installed_id"],
        installed.installed_id
    );
    assert_eq!(list_envelope["data"]["plugs"][0]["state"], "enabled");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn disabled_pdf_plug_is_re_enabled_with_correct_predecessor_linkage() {
    let root = std::env::temp_dir().join(format!("tethers-j24d-reenable-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(root.join("enablements")).unwrap();
    let (installed, scope_root) = install_pdf(&root);
    enable_pdf(&root, &installed, &scope_root);
    run_disable(&root, &installed.installed_id);
    let scope_file = write_scope_file(&root, scope_root.to_str().unwrap(), 1048576);

    let before_json = json_records(&root);
    let (code, envelope) = run_enable(&root, &installed.installed_id, &scope_file);
    assert_eq!(code, 0, "{envelope}");
    assert_eq!(envelope["data"]["state"], "enabled");
    let sequence = envelope["data"]["sequence"].as_u64().unwrap();
    assert!(
        sequence >= 3,
        "sequence should be at least 3 after enable→disable→enable"
    );

    let after_json = json_records(&root);
    assert_eq!(after_json.len(), before_json.len() + 1);
    let new_record: EnablementRecord = serde_json::from_str(
        &after_json
            .iter()
            .find(|j| !before_json.contains(j))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(new_record.state, EnablementState::Enabled);
    assert_eq!(new_record.authority, "tethers-reference-host-cli");
    assert!(
        new_record.previous_record_digest.is_some(),
        "predecessor must be linked"
    );
    assert_eq!(new_record.sequence, sequence);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn success_envelope_exposes_only_authorised_fields() {
    let root = std::env::temp_dir().join(format!("tethers-j24d-fields-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(root.join("enablements")).unwrap();
    let (installed, scope_root) = install_pdf(&root);
    fs::create_dir_all(&scope_root).unwrap();
    let scope_file = write_scope_file(&root, scope_root.to_str().unwrap(), 2048);

    let (code, envelope) = run_enable(&root, &installed.installed_id, &scope_file);
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
            "scope_digest",
            "sequence",
            "state"
        ]
    );
    assert_eq!(envelope["data"]["state"], "enabled");
    assert_eq!(envelope["data"]["installed_id"], installed.installed_id);
    assert_eq!(envelope["data"]["package_id"], installed.package_id);

    for forbidden in [
        "query_root",
        "max_bytes",
        "authority",
        "operational_scope",
        "trust_evidence",
        "installation_approval_id",
        "conformance_evidence_digest",
        "provider_version",
        "predecessor",
        "changed_unix_ms",
        "capabilities",
        "scope_file",
    ] {
        assert!(
            envelope.to_string().find(forbidden).is_none(),
            "exposed {forbidden}"
        );
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn already_enabled_plug_fails_without_mutation() {
    let root = std::env::temp_dir().join(format!("tethers-j24d-already-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(root.join("enablements")).unwrap();
    let (installed, scope_root) = install_pdf(&root);
    enable_pdf(&root, &installed, &scope_root);
    let scope_file = write_scope_file(&root, scope_root.to_str().unwrap(), 1024);

    let before = snapshot(&root);
    let before_json = json_records(&root);
    let (code, envelope) = run_enable(&root, &installed.installed_id, &scope_file);
    assert_eq!(code, 3, "{envelope}");
    assert_eq!(envelope["status"], "invalid_data");
    assert_eq!(envelope["error"]["code"], "enablement_conflict");
    assert_read_only(&root, &before, &snapshot(&root));
    let after_json = json_records(&root);
    assert_eq!(after_json.len(), before_json.len());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn unknown_installed_id_fails_without_mutation() {
    let root = std::env::temp_dir().join(format!("tethers-j24d-unknown-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(root.join("enablements")).unwrap();
    let (_installed, scope_root) = install_pdf(&root);
    fs::create_dir_all(&scope_root).unwrap();
    let scope_file = write_scope_file(&root, scope_root.to_str().unwrap(), 1024);

    let unknown_id = Uuid::new_v4().to_string();
    let before = snapshot(&root);
    let (code, envelope) = run_enable(&root, &unknown_id, &scope_file);
    assert_eq!(code, 3, "{envelope}");
    assert_eq!(envelope["status"], "invalid_data");
    assert_eq!(envelope["error"]["code"], "installed_not_found");
    assert_read_only(&root, &before, &snapshot(&root));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn malformed_scope_file_fails_with_invalid_data() {
    let root = std::env::temp_dir().join(format!("tethers-j24d-malformed-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(root.join("enablements")).unwrap();
    let (installed, scope_root) = install_pdf(&root);
    fs::create_dir_all(&scope_root).unwrap();

    let bad_scope = root.join("bad.json");
    fs::write(&bad_scope, b"not json").unwrap();
    let before = snapshot(&root);
    let (code, envelope) = run_enable(&root, &installed.installed_id, &bad_scope);
    assert_eq!(code, 3, "{envelope}");
    assert_eq!(envelope["status"], "invalid_data");
    assert_eq!(envelope["error"]["code"], "scope_request_invalid");
    assert_read_only(&root, &before, &snapshot(&root));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn oversized_scope_file_fails_with_invalid_data() {
    let root = std::env::temp_dir().join(format!("tethers-j24d-oversized-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(root.join("enablements")).unwrap();
    let (installed, _scope_root) = install_pdf(&root);

    let big_scope = root.join("big.json");
    let mut content = String::from("{\"padding\":\"");
    content.push_str(&"x".repeat(16 * 1024 + 1));
    content.push_str("\"}");
    fs::write(&big_scope, content.as_bytes()).unwrap();

    let before = snapshot(&root);
    let (code, envelope) = run_enable(&root, &installed.installed_id, &big_scope);
    assert_eq!(code, 3, "{envelope}");
    assert_eq!(envelope["status"], "invalid_data");
    assert_eq!(envelope["error"]["code"], "scope_request_invalid");
    assert_read_only(&root, &before, &snapshot(&root));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn duplicate_json_keys_in_scope_file_fail() {
    let root = std::env::temp_dir().join(format!("tethers-j24d-duplicate-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(root.join("enablements")).unwrap();
    let (installed, scope_root) = install_pdf(&root);
    fs::create_dir_all(&scope_root).unwrap();

    let scope_file = root.join("duplicate.json");
    let content = format!(
        r#"{{"schema":"tethers.plug-scope/1","capability":{{"name":"fixture.ping","version":1,"version":1}},"permissions":{{"query_root":"{}","max_bytes":{} }}}}"#,
        scope_root.display(),
        1024
    );
    fs::write(&scope_file, content.as_bytes()).unwrap();

    let before = snapshot(&root);
    let (code, envelope) = run_enable(&root, &installed.installed_id, &scope_file);
    assert_eq!(code, 3, "{envelope}");
    assert_eq!(envelope["status"], "invalid_data");
    assert_eq!(envelope["error"]["code"], "scope_request_invalid");
    assert_read_only(&root, &before, &snapshot(&root));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn unknown_fields_in_scope_file_fail() {
    let root = std::env::temp_dir().join(format!("tethers-j24d-unknown-field-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(root.join("enablements")).unwrap();
    let (installed, scope_root) = install_pdf(&root);
    fs::create_dir_all(&scope_root).unwrap();

    let scope_file = root.join("extra.json");
    let content = format!(
        r#"{{"schema":"tethers.plug-scope/1","capability":{{"name":"fixture.ping","version":1}},"permissions":{{"query_root":"{}","max_bytes":1024}},"extra":"bad"}}"#,
        scope_root.display()
    );
    fs::write(&scope_file, content.as_bytes()).unwrap();

    let before = snapshot(&root);
    let (code, envelope) = run_enable(&root, &installed.installed_id, &scope_file);
    assert_eq!(code, 3, "{envelope}");
    assert_eq!(envelope["status"], "invalid_data");
    assert_eq!(envelope["error"]["code"], "scope_request_invalid");
    assert_read_only(&root, &before, &snapshot(&root));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn missing_scope_file_fails_with_unavailable() {
    let root = std::env::temp_dir().join(format!("tethers-j24d-missing-scope-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(root.join("enablements")).unwrap();
    let (installed, _scope_root) = install_pdf(&root);

    let missing = root.join("missing.json");
    let before = snapshot(&root);
    let (code, envelope) = run_enable(&root, &installed.installed_id, &missing);
    assert_eq!(code, 4, "{envelope}");
    assert_eq!(envelope["status"], "unavailable");
    assert_read_only(&root, &before, &snapshot(&root));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn missing_host_root_fails_with_unavailable() {
    let missing =
        std::env::temp_dir().join(format!("tethers-j24d-missing-root-{}", Uuid::new_v4()));
    let (code, envelope) = run_enable(
        &missing,
        "00000000-0000-4000-8000-000000000000",
        &Path::new("C:\\missing.json"),
    );
    assert_eq!(code, 4);
    assert_eq!(envelope["status"], "unavailable");
    assert_eq!(envelope["error"]["code"], "plug_data_root_unavailable");
    assert!(!missing.exists());
}

#[test]
fn partial_lifecycle_layout_fails_without_mutation() {
    let root = std::env::temp_dir().join(format!("tethers-j24d-partial-{}", Uuid::new_v4()));
    fs::create_dir_all(root.join("install")).unwrap();
    let before = snapshot(&root);
    let (code, envelope) = run_enable(
        &root,
        "00000000-0000-4000-8000-000000000000",
        &Path::new("C:\\scope.json"),
    );
    assert_eq!(code, 3);
    assert_eq!(envelope["status"], "invalid_data");
    assert_eq!(envelope["error"]["code"], "plug_store_incomplete");
    assert_read_only(&root, &before, &snapshot(&root));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn invalid_cli_usage_fails_with_exit_2() {
    let root = std::env::temp_dir().join(format!("tethers-j24d-cli-{}", Uuid::new_v4()));

    let output = Command::new(host_binary())
        .args([
            "plug",
            "enable",
            "--host-data-root",
            "relative/path",
            "--installed-id",
            "00000000-0000-4000-8000-000000000000",
            "--scope",
            "C:\\scope.json",
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
            "enable",
            "--host-data-root",
            "C:\\root",
            "--installed-id",
            "not-a-uuid",
            "--scope",
            "C:\\scope.json",
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
            "enable",
            "--host-data-root",
            "C:\\root",
            "--installed-id",
            "00000000-0000-4000-8000-000000000000",
            "--scope",
            "relative/path",
        ])
        .output()
        .unwrap();
    let code = output.status.code().unwrap();
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(code, 2);
    assert_eq!(envelope["status"], "invalid_cli_usage");

    assert!(!root.exists());
}

#[test]
fn missing_query_root_in_scope_file_fails() {
    let root = std::env::temp_dir().join(format!("tethers-j24d-no-query-root-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(root.join("enablements")).unwrap();
    let (installed, _scope_root) = install_pdf(&root);

    let scope_file = root.join("no_query.json");
    let content = r#"{"schema":"tethers.plug-scope/1","capability":{"name":"fixture.ping","version":1},"permissions":{"max_bytes":1024}}"#;
    fs::write(&scope_file, content.as_bytes()).unwrap();

    let before = snapshot(&root);
    let (code, envelope) = run_enable(&root, &installed.installed_id, &scope_file);
    assert_eq!(code, 3, "{envelope}");
    assert_eq!(envelope["status"], "invalid_data");
    assert_eq!(envelope["error"]["code"], "scope_request_invalid");
    assert_read_only(&root, &before, &snapshot(&root));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn wrong_schema_in_scope_file_fails() {
    let root = std::env::temp_dir().join(format!("tethers-j24d-wrong-schema-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(root.join("enablements")).unwrap();
    let (installed, scope_root) = install_pdf(&root);
    fs::create_dir_all(&scope_root).unwrap();

    let scope_file = root.join("wrong.json");
    let content = format!(
        r#"{{"schema":"tethers.plug-scope/2","capability":{{"name":"fixture.ping","version":1}},"permissions":{{"query_root":"{}","max_bytes":1024}}}}"#,
        scope_root.display()
    );
    fs::write(&scope_file, content.as_bytes()).unwrap();

    let before = snapshot(&root);
    let (code, envelope) = run_enable(&root, &installed.installed_id, &scope_file);
    assert_eq!(code, 3, "{envelope}");
    assert_eq!(envelope["status"], "invalid_data");
    assert_eq!(envelope["error"]["code"], "scope_request_invalid");
    assert_read_only(&root, &before, &snapshot(&root));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn wrong_capability_in_scope_file_fails() {
    let root = std::env::temp_dir().join(format!("tethers-j24d-wrong-cap-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(root.join("enablements")).unwrap();
    let (installed, scope_root) = install_pdf(&root);
    fs::create_dir_all(&scope_root).unwrap();

    let scope_file = root.join("wrong_cap.json");
    let content = format!(
        r#"{{"schema":"tethers.plug-scope/1","capability":{{"name":"file.move","version":1}},"permissions":{{"query_root":"{}","max_bytes":1024}}}}"#,
        scope_root.display()
    );
    fs::write(&scope_file, content.as_bytes()).unwrap();

    let before = snapshot(&root);
    let (code, envelope) = run_enable(&root, &installed.installed_id, &scope_file);
    assert_eq!(code, 3, "{envelope}");
    assert_eq!(envelope["status"], "invalid_data");
    assert_eq!(envelope["error"]["code"], "scope_request_invalid");
    assert_read_only(&root, &before, &snapshot(&root));

    let _ = fs::remove_dir_all(root);
}
