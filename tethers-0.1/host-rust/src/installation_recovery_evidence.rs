//! J24K3c3: read-only recovery evidence-chain revalidator.
//!
//! Given a typed installation request, a validated publication intent, and the
//! existing candidate, exact-trust, launch-profile, conformance, and approval
//! stores, prove that the complete precomputed installed record is still
//! justified by current host-owned evidence.
//!
//! This module performs no destination verification, global installed-root audit,
//! recovery classification, cleanup, publication, intent removal, lock
//! acquisition, or executor wiring.

use crate::candidate::{CandidateRecord, CandidateRegistry};
use crate::conformance::{current_suite_digest, ConformanceEvidence, ConformanceEvidenceStore};
use crate::current_trust::{CurrentTrustAuthority, ExactCandidateTrustAuthority};
use crate::installation_publication_intent::InstallationPublicationIntent;
use crate::installation_request::{
    InstallationRequest, InstallationTargetState, InstallationTrustScope,
    INSTALLATION_REQUEST_SCHEMA,
};
use crate::installation_trust::ExactCandidateTrustStore;
use crate::installed::{InstallationApprovalRecord, InstallationApprovalStore};
use crate::launch_profile::{
    revalidate_candidate, LaunchProfileEvidence, LaunchProfileEvidenceStore,
};
use crate::m3_store::{M3Error, Result};
use crate::trust::PackageTrustEvidence;
use std::path::Path;

