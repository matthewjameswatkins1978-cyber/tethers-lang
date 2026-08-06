use crate::candidate::CandidateRegistry;
use crate::cli::CliEnvelope;
use crate::cli::OutcomeStatus;
use crate::conformance::{ConformanceDisposition, ConformanceEvidenceStore};
use crate::enablement::EnablementStore;
use crate::installation_driver::{drive_installation, InstallationDriveStop};
use crate::installation_execution::{
    InstallationExecutionContext, InstallationExecutionOptions, InstallationStepOutcome,
};
use crate::installation_plan::InstallationPlanAction;
use crate::installation_request::load_installation_request;
use crate::installation_trust::ExactCandidateTrustStore;
use crate::installed::{InstallationApprovalStore, InstalledPlugRegistry};
use crate::launch_profile::LaunchProfileEvidenceStore;
use crate::m3_store::verify_chain;
use crate::plug_command::PlugCommandResult;
use serde_json::json;
use std::path::Path;
use std::time::Duration;

const INSTALL_APPROVING_AUTHORITY: &str = "tethers-reference-host-cli";
const INSTALL_CONFORMANCE_WALL_TIME: Duration = Duration::from_secs(30);

pub(crate) fn run_install(host_data_root: &Path, request_path: &Path) -> PlugCommandResult {
    if !host_data_root.is_absolute() {
        let envelope = CliEnvelope::error(
            "plug install",
            OutcomeStatus::InvalidCliUsage,
            "invalid_cli_usage",
            "--host-data-root must be absolute",
            Some("/host-data-root".into()),
        );
        return PlugCommandResult {
            exit_code: envelope.exit_code,
            envelope,
        };
    }
    if !request_path.is_absolute() {
        let envelope = CliEnvelope::error(
            "plug install",
            OutcomeStatus::InvalidCliUsage,
            "invalid_cli_usage",
            "--request must be absolute",
            Some("/request".into()),
        );
        return PlugCommandResult {
            exit_code: envelope.exit_code,
            envelope,
        };
    }

    match std::fs::symlink_metadata(host_data_root) {
        Ok(metadata) if metadata.is_dir() => {}
        _ => {
            let envelope = CliEnvelope::error(
                "plug install",
                OutcomeStatus::Unavailable,
                "plug_data_root_unavailable",
                "host data root is unavailable",
                None,
            );
            return PlugCommandResult {
                exit_code: envelope.exit_code,
                envelope,
            };
        }
    }

    if let Err(error) = verify_chain(host_data_root) {
        let envelope = CliEnvelope::error(
            "plug install",
            OutcomeStatus::InvalidData,
            error.code,
            error.message,
            None,
        );
        return PlugCommandResult {
            exit_code: envelope.exit_code,
            envelope,
        };
    }

    let request = match load_installation_request(request_path) {
        Ok(r) => r,
        Err(error) => {
            let status = if error.code == "installation_request_io" {
                OutcomeStatus::Unavailable
            } else {
                OutcomeStatus::InvalidData
            };
            let field: Option<String> = error.field;
            let envelope =
                CliEnvelope::error("plug install", status, error.code, error.message, field);
            return PlugCommandResult {
                exit_code: envelope.exit_code,
                envelope,
            };
        }
    };

    let candidate_root = host_data_root.join("candidates");
    let quarantine_root = host_data_root.join("quarantine");

    let candidates = match CandidateRegistry::open_existing(&candidate_root, &quarantine_root) {
        Ok(c) => c,
        Err(error) => {
            let status = match error.code {
                "archive_read" | "candidate_io" => OutcomeStatus::Unavailable,
                "candidate_rollback_failed" | "clock" => OutcomeStatus::Failed,
                _ => OutcomeStatus::InvalidData,
            };
            let envelope =
                CliEnvelope::error("plug install", status, error.code, error.message, None);
            return PlugCommandResult {
                exit_code: envelope.exit_code,
                envelope,
            };
        }
    };

    let exact_trust =
        match ExactCandidateTrustStore::open(&host_data_root.join("installation-trust")) {
            Ok(s) => s,
            Err(error) => {
                let status = if error.code == "store_io" {
                    OutcomeStatus::Unavailable
                } else {
                    OutcomeStatus::InvalidData
                };
                let envelope =
                    CliEnvelope::error("plug install", status, error.code, error.message, None);
                return PlugCommandResult {
                    exit_code: envelope.exit_code,
                    envelope,
                };
            }
        };

    let launch_profiles =
        match LaunchProfileEvidenceStore::open(&host_data_root.join("launch-profiles")) {
            Ok(s) => s,
            Err(error) => {
                let status = if error.code == "store_io" {
                    OutcomeStatus::Unavailable
                } else {
                    OutcomeStatus::InvalidData
                };
                let envelope =
                    CliEnvelope::error("plug install", status, error.code, error.message, None);
                return PlugCommandResult {
                    exit_code: envelope.exit_code,
                    envelope,
                };
            }
        };

    let conformance = match ConformanceEvidenceStore::open(&host_data_root.join("conformance")) {
        Ok(s) => s,
        Err(error) => {
            let status = if error.code == "store_io" {
                OutcomeStatus::Unavailable
            } else {
                OutcomeStatus::InvalidData
            };
            let envelope =
                CliEnvelope::error("plug install", status, error.code, error.message, None);
            return PlugCommandResult {
                exit_code: envelope.exit_code,
                envelope,
            };
        }
    };

    let approvals =
        match InstallationApprovalStore::open(&host_data_root.join("installation-approvals")) {
            Ok(s) => s,
            Err(error) => {
                let status = if error.code == "store_io" {
                    OutcomeStatus::Unavailable
                } else {
                    OutcomeStatus::InvalidData
                };
                let envelope =
                    CliEnvelope::error("plug install", status, error.code, error.message, None);
                return PlugCommandResult {
                    exit_code: envelope.exit_code,
                    envelope,
                };
            }
        };

    let installed = match InstalledPlugRegistry::open(
        &host_data_root.join("install"),
        &host_data_root.join("installed-records"),
    ) {
        Ok(s) => s,
        Err(error) => {
            let status = if error.code == "store_io" {
                OutcomeStatus::Unavailable
            } else {
                OutcomeStatus::InvalidData
            };
            let envelope =
                CliEnvelope::error("plug install", status, error.code, error.message, None);
            return PlugCommandResult {
                exit_code: envelope.exit_code,
                envelope,
            };
        }
    };

    if let Err(error) = EnablementStore::open(&host_data_root.join("enablements")) {
        let status = if error.code == "store_io" {
            OutcomeStatus::Unavailable
        } else {
            OutcomeStatus::InvalidData
        };
        let envelope = CliEnvelope::error("plug install", status, error.code, error.message, None);
        return PlugCommandResult {
            exit_code: envelope.exit_code,
            envelope,
        };
    }

    let lock_path = host_data_root.join("installation.lock");
    let executor_state_root = host_data_root.to_path_buf();
    let conformance_scratch_root = host_data_root.join("conformance-scratch");

    let context = InstallationExecutionContext {
        lock_path: &lock_path,
        executor_state_root: &executor_state_root,
        quarantine_root: &quarantine_root,
        conformance_scratch_root: &conformance_scratch_root,
        candidates: &candidates,
        exact_trust: &exact_trust,
        launch_profiles: &launch_profiles,
        conformance: &conformance,
        approvals: &approvals,
        installed: &installed,
    };

    let host_build_identity = concat!("tethers-reference-host/", env!("CARGO_PKG_VERSION"));
    let options = InstallationExecutionOptions {
        approving_authority: INSTALL_APPROVING_AUTHORITY,
        host_build_identity,
        conformance_wall_time: INSTALL_CONFORMANCE_WALL_TIME,
    };

    let request_candidate_id = request.candidate_id.clone();

    match drive_installation(&request, &context, &options) {
        Ok(result) => map_drive_result(result, &request_candidate_id),
        Err(error) => {
            let status = error_code_to_status(error.code);
            let envelope =
                CliEnvelope::error("plug install", status, error.code, error.message, None);
            PlugCommandResult {
                exit_code: envelope.exit_code,
                envelope,
            }
        }
    }
}

