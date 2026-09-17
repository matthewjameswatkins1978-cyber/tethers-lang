// P4b composition evidence for the existing guarded execution authorities.
//
// These tests deliberately live under the application test module so they can
// drive the real guarded serial boundary.  Faults are deterministic test
// inputs at existing replay, Trail, adapter, provider, and delivery seams;
// there is no production crash hook or second recovery state machine.

use super::*;
use crate::dispatch::{self, DispatchReadyAction, RecordingTrail};
use crate::executor::CapabilityExecutor;
use crate::outcome::{self, ProviderDiagnostic};
use crate::replay::ReplayState;
use crate::replay_runtime::test_support::{FailPoint, TestReplayAuthority};
use crate::resolve_guard::{ResolveGuardAdapterError, ResolveGuardAdmission};
use crate::resolve_outcome::{
    FileResolveOutcomeDeliveryStore, ResolveOutcome, ResolveOutcomeAdapter,
    ResolveOutcomeAdapterError, ResolveOutcomeDeliveryCoordinator, ResolveOutcomeDeliveryResult,
    ResolveOutcomeRequest, TethersActionRef,
};
use serde_json::{json, Value};
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

#[derive(Clone, Copy)]
enum ProviderBehaviour {
    Succeeded,
    Failed,
    Uncertain,
}

struct CountingExecutor {
    calls: Rc<Cell<usize>>,
    behaviour: ProviderBehaviour,
}

impl CapabilityExecutor for CountingExecutor {
    fn provider_identity(&self) -> &str {
        "lantern-local"
    }

    fn execute(&mut self, ready: &DispatchReadyAction) -> Result<Value, String> {
        self.execute_classified(ready, Duration::from_secs(1))
            .map_err(|diagnostic| format!("{diagnostic:?}"))
    }

    fn execute_classified(
        &mut self,
        _ready: &DispatchReadyAction,
        _remaining: Duration,
    ) -> Result<Value, ProviderDiagnostic> {
        self.calls.set(self.calls.get() + 1);
        match self.behaviour {
            ProviderBehaviour::Succeeded => Ok(json!({"status": "recorded"})),
            ProviderBehaviour::Failed => Err(ProviderDiagnostic::ExplicitProviderError),
            ProviderBehaviour::Uncertain => Err(ProviderDiagnostic::ResponseMalformed),
        }
    }
}

struct GuardedRun {
    result: SharedExecutionResult,
    response: Value,
    trail: RecordingTrail,
    provider_calls: usize,
    admission_calls: usize,
}

fn run_guarded(
    fail_point: Option<FailPoint>,
    admission: Result<ResolveGuardAdmission, ResolveGuardAdapterError>,
    behaviour: ProviderBehaviour,
    intent_trail_failure: bool,
    guard_trail_failure: bool,
    outcome_trail_failure: bool,
) -> GuardedRun {
    let (_store, resolved) = resolved_lantern();
    let mut response = make_matched_response(
        "p4b-evaluation",
        "action-1",
        resolved.capability_name(),
        json!({"project": "p", "task": "t"}),
    );
    let mut replay = TestReplayAuthority::default();
    replay.fail_at = fail_point;
    let mut trail = RecordingTrail::new();
    if intent_trail_failure {
        trail.injected_intent_error = Some(dispatch::TrailError::WriteFailed(
            "intent evidence fault".into(),
        ));
    }
    if guard_trail_failure {
        trail.injected_guard_admission_error = Some(dispatch::TrailError::WriteFailed(
            "guard evidence fault".into(),
        ));
    }
    if outcome_trail_failure {
        trail.injected_outcome_error = Some(dispatch::TrailError::WriteFailed(
            "outcome evidence fault".into(),
        ));
    }
    let mut adapter = P2TestAdapter {
        result: admission,
        calls: 0,
        action_refs: Vec::new(),
    };
    let mut guard = crate::resolve_guard::test_guard_admission_context(&mut adapter);
    let provider_calls = Rc::new(Cell::new(0));
    let mut executor = CountingExecutor {
        calls: Rc::clone(&provider_calls),
        behaviour,
    };
    let action = extract_single_action(&response)
        .expect("P4b fixture has one Action")
        .clone();
    let context = InputEventContext::for_initial("p4b-event");
    let clock = outcome::ProductionMonotonicClock::new();
    let mut anchors = ResponseResultAnchorWriter;

    let result = execute_shared_boundary_with_guard(
        &mut response,
        &action,
        allow_decision_for(&resolved),
        &resolved,
        &mut trail,
        &mut executor,
        &context,
        false,
        &clock,
        &replay,
        None,
        &mut anchors,
        None,
        &mut guard,
    )
    .expect("guarded boundary should return a classified result");
    drop(guard);

    GuardedRun {
        result,
        response,
        trail,
        provider_calls: provider_calls.get(),
        admission_calls: adapter.calls,
    }
}

