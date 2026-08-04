use super::current_trust::{CurrentTrustAuthority, ExactCandidateTrustAuthority};
use crate::candidate::CandidateRecord;
use crate::installation_request::{
    InstallationConformanceRequest, InstallationRequest, InstallationTargetRequest,
    InstallationTargetState, InstallationTrustRequest, InstallationTrustScope,
    INSTALLATION_REQUEST_SCHEMA,
};
use crate::installation_trust::ExactCandidateTrustStore;
use crate::m3_store::{canonical, sha256};
use crate::trust::{PackageTrustEvidence, TrustModeEvidence};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn candidate() -> CandidateRecord {
    serde_json::from_str(include_str!("../fixtures/m2/candidate-record-v1.json")).unwrap()
}

fn request(candidate_id: &str) -> InstallationRequest {
    InstallationRequest {
        schema: INSTALLATION_REQUEST_SCHEMA.into(),
        candidate_id: candidate_id.into(),
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

fn root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("tethers-j24k1-{name}-{}", Uuid::new_v4()))
}

fn unsigned_evidence(candidate: &CandidateRecord) -> PackageTrustEvidence {
    let mut evidence = PackageTrustEvidence {
        evidence_format_version: 1,
        semantic_package_digest: candidate.semantic_package_digest.clone(),
        mode: TrustModeEvidence::UnsignedDeveloper {
            approval_id: Uuid::new_v4().to_string(),
            approval_record_digest: format!("sha256:{}", "1".repeat(64)),
            visibly_unsigned: true,
        },
        evidence_digest: String::new(),
    };
    evidence.evidence_digest = sha256(&canonical(&evidence).unwrap());
    evidence
}

#[test]
fn j24k1_exact_authority_accepts_matching_current_record() {
    let path = root("matching");
    let store = ExactCandidateTrustStore::open(&path).unwrap();
    let candidate = candidate();
    let record = store
        .create(&candidate, &request(&candidate.candidate_id), "authority-a")
        .unwrap();
    let evidence = PackageTrustEvidence::exact_candidate(&record).unwrap();

    ExactCandidateTrustAuthority::new(&store)
        .revalidate_current(&candidate, &evidence, 0)
        .unwrap();
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn j24k1_exact_authority_rejects_changed_current_record() {
    let path = root("changed");
    let store = ExactCandidateTrustStore::open(&path).unwrap();
    let candidate = candidate();
    let original = store
        .create(&candidate, &request(&candidate.candidate_id), "authority-a")
        .unwrap();
    let evidence = PackageTrustEvidence::exact_candidate(&original).unwrap();
    let mut current = original.clone();
    current.approving_authority = "authority-b".into();
    current.record_digest.clear();
    current.record_digest = sha256(&canonical(&current).unwrap());
    fs::write(
        path.join(format!("{}.json", candidate.candidate_id)),
        serde_json::to_vec(&current).unwrap(),
    )
    .unwrap();

    let error = ExactCandidateTrustAuthority::new(&store)
        .revalidate_current(&candidate, &evidence, 0)
        .unwrap_err();
    assert_eq!(error.code, "trust_drift");
    assert_eq!(error.message, "exact-candidate installation trust changed");
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn j24k1_exact_authority_rejects_absent_wrong_mode_and_corrupt_store() {
    let candidate = candidate();
    let absent_path = root("absent");
    let absent = ExactCandidateTrustStore::open(&absent_path).unwrap();
    let record_path = root("wrong-mode");
    let wrong_mode_store = ExactCandidateTrustStore::open(&record_path).unwrap();
    let record = wrong_mode_store
        .create(&candidate, &request(&candidate.candidate_id), "authority-a")
        .unwrap();
    let error = ExactCandidateTrustAuthority::new(&absent)
        .revalidate_current(
            &candidate,
            &PackageTrustEvidence::exact_candidate(&record).unwrap(),
            0,
        )
        .unwrap_err();
    assert_eq!(error.code, "trust_drift");
    assert_eq!(
        error.message,
        "exact-candidate installation trust is absent"
    );

    let error = ExactCandidateTrustAuthority::new(&wrong_mode_store)
        .revalidate_current(&candidate, &unsigned_evidence(&candidate), 0)
        .unwrap_err();
    assert_eq!(error.code, "trust_exact_candidate_authority_required");

    fs::write(record_path.join("broken.tmp"), b"partial").unwrap();
    let error = ExactCandidateTrustAuthority::new(&wrong_mode_store)
        .revalidate_current(
            &candidate,
            &PackageTrustEvidence::exact_candidate(&record).unwrap(),
            0,
        )
        .unwrap_err();
    assert_eq!(error.code, "installation_trust_invalid");
    fs::remove_dir_all(absent_path).unwrap();
    fs::remove_dir_all(record_path).unwrap();
}
