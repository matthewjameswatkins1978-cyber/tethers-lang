use crate::installation_publication_intent::InstallationPublicationIntent;
use crate::installation_recovery::{
    classify_installation_recovery, InstallationRecoveryDisposition,
    InstallationRecoveryObservation,
};
use crate::installed::{DisabledBindingRecord, InstalledPlugRecord};
use crate::m3_store::{canonical, sha256};
use crate::package::PayloadEvidence;
use crate::trust::{PackageTrustEvidence, TrustModeEvidence};
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

fn valid_observation<'a>(
    intent: &'a InstallationPublicationIntent,
    staging: bool,
    destination: bool,
    record: Option<&'a InstalledPlugRecord>,
) -> InstallationRecoveryObservation<'a> {
    InstallationRecoveryObservation {
        intent,
        staging_present: staging,
        destination_present: destination,
        installed_record: record,
    }
}

#[test]
fn j24k3b_intent_only_returns_remove_intent_only() {
    let (record, intent) = valid_intent();
    let obs = valid_observation(&intent, false, false, None);
    assert_eq!(
        classify_installation_recovery(obs).unwrap(),
        InstallationRecoveryDisposition::RemoveIntentOnly
    );
    drop(record);
}

#[test]
fn j24k3b_intent_plus_staging_only_returns_remove_staging_then_intent() {
    let (record, intent) = valid_intent();
    let obs = valid_observation(&intent, true, false, None);
    assert_eq!(
        classify_installation_recovery(obs).unwrap(),
        InstallationRecoveryDisposition::RemoveStagingThenIntent
    );
    drop(record);
}

#[test]
fn j24k3b_intent_plus_destination_only_returns_revalidate_then_publish() {
    let (record, intent) = valid_intent();
    let obs = valid_observation(&intent, false, true, None);
    assert_eq!(
        classify_installation_recovery(obs).unwrap(),
        InstallationRecoveryDisposition::RevalidateDestinationThenPublishRecord
    );
    drop(record);
}

#[test]
fn j24k3b_intent_plus_destination_plus_matching_record_returns_verify_then_remove() {
    let (record, intent) = valid_intent();
    let obs = valid_observation(&intent, false, true, Some(&record));
    assert_eq!(
        classify_installation_recovery(obs).unwrap(),
        InstallationRecoveryDisposition::VerifyCompletedPublicationThenRemoveIntent
    );
    drop(record);
}

#[test]
fn j24k3b_record_without_destination_without_staging_conflicts() {
    let (record, intent) = valid_intent();
    let obs = valid_observation(&intent, false, false, Some(&record));
    assert_eq!(
        classify_installation_recovery(obs).unwrap_err().code,
        "installation_recovery_conflict"
    );
    drop(record);
}

#[test]
fn j24k3b_staging_plus_record_without_destination_conflicts() {
    let (record, intent) = valid_intent();
    let obs = valid_observation(&intent, true, false, Some(&record));
    assert_eq!(
        classify_installation_recovery(obs).unwrap_err().code,
        "installation_recovery_conflict"
    );
    drop(record);
}

#[test]
fn j24k3b_staging_plus_destination_without_record_conflicts() {
    let (record, intent) = valid_intent();
    let obs = valid_observation(&intent, true, true, None);
    assert_eq!(
        classify_installation_recovery(obs).unwrap_err().code,
        "installation_recovery_conflict"
    );
    drop(record);
}

#[test]
fn j24k3b_staging_plus_destination_plus_matching_record_conflicts() {
    let (record, intent) = valid_intent();
    let obs = valid_observation(&intent, true, true, Some(&record));
    assert_eq!(
        classify_installation_recovery(obs).unwrap_err().code,
        "installation_recovery_conflict"
    );
    drop(record);
}

#[test]
fn j24k3b_destination_plus_different_valid_record_conflicts() {
    let (record_a, intent) = valid_intent();
    let record_b = valid_record();
    assert_ne!(record_a, record_b);
    let obs = valid_observation(&intent, false, true, Some(&record_b));
    assert_eq!(
        classify_installation_recovery(obs).unwrap_err().code,
        "installation_recovery_conflict"
    );
    drop(record_a);
    drop(record_b);
}

