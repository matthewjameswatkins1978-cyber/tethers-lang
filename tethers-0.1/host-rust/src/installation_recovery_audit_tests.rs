use crate::installation_publication_intent::InstallationPublicationIntent;
use crate::installed::{DisabledBindingRecord, InstalledPlugRecord, InstalledPlugRegistry};
use crate::m3_store::{canonical, sha256};
use crate::package::{CapabilityEvidence, PayloadEvidence};
use crate::trust::{PackageTrustEvidence, TrustModeEvidence};
use std::fs;
use std::path::Path;
use uuid::Uuid;

fn audit_valid_record() -> InstalledPlugRecord {
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
        capability_manifests: vec![CapabilityEvidence {
            name: "file.move".into(),
            version: 1,
            operation: "file_move".into(),
            manifest_path: "manifest.json".into(),
            manifest_digest: digest.into(),
        }],
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
        operational_scope_schema: None,
        operational_scope_schema_digest: None,
        created_unix_ms: 1,
        record_digest: String::new(),
    };
    let mut covered = record.clone();
    covered.record_digest.clear();
    record.record_digest = sha256(&canonical(&covered).unwrap());
    record
}

fn audit_valid_intent() -> (InstalledPlugRecord, InstallationPublicationIntent) {
    let record = audit_valid_record();
    let intent = InstallationPublicationIntent::from_precomputed_record(record.clone()).unwrap();
    (record, intent)
}

fn audit_registry() -> (
    InstalledPlugRegistry,
    std::path::PathBuf,
    std::path::PathBuf,
) {
    let base = std::env::temp_dir().join(format!("tethers-j24k3c4-{}", Uuid::new_v4()));
    let install_root = base.join("install");
    let record_root = base.join("records");
    fs::create_dir_all(&install_root).unwrap();
    fs::create_dir_all(&record_root).unwrap();
    let registry = InstalledPlugRegistry::open_existing(&install_root, &record_root).unwrap();
    (registry, install_root, record_root)
}

fn set_read_only(path: &Path) {
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_readonly(true);
    fs::set_permissions(path, permissions).unwrap();
}

fn set_writable(path: &Path) {
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_readonly(false);
    fs::set_permissions(path, permissions).unwrap();
}

fn write_payload(parent: &Path, relative: &str, bytes: &[u8]) -> PayloadEvidence {
    let path = parent.join(relative);
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).unwrap();
    }
    fs::write(&path, bytes).unwrap();
    set_read_only(&path);
    PayloadEvidence {
        path: relative.into(),
        sha256: sha256(bytes),
        size_bytes: bytes.len() as u64,
        role: "payload".into(),
    }
}

fn build_valid_record_with_payloads(
    plug_json_bytes: &[u8],
    payload_relatives: &[&str],
    payload_bytes: &[u8],
    signature_relatives: &[&str],
    signature_bytes: &[u8],
) -> InstalledPlugRecord {
    let mut record = audit_valid_record();
    record.plug_json.sha256 = sha256(plug_json_bytes);
    record.plug_json.size_bytes = plug_json_bytes.len() as u64;
    record.payloads = payload_relatives
        .iter()
        .map(|relative| PayloadEvidence {
            path: (*relative).into(),
            sha256: sha256(payload_bytes),
            size_bytes: payload_bytes.len() as u64,
            role: "payload".into(),
        })
        .collect();
    record.signature_files = signature_relatives
        .iter()
        .map(|relative| PayloadEvidence {
            path: (*relative).into(),
            sha256: sha256(signature_bytes),
            size_bytes: signature_bytes.len() as u64,
            role: "signature".into(),
        })
        .collect();
    let mut covered = record.clone();
    covered.record_digest.clear();
    record.record_digest = sha256(&canonical(&covered).unwrap());
    record
}

