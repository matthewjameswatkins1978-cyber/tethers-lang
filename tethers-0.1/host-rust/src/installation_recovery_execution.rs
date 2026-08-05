use crate::installation_recovery::InstallationRecoveryDisposition;
use crate::installation_recovery_plan::{
    plan_installation_recovery, InstallationRecoveryPlanningContext,
    ValidatedInstallationRecoveryPlan,
};
use crate::installation_request::InstallationRequest;
use crate::m3_store::{M3Error, Result};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum InstallationRecoveryExecutionOutcome {
    Idle,
    Recovered {
        disposition: InstallationRecoveryDisposition,
    },
}

pub(crate) fn execute_validated_installation_recovery(
    request: &InstallationRequest,
    context: &InstallationRecoveryPlanningContext<'_>,
    plan: ValidatedInstallationRecoveryPlan,
) -> Result<InstallationRecoveryExecutionOutcome> {
    let fresh = plan_installation_recovery(request, context)?;
    require_exact_plan(&plan, &fresh)?;

    let Some(intent) = plan.intent() else {
        if plan.disposition().is_some() {
            return Err(recovery_conflict());
        }
        return Ok(InstallationRecoveryExecutionOutcome::Idle);
    };
    let Some(disposition) = plan.disposition() else {
        return Err(recovery_conflict());
    };

    match disposition {
        InstallationRecoveryDisposition::RemoveIntentOnly => {
            remove_exact_intent(context, intent)?;
        }
        InstallationRecoveryDisposition::RemoveStagingThenIntent => {
            context
                .installed
                .remove_installation_recovery_staging(intent)?;
            let replanned = plan_installation_recovery(request, context)?;
            require_pending_plan(
                &replanned,
                intent,
                InstallationRecoveryDisposition::RemoveIntentOnly,
            )?;
            remove_exact_intent(context, intent)?;
        }
        InstallationRecoveryDisposition::RevalidateDestinationThenPublishRecord => {
            context
                .installed
                .publish_installation_recovery_record(intent)?;
            let replanned = plan_installation_recovery(request, context)?;
            require_pending_plan(
                &replanned,
                intent,
                InstallationRecoveryDisposition::VerifyCompletedPublicationThenRemoveIntent,
            )?;
            remove_exact_intent(context, intent)?;
        }
        InstallationRecoveryDisposition::VerifyCompletedPublicationThenRemoveIntent => {
            remove_exact_intent(context, intent)?;
        }
    }

    let final_plan = plan_installation_recovery(request, context)?;
    if !final_plan.is_idle() || final_plan.disposition().is_some() {
        return Err(recovery_conflict());
    }

    Ok(InstallationRecoveryExecutionOutcome::Recovered { disposition })
}

fn require_exact_plan(
    supplied: &ValidatedInstallationRecoveryPlan,
    fresh: &ValidatedInstallationRecoveryPlan,
) -> Result<()> {
    if supplied == fresh {
        Ok(())
    } else {
        Err(recovery_conflict())
    }
}

fn require_pending_plan(
    plan: &ValidatedInstallationRecoveryPlan,
    intent: &crate::installation_publication_intent::InstallationPublicationIntent,
    disposition: InstallationRecoveryDisposition,
) -> Result<()> {
    if plan.intent() == Some(intent) && plan.disposition() == Some(disposition) {
        Ok(())
    } else {
        Err(recovery_conflict())
    }
}

fn remove_exact_intent(
    context: &InstallationRecoveryPlanningContext<'_>,
    intent: &crate::installation_publication_intent::InstallationPublicationIntent,
) -> Result<()> {
    if context.intents.remove_if_matches(intent)? {
        Ok(())
    } else {
        Err(recovery_conflict())
    }
}

fn recovery_conflict() -> M3Error {
    M3Error::new(
        "installation_recovery_conflict",
        "installation recovery state conflicts with publication intent",
    )
}
