use super::installation_execution::{validate_options, InstallationExecutionOptions};
use crate::candidate::{extract_to_quarantine, CandidateRegistry};
use crate::conformance::{
    current_suite_digest, CaseDisposition, ConformanceCaseEvidence, ConformanceDisposition,
    ConformanceEvidence, ConformanceEvidenceStore,
};
use crate::current_trust::ExactCandidateTrustAuthority;
use crate::installation_execution::{
    execute_next_installation_action, InstallationExecutionContext, InstallationStepOutcome,
};
use crate::installation_plan::{InstallationPlan, InstallationPlanAction};
use crate::installation_request::{
    InstallationConformanceRequest, InstallationRequest, InstallationTargetRequest,
    InstallationTargetState, InstallationTrustRequest, InstallationTrustScope,
    INSTALLATION_REQUEST_SCHEMA,
};
use crate::installation_trust::ExactCandidateTrustStore;
use crate::installed::{InstallationApprovalStore, InstalledPlugRegistry};
use crate::launch_profile::{LaunchProfileEvidenceStore, PreparedSupervisedLaunch};
use crate::m3_store::{canonical, sha256};
use crate::package;
use crate::pdf_tools;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

fn empty_plan() -> InstallationPlan {
    InstallationPlan {
        candidate_id: "a".to_string(),
        package_id: "b".to_string(),
        package_version: "1.0".to_string(),
        semantic_package_digest: "sha256:aaaa".to_string(),
        action: InstallationPlanAction::CreateExactCandidateTrust,
        exact_candidate_trust_record_digest: None,
        trust_evidence_digest: None,
        launch_profile_evidence_digest: None,
        conformance_evidence_id: None,
        conformance_evidence_digest: None,
        installation_approval_id: None,
        installation_approval_digest: None,
        installed_id: None,
        installed_record_digest: None,
    }
}

fn plan_with(
    action: InstallationPlanAction,
    trust_digest: Option<&str>,
    evidence_digest: Option<&str>,
    launch_digest: Option<&str>,
    conformance_id: Option<&str>,
    conformance_digest: Option<&str>,
    approval_id: Option<&str>,
    approval_digest: Option<&str>,
) -> InstallationPlan {
    InstallationPlan {
        action,
        exact_candidate_trust_record_digest: trust_digest.map(|s| s.to_string()),
        trust_evidence_digest: evidence_digest.map(|s| s.to_string()),
        launch_profile_evidence_digest: launch_digest.map(|s| s.to_string()),
        conformance_evidence_id: conformance_id.map(|s| s.to_string()),
        conformance_evidence_digest: conformance_digest.map(|s| s.to_string()),
        installation_approval_id: approval_id.map(|s| s.to_string()),
        installation_approval_digest: approval_digest.map(|s| s.to_string()),
        ..empty_plan()
    }
}

fn complete_request(candidate_id: &str) -> InstallationRequest {
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

fn complete_test_base() -> (
    PathBuf,
    CandidateRegistry,
    crate::candidate::CandidateRecord,
    PathBuf,
) {
    let base = std::env::temp_dir().join(format!("tethers-j24k2-complete-{}", Uuid::new_v4()));
    fs::create_dir_all(&base).unwrap();
    let archive = base.join("pdf-tools.tetherplug");
    fs::write(
        &archive,
        pdf_tools::build_reference_package(b"complete-test-provider").unwrap(),
    )
    .unwrap();
    let quarantine_root = base.join("quarantine");
    let report = package::inspect(&archive).unwrap();
    let quarantined = extract_to_quarantine(&report, &quarantine_root).unwrap();
    let candidates = CandidateRegistry::open(&base.join("candidates"), &quarantine_root).unwrap();
    let candidate = candidates.create(&quarantined).unwrap();
    (base, candidates, candidate, quarantine_root)
}

fn complete_conformance(
    candidate: &crate::candidate::CandidateRecord,
    trust: &crate::trust::PackageTrustEvidence,
    launch: &crate::launch_profile::LaunchProfileEvidence,
) -> ConformanceEvidence {
    let cases = [
        "static_candidate_revalidation",
        "exact_launch_clean_environment",
        "mcp_initialize_protocol_pin",
        "provider_identity",
        "complete_discovery_exact_operations",
        "bounded_valid_fixture_call",
        "invalid_fixture_call_refused",
        "bounded_shutdown_process_cleanup",
    ]
    .into_iter()
    .map(|case_id| ConformanceCaseEvidence {
        case_id: case_id.to_owned(),
        disposition: CaseDisposition::Passed,
        safe_diagnostic_code: None,
    })
    .collect();
    let mut evidence = ConformanceEvidence {
        schema_version: 1,
        evidence_id: Uuid::new_v4().to_string(),
        candidate_id: candidate.candidate_id.clone(),
        package_id: candidate.package_id.clone(),
        package_version: candidate.package_version.clone(),
        semantic_package_digest: candidate.semantic_package_digest.clone(),
        payloads: candidate.payloads.clone(),
        capabilities: candidate.capabilities.clone(),
        trust_evidence_digest: trust.evidence_digest.clone(),
        launch_profile_evidence_digest: launch.profile_evidence_digest.clone(),
        launch_profile_label: launch.profile_label.clone(),
        provider_id: candidate.provider_id.clone(),
        provider_version: candidate.provider_version.clone(),
        socket_major: 1,
        mcp_protocol_version: "2025-11-25".to_owned(),
        binding_version: "mcp-stdio-2025-11-25".to_owned(),
        host_build_identity: "complete-test".to_owned(),
        platform: "windows".to_owned(),
        architecture: "x86_64".to_owned(),
        suite_version: "m3-generic-1".to_owned(),
        suite_digest: current_suite_digest().unwrap(),
        test_configuration_digest: format!("sha256:{}", "d".repeat(64)),
        started_unix_ms: 1,
        ended_unix_ms: 2,
        cases,
        disposition: ConformanceDisposition::Passed,
        retry_count: 0,
        raw_stderr_persisted: false,
        evidence_digest: String::new(),
    };
    let mut covered = evidence.clone();
    covered.evidence_digest.clear();
    evidence.evidence_digest = sha256(&canonical(&covered).unwrap());
    evidence.validate().unwrap();
    evidence
}

fn tree_snapshot(root: &Path) -> std::collections::BTreeMap<String, String> {
    fn visit(root: &Path, path: &Path, output: &mut std::collections::BTreeMap<String, String>) {
        if !path.is_dir() {
            return;
        }
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let relative = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            if path.is_dir() {
                output.insert(relative, "<directory>".to_owned());
                visit(root, &path, output);
            } else {
                output.insert(relative, sha256(&fs::read(path).unwrap()));
            }
        }
    }
    let mut output = std::collections::BTreeMap::new();
    if root.exists() {
        visit(root, root, &mut output);
    }
    output
}

