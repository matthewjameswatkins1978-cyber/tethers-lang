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
use crate::installation_plan::{plan_installation, InstallationPlanAction};
use crate::installation_publication_intent::InstallationPublicationIntentStore;
use crate::installation_publication_mutation::install_post_intent_failure_once_for_test;
use crate::installation_recovery_evidence::InstallationRecoveryEvidenceContext;
use crate::installation_recovery_execution::{
    execute_validated_installation_recovery, InstallationRecoveryExecutionOutcome,
};
use crate::installation_recovery_plan::{
    plan_installation_recovery, InstallationRecoveryPlanningContext,
};
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
use crate::trust::PackageTrustEvidence;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

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
    let executor_state = base.join("executor-state");
    fs::create_dir_all(&executor_state).unwrap();
    let context = InstallationExecutionContext {
        lock_path: &lock_dir.join("anchor.lock"),
        executor_state_root: &executor_state,
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

struct PublicationReadyFixture {
    _base: PathBuf,
    candidates: CandidateRegistry,
    candidate: crate::candidate::CandidateRecord,
    quarantine_root: PathBuf,
    exact_trust: ExactCandidateTrustStore,
    profiles: LaunchProfileEvidenceStore,
    conformance_store: ConformanceEvidenceStore,
    approvals: InstallationApprovalStore,
    installed: InstalledPlugRegistry,
    executor_state: PathBuf,
    lock_dir: PathBuf,
}

fn publication_ready_fixture() -> PublicationReadyFixture {
    let (base, candidates, candidate, quarantine_root) = complete_test_base();
    let request = complete_request(&candidate.candidate_id);
    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let trust_record = exact_trust
        .create(&candidate, &request, "publication-test-authority")
        .unwrap();
    let trust_evidence = PackageTrustEvidence::exact_candidate(&trust_record).unwrap();
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
    approvals
        .approve_with_authority(
            &candidate,
            &quarantine_root,
            &trust_evidence,
            &authority,
            &prepared.evidence,
            &conformance,
            "publication-test-authority",
        )
        .unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();
    prepared.cleanup_scratch().unwrap();

    let lock_dir = base.join("lock");
    fs::create_dir_all(&lock_dir).unwrap();
    let executor_state = base.join("executor-state");
    fs::create_dir_all(&executor_state).unwrap();

    PublicationReadyFixture {
        _base: base,
        candidates,
        candidate,
        quarantine_root,
        exact_trust,
        profiles,
        conformance_store,
        approvals,
        installed,
        executor_state,
        lock_dir,
    }
}

#[test]
fn j24k3f_publication_advances_to_complete() {
    let fix = publication_ready_fixture();
    let request = complete_request(&fix.candidate.candidate_id);
    let context = InstallationExecutionContext {
        lock_path: &fix.lock_dir.join("anchor.lock"),
        executor_state_root: &fix.executor_state,
        quarantine_root: &fix.quarantine_root,
        conformance_scratch_root: &fix._base.join("scratch"),
        candidates: &fix.candidates,
        exact_trust: &fix.exact_trust,
        launch_profiles: &fix.profiles,
        conformance: &fix.conformance_store,
        approvals: &fix.approvals,
        installed: &fix.installed,
    };
    let options = InstallationExecutionOptions {
        approving_authority: "publication-test-authority",
        host_build_identity: "publication-test",
        conformance_wall_time: Duration::from_secs(3),
    };

    let result = execute_next_installation_action(&request, &context, &options).unwrap();
    assert_eq!(
        result.before.action,
        InstallationPlanAction::PublishDisabledInstallation
    );
    assert_eq!(result.after.action, InstallationPlanAction::Complete);
    assert_eq!(
        result.outcome,
        InstallationStepOutcome::Advanced {
            executed: InstallationPlanAction::PublishDisabledInstallation,
        }
    );

    let installed_records = fix.installed.load_all().unwrap();
    assert_eq!(installed_records.len(), 1);
    let record = &installed_records[0];
    assert_eq!(record.source_candidate_id, fix.candidate.candidate_id);
    assert_eq!(record.package_id, fix.candidate.package_id);

    fs::remove_dir_all(&fix._base).unwrap();
}

#[test]
fn j24k3f_intent_store_rooted_under_executor_state() {
    let fix = publication_ready_fixture();

    let request = complete_request(&fix.candidate.candidate_id);
    let context = InstallationExecutionContext {
        lock_path: &fix.lock_dir.join("anchor.lock"),
        executor_state_root: &fix.executor_state,
        quarantine_root: &fix.quarantine_root,
        conformance_scratch_root: &fix._base.join("scratch"),
        candidates: &fix.candidates,
        exact_trust: &fix.exact_trust,
        launch_profiles: &fix.profiles,
        conformance: &fix.conformance_store,
        approvals: &fix.approvals,
        installed: &fix.installed,
    };
    let options = InstallationExecutionOptions {
        approving_authority: "publication-test-authority",
        host_build_identity: "publication-test",
        conformance_wall_time: Duration::from_secs(3),
    };
    execute_next_installation_action(&request, &context, &options).unwrap();

    // Intent was created and cleaned up during successful publication,
    // so the intent store must no longer contain a pending current intent.
    let intent_store = InstallationPublicationIntentStore::open(&fix.executor_state).unwrap();
    assert!(intent_store.load().unwrap().is_none());

    fs::remove_dir_all(&fix._base).unwrap();
}

#[test]
fn j24k3f_no_intent_under_wrong_paths() {
    let fix = publication_ready_fixture();

    let request = complete_request(&fix.candidate.candidate_id);
    let context = InstallationExecutionContext {
        lock_path: &fix.lock_dir.join("anchor.lock"),
        executor_state_root: &fix.executor_state,
        quarantine_root: &fix.quarantine_root,
        conformance_scratch_root: &fix._base.join("scratch"),
        candidates: &fix.candidates,
        exact_trust: &fix.exact_trust,
        launch_profiles: &fix.profiles,
        conformance: &fix.conformance_store,
        approvals: &fix.approvals,
        installed: &fix.installed,
    };
    let options = InstallationExecutionOptions {
        approving_authority: "publication-test-authority",
        host_build_identity: "publication-test",
        conformance_wall_time: Duration::from_secs(3),
    };
    execute_next_installation_action(&request, &context, &options).unwrap();

    // No intent artefacts under lock, quarantine, scratch, install, or records roots.
    for root in &[
        fix.lock_dir,
        fix.quarantine_root,
        fix._base.join("scratch"),
        fix._base.join("install"),
        fix._base.join("records"),
    ] {
        for entry in fs::read_dir(root).ok().into_iter().flatten() {
            let entry = entry.unwrap();
            let name = entry.file_name().to_string_lossy().to_lowercase();
            assert!(
                !name.contains("intent") && !name.contains("current.json"),
                "unexpected intent artefact {:?} under {:?}",
                entry.path(),
                root
            );
        }
    }

    fs::remove_dir_all(&fix._base).unwrap();
}

#[test]
fn j24k3f_destination_and_record_exist_recovery_idle() {
    let fix = publication_ready_fixture();
    let request = complete_request(&fix.candidate.candidate_id);
    let context = InstallationExecutionContext {
        lock_path: &fix.lock_dir.join("anchor.lock"),
        executor_state_root: &fix.executor_state,
        quarantine_root: &fix.quarantine_root,
        conformance_scratch_root: &fix._base.join("scratch"),
        candidates: &fix.candidates,
        exact_trust: &fix.exact_trust,
        launch_profiles: &fix.profiles,
        conformance: &fix.conformance_store,
        approvals: &fix.approvals,
        installed: &fix.installed,
    };
    let options = InstallationExecutionOptions {
        approving_authority: "publication-test-authority",
        host_build_identity: "publication-test",
        conformance_wall_time: Duration::from_secs(3),
    };

    let result = execute_next_installation_action(&request, &context, &options).unwrap();
    assert_eq!(result.after.action, InstallationPlanAction::Complete);

    let installed_records = fix.installed.load_all().unwrap();
    assert_eq!(installed_records.len(), 1);

    let record = &installed_records[0];
    let dest = fix
        ._base
        .join("install")
        .join(format!("plug-{}", record.installed_id));
    assert!(
        dest.is_dir(),
        "destination directory {:?} does not exist",
        dest
    );

    let record_file = fix
        ._base
        .join("records")
        .join(format!("{}.json", record.installed_id));
    assert!(
        record_file.is_file(),
        "record file {:?} does not exist",
        record_file
    );

    // Recovery must be idle after successful publication.
    let intent_store = InstallationPublicationIntentStore::open(&fix.executor_state).unwrap();
    let evidence = InstallationRecoveryEvidenceContext {
        quarantine_root: &fix.quarantine_root,
        candidates: &fix.candidates,
        exact_trust: &fix.exact_trust,
        launch_profiles: &fix.profiles,
        conformance: &fix.conformance_store,
        approvals: &fix.approvals,
    };
    let recovery_ctx = InstallationRecoveryPlanningContext {
        intents: &intent_store,
        installed: &fix.installed,
        evidence,
    };
    let recovery_plan = plan_installation_recovery(&request, &recovery_ctx).unwrap();
    assert!(recovery_plan.is_idle());
    assert!(recovery_plan.disposition().is_none());

    fs::remove_dir_all(&fix._base).unwrap();
}

#[test]
fn j24k3f_returned_plans_and_outcome_exact() {
    let fix = publication_ready_fixture();
    let request = complete_request(&fix.candidate.candidate_id);
    let context = InstallationExecutionContext {
        lock_path: &fix.lock_dir.join("anchor.lock"),
        executor_state_root: &fix.executor_state,
        quarantine_root: &fix.quarantine_root,
        conformance_scratch_root: &fix._base.join("scratch"),
        candidates: &fix.candidates,
        exact_trust: &fix.exact_trust,
        launch_profiles: &fix.profiles,
        conformance: &fix.conformance_store,
        approvals: &fix.approvals,
        installed: &fix.installed,
    };
    let options = InstallationExecutionOptions {
        approving_authority: "publication-test-authority",
        host_build_identity: "publication-test",
        conformance_wall_time: Duration::from_secs(3),
    };

    let result = execute_next_installation_action(&request, &context, &options).unwrap();

    assert_eq!(
        result.before.action,
        InstallationPlanAction::PublishDisabledInstallation
    );
    assert_eq!(result.after.action, InstallationPlanAction::Complete);
    assert!(
        result.after.installed_id.is_some(),
        "after-plan must have installed_id"
    );
    assert!(
        result.after.installed_record_digest.is_some(),
        "after-plan must have installed_record_digest"
    );

    match result.outcome {
        InstallationStepOutcome::Advanced { executed } => {
            assert_eq!(
                executed,
                InstallationPlanAction::PublishDisabledInstallation
            );
        }
        _ => panic!("expected Advanced outcome, got {:?}", result.outcome),
    }

    // All pre-publication pins must be retained in the after-plan.
    assert_eq!(
        result.after.exact_candidate_trust_record_digest,
        result.before.exact_candidate_trust_record_digest
    );
    assert_eq!(
        result.after.trust_evidence_digest,
        result.before.trust_evidence_digest
    );

    fs::remove_dir_all(&fix._base).unwrap();
}

#[test]
fn j24k3f_second_lock_returns_installation_busy() {
    let fix = publication_ready_fixture();
    let lock_path = fix.lock_dir.join("anchor.lock");
    let _held = std::fs::File::create(&lock_path).unwrap();

    let request = complete_request(&fix.candidate.candidate_id);
    let context = InstallationExecutionContext {
        lock_path: &lock_path,
        executor_state_root: &fix.executor_state,
        quarantine_root: &fix.quarantine_root,
        conformance_scratch_root: &fix._base.join("scratch"),
        candidates: &fix.candidates,
        exact_trust: &fix.exact_trust,
        launch_profiles: &fix.profiles,
        conformance: &fix.conformance_store,
        approvals: &fix.approvals,
        installed: &fix.installed,
    };
    let options = InstallationExecutionOptions {
        approving_authority: "publication-test-authority",
        host_build_identity: "publication-test",
        conformance_wall_time: Duration::from_secs(3),
    };

    let result = execute_next_installation_action(&request, &context, &options);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, "installation_busy");

    drop(_held);
    fs::remove_dir_all(&fix._base).unwrap();
}

