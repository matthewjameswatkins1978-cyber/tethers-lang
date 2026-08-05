//! J24K3e1: read-only preparation for a future crash-safe
//! `PublishDisabledInstallation` transaction.
//!
//! The sequence is:
//!
//! ```text
//! receive the current ordinary J24J before-plan
//!   -> create one fresh authoritative J24J plan
//!   -> require exact plan equality and PublishDisabledInstallation
//!   -> create one fresh authoritative J24K3d1 recovery plan
//!   -> require idle recovery state after the global installed-root audit
//!   -> load and revalidate the exact plan-pinned evidence chain
//!   -> precompute one immutable disabled installed record
//!   -> construct one matching publication intent
//!   -> revalidate the complete prepared intent evidence chain
//!   -> prove recovery remains idle
//!   -> return one sealed prepared publication value
//! ```
//!
//! This module generates transaction identity and immutable publication
//! content only. It creates no intent file, no staging directory, no
//! destination, and no installed record; it acquires no lock, executes no
//! ordinary installation action, and performs no recovery mutation.

use crate::candidate::CandidateRecord;
use crate::conformance::{current_suite_digest, ConformanceEvidence};
use crate::current_trust::{CurrentTrustAuthority, ExactCandidateTrustAuthority};
use crate::installation_plan::{plan_installation, InstallationPlan, InstallationPlanAction};
use crate::installation_publication_intent::InstallationPublicationIntent;
use crate::installation_recovery_evidence::revalidate_installation_recovery_evidence;
use crate::installation_recovery_plan::{
    plan_installation_recovery, InstallationRecoveryPlanningContext,
};
use crate::installation_request::InstallationRequest;
use crate::installed::{InstallationApprovalRecord, InstalledPlugRecord};
use crate::launch_profile::{revalidate_candidate, LaunchProfileEvidence};
use crate::m3_store::{M3Error, Result};
use crate::trust::PackageTrustEvidence;

fn plan_stale() -> M3Error {
    M3Error::new(
        "installation_execution_plan_stale",
        "installation plan no longer matches current evidence",
    )
}

fn invalid_transition() -> M3Error {
    M3Error::new(
        "installation_execution_invalid_transition",
        "installation plan action is not publication preparation",
    )
}

fn recovery_conflict() -> M3Error {
    M3Error::new(
        "installation_recovery_conflict",
        "installation recovery state conflicts with publication preparation",
    )
}

fn evidence_stale() -> M3Error {
    M3Error::new(
        "installation_intent_evidence_stale",
        "installation publication evidence is no longer current",
    )
}

fn recovery_io() -> M3Error {
    M3Error::new(
        "installation_recovery_io",
        "installation recovery state could not be observed",
    )
}

/// Preserve path-safety and I/O identity; never collapse `unsafe_store_path`
/// into generic evidence staleness.
fn map_m3_error(error: M3Error) -> M3Error {
    match error.code {
        "unsafe_store_path" => error,
        "store_io" | "install_io" | "install_review_io" | "launch_io" | "candidate_io" => {
            recovery_io()
        }
        _ => evidence_stale(),
    }
}

fn map_candidate_error(error: crate::package::PackageError) -> M3Error {
    match error.code {
        "unsafe_destination" => M3Error::new("unsafe_store_path", "candidate location is unsafe"),
        "candidate_io" => recovery_io(),
        _ => evidence_stale(),
    }
}

/// One sealed, crate-private, read-only publication preparation.
///
/// The value owns exactly one validated publication intent, which in turn owns
/// exactly one precomputed installed record. There is no public constructor, no
/// mutable accessor, and no `Drop` behaviour: holding this value cannot change
/// durable state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparedInstallationPublication {
    intent: InstallationPublicationIntent,
}

impl PreparedInstallationPublication {
    pub(crate) fn intent(&self) -> &InstallationPublicationIntent {
        &self.intent
    }

    pub(crate) fn installed_record(&self) -> &InstalledPlugRecord {
        &self.intent.installed_record
    }
}

/// The complete evidence chain named by the before-plan's exact pins.
struct PlanPinnedEvidence {
    candidate: CandidateRecord,
    trust: PackageTrustEvidence,
    launch: LaunchProfileEvidence,
    conformance: ConformanceEvidence,
    approval: InstallationApprovalRecord,
}

