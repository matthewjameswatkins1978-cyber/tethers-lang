# Worker Note — C3-A3 Failure-Boundary Crucible

Task: `C3-A3 — Failure-Boundary Crucible`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `C3-A3 Failure-Boundary Agent`
Status: `COMPLETE`
Base commit: `d8b094c5d89f78cb5b610f5367f098f6cc0ef277`
Implementation checkpoint: `6df1c55416ee7cb24ee236db1ec427250f9e8eab`
Correction checkpoint: `071470f6c64bf609d2b55e6dd8839a7131697543`

## Remote-review finding

Lucy identified a contamination defect in the Stage C fatal-detection logic when N > 1. The original code scanned the ENTIRE accumulated response Trail for `audit_failure` entries after every member's boundary call:

```rust
if response.get("trail").and_then(Value::as_array).is_some_and(
    |entries| {
        entries.iter().any(|entry| entry["kind"] == "audit_failure")
    },
) {
    result.outcome = SharedExecutionOutcome::AuditFailed;
}
```

When member A's `append_outcome` failed (adding `audit_failure` to the trail), and member B later completed successfully, the scan still found A's old `audit_failure` and incorrectly overwrote B's truthful success with `AuditFailed`. This violated the frozen C3 semantics: already-running workers must be terminalised using their OWN truthful Stage C result.

## Root cause

Historical audit_failure contamination across active members. The scan did not distinguish between entries appended by the current boundary call and entries from prior members.

## Production correction

In `execute_group_concurrent_with_limit`, recorded `response_trail_len_before` before calling `execute_boundary_invoke_only`, then only scanned entries appended from that offset onward:

```rust
let response_trail_len_before = response
    .get("trail")
    .and_then(Value::as_array)
    .map_or(0, Vec::len);

// ... execute_boundary_invoke_only ...

let current_boundary_audit_failed = response
    .get("trail")
    .and_then(Value::as_array)
    .is_some_and(|entries| {
        entries
            .iter()
            .skip(response_trail_len_before)
            .any(|entry| entry["kind"] == "audit_failure")
    });
```

## New N=2 B/A/C regression proof

Added `c3_a3_n2_active_sibling_survives_fatal_halt_truthfully`: N=2, members B (index 0), A (index 1), C (index 2). B and A both launch. A's OutcomeEntry durability fails (injected via `RecordingTrail`). B's outcome succeeds. The test proves:

- B's truthful success is NOT contaminated by A's audit_failure
- Group result identifies `member-a` (not `member-b`)
- C never launches
- No GroupJoinEntry

Uses a 500ms sleep between releasing A and B to ensure deterministic processing order (A's `append_outcome` is called before B's).

## Requested outcome

1. Correct the fatal-halt GroupJoin defect in `tethers-0.1/host-rust/src/host_execution.rs` so that when any member remains nonterminal after Stage B/C, no `GroupJoinEntry` and no `group_joined` response presentation are appended, failing closed through deterministic existing non-success/infrastructure taxonomy.
2. Prove that normal provider failure releases launch capacity and queued siblings continue to launch and join (Proof 1).
3. Prove that worker panic caught via `PanicGuard` terminalises as `Uncertain`, releases launch capacity, and queued siblings continue to launch and join (Proof 2).
4. Prove that Stage C OutcomeEntry durability failure halts queued launches without appending a `GroupJoinEntry` (Proof 3).
5. Prove that replay G2 publication failure halts queued launches without appending a `GroupJoinEntry` (Proof 4).
6. Prove that replay G1 publication failure halts before any provider effect and without appending a `GroupJoinEntry` (Proof 5).
7. Prove all-terminal groups continue to produce standard GroupJoin entries and responses (Proof 6 regression).
8. Perform channel-failure audit on worker/coordinator contract.

## Changes made

