use std::fs;
use tethers_reference_host::enablement::{EnablementState, EnablementStore};
use tethers_reference_host::installed::{DisabledBindingRecord, InstalledPlugRecord};
use tethers_reference_host::operational_scope::OperationalScope;
use tethers_reference_host::trust::{PackageTrustEvidence, TrustModeEvidence};
use uuid::Uuid;

fn digest() -> String {
    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into()
}

fn trust_evidence(digest: &str) -> PackageTrustEvidence {
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
    let mut covered = trust.clone();
    covered.evidence_digest.clear();
    trust.evidence_digest = tethers_reference_host::m3_store::sha256(
        &tethers_reference_host::m3_store::canonical(&covered).unwrap(),
    );
    trust
}

fn pdf_installed_plug() -> InstalledPlugRecord {
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
        trust_evidence: trust_evidence(&d),
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
    installed.record_digest = tethers_reference_host::m3_store::sha256(
        &tethers_reference_host::m3_store::canonical(&covered).unwrap(),
    );
    installed
}

fn pdf_scope(
    installed_id: &str,
    query_root: &std::path::Path,
) -> tethers_reference_host::pdf_tools::PdfOperationalScopeBinding {
    tethers_reference_host::pdf_tools::PdfOperationalScopeBinding::create(
        installed_id,
        query_root,
        64 * 1024,
        "Matthew",
    )
    .unwrap()
}