pub(crate) fn prepare_disabled_installation_publication(
    request: &InstallationRequest,
    context: &InstallationRecoveryPlanningContext<'_>,
    before: &InstallationPlan,
) -> Result<PreparedInstallationPublication> {
    require_fresh_exact_plan(request, context, before)?;
    require_idle_recovery(request, context)?;

    let evidence = load_plan_pinned_evidence(context, before)?;

    let record = context.installed.prepare_disabled_installation_record(
        &evidence.candidate,
        &evidence.trust,
        &evidence.launch,
        &evidence.conformance,
        &evidence.approval,
    )?;

    let intent = InstallationPublicationIntent::from_precomputed_record(record)?;

    // Second, independent proof: the freshly generated record must be fully
    // justified by current candidate, trust, launch, conformance, and approval
    // evidence, exactly as a recovered intent would have to be.
    revalidate_installation_recovery_evidence(request, &intent, &context.evidence)?;

    // Preparation must have been read-only.
    require_idle_recovery(request, context)?;

    Ok(PreparedInstallationPublication { intent })
}

/// Regenerate the authoritative ordinary plan and require the caller's
/// before-plan to be exactly it. The caller's plan is never trusted as a source
/// of pins; it is only ever compared against fresh authority.
fn require_fresh_exact_plan(
    request: &InstallationRequest,
    context: &InstallationRecoveryPlanningContext<'_>,
    before: &InstallationPlan,
) -> Result<()> {
    let fresh = plan_installation(
        request,
        context.evidence.candidates,
        context.evidence.exact_trust,
        context.evidence.launch_profiles,
        context.evidence.conformance,
        context.evidence.approvals,
        context.installed,
    )?;

    if fresh != *before {
        return Err(plan_stale());
    }
    if before.action != InstallationPlanAction::PublishDisabledInstallation {
        return Err(invalid_transition());
    }
    if before.installed_id.is_some() || before.installed_record_digest.is_some() {
        return Err(plan_stale());
    }
    if before.exact_candidate_trust_record_digest.is_none()
        || before.trust_evidence_digest.is_none()
        || before.launch_profile_evidence_digest.is_none()
        || before.conformance_evidence_id.is_none()
        || before.conformance_evidence_digest.is_none()
        || before.installation_approval_id.is_none()
        || before.installation_approval_digest.is_none()
    {
        return Err(plan_stale());
    }
    Ok(())
}

fn require_idle_recovery(
    request: &InstallationRequest,
    context: &InstallationRecoveryPlanningContext<'_>,
) -> Result<()> {
    let plan = plan_installation_recovery(request, context)?;
    if !plan.is_idle() || plan.disposition().is_some() {
        return Err(recovery_conflict());
    }
    Ok(())
}

fn load_plan_pinned_evidence(
    context: &InstallationRecoveryPlanningContext<'_>,
    before: &InstallationPlan,
) -> Result<PlanPinnedEvidence> {
    let candidate = load_candidate(context, before)?;
    let trust = load_trust(context, &candidate, before)?;
    let launch = load_launch(context, &candidate, before)?;
    let conformance = load_conformance(context, &candidate, &trust, &launch, before)?;
    let approval = load_approval(context, &candidate, &trust, &launch, &conformance, before)?;

    Ok(PlanPinnedEvidence {
        candidate,
        trust,
        launch,
        conformance,
        approval,
    })
}

fn load_candidate(
    context: &InstallationRecoveryPlanningContext<'_>,
    before: &InstallationPlan,
) -> Result<CandidateRecord> {
    let all = context
        .evidence
        .candidates
        .load_all()
        .map_err(map_candidate_error)?;

    let mut matching: Vec<CandidateRecord> = all
        .into_iter()
        .filter(|c| c.candidate_id == before.candidate_id)
        .collect();

    if matching.len() != 1 {
        return Err(plan_stale());
    }
    let candidate = matching.remove(0);

    if candidate.package_id != before.package_id
        || candidate.package_version != before.package_version
        || candidate.semantic_package_digest != before.semantic_package_digest
    {
        return Err(plan_stale());
    }

    revalidate_candidate(&candidate, context.evidence.quarantine_root).map_err(map_m3_error)?;

    Ok(candidate)
}