fn build_destination_for_record(
    install_root: &Path,
    record: &InstalledPlugRecord,
    plug_json_bytes: &[u8],
    payload_bytes: &[u8],
    signature_bytes: &[u8],
) {
    let destination = install_root.join(&record.installation_relative_path);
    fs::create_dir(&destination).unwrap();
    write_payload(&destination, &record.plug_json.path, plug_json_bytes);
    for payload in &record.payloads {
        let bytes = if payload_bytes.is_empty() {
            vec![0u8; payload.size_bytes as usize]
        } else {
            payload_bytes.to_vec()
        };
        write_payload(&destination, &payload.path, &bytes);
    }
    for sig in &record.signature_files {
        let bytes = if signature_bytes.is_empty() {
            vec![1u8; sig.size_bytes as usize]
        } else {
            signature_bytes.to_vec()
        };
        write_payload(&destination, &sig.path, &bytes);
    }
}

fn write_record(record_root: &Path, record: &InstalledPlugRecord) {
    let path = record_root.join(format!("{}.json", record.installed_id));
    let bytes = canonical(record).unwrap();
    fs::write(path, bytes).unwrap();
}

#[test]
fn j24k3c4_empty_roots_pass_without_intent() {
    let (registry, _install_root, _record_root) = audit_registry();
    registry
        .audit_installation_recovery_destinations(None)
        .unwrap();
}

#[test]
fn j24k3c4_single_valid_record_and_destination_pass_without_intent() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let record = build_valid_record_with_payloads(
        plug_json_bytes,
        &["manifest.json"],
        payload_bytes,
        &["signature.sig"],
        signature_bytes,
    );
    let (registry, install_root, record_root) = audit_registry();
    build_destination_for_record(
        &install_root,
        &record,
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    write_record(&record_root, &record);
    registry
        .audit_installation_recovery_destinations(None)
        .unwrap();
}

#[test]
fn j24k3c4_multiple_valid_records_pass_without_intent() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let record1 = build_valid_record_with_payloads(
        plug_json_bytes,
        &["manifest.json"],
        payload_bytes,
        &["signature.sig"],
        signature_bytes,
    );
    let mut record2 =
        build_valid_record_with_payloads(b"{\"plug\":false}", &["other.json"], b"other", &[], &[]);
    record2.package_id = "tethers.other".into();
    record2.package_version = "2.0.0".into();
    let mut covered = record2.clone();
    covered.record_digest.clear();
    record2.record_digest = sha256(&canonical(&covered).unwrap());
    let (registry, install_root, record_root) = audit_registry();
    build_destination_for_record(
        &install_root,
        &record1,
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    write_record(&record_root, &record1);
    build_destination_for_record(&install_root, &record2, b"{\"plug\":false}", b"other", &[]);
    write_record(&record_root, &record2);
    registry
        .audit_installation_recovery_destinations(None)
        .unwrap();
}

#[test]
fn j24k3c4_valid_intent_absent_destination_passes() {
    let (record, intent) = audit_valid_intent();
    let (registry, _install_root, _record_root) = audit_registry();
    registry
        .audit_installation_recovery_destinations(Some(&intent))
        .unwrap();
    drop(record);
}

#[test]
fn j24k3c4_valid_intent_destination_present_without_record_passes() {
    let (record, intent) = audit_valid_intent();
    let (registry, install_root, _record_root) = audit_registry();
    let destination = install_root.join(&intent.destination_relative_path);
    fs::create_dir(&destination).unwrap();
    registry
        .audit_installation_recovery_destinations(Some(&intent))
        .unwrap();
    drop(record);
}

#[test]
fn j24k3c4_matching_intent_record_and_destination_pass() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let record = build_valid_record_with_payloads(
        plug_json_bytes,
        &["manifest.json"],
        payload_bytes,
        &["signature.sig"],
        signature_bytes,
    );
    let intent = InstallationPublicationIntent::from_precomputed_record(record.clone()).unwrap();
    let (registry, install_root, record_root) = audit_registry();
    build_destination_for_record(
        &install_root,
        &record,
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    write_record(&record_root, &record);
    registry
        .audit_installation_recovery_destinations(Some(&intent))
        .unwrap();
}

