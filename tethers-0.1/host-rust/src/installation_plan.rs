use crate::candidate::{CandidateRecord, CandidateRegistry};
use crate::conformance::{
    current_suite_digest, ConformanceDisposition, ConformanceEvidence, ConformanceEvidenceStore,
};
use crate::installation_request::{
    InstallationRequest, InstallationTargetState, InstallationTrustScope,
    INSTALLATION_REQUEST_SCHEMA,
};
use crate::installation_trust::{ExactCandidateTrustRecord, ExactCandidateTrustStore};
use crate::installed::{
    InstallationApprovalRecord, InstallationApprovalStore, InstalledPlugRecord,
    InstalledPlugRegistry,
};
use crate::launch_profile::{LaunchProfileEvidence, LaunchProfileEvidenceStore};
use crate::m3_store::{M3Error, Result};
use crate::trust::PackageTrustEvidence;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallationPlanAction {
    CreateExactCandidateTrust,
    RunSupervisedConformance,
    CreateInstallationApproval,
    PublishDisabledInstallation,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallationPlan {
    pub candidate_id: String,
    pub package_id: String,
    pub package_version: String,
    pub semantic_package_digest: String,
    pub action: InstallationPlanAction,
    pub exact_candidate_trust_record_digest: Option<String>,
    pub trust_evidence_digest: Option<String>,
    pub launch_profile_evidence_digest: Option<String>,
    pub conformance_evidence_id: Option<String>,
    pub conformance_evidence_digest: Option<String>,
    pub installation_approval_id: Option<String>,
    pub installation_approval_digest: Option<String>,
    pub installed_id: Option<String>,
    pub installed_record_digest: Option<String>,
}

fn validate_request(request: &InstallationRequest) -> Result<()> {
    if request.schema != INSTALLATION_REQUEST_SCHEMA {
        return Err(M3Error::new(
            "installation_plan_request_invalid",
            "installation request is not valid for reconciliation",
        ));
    }

    let parsed = Uuid::parse_str(&request.candidate_id).map_err(|_| {
        M3Error::new(
            "installation_plan_request_invalid",
            "installation request is not valid for reconciliation",
        )
    })?;
    if parsed.hyphenated().to_string() != request.candidate_id {
        return Err(M3Error::new(
            "installation_plan_request_invalid",
            "installation request is not valid for reconciliation",
        ));
    }

    if !matches!(request.trust.scope, InstallationTrustScope::ExactCandidate) {
        return Err(M3Error::new(
            "installation_plan_request_invalid",
            "installation request is not valid for reconciliation",
        ));
    }

    if !request.conformance.allow_non_isolated_supervised_execution {
        return Err(M3Error::new(
            "installation_plan_request_invalid",
            "installation request is not valid for reconciliation",
        ));
    }

    if !matches!(
        request.installation.target_state,
        InstallationTargetState::Disabled
    ) {
        return Err(M3Error::new(
            "installation_plan_request_invalid",
            "installation request is not valid for reconciliation",
        ));
    }

    Ok(())
}

fn select_candidate(
    request: &InstallationRequest,
    candidates: &CandidateRegistry,
) -> Result<CandidateRecord> {
    let all = candidates
        .load_all()
        .map_err(|error| M3Error::new("candidate_invalid", error.message))?;

    let mut matching: Vec<CandidateRecord> = all
        .into_iter()
        .filter(|c| c.candidate_id == request.candidate_id)
        .collect();

    if matching.is_empty() {
        return Err(M3Error::new(
            "installation_plan_candidate_missing",
            "installation candidate is not present",
        ));
    }
    if matching.len() > 1 {
        return Err(M3Error::new(
            "installation_plan_conflict",
            "installation evidence is ambiguous",
        ));
    }

    Ok(matching.remove(0))
}

fn find_exact_trust(
    candidate: &CandidateRecord,
    exact_trust: &ExactCandidateTrustStore,
) -> Result<Option<(ExactCandidateTrustRecord, PackageTrustEvidence)>> {
    let record = exact_trust.find(&candidate.candidate_id)?;
    match record {
        None => Ok(None),
        Some(record) => {
            record.require_for_candidate(candidate)?;
            let trust = PackageTrustEvidence::exact_candidate(&record)?;
            trust.require_for_candidate(candidate)?;
            Ok(Some((record, trust)))
        }
    }
}

struct CurrentConformance {
    evidence: ConformanceEvidence,
    launch_profile: LaunchProfileEvidence,
}

fn select_current_conformance(
    candidate: &CandidateRecord,
    trust: &PackageTrustEvidence,
    launch_profiles: &LaunchProfileEvidenceStore,
    conformance: &ConformanceEvidenceStore,
) -> Result<Option<CurrentConformance>> {
    let all_launch = launch_profiles.load_all()?;
    let all_conformance = conformance.load_all()?;
    let suite_digest = current_suite_digest()?;

    let mut current: Vec<(ConformanceEvidence, LaunchProfileEvidence)> = Vec::new();

    for evidence in all_conformance {
        if evidence.disposition != ConformanceDisposition::Passed
            || evidence.candidate_id != candidate.candidate_id
        {
            continue;
        }

        let launch = match all_launch
            .iter()
            .find(|lp| lp.profile_evidence_digest == evidence.launch_profile_evidence_digest)
        {
            Some(lp) => lp,
            None => continue,
        };

        if launch.require_for_candidate(candidate).is_err() {
            continue;
        }

        if evidence
            .require_current(candidate, trust, launch, &suite_digest)
            .is_err()
        {
            continue;
        }

        current.push((evidence, launch.clone()));
    }

    if current.is_empty() {
        return Ok(None);
    }

    current.sort_by(|a, b| {
        b.0.ended_unix_ms
            .cmp(&a.0.ended_unix_ms)
            .then_with(|| b.0.evidence_id.cmp(&a.0.evidence_id))
    });

    Ok(Some(CurrentConformance {
        evidence: current[0].0.clone(),
        launch_profile: current[0].1.clone(),
    }))
}

fn check_approval(
    candidate: &CandidateRecord,
    trust: &PackageTrustEvidence,
    conformance: &CurrentConformance,
    approvals: &InstallationApprovalStore,
) -> Result<Option<InstallationApprovalRecord>> {
    let all = approvals.load_all()?;
    let candidate_approvals: Vec<&InstallationApprovalRecord> = all
        .iter()
        .filter(|a| a.candidate_id == candidate.candidate_id)
        .collect();

    if candidate_approvals.len() > 1 {
        return Err(M3Error::new(
            "installation_plan_conflict",
            "installation evidence is ambiguous",
        ));
    }

    match candidate_approvals.into_iter().next() {
        None => Ok(None),
        Some(approval) => {
            approval.validate()?;

            if approval.candidate_id != candidate.candidate_id
                || approval.package_id != candidate.package_id
                || approval.package_version != candidate.package_version
                || approval.semantic_package_digest != candidate.semantic_package_digest
                || approval.raw_archive_digest != candidate.raw_archive_digest
                || approval.trust_evidence.evidence_digest != trust.evidence_digest
                || approval.launch_profile_evidence_digest
                    != conformance.launch_profile.profile_evidence_digest
                || approval.conformance_evidence_id != conformance.evidence.evidence_id
                || approval.conformance_evidence_digest != conformance.evidence.evidence_digest
                || approval.provider_id != candidate.provider_id
                || approval.provider_version != candidate.provider_version
            {
                return Err(M3Error::new(
                    "installation_plan_stale",
                    "installation approval does not match current evidence",
                ));
            }

            Ok(Some(approval.clone()))
        }
    }
}

fn check_installed(
    candidate: &CandidateRecord,
    trust: &PackageTrustEvidence,
    conformance: &CurrentConformance,
    approval: &InstallationApprovalRecord,
    installed: &InstalledPlugRegistry,
) -> Result<Option<InstalledPlugRecord>> {
    let all = installed.load_all()?;
    let candidate_records: Vec<&InstalledPlugRecord> = all
        .iter()
        .filter(|r| r.source_candidate_id == candidate.candidate_id)
        .collect();

    if candidate_records.len() > 1 {
        return Err(M3Error::new(
            "installation_plan_conflict",
            "installation evidence is ambiguous",
        ));
    }

    match candidate_records.into_iter().next() {
        None => Ok(None),
        Some(record) => {
            record.validate()?;

            if record.source_candidate_id != candidate.candidate_id
                || record.state != "present_disabled"
                || record.package_id != candidate.package_id
                || record.package_version != candidate.package_version
                || record.semantic_package_digest != candidate.semantic_package_digest
                || record.raw_archive_digest != candidate.raw_archive_digest
                || record.trust_evidence.evidence_digest != trust.evidence_digest
                || record.installation_approval_id != approval.approval_id
                || record.installation_approval_digest != approval.record_digest
                || record.conformance_evidence_id != conformance.evidence.evidence_id
                || record.conformance_evidence_digest != conformance.evidence.evidence_digest
                || record.provider_id != candidate.provider_id
                || record.provider_version != candidate.provider_version
                || record.launch_profile_label != "supervised"
            {
                return Err(M3Error::new(
                    "installation_plan_stale",
                    "installed state does not match current evidence",
                ));
            }

            Ok(Some(record.clone()))
        }
    }
}

pub fn plan_installation(
    request: &InstallationRequest,
    candidates: &CandidateRegistry,
    exact_trust: &ExactCandidateTrustStore,
    launch_profiles: &LaunchProfileEvidenceStore,
    conformance: &ConformanceEvidenceStore,
    approvals: &InstallationApprovalStore,
    installed: &InstalledPlugRegistry,
) -> Result<InstallationPlan> {
    validate_request(request)?;
    let candidate = select_candidate(request, candidates)?;

    let empty_plan = || InstallationPlan {
        candidate_id: candidate.candidate_id.clone(),
        package_id: candidate.package_id.clone(),
        package_version: candidate.package_version.clone(),
        semantic_package_digest: candidate.semantic_package_digest.clone(),
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
    };

    let (trust_record, trust_evidence) = match find_exact_trust(&candidate, exact_trust)? {
        None => return Ok(empty_plan()),
        Some((record, evidence)) => (record, evidence),
    };

    let current = match select_current_conformance(
        &candidate,
        &trust_evidence,
        launch_profiles,
        conformance,
    )? {
        None => {
            return Ok(InstallationPlan {
                action: InstallationPlanAction::RunSupervisedConformance,
                exact_candidate_trust_record_digest: Some(trust_record.record_digest.clone()),
                trust_evidence_digest: Some(trust_evidence.evidence_digest.clone()),
                ..empty_plan()
            });
        }
        Some(current) => current,
    };

    let trust_plan = InstallationPlan {
        action: InstallationPlanAction::RunSupervisedConformance,
        exact_candidate_trust_record_digest: Some(trust_record.record_digest.clone()),
        trust_evidence_digest: Some(trust_evidence.evidence_digest.clone()),
        ..empty_plan()
    };

    let approval = match check_approval(&candidate, &trust_evidence, &current, approvals)? {
        None => {
            return Ok(InstallationPlan {
                action: InstallationPlanAction::CreateInstallationApproval,
                exact_candidate_trust_record_digest: trust_plan.exact_candidate_trust_record_digest,
                trust_evidence_digest: trust_plan.trust_evidence_digest,
                launch_profile_evidence_digest: Some(
                    current.launch_profile.profile_evidence_digest.clone(),
                ),
                conformance_evidence_id: Some(current.evidence.evidence_id.clone()),
                conformance_evidence_digest: Some(current.evidence.evidence_digest.clone()),
                ..empty_plan()
            });
        }
        Some(approval) => approval,
    };

    let installed = check_installed(&candidate, &trust_evidence, &current, &approval, installed)?;

    let plan = InstallationPlan {
        candidate_id: candidate.candidate_id.clone(),
        package_id: candidate.package_id.clone(),
        package_version: candidate.package_version.clone(),
        semantic_package_digest: candidate.semantic_package_digest.clone(),
        action: match &installed {
            None => InstallationPlanAction::PublishDisabledInstallation,
            Some(_) => InstallationPlanAction::Complete,
        },
        exact_candidate_trust_record_digest: trust_plan.exact_candidate_trust_record_digest,
        trust_evidence_digest: trust_plan.trust_evidence_digest,
        launch_profile_evidence_digest: Some(
            current.launch_profile.profile_evidence_digest.clone(),
        ),
        conformance_evidence_id: Some(current.evidence.evidence_id.clone()),
        conformance_evidence_digest: Some(current.evidence.evidence_digest.clone()),
        installation_approval_id: Some(approval.approval_id.clone()),
        installation_approval_digest: Some(approval.record_digest.clone()),
        installed_id: installed.as_ref().map(|r| r.installed_id.clone()),
        installed_record_digest: installed.as_ref().map(|r| r.record_digest.clone()),
    };

    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::installation_request::{
        InstallationConformanceRequest, InstallationTargetRequest, InstallationTrustRequest,
    };

    fn valid_request() -> InstallationRequest {
        InstallationRequest {
            schema: INSTALLATION_REQUEST_SCHEMA.to_owned(),
            candidate_id: "3d846d40-01fc-4e1e-b77d-83944dbed76f".to_owned(),
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
    fn validates_canonical_lowercase_hyphenated_uuid() {
        let req = valid_request();
        assert!(validate_request(&req).is_ok());

        let mut bad = valid_request();
        bad.candidate_id = "3D846D40-01FC-4E1E-B77D-83944DBED76F".to_owned();
        assert_eq!(
            validate_request(&bad).unwrap_err().code,
            "installation_plan_request_invalid"
        );

        let mut no_hyphens = valid_request();
        no_hyphens.candidate_id = "3d846d4001fc4e1eb77d83944dbed76f".to_owned();
        assert_eq!(
            validate_request(&no_hyphens).unwrap_err().code,
            "installation_plan_request_invalid"
        );
    }

    #[test]
    fn rejects_wrong_schema() {
        let mut req = valid_request();
        req.schema = "wrong-schema".to_owned();
        assert_eq!(
            validate_request(&req).unwrap_err().code,
            "installation_plan_request_invalid"
        );
    }

    #[test]
    fn rejects_false_supervised_approval() {
        let mut req = valid_request();
        req.conformance.allow_non_isolated_supervised_execution = false;
        assert_eq!(
            validate_request(&req).unwrap_err().code,
            "installation_plan_request_invalid"
        );
    }

    #[test]
    fn rejects_non_disabled_target_state() {
        let mut req = valid_request();
        req.installation.target_state = InstallationTargetState::Disabled;
        assert!(validate_request(&req).is_ok());
    }

    #[test]
    fn plan_action_enum_is_exhaustive() {
        assert_ne!(
            InstallationPlanAction::CreateExactCandidateTrust,
            InstallationPlanAction::RunSupervisedConformance
        );
        assert_ne!(
            InstallationPlanAction::CreateExactCandidateTrust,
            InstallationPlanAction::CreateInstallationApproval
        );
        assert_ne!(
            InstallationPlanAction::CreateExactCandidateTrust,
            InstallationPlanAction::PublishDisabledInstallation
        );
        assert_ne!(
            InstallationPlanAction::CreateExactCandidateTrust,
            InstallationPlanAction::Complete
        );
    }
}
