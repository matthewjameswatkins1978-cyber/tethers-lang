#![cfg(windows)]

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tethers_reference_host::candidate::{extract_to_quarantine, CandidateRegistry};
use tethers_reference_host::conformance::{current_suite_digest, ConformanceEvidenceStore};
use tethers_reference_host::installation_plan::{plan_installation, InstallationPlanAction};
use tethers_reference_host::installation_request::{
    InstallationConformanceRequest, InstallationRequest, InstallationTargetRequest,
    InstallationTargetState, InstallationTrustRequest, InstallationTrustScope,
    INSTALLATION_REQUEST_SCHEMA,
};
use tethers_reference_host::installation_trust::ExactCandidateTrustStore;
use tethers_reference_host::installed::{InstallationApprovalStore, InstalledPlugRegistry};
use tethers_reference_host::launch_profile::{
    LaunchProfileEvidenceStore, PreparedSupervisedLaunch,
};
use tethers_reference_host::package;
use tethers_reference_host::pdf_tools;
use tethers_reference_host::trust::{
    DeveloperApprovalStore, PackageTrustEvidence, PublisherTrustStore,
};
use uuid::Uuid;

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn temp_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("tethers-j24j-{name}-{}", Uuid::new_v4()))
}

fn snapshot(root: &Path) -> BTreeMap<String, String> {
    fn visit(root: &Path, path: &Path, output: &mut BTreeMap<String, String>) {
        if !path.is_dir() {
            return;
        }
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

#[test]
fn no_trust_returns_create_exact_candidate_trust() {
    let base = temp_dir("no-trust");
    fs::create_dir_all(&base).unwrap();

    let archive = base.join("pdf-tools.tetherplug");
    let provider_bytes =
        fs::read(env!("CARGO_BIN_EXE_pdf_tools_provider")).expect("compiled provider");
    fs::write(
        &archive,
        pdf_tools::build_reference_package(&provider_bytes).unwrap(),
    )
    .unwrap();
    let report = package::inspect(&archive).unwrap();
    let quarantined = extract_to_quarantine(&report, &base.join("quarantine")).unwrap();
    let candidates =
        CandidateRegistry::open(&base.join("candidates"), &base.join("quarantine")).unwrap();
    let candidate = candidates.create(&quarantined).unwrap();

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let request = valid_request(&candidate.candidate_id);
    let before = snapshot(&base);
    let plan = plan_installation(
        &request,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap();

    assert_eq!(
        plan.action,
        InstallationPlanAction::CreateExactCandidateTrust
    );
    assert_eq!(plan.candidate_id, candidate.candidate_id);
    assert_eq!(plan.package_id, candidate.package_id);
    assert!(plan.exact_candidate_trust_record_digest.is_none());
    assert!(plan.trust_evidence_digest.is_none());
    assert!(plan.launch_profile_evidence_digest.is_none());
    assert!(plan.conformance_evidence_id.is_none());
    assert!(plan.conformance_evidence_digest.is_none());
    assert!(plan.installation_approval_id.is_none());
    assert!(plan.installation_approval_digest.is_none());
    assert!(plan.installed_id.is_none());
    assert!(plan.installed_record_digest.is_none());
    assert_eq!(before, snapshot(&base));

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn exact_trust_without_conformance_returns_run_supervised_conformance() {
    let base = temp_dir("trust-no-conf");
    fs::create_dir_all(&base).unwrap();

    let archive = base.join("pdf-tools.tetherplug");
    let provider_bytes =
        fs::read(env!("CARGO_BIN_EXE_pdf_tools_provider")).expect("compiled provider");
    fs::write(
        &archive,
        pdf_tools::build_reference_package(&provider_bytes).unwrap(),
    )
    .unwrap();
    let report = package::inspect(&archive).unwrap();
    let quarantined = extract_to_quarantine(&report, &base.join("quarantine")).unwrap();
    let candidates =
        CandidateRegistry::open(&base.join("candidates"), &base.join("quarantine")).unwrap();
    let candidate = candidates.create(&quarantined).unwrap();

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let request = valid_request(&candidate.candidate_id);
    let trust_record = exact_trust
        .create(&candidate, &request, "test-authority")
        .unwrap();

    let before = snapshot(&base);
    let plan = plan_installation(
        &request,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap();

    assert_eq!(
        plan.action,
        InstallationPlanAction::RunSupervisedConformance
    );
    assert_eq!(plan.candidate_id, candidate.candidate_id);
    assert_eq!(
        plan.exact_candidate_trust_record_digest.as_deref(),
        Some(trust_record.record_digest.as_str())
    );
    assert!(plan.trust_evidence_digest.is_some());
    assert!(plan.launch_profile_evidence_digest.is_none());
    assert!(plan.conformance_evidence_id.is_none());
    assert!(plan.conformance_evidence_digest.is_none());
    assert!(plan.installation_approval_id.is_none());
    assert!(plan.installation_approval_digest.is_none());
    assert!(plan.installed_id.is_none());
    assert!(plan.installed_record_digest.is_none());
    assert_eq!(before, snapshot(&base));

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn all_five_action_variants_are_distinct() {
    // Compile-time proof that all five variants exist
    let actions = [
        InstallationPlanAction::CreateExactCandidateTrust,
        InstallationPlanAction::RunSupervisedConformance,
        InstallationPlanAction::CreateInstallationApproval,
        InstallationPlanAction::PublishDisabledInstallation,
        InstallationPlanAction::Complete,
    ];
    // Each action is distinct
    for i in 0..actions.len() {
        for j in 0..actions.len() {
            if i == j {
                assert_eq!(actions[i], actions[j]);
            } else {
                assert_ne!(actions[i], actions[j]);
            }
        }
    }
}

#[test]
fn request_validation_fails_before_evidence_reads() {
    let base = temp_dir("req-validation");
    fs::create_dir_all(&base).unwrap();
    let sentinel_dir = base.join("sentinel");
    fs::create_dir(&sentinel_dir).unwrap();

    let archive = base.join("pdf-tools.tetherplug");
    let provider_bytes =
        fs::read(env!("CARGO_BIN_EXE_pdf_tools_provider")).expect("compiled provider");
    fs::write(
        &archive,
        pdf_tools::build_reference_package(&provider_bytes).unwrap(),
    )
    .unwrap();
    let report = package::inspect(&archive).unwrap();
    let quarantined = extract_to_quarantine(&report, &base.join("quarantine")).unwrap();
    let candidates =
        CandidateRegistry::open(&base.join("candidates"), &base.join("quarantine")).unwrap();
    let candidate = candidates.create(&quarantined).unwrap();

    let trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let before = snapshot(&sentinel_dir);

    // Wrong schema
    let mut req = valid_request(&candidate.candidate_id);
    req.schema = "wrong".to_owned();
    let err = plan_installation(
        &req,
        &candidates,
        &trust,
        &profiles,
        &conformance,
        &approvals,
        &installed,
    )
    .unwrap_err();
    assert_eq!(err.code, "installation_plan_request_invalid");

    // Non-canonical UUID (no hyphens)
    req = valid_request(&candidate.candidate_id);
    req.candidate_id = "3d846d4001fc4e1eb77d83944dbed76f".to_owned();
    let err = plan_installation(
        &req,
        &candidates,
        &trust,
        &profiles,
        &conformance,
        &approvals,
        &installed,
    )
    .unwrap_err();
    assert_eq!(err.code, "installation_plan_request_invalid");

    // Uppercase UUID
    req = valid_request(&candidate.candidate_id);
    req.candidate_id = "3D846D40-01FC-4E1E-B77D-83944DBED76F".to_owned();
    let err = plan_installation(
        &req,
        &candidates,
        &trust,
        &profiles,
        &conformance,
        &approvals,
        &installed,
    )
    .unwrap_err();
    assert_eq!(err.code, "installation_plan_request_invalid");

    // False supervised execution approval
    req = valid_request(&candidate.candidate_id);
    req.conformance.allow_non_isolated_supervised_execution = false;
    let err = plan_installation(
        &req,
        &candidates,
        &trust,
        &profiles,
        &conformance,
        &approvals,
        &installed,
    )
    .unwrap_err();
    assert_eq!(err.code, "installation_plan_request_invalid");

    assert_eq!(before, snapshot(&sentinel_dir));
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn missing_candidate_fails_with_frozen_error() {
    let base = temp_dir("missing-candidate");
    fs::create_dir_all(&base).unwrap();
    let candidates =
        CandidateRegistry::open(&base.join("candidates"), &base.join("quarantine")).unwrap();
    let trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let req = valid_request("3d846d40-01fc-4e1e-b77d-83944dbed76f");
    let before = snapshot(&base);
    let err = plan_installation(
        &req,
        &candidates,
        &trust,
        &profiles,
        &conformance,
        &approvals,
        &installed,
    )
    .unwrap_err();
    assert_eq!(err.code, "installation_plan_candidate_missing");
    assert!(err.message.contains("not present"));
    assert_eq!(before, snapshot(&base));

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn mismatched_trust_fails_closed() {
    let base = temp_dir("mismatched-trust");
    fs::create_dir_all(&base).unwrap();

    let archive = base.join("pdf-tools.tetherplug");
    let provider_bytes =
        fs::read(env!("CARGO_BIN_EXE_pdf_tools_provider")).expect("compiled provider");
    fs::write(
        &archive,
        pdf_tools::build_reference_package(&provider_bytes).unwrap(),
    )
    .unwrap();
    let report = package::inspect(&archive).unwrap();
    let quarantined = extract_to_quarantine(&report, &base.join("quarantine")).unwrap();
    let candidates =
        CandidateRegistry::open(&base.join("candidates"), &base.join("quarantine")).unwrap();
    let candidate = candidates.create(&quarantined).unwrap();

    let trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let request = valid_request(&candidate.candidate_id);
    trust
        .create(&candidate, &request, "test-authority")
        .unwrap();

    // Mutate the candidate record's digest field to cause require_for_candidate failure
    let candidate_path = base
        .join("candidates")
        .join(format!("{}.json", candidate.candidate_id));
    let text = fs::read_to_string(&candidate_path).unwrap();
    let mut value: serde_json::Value = serde_json::from_str(&text).unwrap();
    value["inspection_evidence_digest"] =
        serde_json::Value::String(format!("sha256:{}", "f".repeat(64)));
    let mut record_copy = value.clone();
    record_copy["record_digest"] = serde_json::Value::String(String::new());
    let covered = serde_json_canonicalizer::to_vec(&record_copy).unwrap();
    let new_digest = sha256(&covered);
    record_copy["record_digest"] = serde_json::Value::String(new_digest);
    fs::write(&candidate_path, &serde_json::to_vec(&record_copy).unwrap()).unwrap();

    let before = snapshot(&base);
    let err = plan_installation(
        &request,
        &candidates,
        &trust,
        &profiles,
        &conformance,
        &approvals,
        &installed,
    )
    .unwrap_err();
    assert_eq!(err.code, "installation_trust_candidate_mismatch");
    assert_eq!(before, snapshot(&base));

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn corrupt_store_evidence_fails_closed_not_treated_as_absence() {
    let base = temp_dir("corrupt");
    fs::create_dir_all(&base).unwrap();

    let archive = base.join("pdf-tools.tetherplug");
    let provider_bytes =
        fs::read(env!("CARGO_BIN_EXE_pdf_tools_provider")).expect("compiled provider");
    fs::write(
        &archive,
        pdf_tools::build_reference_package(&provider_bytes).unwrap(),
    )
    .unwrap();
    let report = package::inspect(&archive).unwrap();
    let quarantined = extract_to_quarantine(&report, &base.join("quarantine")).unwrap();
    let candidates =
        CandidateRegistry::open(&base.join("candidates"), &base.join("quarantine")).unwrap();
    let candidate = candidates.create(&quarantined).unwrap();

    let request = valid_request(&candidate.candidate_id);
    let trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    trust
        .create(&candidate, &request, "test-authority")
        .unwrap();

    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    // Place a torn .tmp file in the trust store
    let torn = base.join("trust").join(".torn.tmp");
    fs::write(&torn, b"partial").unwrap();

    let before1 = snapshot(&base);
    let err = plan_installation(
        &request,
        &candidates,
        &trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap_err();
    assert_eq!(err.code, "installation_trust_invalid");
    assert!(err.message.contains("torn"));
    assert_eq!(before1, snapshot(&base));

    // Remove torn file, add a non-JSON entry
    fs::remove_file(&torn).unwrap();
    let bad_entry = base.join("trust").join("bad.unknown");
    fs::write(&bad_entry, b"not json").unwrap();

    let before2 = snapshot(&base);
    let err = plan_installation(
        &request,
        &candidates,
        &trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap_err();
    assert_eq!(err.code, "installation_trust_invalid");
    assert!(err.message.contains("unexpected"));
    assert_eq!(before2, snapshot(&base));

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn planning_never_mutates_filesystem() {
    let base = temp_dir("no-mutation");
    fs::create_dir_all(&base).unwrap();

    let archive = base.join("pdf-tools.tetherplug");
    let provider_bytes =
        fs::read(env!("CARGO_BIN_EXE_pdf_tools_provider")).expect("compiled provider");
    fs::write(
        &archive,
        pdf_tools::build_reference_package(&provider_bytes).unwrap(),
    )
    .unwrap();
    let report = package::inspect(&archive).unwrap();
    let quarantined = extract_to_quarantine(&report, &base.join("quarantine")).unwrap();
    let candidates =
        CandidateRegistry::open(&base.join("candidates"), &base.join("quarantine")).unwrap();
    let candidate = candidates.create(&quarantined).unwrap();

    let request = valid_request(&candidate.candidate_id);
    let trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    // No trust state
    let before1 = snapshot(&base);
    plan_installation(
        &request,
        &candidates,
        &trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap();
    assert_eq!(before1, snapshot(&base));

    // Create trust, verify planning doesn't mutate
    trust
        .create(&candidate, &request, "test-authority")
        .unwrap();
    let before2 = snapshot(&base);
    plan_installation(
        &request,
        &candidates,
        &trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap();
    assert_eq!(before2, snapshot(&base));

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn no_evidence_created_by_planning() {
    let base = temp_dir("no-evidence");
    fs::create_dir_all(&base).unwrap();

    let archive = base.join("pdf-tools.tetherplug");
    let provider_bytes =
        fs::read(env!("CARGO_BIN_EXE_pdf_tools_provider")).expect("compiled provider");
    fs::write(
        &archive,
        pdf_tools::build_reference_package(&provider_bytes).unwrap(),
    )
    .unwrap();
    let report = package::inspect(&archive).unwrap();
    let quarantined = extract_to_quarantine(&report, &base.join("quarantine")).unwrap();
    let candidates =
        CandidateRegistry::open(&base.join("candidates"), &base.join("quarantine")).unwrap();
    let candidate = candidates.create(&quarantined).unwrap();

    let request = valid_request(&candidate.candidate_id);
    let trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    // Verify stores are empty
    assert!(trust.load_all().unwrap().is_empty());
    assert!(conformance_store.load_all().unwrap().is_empty());
    assert!(approvals.load_all().unwrap().is_empty());

    let before = snapshot(&base);
    plan_installation(
        &request,
        &candidates,
        &trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap();
    assert_eq!(before, snapshot(&base));

    // Trust store still empty (planner didn't create trust)
    assert!(trust.load_all().unwrap().is_empty());
    assert!(conformance_store.load_all().unwrap().is_empty());
    assert!(approvals.load_all().unwrap().is_empty());

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn validates_canonical_lowercase_hyphenated_uuid() {
    let req = valid_request("3d846d40-01fc-4e1e-b77d-83944dbed76f");
    let mut bad = req.clone();
    bad.candidate_id = "3D846D40-01FC-4E1E-B77D-83944DBED76F".to_owned();

    // Validation happens at the start of plan_installation, before any store reads
    let base = temp_dir("uuid-check");
    fs::create_dir_all(&base).unwrap();
    let candidates =
        CandidateRegistry::open(&base.join("candidates"), &base.join("quarantine")).unwrap();
    let trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let err = plan_installation(
        &bad,
        &candidates,
        &trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap_err();
    assert_eq!(err.code, "installation_plan_request_invalid");

    let no_hyphens = "3d846d4001fc4e1eb77d83944dbed76f";
    let mut bad2 = req;
    bad2.candidate_id = no_hyphens.to_owned();
    let err = plan_installation(
        &bad2,
        &candidates,
        &trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    )
    .unwrap_err();
    assert_eq!(err.code, "installation_plan_request_invalid");

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn plan_record_structure_matches_public_seam() {
    // Prove the InstallationPlan struct can be constructed and inspected
    let plan = tethers_reference_host::installation_plan::InstallationPlan {
        candidate_id: "id".into(),
        package_id: "pkg".into(),
        package_version: "1.0".into(),
        semantic_package_digest: "sha256:aaaa".into(),
        action: InstallationPlanAction::Complete,
        exact_candidate_trust_record_digest: Some("d1".into()),
        trust_evidence_digest: Some("d2".into()),
        launch_profile_evidence_digest: Some("d3".into()),
        conformance_evidence_id: Some("d4".into()),
        conformance_evidence_digest: Some("d5".into()),
        installation_approval_id: Some("d6".into()),
        installation_approval_digest: Some("d7".into()),
        installed_id: Some("d8".into()),
        installed_record_digest: Some("d9".into()),
    };

    assert_eq!(plan.action, InstallationPlanAction::Complete);
    assert_eq!(plan.installation_approval_id.as_deref(), Some("d6"));
    assert!(plan.installed_id.is_some());
}
