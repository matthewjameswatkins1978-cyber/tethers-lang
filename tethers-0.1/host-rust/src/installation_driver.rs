use crate::installation_execution::{
    execute_next_installation_action, InstallationExecutionContext, InstallationExecutionOptions,
    InstallationStepOutcome, InstallationStepResult,
};
use crate::installation_plan::InstallationPlanAction;
use crate::installation_request::InstallationRequest;
use crate::m3_store::{M3Error, Result};

const MAX_INSTALLATION_EXECUTOR_CALLS: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum InstallationDriveStop {
    Complete,
    ConformanceRecordedWithoutAdvance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InstallationDriveResult {
    pub steps: Vec<InstallationStepResult>,
    pub stop: InstallationDriveStop,
}

pub(crate) fn drive_installation(
    request: &InstallationRequest,
    context: &InstallationExecutionContext<'_>,
    options: &InstallationExecutionOptions<'_>,
) -> Result<InstallationDriveResult> {
    drive_with(|| execute_next_installation_action(request, context, options))
}

pub(crate) fn drive_with<F>(mut next_step: F) -> Result<InstallationDriveResult>
where
    F: FnMut() -> Result<InstallationStepResult>,
{
    let mut steps = Vec::new();

    for _ in 0..MAX_INSTALLATION_EXECUTOR_CALLS {
        let step = next_step()?;

        match &step.outcome {
            InstallationStepOutcome::AlreadyComplete => {
                steps.push(step);
                return Ok(InstallationDriveResult {
                    steps,
                    stop: InstallationDriveStop::Complete,
                });
            }
            InstallationStepOutcome::Advanced { .. } => {
                if step.after.action == InstallationPlanAction::Complete {
                    steps.push(step);
                    return Ok(InstallationDriveResult {
                        steps,
                        stop: InstallationDriveStop::Complete,
                    });
                }
                steps.push(step);
            }
            InstallationStepOutcome::ConformanceRecordedWithoutAdvance { .. } => {
                steps.push(step);
                return Ok(InstallationDriveResult {
                    steps,
                    stop: InstallationDriveStop::ConformanceRecordedWithoutAdvance,
                });
            }
        }
    }

    Err(M3Error::new(
        "installation_iteration_limit",
        "installation did not complete within four executor calls",
    ))
}
