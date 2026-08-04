use crate::candidate::{CandidateRecord, CandidateRegistry};
use crate::conformance::{
    run_host_conformance_with_authority, ConformanceDisposition, ConformanceEvidence,
    ConformanceEvidenceStore,
};
use crate::current_trust::ExactCandidateTrustAuthority;
use crate::installation_plan::{plan_installation, InstallationPlan, InstallationPlanAction};
use crate::installation_request::InstallationRequest;
use crate::installation_trust::ExactCandidateTrustStore;
use crate::installed::{InstallationApprovalStore, InstalledPlugRegistry};
use crate::launch_profile::{LaunchProfileEvidenceStore, PreparedSupervisedLaunch};
use crate::m3_store::{reject_reparse, verify_chain, M3Error, Result};
use crate::trust::PackageTrustEvidence;
use std::path::Path;
use std::time::Duration;

#[cfg(windows)]
use std::os::windows::io::AsRawHandle;

#[cfg(windows)]
use windows_sys::Win32::Foundation::{SetHandleInformation, HANDLE_FLAG_INHERIT};

#[cfg(windows)]
const ERROR_SHARING_VIOLATION: i32 = 32;

#[cfg(windows)]
const ERROR_LOCK_VIOLATION: i32 = 33;

#[derive(Debug)]
struct InstallationLockGuard {
    #[cfg(windows)]
    _file: std::fs::File,
}

impl InstallationLockGuard {
    /// Returns `Ok(())` when compiled targeting a non-Windows OS.
    /// The caller must still return an error before any mutation or planning.
    fn assert_windows_lock_support() -> Result<()> {
        #[cfg(windows)]
        {
            Ok(())
        }
        #[cfg(not(windows))]
        {
            Err(M3Error::new(
                "installation_lock_invalid",
                "installation lock path is invalid",
            ))
        }
    }

    #[cfg(windows)]
    fn acquire(lock_path: &Path) -> Result<Self> {
        use std::fs::OpenOptions;
        use std::os::windows::fs::OpenOptionsExt;

        Self::assert_windows_lock_support()?;

        // Lock path must be absolute.
        if !lock_path.is_absolute() {
            return Err(M3Error::new(
                "installation_lock_invalid",
                "installation lock path is invalid",
            ));
        }

        // Parent must exist, be a directory, and be reparse-safe.
        let parent = lock_path.parent().ok_or_else(|| {
            M3Error::new(
                "installation_lock_invalid",
                "installation lock path is invalid",
            )
        })?;
        verify_chain(parent)?;
        if !parent.is_dir() {
            return Err(M3Error::new(
                "installation_lock_invalid",
                "installation lock path is invalid",
            ));
        }
        reject_reparse(parent)?;

        // If the lock anchor already exists it must be an ordinary empty non-reparse file.
        if lock_path.exists() {
            reject_reparse(lock_path)?;
            let metadata = lock_path
                .metadata()
                .map_err(|error| M3Error::new("installation_lock_io", error.to_string()))?;
            if !metadata.is_file() {
                return Err(M3Error::new(
                    "installation_lock_invalid",
                    "installation lock path is invalid",
                ));
            }
            if metadata.len() > 0 {
                return Err(M3Error::new(
                    "installation_lock_invalid",
                    "installation lock path is invalid",
                ));
            }
        }

        // Acquire exclusive handle (share_mode(0) = no concurrent access).
        // write(true) is needed alongside create(true) so Windows can create
        // the anchor if it does not exist. The holder never writes bytes.
        let file_result = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .share_mode(0)
            .open(lock_path);

        let file = match file_result {
            Ok(f) => f,
            Err(error) => {
                let os_code = error.raw_os_error().unwrap_or(0);
                if os_code == ERROR_SHARING_VIOLATION || os_code == ERROR_LOCK_VIOLATION {
                    return Err(M3Error::new(
                        "installation_busy",
                        "another installation action is already running",
                    ));
                }
                return Err(M3Error::new(
                    "installation_lock_io",
                    "installation lock could not be acquired",
                ));
            }
        };

        // Clear inherit flag so supervised children cannot retain the lock.
        let raw = file.as_raw_handle() as windows_sys::Win32::Foundation::HANDLE;
        let inherit_result = unsafe { SetHandleInformation(raw, HANDLE_FLAG_INHERIT, 0) };
        if inherit_result == 0 {
            return Err(M3Error::new(
                "installation_lock_io",
                "installation lock could not be acquired",
            ));
        }

        // Verify the opened path again after acquisition.
        reject_reparse(lock_path)?;

        Ok(Self { _file: file })
    }

