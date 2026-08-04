use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tethers_reference_host::candidate::CandidateRecord;
use tethers_reference_host::installation_request::{
    InstallationConformanceRequest, InstallationRequest, InstallationTargetRequest,
    InstallationTargetState, InstallationTrustRequest, InstallationTrustScope,
    INSTALLATION_REQUEST_SCHEMA,
};
use tethers_reference_host::installation_trust::{
    ExactCandidateTrustRecord, ExactCandidateTrustStore,
};
use tethers_reference_host::trust::{
    DeveloperApprovalStore, PackageTrustEvidence, PublisherTrustStore, TrustModeEvidence,
};
use uuid::Uuid;

fn golden_candidate() -> CandidateRecord {
    let text = include_str!("../fixtures/m2/candidate-record-v1.json");
    serde_json::from_str(text).unwrap()
}

fn valid_request(candidate_id: &str) -> InstallationRequest {
    InstallationRequest {
        schema: INSTALLATION_REQUEST_SCHEMA.to_owned(),
        candidate_id: candidate_id.to_owned(),
        trust: InstallationTrustRequest {
            scope: InstallationTrustScope::ExactCandidate,
        },
        conformance: InstallationConformanceRequest {
            allow_non_isolated_supervised_execution: true,
        },
        installation: InstallationTargetRequest {
            target_state: InstallationTargetState::Disabled,
        },
    }
}

fn temp_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("tethers-j24i-{name}-{}", Uuid::new_v4()))
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn snapshot(root: &Path) -> BTreeMap<String, String> {
    fn visit(root: &Path, path: &Path, output: &mut BTreeMap<String, String>) {
        let mut entries = fs::read_dir(path)
            .unwrap()
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
                output.insert(format!("{relative}/"), "<directory>".into());
                visit(root, &entry, output);
            } else if metadata.is_file() {
                output.insert(relative, sha256(&fs::read(&entry).unwrap()));
            }
        }
    }

    let mut output = BTreeMap::new();
    visit(root, root, &mut output);
    output
}

fn candidate_with_different_id(from: &CandidateRecord, new_id: &str) -> CandidateRecord {
    let mut record = from.clone();
    record.candidate_id = new_id.to_owned();
    record.quarantine_relative_path = format!("candidate-{}", Uuid::new_v4());
    record.record_digest = String::new();
    let covered = serde_json_canonicalizer::to_vec(&record).unwrap();
    record.record_digest = sha256(&covered);
    record
}