fn error_code_to_status(code: &str) -> OutcomeStatus {
    match code {
        "installation_request_io"
        | "candidate_io"
        | "store_io"
        | "installation_busy"
        | "installation_lock_io"
        | "installation_recovery_io" => OutcomeStatus::Unavailable,
        "installation_iteration_limit"
        | "installation_execution_stagnant"
        | "installation_execution_regressed"
        | "installation_execution_invalid_transition"
        | "installation_execution_postcondition_failed"
        | "installation_scratch_cleanup_failed" => OutcomeStatus::Failed,
        _ => OutcomeStatus::InvalidData,
    }
}

fn action_name(action: &InstallationPlanAction) -> &'static str {
    match action {
        InstallationPlanAction::CreateExactCandidateTrust => "create_exact_candidate_trust",
        InstallationPlanAction::RunSupervisedConformance => "run_supervised_conformance",
        InstallationPlanAction::CreateInstallationApproval => "create_installation_approval",
        InstallationPlanAction::PublishDisabledInstallation => "publish_disabled_installation",
        InstallationPlanAction::Complete => "complete",
    }
}

fn map_step(
    step: &crate::installation_execution::InstallationStepResult,
) -> Result<serde_json::Value, &'static str> {
    match &step.outcome {
        InstallationStepOutcome::AlreadyComplete => Ok(json!({
            "before_action": action_name(&step.before.action),
            "after_action": action_name(&step.after.action),
            "outcome": "already_complete"
        })),
        InstallationStepOutcome::Advanced { executed } => Ok(json!({
            "before_action": action_name(&step.before.action),
            "after_action": action_name(&step.after.action),
            "outcome": "advanced",
            "executed_action": action_name(executed)
        })),
        InstallationStepOutcome::ConformanceRecordedWithoutAdvance {
            evidence_id,
            disposition,
        } => match disposition {
            ConformanceDisposition::Failed => Ok(json!({
                "before_action": action_name(&step.before.action),
                "after_action": action_name(&step.after.action),
                "outcome": "conformance_recorded_without_advance",
                "evidence_id": evidence_id,
                "conformance_disposition": "failed"
            })),
            ConformanceDisposition::Interrupted => Ok(json!({
                "before_action": action_name(&step.before.action),
                "after_action": action_name(&step.after.action),
                "outcome": "conformance_recorded_without_advance",
                "evidence_id": evidence_id,
                "conformance_disposition": "interrupted"
            })),
            _ => Err("non-advancing conformance result was contradictory"),
        },
    }
}

