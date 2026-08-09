use std::fs;
use tethers_reference_host::enablement::{EnablementState, EnablementStore};
use tethers_reference_host::installed::{DisabledBindingRecord, InstalledPlugRecord};
use tethers_reference_host::operational_scope::OperationalScopeEvidence;
use tethers_reference_host::trust::{PackageTrustEvidence, TrustModeEvidence};
use uuid::Uuid;

fn local_sha256(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("sha256:{:x}", Sha256::digest(data))
}

fn local_canonical<T: serde::Serialize>(value: &T) -> Vec<u8> {
    serde_json_canonicalizer::to_vec(value).expect("canonical serialization")
}

fn digest() -> String {
    let zeros = "a".repeat(64);
    format!("sha256:{zeros}")
}

fn scope_digest() -> String {
    let ees = "e".repeat(64);
    format!("sha256:{ees}")
}

fn pdf_scope_evidence(installed_id: &str, provider_id: &str) -> OperationalScopeEvidence {
    OperationalScopeEvidence::create(
        installed_id,
        "tethers.pdf-tools",
        provider_id,
        &scope_digest(),
        &serde_json::json!({"query_root": "/tmp/pdf-query", "max_bytes": 65536}),
        "Matthew",
    )
    .unwrap()
}

fn ft_scope_evidence(installed_id: &str, provider_id: &str) -> OperationalScopeEvidence {
    OperationalScopeEvidence::create(
        installed_id,
        "tethers.file-tools",
        provider_id,
        &scope_digest(),
        &serde_json::json!({
            "query_root": "/tmp/ft-query",
            "move_source_root": "/tmp/ft-source",
            "move_destination_root": "/tmp/ft-dest",
            "max_content_bytes": 65536,
        }),
        "Matthew",
    )
    .unwrap()
}

// 1. OperationalScopeEvidence::create produces correct values.
#[test]
fn evidence_create_produces_correct_identity() {
    let evidence = pdf_scope_evidence("inst-1", "tethers-pdf-provider");
    assert_eq!(evidence.schema_version, 1);
    assert_eq!(evidence.installed_id(), "inst-1");
    assert_eq!(evidence.package_identity, "tethers.pdf-tools");
    assert_eq!(evidence.provider_identity, "tethers-pdf-provider");
    assert!(evidence.integrity_digest.starts_with("sha256:"));
    assert_eq!(evidence.integrity_digest().len(), 71);
    assert_eq!(evidence.authority, "Matthew");
    evidence.validate().unwrap();
}

// 2. Digest is deterministic.
#[test]
fn evidence_digest_deterministic() {
    let a = pdf_scope_evidence("inst-2", "tethers-pdf-provider");
    let b = pdf_scope_evidence("inst-2", "tethers-pdf-provider");
    assert_eq!(a.integrity_digest, b.integrity_digest);
    assert_eq!(a.canonical_scope_json, b.canonical_scope_json);
}

// 3. Different installed_id yields different digest.
#[test]
fn different_installed_id_yields_different_digest() {
    let a = pdf_scope_evidence("inst-a", "tethers-pdf-provider");
    let b = pdf_scope_evidence("inst-b", "tethers-pdf-provider");
    assert_ne!(a.integrity_digest, b.integrity_digest);
}

// 4. Empty fields rejected.
#[test]
fn empty_installed_id_rejected() {
    let result = OperationalScopeEvidence::create(
        "",
        "pkg",
        "prv",
        &scope_digest(),
        &serde_json::json!({"key": "val"}),
        "auth",
    );
    assert!(result.is_err());
}

#[test]
fn empty_authority_rejected() {
    let result = OperationalScopeEvidence::create(
        "inst",
        "pkg",
        "prv",
        &scope_digest(),
        &serde_json::json!({"key": "val"}),
        "",
    );
    assert!(result.is_err());
}

