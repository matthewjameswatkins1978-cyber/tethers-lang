use crate::installation_publication_intent::InstallationPublicationIntent;
use crate::installed::{DisabledBindingRecord, InstalledPlugRecord, InstalledPlugRegistry};
use crate::m3_store::{canonical, sha256};
use crate::package::{CapabilityEvidence, PayloadEvidence};
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
    let base = std::env::temp_dir().join(format!("tethers-j24k3c2-{}", Uuid::new_v4()));
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
    for (idx, payload) in record.payloads.iter().enumerate() {
        let bytes = if payload_bytes.is_empty() {
            vec![idx as u8; payload.size_bytes as usize]
        } else {
            payload_bytes.to_vec()
        };
        write_payload(&destination, &payload.path, &bytes);
    }
    for (idx, signature) in record.signature_files.iter().enumerate() {
        let bytes = if signature_bytes.is_empty() {
            vec![(idx + 100) as u8; signature.size_bytes as usize]
        } else {
            signature_bytes.to_vec()
        };
        write_payload(&destination, &signature.path, &bytes);
    }
}

fn build_valid_record_with_payloads(
    payload_relatives: &[&str],
    signature_relatives: &[&str],
    plug_json_bytes: &[u8],
    payload_bytes: &[u8],
    signature_bytes: &[u8],
) -> InstalledPlugRecord {
    let mut record = valid_record();
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
    record.plug_json.sha256 = sha256(plug_json_bytes);
    record.plug_json.size_bytes = plug_json_bytes.len() as u64;
    let mut covered = record.clone();
    covered.record_digest.clear();
    record.record_digest = sha256(&canonical(&covered).unwrap());
    record
}

