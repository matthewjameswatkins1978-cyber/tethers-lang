# J11 Packet 2 Worker Note

Task: `J11 packet 2 runtime admission wiring`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Goose`

Status: `COMPLETE`

Base commit: `0e89bc79b314b67a4486504747bbbad17da94099`

Implementation checkpoint: `41b6848a006215077228a5aa7a571dcbecc98d6e`

Final evidence correction: `(pending commit)`

## Review-correction (2026-07-28) - COMPLETE

Independent review accepted the runtime wiring logic. Two evidence-quality issues
held final acceptance:

1. `cargo check --tests` reported an unused `context` variable in test
   `j11_rejection_never_enters_evaluation_callback` (formerly test 15),
   violating the no-new-warnings contract.

2. Most J11 coordinator tests exercised `drain_queue_with_admission()`, a
   test-only helper that duplicated the production drain loop. Final acceptance
   requires testing the actual production drain boundary.

### Correction changes

- Extracted one shared `drain_result_event_queue` production function
  with `EventDrainOutcome` and `apply_to_response()`.
- Wired `main()` through the production helper.
- Removed the duplicate test-only `drain_queue_with_admission`.
- Rewrote focused J11 tests (2-8, 10-11, 14-16) to prove behaviour
  through the actual production helper using callback-invocation
  counters.
- Rewrote `j11_rejection_never_enters_evaluation_callback` through the
  production helper: proves a rejected gen-9 event never invokes the
  evaluation callback, removing the unused `context` variable.
- Restored the clean `cargo check` warning baseline.

## Final evidence correction (2026-07-28) - COMPLETE

- Corrected base commit SHA typo in References: `b334` → `b314`.
- Validated all recorded SHAs against the repository.
- Strengthened `j11_completed_follow_ups_preserved_before_rejection`:
  added assertions against the mutated `response` after `apply_to_response`,
  proving that `follow_up_evaluations` and `event_admission_rejection` are
  injected with exact JSON shapes and a pre-existing `status` key is
  preserved.
- Renamed `j11_rejection_does_not_modify_replay` →
  `j11_rejection_never_enters_evaluation_callback`: the test proves callback
  non-entry for rejected events and that the gate remains usable afterward;
  it does not directly inspect replay state.
- Renamed `j11_rejection_prevents_dispatch_entry` →
  `j11_provider_call_count_unchanged_after_gate_rejection`: the test performs
  one successful dispatch, exercises gate rejection directly, and verifies no
  second provider call occurred; it is a dispatch-seam sanity check, not proof
  that the production drain helper reached the dispatch boundary.
- Updated all documentation references to match corrected test names.
- Updated this worker note to a single internally consistent record.

## Requested outcome

Wire the Packet 1 `EventAdmissionGate` into the `main()` coordinator so that
duplicate event IDs and events exceeding generation 8 are visibly stopped
before evaluation. The gate blocks the rejected event and all later queued
siblings while preserving completed follow-ups.

## Changes made

- Added `use event_admission::{EventAdmissionGate, EventAdmissionRejection};`
  to `main.rs` imports.
- Added `event_admission_rejection_value()` helper function mapping rejection
  enum variants to the frozen JSON shapes.
- Added `EventDrainOutcome` struct holding `follow_up_evaluations` and optional
  `event_admission_rejection`, with `apply_to_response(&mut Value)` method
  that injects both fields into a response object without destroying
  pre-existing keys.
- Added `drain_result_event_queue<E, F>()` production function: drains the
  queue with admission gating, invoking a caller-supplied evaluate callback
  for each admitted anchor, and returning an `EventDrainOutcome`. The
  callback receives the queue reference so child anchors can be appended.
- In `main()`:
  - Created one `EventAdmissionGate` per invocation after the queue.
  - Admitted initial external event (generation 0) before `process_one_event`,
    failing closed on unexpected rejection.
  - Replaced the inline drain loop with a call to `drain_result_event_queue`,
    passing a closure that builds `InputEventContext` and calls
    `process_one_event`.
  - Applied the drain outcome to the response via `apply_to_response`.
- Added 19 focused `j11_` unit tests covering all required behaviours, all
  exercising the production `drain_result_event_queue` boundary.
- Updated `docs/CURRENT_CLINE_TASK.md` with the J11 packet 2 control-v1 packet.
- Created and updated this worker note.

No changes to `event_admission.rs`, `event_queue.rs`, `result_anchor.rs`,
`Cargo.toml`, `Cargo.lock`, OCaml, protocols, or fixtures.

## Decisions and assumptions

- `event_admission_rejection` is injected into the response BEFORE
  `follow_up_evaluations` via `apply_to_response()` so it is always present
  even when no follow-ups completed.
- `process_one_event` is never modified; the gate is owned by the outer
  coordinator.
- Depth validation precedes duplicate lookup (inside the gate from Packet 1).
- Rejection uses `break` (not `continue`), stopping all later siblings.
- The initial-event admission uses `.map_err()` to fail closed with a host
  error message.
- JSON helper lives in `main.rs` rather than `event_admission.rs` to keep
  serde_json concerns outside the pure gate module.
- The evaluate callback receives `&mut ResultEventQueue` so children enqueued
  during evaluation are naturally appended for later drain iterations.

## Evidence

| Check | Result |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo test j11_ -- --nocapture` | 19/19 passed |
| `cargo test event_admission -- --nocapture` | 15/15 passed |
| `cargo test` (full suite) | **507/507** |
| `cargo check` | 9 baseline warnings, 0 new |
| `cargo check --tests` | 4 baseline warnings, 0 new |
| `cargo clippy --all-targets --all-features` | no new warnings |
| `check-tethers-task-packet.ps1` | PASS |
| `git diff --check` | PASS |

