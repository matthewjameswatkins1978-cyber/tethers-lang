use crate::installation_publication_intent::{
    InstallationPublicationIntent, InstallationPublicationIntentStore,
};
use crate::installation_recovery::{
    classify_installation_recovery, InstallationRecoveryDisposition,
};
use crate::installation_recovery_evidence::{
    revalidate_installation_recovery_evidence, InstallationRecoveryEvidenceContext,
};
use crate::installation_request::InstallationRequest;
use crate::installed::InstalledPlugRegistry;
use crate::m3_store::Result;

pub(crate) struct InstallationRecoveryPlanningContext<'a> {
    pub intents: &'a InstallationPublicationIntentStore,
    pub installed: &'a InstalledPlugRegistry,
    pub evidence: InstallationRecoveryEvidenceContext<'a>,
}

#[derive(PartialEq, Eq)]
pub(crate) struct ValidatedInstallationRecoveryPlan {
    intent: Option<InstallationPublicationIntent>,
    disposition: Option<InstallationRecoveryDisposition>,
}

impl std::fmt::Debug for ValidatedInstallationRecoveryPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValidatedInstallationRecoveryPlan")
            .field("is_idle", &self.is_idle())
            .field("disposition", &self.disposition)
            .finish()
    }
}

impl ValidatedInstallationRecoveryPlan {
    pub(crate) fn intent(&self) -> Option<&InstallationPublicationIntent> {
        self.intent.as_ref()
    }

    pub(crate) fn disposition(&self) -> Option<InstallationRecoveryDisposition> {
        self.disposition
    }

    pub(crate) fn is_idle(&self) -> bool {
        self.intent.is_none()
    }
}

pub(crate) fn plan_installation_recovery(
    request: &InstallationRequest,
    context: &InstallationRecoveryPlanningContext<'_>,
) -> Result<ValidatedInstallationRecoveryPlan> {
    let intent = context.intents.load()?;
    context
        .installed
        .audit_installation_recovery_destinations(intent.as_ref())?;

    let Some(intent) = intent else {
        return Ok(ValidatedInstallationRecoveryPlan {
            intent: None,
            disposition: None,
        });
    };

    let snapshot = context.installed.observe_installation_recovery(&intent)?;
    let disposition = classify_installation_recovery(snapshot.as_observation(&intent))?;

    match disposition {
        InstallationRecoveryDisposition::RemoveIntentOnly
        | InstallationRecoveryDisposition::RemoveStagingThenIntent => {}
        InstallationRecoveryDisposition::RevalidateDestinationThenPublishRecord
        | InstallationRecoveryDisposition::VerifyCompletedPublicationThenRemoveIntent => {
            revalidate_installation_recovery_evidence(request, &intent, &context.evidence)?;
            context
                .installed
                .verify_installation_recovery_destination(&intent)?;
        }
    }

    Ok(ValidatedInstallationRecoveryPlan {
        intent: Some(intent),
        disposition: Some(disposition),
    })
}