#[test]
fn j24k3f_no_second_action_executed() {
    let fix = publication_ready_fixture();
    let request = complete_request(&fix.candidate.candidate_id);
    let context = InstallationExecutionContext {
        lock_path: &fix.lock_dir.join("anchor.lock"),
        executor_state_root: &fix.executor_state,
        quarantine_root: &fix.quarantine_root,
        conformance_scratch_root: &fix._base.join("scratch"),
        candidates: &fix.candidates,
        exact_trust: &fix.exact_trust,
        launch_profiles: &fix.profiles,
        conformance: &fix.conformance_store,
        approvals: &fix.approvals,
        installed: &fix.installed,
    };
    let options = InstallationExecutionOptions {
        approving_authority: "publication-test-authority",
        host_build_identity: "publication-test",
        conformance_wall_time: Duration::from_secs(3),
    };

    // First invocation: publication.
    let result1 = execute_next_installation_action(&request, &context, &options).unwrap();
    assert_eq!(
        result1.outcome,
        InstallationStepOutcome::Advanced {
            executed: InstallationPlanAction::PublishDisabledInstallation,
        }
    );

    let counts_after_first = (
        fix.exact_trust.load_all().unwrap().len(),
        fix.profiles.load_all().unwrap().len(),
        fix.conformance_store.load_all().unwrap().len(),
        fix.approvals.load_all().unwrap().len(),
        fix.installed.load_all().unwrap().len(),
    );
    let install_snapshot = tree_snapshot(&fix._base.join("install"));
    let records_snapshot = tree_snapshot(&fix._base.join("records"));

    // Second invocation: must return AlreadyComplete with no mutation.
    let result2 = execute_next_installation_action(&request, &context, &options).unwrap();
    assert_eq!(result2.before.action, InstallationPlanAction::Complete);
    assert_eq!(result2.after.action, InstallationPlanAction::Complete);
    assert_eq!(result2.outcome, InstallationStepOutcome::AlreadyComplete);

    // All store counts and file trees must be identical.
    assert_eq!(
        counts_after_first,
        (
            fix.exact_trust.load_all().unwrap().len(),
            fix.profiles.load_all().unwrap().len(),
            fix.conformance_store.load_all().unwrap().len(),
            fix.approvals.load_all().unwrap().len(),
            fix.installed.load_all().unwrap().len(),
        )
    );
    assert_eq!(install_snapshot, tree_snapshot(&fix._base.join("install")));
    assert_eq!(records_snapshot, tree_snapshot(&fix._base.join("records")));

    fs::remove_dir_all(&fix._base).unwrap();
}