#[test]
fn short_scope_schema_digest_rejected() {
    let result = OperationalScopeEvidence::create(
        "inst",
        "pkg",
        "prv",
        "too-short",
        &serde_json::json!({"key": "val"}),
        "auth",
    );
    assert!(result.is_err());
}

// 5. Tamper detection.
#[test]
fn tampered_integrity_digest_fails_validation() {
    let mut evidence = pdf_scope_evidence("tamper-1", "tethers-pdf-provider");
    evidence.integrity_digest = format!("sha256:{}", "b".repeat(64));
    assert!(evidence.validate().is_err());
}

#[test]
fn tampered_canonical_json_fails_validation() {
    let mut evidence = pdf_scope_evidence("tamper-2", "tethers-pdf-provider");
    evidence.canonical_scope_json.push('x');
    assert!(evidence.validate().is_err());
}

// 6. Serialize-deserialize round-trip.
#[test]
fn round_trip_preserves_equality() {
    let evidence = pdf_scope_evidence("rt-1", "tethers-pdf-provider");
    let json = serde_json::to_string(&evidence).unwrap();
    let parsed: OperationalScopeEvidence = serde_json::from_str(&json).unwrap();
    assert_eq!(evidence, parsed);
}

// 7. No variant tag in serialized JSON.
#[test]
fn serialized_evidence_has_no_variant_tag() {
    let evidence = pdf_scope_evidence("ser-1", "tethers-pdf-provider");
    let json_str = serde_json::to_string(&evidence).unwrap();
    let obj: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let map = obj.as_object().unwrap();
    assert!(!map.contains_key("variant"));
    assert!(!map.contains_key("type"));
    assert!(!map.contains_key("scope_kind"));
    assert!(map.contains_key("installed_identity"));
    assert!(map.contains_key("integrity_digest"));
    assert!(map.contains_key("canonical_scope_json"));
}

// 8. File Tools scope evidence works.
#[test]
fn file_tools_scope_evidence_created() {
    let evidence = ft_scope_evidence("ft-1", "tethers-file-tools");
    assert_eq!(evidence.installed_id(), "ft-1");
    assert_eq!(evidence.package_identity, "tethers.file-tools");
    assert_eq!(evidence.provider_identity, "tethers-file-tools");
    evidence.validate().unwrap();
}

// 9. canonical_scope method round-trips.
#[test]
fn canonical_scope_round_trips() {
    let evidence = pdf_scope_evidence("cs-1", "tethers-pdf-provider");
    let scope = evidence.canonical_scope().unwrap();
    let obj = scope.as_object().unwrap();
    assert_eq!(obj["query_root"], "/tmp/pdf-query");
    assert_eq!(obj["max_bytes"], 65536);
}