#[test]
fn p4b_serial_happy_path_preserves_authority_order_and_effect_count() {
    let run = run_guarded(
        None,
        Ok(ResolveGuardAdmission::Admitted),
        ProviderBehaviour::Succeeded,
        false,
        false,
        false,
    );

    assert_eq!(run.result.outcome, SharedExecutionOutcome::Completed);
    assert_eq!(run.provider_calls, 1);
    assert_eq!(run.admission_calls, 1);
    assert_eq!(
        run.trail.entries.len(),
        1,
        "durable intent precedes admission"
    );
    assert_eq!(run.trail.guard_admission_entries.len(), 1);
    assert_eq!(run.trail.outcome_entries.len(), 1);
    assert_eq!(run.trail.outcome_entries[0].status, "succeeded");
    assert_eq!(
        run.response["result_anchor"]["event_name"],
        "capability.succeeded"
    );
}

#[test]
fn p4b_replay_and_trail_faults_fail_closed_with_at_most_one_effect() {
    let replay_cases = [
        (FailPoint::Admit, 0, 0),
        (FailPoint::Intent, 0, 0),
        (FailPoint::Armed, 0, 1),
        (FailPoint::Terminal, 1, 1),
    ];
    for (point, expected_provider_calls, expected_admission_calls) in replay_cases {
        let run = run_guarded(
            Some(point),
            Ok(ResolveGuardAdmission::Admitted),
            ProviderBehaviour::Succeeded,
            false,
            false,
            false,
        );
        assert!(run.provider_calls <= 1, "{point:?} provider effect bound");
        assert_eq!(run.provider_calls, expected_provider_calls, "{point:?}");
        assert_eq!(run.admission_calls, expected_admission_calls, "{point:?}");
    }

    let intent = run_guarded(
        None,
        Ok(ResolveGuardAdmission::Admitted),
        ProviderBehaviour::Succeeded,
        true,
        false,
        false,
    );
    assert_eq!(intent.provider_calls, 0);
    assert_eq!(intent.admission_calls, 0);
    assert_eq!(intent.result.outcome, SharedExecutionOutcome::Denied);

    let guard_evidence = run_guarded(
        None,
        Ok(ResolveGuardAdmission::Admitted),
        ProviderBehaviour::Succeeded,
        false,
        true,
        false,
    );
    assert_eq!(guard_evidence.provider_calls, 0);
    assert_eq!(guard_evidence.admission_calls, 1);
    assert_eq!(
        guard_evidence.result.outcome,
        SharedExecutionOutcome::AuditFailed
    );

    let outcome_evidence = run_guarded(
        None,
        Ok(ResolveGuardAdmission::Admitted),
        ProviderBehaviour::Succeeded,
        false,
        false,
        true,
    );
    assert_eq!(outcome_evidence.provider_calls, 1);
    assert_eq!(
        outcome_evidence.result.outcome,
        SharedExecutionOutcome::AuditFailed
    );

    for decision in [
        ResolveGuardAdmission::Rejected,
        ResolveGuardAdmission::Indeterminate,
    ] {
        let run = run_guarded(
            None,
            Ok(decision),
            ProviderBehaviour::Succeeded,
            false,
            false,
            false,
        );
        assert_eq!(run.admission_calls, 1);
        assert_eq!(
            run.provider_calls, 0,
            "{decision:?} must stop before provider"
        );
    }

    let unavailable = run_guarded(
        None,
        Err(ResolveGuardAdapterError),
        ProviderBehaviour::Succeeded,
        false,
        false,
        false,
    );
    assert_eq!(unavailable.admission_calls, 1);
    assert_eq!(unavailable.provider_calls, 0);
    assert_eq!(
        unavailable.result.outcome,
        SharedExecutionOutcome::GuardIndeterminate
    );
}

