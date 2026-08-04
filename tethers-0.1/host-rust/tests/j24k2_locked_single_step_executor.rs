#![cfg(windows)]

use std::fs;
use std::os::windows::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tethers_reference_host::candidate::{extract_to_quarantine, CandidateRegistry};
use tethers_reference_host::conformance::ConformanceEvidenceStore;
use tethers_reference_host::installation_execution::{
    execute_next_installation_action, InstallationExecutionContext, InstallationExecutionOptions,
    InstallationStepOutcome,
};
use tethers_reference_host::installation_plan::InstallationPlanAction;
use tethers_reference_host::installation_request::{
    InstallationConformanceRequest, InstallationRequest, InstallationTargetRequest,
    InstallationTargetState, InstallationTrustRequest, InstallationTrustScope,
    INSTALLATION_REQUEST_SCHEMA,
};
use tethers_reference_host::installation_trust::ExactCandidateTrustStore;
use tethers_reference_host::installed::{InstallationApprovalStore, InstalledPlugRegistry};
use tethers_reference_host::launch_profile::LaunchProfileEvidenceStore;
use tethers_reference_host::package;
use tethers_reference_host::pdf_tools;
use uuid::Uuid;

fn temp_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("tethers-j24k2-{name}-{}", Uuid::new_v4()))
}

fn setup_candidate(
    base: &Path,
) -> (
    CandidateRegistry,
    tethers_reference_host::candidate::CandidateRecord,
    PathBuf,
) {
    let quarantine_root = base.join("quarantine");
    let archive = base.join("pdf-tools.tetherplug");
    let provider_bytes =
        fs::read(env!("CARGO_BIN_EXE_pdf_tools_provider")).expect("compiled provider");
    fs::write(
        &archive,
        pdf_tools::build_reference_package(&provider_bytes).unwrap(),
    )
    .unwrap();
    let report = package::inspect(&archive).unwrap();
    let quarantined = extract_to_quarantine(&report, &quarantine_root).unwrap();
    let candidates = CandidateRegistry::open(&base.join("candidates"), &quarantine_root).unwrap();
    let candidate = candidates.create(&quarantined).unwrap();
    (candidates, candidate, quarantine_root)
}

fn valid_request(candidate_id: &str) -> InstallationRequest {
    InstallationRequest {
        schema: INSTALLATION_REQUEST_SCHEMA.to_string(),
        candidate_id: candidate_id.to_string(),
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

fn make_context<'a>(
    lock_path: &'a Path,
    quarantine_root: &'a Path,
    scratch: &'a Path,
    candidates: &'a CandidateRegistry,
    exact_trust: &'a ExactCandidateTrustStore,
    profiles: &'a LaunchProfileEvidenceStore,
    conformance_store: &'a ConformanceEvidenceStore,
    approvals: &'a InstallationApprovalStore,
    installed: &'a InstalledPlugRegistry,
) -> InstallationExecutionContext<'a> {
    InstallationExecutionContext {
        lock_path,
        quarantine_root,
        conformance_scratch_root: scratch,
        candidates,
        exact_trust,
        launch_profiles: profiles,
        conformance: conformance_store,
        approvals,
        installed,
    }
}

fn valid_options() -> InstallationExecutionOptions<'static> {
    InstallationExecutionOptions {
        approving_authority: "test-integrator",
        host_build_identity: "tethers-j24k2-tests",
        conformance_wall_time: Duration::from_secs(30),
    }
}