#[test]
fn j24k2_validate_options_rejects_empty_authority() {
    let opts = InstallationExecutionOptions {
        approving_authority: "",
        host_build_identity: "host",
        conformance_wall_time: Duration::from_secs(1),
    };
    let result = validate_options(&opts);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().code,
        "installation_execution_options_invalid"
    );
}

#[test]
fn j24k2_validate_options_rejects_empty_build_identity() {
    let opts = InstallationExecutionOptions {
        approving_authority: "auth",
        host_build_identity: "",
        conformance_wall_time: Duration::from_secs(1),
    };
    let result = validate_options(&opts);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().code,
        "installation_execution_options_invalid"
    );
}

#[test]
fn j24k2_validate_options_rejects_zero_wall_time() {
    let opts = InstallationExecutionOptions {
        approving_authority: "auth",
        host_build_identity: "host",
        conformance_wall_time: Duration::ZERO,
    };
    let result = validate_options(&opts);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().code,
        "installation_execution_options_invalid"
    );
}

#[test]
fn j24k2_validate_options_accepts_valid() {
    let opts = InstallationExecutionOptions {
        approving_authority: "auth",
        host_build_identity: "host",
        conformance_wall_time: Duration::from_millis(1),
    };
    let result = validate_options(&opts);
    assert!(result.is_ok());
}

#[test]
fn j24k2_complete_state_is_idempotent_and_mutation_free() {
    let (base, candidates, candidate, quarantine_root) = complete_test_base();
    let request = complete_request(&candidate.candidate_id);
    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let trust_record = exact_trust
        .create(&candidate, &request, "complete-test-authority")
        .unwrap();
    let trust_evidence =
        crate::trust::PackageTrustEvidence::exact_candidate(&trust_record).unwrap();
    let prepared = PreparedSupervisedLaunch::prepare(
        &candidate,
        &quarantine_root,
        &base.join("scratch"),
        Duration::from_secs(3),
    )
    .unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    profiles.create(&prepared.evidence).unwrap();
    let conformance = complete_conformance(&candidate, &trust_evidence, &prepared.evidence);
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    conformance_store.create(&conformance).unwrap();
    let authority = ExactCandidateTrustAuthority::new(&exact_trust);
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let approval = approvals
        .approve_with_authority(
            &candidate,
            &quarantine_root,
            &trust_evidence,
            &authority,
            &prepared.evidence,
            &conformance,
            "complete-test-authority",
        )
        .unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();
    installed
        .install_disabled_with_authority(
            &candidate,
            &quarantine_root,
            &trust_evidence,
            &authority,
            &prepared.evidence,
            &conformance,
            &approval,
        )
        .unwrap();
    prepared.cleanup_scratch().unwrap();

    let lock_dir = base.join("lock");
    fs::create_dir_all(&lock_dir).unwrap();
    let context = InstallationExecutionContext {
        lock_path: &lock_dir.join("anchor.lock"),
        quarantine_root: &quarantine_root,
        conformance_scratch_root: &base.join("scratch"),
        candidates: &candidates,
        exact_trust: &exact_trust,
        launch_profiles: &profiles,
        conformance: &conformance_store,
        approvals: &approvals,
        installed: &installed,
    };
    let options = InstallationExecutionOptions {
        approving_authority: "complete-test-authority",
        host_build_identity: "complete-test",
        conformance_wall_time: Duration::from_secs(3),
    };
    let counts_before = (
        exact_trust.load_all().unwrap().len(),
        profiles.load_all().unwrap().len(),
        conformance_store.load_all().unwrap().len(),
        approvals.load_all().unwrap().len(),
        installed.load_all().unwrap().len(),
    );
    let install_before = tree_snapshot(&base.join("install"));
    let result = execute_next_installation_action(&request, &context, &options).unwrap();
    assert_eq!(result.before.action, InstallationPlanAction::Complete);
    assert_eq!(result.after, result.before);
    assert_eq!(result.outcome, InstallationStepOutcome::AlreadyComplete);
    assert_eq!(
        counts_before,
        (
            exact_trust.load_all().unwrap().len(),
            profiles.load_all().unwrap().len(),
            conformance_store.load_all().unwrap().len(),
            approvals.load_all().unwrap().len(),
            installed.load_all().unwrap().len(),
        )
    );
    assert_eq!(install_before, tree_snapshot(&base.join("install")));
    fs::remove_dir_all(base).unwrap();
}