fn collect_steps(
    steps: &[crate::installation_execution::InstallationStepResult],
) -> Result<Vec<serde_json::Value>, &'static str> {
    steps.iter().map(map_step).collect()
}

fn map_drive_result(
    result: crate::installation_driver::InstallationDriveResult,
    candidate_id: &str,
) -> PlugCommandResult {
    match result.stop {
        InstallationDriveStop::Complete => map_complete(result, candidate_id),
        InstallationDriveStop::ConformanceRecordedWithoutAdvance => {
            map_conformance_stop(result, candidate_id)
        }
    }
}

fn map_complete(
    result: crate::installation_driver::InstallationDriveResult,
    candidate_id: &str,
) -> PlugCommandResult {
    let last_step = match result.steps.last() {
        Some(s) => s,
        None => {
            return contradiction_error(
                "completed installation result is missing installed evidence",
            );
        }
    };

    let installed_id = match &last_step.after.installed_id {
        Some(id) => id.clone(),
        None => {
            return contradiction_error(
                "completed installation result is missing installed evidence",
            );
        }
    };

    let installed_record_digest = match &last_step.after.installed_record_digest {
        Some(d) => d.clone(),
        None => {
            return contradiction_error(
                "completed installation result is missing installed evidence",
            );
        }
    };

    let steps = match collect_steps(&result.steps) {
        Ok(s) => s,
        Err(_) => return contradiction_error("non-advancing conformance result was contradictory"),
    };
    let step_count = steps.len();

    let data = json!({
        "result": "complete",
        "candidate_id": candidate_id,
        "step_count": step_count,
        "steps": steps,
        "installed_id": installed_id,
        "installed_record_digest": installed_record_digest,
    });

    let envelope = CliEnvelope::ok("plug install", data);
    PlugCommandResult {
        exit_code: envelope.exit_code,
        envelope,
    }
}