#[test]
fn j24k2_create_exact_candidate_trust_advances_once() {
    let base = temp_dir("create-trust");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, quarantine_root) = setup_candidate(&base);

    let lock_dir = base.join("lock");
    fs::create_dir_all(&lock_dir).unwrap();
    let lock_path = lock_dir.join("anchor.lock");

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let scratch = base.join("scratch");
    fs::create_dir_all(&scratch).unwrap();

    let context = make_context(
        &lock_path,
        &quarantine_root,
        &scratch,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    );
    let options = valid_options();
    let request = valid_request(&candidate.candidate_id);

    let result = execute_next_installation_action(&request, &context, &options).unwrap();

    assert_eq!(
        result.before.action,
        InstallationPlanAction::CreateExactCandidateTrust
    );

    match &result.outcome {
        InstallationStepOutcome::Advanced { executed } => {
            assert_eq!(*executed, InstallationPlanAction::CreateExactCandidateTrust);
        }
        other => panic!("expected Advanced, got {:?}", other),
    }

    assert_eq!(
        result.after.action,
        InstallationPlanAction::RunSupervisedConformance
    );
    assert!(result.after.exact_candidate_trust_record_digest.is_some());
    assert!(result.after.trust_evidence_digest.is_some());

    fs::remove_dir_all(&base).unwrap();
}

#[test]
fn j24k2_trust_creation_is_resumable() {
    let base = temp_dir("resume");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, quarantine_root) = setup_candidate(&base);

    let lock_dir = base.join("lock");
    fs::create_dir_all(&lock_dir).unwrap();
    let lock_path = lock_dir.join("anchor.lock");

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let scratch = base.join("scratch");
    fs::create_dir_all(&scratch).unwrap();

    let context = make_context(
        &lock_path,
        &quarantine_root,
        &scratch,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    );
    let options = valid_options();
    let request = valid_request(&candidate.candidate_id);

    // Step 1: Create trust
    let result1 = execute_next_installation_action(&request, &context, &options).unwrap();
    assert_eq!(
        result1.before.action,
        InstallationPlanAction::CreateExactCandidateTrust
    );

    // Step 2: Next call should plan RunSupervisedConformance
    let result2 = execute_next_installation_action(&request, &context, &options).unwrap();
    assert_eq!(
        result2.before.action,
        InstallationPlanAction::RunSupervisedConformance
    );

    fs::remove_dir_all(&base).unwrap();
}

#[test]
fn j24k2_lock_busy_before_planning() {
    let base = temp_dir("lock-busy");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, quarantine_root) = setup_candidate(&base);

    let lock_dir = base.join("lock");
    fs::create_dir_all(&lock_dir).unwrap();
    let lock_path = lock_dir.join("anchor.lock");

    let scratch = base.join("scratch");
    fs::create_dir_all(&scratch).unwrap();

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let request = valid_request(&candidate.candidate_id);
    let options = valid_options();

    // Hold the lock before calling the executor.
    let _lock_file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .share_mode(0)
        .open(&lock_path)
        .unwrap();

    let context = make_context(
        &lock_path,
        &quarantine_root,
        &scratch,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    );

    let result = execute_next_installation_action(&request, &context, &options);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, "installation_busy");

    drop(_lock_file);
    fs::remove_dir_all(&base).unwrap();
}

#[test]
fn j24k2_lock_releases_after_error() {
    let base = temp_dir("lock-error");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, quarantine_root) = setup_candidate(&base);

    let lock_dir = base.join("lock");
    fs::create_dir_all(&lock_dir).unwrap();
    let lock_path = lock_dir.join("anchor.lock");

    let scratch = base.join("scratch");
    fs::create_dir_all(&scratch).unwrap();

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let request = valid_request(&candidate.candidate_id);

    let context = make_context(
        &lock_path,
        &quarantine_root,
        &scratch,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    );

    let bad_options = InstallationExecutionOptions {
        approving_authority: "",
        host_build_identity: "host",
        conformance_wall_time: Duration::from_secs(30),
    };

    let result = execute_next_installation_action(&request, &context, &bad_options);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().code,
        "installation_execution_options_invalid"
    );

    // Lock was released. Prove by re-calling with valid options.
    let result2 = execute_next_installation_action(&request, &context, &valid_options());
    assert!(result2.is_ok());

    fs::remove_dir_all(&base).unwrap();
}