fn installed_plug_for_pdf() -> InstalledPlugRecord {
    let d = digest();
    let mut installed = InstalledPlugRecord {
        schema_version: 1,
        installed_id: Uuid::new_v4().to_string(),
        state: "present_disabled".into(),
        package_id: "tethers.pdf-tools".into(),
        package_version: "1.0.0".into(),
        semantic_package_digest: d.clone(),
        source_candidate_id: "candidate".into(),
        installation_relative_path: "plug".into(),
        raw_archive_digest: d.clone(),
        plug_json: tethers_reference_host::package::PayloadEvidence {
            path: "plug.json".into(),
            sha256: d.clone(),
            size_bytes: 1,
            role: "package_descriptor".into(),
        },
        payloads: Vec::new(),
        signature_files: Vec::new(),
        capability_manifests: Vec::new(),
        trust_evidence: {
            let mut trust = tethers_reference_host::trust::PackageTrustEvidence {
                evidence_format_version: 1,
                semantic_package_digest: d.clone(),
                mode: TrustModeEvidence::UnsignedDeveloper {
                    approval_id: "approval".into(),
                    approval_record_digest: d.clone(),
                    visibly_unsigned: true,
                },
                evidence_digest: String::new(),
            };
            let mut covered = trust.clone();
            covered.evidence_digest.clear();
            trust.evidence_digest = local_sha256(&local_canonical(&covered));
            trust
        },
        installation_approval_id: "approval".into(),
        installation_approval_digest: d.clone(),
        conformance_evidence_id: "conformance".into(),
        conformance_evidence_digest: d.clone(),
        provider_id: "tethers-pdf-provider".into(),
        provider_version: "1.0.0".into(),
        launch_path: "provider/pdf_tools_provider.exe".into(),
        launch_arguments: Vec::new(),
        provider_working_directory: "provider".into(),
        launch_profile_label: "supervised".into(),
        socket_major: 1,
        mcp_protocol_version: "2025-11-25".into(),
        platform: "windows".into(),
        architecture: "x86_64".into(),
        disabled_bindings: vec![DisabledBindingRecord {
            state: "disabled".into(),
            capability_name: "pdf.inspect".into(),
            capability_version: 1,
            manifest_digest: d.clone(),
            provider_operation_name: "pdf_inspect".into(),
        }],
        created_unix_ms: 1,
        record_digest: String::new(),
    };
    let mut covered = installed.clone();
    covered.record_digest.clear();
    installed.record_digest = local_sha256(&local_canonical(&covered));
    installed
}

fn installed_plug_for_ft(installed_id: &str) -> InstalledPlugRecord {
    let d = digest();
    let mut installed = InstalledPlugRecord {
        schema_version: 1,
        installed_id: installed_id.to_owned(),
        state: "present_disabled".into(),
        package_id: "tethers.file-tools".into(),
        package_version: "1.1.0".into(),
        semantic_package_digest: d.clone(),
        source_candidate_id: "candidate".into(),
        installation_relative_path: "plug".into(),
        raw_archive_digest: d.clone(),
        plug_json: tethers_reference_host::package::PayloadEvidence {
            path: "plug.json".into(),
            sha256: d.clone(),
            size_bytes: 1,
            role: "package_descriptor".into(),
        },
        payloads: Vec::new(),
        signature_files: Vec::new(),
        capability_manifests: Vec::new(),
        trust_evidence: {
            let mut trust = tethers_reference_host::trust::PackageTrustEvidence {
                evidence_format_version: 1,
                semantic_package_digest: d.clone(),
                mode: TrustModeEvidence::UnsignedDeveloper {
                    approval_id: "approval".into(),
                    approval_record_digest: d.clone(),
                    visibly_unsigned: true,
                },
                evidence_digest: String::new(),
            };
            let mut covered = trust.clone();
            covered.evidence_digest.clear();
            trust.evidence_digest = local_sha256(&local_canonical(&covered));
            trust
        },
        installation_approval_id: "approval".into(),
        installation_approval_digest: d.clone(),
        conformance_evidence_id: "conformance".into(),
        conformance_evidence_digest: d.clone(),
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
            manifest_digest: d.clone(),
            provider_operation_name: "file_move".into(),
        }],
        created_unix_ms: 1,
        record_digest: String::new(),
    };
    let mut covered = installed.clone();
    covered.record_digest.clear();
    installed.record_digest = local_sha256(&local_canonical(&covered));
    installed
}