// 1. PdfOperationalScopeBinding::create produces correct values.
#[test]
fn pdf_scope_create_produces_correct_identity() {
    let root = std::env::temp_dir().join(format!("tethers-j23c1-create-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let binding = pdf_scope("inst-1", &root);
    assert_eq!(binding.capability_name, "pdf.inspect");
    assert_eq!(binding.capability_version, 1);
    assert_eq!(binding.query_root, fs::canonicalize(&root).unwrap());
    assert_eq!(binding.max_bytes, 64 * 1024);
    assert!(binding.integrity_digest.starts_with("sha256:"));
    assert_eq!(binding.integrity_digest.len(), 71);
    assert_eq!(binding.authority, "Matthew");
    fs::remove_dir_all(root).unwrap();
}

// 2. An unchanged PDF binding validates.
#[test]
fn unchanged_pdf_binding_validates() {
    let root = std::env::temp_dir().join(format!("tethers-j23c1-valid-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let binding = pdf_scope("inst-2", &root);
    binding.validate().unwrap();
    fs::remove_dir_all(root).unwrap();
}

// 3. Tampering with any field after creation causes validation failure.
#[test]
fn tampered_installed_id_fails_validation() {
    let root = std::env::temp_dir().join(format!("tethers-j23c1-tamper-id-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let mut binding = pdf_scope("inst-3", &root);
    binding.installed_id.clear();
    assert!(binding.validate().is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn tampered_capability_name_fails_validation() {
    let root = std::env::temp_dir().join(format!("tethers-j23c1-tamper-name-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let mut binding = pdf_scope("inst-4", &root);
    binding.capability_name = "wrong".into();
    assert!(binding.validate().is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn tampered_capability_version_fails_validation() {
    let root = std::env::temp_dir().join(format!("tethers-j23c1-tamper-ver-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let mut binding = pdf_scope("inst-5", &root);
    binding.capability_version = 2;
    assert!(binding.validate().is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn tampered_query_root_fails_validation() {
    let root = std::env::temp_dir().join(format!("tethers-j23c1-tamper-root-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let mut binding = pdf_scope("inst-6", &root);
    binding.query_root = root.join("nonexistent");
    assert!(binding.validate().is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn tampered_max_bytes_fails_validation() {
    let root = std::env::temp_dir().join(format!("tethers-j23c1-tamper-bytes-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let mut binding = pdf_scope("inst-7", &root);
    binding.max_bytes = 0;
    assert!(binding.validate().is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn tampered_authority_fails_validation() {
    let root = std::env::temp_dir().join(format!("tethers-j23c1-tamper-auth-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let mut binding = pdf_scope("inst-8", &root);
    binding.authority.clear();
    assert!(binding.validate().is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn tampered_integrity_digest_fails_validation() {
    let root = std::env::temp_dir().join(format!("tethers-j23c1-tamper-digest-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let mut binding = pdf_scope("inst-9", &root);
    binding.integrity_digest.push('x');
    assert!(binding.validate().is_err());
    fs::remove_dir_all(root).unwrap();
}

// 4. PdfOperationalScopeBinding::scope returns a working PdfScope.
#[test]
fn pdf_scope_method_returns_working_scope() {
    let root = std::env::temp_dir().join(format!("tethers-j23c1-scope-method-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let binding = pdf_scope("inst-10", &root);
    let scope = binding.scope().unwrap();
    assert_eq!(scope.query_root, fs::canonicalize(&root).unwrap());
    assert_eq!(scope.max_bytes, 64 * 1024);
    fs::remove_dir_all(root).unwrap();
}

// 5. OperationalScope::Pdf exposes correct common values.
#[test]
fn operational_scope_pdf_exposes_common_values() {
    let root = std::env::temp_dir().join(format!("tethers-j23c1-enum-pdf-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let binding = pdf_scope("inst-11", &root);
    let scope: OperationalScope = binding.into();
    assert_eq!(scope.installed_id(), "inst-11");
    assert_eq!(scope.capability_name(), "pdf.inspect");
    assert_eq!(scope.capability_version(), 1);
    assert!(scope.integrity_digest().starts_with("sha256:"));
    fs::remove_dir_all(root).unwrap();
}

// 6. OperationalScope::FileTools exposes existing File Tools values.
#[test]
fn operational_scope_file_tools_exposes_common_values() {
    let root = std::env::temp_dir().join(format!("tethers-j23c1-ft-scope-{}", Uuid::new_v4()));
    let query = root.join("query");
    let source = root.join("source");
    let dest = root.join("destination");
    fs::create_dir_all(&query).unwrap();
    fs::create_dir_all(&source).unwrap();
    fs::create_dir_all(&dest).unwrap();
    let binding = tethers_reference_host::file_tools::OperationalScopeBinding::create(
        "ft-inst-1",
        "file.move",
        1,
        &query,
        &source,
        &dest,
        tethers_reference_host::file_tools::MAX_CONTENT_BYTES,
        "Matthew",
    )
    .unwrap();
    let scope: OperationalScope = binding.into();
    assert_eq!(scope.installed_id(), "ft-inst-1");
    assert_eq!(scope.capability_name(), "file.move");
    assert_eq!(scope.capability_version(), 1);
    assert!(scope.integrity_digest().starts_with("sha256:"));
    fs::remove_dir_all(root).unwrap();
}

// 7 & 8. Serialized File Tools JSON has no enum tag and matches direct binding.
#[test]
fn file_tools_scope_serializes_without_wrapper() {
    let root = std::env::temp_dir().join(format!("tethers-j23c1-serde-ft-{}", Uuid::new_v4()));
    let query = root.join("query");
    let source = root.join("source");
    let dest = root.join("destination");
    fs::create_dir_all(&query).unwrap();
    fs::create_dir_all(&source).unwrap();
    fs::create_dir_all(&dest).unwrap();
    let binding = tethers_reference_host::file_tools::OperationalScopeBinding::create(
        "serde-1",
        "file.move",
        1,
        &query,
        &source,
        &dest,
        tethers_reference_host::file_tools::MAX_CONTENT_BYTES,
        "Lucy",
    )
    .unwrap();
    let scope: OperationalScope = binding.clone().into();
    let direct_json = serde_json::to_value(&binding).unwrap();
    let enum_json = serde_json::to_value(&scope).unwrap();
    assert_eq!(direct_json, enum_json);
    let obj = enum_json.as_object().unwrap();
    assert!(!obj.contains_key("scope_kind"));
    assert!(!obj.contains_key("type"));
    assert!(!obj.contains_key("variant"));
    assert!(!obj.contains_key("wrapper"));
    assert!(obj.contains_key("installed_id"));
    assert!(obj.contains_key("integrity_digest"));
    fs::remove_dir_all(root).unwrap();
}

// 9. Serialize-deserialize round-trip for both variants.
#[test]
fn round_trip_file_tools_preserves_equality() {
    let root = std::env::temp_dir().join(format!("tethers-j23c1-rt-ft-{}", Uuid::new_v4()));
    let query = root.join("query");
    let source = root.join("source");
    let dest = root.join("destination");
    fs::create_dir_all(&query).unwrap();
    fs::create_dir_all(&source).unwrap();
    fs::create_dir_all(&dest).unwrap();
    let binding = tethers_reference_host::file_tools::OperationalScopeBinding::create(
        "rt-ft-1",
        "file.move",
        1,
        &query,
        &source,
        &dest,
        tethers_reference_host::file_tools::MAX_CONTENT_BYTES,
        "Codex",
    )
    .unwrap();
    let scope: OperationalScope = binding.into();
    let json = serde_json::to_string(&scope).unwrap();
    let parsed: OperationalScope = serde_json::from_str(&json).unwrap();
    assert_eq!(scope, parsed);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn round_trip_pdf_preserves_equality() {
    let root = std::env::temp_dir().join(format!("tethers-j23c1-rt-pdf-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let binding = pdf_scope("rt-pdf-1", &root);
    let scope: OperationalScope = binding.into();
    let json = serde_json::to_string(&scope).unwrap();
    let parsed: OperationalScope = serde_json::from_str(&json).unwrap();
    assert_eq!(scope, parsed);
    fs::remove_dir_all(root).unwrap();
}

// 10. EnablementStore::enable accepts File Tools binding directly.
#[test]
fn enable_accepts_file_tools_binding_directly() {
    let d = digest();
    let temp = std::env::temp_dir().join(format!("tethers-j23c1-ft-enable-{}", Uuid::new_v4()));
    let enablement_root = temp.join("enablement");
    let scope_root = temp.join("scope");
    let query = scope_root.join("query");
    let source = scope_root.join("source");
    let dest = scope_root.join("destination");
    fs::create_dir_all(&query).unwrap();
    fs::create_dir_all(&source).unwrap();
    fs::create_dir_all(&dest).unwrap();
    let mut trust = PackageTrustEvidence {
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
    trust.evidence_digest = tethers_reference_host::m3_store::sha256(
        &tethers_reference_host::m3_store::canonical(&covered).unwrap(),
    );
    let installed_id = Uuid::new_v4().to_string();
    let mut installed = InstalledPlugRecord {
        schema_version: 1,
        installed_id: installed_id.clone(),
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
        trust_evidence: trust,
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
    let mut cov = installed.clone();
    cov.record_digest.clear();
    installed.record_digest = tethers_reference_host::m3_store::sha256(
        &tethers_reference_host::m3_store::canonical(&cov).unwrap(),
    );
    let scope = tethers_reference_host::file_tools::OperationalScopeBinding::create(
        &installed_id,
        "file.move",
        1,
        &query,
        &source,
        &dest,
        tethers_reference_host::file_tools::MAX_CONTENT_BYTES,
        "Matthew",
    )
    .unwrap();
    let store = EnablementStore::open(&enablement_root).unwrap();
    let record = store.enable(&installed, scope, "Matthew").unwrap();
    assert_eq!(record.state, EnablementState::Enabled);
    assert_eq!(record.package_id, "tethers.file-tools");
    store.disable(&installed, "Matthew").unwrap();
    assert!(!store.is_available(&installed_id).unwrap());
    fs::remove_dir_all(temp).unwrap();
}

// 11. A PDF installed Plug can be enabled with PdfOperationalScopeBinding.
#[test]
fn enable_accepts_pdf_binding() {
    let installed = pdf_installed_plug();
    let iid = installed.installed_id.clone();
    let temp = std::env::temp_dir().join(format!("tethers-j23c1-pdf-enable-{}", Uuid::new_v4()));
    let enablement_root = temp.join("enablement");
    let scope_root = temp.join("scope");
    fs::create_dir_all(&scope_root).unwrap();
    let scope = pdf_scope(&iid, &scope_root);
    let store = EnablementStore::open(&enablement_root).unwrap();
    let record = store.enable(&installed, scope, "Matthew").unwrap();
    assert_eq!(record.state, EnablementState::Enabled);
    assert_eq!(record.package_id, "tethers.pdf-tools");
    fs::remove_dir_all(temp).unwrap();
}

// 12. The resulting PDF enablement record contains correct fields.
#[test]
fn pdf_enablement_record_has_correct_fields() {
    let installed = pdf_installed_plug();
    let iid = installed.installed_id.clone();
    let temp = std::env::temp_dir().join(format!("tethers-j23c1-pdf-record-{}", Uuid::new_v4()));
    let enablement_root = temp.join("enablement");
    let scope_root = temp.join("scope");
    fs::create_dir_all(&scope_root).unwrap();
    let scope = pdf_scope(&iid, &scope_root);
    let store = EnablementStore::open(&enablement_root).unwrap();
    let record = store.enable(&installed, scope, "Matthew").unwrap();
    assert_eq!(record.package_id, "tethers.pdf-tools");
    assert_eq!(record.provider_id, "tethers-pdf-provider");
    assert!(matches!(record.operational_scope, OperationalScope::Pdf(_)));
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
    let has_operation = record
        .capabilities
        .iter()
        .any(|c| c.provider_operation_name == "pdf_inspect");
    assert!(has_operation);
    assert_eq!(record.state, EnablementState::Enabled);
    fs::remove_dir_all(temp).unwrap();
}

// 13. A PDF scope with wrong installed_id is refused.
#[test]
fn wrong_installed_id_is_refused_by_enablement() {
    let installed = pdf_installed_plug();
    let temp = std::env::temp_dir().join(format!("tethers-j23c1-wrong-id-{}", Uuid::new_v4()));
    let enablement_root = temp.join("enablement");
    let scope_root = temp.join("scope");
    fs::create_dir_all(&scope_root).unwrap();
    let scope = pdf_scope("wrong-id", &scope_root);
    let store = EnablementStore::open(&enablement_root).unwrap();
    let result = store.enable(&installed, scope, "Matthew");
    assert!(result.is_err());
    fs::remove_dir_all(temp).unwrap();
}

// 14. A PDF scope whose capability is not in installed bindings is refused.
#[test]
fn capability_not_in_bindings_is_refused() {
    let temp = std::env::temp_dir().join(format!("tethers-j23c1-wrong-cap-{}", Uuid::new_v4()));
    let enablement_root = temp.join("enablement");
    let scope_root = temp.join("scope");
    fs::create_dir_all(&scope_root).unwrap();
    let mut installed = pdf_installed_plug();
    installed.disabled_bindings = vec![DisabledBindingRecord {
        state: "disabled".into(),
        capability_name: "other.capability".into(),
        capability_version: 1,
        manifest_digest: digest(),
        provider_operation_name: "other_operation".into(),
    }];
    let mut cov = installed.clone();
    cov.record_digest.clear();
    installed.record_digest = tethers_reference_host::m3_store::sha256(
        &tethers_reference_host::m3_store::canonical(&cov).unwrap(),
    );
    let binding = pdf_scope(&installed.installed_id, &scope_root);
    let store = EnablementStore::open(&enablement_root).unwrap();
    let result = store.enable(&installed, binding, "Matthew");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("capability"),
        "expected capability error, got: {msg}"
    );
    fs::remove_dir_all(temp).unwrap();
}

// 15. Disablement works for both File Tools and PDF enablement records.
#[test]
fn disablement_works_for_pdf() {
    let installed = pdf_installed_plug();
    let iid = installed.installed_id.clone();
    let temp = std::env::temp_dir().join(format!("tethers-j23c1-disable-pdf-{}", Uuid::new_v4()));
    let enablement_root = temp.join("enablement");
    let scope_root = temp.join("scope");
    fs::create_dir_all(&scope_root).unwrap();
    let scope = pdf_scope(&iid, &scope_root);
    let store = EnablementStore::open(&enablement_root).unwrap();
    store.enable(&installed, scope, "Matthew").unwrap();
    assert!(store.is_available(&iid).unwrap());
    store.disable(&installed, "Matthew").unwrap();
    assert!(!store.is_available(&iid).unwrap());
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn disablement_works_for_file_tools() {
    let d = digest();
    let temp = std::env::temp_dir().join(format!("tethers-j23c1-disable-ft-{}", Uuid::new_v4()));
    let enablement_root = temp.join("enablement");
    let scope_root = temp.join("scope");
    let query = scope_root.join("query");
    let source = scope_root.join("source");
    let dest = scope_root.join("destination");
    fs::create_dir_all(&query).unwrap();
    fs::create_dir_all(&source).unwrap();
    fs::create_dir_all(&dest).unwrap();
    let installed_id = Uuid::new_v4().to_string();
    let mut trust = PackageTrustEvidence {
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
    trust.evidence_digest = tethers_reference_host::m3_store::sha256(
        &tethers_reference_host::m3_store::canonical(&covered).unwrap(),
    );
    let mut installed = InstalledPlugRecord {
        schema_version: 1,
        installed_id: installed_id.clone(),
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
        trust_evidence: trust,
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
    let mut cov = installed.clone();
    cov.record_digest.clear();
    installed.record_digest = tethers_reference_host::m3_store::sha256(
        &tethers_reference_host::m3_store::canonical(&cov).unwrap(),
    );
    let scope = tethers_reference_host::file_tools::OperationalScopeBinding::create(
        &installed_id,
        "file.move",
        1,
        &query,
        &source,
        &dest,
        tethers_reference_host::file_tools::MAX_CONTENT_BYTES,
        "Matthew",
    )
    .unwrap();
    let store = EnablementStore::open(&enablement_root).unwrap();
    store.enable(&installed, scope, "Matthew").unwrap();
    assert!(store.is_available(&installed_id).unwrap());
    store.disable(&installed, "Matthew").unwrap();
    assert!(!store.is_available(&installed_id).unwrap());
    fs::remove_dir_all(temp).unwrap();
}