fn intent_invalid() -> M3Error {
    M3Error::new(
        "installation_intent_invalid",
        "installation publication intent is invalid",
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

pub(crate) struct InstallationRecoveryEvidenceContext<'a> {
    pub quarantine_root: &'a Path,
    pub candidates: &'a CandidateRegistry,
    pub exact_trust: &'a ExactCandidateTrustStore,
    pub launch_profiles: &'a LaunchProfileEvidenceStore,
    pub conformance: &'a ConformanceEvidenceStore,
    pub approvals: &'a InstallationApprovalStore,
}

pub(crate) fn revalidate_installation_recovery_evidence(
    request: &InstallationRequest,
    intent: &InstallationPublicationIntent,
    context: &InstallationRecoveryEvidenceContext<'_>,
) -> Result<()> {
    intent.validate().map_err(|_| intent_invalid())?;

    validate_request(request, intent)?;

    let candidate = load_and_revalidate_candidate(request, intent, context)?;

    let trust_evidence = revalidate_trust(&candidate, intent, context)?;

    let approval = load_approval(intent, context)?;
    let conformance = load_conformance(intent, context)?;

    if approval.conformance_evidence_id != conformance.evidence_id
        || approval.conformance_evidence_digest != conformance.evidence_digest
    {
        return Err(evidence_stale());
    }

    let launch = load_launch_profile(&approval, &conformance, context)?;

    launch
        .require_for_candidate(&candidate)
        .map_err(|_| evidence_stale())?;
    conformance
        .require_current(
            &candidate,
            &trust_evidence,
            &launch,
            &current_suite_digest().map_err(|_| evidence_stale())?,
        )
        .map_err(|_| evidence_stale())?;

    let quarantine =
        revalidate_candidate(&candidate, context.quarantine_root).map_err(map_m3_error)?;
    approval
        .require_for_recovery(
            &candidate,
            &quarantine,
            &trust_evidence,
            &launch,
            &conformance,
        )
        .map_err(map_m3_error)?;

    intent
        .installed_record
        .require_for_recovery(
            intent,
            &candidate,
            &trust_evidence,
            &launch,
            &conformance,
            &approval,
        )
        .map_err(map_m3_error)?;

    Ok(())
}

fn validate_request(
    request: &InstallationRequest,
    intent: &InstallationPublicationIntent,
) -> Result<()> {
    if request.schema != INSTALLATION_REQUEST_SCHEMA
        || request.candidate_id != intent.candidate_id
        || request.candidate_id != intent.installed_record.source_candidate_id
        || !matches!(request.trust.scope, InstallationTrustScope::ExactCandidate)
        || !request.conformance.allow_non_isolated_supervised_execution
        || !matches!(
            request.installation.target_state,
            InstallationTargetState::Disabled
        )
    {
        return Err(evidence_stale());
    }
    Ok(())
}

fn load_and_revalidate_candidate(
    request: &InstallationRequest,
    intent: &InstallationPublicationIntent,
    context: &InstallationRecoveryEvidenceContext<'_>,
) -> Result<CandidateRecord> {
    let all = context.candidates.load_all().map_err(map_candidate_error)?;

    let mut matching: Vec<CandidateRecord> = all
        .into_iter()
        .filter(|c| c.candidate_id == request.candidate_id)
        .collect();

    if matching.len() != 1 {
        return Err(evidence_stale());
    }

    let candidate = matching.remove(0);

    if candidate.candidate_id != intent.candidate_id
        || candidate.candidate_id != intent.installed_record.source_candidate_id
        || candidate.package_id != intent.installed_record.package_id
        || candidate.package_version != intent.installed_record.package_version
        || candidate.semantic_package_digest != intent.installed_record.semantic_package_digest
        || candidate.raw_archive_digest != intent.installed_record.raw_archive_digest
        || candidate.provider_id != intent.installed_record.provider_id
        || candidate.provider_version != intent.installed_record.provider_version
        || candidate.launch_path != intent.installed_record.launch_path
        || candidate.launch_arguments != intent.installed_record.launch_arguments
        || candidate.provider_working_directory
            != intent.installed_record.provider_working_directory
        || candidate.selected_platform.os != intent.installed_record.platform
        || candidate.selected_platform.architecture != intent.installed_record.architecture
        || candidate.plug_json != intent.installed_record.plug_json
        || candidate.payloads != intent.installed_record.payloads
        || candidate.signature_files != intent.installed_record.signature_files
        || candidate.capabilities != intent.installed_record.capability_manifests
    {
        return Err(evidence_stale());
    }

    revalidate_candidate(&candidate, context.quarantine_root).map_err(map_m3_error)?;

    Ok(candidate)
}

fn revalidate_trust(
    candidate: &CandidateRecord,
    intent: &InstallationPublicationIntent,
    context: &InstallationRecoveryEvidenceContext<'_>,
) -> Result<PackageTrustEvidence> {
    let record = context
        .exact_trust
        .find(&candidate.candidate_id)
        .map_err(map_m3_error)?
        .ok_or_else(evidence_stale)?;

    record
        .require_for_candidate(candidate)
        .map_err(|_| evidence_stale())?;

    let trust_evidence =
        PackageTrustEvidence::exact_candidate(&record).map_err(|_| evidence_stale())?;
    trust_evidence
        .require_for_candidate(candidate)
        .map_err(|_| evidence_stale())?;

    if trust_evidence != intent.installed_record.trust_evidence {
        return Err(evidence_stale());
    }

    let authority = ExactCandidateTrustAuthority::new(context.exact_trust);
    authority
        .revalidate_current(candidate, &trust_evidence, 0)
        .map_err(map_m3_error)?;

    Ok(trust_evidence)
}

fn load_approval(
    intent: &InstallationPublicationIntent,
    context: &InstallationRecoveryEvidenceContext<'_>,
) -> Result<InstallationApprovalRecord> {
    let all = context.approvals.load_all().map_err(map_m3_error)?;
    let id = &intent.installed_record.installation_approval_id;
    let expected_digest = &intent.installed_record.installation_approval_digest;
    let record = all
        .into_iter()
        .find(|r| r.approval_id == *id)
        .ok_or_else(evidence_stale)?;
    if record.record_digest != *expected_digest {
        return Err(evidence_stale());
    }
    Ok(record)
}

fn load_conformance(
    intent: &InstallationPublicationIntent,
    context: &InstallationRecoveryEvidenceContext<'_>,
) -> Result<ConformanceEvidence> {
    let all = context.conformance.load_all().map_err(map_m3_error)?;
    let id = &intent.installed_record.conformance_evidence_id;
    let expected_digest = &intent.installed_record.conformance_evidence_digest;
    let record = all
        .into_iter()
        .find(|r| r.evidence_id == *id)
        .ok_or_else(evidence_stale)?;
    if record.evidence_digest != *expected_digest {
        return Err(evidence_stale());
    }
    Ok(record)
}

fn load_launch_profile(
    approval: &InstallationApprovalRecord,
    conformance: &ConformanceEvidence,
    context: &InstallationRecoveryEvidenceContext<'_>,
) -> Result<LaunchProfileEvidence> {
    if approval.launch_profile_evidence_digest != conformance.launch_profile_evidence_digest {
        return Err(evidence_stale());
    }
    let digest = &conformance.launch_profile_evidence_digest;
    let all = context.launch_profiles.load_all().map_err(map_m3_error)?;
    all.into_iter()
        .find(|l| l.profile_evidence_digest == *digest)
        .ok_or_else(evidence_stale)
}