#[test]
fn j24k3c4_untracked_canonical_directory_fails_without_intent() {
    let (registry, install_root, _record_root) = audit_registry();
    let orphan_id = Uuid::new_v4().to_string();
    let orphan = install_root.join(format!("plug-{orphan_id}"));
    fs::create_dir(&orphan).unwrap();
    let err = registry
        .audit_installation_recovery_destinations(None)
        .unwrap_err();
    assert_eq!(err.code, "installation_destination_untracked");
}

#[test]
fn j24k3c4_intent_does_not_excuse_second_untracked() {
    let (record, intent) = audit_valid_intent();
    let (registry, install_root, _record_root) = audit_registry();
    let orphan_id = Uuid::new_v4().to_string();
    let orphan = install_root.join(format!("plug-{orphan_id}"));
    fs::create_dir(&orphan).unwrap();
    let err = registry
        .audit_installation_recovery_destinations(Some(&intent))
        .unwrap_err();
    assert_eq!(err.code, "installation_destination_untracked");
    drop(record);
}

#[test]
fn j24k3c4_malformed_plug_directory_name_fails_as_untracked() {
    let (registry, install_root, _record_root) = audit_registry();
    let malformed = install_root.join("plug-not-a-uuid");
    fs::create_dir(&malformed).unwrap();
    let err = registry
        .audit_installation_recovery_destinations(None)
        .unwrap_err();
    assert_eq!(err.code, "installation_destination_untracked");
}

#[test]
fn j24k3c4_untracked_canonical_plug_file_fails_as_untracked() {
    let (registry, install_root, _record_root) = audit_registry();
    let id = Uuid::new_v4().to_string();
    let file_path = install_root.join(format!("plug-{id}"));
    fs::write(&file_path, b"not-a-directory").unwrap();
    let err = registry
        .audit_installation_recovery_destinations(None)
        .unwrap_err();
    assert_eq!(err.code, "installation_destination_untracked");
}

#[test]
fn j24k3c4_accounted_destination_present_as_file_fails_conflict() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let record = build_valid_record_with_payloads(
        plug_json_bytes,
        &["manifest.json"],
        payload_bytes,
        &["signature.sig"],
        signature_bytes,
    );
    let (registry, install_root, record_root) = audit_registry();
    build_destination_for_record(
        &install_root,
        &record,
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    write_record(&record_root, &record);
    let destination = install_root.join(&record.installation_relative_path);
    fs::remove_dir_all(&destination).unwrap();
    fs::write(&destination, b"now-a-file").unwrap();
    let err = registry
        .audit_installation_recovery_destinations(None)
        .unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
}

