//! F3d — Bounded persistence store characterization evidence.
//!
//! Every test directly proves one F3d property with a hard assertion.
//! Tests are added only where no existing test provides the exact assertion.
//!
//! This module does not infer any store-specific property from `StoreRoot`.
//! `PERSISTENCE_INVENTORY.md` names the exact hard assertion for each F3d
//! PROVEN claim; dimensions without one remain UNVERIFIED.

use crate::enablement::EnablementStore;
use crate::installed::InstalledPlugRecord;
use crate::local_anchor::{AdmissionStore, InboundEvent};
use crate::m3_store::{canonical, sha256};
use crate::trust::DeveloperApprovalStore;
use std::fs;
use uuid::Uuid;

fn temp_dir(label: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("tethers-f3d-{label}-{}", Uuid::new_v4()));
    let _ = fs::create_dir_all(&path);
    path
}

// =========================================================================
// Developer Approval Store (trust.rs)
// Store-specific properties characterized here:
// - duplicate digest create conflict (store-specific check in approve_exact_digest)
// - torn .tmp detection in find() (store-specific error path)
// - filename/content disagreement in find() (store-specific error path)
// - close/reopen preserves record via find() (StoreRoot-backed, verify on this store)
// =========================================================================

#[test]
fn f3d_dev_approval_duplicate_digest_is_conflict() {
    let root = temp_dir("da-dup");
    let digest = format!("sha256:{}", "a".repeat(64));
    let store = DeveloperApprovalStore::open(&root).unwrap();
    store.approve_exact_digest(&digest, "f3d").unwrap();
    assert_eq!(
        store.approve_exact_digest(&digest, "f3d").unwrap_err().code,
        "developer_approval_conflict"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn f3d_dev_approval_torn_tmp_detected_in_find() {
    let root = temp_dir("da-torn");
    let digest = format!("sha256:{}", "b".repeat(64));
    let store = DeveloperApprovalStore::open(&root).unwrap();
    store.approve_exact_digest(&digest, "f3d").unwrap();
    // Write a .tmp file to trigger the torn-detection path in find()
    fs::write(root.join(".torn.tmp"), b"partial").unwrap();
    assert_eq!(
        store.find(&digest).unwrap_err().code,
        "developer_approval_invalid"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn f3d_dev_approval_filename_mismatch_detected() {
    let root = temp_dir("da-filename");
    let digest = format!("sha256:{}", "c".repeat(64));
    let store = DeveloperApprovalStore::open(&root).unwrap();
    let record = store.approve_exact_digest(&digest, "f3d").unwrap();
    let correct = format!("{}.json", record.approval_id);

    // Rename the file to a wrong stem; find() checks file_stem == record.approval_id
    fs::rename(root.join(&correct), root.join("wrong-id.json")).unwrap();
    assert_eq!(
        store.find(&digest).unwrap_err().code,
        "developer_approval_invalid"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn f3d_dev_approval_reopen_preserves_record() {
    let root = temp_dir("da-reopen");
    let digest = format!("sha256:{}", "d".repeat(64));
    {
        let store = DeveloperApprovalStore::open(&root).unwrap();
        store.approve_exact_digest(&digest, "f3d").unwrap();
    }
    let store2 = DeveloperApprovalStore::open_existing(&root).unwrap();
    let found = store2.find(&digest).unwrap().unwrap();
    assert!(found.visibly_unsigned);
    assert_eq!(found.semantic_package_digest, digest);
    let _ = fs::remove_dir_all(root);
}

// =========================================================================
// Enablement Records (enablement.rs)
// Store-specific chain validation: filename/id agreement.
// This module proves filename/id agreement only. Other enablement properties
// are cited individually in the F3d evidence map.
// =========================================================================

#[test]
fn f3d_enablement_record_filename_mismatch_detected() {
    // EnablementStore::load_all checks file_stem == record.enablement_id.
    // Write a syntactically-valid record with wrong filename to trigger the check.
    let digest = format!("sha256:{}", "e".repeat(64));
    let installed = {
        use crate::installed::DisabledBindingRecord;
        use crate::package::PayloadEvidence;
        use crate::trust::{PackageTrustEvidence, TrustModeEvidence};
        let id = Uuid::new_v4().to_string();
        let mut trust = PackageTrustEvidence {
            evidence_format_version: 1,
            semantic_package_digest: digest.clone(),
            mode: TrustModeEvidence::UnsignedDeveloper {
                approval_id: "approval".into(),
                approval_record_digest: digest.clone(),
                visibly_unsigned: true,
            },
            evidence_digest: String::new(),
        };
        let mut tc = trust.clone();
        tc.evidence_digest.clear();
        trust.evidence_digest = sha256(&canonical(&tc).unwrap());
        let mut r = InstalledPlugRecord {
            schema_version: 1,
            installed_id: id.clone(),
            state: "present_disabled".into(),
            package_id: "tethers.file-tools".into(),
            package_version: "1.1.0".into(),
            semantic_package_digest: digest.clone(),
            source_candidate_id: "candidate".into(),
            installation_relative_path: format!("plug-{id}"),
            raw_archive_digest: digest.clone(),
            plug_json: PayloadEvidence {
                path: "plug.json".into(),
                sha256: digest.clone(),
                size_bytes: 1,
                role: "package_descriptor".into(),
            },
            payloads: Vec::new(),
            signature_files: Vec::new(),
            capability_manifests: Vec::new(),
            trust_evidence: trust,
            installation_approval_id: "approval".into(),
            installation_approval_digest: digest.clone(),
            conformance_evidence_id: "conformance".into(),
            conformance_evidence_digest: digest.clone(),
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
                manifest_digest: digest.clone(),
                provider_operation_name: "file_move".into(),
            }],
            operational_scope_schema: None,
            operational_scope_schema_digest: None,
            created_unix_ms: 1,
            record_digest: String::new(),
        };
        let mut c = r.clone();
        c.record_digest.clear();
        r.record_digest = sha256(&canonical(&c).unwrap());
        r
    };

    let root = temp_dir("en-filename");
    let scope_root = temp_dir("en-filename-scope");
    fs::create_dir_all(scope_root.join("query")).unwrap();
    fs::create_dir_all(scope_root.join("source")).unwrap();
    fs::create_dir_all(scope_root.join("destination")).unwrap();

    let ft_schema = serde_json::json!({
        "type": "object",
        "properties": {
            "query_root": {"type": "string", "x-tethers-path": "canonical-directory"},
            "move_source_root": {"type": "string", "x-tethers-path": "canonical-directory"},
            "move_destination_root": {"type": "string", "x-tethers-path": "canonical-directory"},
            "max_content_bytes": {"type": "integer", "minimum": 1, "maximum": 65536}
        },
        "required": ["query_root", "move_source_root", "move_destination_root", "max_content_bytes"],
        "additionalProperties": false
    });
    let ft_schema_bytes = serde_json_canonicalizer::to_vec(&ft_schema).unwrap();
    let ft_digest = sha256(&ft_schema_bytes);

    let scope = crate::operational_scope::OperationalScopeEvidence::create(
        &installed.installed_id,
        &installed.package_id,
        &installed.provider_id,
        &ft_digest,
        &serde_json::json!({
            "query_root": scope_root.join("query").to_string_lossy(),
            "move_source_root": scope_root.join("source").to_string_lossy(),
            "move_destination_root": scope_root.join("destination").to_string_lossy(),
            "max_content_bytes": crate::file_tools::MAX_CONTENT_BYTES,
        }),
        "f3d",
    )
    .unwrap();

    let store = EnablementStore::open(&root).unwrap();
    store.enable(&installed, scope, "f3d").unwrap();

    // Find the enablement record file (stored directly in root, not root/enablement)
    // and rename it to a wrong stem
    let entries: Vec<_> = fs::read_dir(&root)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
        .collect();
    assert!(!entries.is_empty());
    let old = entries[0].path();
    let new = old.with_file_name("wrong-enablement-id.json");
    fs::rename(&old, &new).unwrap();

    assert!(store.load_all().is_err());
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(scope_root);
}

// =========================================================================
// Local Anchor Admission Store
// Store-specific journal property:
// - duplicate evaluation completion with different result → Corrupt
// - conflict recording on same ID different digest
// Other Local Anchor properties are cited individually in the F3d evidence map.
// =========================================================================

#[test]
fn f3d_local_anchor_conflicting_evaluation_result_is_corrupt() {
    let root = temp_dir("la-eval-conflict");
    let e = {
        use crate::local_anchor::EVENT_NAME;
        let payload = crate::m3_store::canonical(&serde_json::json!({"path":"in/a.txt"})).unwrap();
        let digest = sha256(&payload);
        InboundEvent {
            event_id: "evt-eval-conflict".into(),
            event_name: EVENT_NAME.into(),
            provider_identity: "file-tools".into(),
            installed_plug_id: "plug-1".into(),
            session_id: "session-1".into(),
            occurred_at_unix_ms: 1,
            payload: serde_json::json!({"path":"in/a.txt"}),
            payload_digest: format!("sha256:{digest}"),
            source_relative_path: Some("in/a.txt".into()),
            generation: 0,
        }
    };
    let mut store = AdmissionStore::open(&root).unwrap();
    store.admit(&e, 1).unwrap();
    // First evaluation succeeds
    store
        .mark_evaluation_completed("evt-eval-conflict", "sha256:first")
        .unwrap();
    // Second evaluation with DIFFERENT result → Corrupt
    assert!(matches!(
        store.mark_evaluation_completed("evt-eval-conflict", "sha256:second"),
        Err(crate::local_anchor::EventError::Corrupt(_))
    ));
    let _ = fs::remove_dir_all(root);
}