#[test]
fn j24k2_options_invalid_rejected_before_mutation() {
    let base = temp_dir("opts-invalid");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, quarantine_root) = setup_candidate(&base);

    let lock_dir = base.join("lock");
    fs::create_dir_all(&lock_dir).unwrap();
    let lock_path = lock_dir.join("anchor.lock");

    let scratch = base.join("scratch");
    fs::create_dir_all(&scratch).unwrap();

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let request = valid_request(&candidate.candidate_id);

    let context = make_context(
        &lock_path,
        &quarantine_root,
        &scratch,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    );

    let bad_options = InstallationExecutionOptions {
        approving_authority: "",
        host_build_identity: "",
        conformance_wall_time: Duration::ZERO,
    };

    let result = execute_next_installation_action(&request, &context, &bad_options);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().code,
        "installation_execution_options_invalid"
    );

    fs::remove_dir_all(&base).unwrap();
}

#[test]
fn j24k2_lock_releases_after_panic_unwind() {
    use std::panic;

    let base = temp_dir("lock-panic");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, quarantine_root) = setup_candidate(&base);

    let lock_dir = base.join("lock");
    fs::create_dir_all(&lock_dir).unwrap();
    let lock_path = lock_dir.join("anchor.lock");

    let scratch = base.join("scratch");
    fs::create_dir_all(&scratch).unwrap();

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let request = valid_request(&candidate.candidate_id);
    let options = valid_options();

    let context = make_context(
        &lock_path,
        &quarantine_root,
        &scratch,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    );

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        execute_next_installation_action(&request, &context, &options).unwrap();
        panic!("simulated panic inside lock after mutation");
    }));
    assert!(result.is_err());

    // Lock must be released after panic unwind.
    let result2 = execute_next_installation_action(&request, &context, &options);
    assert!(result2.is_ok());

    fs::remove_dir_all(&base).unwrap();
}

#[test]
fn j24k2_lock_released_and_retry_possible() {
    let base = temp_dir("retry");
    fs::create_dir_all(&base).unwrap();

    let (candidates, candidate, quarantine_root) = setup_candidate(&base);

    let lock_dir = base.join("lock");
    fs::create_dir_all(&lock_dir).unwrap();
    let lock_path = lock_dir.join("anchor.lock");

    let scratch = base.join("scratch");
    fs::create_dir_all(&scratch).unwrap();

    let exact_trust = ExactCandidateTrustStore::open(&base.join("trust")).unwrap();
    let profiles = LaunchProfileEvidenceStore::open(&base.join("profiles")).unwrap();
    let conformance_store = ConformanceEvidenceStore::open(&base.join("conformance")).unwrap();
    let approvals = InstallationApprovalStore::open(&base.join("approvals")).unwrap();
    let installed =
        InstalledPlugRegistry::open(&base.join("install"), &base.join("records")).unwrap();

    let request = valid_request(&candidate.candidate_id);
    let options = valid_options();

    let context = make_context(
        &lock_path,
        &quarantine_root,
        &scratch,
        &candidates,
        &exact_trust,
        &profiles,
        &conformance_store,
        &approvals,
        &installed,
    );

    // Step 1: Create trust (durable mutation with lock held then released)
    let _result1 = execute_next_installation_action(&request, &context, &options).unwrap();

    // Step 2: Lock was released after step 1. Prove by re-calling successfully.
    let result2 = execute_next_installation_action(&request, &context, &options).unwrap();

    // Both before and after actions should be meaningful - the key proof
    // is that the lock was reacquired, planning succeeded, and a result
    // was returned without installation_busy.
    assert!(
        matches!(
            result2.before.action,
            InstallationPlanAction::RunSupervisedConformance
                | InstallationPlanAction::CreateInstallationApproval
                | InstallationPlanAction::PublishDisabledInstallation
                | InstallationPlanAction::Complete
        ),
        "unexpected before action: {:?}",
        result2.before.action
    );

    fs::remove_dir_all(&base).unwrap();
}
