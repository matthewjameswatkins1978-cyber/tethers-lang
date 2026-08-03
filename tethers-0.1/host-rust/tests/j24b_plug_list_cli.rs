use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use tethers_reference_host::candidate::{extract_to_quarantine, CandidateRegistry};
use tethers_reference_host::conformance::{run_host_conformance, ConformanceEvidenceStore};
use tethers_reference_host::enablement::{EnablementRecord, EnablementState, EnablementStore};
use tethers_reference_host::installed::{
    DisabledBindingRecord, InstallationApprovalStore, InstalledPlugRegistry,
};
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

fn run(root: &Path) -> (i32, Value) {
    let output = Command::new(host_binary())
        .args(["plug", "list", "--host-data-root"])
        .arg(root)
        .output()
        .expect("reference host process");
    let code = output.status.code().expect("process exit code");
    let envelope: Value = serde_json::from_slice(&output.stdout).expect("one JSON envelope");
    assert_eq!(code, envelope["exit_code"].as_i64().unwrap() as i32);
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
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
    assert_eq!(before, after, "plug list changed lifecycle state");
    assert!(!serde_json::to_string(after)
        .unwrap()
        .contains(root.to_string_lossy().as_ref()));
}

fn assert_plug_shape(envelope: &Value, installed_id: &str, state: &str) {
    let plug = &envelope["data"]["plugs"][0];
    let keys = plug
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        keys,
        vec![
            "capabilities",
            "created_unix_ms",
            "installed_id",
            "package_id",
            "package_version",
            "provider_id",
            "provider_version",
            "semantic_package_digest",
            "state"
        ]
    );
    assert_eq!(plug["installed_id"], installed_id);
    assert_eq!(plug["package_id"], "tethers.pdf-tools");
    assert_eq!(plug["package_version"], "1.0.0");
    assert_eq!(plug["provider_id"], "tethers-pdf-provider");
    assert_eq!(plug["state"], state);
    let capabilities = plug["capabilities"].as_array().unwrap();
    assert_eq!(capabilities.len(), 1);
    assert_eq!(capabilities[0]["name"], "pdf.inspect");
    assert_eq!(capabilities[0]["version"], 1);
    assert_eq!(capabilities[0]["provider_operation_name"], "pdf_inspect");
    assert_eq!(capabilities[0].as_object().unwrap().len(), 4);
    for forbidden in [
        "installation_relative_path",
        "operational_scope",
        "authority",
        "trust_evidence",
        "installation_approval_id",
        "conformance_evidence_digest",
        "transition_history",
        "store_path",
    ] {
        assert!(
            envelope.to_string().find(forbidden).is_none(),
            "exposed {forbidden}"
        );
    }
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
        "tethers-reference-host@0.2.0+j24b",
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

