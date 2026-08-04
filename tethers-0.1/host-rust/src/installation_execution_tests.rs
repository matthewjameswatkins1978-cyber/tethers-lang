use super::installation_execution::{validate_options, InstallationExecutionOptions};
use crate::installation_plan::{InstallationPlan, InstallationPlanAction};
use std::time::Duration;

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