#[test]
fn j24k3b_destination_plus_invalid_record_conflicts() {
    let (record, intent) = valid_intent();
    let mut invalid_record = record.clone();
    invalid_record.schema_version = 0;
    assert_ne!(invalid_record, record);
    let obs = valid_observation(&intent, false, true, Some(&invalid_record));
    assert_eq!(
        classify_installation_recovery(obs).unwrap_err().code,
        "installation_recovery_conflict"
    );
    drop(record);
}

#[test]
fn j24k3b_invalid_intent_reported_before_state_classification() {
    let (record, intent) = valid_intent();
    let mut invalid_intent = intent.clone();
    invalid_intent.schema_version = 0;
    assert_ne!(invalid_intent, intent);
    let obs = valid_observation(&invalid_intent, true, true, None);
    assert_eq!(
        classify_installation_recovery(obs).unwrap_err().code,
        "installation_intent_invalid"
    );
    drop(record);
}

#[test]
fn j24k3b_same_installed_id_different_fields_still_conflicts() {
    let record_a = valid_record();
    let same_id = record_a.installed_id.clone();
    let package_version = record_a.package_version.clone();
    let intent = InstallationPublicationIntent::from_precomputed_record(record_a.clone()).unwrap();

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
    let mut record_b = InstalledPlugRecord {
        schema_version: 1,
        installed_id: same_id.clone(),
        state: "present_disabled".into(),
        package_id: "tethers.file-tools".into(),
        package_version: "2.0.0".into(),
        semantic_package_digest: digest.into(),
        source_candidate_id: "candidate".into(),
        installation_relative_path: format!("plug-{same_id}"),
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
        operational_scope_schema: None,
        operational_scope_schema_digest: None,
        created_unix_ms: 1,
        record_digest: String::new(),
    };
    let mut covered = record_b.clone();
    covered.record_digest.clear();
    record_b.record_digest = sha256(&canonical(&covered).unwrap());
    record_b.validate().unwrap();
    assert_eq!(record_b.installed_id, record_a.installed_id);
    assert_ne!(record_b.package_version, package_version);
    assert_ne!(record_b, record_a);

    let obs = valid_observation(&intent, false, true, Some(&record_b));
    assert_eq!(
        classify_installation_recovery(obs).unwrap_err().code,
        "installation_recovery_conflict"
    );
    drop(record_a);
}

#[test]
fn j24k3b_invalid_record_without_destination_conflicts() {
    let (record, intent) = valid_intent();
    let mut invalid_record = record.clone();
    invalid_record.schema_version = 0;
    let obs = valid_observation(&intent, false, false, Some(&invalid_record));
    assert_eq!(
        classify_installation_recovery(obs).unwrap_err().code,
        "installation_recovery_conflict"
    );
    drop(record);
}

#[test]
fn j24k3b_staging_plus_destination_plus_invalid_record_conflicts() {
    let (record, intent) = valid_intent();
    let mut invalid_record = record.clone();
    invalid_record.schema_version = 0;
    let obs = valid_observation(&intent, true, true, Some(&invalid_record));
    assert_eq!(
        classify_installation_recovery(obs).unwrap_err().code,
        "installation_recovery_conflict"
    );
    drop(record);
}

#[test]
fn j24k3b_classification_does_not_alter_supplied_intent_or_record() {
    let (record, intent) = valid_intent();
    let intent_clone = intent.clone();
    let record_clone = record.clone();
    let obs = valid_observation(&intent, false, true, Some(&record));
    classify_installation_recovery(obs).unwrap();
    assert_eq!(intent, intent_clone);
    assert_eq!(record, record_clone);
    drop(record);
}

#[test]
fn j24k3b_repeated_classification_is_deterministic() {
    let (record, intent) = valid_intent();
    let obs_a = valid_observation(&intent, false, false, None);
    let obs_b = valid_observation(&intent, false, false, None);
    assert_eq!(
        classify_installation_recovery(obs_a).unwrap(),
        classify_installation_recovery(obs_b).unwrap()
    );
    let obs_a = valid_observation(&intent, false, true, Some(&record));
    let obs_b = valid_observation(&intent, false, true, Some(&record));
    assert_eq!(
        classify_installation_recovery(obs_a).unwrap(),
        classify_installation_recovery(obs_b).unwrap()
    );
    drop(record);
}