fn load_trust(
    context: &InstallationRecoveryPlanningContext<'_>,
    candidate: &CandidateRecord,
    before: &InstallationPlan,
) -> Result<PackageTrustEvidence> {
    let record = context
        .evidence
        .exact_trust
        .find(&candidate.candidate_id)
        .map_err(map_m3_error)?
        .ok_or_else(evidence_stale)?;

    if Some(&record.record_digest) != before.exact_candidate_trust_record_digest.as_ref() {
        return Err(evidence_stale());
    }

    record
        .require_for_candidate(candidate)
        .map_err(|_| evidence_stale())?;

    let trust = PackageTrustEvidence::exact_candidate(&record).map_err(|_| evidence_stale())?;
    trust
        .require_for_candidate(candidate)
        .map_err(|_| evidence_stale())?;

    if Some(&trust.evidence_digest) != before.trust_evidence_digest.as_ref() {
        return Err(evidence_stale());
    }

    // Exact-candidate authority only: no publisher or developer fallback.
    let authority = ExactCandidateTrustAuthority::new(context.evidence.exact_trust);
    authority
        .revalidate_current(candidate, &trust, 0)
        .map_err(map_m3_error)?;

    Ok(trust)
}

fn load_launch(
    context: &InstallationRecoveryPlanningContext<'_>,
    candidate: &CandidateRecord,
    before: &InstallationPlan,
) -> Result<LaunchProfileEvidence> {
    let digest = before
        .launch_profile_evidence_digest
        .as_ref()
        .ok_or_else(plan_stale)?;

    let launch = context
        .evidence
        .launch_profiles
        .load_all()
        .map_err(map_m3_error)?
        .into_iter()
        .find(|profile| profile.profile_evidence_digest == *digest)
        .ok_or_else(evidence_stale)?;

    launch
        .require_for_candidate(candidate)
        .map_err(|_| evidence_stale())?;

    Ok(launch)
}

fn load_conformance(
    context: &InstallationRecoveryPlanningContext<'_>,
    candidate: &CandidateRecord,
    trust: &PackageTrustEvidence,
    launch: &LaunchProfileEvidence,
    before: &InstallationPlan,
) -> Result<ConformanceEvidence> {
    let id = before
        .conformance_evidence_id
        .as_ref()
        .ok_or_else(plan_stale)?;
    let digest = before
        .conformance_evidence_digest
        .as_ref()
        .ok_or_else(plan_stale)?;

    let conformance = context
        .evidence
        .conformance
        .load_all()
        .map_err(map_m3_error)?
        .into_iter()
        .find(|evidence| evidence.evidence_id == *id)
        .ok_or_else(evidence_stale)?;

    if conformance.evidence_digest != *digest
        || conformance.candidate_id != candidate.candidate_id
        || conformance.launch_profile_evidence_digest != launch.profile_evidence_digest
    {
        return Err(evidence_stale());
    }

    conformance
        .require_current(
            candidate,
            trust,
            launch,
            &current_suite_digest().map_err(|_| evidence_stale())?,
        )
        .map_err(|_| evidence_stale())?;

    Ok(conformance)
}

fn load_approval(
    context: &InstallationRecoveryPlanningContext<'_>,
    candidate: &CandidateRecord,
    trust: &PackageTrustEvidence,
    launch: &LaunchProfileEvidence,
    conformance: &ConformanceEvidence,
    before: &InstallationPlan,
) -> Result<InstallationApprovalRecord> {
    let id = before
        .installation_approval_id
        .as_ref()
        .ok_or_else(plan_stale)?;
    let digest = before
        .installation_approval_digest
        .as_ref()
        .ok_or_else(plan_stale)?;

    let approval = context
        .evidence
        .approvals
        .load_all()
        .map_err(map_m3_error)?
        .into_iter()
        .find(|record| record.approval_id == *id)
        .ok_or_else(evidence_stale)?;

    if approval.record_digest != *digest {
        return Err(evidence_stale());
    }

    approval.validate().map_err(map_m3_error)?;

    if approval.candidate_id != candidate.candidate_id
        || approval.conformance_evidence_id != conformance.evidence_id
        || approval.conformance_evidence_digest != conformance.evidence_digest
        || approval.launch_profile_evidence_digest != launch.profile_evidence_digest
        || approval.trust_evidence.evidence_digest != trust.evidence_digest
    {
        return Err(evidence_stale());
    }

    let quarantine =
        revalidate_candidate(candidate, context.evidence.quarantine_root).map_err(map_m3_error)?;

    // Complete approval chain, including reviewed capabilities.
    approval
        .require_for_recovery(candidate, &quarantine, trust, launch, conformance)
        .map_err(map_m3_error)?;

    Ok(approval)
}