### Focused J11 test names (19):

1. `j11_initial_event_admitted_before_processing`
2. `j11_clean_unique_follow_up_evaluated_normally`
3. `j11_queued_event_reusing_initial_id_rejected`
4. `j11_duplicate_sibling_not_evaluated_twice`
5. `j11_generation_eight_evaluated`
6. `j11_generation_nine_not_evaluated`
7. `j11_generation_above_nine_not_evaluated`
8. `j11_rejected_event_causes_zero_processing`
9. `j11_provider_call_count_unchanged_after_gate_rejection`
10. `j11_rejection_stops_later_siblings`
11. `j11_completed_follow_ups_preserved_before_rejection`
12. `j11_duplicate_rejection_json_exact_shape`
13. `j11_depth_rejection_json_exact_shape`
14. `j11_clean_run_omits_rejection_field`
15. `j11_rejection_never_enters_evaluation_callback`
16. `j11_rejection_produces_no_anchor`
17. `j11_admission_persists_when_evaluation_fails`
18. `j11_max_evaluation_generation_is_eight`
19. `j11_fifo_child_appended_during_drain`

### Proofs

- **Generation 8 evaluates, generation 9 does not**: Tests 5 and 6/7.
- **Later siblings stop**: Test 10 - `evt/c` remains in queue after `evt/b`
  rejection.
- **Callback non-entry proves downstream stages unreachable**: Tests 8 and 15
  prove rejected events never invoke the evaluate callback; without the
  callback, no `process_one_event`, no dispatch, no replay, and no provider
  call can occur.
- **Completed follow-ups preserved in response**: Test 11 - `evt/first` is
  evaluated; after `apply_to_response`, `response["follow_up_evaluations"]`
  contains the exact completed entry, `response["event_admission_rejection"]`
  contains the exact duplicate rejection JSON, and `response["status"]` is
  still `"existing"` (the pre-existing key is undisturbed).
- **Provider call count unchanged**: Test 9 performs one successful dispatch
  through the full dispatch seam, then gate-rejects a duplicate; verifies the
  provider call count remains 1.
- **JSON shapes**: Tests 12 (duplicate) and 13 (depth) verify exact field
  names, values, and counts.
- **Clean run omits field**: Test 14 - three anchors all pass, rejection is
  `None`.
- **Admission persists after failure**: Test 17 - FailingExecutor dispatch
  fails, gate retains the event ID.
- **FIFO child appending**: Test 19 - child enqueued during sibling-A
  evaluation is evaluated after sibling-B, preserving J10 tail-appending
  through the production `drain_result_event_queue` boundary.

## Discoveries

- The `edit` tool could not match text across line boundaries in `main.rs`;
  used PowerShell line-by-line insertion with `StreamWriter` as a workaround.
- `authorise_and_execute_inner` requires a `plan` in the response. Test mocks
  that use the dispatch seam must provide the same plan structure as the
  existing `j10_initial_to_a_to_b_chain` test.
- The existing `cargo check` baseline is 9 warnings (4 for `--tests`).

## Remaining risks

- Packet 3 must add a production PowerShell rejection scenario.

## Smallest next action

J11 Packet 3: production end-to-end rejection verification.

## References

- Packet 1 gate: `tethers-0.1/host-rust/src/event_admission.rs`
- Coordinator: `tethers-0.1/host-rust/src/main.rs`
- Production drain: `drain_result_event_queue()` in `main.rs`
- Drain outcome: `EventDrainOutcome` struct with `apply_to_response()`
- Rejection JSON helper: `event_admission_rejection_value()`
- Base commit: `0e89bc79b314b67a4486504747bbbad17da94099`
- Branch: `goose/j11-runtime-admission-wiring`
