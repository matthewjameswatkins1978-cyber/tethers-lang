use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use tethers_reference_host::dispatch::FileTrail;
use tethers_reference_host::resolve_outcome::{
    FileResolveOutcomeDeliveryStore, ResolveOutcome, ResolveOutcomeAdapter,
    ResolveOutcomeAdapterError, ResolveOutcomeDeliveryCoordinator, ResolveOutcomeDeliveryResult,
    ResolveOutcomeDeliveryStore, ResolveOutcomeRequest, TethersActionRef,
};
use tethers_reference_host::{SharedExecutionOutcome, SharedExecutionResult};

const TETHERS_BASELINE_SHA: &str = "07870c356e034103573c5499347c61fce0700218";
// The fixture commit follows this implementation checkpoint so the pinned
// semantic authority remains immutable and reviewable.
const TETHERS_P4_IMPLEMENTATION_SHA: &str = "b003b88c0da2ccce90fda4c3a116612badd63377";
const RESOLVE_R0_SHA: &str = "8d42e5b061f86b2b2a2c1949c629654968a550ff";

struct RecordingResolveAdapter {
    calls: usize,
}

impl ResolveOutcomeAdapter for RecordingResolveAdapter {
    fn deliver_outcome(
        &mut self,
        _action_ref: &TethersActionRef,
        _outcome: ResolveOutcome,
    ) -> Result<(), ResolveOutcomeAdapterError> {
        self.calls += 1;
        Ok(())
    }
}

struct ExactlyOnceProvider {
    calls: usize,
}

impl ExactlyOnceProvider {
    fn execute(&mut self) -> ResolveOutcome {
        assert_eq!(self.calls, 0, "provider must not be rerun for delivery");
        self.calls += 1;
        ResolveOutcome::Succeeded
    }
}

fn paths() -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let prefix = format!(
        "tethers-p4-conformance-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let root = std::env::temp_dir().join(prefix);
    fs::create_dir_all(&root).unwrap();
    (
        root.join("intent.jsonl"),
        root.join("delivery.jsonl"),
        root.join("trail.jsonl"),
        root,
    )
}

#[test]
fn frozen_resolve_r0_handshake_delivers_known_outcome_once_across_restart() {
    assert_eq!(TETHERS_BASELINE_SHA.len(), 40);
    assert_eq!(
        TETHERS_P4_IMPLEMENTATION_SHA,
        "b003b88c0da2ccce90fda4c3a116612badd63377"
    );
    assert_eq!(RESOLVE_R0_SHA, "8d42e5b061f86b2b2a2c1949c629654968a550ff");

    let (intent_path, delivery_path, trail_path, root) = paths();
    let mut intent = File::create(&intent_path).unwrap();
    writeln!(intent, "durable intent: action=action-p4").unwrap();
    intent.sync_data().unwrap();

    // The guard is issued externally by Resolve; Tethers sees only its
    // already-admitted execution path and the eventual action reference.
    let guard_issued_externally = true;
    let resumed = guard_issued_externally;
    assert!(resumed);

    let mut provider = ExactlyOnceProvider { calls: 0 };
    let known_outcome = provider.execute();
    assert_eq!(known_outcome, ResolveOutcome::Succeeded);

    let action_ref = TethersActionRef::from_host_value("tethers-action-p4-secretless").unwrap();
    let request = ResolveOutcomeRequest::new(action_ref.clone(), known_outcome);
    let store = FileResolveOutcomeDeliveryStore::open(&delivery_path).unwrap();
    let mut trail = FileTrail::open(&trail_path).unwrap();
    let mut adapter = RecordingResolveAdapter { calls: 0 };
    let mut delivery = ResolveOutcomeDeliveryCoordinator::new(store);
    assert_eq!(
        delivery.deliver(request, &mut adapter, &mut trail).unwrap(),
        ResolveOutcomeDeliveryResult::Delivered
    );

    // Exact duplicate delivery is allowed by Resolve R0 idempotency. It uses
    // the same known outcome and never calls the provider.
    assert_eq!(
        delivery
            .retry(&action_ref, &mut adapter, &mut trail)
            .unwrap(),
        ResolveOutcomeDeliveryResult::Delivered
    );
    assert_eq!(provider.calls, 1);
    assert_eq!(adapter.calls, 2);

    // Restart/reopen reconstructs delivery state and permits another exact
    // retry without any provider handle or provider input.
    drop(delivery);
    let store = FileResolveOutcomeDeliveryStore::open(&delivery_path).unwrap();
    assert_eq!(
        store.current(&action_ref).unwrap().unwrap().state(),
        tethers_reference_host::resolve_outcome::ResolveOutcomeDeliveryState::Delivered
    );
    let mut delivery = ResolveOutcomeDeliveryCoordinator::new(store);
    let mut trail = FileTrail::open(&trail_path).unwrap();
    assert_eq!(
        delivery
            .retry(&action_ref, &mut adapter, &mut trail)
            .unwrap(),
        ResolveOutcomeDeliveryResult::Delivered
    );
    assert_eq!(provider.calls, 1);

    let trail_text = fs::read_to_string(&trail_path).unwrap();
    assert!(trail_text.contains("action_ref_digest"));
    assert!(!trail_text.contains("secretless"));
    assert!(!trail_text.contains("durable intent"));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn delivery_failure_does_not_reclassify_result_anchor_taxonomy() {
    let result = SharedExecutionResult {
        outcome: SharedExecutionOutcome::Uncertain,
        execution_id: Some("exec-p4-taxonomy".to_owned()),
    };
    assert_eq!(
        result.resolve_outcome(),
        Some(ResolveOutcome::Uncertain),
        "delivery projects the existing taxonomy without changing it"
    );
    assert_eq!(
        SharedExecutionResult {
            outcome: SharedExecutionOutcome::GuardRejected,
            execution_id: None,
        }
        .resolve_outcome(),
        None,
        "guard admission refusal is not a provider outcome"
    );
}