fn contradict_non_advancing() -> PlugCommandResult {
    contradiction_error("non-advancing conformance result was contradictory")
}

fn contradiction_error(message: &str) -> PlugCommandResult {
    let envelope = CliEnvelope::error(
        "plug install",
        OutcomeStatus::Failed,
        "installation_execution_postcondition_failed",
        message,
        None,
    );
    PlugCommandResult {
        exit_code: envelope.exit_code,
        envelope,
    }
}

fn map_conformance_stop(
    result: crate::installation_driver::InstallationDriveResult,
    candidate_id: &str,
) -> PlugCommandResult {
    let last_step = match result.steps.last() {
        Some(s) => s,
        None => return contradict_non_advancing(),
    };

    let disposition = match &last_step.outcome {
        InstallationStepOutcome::ConformanceRecordedWithoutAdvance { disposition, .. } => {
            disposition
        }
        _ => return contradict_non_advancing(),
    };

    if matches!(
        disposition,
        ConformanceDisposition::Passed | ConformanceDisposition::Invalidated
    ) {
        return contradict_non_advancing();
    }

    let steps = match collect_steps(&result.steps) {
        Ok(s) => s,
        Err(_) => return contradict_non_advancing(),
    };
    let step_count = steps.len();

    let (status, code, message) = match disposition {
        ConformanceDisposition::Failed => (
            OutcomeStatus::Failed,
            "installation_conformance_failed",
            "supervised conformance did not pass",
        ),
        ConformanceDisposition::Interrupted => (
            OutcomeStatus::Interrupted,
            "installation_conformance_interrupted",
            "supervised conformance was interrupted",
        ),
        ConformanceDisposition::Passed | ConformanceDisposition::Invalidated => {
            return contradict_non_advancing();
        }
    };

    let data = json!({
        "result": "conformance_recorded_without_advance",
        "candidate_id": candidate_id,
        "step_count": step_count,
        "steps": steps,
    });

    let envelope = CliEnvelope::error_with_data("plug install", status, code, message, None, data);

    PlugCommandResult {
        exit_code: envelope.exit_code,
        envelope,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conformance::ConformanceDisposition;
    use crate::installation_driver::{InstallationDriveResult, InstallationDriveStop};
    use crate::installation_execution::{
        InstallationStepOutcome, InstallationStepResult as ExecStepResult,
    };
    use crate::installation_plan::{InstallationPlan, InstallationPlanAction};
    use crate::m3_store::M3Error;
    use std::path::PathBuf;

    fn plan_with(action: InstallationPlanAction) -> InstallationPlan {
        InstallationPlan {
            candidate_id: "test-candidate".to_string(),
            package_id: "test-pkg".to_string(),
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

    fn advanced(
        before_action: InstallationPlanAction,
        after_action: InstallationPlanAction,
        executed: InstallationPlanAction,
    ) -> ExecStepResult {
        ExecStepResult {
            before: plan_with(before_action),
            after: plan_with(after_action),
            outcome: InstallationStepOutcome::Advanced { executed },
        }
    }

    fn already_complete() -> ExecStepResult {
        let mut plan = plan_with(InstallationPlanAction::Complete);
        plan.installed_id = Some("inst-42".to_string());
        plan.installed_record_digest = Some("sha256:abc".to_string());
        ExecStepResult {
            before: plan.clone(),
            after: plan,
            outcome: InstallationStepOutcome::AlreadyComplete,
        }
    }

    #[test]
    fn j24l2_completed_four_step_maps_action_strings_and_installed_evidence() {
        let mut plan = plan_with(InstallationPlanAction::Complete);
        plan.installed_id = Some("inst-99".to_string());
        plan.installed_record_digest = Some("sha256:zzz".to_string());

        let steps = vec![
            advanced(
                InstallationPlanAction::CreateExactCandidateTrust,
                InstallationPlanAction::RunSupervisedConformance,
                InstallationPlanAction::CreateExactCandidateTrust,
            ),
            advanced(
                InstallationPlanAction::RunSupervisedConformance,
                InstallationPlanAction::CreateInstallationApproval,
                InstallationPlanAction::RunSupervisedConformance,
            ),
            advanced(
                InstallationPlanAction::CreateInstallationApproval,
                InstallationPlanAction::PublishDisabledInstallation,
                InstallationPlanAction::CreateInstallationApproval,
            ),
            ExecStepResult {
                before: plan_with(InstallationPlanAction::PublishDisabledInstallation),
                after: plan.clone(),
                outcome: InstallationStepOutcome::Advanced {
                    executed: InstallationPlanAction::PublishDisabledInstallation,
                },
            },
        ];

        let result = crate::installation_driver::InstallationDriveResult {
            steps,
            stop: InstallationDriveStop::Complete,
        };

        let output = map_drive_result(result, "test-candidate");
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.envelope.status, OutcomeStatus::Ok);

        let data = &output.envelope.data;
        assert_eq!(data["result"], "complete");
        assert_eq!(data["candidate_id"], "test-candidate");
        assert_eq!(data["step_count"], 4);
        assert_eq!(data["steps"].as_array().unwrap().len(), 4);
        assert_eq!(data["installed_id"], "inst-99");
        assert_eq!(data["installed_record_digest"], "sha256:zzz");

        let step0 = &data["steps"][0];
        assert_eq!(step0["before_action"], "create_exact_candidate_trust");
        assert_eq!(step0["after_action"], "run_supervised_conformance");
        assert_eq!(step0["outcome"], "advanced");
        assert_eq!(step0["executed_action"], "create_exact_candidate_trust");
    }

    #[test]
    fn j24l2_already_complete_maps_success() {
        let step = already_complete();
        let result = InstallationDriveResult {
            steps: vec![step],
            stop: InstallationDriveStop::Complete,
        };

        let output = map_drive_result(result, "test-candidate");
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.envelope.status, OutcomeStatus::Ok);

        let data = &output.envelope.data;
        assert_eq!(data["result"], "complete");
        assert_eq!(data["candidate_id"], "test-candidate");
        assert_eq!(data["step_count"], 1);
        assert_eq!(data["steps"].as_array().unwrap().len(), 1);
        assert_eq!(data["installed_id"], "inst-42");
        assert_eq!(data["installed_record_digest"], "sha256:abc");

        let step0 = &data["steps"][0];
        assert_eq!(step0["before_action"], "complete");
        assert_eq!(step0["after_action"], "complete");
        assert_eq!(step0["outcome"], "already_complete");
    }

    #[test]
    fn j24l2_failed_conformance_maps_status_6() {
        let evidence_id = "ev-1".to_string();
        let disposition = ConformanceDisposition::Failed;
        let step = ExecStepResult {
            before: plan_with(InstallationPlanAction::RunSupervisedConformance),
            after: plan_with(InstallationPlanAction::RunSupervisedConformance),
            outcome: InstallationStepOutcome::ConformanceRecordedWithoutAdvance {
                evidence_id: evidence_id.clone(),
                disposition,
            },
        };

        let result = InstallationDriveResult {
            steps: vec![step],
            stop: InstallationDriveStop::ConformanceRecordedWithoutAdvance,
        };

        let output = map_drive_result(result, "test-candidate");
        assert_eq!(output.exit_code, 6);
        assert_eq!(output.envelope.status, OutcomeStatus::Failed);
        assert_eq!(
            output.envelope.error.as_ref().unwrap().code,
            "installation_conformance_failed"
        );
        assert_eq!(
            output.envelope.error.as_ref().unwrap().message,
            "supervised conformance did not pass"
        );

        let data = &output.envelope.data;
        assert_eq!(data["result"], "conformance_recorded_without_advance");
        assert_eq!(data["candidate_id"], "test-candidate");
        assert_eq!(data["step_count"], 1);
    }

    #[test]
    fn j24l2_interrupted_conformance_maps_status_10() {
        let evidence_id = "ev-int".to_string();
        let disposition = ConformanceDisposition::Interrupted;
        let step = ExecStepResult {
            before: plan_with(InstallationPlanAction::RunSupervisedConformance),
            after: plan_with(InstallationPlanAction::RunSupervisedConformance),
            outcome: InstallationStepOutcome::ConformanceRecordedWithoutAdvance {
                evidence_id: evidence_id.clone(),
                disposition,
            },
        };

        let result = InstallationDriveResult {
            steps: vec![step],
            stop: InstallationDriveStop::ConformanceRecordedWithoutAdvance,
        };

        let output = map_drive_result(result, "test-candidate");
        assert_eq!(output.exit_code, 10);
        assert_eq!(output.envelope.status, OutcomeStatus::Interrupted);
        assert_eq!(
            output.envelope.error.as_ref().unwrap().code,
            "installation_conformance_interrupted"
        );
        assert_eq!(
            output.envelope.error.as_ref().unwrap().message,
            "supervised conformance was interrupted"
        );
    }

    #[test]
    fn j24l2_contradictory_passed_non_advance_fails_closed() {
        let step = ExecStepResult {
            before: plan_with(InstallationPlanAction::RunSupervisedConformance),
            after: plan_with(InstallationPlanAction::RunSupervisedConformance),
            outcome: InstallationStepOutcome::ConformanceRecordedWithoutAdvance {
                evidence_id: "ev-pass".to_string(),
                disposition: ConformanceDisposition::Passed,
            },
        };

        let result = InstallationDriveResult {
            steps: vec![step],
            stop: InstallationDriveStop::ConformanceRecordedWithoutAdvance,
        };

        let output = map_drive_result(result, "test-candidate");
        assert_eq!(output.exit_code, 6);
        assert_eq!(output.envelope.status, OutcomeStatus::Failed);
        assert_eq!(
            output.envelope.error.as_ref().unwrap().code,
            "installation_execution_postcondition_failed"
        );
        assert_eq!(
            output.envelope.error.as_ref().unwrap().message,
            "non-advancing conformance result was contradictory"
        );
    }

    #[test]
    fn j24l2_missing_installed_pins_fails_closed() {
        let step = advanced(
            InstallationPlanAction::PublishDisabledInstallation,
            InstallationPlanAction::Complete,
            InstallationPlanAction::PublishDisabledInstallation,
        );
        // step.after has no installed_id or installed_record_digest

        let result = InstallationDriveResult {
            steps: vec![step],
            stop: InstallationDriveStop::Complete,
        };

        let output = map_drive_result(result, "test-candidate");
        assert_eq!(output.exit_code, 6);
        assert_eq!(output.envelope.status, OutcomeStatus::Failed);
        assert_eq!(
            output.envelope.error.as_ref().unwrap().code,
            "installation_execution_postcondition_failed"
        );
    }

    #[test]
    fn j24l2_error_status_mapping_follows_explicit_table() {
        for (code, expected) in [
            ("installation_request_io", OutcomeStatus::Unavailable),
            ("candidate_io", OutcomeStatus::Unavailable),
            ("store_io", OutcomeStatus::Unavailable),
            ("installation_busy", OutcomeStatus::Unavailable),
            ("installation_lock_io", OutcomeStatus::Unavailable),
            ("installation_recovery_io", OutcomeStatus::Unavailable),
            ("installation_iteration_limit", OutcomeStatus::Failed),
            ("installation_execution_stagnant", OutcomeStatus::Failed),
            ("installation_execution_regressed", OutcomeStatus::Failed),
            (
                "installation_execution_invalid_transition",
                OutcomeStatus::Failed,
            ),
            (
                "installation_execution_postcondition_failed",
                OutcomeStatus::Failed,
            ),
            ("installation_scratch_cleanup_failed", OutcomeStatus::Failed),
        ] {
            assert_eq!(
                error_code_to_status(code),
                expected,
                "code {code} mapped incorrectly"
            );
        }
    }

    #[test]
    fn j24l2_unlisted_errors_default_to_invalid_data() {
        assert_eq!(
            error_code_to_status("some_unknown_error"),
            OutcomeStatus::InvalidData
        );
        assert_eq!(error_code_to_status("trust_io"), OutcomeStatus::InvalidData);
    }

    #[test]
    fn j24l2_optional_step_fields_omitted_not_null() {
        let step = already_complete();
        let mapped = map_step(&step).unwrap();
        let json_str = serde_json::to_string(&mapped).unwrap();
        // AlreadyComplete should have no executed_action field
        assert!(!json_str.contains("executed_action"));
        // But should have the three basic fields
        assert_eq!(mapped["outcome"], "already_complete");
        assert!(mapped.get("evidence_id").is_none());
        assert!(mapped.get("conformance_disposition").is_none());
    }

    #[test]
    fn j24l2_conformance_step_includes_optional_fields() {
        let step = ExecStepResult {
            before: plan_with(InstallationPlanAction::RunSupervisedConformance),
            after: plan_with(InstallationPlanAction::RunSupervisedConformance),
            outcome: InstallationStepOutcome::ConformanceRecordedWithoutAdvance {
                evidence_id: "ev-abc".to_string(),
                disposition: ConformanceDisposition::Failed,
            },
        };
        let mapped = map_step(&step).unwrap();
        assert_eq!(mapped["outcome"], "conformance_recorded_without_advance");
        assert_eq!(mapped["evidence_id"], "ev-abc");
        assert_eq!(mapped["conformance_disposition"], "failed");
        let json_str = serde_json::to_string(&mapped).unwrap();
        assert!(!json_str.contains("\"executed_action\""));
    }

    #[test]
    fn j24l2_invalidated_non_advancing_fails_closed() {
        let step = ExecStepResult {
            before: plan_with(InstallationPlanAction::RunSupervisedConformance),
            after: plan_with(InstallationPlanAction::RunSupervisedConformance),
            outcome: InstallationStepOutcome::ConformanceRecordedWithoutAdvance {
                evidence_id: "ev-inv".to_string(),
                disposition: ConformanceDisposition::Invalidated,
            },
        };

        let result = InstallationDriveResult {
            steps: vec![step],
            stop: InstallationDriveStop::ConformanceRecordedWithoutAdvance,
        };

        let output = map_drive_result(result, "test-candidate");
        assert_eq!(output.exit_code, 6);
        assert_eq!(output.envelope.status, OutcomeStatus::Failed);
        assert_eq!(
            output.envelope.error.as_ref().unwrap().code,
            "installation_execution_postcondition_failed"
        );
        assert_eq!(
            output.envelope.error.as_ref().unwrap().message,
            "non-advancing conformance result was contradictory"
        );
    }

    #[test]
    fn j24l2_error_code_and_message_survive_mapping() {
        let error = M3Error::new("store_io", "disk full");
        let status = error_code_to_status(error.code);
        assert_eq!(status, OutcomeStatus::Unavailable);
        assert_eq!(error.code, "store_io");
        assert_eq!(error.message, "disk full");
    }

    fn temp_dir(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("tethers-j24l2-{name}-{}", uuid::Uuid::new_v4()))
    }

    #[test]
    fn j24l2_relative_host_data_root_returns_error_creates_nothing() {
        let root = temp_dir("relative-host");
        std::fs::create_dir_all(&root).unwrap();

        let result = run_install(
            &PathBuf::from("relative-host"),
            &PathBuf::from("C:\\req.json"),
        );
        assert_eq!(result.exit_code, 2);
        assert_eq!(result.envelope.status, OutcomeStatus::InvalidCliUsage);
        assert_eq!(
            result.envelope.error.as_ref().unwrap().code,
            "invalid_cli_usage"
        );
        assert_eq!(
            result.envelope.error.as_ref().unwrap().message,
            "--host-data-root must be absolute"
        );
        assert_eq!(
            result.envelope.error.as_ref().unwrap().field.as_deref(),
            Some("/host-data-root")
        );

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn j24l2_relative_request_path_returns_error_creates_nothing() {
        let root = temp_dir("relative-request");
        std::fs::create_dir_all(&root).unwrap();

        let result = run_install(
            &PathBuf::from("C:\\host"),
            &PathBuf::from("relative-req.json"),
        );
        assert_eq!(result.exit_code, 2);
        assert_eq!(result.envelope.status, OutcomeStatus::InvalidCliUsage);
        assert_eq!(
            result.envelope.error.as_ref().unwrap().code,
            "invalid_cli_usage"
        );
        assert_eq!(
            result.envelope.error.as_ref().unwrap().message,
            "--request must be absolute"
        );
        assert_eq!(
            result.envelope.error.as_ref().unwrap().field.as_deref(),
            Some("/request")
        );

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn j24l2_missing_host_data_root_returns_unavailable() {
        let root = temp_dir("missing-host");
        let missing = root.join("nonexistent");

        let result = run_install(&missing, &PathBuf::from("C:\\req.json"));
        assert_eq!(result.exit_code, 4);
        assert_eq!(result.envelope.status, OutcomeStatus::Unavailable);
        assert_eq!(
            result.envelope.error.as_ref().unwrap().code,
            "plug_data_root_unavailable"
        );
        assert_eq!(
            result.envelope.error.as_ref().unwrap().message,
            "host data root is unavailable"
        );

        assert!(
            !missing.exists(),
            "missing host-data root must not be created"
        );
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn j24l2_malformed_request_creates_no_lifecycle_state() {
        let root = temp_dir("malformed-req");
        std::fs::create_dir_all(&root).unwrap();

        let request_path = root.join("bad.json");
        std::fs::write(&request_path, b"not json").unwrap();

        let result = run_install(&root, &request_path);
        assert!(result.exit_code != 0);

        let children: Vec<String> = std::fs::read_dir(&root)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .filter(|n| n != "bad.json")
            .collect();
        for name in &children {
            eprintln!("unexpected child after bad request: {name}");
        }
        assert!(
            children.is_empty(),
            "malformed request must not create state"
        );

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn j24l2_missing_candidate_roots_creates_no_later_lifecycle_roots() {
        let root = temp_dir("missing-stage");
        std::fs::create_dir_all(&root).unwrap();

        let request_path = root.join("req.json");
        let req = serde_json::json!({
            "schema": "tethers.plug-install/1",
            "candidate_id": "3d846d40-01fc-4e1e-b77d-83944dbed76f",
            "trust": {"scope": "exact_candidate"},
            "conformance": {"allow_non_isolated_supervised_execution": true},
            "installation": {"target_state": "disabled"}
        });
        std::fs::write(&request_path, serde_json::to_vec(&req).unwrap()).unwrap();

        let result = run_install(&root, &request_path);
        assert!(result.exit_code != 0);

        let lifecycle_children = [
            "installation-trust",
            "launch-profiles",
            "conformance",
            "installation-approvals",
            "install",
            "installed-records",
            "enablements",
            "installation-intent",
            "conformance-scratch",
            "installation.lock",
        ];
        for child in &lifecycle_children {
            let path = root.join(child);
            assert!(
                !path.exists(),
                "{child} must not exist after missing stage roots"
            );
        }

        std::fs::remove_dir_all(&root).ok();
    }
}
