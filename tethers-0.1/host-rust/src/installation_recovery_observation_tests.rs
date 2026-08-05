use crate::installation_publication_intent::InstallationPublicationIntent;
use crate::installed::{DisabledBindingRecord, InstalledPlugRecord, InstalledPlugRegistry};
use crate::m3_store::{canonical, sha256};
use crate::package::PayloadEvidence;
use crate::trust::{PackageTrustEvidence, TrustModeEvidence};
use std::fs;
use std::path::Path;
use uuid::Uuid;

fn valid_record() -> InstalledPlugRecord {
    let digest = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let mut trust = PackageTrustEvidence {
        evidence_format_version: 1,
        semantic_package_digest: digest.into(),
        mode: TrustModeEvidence::UnsignedDeveloper {
            approval_id: "approval".into(),
            approval_record_digest: digest.into(),
            visibly_unsigned: true,
        },
        evidence_digest: String::new(),
    };
    let mut trust_covered = trust.clone();
    trust_covered.evidence_digest.clear();
    trust.evidence_digest = sha256(&canonical(&trust_covered).unwrap());
    let id = Uuid::new_v4().to_string();
    let mut record = InstalledPlugRecord {
        schema_version: 1,
        installed_id: id.clone(),
        state: "present_disabled".into(),
        package_id: "tethers.file-tools".into(),
        package_version: "1.1.0".into(),
        semantic_package_digest: digest.into(),
        source_candidate_id: "candidate".into(),
        installation_relative_path: format!("plug-{id}"),
        raw_archive_digest: digest.into(),
        plug_json: PayloadEvidence {
            path: "plug.json".into(),
            sha256: digest.into(),
            size_bytes: 1,
            role: "package_descriptor".into(),
        },
        payloads: Vec::new(),
        signature_files: Vec::new(),
        capability_manifests: Vec::new(),
        trust_evidence: trust,
        installation_approval_id: "approval".into(),
        installation_approval_digest: digest.into(),
        conformance_evidence_id: "conformance".into(),
        conformance_evidence_digest: digest.into(),
        provider_id: "tethers-file-tools".into(),
        provider_version: "1.0.0".into(),
        launch_path: "provider/file_tools_provider.exe".into(),
        launch_arguments: Vec::new(),
        provider_working_directory: "provider".into(),
        launch_profile_label: "supervised".into(),
        socket_major: 1,
        mcp_protocol_version: "2025-11-25".into(),
        platform: "windows".into(),
        architecture: "x86_64".into(),
        disabled_bindings: vec![DisabledBindingRecord {
            state: "disabled".into(),
            capability_name: "file.move".into(),
            capability_version: 1,
            manifest_digest: digest.into(),
            provider_operation_name: "file_move".into(),
        }],
        created_unix_ms: 1,
        record_digest: String::new(),
    };
    let mut covered = record.clone();
    covered.record_digest.clear();
    record.record_digest = sha256(&canonical(&covered).unwrap());
    record
}

fn valid_intent() -> (InstalledPlugRecord, InstallationPublicationIntent) {
    let record = valid_record();
    let intent = InstallationPublicationIntent::from_precomputed_record(record.clone()).unwrap();
    (record, intent)
}

fn registry() -> (
    InstalledPlugRegistry,
    std::path::PathBuf,
    std::path::PathBuf,
) {
    let base = std::env::temp_dir().join(format!("tethers-j24k3c1-{}", Uuid::new_v4()));
    let install_root = base.join("install");
    let record_root = base.join("records");
    fs::create_dir_all(&install_root).unwrap();
    fs::create_dir_all(&record_root).unwrap();
    let registry = InstalledPlugRegistry::open_existing(&install_root, &record_root).unwrap();
    (registry, install_root, record_root)
}

fn create_staging(install_root: &Path, transaction_id: &str) {
    fs::create_dir(install_root.join(format!(".staging-{transaction_id}"))).unwrap();
}

fn create_destination(install_root: &Path, relative_path: &str) {
    fs::create_dir(install_root.join(relative_path)).unwrap();
}

fn write_record(record_root: &Path, transaction_id: &str, record: &InstalledPlugRecord) {
    let path = record_root.join(format!("{transaction_id}.json"));
    let bytes = canonical(record).unwrap();
    fs::create_dir_all(record_root).unwrap();
    fs::write(path, bytes).unwrap();
}