#[cfg(windows)]
#[test]
fn j24k3c4_windows_junction_tracked_destination_load_all_refused() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let record = build_valid_record_with_payloads(
        plug_json_bytes,
        &["manifest.json"],
        payload_bytes,
        &["signature.sig"],
        signature_bytes,
    );
    let (registry, install_root, record_root) = audit_registry();
    build_destination_for_record(
        &install_root,
        &record,
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    write_record(&record_root, &record);
    let destination = install_root.join(&record.installation_relative_path);
    fs::remove_dir_all(&destination).unwrap();
    let target = install_root.join(format!("target-{}", Uuid::new_v4()));
    fs::create_dir(&target).unwrap();
    let status = std::process::Command::new("cmd")
        .args([
            "/C",
            "mklink",
            "/J",
            destination.to_str().unwrap(),
            target.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(
        status.success(),
        "could not create Windows junction fixture"
    );
    let err = registry
        .audit_installation_recovery_destinations(None)
        .unwrap_err();
    assert_eq!(err.code, "unsafe_store_path");
}

#[cfg(unix)]
#[test]
fn j24k3c4_unix_symlink_tracked_destination_load_all_refused() {
    use std::os::unix::fs::symlink;
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let record = build_valid_record_with_payloads(
        plug_json_bytes,
        &["manifest.json"],
        payload_bytes,
        &["signature.sig"],
        signature_bytes,
    );
    let (registry, install_root, record_root) = audit_registry();
    build_destination_for_record(
        &install_root,
        &record,
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    write_record(&record_root, &record);
    let destination = install_root.join(&record.installation_relative_path);
    fs::remove_dir_all(&destination).unwrap();
    let target = install_root.join(format!("target-{}", Uuid::new_v4()));
    fs::create_dir(&target).unwrap();
    symlink(&target, &destination).unwrap();
    let err = registry
        .audit_installation_recovery_destinations(None)
        .unwrap_err();
    assert_eq!(err.code, "unsafe_store_path");
}

#[cfg(windows)]
#[test]
fn j24k3c4_windows_junction_final_destination_refused() {
    let (registry, install_root, _record_root) = audit_registry();
    let id = Uuid::new_v4().to_string();
    let junction_path = install_root.join(format!("plug-{id}"));
    let target = install_root.join(format!("target-{id}"));
    fs::create_dir(&target).unwrap();
    let status = std::process::Command::new("cmd")
        .args([
            "/C",
            "mklink",
            "/J",
            junction_path.to_str().unwrap(),
            target.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(
        status.success(),
        "could not create Windows junction fixture"
    );
    let err = registry
        .audit_installation_recovery_destinations(None)
        .unwrap_err();
    assert_eq!(err.code, "unsafe_store_path");
}

#[cfg(unix)]
#[test]
fn j24k3c4_unix_symlink_final_destination_refused() {
    use std::os::unix::fs::symlink;
    let (registry, install_root, _record_root) = audit_registry();
    let id = Uuid::new_v4().to_string();
    let link_path = install_root.join(format!("plug-{id}"));
    let target = install_root.join(format!("target-{id}"));
    fs::create_dir(&target).unwrap();
    symlink(&target, &link_path).unwrap();
    let err = registry
        .audit_installation_recovery_destinations(None)
        .unwrap_err();
    assert_eq!(err.code, "unsafe_store_path");
}

#[test]
fn j24k3c4_record_destination_not_exact_plug_id_fails_closed() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let mut record = build_valid_record_with_payloads(
        plug_json_bytes,
        &["manifest.json"],
        payload_bytes,
        &["signature.sig"],
        signature_bytes,
    );
    record.installation_relative_path = "plug-wrong-destination".into();
    let mut covered = record.clone();
    covered.record_digest.clear();
    record.record_digest = sha256(&canonical(&covered).unwrap());
    let (registry, install_root, record_root) = audit_registry();
    build_destination_for_record(
        &install_root,
        &record,
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    write_record(&record_root, &record);
    let err = registry
        .audit_installation_recovery_destinations(None)
        .unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
}

#[test]
fn j24k3c4_intent_and_record_claim_same_destination_with_different_records_fails() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let existing = build_valid_record_with_payloads(
        plug_json_bytes,
        &["manifest.json"],
        payload_bytes,
        &["signature.sig"],
        signature_bytes,
    );
    let (registry, install_root, record_root) = audit_registry();
    build_destination_for_record(
        &install_root,
        &existing,
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    write_record(&record_root, &existing);

    let mut different_record = existing.clone();
    different_record.plug_json.sha256 = sha256(b"different");
    different_record.plug_json.size_bytes = 9;
    let mut covered = different_record.clone();
    covered.record_digest.clear();
    different_record.record_digest = sha256(&canonical(&covered).unwrap());
    let intent =
        InstallationPublicationIntent::from_precomputed_record(different_record.clone()).unwrap();

    let err = registry
        .audit_installation_recovery_destinations(Some(&intent))
        .unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
}

#[test]
fn j24k3c4_tracked_record_missing_destination_fails_conflict() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let record = build_valid_record_with_payloads(
        plug_json_bytes,
        &["manifest.json"],
        payload_bytes,
        &["signature.sig"],
        signature_bytes,
    );
    let (registry, install_root, record_root) = audit_registry();
    write_record(&record_root, &record);
    let _unused = install_root; // destination dir is not created
    let err = registry
        .audit_installation_recovery_destinations(None)
        .unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
}

#[test]
fn j24k3c4_installed_bytes_drift_fails_conflict() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let record = build_valid_record_with_payloads(
        plug_json_bytes,
        &["manifest.json"],
        payload_bytes,
        &["signature.sig"],
        signature_bytes,
    );
    let (registry, install_root, record_root) = audit_registry();
    build_destination_for_record(
        &install_root,
        &record,
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    write_record(&record_root, &record);
    let destination = install_root.join(&record.installation_relative_path);
    let manifest = destination.join("manifest.json");
    set_writable(&manifest);
    fs::write(&manifest, b"drifted-payload-data").unwrap();
    set_read_only(&manifest);
    let err = registry
        .audit_installation_recovery_destinations(None)
        .unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
}

#[test]
fn j24k3c4_staging_and_unrelated_entries_ignored() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let record = build_valid_record_with_payloads(
        plug_json_bytes,
        &["manifest.json"],
        payload_bytes,
        &["signature.sig"],
        signature_bytes,
    );
    let (registry, install_root, record_root) = audit_registry();
    build_destination_for_record(
        &install_root,
        &record,
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    write_record(&record_root, &record);
    let staging = install_root.join(".staging-00000000-0000-0000-0000-000000000000");
    fs::create_dir(&staging).unwrap();
    let unrelated_dir = install_root.join("other-things");
    fs::create_dir(&unrelated_dir).unwrap();
    let unrelated_file = install_root.join("some.txt");
    fs::write(&unrelated_file, b"hello").unwrap();
    registry
        .audit_installation_recovery_destinations(None)
        .unwrap();
    assert!(staging.exists());
    assert!(unrelated_dir.exists());
    assert!(unrelated_file.exists());
}

#[test]
fn j24k3c4_invalid_intent_wins_before_missing_roots() {
    let (record, mut intent) = audit_valid_intent();
    intent.schema_version = 0;
    let (registry, install_root, _record_root) = audit_registry();
    fs::remove_dir_all(&install_root).unwrap();
    let err = registry
        .audit_installation_recovery_destinations(Some(&intent))
        .unwrap_err();
    assert_eq!(err.code, "installation_intent_invalid");
    drop(record);
}

#[test]
fn j24k3c4_missing_install_root_returns_recovery_io() {
    let (registry, install_root, _record_root) = audit_registry();
    fs::remove_dir_all(&install_root).unwrap();
    let err = registry
        .audit_installation_recovery_destinations(None)
        .unwrap_err();
    assert_eq!(err.code, "installation_recovery_io");
}

#[test]
fn j24k3c4_missing_record_root_returns_recovery_io() {
    let (registry, _install_root, record_root) = audit_registry();
    fs::remove_dir_all(&record_root).unwrap();
    let err = registry
        .audit_installation_recovery_destinations(None)
        .unwrap_err();
    assert_eq!(err.code, "installation_recovery_io");
}

#[test]
fn j24k3c4_non_canonical_uuid_in_plug_name_fails_as_untracked() {
    let (registry, install_root, _record_root) = audit_registry();
    let id = "550E8400-E29B-41D4-A716-446655440000";
    let path = install_root.join(format!("plug-{id}"));
    fs::create_dir(&path).unwrap();
    let err = registry
        .audit_installation_recovery_destinations(None)
        .unwrap_err();
    assert_eq!(err.code, "installation_destination_untracked");
}

#[test]
fn j24k3c4_duplicate_destination_claims_fail_conflict() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let record1 = build_valid_record_with_payloads(
        plug_json_bytes,
        &["manifest.json"],
        payload_bytes,
        &["signature.sig"],
        signature_bytes,
    );
    let mut record2 = record1.clone();
    record2.installed_id = Uuid::new_v4().to_string();
    record2.installation_relative_path = record1.installation_relative_path.clone();
    let mut covered = record2.clone();
    covered.record_digest.clear();
    record2.record_digest = sha256(&canonical(&covered).unwrap());
    let (registry, install_root, record_root) = audit_registry();
    build_destination_for_record(
        &install_root,
        &record1,
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    write_record(&record_root, &record1);
    write_record(&record_root, &record2);
    let err = registry
        .audit_installation_recovery_destinations(None)
        .unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
}

#[test]
fn j24k3c4_success_leaves_state_unchanged() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let record1 = build_valid_record_with_payloads(
        plug_json_bytes,
        &["manifest.json"],
        payload_bytes,
        &["signature.sig"],
        signature_bytes,
    );
    let mut record2 =
        build_valid_record_with_payloads(b"{\"plug\":false}", &["other.json"], b"other", &[], &[]);
    record2.package_id = "tethers.other".into();
    record2.package_version = "2.0.0".into();
    let mut covered = record2.clone();
    covered.record_digest.clear();
    record2.record_digest = sha256(&canonical(&covered).unwrap());
    let (registry, install_root, record_root) = audit_registry();
    build_destination_for_record(
        &install_root,
        &record1,
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    write_record(&record_root, &record1);
    build_destination_for_record(&install_root, &record2, b"{\"plug\":false}", b"other", &[]);
    write_record(&record_root, &record2);

    let snapshot_before = snapshot_state(&install_root, &record_root);

    registry
        .audit_installation_recovery_destinations(None)
        .unwrap();

    let snapshot_after = snapshot_state(&install_root, &record_root);
    for (path, before) in &snapshot_before {
        let after = snapshot_after.get(path).expect("entry should still exist");
        assert_eq!(before.bytes, after.bytes, "bytes changed for {:?}", path);
        assert_eq!(
            before.modified, after.modified,
            "modification time changed for {:?}",
            path
        );
        assert_eq!(
            before.readonly, after.readonly,
            "readonly changed for {:?}",
            path
        );
    }
    assert_eq!(
        snapshot_before.len(),
        snapshot_after.len(),
        "entry count changed"
    );
}

struct EntrySnapshot {
    bytes: Vec<u8>,
    modified: Option<std::time::SystemTime>,
    readonly: bool,
}

fn snapshot_state(
    install_root: &Path,
    record_root: &Path,
) -> std::collections::BTreeMap<std::path::PathBuf, EntrySnapshot> {
    let mut snap = std::collections::BTreeMap::new();
    for root in &[install_root, record_root] {
        snapshot_root(root, root, &mut snap);
    }
    snap
}

fn snapshot_root(
    base: &Path,
    current: &Path,
    snap: &mut std::collections::BTreeMap<std::path::PathBuf, EntrySnapshot>,
) {
    if let Ok(entries) = fs::read_dir(current) {
        for entry in entries.flatten() {
            let path = entry.path();
            let kind = entry.file_type().unwrap();
            if kind.is_dir() {
                snapshot_root(base, &path, snap);
            } else if kind.is_file() {
                let relative = path.strip_prefix(base).unwrap().to_path_buf();
                let bytes = fs::read(&path).unwrap();
                let meta = fs::symlink_metadata(&path).unwrap();
                snap.insert(
                    relative,
                    EntrySnapshot {
                        bytes,
                        modified: meta.modified().ok(),
                        readonly: meta.permissions().readonly(),
                    },
                );
            }
        }
    }
}