// 10. EnablementStore::enable accepts generic scope evidence.
#[test]
fn enable_accepts_generic_pdf_evidence() {
    let installed = installed_plug_for_pdf();
    let iid = installed.installed_id.clone();
    let temp = std::env::temp_dir().join(format!("tethers-j23c1-pdf-enable-{}", Uuid::new_v4()));
    let enablement_root = temp.join("enablement");
    let scope = pdf_scope_evidence(&iid, "tethers-pdf-provider");
    let store = EnablementStore::open(&enablement_root).unwrap();
    let record = store.enable(&installed, scope, "Matthew").unwrap();
    assert_eq!(record.state, EnablementState::Enabled);
    assert_eq!(record.package_id, "tethers.pdf-tools");
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn enablement_record_has_correct_fields() {
    let installed = installed_plug_for_pdf();
    let iid = installed.installed_id.clone();
    let temp = std::env::temp_dir().join(format!("tethers-j23c1-pdf-record-{}", Uuid::new_v4()));
    let enablement_root = temp.join("enablement");
    let scope = pdf_scope_evidence(&iid, "tethers-pdf-provider");
    let store = EnablementStore::open(&enablement_root).unwrap();
    let record = store.enable(&installed, scope, "Matthew").unwrap();
    assert_eq!(record.package_id, "tethers.pdf-tools");
    assert_eq!(record.provider_id, "tethers-pdf-provider");
    assert_eq!(record.operational_scope.schema_version, 1);
    assert!(record.operational_scope_digest.starts_with("sha256:"));
    assert_eq!(
        record.operational_scope_digest,
        record.operational_scope.integrity_digest()
    );
    let has_capability = record
        .capabilities
        .iter()
        .any(|c| c.name == "pdf.inspect" && c.version == 1);
    assert!(has_capability);
    assert_eq!(record.state, EnablementState::Enabled);
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn enable_accepts_file_tools_evidence() {
    let ft_id = Uuid::new_v4().to_string();
    let installed = installed_plug_for_ft(&ft_id);
    let temp = std::env::temp_dir().join(format!("tethers-j23c1-ft-enable-{}", Uuid::new_v4()));
    let enablement_root = temp.join("enablement");
    let scope = ft_scope_evidence(&ft_id, "tethers-file-tools");
    let store = EnablementStore::open(&enablement_root).unwrap();
    let record = store.enable(&installed, scope, "Matthew").unwrap();
    assert_eq!(record.state, EnablementState::Enabled);
    assert_eq!(record.package_id, "tethers.file-tools");
    store.disable(&installed, "Matthew").unwrap();
    assert!(!store.is_available(&ft_id).unwrap());
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn wrong_installed_id_is_refused_by_enablement() {
    let installed = installed_plug_for_pdf();
    let temp = std::env::temp_dir().join(format!("tethers-j23c1-wrong-id-{}", Uuid::new_v4()));
    let enablement_root = temp.join("enablement");
    let scope = pdf_scope_evidence("wrong-id", "tethers-pdf-provider");
    let store = EnablementStore::open(&enablement_root).unwrap();
    let result = store.enable(&installed, scope, "Matthew");
    assert!(result.is_err());
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn disablement_works_for_pdf() {
    let installed = installed_plug_for_pdf();
    let iid = installed.installed_id.clone();
    let temp = std::env::temp_dir().join(format!("tethers-j23c1-disable-pdf-{}", Uuid::new_v4()));
    let enablement_root = temp.join("enablement");
    let scope = pdf_scope_evidence(&iid, "tethers-pdf-provider");
    let store = EnablementStore::open(&enablement_root).unwrap();
    store.enable(&installed, scope, "Matthew").unwrap();
    assert!(store.is_available(&iid).unwrap());
    store.disable(&installed, "Matthew").unwrap();
    assert!(!store.is_available(&iid).unwrap());
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn disablement_works_for_file_tools() {
    let ft_id = Uuid::new_v4().to_string();
    let installed = installed_plug_for_ft(&ft_id);
    let temp = std::env::temp_dir().join(format!("tethers-j23c1-disable-ft-{}", Uuid::new_v4()));
    let enablement_root = temp.join("enablement");
    let scope = ft_scope_evidence(&ft_id, "tethers-file-tools");
    let store = EnablementStore::open(&enablement_root).unwrap();
    store.enable(&installed, scope, "Matthew").unwrap();
    assert!(store.is_available(&ft_id).unwrap());
    store.disable(&installed, "Matthew").unwrap();
    assert!(!store.is_available(&ft_id).unwrap());
    fs::remove_dir_all(temp).unwrap();
}