fn write_record_bytes(record_root: &Path, transaction_id: &str, bytes: &[u8]) {
    let path = record_root.join(format!("{transaction_id}.json"));
    fs::create_dir_all(record_root).unwrap();
    fs::write(path, bytes).unwrap();
}

#[test]
fn j24k3c1_empty_transaction_state() {
    let (record, intent) = valid_intent();
    let (registry, _install_root, _record_root) = registry();
    let snapshot = registry.observe_installation_recovery(&intent).unwrap();
    assert!(!snapshot.staging_present);
    assert!(!snapshot.destination_present);
    assert!(snapshot.installed_record.is_none());
    drop(record);
}

#[test]
fn j24k3c1_staging_only() {
    let (record, intent) = valid_intent();
    let (registry, install_root, _record_root) = registry();
    create_staging(&install_root, &intent.transaction_id);
    let snapshot = registry.observe_installation_recovery(&intent).unwrap();
    assert!(snapshot.staging_present);
    assert!(!snapshot.destination_present);
    assert!(snapshot.installed_record.is_none());
    drop(record);
}

#[test]
fn j24k3c1_destination_only() {
    let (record, intent) = valid_intent();
    let (registry, install_root, _record_root) = registry();
    create_destination(&install_root, &intent.destination_relative_path);
    let snapshot = registry.observe_installation_recovery(&intent).unwrap();
    assert!(!snapshot.staging_present);
    assert!(snapshot.destination_present);
    assert!(snapshot.installed_record.is_none());
    drop(record);
}

#[test]
fn j24k3c1_record_only() {
    let (record, intent) = valid_intent();
    let (registry, _install_root, record_root) = registry();
    write_record(&record_root, &intent.transaction_id, &record);
    let snapshot = registry.observe_installation_recovery(&intent).unwrap();
    assert!(!snapshot.staging_present);
    assert!(!snapshot.destination_present);
    assert_eq!(snapshot.installed_record.as_ref(), Some(&record));
    drop(record);
}

#[test]
fn j24k3c1_all_three_facts() {
    let (record, intent) = valid_intent();
    let (registry, install_root, record_root) = registry();
    create_staging(&install_root, &intent.transaction_id);
    create_destination(&install_root, &intent.destination_relative_path);
    write_record(&record_root, &intent.transaction_id, &record);
    let snapshot = registry.observe_installation_recovery(&intent).unwrap();
    assert!(snapshot.staging_present);
    assert!(snapshot.destination_present);
    assert_eq!(snapshot.installed_record.as_ref(), Some(&record));
    drop(record);
}

#[test]
fn j24k3c1_snapshot_to_observation_bridge_preserves_facts() {
    let (record, intent) = valid_intent();
    let (registry, install_root, record_root) = registry();
    create_staging(&install_root, &intent.transaction_id);
    write_record(&record_root, &intent.transaction_id, &record);
    let snapshot = registry.observe_installation_recovery(&intent).unwrap();
    let obs = snapshot.as_observation(&intent);
    assert_eq!(obs.staging_present, snapshot.staging_present);
    assert_eq!(obs.destination_present, snapshot.destination_present);
    assert_eq!(obs.installed_record, snapshot.installed_record.as_ref());
    assert!(std::ptr::eq(obs.intent, &intent));
    drop(record);
}

#[test]
fn j24k3c1_malformed_record_json_fails_closed() {
    let (record, intent) = valid_intent();
    let (registry, _install_root, record_root) = registry();
    write_record_bytes(&record_root, &intent.transaction_id, b"{");
    let err = registry.observe_installation_recovery(&intent).unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
    drop(record);
}

#[test]
fn j24k3c1_duplicate_key_record_json_fails_closed() {
    let (record, intent) = valid_intent();
    let (registry, _install_root, record_root) = registry();
    write_record_bytes(
        &record_root,
        &intent.transaction_id,
        br#"{"schema_version":1,"schema_version":1}"#,
    );
    let err = registry.observe_installation_recovery(&intent).unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
    drop(record);
}

