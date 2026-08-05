use crate::installation_publication_intent::InstallationPublicationIntent;
use crate::installed::InstalledPlugRecord;
use crate::m3_store::{M3Error, Result};

fn intent_invalid() -> M3Error {
    M3Error::new(
        "installation_intent_invalid",
        "installation publication intent is invalid",
    )
}

fn recovery_conflict() -> M3Error {
    M3Error::new(
        "installation_recovery_conflict",
        "installation recovery state conflicts with publication intent",
    )
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct InstallationRecoveryObservation<'a> {
    pub intent: &'a InstallationPublicationIntent,
    pub staging_present: bool,
    pub destination_present: bool,
    pub installed_record: Option<&'a InstalledPlugRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InstallationRecoveryDisposition {
    RemoveIntentOnly,
    RemoveStagingThenIntent,
    RevalidateDestinationThenPublishRecord,
    VerifyCompletedPublicationThenRemoveIntent,
}

pub(crate) fn classify_installation_recovery(
    observation: InstallationRecoveryObservation<'_>,
) -> Result<InstallationRecoveryDisposition> {
    observation
        .intent
        .validate()
        .map_err(|_| intent_invalid())?;

    if let Some(record) = observation.installed_record {
        record.validate().map_err(|_| recovery_conflict())?;
    }

    if observation.staging_present && observation.destination_present {
        return Err(recovery_conflict());
    }

    match (
        observation.staging_present,
        observation.destination_present,
        observation.installed_record,
    ) {
        (false, false, None) => Ok(InstallationRecoveryDisposition::RemoveIntentOnly),
        (true, false, None) => Ok(InstallationRecoveryDisposition::RemoveStagingThenIntent),
        (false, true, None) => {
            Ok(InstallationRecoveryDisposition::RevalidateDestinationThenPublishRecord)
        }
        (false, true, Some(record)) => {
            if record == &observation.intent.installed_record {
                Ok(InstallationRecoveryDisposition::VerifyCompletedPublicationThenRemoveIntent)
            } else {
                Err(recovery_conflict())
            }
        }
        _ => Err(recovery_conflict()),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InstallationRecoverySnapshot {
    pub staging_present: bool,
    pub destination_present: bool,
    pub installed_record: Option<InstalledPlugRecord>,
}

impl InstallationRecoverySnapshot {
    pub(crate) fn as_observation<'a>(
        &'a self,
        intent: &'a InstallationPublicationIntent,
    ) -> InstallationRecoveryObservation<'a> {
        InstallationRecoveryObservation {
            intent,
            staging_present: self.staging_present,
            destination_present: self.destination_present,
            installed_record: self.installed_record.as_ref(),
        }
    }
}
