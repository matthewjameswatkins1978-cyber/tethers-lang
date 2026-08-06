//! J24K3e2: exact crash-safe disabled-installation publication mutation.
//!
//! Consumes the sealed J24K3e1 prepared publication, freshly revalidates it
//! against current authoritative stores, and performs the exact durable
//! transaction in one owned sequence:
//!
//! ```text
//! write durable intent
//!   -> build and verify staging directory
//!   -> rename staging to final destination
//!   -> publish exact precomputed installed record
//!   -> remove intent
//! ```
//!
//! The module acquires no installation lock, changes no public execution
//! context, wires no public executor, and executes no ordinary J24J action. It
//! performs no recovery planning or execution of its own beyond the exact
//! completed-publication intent removal that the accepted recovery authority
//! performs.

use crate::candidate::CandidateRecord;
use crate::installation_publication_intent::InstallationPublicationIntent;
use crate::installation_publication_preparation::PreparedInstallationPublication;
use crate::installation_recovery::InstallationRecoveryDisposition;
use crate::installation_recovery_evidence::revalidate_installation_recovery_evidence;
use crate::installation_recovery_execution::execute_validated_installation_recovery;
use crate::installation_recovery_plan::{
    plan_installation_recovery, InstallationRecoveryPlanningContext,
};
use crate::installation_request::InstallationRequest;
use crate::installed::InstalledPlugRecord;
use crate::m3_store::{M3Error, Result};
use crate::package::PackageError;

fn recovery_conflict() -> M3Error {
    M3Error::new(
        "installation_recovery_conflict",
        "installation recovery state conflicts with publication intent",
    )
}

fn recovery_io() -> M3Error {
    M3Error::new(
        "installation_recovery_io",
        "installation recovery state could not be observed",
    )
}

fn evidence_stale() -> M3Error {
    M3Error::new(
        "installation_intent_evidence_stale",
        "installation publication evidence is no longer current",
    )
}

fn installed_conflict() -> M3Error {
    M3Error::new(
        "installed_conflict",
        "installed registry already contains this package release or candidate",
    )
}

fn map_candidate_error(error: PackageError) -> M3Error {
    match error.code {
        "unsafe_destination" => M3Error::new("unsafe_store_path", "candidate location is unsafe"),
        "candidate_io" => recovery_io(),
        _ => evidence_stale(),
    }
}

/// Load the single current candidate matching `candidate_id`.
fn load_candidate(
    context: &InstallationRecoveryPlanningContext<'_>,
    candidate_id: &str,
) -> Result<CandidateRecord> {
    let all = context
        .evidence
        .candidates
        .load_all()
        .map_err(map_candidate_error)?;
    let mut matching: Vec<CandidateRecord> = all
        .into_iter()
        .filter(|candidate| candidate.candidate_id == candidate_id)
        .collect();
    if matching.len() != 1 {
        return Err(evidence_stale());
    }
    Ok(matching.remove(0))
}

/// Execute the exact prepared durable publication transaction.
///
/// The sealed prepared value is consumed (taken by value) so it cannot be
/// reused after a successful mutation. Preparation identity and content are
/// never regenerated; the exact intent, staging, destination, and record are
/// published unchanged.
pub(crate) fn execute_prepared_disabled_installation_publication(
    request: &InstallationRequest,
    context: &InstallationRecoveryPlanningContext<'_>,
    prepared: PreparedInstallationPublication,
) -> Result<InstalledPlugRecord> {
    let intent: &InstallationPublicationIntent = prepared.intent();
    let record: &InstalledPlugRecord = prepared.installed_record();

    // 2. Freshly revalidate before the first durable write.
    revalidate_installation_recovery_evidence(request, intent, &context.evidence)?;

    let plan = plan_installation_recovery(request, context)?;
    if !plan.is_idle() || plan.disposition().is_some() {
        return Err(recovery_conflict());
    }

    context
        .installed
        .audit_installation_recovery_destinations(Some(intent))?;

    // No contradictory installed state: refuse a duplicate package release or
    // a duplicate source candidate before any durable write.
    let existing = context.installed.load_all()?;
    for current in &existing {
        if current.package_id == record.package_id
            && current.package_version == record.package_version
        {
            return Err(installed_conflict());
        }
        if current.source_candidate_id == record.source_candidate_id {
            return Err(installed_conflict());
        }
    }

    // Prepared intent and record must both validate exactly and agree.
    intent.validate()?;
    record.validate()?;

    // 3. Persist the exact precomputed publication intent atomically.
    context.intents.create(intent)?;
    let loaded = context.intents.load()?;
    if loaded.as_ref() != Some(intent) {
        return Err(recovery_conflict());
    }

    #[cfg(test)]
    post_intent_failure_test_hook::fail_after_durable_intent_if_installed()?;

    // 4. Build and verify one exact staging directory.
    let candidate = load_candidate(context, &intent.candidate_id)?;
    context.installed.build_installation_recovery_staging(
        intent,
        &candidate,
        context.evidence.quarantine_root,
    )?;

    // 5. Rename staging to the exact final destination.
    context
        .installed
        .rename_installation_recovery_staging(intent)?;

    // 6. Publish the exact precomputed installed record unchanged.
    context
        .installed
        .publish_installation_recovery_record(intent)?;

    // 7. Remove the intent only after completed publication is proven.
    let removal_plan = plan_installation_recovery(request, context)?;
    match removal_plan.disposition() {
        Some(InstallationRecoveryDisposition::VerifyCompletedPublicationThenRemoveIntent) => {}
        _ => return Err(recovery_conflict()),
    }
    execute_validated_installation_recovery(request, context, removal_plan)?;

    // 8/10. Prove idle recovery and that the exact destination plus record remain.
    if context.intents.load()?.is_some() {
        return Err(recovery_conflict());
    }
    let final_plan = plan_installation_recovery(request, context)?;
    if !final_plan.is_idle() || final_plan.disposition().is_some() {
        return Err(recovery_conflict());
    }

    Ok(record.clone())
}

#[cfg(test)]
mod post_intent_failure_test_hook {
    use super::{M3Error, Result};
    use std::cell::Cell;

    std::thread_local! {
        static FAIL_AFTER_DURABLE_INTENT: Cell<bool> = const { Cell::new(false) };
    }

    /// A crate-test-only one-shot failure installation at the durable-intent
    /// boundary. Thread-local state prevents unrelated concurrently running
    /// tests from observing the installation.
    pub(crate) struct PostIntentFailureTestGuard {
        _private: (),
    }

    pub(crate) fn install_post_intent_failure_once_for_test() -> PostIntentFailureTestGuard {
        FAIL_AFTER_DURABLE_INTENT.with(|armed| {
            assert!(
                !armed.replace(true),
                "post-intent failure hook may be installed once per test thread"
            );
        });
        PostIntentFailureTestGuard { _private: () }
    }

    impl Drop for PostIntentFailureTestGuard {
        fn drop(&mut self) {
            FAIL_AFTER_DURABLE_INTENT.with(|armed| armed.set(false));
        }
    }

    pub(super) fn fail_after_durable_intent_if_installed() -> Result<()> {
        let forced = FAIL_AFTER_DURABLE_INTENT.with(|armed| armed.replace(false));
        if forced {
            return Err(M3Error::new(
                "installation_test_forced_failure",
                "test-only post-intent publication failure",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
pub(crate) use post_intent_failure_test_hook::install_post_intent_failure_once_for_test;