#[test]
fn j24k3c1_unknown_field_record_json_fails_closed() {
    let (record, intent) = valid_intent();
    let (registry, _install_root, record_root) = registry();
    write_record_bytes(&record_root, &intent.transaction_id, br#"{"unknown":1}"#);
    let err = registry.observe_installation_recovery(&intent).unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
    drop(record);
}

#[test]
fn j24k3c1_record_path_is_directory_fails_closed() {
    let (record, intent) = valid_intent();
    let (registry, _install_root, record_root) = registry();
    let dir_path = record_root.join(format!("{}.json", intent.transaction_id));
    fs::create_dir(&dir_path).unwrap();
    let err = registry.observe_installation_recovery(&intent).unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
    drop(record);
}

#[test]
fn j24k3c1_staging_path_is_file_fails_closed() {
    let (record, intent) = valid_intent();
    let (registry, install_root, _record_root) = registry();
    let file_path = install_root.join(format!(".staging-{}", intent.transaction_id));
    fs::write(&file_path, b"").unwrap();
    let err = registry.observe_installation_recovery(&intent).unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
    drop(record);
}

#[test]
fn j24k3c1_destination_path_is_file_fails_closed() {
    let (record, intent) = valid_intent();
    let (registry, install_root, _record_root) = registry();
    let file_path = install_root.join(&intent.destination_relative_path);
    fs::write(&file_path, b"").unwrap();
    let err = registry.observe_installation_recovery(&intent).unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
    drop(record);
}

#[cfg(windows)]
#[test]
fn j24k3c1_windows_junction_staging_path_is_refused() {
    let (record, intent) = valid_intent();
    let (registry, install_root, _record_root) = registry();
    let staging_path = install_root.join(format!(".staging-{}", intent.transaction_id));
    let target = install_root.join(format!("target-{}", Uuid::new_v4()));
    fs::create_dir(&target).unwrap();
    let status = std::process::Command::new("cmd")
        .args([
            "/C",
            "mklink",
            "/J",
            staging_path.to_str().unwrap(),
            target.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(
        status.success(),
        "could not create Windows junction fixture"
    );
    let err = registry.observe_installation_recovery(&intent).unwrap_err();
    assert_eq!(err.code, "unsafe_store_path");
    drop(record);
}

#[cfg(windows)]
#[test]
fn j24k3c1_windows_junction_install_root_verify_chain_is_refused() {
    let (record, intent) = valid_intent();
    let base = std::env::temp_dir().join(format!("tethers-j24k3c1-{}", Uuid::new_v4()));
    let install_root = base.join("install");
    let record_root = base.join("records");
    fs::create_dir_all(&install_root).unwrap();
    fs::create_dir_all(&record_root).unwrap();
    let registry = InstalledPlugRegistry::open_existing(&install_root, &record_root).unwrap();

    fs::remove_dir(&install_root).unwrap();
    let target = base.join("target");
    fs::create_dir(&target).unwrap();
    let status = std::process::Command::new("cmd")
        .args([
            "/C",
            "mklink",
            "/J",
            install_root.to_str().unwrap(),
            target.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(
        status.success(),
        "could not create Windows junction fixture"
    );
    let err = registry.observe_installation_recovery(&intent).unwrap_err();
    assert_eq!(err.code, "unsafe_store_path");
    drop(record);
}

#[cfg(windows)]
#[test]
fn j24k3c1_windows_junction_record_root_verify_chain_is_refused() {
    let (record, intent) = valid_intent();
    let base = std::env::temp_dir().join(format!("tethers-j24k3c1-{}", Uuid::new_v4()));
    let install_root = base.join("install");
    let record_root = base.join("records");
    fs::create_dir_all(&install_root).unwrap();
    fs::create_dir_all(&record_root).unwrap();
    let registry = InstalledPlugRegistry::open_existing(&install_root, &record_root).unwrap();

    fs::remove_dir(&record_root).unwrap();
    let target = base.join("target");
    fs::create_dir(&target).unwrap();
    let status = std::process::Command::new("cmd")
        .args([
            "/C",
            "mklink",
            "/J",
            record_root.to_str().unwrap(),
            target.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(
        status.success(),
        "could not create Windows junction fixture"
    );
    let err = registry.observe_installation_recovery(&intent).unwrap_err();
    assert_eq!(err.code, "unsafe_store_path");
    drop(record);
}

#[cfg(unix)]
#[test]
fn j24k3c1_unix_symlink_staging_path_is_refused() {
    use std::os::unix::fs::symlink;
    let (record, intent) = valid_intent();
    let (registry, install_root, _record_root) = registry();
    let staging_path = install_root.join(format!(".staging-{}", intent.transaction_id));
    let target = install_root.join(format!("target-{}", Uuid::new_v4()));
    fs::create_dir(&target).unwrap();
    symlink(&target, &staging_path).unwrap();
    let err = registry.observe_installation_recovery(&intent).unwrap_err();
    assert_eq!(err.code, "unsafe_store_path");
    drop(record);
}

#[test]
fn j24k3c1_unrelated_entries_untouched_and_not_mistaken() {
    let (record, intent) = valid_intent();
    let (registry, install_root, record_root) = registry();

    let unrelated_staging = install_root.join(".staging-00000000-0000-0000-0000-000000000000");
    fs::create_dir(&unrelated_staging).unwrap();
    let unrelated_dest = install_root.join("plug-00000000-0000-0000-0000-000000000000");
    fs::create_dir(&unrelated_dest).unwrap();
    let unrelated_record_path = record_root.join("00000000-0000-0000-0000-000000000000.json");
    let unrelated_record = valid_record();
    let unrelated_bytes = canonical(&unrelated_record).unwrap();
    fs::write(&unrelated_record_path, &unrelated_bytes).unwrap();

    let snapshot = registry.observe_installation_recovery(&intent).unwrap();
    assert!(!snapshot.staging_present);
    assert!(!snapshot.destination_present);
    assert!(snapshot.installed_record.is_none());

    assert!(unrelated_staging.exists());
    assert!(unrelated_dest.exists());
    assert!(unrelated_record_path.exists());
    assert_eq!(fs::read(&unrelated_record_path).unwrap(), unrelated_bytes);

    drop(record);
}

#[test]
fn j24k3c1_observation_is_read_only_no_mutation() {
    let (record, intent) = valid_intent();
    let (registry, install_root, record_root) = registry();

    create_staging(&install_root, &intent.transaction_id);
    create_destination(&install_root, &intent.destination_relative_path);
    write_record(&record_root, &intent.transaction_id, &record);

    let staging_path = install_root.join(format!(".staging-{}", intent.transaction_id));
    let dest_path = install_root.join(&intent.destination_relative_path);
    let record_path = record_root.join(format!("{}.json", intent.transaction_id));
    let staging_meta_before = fs::symlink_metadata(&staging_path).unwrap();
    let dest_meta_before = fs::symlink_metadata(&dest_path).unwrap();
    let record_bytes_before = fs::read(&record_path).unwrap();

    let _snapshot = registry.observe_installation_recovery(&intent).unwrap();

    let staging_meta_after = fs::symlink_metadata(&staging_path).unwrap();
    let dest_meta_after = fs::symlink_metadata(&dest_path).unwrap();
    let record_bytes_after = fs::read(&record_path).unwrap();
    assert_eq!(
        staging_meta_before.modified().unwrap(),
        staging_meta_after.modified().unwrap()
    );
    assert_eq!(
        dest_meta_before.modified().unwrap(),
        dest_meta_after.modified().unwrap()
    );
    assert_eq!(record_bytes_before, record_bytes_after);

    drop(record);
}

#[test]
fn j24k3c1_invalid_intent_rejected_before_state_inspection() {
    let (record, intent) = valid_intent();
    let (registry, _install_root, _record_root) = registry();
    let mut invalid_intent = intent.clone();
    invalid_intent.schema_version = 0;
    let err = registry
        .observe_installation_recovery(&invalid_intent)
        .unwrap_err();
    assert_eq!(err.code, "installation_intent_invalid");
    drop(record);
}

#[test]
fn j24k3c1_missing_install_root_returns_recovery_io() {
    let (record, intent) = valid_intent();
    let (registry, install_root, _record_root) = registry();
    fs::remove_dir_all(&install_root).unwrap();
    let err = registry.observe_installation_recovery(&intent).unwrap_err();
    assert_eq!(err.code, "installation_recovery_io");
    drop(record);
}

#[test]
fn j24k3c1_missing_record_root_returns_recovery_io() {
    let (record, intent) = valid_intent();
    let (registry, _install_root, record_root) = registry();
    fs::remove_dir_all(&record_root).unwrap();
    let err = registry.observe_installation_recovery(&intent).unwrap_err();
    assert_eq!(err.code, "installation_recovery_io");
    drop(record);
}