    #[cfg(not(windows))]
    fn acquire(_lock_path: &Path) -> Result<Self> {
        Self::assert_windows_lock_support()?;
        unreachable!()
    }
}

pub struct InstallationExecutionContext<'a> {
    pub lock_path: &'a Path,
    pub quarantine_root: &'a Path,
    pub conformance_scratch_root: &'a Path,
    pub candidates: &'a CandidateRegistry,
    pub exact_trust: &'a ExactCandidateTrustStore,
    pub launch_profiles: &'a LaunchProfileEvidenceStore,
    pub conformance: &'a ConformanceEvidenceStore,
    pub approvals: &'a InstallationApprovalStore,
    pub installed: &'a InstalledPlugRegistry,
}

pub struct InstallationExecutionOptions<'a> {
    pub approving_authority: &'a str,
    pub host_build_identity: &'a str,
    pub conformance_wall_time: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallationStepResult {
    pub before: InstallationPlan,
    pub after: InstallationPlan,
    pub outcome: InstallationStepOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallationStepOutcome {
    AlreadyComplete,
    Advanced {
        executed: InstallationPlanAction,
    },
    ConformanceRecordedWithoutAdvance {
        evidence_id: String,
        disposition: ConformanceDisposition,
    },
}

pub fn execute_next_installation_action(
    request: &InstallationRequest,
    context: &InstallationExecutionContext<'_>,
    options: &InstallationExecutionOptions<'_>,
) -> Result<InstallationStepResult> {
    let _lock = InstallationLockGuard::acquire(context.lock_path)?;
    execute_installation_action_while_locked(request, context, options)
}

pub(crate) fn validate_options(options: &InstallationExecutionOptions<'_>) -> Result<()> {
    if options.approving_authority.is_empty()
        || options.host_build_identity.is_empty()
        || options.conformance_wall_time.as_millis() == 0
    {
        return Err(M3Error::new(
            "installation_execution_options_invalid",
            "installation execution options are invalid",
        ));
    }
    Ok(())
}

fn execute_installation_action_while_locked(
    request: &InstallationRequest,
    context: &InstallationExecutionContext<'_>,
    options: &InstallationExecutionOptions<'_>,
) -> Result<InstallationStepResult> {
    validate_options(options)?;

    let before = plan_installation(
        request,
        context.candidates,
        context.exact_trust,
        context.launch_profiles,
        context.conformance,
        context.approvals,
        context.installed,
    )?;

    let candidate = load_candidate(context, &before)?;

    match before.action {
        InstallationPlanAction::CreateExactCandidateTrust => {
            handle_create_exact_trust(request, context, options, &before, &candidate)
        }
        InstallationPlanAction::RunSupervisedConformance => {
            handle_supervised_conformance(request, context, options, &before, &candidate)
        }
        InstallationPlanAction::CreateInstallationApproval => {
            handle_installation_approval(request, context, options, &before, &candidate)
        }
        InstallationPlanAction::PublishDisabledInstallation => handle_deferred_publication(),
        InstallationPlanAction::Complete => handle_complete(request, context, &before),
    }
}

fn load_candidate(
    context: &InstallationExecutionContext<'_>,
    plan: &InstallationPlan,
) -> Result<CandidateRecord> {
    let all = context
        .candidates
        .load_all()
        .map_err(|error| M3Error::new("candidate_invalid", error.message))?;

    let mut matching: Vec<CandidateRecord> = all
        .into_iter()
        .filter(|c| c.candidate_id == plan.candidate_id)
        .collect();

    if matching.is_empty() {
        return Err(M3Error::new(
            "installation_execution_plan_stale",
            "installation plan no longer matches current evidence",
        ));
    }
    if matching.len() > 1 {
        return Err(M3Error::new(
            "installation_execution_plan_stale",
            "installation plan no longer matches current evidence",
        ));
    }

    let candidate = matching.remove(0);

    if candidate.package_id != plan.package_id
        || candidate.package_version != plan.package_version
        || candidate.semantic_package_digest != plan.semantic_package_digest
    {
        return Err(M3Error::new(
            "installation_execution_plan_stale",
            "installation plan no longer matches current evidence",
        ));
    }

    candidate
        .validate()
        .map_err(|error| M3Error::new("candidate_invalid", error.message))?;

    Ok(candidate)
}

fn validate_exact_trust_pins(
    context: &InstallationExecutionContext<'_>,
    candidate: &CandidateRecord,
    before: &InstallationPlan,
) -> Result<(
    crate::installation_trust::ExactCandidateTrustRecord,
    PackageTrustEvidence,
)> {
    let trust_digest = before
        .exact_candidate_trust_record_digest
        .as_ref()
        .ok_or_else(|| {
            M3Error::new(
                "installation_execution_postcondition_failed",
                "installation state could not be reconciled after mutation: missing trust record digest",
            )
        })?;

    let evidence_digest = before.trust_evidence_digest.as_ref().ok_or_else(|| {
        M3Error::new(
            "installation_execution_postcondition_failed",
            "installation state could not be reconciled after mutation: missing trust evidence digest",
        )
    })?;

    let trust_record = context
        .exact_trust
        .find(&before.candidate_id)?
        .ok_or_else(|| {
            M3Error::new(
                "installation_execution_plan_stale",
                "installation plan no longer matches current evidence",
            )
        })?;

    trust_record.require_for_candidate(candidate)?;

    if &trust_record.record_digest != trust_digest {
        return Err(M3Error::new(
            "installation_execution_plan_stale",
            "installation plan no longer matches current evidence",
        ));
    }

    let trust_evidence = PackageTrustEvidence::exact_candidate(&trust_record)?;
    trust_evidence.require_for_candidate(candidate)?;

    if &trust_evidence.evidence_digest != evidence_digest {
        return Err(M3Error::new(
            "installation_execution_plan_stale",
            "installation plan no longer matches current evidence",
        ));
    }

    Ok((trust_record, trust_evidence))
}

fn load_launch_profile_by_digest(
    store: &LaunchProfileEvidenceStore,
    digest: &str,
) -> Result<crate::launch_profile::LaunchProfileEvidence> {
    let all = store.load_all()?;
    all.into_iter()
        .find(|lp| lp.profile_evidence_digest == digest)
        .ok_or_else(|| {
            M3Error::new(
                "installation_execution_plan_stale",
                "installation plan no longer matches current evidence",
            )
        })
}

fn load_conformance_by_id(
    store: &ConformanceEvidenceStore,
    id: &str,
    digest: &str,
) -> Result<ConformanceEvidence> {
    let all = store.load_all()?;
    all.into_iter()
        .find(|ce| ce.evidence_id == id)
        .ok_or_else(|| {
            M3Error::new(
                "installation_execution_plan_stale",
                "installation plan no longer matches current evidence",
            )
        })
        .and_then(|ce| {
            if ce.evidence_digest == digest {
                Ok(ce)
            } else {
                Err(M3Error::new(
                    "installation_execution_plan_stale",
                    "installation plan no longer matches current evidence",
                ))
            }
        })
}

fn action_rank(action: &InstallationPlanAction) -> u32 {
    match action {
        InstallationPlanAction::CreateExactCandidateTrust => 0,
        InstallationPlanAction::RunSupervisedConformance => 1,
        InstallationPlanAction::CreateInstallationApproval => 2,
        InstallationPlanAction::PublishDisabledInstallation => 3,
        InstallationPlanAction::Complete => 4,
    }
}

fn validate_transition(
    before: &InstallationPlan,
    after: &InstallationPlan,
    expected_executed: InstallationPlanAction,
) -> Result<()> {
    let before_rank = action_rank(&before.action);
    let after_rank = action_rank(&after.action);

    if after_rank < before_rank {
        return Err(M3Error::new(
            "installation_execution_regressed",
            format!(
                "installation action regressed: expected {:?} but planner returned {:?}",
                expected_executed, after.action
            ),
        ));
    }

    if after_rank > before_rank + 1 {
        return Err(M3Error::new(
            "installation_execution_invalid_transition",
            format!(
                "installation action skipped: expected {:?} but planner returned {:?}",
                expected_executed, after.action
            ),
        ));
    }

    if after_rank > before_rank {
        // Check that candidate identity fields match
        if after.candidate_id != before.candidate_id
            || after.package_id != before.package_id
            || after.package_version != before.package_version
            || after.semantic_package_digest != before.semantic_package_digest
        {
            return Err(M3Error::new(
                "installation_execution_postcondition_failed",
                "installation state could not be reconciled after mutation: candidate identity changed",
            ));
        }

        // All previous pins must be retained (if before had a Some,
        // after must have the same Some).
        if before.exact_candidate_trust_record_digest.is_some()
            && after.exact_candidate_trust_record_digest
                != before.exact_candidate_trust_record_digest
        {
            return Err(M3Error::new(
                "installation_execution_postcondition_failed",
                "installation state could not be reconciled after mutation: trust record digest lost",
            ));
        }
        if before.trust_evidence_digest.is_some()
            && after.trust_evidence_digest != before.trust_evidence_digest
        {
            return Err(M3Error::new(
                "installation_execution_postcondition_failed",
                "installation state could not be reconciled after mutation: trust evidence digest lost",
            ));
        }
        if before.launch_profile_evidence_digest.is_some()
            && after.launch_profile_evidence_digest != before.launch_profile_evidence_digest
        {
            return Err(M3Error::new(
                "installation_execution_postcondition_failed",
                "installation state could not be reconciled after mutation: launch profile digest lost",
            ));
        }
        if before.conformance_evidence_id.is_some()
            && after.conformance_evidence_id != before.conformance_evidence_id
        {
            return Err(M3Error::new(
                "installation_execution_postcondition_failed",
                "installation state could not be reconciled after mutation: conformance evidence id lost",
            ));
        }
        if before.conformance_evidence_digest.is_some()
            && after.conformance_evidence_digest != before.conformance_evidence_digest
        {
            return Err(M3Error::new(
                "installation_execution_postcondition_failed",
                "installation state could not be reconciled after mutation: conformance evidence digest lost",
            ));
        }
        if before.installation_approval_id.is_some()
            && after.installation_approval_id != before.installation_approval_id
        {
            return Err(M3Error::new(
                "installation_execution_postcondition_failed",
                "installation state could not be reconciled after mutation: approval id lost",
            ));
        }
        if before.installation_approval_digest.is_some()
            && after.installation_approval_digest != before.installation_approval_digest
        {
            return Err(M3Error::new(
                "installation_execution_postcondition_failed",
                "installation state could not be reconciled after mutation: approval digest lost",
            ));
        }
    }

    Ok(())
}

fn replan(
    request: &InstallationRequest,
    context: &InstallationExecutionContext<'_>,
) -> Result<InstallationPlan> {
    plan_installation(
        request,
        context.candidates,
        context.exact_trust,
        context.launch_profiles,
        context.conformance,
        context.approvals,
        context.installed,
    )
}

fn handle_create_exact_trust(
    request: &InstallationRequest,
    context: &InstallationExecutionContext<'_>,
    options: &InstallationExecutionOptions<'_>,
    before: &InstallationPlan,
    candidate: &CandidateRecord,
) -> Result<InstallationStepResult> {
    let _record = context
        .exact_trust
        .create(candidate, request, options.approving_authority)?;

    let after = replan(request, context).map_err(|error| {
        M3Error::new(
            "installation_execution_postcondition_failed",
            format!(
                "installation state could not be reconciled after mutation: {}",
                error.code
            ),
        )
    })?;

    if after.action != InstallationPlanAction::RunSupervisedConformance {
        return Err(M3Error::new(
            "installation_execution_postcondition_failed",
            format!(
                "installation state could not be reconciled after mutation: expected RunSupervisedConformance but planner returned {:?}",
                after.action
            ),
        ));
    }

    validate_transition(
        before,
        &after,
        InstallationPlanAction::CreateExactCandidateTrust,
    )?;

    Ok(InstallationStepResult {
        before: before.clone(),
        after,
        outcome: InstallationStepOutcome::Advanced {
            executed: InstallationPlanAction::CreateExactCandidateTrust,
        },
    })
}

struct ConformanceScratchGuard(Option<std::path::PathBuf>);

impl ConformanceScratchGuard {
    fn new(path: std::path::PathBuf) -> Self {
        Self(Some(path))
    }
}

impl Drop for ConformanceScratchGuard {
    fn drop(&mut self) {
        if let Some(path) = &self.0 {
            let _ = std::fs::remove_dir_all(path);
        }
    }
}

fn handle_supervised_conformance(
    request: &InstallationRequest,
    context: &InstallationExecutionContext<'_>,
    options: &InstallationExecutionOptions<'_>,
    before: &InstallationPlan,
    candidate: &CandidateRecord,
) -> Result<InstallationStepResult> {
    let (_trust_record, trust_evidence) = validate_exact_trust_pins(context, candidate, before)?;

    let authority = ExactCandidateTrustAuthority::new(context.exact_trust);

    let prepared = PreparedSupervisedLaunch::prepare(
        candidate,
        context.quarantine_root,
        context.conformance_scratch_root,
        options.conformance_wall_time,
    )?;

    let scratch_path = prepared.scratch_directory().to_path_buf();
    let mut guard = ConformanceScratchGuard::new(scratch_path);
    let launch_evidence = prepared.evidence.clone();

    context.launch_profiles.create(&launch_evidence)?;

    let conformance_evidence = run_host_conformance_with_authority(
        &prepared,
        candidate,
        context.quarantine_root,
        &trust_evidence,
        &authority,
        options.host_build_identity,
    )?;

    context.conformance.create(&conformance_evidence)?;

    // Explicit scratch cleanup.
    let cleanup_result = prepared.cleanup_scratch();
    if cleanup_result.is_err() {
        return Err(M3Error::new(
            "installation_scratch_cleanup_failed",
            "conformance scratch cleanup failed after evidence publication",
        ));
    }
    guard.0 = None;

    let after = match replan(request, context) {
        Ok(plan) => plan,
        Err(error) => {
            return Err(M3Error::new(
                "installation_execution_postcondition_failed",
                format!(
                    "installation state could not be reconciled after mutation: {}",
                    error.code
                ),
            ));
        }
    };

    match conformance_evidence.disposition {
        ConformanceDisposition::Passed => {
            if after.action != InstallationPlanAction::CreateInstallationApproval {
                return Err(M3Error::new(
                    "installation_execution_postcondition_failed",
                    format!(
                        "installation state could not be reconciled after mutation: expected CreateInstallationApproval but planner returned {:?}",
                        after.action
                    ),
                ));
            }

            validate_transition(
                before,
                &after,
                InstallationPlanAction::RunSupervisedConformance,
            )?;

            Ok(InstallationStepResult {
                before: before.clone(),
                after,
                outcome: InstallationStepOutcome::Advanced {
                    executed: InstallationPlanAction::RunSupervisedConformance,
                },
            })
        }
        _ => {
            if after.action != InstallationPlanAction::RunSupervisedConformance {
                return Err(M3Error::new(
                    "installation_execution_postcondition_failed",
                    format!(
                        "installation state could not be reconciled after mutation: expected RunSupervisedConformance but planner returned {:?}",
                        after.action
                    ),
                ));
            }

            Ok(InstallationStepResult {
                before: before.clone(),
                after,
                outcome: InstallationStepOutcome::ConformanceRecordedWithoutAdvance {
                    evidence_id: conformance_evidence.evidence_id.clone(),
                    disposition: conformance_evidence.disposition.clone(),
                },
            })
        }
    }
}

fn handle_installation_approval(
    request: &InstallationRequest,
    context: &InstallationExecutionContext<'_>,
    options: &InstallationExecutionOptions<'_>,
    before: &InstallationPlan,
    candidate: &CandidateRecord,
) -> Result<InstallationStepResult> {
    let (_trust_record, trust_evidence) = validate_exact_trust_pins(context, candidate, before)?;

    let launch_digest = before
        .launch_profile_evidence_digest
        .as_ref()
        .ok_or_else(|| {
            M3Error::new(
                "installation_execution_postcondition_failed",
                "installation state could not be reconciled after mutation: missing launch profile digest",
            )
        })?;
    let launch = load_launch_profile_by_digest(context.launch_profiles, launch_digest)?;

    let conformance_id = before.conformance_evidence_id.as_ref().ok_or_else(|| {
        M3Error::new(
            "installation_execution_postcondition_failed",
            "installation state could not be reconciled after mutation: missing conformance evidence id",
        )
    })?;
    let conformance_digest = before.conformance_evidence_digest.as_ref().ok_or_else(|| {
        M3Error::new(
            "installation_execution_postcondition_failed",
            "installation state could not be reconciled after mutation: missing conformance evidence digest",
        )
    })?;
    let conformance =
        load_conformance_by_id(context.conformance, conformance_id, conformance_digest)?;

    let authority = ExactCandidateTrustAuthority::new(context.exact_trust);

    let _approval = context.approvals.approve_with_authority(
        candidate,
        context.quarantine_root,
        &trust_evidence,
        &authority,
        &launch,
        &conformance,
        options.approving_authority,
    )?;

    let after = replan(request, context).map_err(|error| {
        M3Error::new(
            "installation_execution_postcondition_failed",
            format!(
                "installation state could not be reconciled after mutation: {}",
                error.code
            ),
        )
    })?;

    if after.action != InstallationPlanAction::PublishDisabledInstallation {
        return Err(M3Error::new(
            "installation_execution_postcondition_failed",
            format!(
                "installation state could not be reconciled after mutation: expected PublishDisabledInstallation but planner returned {:?}",
                after.action
            ),
        ));
    }

    validate_transition(
        before,
        &after,
        InstallationPlanAction::CreateInstallationApproval,
    )?;

    Ok(InstallationStepResult {
        before: before.clone(),
        after,
        outcome: InstallationStepOutcome::Advanced {
            executed: InstallationPlanAction::CreateInstallationApproval,
        },
    })
}

fn handle_deferred_publication() -> Result<InstallationStepResult> {
    Err(M3Error::new(
        "installation_publication_deferred",
        "disabled installation publication requires J24K3",
    ))
}

fn handle_complete(
    request: &InstallationRequest,
    context: &InstallationExecutionContext<'_>,
    before: &InstallationPlan,
) -> Result<InstallationStepResult> {
    let after = replan(request, context).map_err(|error| {
        M3Error::new(
            "installation_execution_postcondition_failed",
            format!(
                "installation state could not be reconciled after mutation: {}",
                error.code
            ),
        )
    })?;

    if before != &after {
        return Err(M3Error::new(
            "installation_execution_postcondition_failed",
            "installation state could not be reconciled after mutation: plan changed after Complete",
        ));
    }

    Ok(InstallationStepResult {
        before: before.clone(),
        after,
        outcome: InstallationStepOutcome::AlreadyComplete,
    })
}

#[cfg(test)]
mod lock_tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;
    use uuid::Uuid;

    fn lock_path(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("tethers-j24k2-lock-{name}-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir.join("anchor.lock")
    }

    #[test]
    fn j24k2_lock_acquire_and_release() {
        let path = lock_path("acquire-release");
        {
            let _lock = InstallationLockGuard::acquire(&path).unwrap();
            assert!(path.exists());
        }
        // Lock released; can re-acquire.
        let _lock2 = InstallationLockGuard::acquire(&path).unwrap();
        fs::remove_dir_all(path.parent().unwrap()).ok();
    }

    #[test]
    fn j24k2_lock_second_acquisition_fails_busy() {
        let path = lock_path("second-busy");
        let _lock = InstallationLockGuard::acquire(&path).unwrap();
        let result = InstallationLockGuard::acquire(&path);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "installation_busy");
        drop(_lock);
        fs::remove_dir_all(path.parent().unwrap()).ok();
    }

    #[test]
    fn j24k2_lock_release_after_drop_allows_reacquisition() {
        let path = lock_path("drop-reacquire");
        let guard = InstallationLockGuard::acquire(&path).unwrap();
        drop(guard);
        let _guard2 = InstallationLockGuard::acquire(&path).unwrap();
        fs::remove_dir_all(path.parent().unwrap()).ok();
    }

    #[test]
    fn j24k2_lock_busy_from_another_thread() {
        let path = lock_path("thread-busy");
        let path_clone = path.clone();
        let (tx, rx) = mpsc::channel();

        let _lock = InstallationLockGuard::acquire(&path).unwrap();

        let handle = thread::spawn(move || {
            let result = InstallationLockGuard::acquire(&path_clone);
            tx.send(result.map(|_| ()).unwrap_err().code.to_string())
                .unwrap();
        });

        let code = rx.recv_timeout(Duration::from_secs(3)).unwrap();
        assert_eq!(code, "installation_busy");

        drop(_lock);
        handle.join().unwrap();

        let _lock2 = InstallationLockGuard::acquire(&path).unwrap();
        drop(_lock2);

        fs::remove_dir_all(path.parent().unwrap()).ok();
    }

    #[test]
    fn j24k2_lock_non_absolute_path_rejected() {
        let result = InstallationLockGuard::acquire(Path::new("relative.lock"));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "installation_lock_invalid");
    }

    #[test]
    fn j24k2_lock_missing_parent_directory_rejected() {
        let path = std::env::temp_dir()
            .join(format!("tethers-j24k2-nonexistent-{}", Uuid::new_v4()))
            .join("anchor.lock");
        let result = InstallationLockGuard::acquire(&path);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "installation_lock_invalid");
    }

    #[test]
    fn j24k2_lock_non_empty_existing_anchor_rejected() {
        let path = lock_path("nonempty");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"junk").unwrap();
        drop(f);

        let result = InstallationLockGuard::acquire(&path);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "installation_lock_invalid");

        fs::remove_dir_all(path.parent().unwrap()).ok();
    }

    #[test]
    fn j24k2_lock_preexisting_empty_anchor_accepted() {
        let path = lock_path("preexisting-empty");
        let _f = std::fs::File::create(&path).unwrap();
        drop(_f);

        let guard = InstallationLockGuard::acquire(&path).unwrap();
        drop(guard);

        // Anchor file remains but lock is released.
        assert!(path.exists());

        fs::remove_dir_all(path.parent().unwrap()).ok();
    }
}
