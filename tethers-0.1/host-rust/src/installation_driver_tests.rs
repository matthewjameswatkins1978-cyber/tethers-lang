use super::installation_driver::{drive_with, InstallationDriveStop};
use crate::conformance::ConformanceDisposition;
use crate::installation_execution::{InstallationStepOutcome, InstallationStepResult};
use crate::installation_plan::{InstallationPlan, InstallationPlanAction};
use crate::m3_store::M3Error;

fn plan_with(action: InstallationPlanAction, candidate_id: &str) -> InstallationPlan {
    InstallationPlan {
        candidate_id: candidate_id.to_string(),
        package_id: "tether".to_string(),
        package_version: "1.0.0".to_string(),
        semantic_package_digest: "sha256:aa".to_string(),
        action,
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

fn step_result(
    before_action: InstallationPlanAction,
    after_action: InstallationPlanAction,
    outcome: InstallationStepOutcome,
) -> InstallationStepResult {
    let candidate_id = if before_action == InstallationPlanAction::Complete {
        "complete-id"
    } else {
        "candidate-1"
    };
    InstallationStepResult {
        before: plan_with(before_action, candidate_id),
        after: plan_with(after_action, candidate_id),
        outcome,
    }
}

fn advanced_step(
    before_action: InstallationPlanAction,
    after_action: InstallationPlanAction,
    executed: InstallationPlanAction,
) -> InstallationStepResult {
    step_result(
        before_action,
        after_action,
        InstallationStepOutcome::Advanced { executed },
    )
}

fn already_complete_step() -> InstallationStepResult {
    step_result(
        InstallationPlanAction::Complete,
        InstallationPlanAction::Complete,
        InstallationStepOutcome::AlreadyComplete,
    )
}

fn conformance_without_advance_step(
    evidence_id: &str,
    disposition: ConformanceDisposition,
) -> InstallationStepResult {
    step_result(
        InstallationPlanAction::RunSupervisedConformance,
        InstallationPlanAction::RunSupervisedConformance,
        InstallationStepOutcome::ConformanceRecordedWithoutAdvance {
            evidence_id: evidence_id.to_string(),
            disposition,
        },
    )
}

#[test]
fn j24l1_already_complete_stops_after_one_call() {
    let mut call_count = 0u32;
    let captured_step = already_complete_step();
    let step_clone = captured_step.clone();

    let result = drive_with(|| {
        call_count += 1;
        Ok(step_clone.clone())
    })
    .unwrap();

    assert_eq!(call_count, 1);
    assert_eq!(result.stop, InstallationDriveStop::Complete);
    assert_eq!(result.steps.len(), 1);
    assert_eq!(result.steps[0], captured_step);
}

#[test]
fn j24l1_advanced_to_complete_stops_without_confirmation_call() {
    let mut call_count = 0u32;
    let step = advanced_step(
        InstallationPlanAction::PublishDisabledInstallation,
        InstallationPlanAction::Complete,
        InstallationPlanAction::PublishDisabledInstallation,
    );
    let step_clone = step.clone();

    let result = drive_with(|| {
        call_count += 1;
        Ok(step_clone.clone())
    })
    .unwrap();

    assert_eq!(call_count, 1);
    assert_eq!(result.stop, InstallationDriveStop::Complete);
    assert_eq!(result.steps.len(), 1);
    assert_eq!(result.steps[0], step);
}

#[test]
fn j24l1_fresh_sequence_completes_in_exactly_four_calls() {
    let steps = vec![
        advanced_step(
            InstallationPlanAction::CreateExactCandidateTrust,
            InstallationPlanAction::RunSupervisedConformance,
            InstallationPlanAction::CreateExactCandidateTrust,
        ),
        advanced_step(
            InstallationPlanAction::RunSupervisedConformance,
            InstallationPlanAction::CreateInstallationApproval,
            InstallationPlanAction::RunSupervisedConformance,
        ),
        advanced_step(
            InstallationPlanAction::CreateInstallationApproval,
            InstallationPlanAction::PublishDisabledInstallation,
            InstallationPlanAction::CreateInstallationApproval,
        ),
        advanced_step(
            InstallationPlanAction::PublishDisabledInstallation,
            InstallationPlanAction::Complete,
            InstallationPlanAction::PublishDisabledInstallation,
        ),
    ];

    let mut step_iter = steps.into_iter();
    let mut call_count = 0u32;

    let result = drive_with(|| {
        call_count += 1;
        step_iter
            .next()
            .ok_or_else(|| panic!("fifth call should not occur"))
            .map(Ok)?
    })
    .unwrap();

    assert_eq!(call_count, 4);
    assert_eq!(result.stop, InstallationDriveStop::Complete);
    assert_eq!(result.steps.len(), 4);
    assert_eq!(
        result.steps[0].outcome,
        InstallationStepOutcome::Advanced {
            executed: InstallationPlanAction::CreateExactCandidateTrust,
        }
    );
    assert_eq!(
        result.steps[1].outcome,
        InstallationStepOutcome::Advanced {
            executed: InstallationPlanAction::RunSupervisedConformance,
        }
    );
    assert_eq!(
        result.steps[2].outcome,
        InstallationStepOutcome::Advanced {
            executed: InstallationPlanAction::CreateInstallationApproval,
        }
    );
    assert_eq!(
        result.steps[3].outcome,
        InstallationStepOutcome::Advanced {
            executed: InstallationPlanAction::PublishDisabledInstallation,
        }
    );
}

#[test]
fn j24l1_conformance_without_advance_stops_immediately() {
    let mut call_count = 0u32;
    let step = conformance_without_advance_step("ev-abc", ConformanceDisposition::Failed);
    let step_clone = step.clone();

    let result = drive_with(|| {
        call_count += 1;
        Ok(step_clone.clone())
    })
    .unwrap();

    assert_eq!(call_count, 1);
    assert_eq!(
        result.stop,
        InstallationDriveStop::ConformanceRecordedWithoutAdvance
    );
    assert_eq!(result.steps.len(), 1);
    assert_eq!(
        result.steps[0].outcome,
        InstallationStepOutcome::ConformanceRecordedWithoutAdvance {
            evidence_id: "ev-abc".to_string(),
            disposition: ConformanceDisposition::Failed,
        }
    );
    assert_eq!(result.steps[0], step);
}

#[test]
fn j24l1_executor_error_propagates_without_another_call() {
    let mut call_count = 0u32;

    let error = drive_with(|| {
        call_count += 1;
        Err(M3Error::new("test_code", "test message"))
    })
    .unwrap_err();

    assert_eq!(call_count, 1);
    assert_eq!(error.code, "test_code");
    assert_eq!(error.message, "test message");
}

#[test]
fn j24l1_four_noncomplete_advances_hit_exact_iteration_limit() {
    let mut call_count = 0u32;

    let error = drive_with(|| {
        call_count += 1;
        Ok(advanced_step(
            InstallationPlanAction::CreateExactCandidateTrust,
            InstallationPlanAction::RunSupervisedConformance,
            InstallationPlanAction::CreateExactCandidateTrust,
        ))
    })
    .unwrap_err();

    assert_eq!(call_count, 4);
    assert_eq!(error.code, "installation_iteration_limit");
    assert_eq!(
        error.message,
        "installation did not complete within four executor calls"
    );
}

#[test]
fn j24l1_preserves_returned_steps_without_rewriting() {
    let candidate_id = "dist-cand-42";
    let trust_digest = Some("sha256:ttt".to_string());
    let evidence_digest = Some("sha256:eee".to_string());
    let launch_digest = Some("sha256:lll".to_string());
    let conformance_id = Some("conf-x".to_string());
    let conformance_digest = Some("sha256:ccc".to_string());
    let approval_id = Some("appr-y".to_string());
    let approval_digest = Some("sha256:aaa".to_string());

    let step1 = InstallationStepResult {
        before: InstallationPlan {
            action: InstallationPlanAction::CreateExactCandidateTrust,
            candidate_id: candidate_id.to_string(),
            package_id: "pkg-dist".to_string(),
            package_version: "2.0.0".to_string(),
            semantic_package_digest: "sha256:bb".to_string(),
            exact_candidate_trust_record_digest: None,
            trust_evidence_digest: None,
            launch_profile_evidence_digest: None,
            conformance_evidence_id: None,
            conformance_evidence_digest: None,
            installation_approval_id: None,
            installation_approval_digest: None,
            installed_id: None,
            installed_record_digest: None,
        },
        after: InstallationPlan {
            action: InstallationPlanAction::RunSupervisedConformance,
            candidate_id: candidate_id.to_string(),
            package_id: "pkg-dist".to_string(),
            package_version: "2.0.0".to_string(),
            semantic_package_digest: "sha256:bb".to_string(),
            exact_candidate_trust_record_digest: trust_digest.clone(),
            trust_evidence_digest: evidence_digest.clone(),
            launch_profile_evidence_digest: None,
            conformance_evidence_id: None,
            conformance_evidence_digest: None,
            installation_approval_id: None,
            installation_approval_digest: None,
            installed_id: None,
            installed_record_digest: None,
        },
        outcome: InstallationStepOutcome::Advanced {
            executed: InstallationPlanAction::CreateExactCandidateTrust,
        },
    };

    let step2 = InstallationStepResult {
        before: InstallationPlan {
            action: InstallationPlanAction::RunSupervisedConformance,
            candidate_id: candidate_id.to_string(),
            package_id: "pkg-dist".to_string(),
            package_version: "2.0.0".to_string(),
            semantic_package_digest: "sha256:bb".to_string(),
            exact_candidate_trust_record_digest: trust_digest.clone(),
            trust_evidence_digest: evidence_digest.clone(),
            launch_profile_evidence_digest: None,
            conformance_evidence_id: None,
            conformance_evidence_digest: None,
            installation_approval_id: None,
            installation_approval_digest: None,
            installed_id: None,
            installed_record_digest: None,
        },
        after: InstallationPlan {
            action: InstallationPlanAction::Complete,
            candidate_id: candidate_id.to_string(),
            package_id: "pkg-dist".to_string(),
            package_version: "2.0.0".to_string(),
            semantic_package_digest: "sha256:bb".to_string(),
            exact_candidate_trust_record_digest: trust_digest.clone(),
            trust_evidence_digest: evidence_digest.clone(),
            launch_profile_evidence_digest: launch_digest.clone(),
            conformance_evidence_id: conformance_id.clone(),
            conformance_evidence_digest: conformance_digest.clone(),
            installation_approval_id: approval_id.clone(),
            installation_approval_digest: approval_digest.clone(),
            installed_id: Some("inst-99".to_string()),
            installed_record_digest: Some("sha256:zzz".to_string()),
        },
        outcome: InstallationStepOutcome::Advanced {
            executed: InstallationPlanAction::PublishDisabledInstallation,
        },
    };

    let sequence: Vec<std::result::Result<InstallationStepResult, M3Error>> =
        vec![Ok(step1.clone()), Ok(step2.clone())];
    let mut iter = sequence.into_iter();
    let mut call_count = 0u32;

    let result = drive_with(|| {
        call_count += 1;
        iter.next()
            .ok_or_else(|| panic!("fifth call should not occur"))
            .unwrap()
    })
    .unwrap();

    assert_eq!(call_count, 2);
    assert_eq!(result.stop, InstallationDriveStop::Complete);
    assert_eq!(result.steps.len(), 2);
    assert_eq!(result.steps[0], step1);
    assert_eq!(result.steps[1], step2);
}