#[test]
fn j24k3c2_exact_valid_flat_destination_passes() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let record = build_valid_record_with_payloads(
        &["manifest.json"],
        &["signature.sig"],
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    let (registry, install_root, _record_root) = registry();
    build_destination_for_record(
        &install_root,
        &record,
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    let intent = InstallationPublicationIntent::from_precomputed_record(record.clone()).unwrap();
    registry
        .verify_installation_recovery_destination(&intent)
        .unwrap();
}

#[test]
fn j24k3c2_exact_valid_nested_destination_passes() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let record = build_valid_record_with_payloads(
        &["nested/manifest.json", "other/file.txt"],
        &["nested/signature.sig"],
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    let (registry, install_root, _record_root) = registry();
    build_destination_for_record(
        &install_root,
        &record,
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    let intent = InstallationPublicationIntent::from_precomputed_record(record.clone()).unwrap();
    registry
        .verify_installation_recovery_destination(&intent)
        .unwrap();
}

#[test]
fn j24k3c2_missing_destination_fails_closed() {
    let (record, intent) = valid_intent();
    let (registry, _install_root, _record_root) = registry();
    let err = registry
        .verify_installation_recovery_destination(&intent)
        .unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
    drop(record);
}

#[test]
fn j24k3c2_destination_as_file_fails_closed() {
    let (record, intent) = valid_intent();
    let (registry, install_root, _record_root) = registry();
    let destination = install_root.join(&intent.destination_relative_path);
    fs::write(&destination, b"").unwrap();
    let err = registry
        .verify_installation_recovery_destination(&intent)
        .unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
    drop(record);
}

#[test]
fn j24k3c2_missing_expected_file_fails_closed() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let record = build_valid_record_with_payloads(
        &["manifest.json"],
        &["signature.sig"],
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    let (registry, install_root, _record_root) = registry();
    let destination = install_root.join(&record.installation_relative_path);
    fs::create_dir(&destination).unwrap();
    write_payload(&destination, &record.plug_json.path, plug_json_bytes);
    write_payload(&destination, &record.payloads[0].path, payload_bytes);
    let intent = InstallationPublicationIntent::from_precomputed_record(record.clone()).unwrap();
    let err = registry
        .verify_installation_recovery_destination(&intent)
        .unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
    drop(record);
}

#[test]
fn j24k3c2_extra_file_fails_closed() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let record = build_valid_record_with_payloads(
        &["manifest.json"],
        &["signature.sig"],
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    let (registry, install_root, _record_root) = registry();
    build_destination_for_record(
        &install_root,
        &record,
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    let destination = install_root.join(&record.installation_relative_path);
    write_payload(&destination, "extra.txt", b"extra");
    let intent = InstallationPublicationIntent::from_precomputed_record(record.clone()).unwrap();
    let err = registry
        .verify_installation_recovery_destination(&intent)
        .unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
    drop(record);
}

#[test]
fn j24k3c2_changed_bytes_equal_length_fails_by_digest() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let record = build_valid_record_with_payloads(
        &["manifest.json"],
        &["signature.sig"],
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    let (registry, install_root, _record_root) = registry();
    build_destination_for_record(
        &install_root,
        &record,
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    let destination = install_root.join(&record.installation_relative_path);
    let manifest = destination.join("manifest.json");
    set_writable(&manifest);
    fs::write(&manifest, b"payl0ad").unwrap();
    set_read_only(&manifest);
    let intent = InstallationPublicationIntent::from_precomputed_record(record.clone()).unwrap();
    let err = registry
        .verify_installation_recovery_destination(&intent)
        .unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
    drop(record);
}

#[test]
fn j24k3c2_changed_length_fails_closed() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let record = build_valid_record_with_payloads(
        &["manifest.json"],
        &["signature.sig"],
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    let (registry, install_root, _record_root) = registry();
    build_destination_for_record(
        &install_root,
        &record,
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    let destination = install_root.join(&record.installation_relative_path);
    let manifest = destination.join("manifest.json");
    set_writable(&manifest);
    fs::write(&manifest, b"longer-payload").unwrap();
    set_read_only(&manifest);
    let intent = InstallationPublicationIntent::from_precomputed_record(record.clone()).unwrap();
    let err = registry
        .verify_installation_recovery_destination(&intent)
        .unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
    drop(record);
}

#[test]
fn j24k3c2_writable_expected_file_fails_closed() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let record = build_valid_record_with_payloads(
        &["manifest.json"],
        &["signature.sig"],
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    let (registry, install_root, _record_root) = registry();
    build_destination_for_record(
        &install_root,
        &record,
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    let destination = install_root.join(&record.installation_relative_path);
    let manifest = destination.join("manifest.json");
    set_writable(&manifest);
    let intent = InstallationPublicationIntent::from_precomputed_record(record.clone()).unwrap();
    let err = registry
        .verify_installation_recovery_destination(&intent)
        .unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
    drop(record);
}

#[cfg(windows)]
#[test]
fn j24k3c2_windows_junction_nested_entry_refused() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let record = build_valid_record_with_payloads(
        &["manifest.json"],
        &["signature.sig"],
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    let (registry, install_root, _record_root) = registry();
    build_destination_for_record(
        &install_root,
        &record,
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    let destination = install_root.join(&record.installation_relative_path);
    let junction = destination.join("junction");
    let target = install_root.join(format!("target-{}", Uuid::new_v4()));
    fs::create_dir(&target).unwrap();
    let status = std::process::Command::new("cmd")
        .args([
            "/C",
            "mklink",
            "/J",
            junction.to_str().unwrap(),
            target.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(
        status.success(),
        "could not create Windows junction fixture"
    );
    let intent = InstallationPublicationIntent::from_precomputed_record(record.clone()).unwrap();
    let err = registry
        .verify_installation_recovery_destination(&intent)
        .unwrap_err();
    assert_eq!(err.code, "unsafe_store_path");
    drop(record);
}

#[cfg(windows)]
#[test]
fn j24k3c2_windows_junction_destination_root_refused() {
    let (record, intent) = valid_intent();
    let (registry, install_root, _record_root) = registry();
    let destination = install_root.join(&intent.destination_relative_path);
    fs::create_dir(&destination).unwrap();
    fs::remove_dir(&destination).unwrap();
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
        .verify_installation_recovery_destination(&intent)
        .unwrap_err();
    assert_eq!(err.code, "unsafe_store_path");
    drop(record);
}

#[cfg(unix)]
#[test]
fn j24k3c2_unix_symlink_nested_entry_refused() {
    use std::os::unix::fs::symlink;
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let record = build_valid_record_with_payloads(
        &["manifest.json"],
        &["signature.sig"],
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    let (registry, install_root, _record_root) = registry();
    build_destination_for_record(
        &install_root,
        &record,
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    let destination = install_root.join(&record.installation_relative_path);
    let link = destination.join("link");
    let target = install_root.join(format!("target-{}", Uuid::new_v4()));
    fs::create_dir(&target).unwrap();
    symlink(&target, &link).unwrap();
    let intent = InstallationPublicationIntent::from_precomputed_record(record.clone()).unwrap();
    let err = registry
        .verify_installation_recovery_destination(&intent)
        .unwrap_err();
    assert_eq!(err.code, "unsafe_store_path");
    drop(record);
}

#[cfg(unix)]
#[test]
fn j24k3c2_unix_symlink_destination_root_refused() {
    use std::os::unix::fs::symlink;
    let (record, intent) = valid_intent();
    let (registry, install_root, _record_root) = registry();
    let destination = install_root.join(&intent.destination_relative_path);
    fs::create_dir(&destination).unwrap();
    fs::remove_dir(&destination).unwrap();
    let target = install_root.join(format!("target-{}", Uuid::new_v4()));
    fs::create_dir(&target).unwrap();
    symlink(&target, &destination).unwrap();
    let err = registry
        .verify_installation_recovery_destination(&intent)
        .unwrap_err();
    assert_eq!(err.code, "unsafe_store_path");
    drop(record);
}

#[test]
fn j24k3c2_duplicate_expected_paths_fail_closed() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let record = build_valid_record_with_payloads(
        &["manifest.json", "manifest.json"],
        &["signature.sig"],
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    let (registry, install_root, _record_root) = registry();
    let destination = install_root.join(&record.installation_relative_path);
    fs::create_dir(&destination).unwrap();
    write_payload(&destination, &record.plug_json.path, plug_json_bytes);
    write_payload(
        &destination,
        &record.signature_files[0].path,
        signature_bytes,
    );
    let intent = InstallationPublicationIntent::from_precomputed_record(record.clone()).unwrap();
    let err = registry
        .verify_installation_recovery_destination(&intent)
        .unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
    drop(record);
}

#[test]
fn j24k3c2_unsafe_expected_parent_traversal_fail_closed() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let mut record = build_valid_record_with_payloads(
        &["../escape.txt"],
        &["signature.sig"],
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    let (registry, install_root, _record_root) = registry();
    let destination = install_root.join(&record.installation_relative_path);
    fs::create_dir(&destination).unwrap();
    write_payload(&destination, &record.plug_json.path, plug_json_bytes);
    let escape_path = install_root.join("escape.txt");
    fs::write(&escape_path, payload_bytes).unwrap();
    record.payloads[0].sha256 = sha256(payload_bytes);
    record.payloads[0].size_bytes = payload_bytes.len() as u64;
    let mut covered = record.clone();
    covered.record_digest.clear();
    record.record_digest = sha256(&canonical(&covered).unwrap());
    let intent = InstallationPublicationIntent::from_precomputed_record(record.clone()).unwrap();
    let err = registry
        .verify_installation_recovery_destination(&intent)
        .unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
    drop(record);
}

#[test]
fn j24k3c2_unsafe_expected_absolute_path_fail_closed() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let mut record = build_valid_record_with_payloads(
        &["/abs.txt"],
        &["signature.sig"],
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    let (registry, install_root, _record_root) = registry();
    let destination = install_root.join(&record.installation_relative_path);
    fs::create_dir(&destination).unwrap();
    write_payload(&destination, &record.plug_json.path, plug_json_bytes);
    let abs_path = std::path::Path::new("/abs.txt");
    if let Some(parent) = abs_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&abs_path, payload_bytes).unwrap();
    record.payloads[0].sha256 = sha256(payload_bytes);
    record.payloads[0].size_bytes = payload_bytes.len() as u64;
    let mut covered = record.clone();
    covered.record_digest.clear();
    record.record_digest = sha256(&canonical(&covered).unwrap());
    let intent = InstallationPublicationIntent::from_precomputed_record(record.clone()).unwrap();
    let err = registry
        .verify_installation_recovery_destination(&intent)
        .unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
    drop(record);
}

#[test]
fn j24k3c2_unsafe_expected_current_dir_component_fail_closed() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let mut record = build_valid_record_with_payloads(
        &["./manifest.json"],
        &["signature.sig"],
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    let (registry, install_root, _record_root) = registry();
    let destination = install_root.join(&record.installation_relative_path);
    fs::create_dir(&destination).unwrap();
    write_payload(&destination, &record.plug_json.path, plug_json_bytes);
    record.payloads[0].sha256 = sha256(payload_bytes);
    record.payloads[0].size_bytes = payload_bytes.len() as u64;
    let mut covered = record.clone();
    covered.record_digest.clear();
    record.record_digest = sha256(&canonical(&covered).unwrap());
    let intent = InstallationPublicationIntent::from_precomputed_record(record.clone()).unwrap();
    let err = registry
        .verify_installation_recovery_destination(&intent)
        .unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
    drop(record);
}

#[test]
fn j24k3c2_unsafe_expected_separator_ambiguity_fail_closed() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let mut record = build_valid_record_with_payloads(
        &["foo//bar.txt"],
        &["signature.sig"],
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    let (registry, install_root, _record_root) = registry();
    let destination = install_root.join(&record.installation_relative_path);
    fs::create_dir(&destination).unwrap();
    write_payload(&destination, &record.plug_json.path, plug_json_bytes);
    record.payloads[0].sha256 = sha256(payload_bytes);
    record.payloads[0].size_bytes = payload_bytes.len() as u64;
    let mut covered = record.clone();
    covered.record_digest.clear();
    record.record_digest = sha256(&canonical(&covered).unwrap());
    let intent = InstallationPublicationIntent::from_precomputed_record(record.clone()).unwrap();
    let err = registry
        .verify_installation_recovery_destination(&intent)
        .unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
    drop(record);
}

#[test]
fn j24k3c2_unsafe_expected_backslash_separator_fail_closed() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let mut record = build_valid_record_with_payloads(
        &["foo\\bar.txt"],
        &["signature.sig"],
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    let (registry, install_root, _record_root) = registry();
    let destination = install_root.join(&record.installation_relative_path);
    fs::create_dir(&destination).unwrap();
    write_payload(&destination, &record.plug_json.path, plug_json_bytes);
    record.payloads[0].sha256 = sha256(payload_bytes);
    record.payloads[0].size_bytes = payload_bytes.len() as u64;
    let mut covered = record.clone();
    covered.record_digest.clear();
    record.record_digest = sha256(&canonical(&covered).unwrap());
    let intent = InstallationPublicationIntent::from_precomputed_record(record.clone()).unwrap();
    let err = registry
        .verify_installation_recovery_destination(&intent)
        .unwrap_err();
    assert_eq!(err.code, "installation_recovery_conflict");
    drop(record);
}

#[test]
fn j24k3c2_unrelated_sibling_destinations_untouched_and_not_scanned() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let record = build_valid_record_with_payloads(
        &["manifest.json"],
        &["signature.sig"],
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    let (registry, install_root, _record_root) = registry();
    build_destination_for_record(
        &install_root,
        &record,
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    let sibling = install_root.join("plug-00000000-0000-0000-0000-000000000000");
    fs::create_dir(&sibling).unwrap();
    write_payload(&sibling, "unexpected.txt", b"unexpected");
    let intent = InstallationPublicationIntent::from_precomputed_record(record.clone()).unwrap();
    registry
        .verify_installation_recovery_destination(&intent)
        .unwrap();
    assert!(sibling.exists());
    drop(record);
}

#[test]
fn j24k3c2_verification_leaves_entries_bytes_timestamps_permissions_unchanged() {
    let plug_json_bytes = b"{\"plug\":true}";
    let payload_bytes = b"payload";
    let signature_bytes = b"signature";
    let record = build_valid_record_with_payloads(
        &["manifest.json"],
        &["signature.sig"],
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    let (registry, install_root, _record_root) = registry();
    build_destination_for_record(
        &install_root,
        &record,
        plug_json_bytes,
        payload_bytes,
        signature_bytes,
    );
    let destination = install_root.join(&record.installation_relative_path);
    let manifest = destination.join("manifest.json");
    let sig = destination.join("signature.sig");
    let manifest_meta_before = fs::symlink_metadata(&manifest).unwrap();
    let sig_meta_before = fs::symlink_metadata(&sig).unwrap();
    let manifest_bytes_before = fs::read(&manifest).unwrap();
    let sig_bytes_before = fs::read(&sig).unwrap();

    let intent = InstallationPublicationIntent::from_precomputed_record(record.clone()).unwrap();
    registry
        .verify_installation_recovery_destination(&intent)
        .unwrap();

    let manifest_meta_after = fs::symlink_metadata(&manifest).unwrap();
    let sig_meta_after = fs::symlink_metadata(&sig).unwrap();
    let manifest_bytes_after = fs::read(&manifest).unwrap();
    let sig_bytes_after = fs::read(&sig).unwrap();
    assert_eq!(
        manifest_meta_before.modified().unwrap(),
        manifest_meta_after.modified().unwrap()
    );
    assert_eq!(
        sig_meta_before.modified().unwrap(),
        sig_meta_after.modified().unwrap()
    );
    assert_eq!(
        manifest_meta_before.permissions(),
        manifest_meta_after.permissions()
    );
    assert_eq!(sig_meta_before.permissions(), sig_meta_after.permissions());
    assert_eq!(manifest_bytes_before, manifest_bytes_after);
    assert_eq!(sig_bytes_before, sig_bytes_after);
    drop(record);
}

#[test]
fn j24k3c2_invalid_intent_rejected_before_destination_inspection() {
    let (record, mut intent) = valid_intent();
    intent.schema_version = 0;
    let (registry, install_root, _record_root) = registry();
    fs::create_dir(install_root.join(&intent.destination_relative_path)).unwrap();
    let err = registry
        .verify_installation_recovery_destination(&intent)
        .unwrap_err();
    assert_eq!(err.code, "installation_intent_invalid");
    drop(record);
}

#[test]
fn j24k3c2_missing_install_root_returns_recovery_io() {
    let (record, intent) = valid_intent();
    let (registry, install_root, _record_root) = registry();
    fs::remove_dir_all(&install_root).unwrap();
    let err = registry
        .verify_installation_recovery_destination(&intent)
        .unwrap_err();
    assert_eq!(err.code, "installation_recovery_io");
    drop(record);
}