#[test]
fn real_pdf_lifecycle_list_is_deterministic_read_only_and_fail_closed() {
    let root = std::env::temp_dir().join(format!("tethers-j24b-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(root.join("enablements")).unwrap();
    let (installed, scope_root) = install_pdf(&root);

    let before = snapshot(&root);
    let (code, envelope) = run(&root);
    assert_eq!(code, 0, "{envelope}");
    assert_eq!(envelope["schema"], "tethers.cli/1");
    assert_eq!(envelope["command"], "plug list");
    assert_eq!(envelope["status"], "ok");
    assert_plug_shape(&envelope, &installed.installed_id, "disabled");
    assert_read_only(&root, &before, &snapshot(&root));

    fs::create_dir_all(&scope_root).unwrap();
    let scope =
        PdfOperationalScopeBinding::create(&installed.installed_id, &scope_root, 1024, "Matthew")
            .unwrap();
    let enablements = EnablementStore::open(&root.join("enablements")).unwrap();
    let enabled = enablements.enable(&installed, scope, "Matthew").unwrap();
    let before = snapshot(&root);
    let (code, envelope) = run(&root);
    assert_eq!(code, 0);
    assert_plug_shape(&envelope, &installed.installed_id, "enabled");
    assert_read_only(&root, &before, &snapshot(&root));

    let disabled = enablements.disable(&installed, "Matthew").unwrap();
    let enabled_path = root
        .join("enablements")
        .join(format!("{}.json", enabled.enablement_id));
    let disabled_path = root
        .join("enablements")
        .join(format!("{}.json", disabled.enablement_id));
    let mut enabled_fixture = enabled.clone();
    enabled_fixture.enablement_id = "ffffffff-ffff-4fff-8fff-ffffffffffff".into();
    resign(&mut enabled_fixture);
    let mut disabled_fixture = disabled.clone();
    disabled_fixture.enablement_id = "00000000-0000-4000-8000-000000000000".into();
    disabled_fixture.previous_record_digest = Some(enabled_fixture.record_digest.clone());
    resign(&mut disabled_fixture);
    fs::remove_file(enabled_path).unwrap();
    fs::remove_file(disabled_path).unwrap();
    write_record(&root, &enabled_fixture);
    write_record(&root, &disabled_fixture);
    let before = snapshot(&root);
    let (code, envelope) = run(&root);
    assert_eq!(code, 0);
    assert_plug_shape(&envelope, &installed.installed_id, "disabled");
    assert_read_only(&root, &before, &snapshot(&root));

    let unknown_id = Uuid::new_v4().to_string();
    let unknown_scope_root = root.join("unknown-scope");
    fs::create_dir_all(&unknown_scope_root).unwrap();
    let mut unknown = disabled_fixture.clone();
    unknown.enablement_id = "11111111-1111-4111-8111-111111111111".into();
    unknown.installed_id = unknown_id.clone();
    unknown.sequence = 1;
    unknown.previous_record_digest = None;
    unknown.state = EnablementState::Disabled;
    let unknown_scope =
        PdfOperationalScopeBinding::create(&unknown_id, &unknown_scope_root, 1024, "Matthew")
            .unwrap();
    unknown.operational_scope_digest = unknown_scope.integrity_digest.clone();
    unknown.operational_scope = unknown_scope.into();
    resign(&mut unknown);
    write_record(&root, &unknown);
    let before = snapshot(&root);
    let (code, envelope) = run(&root);
    assert_eq!(code, 3);
    assert_eq!(envelope["status"], "invalid_data");
    assert_eq!(envelope["error"]["code"], "enablement_invalid");
    assert_read_only(&root, &before, &snapshot(&root));
    fs::remove_file(
        root.join("enablements")
            .join(format!("{}.json", unknown.enablement_id)),
    )
    .unwrap();

    let mut mismatch = disabled_fixture;
    mismatch.enablement_id = "22222222-2222-4222-8222-222222222222".into();
    mismatch.provider_version.push_str("-mismatch");
    resign(&mut mismatch);
    write_record(&root, &mismatch);
    let before = snapshot(&root);
    let (code, envelope) = run(&root);
    assert_eq!(code, 3);
    assert_eq!(envelope["status"], "invalid_data");
    assert_eq!(envelope["error"]["code"], "enablement_invalid");
    assert_read_only(&root, &before, &snapshot(&root));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn compiled_list_orders_plugs_and_capabilities_stably() {
    let root = std::env::temp_dir().join(format!("tethers-j24b-order-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(root.join("enablements")).unwrap();
    let (installed, _) = install_pdf(&root);
    let mut second = installed.clone();
    second.installed_id = "ffffffff-ffff-4fff-8fff-ffffffffffff".into();
    second.package_id = "tethers.z-pdf".into();
    second.package_version = "0.1.0".into();
    second.disabled_bindings = vec![
        DisabledBindingRecord {
            state: "disabled".into(),
            capability_name: "z.last".into(),
            capability_version: 1,
            manifest_digest: format!("sha256:{}", "b".repeat(64)),
            provider_operation_name: "z_last".into(),
        },
        DisabledBindingRecord {
            state: "disabled".into(),
            capability_name: "a.first".into(),
            capability_version: 2,
            manifest_digest: format!("sha256:{}", "a".repeat(64)),
            provider_operation_name: "a_first".into(),
        },
    ];
    second.record_digest.clear();
    second.record_digest = sha256(&canonical(&second));
    second.validate().unwrap();
    fs::write(
        root.join("installed-records")
            .join(format!("{}.json", second.installed_id)),
        canonical(&second),
    )
    .unwrap();
    let before = snapshot(&root);
    let (code, envelope) = run(&root);
    assert_eq!(code, 0, "{envelope}");
    let plugs = envelope["data"]["plugs"].as_array().unwrap();
    assert_eq!(plugs.len(), 2);
    assert_eq!(plugs[0]["package_id"], "tethers.pdf-tools");
    assert_eq!(plugs[1]["package_id"], "tethers.z-pdf");
    let capabilities = plugs[1]["capabilities"].as_array().unwrap();
    assert_eq!(capabilities[0]["name"], "a.first");
    assert_eq!(capabilities[1]["name"], "z.last");
    assert_read_only(&root, &before, &snapshot(&root));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn empty_root_is_successful_and_read_only() {
    let root = std::env::temp_dir().join(format!("tethers-j24b-empty-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let before = snapshot(&root);
    let (code, envelope) = run(&root);
    assert_eq!(code, 0);
    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["data"]["count"], 0);
    assert_read_only(&root, &before, &snapshot(&root));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn missing_and_partial_roots_fail_closed_without_creation() {
    let missing = std::env::temp_dir().join(format!("tethers-j24b-missing-{}", Uuid::new_v4()));
    let (code, envelope) = run(&missing);
    assert_eq!(code, 4);
    assert_eq!(envelope["status"], "unavailable");
    assert_eq!(envelope["error"]["code"], "plug_data_root_unavailable");
    assert!(!missing.exists());

    let root = std::env::temp_dir().join(format!("tethers-j24b-partial-{}", Uuid::new_v4()));
    fs::create_dir_all(root.join("install")).unwrap();
    let before = snapshot(&root);
    let (code, envelope) = run(&root);
    assert_eq!(code, 3);
    assert_eq!(envelope["status"], "invalid_data");
    assert_eq!(envelope["error"]["code"], "plug_store_incomplete");
    assert_read_only(&root, &before, &snapshot(&root));
    fs::remove_dir_all(root).unwrap();
}