#[test]
fn p4b_old_replay_state_cannot_reuse_admission_after_restart() {
    let (_store, resolved) = resolved_lantern();
    let mut response = make_matched_response(
        "p4b-restart-evaluation",
        "action-1",
        resolved.capability_name(),
        json!({"project": "p", "task": "t"}),
    );
    let mut replay = TestReplayAuthority::default();
    replay.fresh = false;
    replay.recovered_state = ReplayState::IntentRecorded;
    let mut adapter = P2TestAdapter {
        result: Ok(ResolveGuardAdmission::Admitted),
        calls: 0,
        action_refs: Vec::new(),
    };
    let mut guard = crate::resolve_guard::test_guard_admission_context(&mut adapter);
    let mut executor = MockExecutor::new();
    let mut trail = RecordingTrail::new();
    let action = extract_single_action(&response)
        .expect("P4b restart fixture has one Action")
        .clone();
    let context = InputEventContext::for_initial("p4b-restart-event");
    let clock = outcome::ProductionMonotonicClock::new();
    let mut anchors = ResponseResultAnchorWriter;
    let result = execute_shared_boundary_with_guard(
        &mut response,
        &action,
        allow_decision_for(&resolved),
        &resolved,
        &mut trail,
        &mut executor,
        &context,
        false,
        &clock,
        &replay,
        None,
        &mut anchors,
        None,
        &mut guard,
    )
    .unwrap();
    drop(guard);

    assert_eq!(
        result.outcome,
        SharedExecutionOutcome::Replay(
            crate::replay_runtime::ReplayDispatchResult::RequiresManualResolution
        )
    );
    assert_eq!(adapter.calls, 0, "old admission cannot be reused");
    assert!(executor.completed.is_empty());
    assert!(trail.entries.is_empty());
}

#[test]
fn p4b_failed_and_uncertain_provider_truth_remains_durable_and_distinct() {
    for (behaviour, expected_outcome, expected_status) in [
        (
            ProviderBehaviour::Failed,
            SharedExecutionOutcome::Failed,
            "failed",
        ),
        (
            ProviderBehaviour::Uncertain,
            SharedExecutionOutcome::Uncertain,
            "uncertain",
        ),
    ] {
        let run = run_guarded(
            None,
            Ok(ResolveGuardAdmission::Admitted),
            behaviour,
            false,
            false,
            false,
        );
        assert_eq!(run.result.outcome, expected_outcome);
        assert_eq!(run.provider_calls, 1);
        assert_eq!(run.trail.outcome_entries[0].status, expected_status);
        assert_eq!(run.admission_calls, 1);
    }
}

struct RedeliveryAdapter {
    calls: Rc<Cell<usize>>,
}

impl ResolveOutcomeAdapter for RedeliveryAdapter {
    fn deliver_outcome(
        &mut self,
        _request: &ResolveOutcomeRequest,
    ) -> Result<crate::resolve_outcome::ResolveOutcomeDeliveryAck, ResolveOutcomeAdapterError> {
        let call = self.calls.get();
        self.calls.set(call + 1);
        if call == 0 {
            Err(ResolveOutcomeAdapterError)
        } else {
            Ok(crate::resolve_outcome::ResolveOutcomeDeliveryAck::Recorded)
        }
    }
}