#[test]
fn creates_exact_trust_record() {
    let root = temp_dir("create");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    let candidate = golden_candidate();
    let request = valid_request(&candidate.candidate_id);

    let record = store
        .create(&candidate, &request, "test-authority")
        .unwrap();

    assert_eq!(record.schema_version, 1);
    assert_eq!(record.candidate_id, candidate.candidate_id);
    assert_eq!(record.candidate_record_digest, candidate.record_digest);
    assert_eq!(record.package_id, candidate.package_id);
    assert_eq!(record.package_version, candidate.package_version);
    assert_eq!(
        record.semantic_package_digest,
        candidate.semantic_package_digest
    );
    assert_eq!(record.raw_archive_digest, candidate.raw_archive_digest);
    assert_eq!(record.provider_id, candidate.provider_id);
    assert_eq!(record.provider_version, candidate.provider_version);
    assert_eq!(record.request_schema, INSTALLATION_REQUEST_SCHEMA);
    assert_eq!(record.trust_scope, "exact_candidate");
    assert_eq!(record.approving_authority, "test-authority");
    assert!(!record.record_digest.is_empty());
    assert!(record.record_digest.starts_with("sha256:"));

    let expected_path = root.join(format!("{}.json", candidate.candidate_id));
    assert!(expected_path.is_file());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn filename_is_candidate_id_with_no_new_uuid() {
    let root = temp_dir("filename");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    let candidate = golden_candidate();
    let request = valid_request(&candidate.candidate_id);

    let record = store
        .create(&candidate, &request, "test-authority")
        .unwrap();

    let expected = format!("{}.json", candidate.candidate_id);
    let path = root.join(&expected);
    assert!(path.is_file());
    let stored: ExactCandidateTrustRecord =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(stored.candidate_id, record.candidate_id);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn load_all_and_find_round_trip() {
    let root = temp_dir("roundtrip");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    let candidate = golden_candidate();
    let request = valid_request(&candidate.candidate_id);

    let record = store
        .create(&candidate, &request, "test-authority")
        .unwrap();

    let loaded = store.load_all().unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0], record);

    let found = store.find(&candidate.candidate_id).unwrap();
    assert_eq!(found, Some(record));

    assert!(store
        .find("00000000-0000-0000-0000-000000000001")
        .unwrap()
        .is_none());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn open_existing_missing_root_remains_missing() {
    let root = std::env::temp_dir().join(format!("tethers-j24i-missing-{}", Uuid::new_v4()));
    let result = ExactCandidateTrustStore::open_existing(&root);
    assert!(result.is_err());
    assert!(!root.exists());
}

#[test]
fn unrelated_files_preserved_after_create() {
    let root = temp_dir("unrelated");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    let unrelated = root.join("annotations.json");
    fs::write(&unrelated, b"preserved content").unwrap();
    let unrelated_snapshot = sha256(&fs::read(&unrelated).unwrap());

    let candidate = golden_candidate();
    let request = valid_request(&candidate.candidate_id);
    store
        .create(&candidate, &request, "test-authority")
        .unwrap();

    assert_eq!(sha256(&fs::read(&unrelated).unwrap()), unrelated_snapshot);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn second_create_returns_record_conflict() {
    let root = temp_dir("conflict");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    let candidate = golden_candidate();
    let request = valid_request(&candidate.candidate_id);

    store
        .create(&candidate, &request, "test-authority")
        .unwrap();

    let pre_snapshot = snapshot(&root);

    let error = store
        .create(&candidate, &request, "test-authority")
        .unwrap_err();
    assert_eq!(error.code, "record_conflict");

    let post_snapshot = snapshot(&root);
    assert_eq!(pre_snapshot, post_snapshot);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn wrong_request_schema_fails() {
    let root = temp_dir("wrong-schema");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    let candidate = golden_candidate();
    let mut request = valid_request(&candidate.candidate_id);
    request.schema = "wrong-schema".into();

    let error = store
        .create(&candidate, &request, "test-authority")
        .unwrap_err();
    assert_eq!(error.code, "installation_trust_request_invalid");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn mismatched_candidate_id_fails() {
    let root = temp_dir("mismatch-id");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    let candidate = golden_candidate();
    let request = valid_request("00000000-0000-0000-0000-000000000001");

    let error = store
        .create(&candidate, &request, "test-authority")
        .unwrap_err();
    assert_eq!(error.code, "installation_trust_request_invalid");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn false_execution_approval_fails() {
    let root = temp_dir("false-approval");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    let candidate = golden_candidate();
    let mut request = valid_request(&candidate.candidate_id);
    request.conformance.allow_non_isolated_supervised_execution = false;

    let error = store
        .create(&candidate, &request, "test-authority")
        .unwrap_err();
    assert_eq!(error.code, "installation_trust_request_invalid");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn empty_approving_authority_fails() {
    let root = temp_dir("empty-authority");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    let candidate = golden_candidate();
    let request = valid_request(&candidate.candidate_id);

    let error = store.create(&candidate, &request, "").unwrap_err();
    assert_eq!(error.code, "installation_trust_invalid");
    assert!(error.message.contains("approving authority"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn torn_temporary_fails_load_all() {
    let root = temp_dir("torn");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    let candidate = golden_candidate();
    let request = valid_request(&candidate.candidate_id);
    store
        .create(&candidate, &request, "test-authority")
        .unwrap();

    fs::write(root.join(".torn.tmp"), b"partial").unwrap();
    let error = store.load_all().unwrap_err();
    assert_eq!(error.code, "installation_trust_invalid");
    assert!(error.message.contains("torn"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn non_json_entry_fails_load_all() {
    let root = temp_dir("non-json");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    fs::write(root.join("junk.txt"), b"hello").unwrap();

    let error = store.load_all().unwrap_err();
    assert_eq!(error.code, "installation_trust_invalid");
    assert!(error.message.contains("unexpected"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn malformed_json_fails_load_all() {
    let root = temp_dir("malformed");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    fs::write(
        root.join("00000000-0000-0000-0000-000000000001.json"),
        b"not json",
    )
    .unwrap();

    let error = store.load_all().unwrap_err();
    assert_eq!(error.code, "record_invalid");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn filename_mismatch_fails_load_all() {
    let root = temp_dir("name-mismatch");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    let candidate = golden_candidate();
    let request = valid_request(&candidate.candidate_id);
    store
        .create(&candidate, &request, "test-authority")
        .unwrap();

    let original_path = root.join(format!("{}.json", candidate.candidate_id));
    let mismatched_path = root.join("00000000-0000-0000-0000-000000000001.json");
    fs::copy(&original_path, &mismatched_path).unwrap();

    let error = store.load_all().unwrap_err();
    assert_eq!(error.code, "installation_trust_invalid");
    assert!(error.message.contains("filename mismatch"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn record_refuses_different_candidate() {
    let root = temp_dir("different-candidate");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    let candidate = golden_candidate();
    let request = valid_request(&candidate.candidate_id);
    let record = store
        .create(&candidate, &request, "test-authority")
        .unwrap();

    let different = candidate_with_different_id(&candidate, "11111111-1111-1111-1111-111111111111");

    let error = record.require_for_candidate(&different).unwrap_err();
    assert_eq!(error.code, "installation_trust_candidate_mismatch");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn record_refuses_different_candidate_same_digest() {
    let root = temp_dir("same-digest");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    let candidate = golden_candidate();
    let request = valid_request(&candidate.candidate_id);
    let record = store
        .create(&candidate, &request, "test-authority")
        .unwrap();

    let mut different = candidate.clone();
    different.candidate_id = "22222222-2222-2222-2222-222222222222".to_owned();
    different.quarantine_relative_path = format!("candidate-{}", Uuid::new_v4());
    different.record_digest = String::new();
    let covered = serde_json_canonicalizer::to_vec(&different).unwrap();
    different.record_digest = sha256(&covered);

    let error = record.require_for_candidate(&different).unwrap_err();
    assert_eq!(error.code, "installation_trust_candidate_mismatch");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn exact_package_trust_evidence_is_deterministic_and_validates() {
    let root = temp_dir("evidence-det");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    let candidate = golden_candidate();
    let request = valid_request(&candidate.candidate_id);
    let record = store
        .create(&candidate, &request, "test-authority")
        .unwrap();

    let evidence1 = PackageTrustEvidence::exact_candidate(&record).unwrap();
    let evidence2 = PackageTrustEvidence::exact_candidate(&record).unwrap();

    assert_eq!(evidence1, evidence2);
    evidence1.validate().unwrap();

    match &evidence1.mode {
        TrustModeEvidence::ExactCandidate {
            candidate_id,
            candidate_record_digest,
            installation_trust_record_digest,
            approving_authority,
        } => {
            assert_eq!(*candidate_id, record.candidate_id);
            assert_eq!(*candidate_record_digest, record.candidate_record_digest);
            assert_eq!(*installation_trust_record_digest, record.record_digest);
            assert_eq!(*approving_authority, record.approving_authority);
        }
        _ => panic!("expected ExactCandidate mode"),
    }

    assert_eq!(
        evidence1.semantic_package_digest,
        record.semantic_package_digest
    );
    assert_eq!(evidence1.evidence_format_version, 1);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn exact_trust_evidence_accepts_only_exact_candidate() {
    let root = temp_dir("accept-exact");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    let candidate = golden_candidate();
    let request = valid_request(&candidate.candidate_id);
    let record = store
        .create(&candidate, &request, "test-authority")
        .unwrap();
    let evidence = PackageTrustEvidence::exact_candidate(&record).unwrap();

    evidence.require_for_candidate(&candidate).unwrap();

    let different = candidate_with_different_id(&candidate, "33333333-3333-3333-3333-333333333333");

    let error = evidence.require_for_candidate(&different).unwrap_err();
    assert_eq!(error.code, "trust_candidate_mismatch");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn revalidate_current_refuses_exact_candidate() {
    let root = temp_dir("reval-refuse");
    let trust_root = temp_dir("reval-trust");
    let dev_root = temp_dir("reval-dev");
    let trust_store = PublisherTrustStore::open(&trust_root).unwrap();
    let dev_store = DeveloperApprovalStore::open(&dev_root).unwrap();

    let store = ExactCandidateTrustStore::open(&root).unwrap();
    let candidate = golden_candidate();
    let request = valid_request(&candidate.candidate_id);
    let record = store
        .create(&candidate, &request, "test-authority")
        .unwrap();
    let evidence = PackageTrustEvidence::exact_candidate(&record).unwrap();

    let error = evidence
        .revalidate_current("tethers.file-tools", &trust_store, &dev_store, 0)
        .unwrap_err();
    assert_eq!(error.code, "trust_exact_candidate_authority_required");
    assert!(error
        .message
        .contains("requires current installation-trust authority"));
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(trust_root);
    let _ = fs::remove_dir_all(dev_root);
}

#[test]
fn exact_trust_evidence_rejects_altered_record() {
    let root = temp_dir("ev-altered");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    let candidate = golden_candidate();
    let request = valid_request(&candidate.candidate_id);
    let record = store
        .create(&candidate, &request, "test-authority")
        .unwrap();

    let mut altered = record.clone();
    altered.candidate_record_digest =
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned();
    let error = PackageTrustEvidence::exact_candidate(&altered).unwrap_err();
    assert_eq!(error.code, "installation_trust_invalid");
    let _ = fs::remove_dir_all(root);
}

fn evidence_digest_for(evidence: &PackageTrustEvidence) -> String {
    let mut covered = evidence.clone();
    covered.evidence_digest.clear();
    let bytes = serde_json_canonicalizer::to_vec(&covered).unwrap();
    sha256(&bytes)
}

fn exact_evidence_with(
    _record: &ExactCandidateTrustRecord,
    candidate_id: &str,
    candidate_record_digest: &str,
    installation_trust_record_digest: &str,
    approving_authority: &str,
    semantic_package_digest: &str,
) -> PackageTrustEvidence {
    let mut evidence = PackageTrustEvidence {
        evidence_format_version: 1,
        semantic_package_digest: semantic_package_digest.to_owned(),
        mode: TrustModeEvidence::ExactCandidate {
            candidate_id: candidate_id.to_owned(),
            candidate_record_digest: candidate_record_digest.to_owned(),
            installation_trust_record_digest: installation_trust_record_digest.to_owned(),
            approving_authority: approving_authority.to_owned(),
        },
        evidence_digest: String::new(),
    };
    evidence.evidence_digest = evidence_digest_for(&evidence);
    evidence
}

#[test]
fn record_rejects_non_uuid_candidate_id_after_digest_recompute() {
    let root = temp_dir("non-uuid-rec");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    let candidate = golden_candidate();
    let request = valid_request(&candidate.candidate_id);
    let record = store
        .create(&candidate, &request, "test-authority")
        .unwrap();

    let mut malformed = record.clone();
    malformed.candidate_id = "abc".to_owned();
    malformed.record_digest = String::new();
    let covered = serde_json_canonicalizer::to_vec(&malformed).unwrap();
    malformed.record_digest = sha256(&covered);

    let error = PackageTrustEvidence::exact_candidate(&malformed).unwrap_err();
    assert_eq!(error.code, "installation_trust_invalid");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn record_rejects_uppercase_uuid_after_digest_recompute() {
    let root = temp_dir("upper-uuid-rec");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    let candidate = golden_candidate();
    let request = valid_request(&candidate.candidate_id);
    let record = store
        .create(&candidate, &request, "test-authority")
        .unwrap();

    let mut malformed = record.clone();
    malformed.candidate_id = "D9A8BA8A-4543-4D9C-9E05-E4DE90249D71".to_owned();
    malformed.record_digest = String::new();
    let covered = serde_json_canonicalizer::to_vec(&malformed).unwrap();
    malformed.record_digest = sha256(&covered);

    let error = PackageTrustEvidence::exact_candidate(&malformed).unwrap_err();
    assert_eq!(error.code, "installation_trust_invalid");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn require_for_candidate_rejects_invalid_record() {
    let root = temp_dir("require-invalid");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    let candidate = golden_candidate();
    let request = valid_request(&candidate.candidate_id);
    let record = store
        .create(&candidate, &request, "test-authority")
        .unwrap();

    let evidence = exact_evidence_with(
        &record,
        &record.candidate_id,
        "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        &record.record_digest,
        &record.approving_authority,
        &record.semantic_package_digest,
    );
    evidence.validate().unwrap();

    let error = evidence.require_for_candidate(&candidate).unwrap_err();
    assert_eq!(error.code, "trust_candidate_mismatch");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn evidence_rejects_invalid_candidate_id_with_recomputed_digest() {
    let root = temp_dir("ev-invalid-id");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    let candidate = golden_candidate();
    let request = valid_request(&candidate.candidate_id);
    let record = store
        .create(&candidate, &request, "test-authority")
        .unwrap();

    let evidence = exact_evidence_with(
        &record,
        "not-a-uuid",
        &record.candidate_record_digest,
        &record.record_digest,
        &record.approving_authority,
        &record.semantic_package_digest,
    );

    let error = evidence.validate().unwrap_err();
    assert_eq!(error.code, "trust_evidence_invalid");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn evidence_rejects_invalid_candidate_record_digest_with_recomputed_digest() {
    let root = temp_dir("ev-invalid-cr-digest");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    let candidate = golden_candidate();
    let request = valid_request(&candidate.candidate_id);
    let record = store
        .create(&candidate, &request, "test-authority")
        .unwrap();

    let evidence = exact_evidence_with(
        &record,
        &record.candidate_id,
        "not-a-valid-digest",
        &record.record_digest,
        &record.approving_authority,
        &record.semantic_package_digest,
    );

    let error = evidence.validate().unwrap_err();
    assert_eq!(error.code, "trust_evidence_invalid");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn evidence_rejects_invalid_installation_trust_digest_with_recomputed_digest() {
    let root = temp_dir("ev-invalid-it-digest");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    let candidate = golden_candidate();
    let request = valid_request(&candidate.candidate_id);
    let record = store
        .create(&candidate, &request, "test-authority")
        .unwrap();

    let evidence = exact_evidence_with(
        &record,
        &record.candidate_id,
        &record.candidate_record_digest,
        "sha256:TOO_SHORT",
        &record.approving_authority,
        &record.semantic_package_digest,
    );

    let error = evidence.validate().unwrap_err();
    assert_eq!(error.code, "trust_evidence_invalid");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn evidence_rejects_empty_approving_authority_with_recomputed_digest() {
    let root = temp_dir("ev-empty-auth");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    let candidate = golden_candidate();
    let request = valid_request(&candidate.candidate_id);
    let record = store
        .create(&candidate, &request, "test-authority")
        .unwrap();

    let evidence = exact_evidence_with(
        &record,
        &record.candidate_id,
        &record.candidate_record_digest,
        &record.record_digest,
        "",
        &record.semantic_package_digest,
    );

    let error = evidence.validate().unwrap_err();
    assert_eq!(error.code, "trust_evidence_invalid");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn evidence_rejects_invalid_semantic_digest_with_recomputed_digest() {
    let root = temp_dir("ev-invalid-sem-digest");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    let candidate = golden_candidate();
    let request = valid_request(&candidate.candidate_id);
    let record = store
        .create(&candidate, &request, "test-authority")
        .unwrap();

    let evidence = exact_evidence_with(
        &record,
        &record.candidate_id,
        &record.candidate_record_digest,
        &record.record_digest,
        &record.approving_authority,
        "sha256:BAD_DIGEST_123",
    );

    let error = evidence.validate().unwrap_err();
    assert_eq!(error.code, "trust_evidence_invalid");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn load_all_find_corrupt_evidence_not_treated_as_absence() {
    let root = temp_dir("corrupt-find");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    fs::write(
        root.join("00000000-0000-0000-0000-000000000001.json"),
        b"corrupt",
    )
    .unwrap();

    let error = store
        .find("00000000-0000-0000-0000-000000000001")
        .unwrap_err();
    assert!(error.code.contains("invalid"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn empty_store_load_all_returns_empty() {
    let root = temp_dir("empty");
    let store = ExactCandidateTrustStore::open(&root).unwrap();
    let records = store.load_all().unwrap();
    assert!(records.is_empty());
    let _ = fs::remove_dir_all(root);
}