#[test]
fn j24k3f_complete_returns_already_complete_no_mutation() {
    // Full setup to Complete state (with installation), then verify executor is idempotent.
    let (base, candidates, candidate, quarantine_root) = complete_test_base();
    let request = complete_request(&candidate.candidate_id);
    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let trust_record = exact_trust
        .create(&candidate, &request, "complete-test-authority")
        .unwrap();
    let trust_evidence = PackageTrustEvidence::exact_candidate(&trust_record).unwrap();
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
    let executor_state = base.join("executor-state");
    fs::create_dir_all(&executor_state).unwrap();
    let context = InstallationExecutionContext {
        lock_path: &lock_dir.join("anchor.lock"),
        executor_state_root: &executor_state,
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

#[test]
fn j24k3f_global_untracked_destination_blocks_publication_pre_intent_lock_released() {
    // Genuine pre-intent preparation/evidence failure through the public locked
    // executor. The publication-intent store begins EMPTY; no intent is
    // pre-created. A foreign installed-looking destination directory (`plug-<uuid>`)
    // with no owning record is a legitimate filesystem obstruction already used by
    // `j24k3e1_global_untracked_final_destination_blocks_preparation`. It leaves the
    // installed registry empty, so `plan_installation` still returns
    // `PublishDisabledInstallation` and the executor reaches the accepted J24K3e1
    // preparation boundary through the publication route. Preparation's idle-recovery
    // global destination audit then fails closed BEFORE any durable intent is created.
    let fix = publication_ready_fixture();
    let intent_store = InstallationPublicationIntentStore::open(&fix.executor_state).unwrap();
    assert!(
        intent_store.load().unwrap().is_none(),
        "intent store must start empty"
    );

    let install_root = fix._base.join("install");
    let untracked = install_root.join(format!("plug-{}", Uuid::new_v4()));
    fs::create_dir(&untracked).unwrap();
    let install_before = tree_snapshot(&install_root);
    let records_before = tree_snapshot(&fix._base.join("records"));
    assert!(
        fix.installed.load_all().unwrap().is_empty(),
        "no installed records may exist before the boundary"
    );

    let request = complete_request(&fix.candidate.candidate_id);
    let lock_path = fix.lock_dir.join("anchor.lock");
    let context = InstallationExecutionContext {
        lock_path: &lock_path,
        executor_state_root: &fix.executor_state,
        quarantine_root: &fix.quarantine_root,
        conformance_scratch_root: &fix._base.join("scratch"),
        candidates: &fix.candidates,
        exact_trust: &fix.exact_trust,
        launch_profiles: &fix.profiles,
        conformance: &fix.conformance_store,
        approvals: &fix.approvals,
        installed: &fix.installed,
    };
    let options = InstallationExecutionOptions {
        approving_authority: "publication-test-authority",
        host_build_identity: "publication-test",
        conformance_wall_time: Duration::from_secs(3),
    };

    let error = execute_next_installation_action(&request, &context, &options).unwrap_err();
    assert_eq!(
        error.code, "installation_destination_untracked",
        "accepted pre-intent failure must surface through the public executor"
    );

    // Pre-intent failure: the executor created no durable publication state.
    assert!(
        intent_store.load().unwrap().is_none(),
        "no durable intent may be created before the publication boundary"
    );
    assert_eq!(
        install_before,
        tree_snapshot(&install_root),
        "no destination may be created before intent creation"
    );
    assert!(
        fix.installed.load_all().unwrap().is_empty(),
        "no installed record may be created before intent creation"
    );
    assert_eq!(
        records_before,
        tree_snapshot(&fix._base.join("records")),
        "no record file may be created before intent creation"
    );
    assert!(untracked.exists(), "the obstruction must be left untouched");

    // The lock must have been released on the failure: a second public invocation
    // must NOT return `installation_busy`. It reaches the same fail-closed
    // boundary again, which also proves no second mutation occurs.
    let second = execute_next_installation_action(&request, &context, &options).unwrap_err();
    assert_ne!(
        second.code, "installation_busy",
        "the lock must be released after a pre-intent failure"
    );
    assert_eq!(
        second.code, "installation_destination_untracked",
        "the failure must be reproducible without a second mutation"
    );
    assert!(intent_store.load().unwrap().is_none());
    assert_eq!(install_before, tree_snapshot(&install_root));

    fs::remove_dir_all(&fix._base).unwrap();
}

#[test]
fn j24k3f_test_only_post_intent_failure_is_recoverable_and_publishes_once() {
    let fix = publication_ready_fixture();
    let request = complete_request(&fix.candidate.candidate_id);
    let lock_path = fix.lock_dir.join("anchor.lock");
    let context = InstallationExecutionContext {
        lock_path: &lock_path,
        executor_state_root: &fix.executor_state,
        quarantine_root: &fix.quarantine_root,
        conformance_scratch_root: &fix._base.join("scratch"),
        candidates: &fix.candidates,
        exact_trust: &fix.exact_trust,
        launch_profiles: &fix.profiles,
        conformance: &fix.conformance_store,
        approvals: &fix.approvals,
        installed: &fix.installed,
    };
    let options = InstallationExecutionOptions {
        approving_authority: "publication-test-authority",
        host_build_identity: "publication-test",
        conformance_wall_time: Duration::from_secs(3),
    };
    let intent_store = InstallationPublicationIntentStore::open(&fix.executor_state).unwrap();
    let install_root = fix._base.join("install");
    let records_root = fix._base.join("records");
    let install_before = tree_snapshot(&install_root);
    let records_before = tree_snapshot(&records_root);

    // The hook is installed only for this test thread. The public executor must
    // first create and read back the exact durable intent, then fail before
    // staging, destination, or record publication.
    let _forced_failure = install_post_intent_failure_once_for_test();
    let error = execute_next_installation_action(&request, &context, &options).unwrap_err();
    assert_eq!(error.code, "installation_test_forced_failure");

    let retained_intent = intent_store
        .load()
        .unwrap()
        .expect("post-intent failure must retain the exact durable intent");
    assert!(
        !install_root
            .join(&retained_intent.destination_relative_path)
            .exists(),
        "the forced boundary is before destination publication"
    );
    assert!(fix.installed.load_all().unwrap().is_empty());
    assert_eq!(install_before, tree_snapshot(&install_root));
    assert_eq!(records_before, tree_snapshot(&records_root));

    let ordinary_plan = plan_installation(
        &request,
        &fix.candidates,
        &fix.exact_trust,
        &fix.profiles,
        &fix.conformance_store,
        &fix.approvals,
        &fix.installed,
    )
    .unwrap();
    assert_eq!(
        ordinary_plan.action,
        InstallationPlanAction::PublishDisabledInstallation,
        "a retained intent must not create a false Complete result"
    );

    // The outer RAII lock is released with the error. A second public call
    // cannot continue ordinary publication while recovery is pending, but it
    // must not report a held lock or create a second mutation.
    let pending = execute_next_installation_action(&request, &context, &options).unwrap_err();
    assert_ne!(pending.code, "installation_busy");
    assert_eq!(pending.code, "installation_recovery_conflict");
    assert_eq!(
        intent_store.load().unwrap().as_ref(),
        Some(&retained_intent)
    );
    assert_eq!(install_before, tree_snapshot(&install_root));
    assert_eq!(records_before, tree_snapshot(&records_root));

    let evidence = InstallationRecoveryEvidenceContext {
        quarantine_root: &fix.quarantine_root,
        candidates: &fix.candidates,
        exact_trust: &fix.exact_trust,
        launch_profiles: &fix.profiles,
        conformance: &fix.conformance_store,
        approvals: &fix.approvals,
    };
    let recovery_context = InstallationRecoveryPlanningContext {
        intents: &intent_store,
        installed: &fix.installed,
        evidence,
    };
    let recovery_plan = plan_installation_recovery(&request, &recovery_context).unwrap();
    assert_eq!(
        recovery_plan.disposition(),
        Some(crate::installation_recovery::InstallationRecoveryDisposition::RemoveIntentOnly)
    );
    assert_eq!(
        execute_validated_installation_recovery(&request, &recovery_context, recovery_plan)
            .unwrap(),
        InstallationRecoveryExecutionOutcome::Recovered {
            disposition:
                crate::installation_recovery::InstallationRecoveryDisposition::RemoveIntentOnly,
        }
    );
    assert!(intent_store.load().unwrap().is_none());
    assert!(plan_installation_recovery(&request, &recovery_context)
        .unwrap()
        .is_idle());

    // With accepted recovery complete, the next public locked invocation
    // performs the sole ordinary mutation and reaches Complete.
    let completed = execute_next_installation_action(&request, &context, &options).unwrap();
    assert_eq!(
        completed.before.action,
        InstallationPlanAction::PublishDisabledInstallation
    );
    assert_eq!(completed.after.action, InstallationPlanAction::Complete);
    assert_eq!(
        completed.outcome,
        InstallationStepOutcome::Advanced {
            executed: InstallationPlanAction::PublishDisabledInstallation,
        }
    );
    let records = fix.installed.load_all().unwrap();
    assert_eq!(records.len(), 1, "recovery must not create a second record");
    assert_ne!(
        records[0].installed_id, retained_intent.transaction_id,
        "intent-only recovery removes the failed transaction before the one later publication"
    );
    assert!(
        install_root
            .join(format!("plug-{}", records[0].installed_id))
            .is_dir(),
        "the sole installed record must own the sole final destination"
    );

    fs::remove_dir_all(&fix._base).unwrap();
}
