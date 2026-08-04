use crate::installation_publication_intent::{
    InstallationPublicationIntent, InstallationPublicationIntentStore,
};
use crate::installed::{DisabledBindingRecord, InstalledPlugRecord};
use crate::m3_store::{canonical, sha256};
use crate::package::PayloadEvidence;
use crate::trust::{PackageTrustEvidence, TrustModeEvidence};
use std::fs;
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

fn store() -> (
    InstallationPublicationIntentStore,
    InstallationPublicationIntent,
    std::path::PathBuf,
) {
    let root = std::env::temp_dir().join(format!("tethers-j24k3a-{}", Uuid::new_v4()));
    let intent = InstallationPublicationIntent::from_precomputed_record(valid_record()).unwrap();
    let store = InstallationPublicationIntentStore::open(&root).unwrap();
    (store, intent, root)
}

fn write_current(root: &std::path::Path, value: &serde_json::Value) {
    fs::write(
        root.join("installation-intent/current.json"),
        canonical(value).unwrap(),
    )
    .unwrap();
}

#[test]
fn j24k3a_construction_preserves_precomputed_identity_digest_and_timestamp() {
    let record = valid_record();
    let id = record.installed_id.clone();
    let record_digest = record.record_digest.clone();
    let timestamp = record.created_unix_ms;
    let intent = InstallationPublicationIntent::from_precomputed_record(record.clone()).unwrap();
    assert_eq!(intent.transaction_id, id);
    assert_eq!(intent.installed_record, record);
    assert_eq!(intent.installed_record_digest, record_digest);
    assert_eq!(intent.installed_record.created_unix_ms, timestamp);
}

#[test]
fn j24k3a_create_and_load_round_trip_is_atomic_and_private() {
    let (store, intent, root) = store();
    store.create(&intent).unwrap();
    assert_eq!(store.load().unwrap(), Some(intent.clone()));
    assert!(!root.join("installation-intent/.current.tmp").exists());
    assert_eq!(
        store.create(&intent).unwrap_err().code,
        "installation_intent_conflict"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn j24k3a_exact_removal_and_absent_removal() {
    let (store, intent, root) = store();
    assert!(!store.remove_if_matches(&intent).unwrap());
    store.create(&intent).unwrap();
    assert!(store.remove_if_matches(&intent).unwrap());
    assert!(!store.remove_if_matches(&intent).unwrap());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn j24k3a_mismatched_removal_preserves_original_bytes() {
    let (store, intent, root) = store();
    store.create(&intent).unwrap();
    let bytes = fs::read(root.join("installation-intent/current.json")).unwrap();
    let mut other = intent.clone();
    other.candidate_id = "other".into();
    assert_eq!(
        store.remove_if_matches(&other).unwrap_err().code,
        "installation_intent_invalid"
    );
    assert_eq!(
        fs::read(root.join("installation-intent/current.json")).unwrap(),
        bytes
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn j24k3a_every_intent_field_is_digest_covered() {
    let (store, intent, root) = store();
    store.create(&intent).unwrap();
    let fields = [
        "transaction_id",
        "candidate_id",
        "destination_relative_path",
        "installed_record_digest",
    ];
    for field in fields {
        let mut value: serde_json::Value = serde_json::from_slice(
            &fs::read(root.join("installation-intent/current.json")).unwrap(),
        )
        .unwrap();
        value[field] = serde_json::Value::String("changed".into());
        write_current(&root, &value);
        assert_eq!(
            store.load().unwrap_err().code,
            "installation_intent_invalid"
        );
        fs::write(
            root.join("installation-intent/current.json"),
            canonical(&intent).unwrap(),
        )
        .unwrap();
    }
    let mut value: serde_json::Value =
        serde_json::from_slice(&canonical(&intent).unwrap()).unwrap();
    value["installed_record"]["created_unix_ms"] = serde_json::Value::from(2);
    write_current(&root, &value);
    assert_eq!(
        store.load().unwrap_err().code,
        "installation_intent_invalid"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn j24k3a_destination_identity_is_exact_and_record_drift_fails() {
    let mut record = valid_record();
    record.installation_relative_path = format!("./{}", record.installation_relative_path);
    let mut covered = record.clone();
    covered.record_digest.clear();
    record.record_digest = sha256(&canonical(&covered).unwrap());
    assert_eq!(
        InstallationPublicationIntent::from_precomputed_record(record)
            .unwrap_err()
            .code,
        "installation_intent_invalid"
    );

    let (store, intent, root) = store();
    store.create(&intent).unwrap();
    let mut value: serde_json::Value =
        serde_json::from_slice(&canonical(&intent).unwrap()).unwrap();
    value["installed_record"]["package_version"] = serde_json::Value::String("drifted".into());
    write_current(&root, &value);
    assert_eq!(
        store.load().unwrap_err().code,
        "installation_intent_invalid"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn j24k3a_malformed_duplicate_unknown_and_unsafe_entries_fail_closed() {
    let (store, _intent, root) = store();
    fs::write(root.join("installation-intent/current.json"), b"{").unwrap();
    assert_eq!(
        store.load().unwrap_err().code,
        "installation_intent_invalid"
    );
    fs::write(
        root.join("installation-intent/current.json"),
        br#"{"schema_version":1,"schema_version":1}"#,
    )
    .unwrap();
    assert_eq!(
        store.load().unwrap_err().code,
        "installation_intent_invalid"
    );
    fs::write(
        root.join("installation-intent/current.json"),
        br#"{"unknown":1}"#,
    )
    .unwrap();
    assert_eq!(
        store.load().unwrap_err().code,
        "installation_intent_invalid"
    );
    fs::remove_file(root.join("installation-intent/current.json")).unwrap();
    fs::write(root.join("installation-intent/.current.tmp"), b"torn").unwrap();
    assert_eq!(
        store.load().unwrap_err().code,
        "installation_intent_invalid"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn j24k3a_unknown_directory_and_multiple_entries_fail_closed() {
    let (store, _intent, root) = store();
    fs::create_dir(root.join("installation-intent/unknown")).unwrap();
    assert_eq!(
        store.load().unwrap_err().code,
        "installation_intent_invalid"
    );
    fs::remove_dir(root.join("installation-intent/unknown")).unwrap();
    fs::write(root.join("installation-intent/other.json"), b"{}").unwrap();
    assert_eq!(
        store.load().unwrap_err().code,
        "installation_intent_invalid"
    );
    fs::remove_dir_all(root).unwrap();
}