#[test]
fn p4b_outcome_delivery_recovers_exactly_across_restart_without_provider_access() {
    let action_ref = TethersActionRef::from_host_value("p4b-execution-1").unwrap();
    let preparation_digest = crate::resolve_guard::GuardPreparationProofDigest::from_host_value(
        &format!("sha256:{}", "a".repeat(64)),
    )
    .unwrap();
    let request = ResolveOutcomeRequest::new(
        action_ref.clone(),
        preparation_digest.clone(),
        ResolveOutcome::Succeeded,
    );
    let path = std::env::temp_dir().join(format!(
        "tethers-p4b-delivery-{}-{}.jsonl",
        std::process::id(),
        crate::approval::digest(&json!("restart"))
    ));
    let _ = std::fs::remove_file(&path);
    let calls = Rc::new(Cell::new(0));
    let mut first = ResolveOutcomeDeliveryCoordinator::new(
        FileResolveOutcomeDeliveryStore::open(&path).unwrap(),
    );
    let mut first_trail = RecordingTrail::new();
    assert_eq!(
        first
            .deliver(
                request,
                &mut RedeliveryAdapter {
                    calls: Rc::clone(&calls),
                },
                &mut first_trail,
            )
            .unwrap(),
        ResolveOutcomeDeliveryResult::Indeterminate
    );
    drop(first);

    let mut second = ResolveOutcomeDeliveryCoordinator::new(
        FileResolveOutcomeDeliveryStore::open(&path).unwrap(),
    );
    let mut second_trail = RecordingTrail::new();
    assert_eq!(
        second
            .retry(
                &action_ref,
                &preparation_digest,
                &mut RedeliveryAdapter {
                    calls: Rc::clone(&calls),
                },
                &mut second_trail,
            )
            .unwrap(),
        ResolveOutcomeDeliveryResult::Delivered
    );
    assert_eq!(calls.get(), 2);

    let mut third = ResolveOutcomeDeliveryCoordinator::new(
        FileResolveOutcomeDeliveryStore::open(&path).unwrap(),
    );
    let mut third_trail = RecordingTrail::new();
    assert_eq!(
        third
            .retry(
                &action_ref,
                &preparation_digest,
                &mut RedeliveryAdapter {
                    calls: Rc::clone(&calls),
                },
                &mut third_trail,
            )
            .unwrap(),
        ResolveOutcomeDeliveryResult::Delivered
    );
    assert_eq!(calls.get(), 2, "Delivered is terminal after restart");
    std::fs::remove_file(path).unwrap();
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum P4bCrashPoint {
    BeforeDurableIntent,
    AfterDurableIntentBeforeResolveRequest,
    AfterResolveRequestBeforeResponse,
    AfterAdmittedBeforeAdmissionEvidence,
    AfterAdmissionEvidenceBeforeG1,
    AfterG1BeforeProvider,
    DuringProvider,
    AfterProviderBeforeDurableOutcome,
    AfterDurableOutcomeBeforeDeliveryRequest,
    AfterDeliveryRequestBeforeResponse,
    AfterRecordedBeforeDeliveredPersistence,
    AfterDeliveredPersistence,
}

const P4B_CRASH_POINTS: [P4bCrashPoint; 12] = [
    P4bCrashPoint::BeforeDurableIntent,
    P4bCrashPoint::AfterDurableIntentBeforeResolveRequest,
    P4bCrashPoint::AfterResolveRequestBeforeResponse,
    P4bCrashPoint::AfterAdmittedBeforeAdmissionEvidence,
    P4bCrashPoint::AfterAdmissionEvidenceBeforeG1,
    P4bCrashPoint::AfterG1BeforeProvider,
    P4bCrashPoint::DuringProvider,
    P4bCrashPoint::AfterProviderBeforeDurableOutcome,
    P4bCrashPoint::AfterDurableOutcomeBeforeDeliveryRequest,
    P4bCrashPoint::AfterDeliveryRequestBeforeResponse,
    P4bCrashPoint::AfterRecordedBeforeDeliveredPersistence,
    P4bCrashPoint::AfterDeliveredPersistence,
];

#[test]
fn p4b_crash_matrix_has_one_explicit_entry_for_each_consequential_boundary() {
    assert_eq!(P4B_CRASH_POINTS.len(), 12);
    for window in P4B_CRASH_POINTS.windows(2) {
        assert_ne!(window[0], window[1]);
    }
}