- In `tethers-0.1/host-rust/src/host_execution.rs`:
  - Added Stage C audit failure detection on `execute_boundary_invoke_only` response trail, setting `result.outcome = SharedExecutionOutcome::AuditFailed` and `launches_halted = true` on audit failure or `ReplayPersistenceUnavailable`.
  - **CORRECTED**: Isolated audit_failure detection to current boundary call only by recording `response_trail_len_before` and skipping prior entries.
  - Added narrow fail-closed pre-Stage-D guard before GroupJoin publication in `execute_group_concurrent_with_limit`: if `any_nonterminal` is true (one or more semantic members remain in `Prepared`, `Launched`, or `Transitioning`), GroupJoin publication and `group_joined` presentation are skipped, returning the first semantic terminal non-success via `first_non_success_member_step` or failing closed with `AuditFailed`.
  - Extended test-only `ObservingReplayAuthority` and `ObservingAdmission` with `with_fail_points` to support deterministic G1 armed and G2 terminal failure injection for selected action IDs.
  - Added test helper methods on `C3A1GroupHarness`: `set_member_outcome`, `run_group_with_trail_and_authority`, `run_group_with_authority`.
  - Implemented 7 deterministic tests in `host_execution.rs`:
    - `c3_a3_normal_provider_failure_releases_slot_and_joins`
    - `c3_a3_worker_panic_terminalises_uncertain_and_releases_slot`
    - `c3_a3_outcome_durability_failure_halts_queued_effects_without_join`
    - `c3_a3_g2_failure_halts_queued_effects_without_join`
    - `c3_a3_g1_failure_halts_before_any_later_effect_without_join`
    - `c3_a3_all_terminal_preserves_group_join`
    - `c3_a3_n2_active_sibling_survives_fatal_halt_truthfully` (new N=2 regression proof)

## Decisions and assumptions

- **GroupJoin Invariant**: GroupJoin exists only when every semantic group member has reached its legitimate terminal state (`Terminal` or `PreparationTerminal`). Queued `Prepared` members following a fatal trusted-state halt are not terminal, and terminal states are never fabricated for them.
- **Fail-closed Result Aggregation**: When execution halts with nonterminal members, the first legitimate terminal non-success in semantic Runtime Plan order is aggregated. If no terminal non-success exists, `ExecutionServiceResult::AuditFailed` is returned.
- **Channel Failure Audit**: The current worker contract executes within `std::thread::scope` where `worker_invoke_provider` wraps execution in `catch_unwind` and always attempts `tx.send(WorkerResult)`. A missing `WorkerResult` without whole-process death is not constructible through the normal worker path without deliberately hanging a thread or modifying channel transport; adversarial channel failure testing is deferred to C4.

## Evidence

- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_a3 --test-threads=1`: PASS (7/7 tests passed)
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_a2 --test-threads=1`: PASS (4/4 tests passed)
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_a1 --test-threads=1`: PASS (3/3 tests passed)
- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml -- --check`: PASS (clean formatting)
- `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml`: PASS (compiles clean)
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- --test-threads=1`: PASS (1526 passed, 0 failed, 2 ignored)
- `git diff --check`: PASS (clean diff, LF→CRLF warnings only)
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`: PASS (control-v1/COMPLETE)

## Discoveries

- The test-only `RecordingTrail` is `!Send` because it retains an internal `Rc<RefCell<Vec<&'static str>>>` events log. Constructing it within the scoped thread closure avoids crossing thread boundaries while allowing full inspection of `outcome_entries` and `group_join_entries`.
- The `sealed` module in `dispatch.rs` is private, preventing custom `Trail` implementations from outside `dispatch`. This limits test-only trail wrappers.
- The barrier script does not remove `active-member-{tag}` files after provider return. `currently_active_count` compensates by filtering out members with outcomes in trail, but when `append_outcome` fails (injected error), the outcome is never written, making `has_active` unreliable for polling completion.

## Remaining risks

- The N=2 test uses a 500ms sleep for deterministic ordering, which is fragile under extreme system load. A `SelectiveRecordingTrail` (failing only for specific action IDs) would be more robust but requires `sealed` module visibility.

## Smallest next action

- Await Lucy review and acceptance of C3-A3 on published branch `feature/c3-failure-boundary`.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/concurrency/C3_BOUNDED_CONCURRENCY_DESIGN.md`
- `tethers-0.1/host-rust/src/host_execution.rs`
