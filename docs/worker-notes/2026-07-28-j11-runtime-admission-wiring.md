# J11 Packet 2 Worker Note

Task: `J11 packet 2 runtime admission wiring`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Goose`

Status: `COMPLETE`

Base commit: `0e89bc79b314b67a4486504747bbbad17da94099`

Implementation checkpoint: `41b6848a09db5edb0bd36f4bcc1e0aa39cf7eeb3`

Final acceptance cleanup: `148225399d51d325681c61b847c36510e9867ff2`

## Review-correction (2026-07-28) - COMPLETE

Independent review accepted the runtime wiring logic. Two evidence-quality issues
held final acceptance:

1. `cargo check --tests` reported an unused `context` variable in test 15
   (`j11_rejection_does_not_modify_replay`), violating the no-new-warnings
   contract.

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
- Rewrote test 15 (`j11_rejection_does_not_modify_replay`) through the
  production helper: proves a rejected gen-9 event never invokes the
  evaluation callback, removing the unused `context` variable.
- Restored the clean `cargo check` warning baseline.

## Final acceptance cleanup (2026-07-28) - COMPLETE

- Removed an embedded ESC (0x1B) control character in the doc comment for
  `drain_result_event_queue`, repairing "evaluate" to ordinary UTF-8.
- Scanned both authorised files for unexpected control characters (excluding
  TAB, CR, LF); none found.
- Removed untracked `tools/` directory generated during Goose task work.
- Strengthened `j11_completed_follow_ups_preserved_before_rejection`:
  added exact `follow_up_evaluations[0]` content assertion, verified
  `apply_to_response` preserves a pre-existing `status` field on the
  response object, and re-verified rejection JSON after mutation.
- Added `j11_fifo_child_appended_during_drain`: proves that enqueueing a
  child anchor inside the evaluate callback preserves J10 FIFO
  tail-appending through the production `drain_result_event_queue`
  boundary. Evaluated order: sibling-A, sibling-B, child-A1.
- Updated this worker note to a single internally consistent record.

### Final acceptance evidence

- `cargo check`: 9 baseline warnings, 0 new
- `cargo check --tests`: 4 baseline test warnings, 0 new
- `cargo test j11_`: 19/19 passed
- `cargo test event_admission`: 15/15 passed
- `cargo test` full suite: 507/507 passed
- `cargo fmt --check`: PASS
- `cargo clippy --all-targets --all-features`: no new warnings
- Task packet checker: PASS
- `git diff --check`: no whitespace errors
- Only `docs/worker-notes/2026-07-28-j11-runtime-admission-wiring.md`
  and `tethers-0.1/host-rust/src/main.rs` modified.

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
9. `j11_rejection_prevents_dispatch_entry`
10. `j11_rejection_stops_later_siblings`
11. `j11_completed_follow_ups_preserved_before_rejection`
12. `j11_duplicate_rejection_json_exact_shape`
13. `j11_depth_rejection_json_exact_shape`
14. `j11_clean_run_omits_rejection_field`
15. `j11_rejection_does_not_modify_replay`
16. `j11_rejection_produces_no_anchor`
17. `j11_admission_persists_when_evaluation_fails`
18. `j11_max_evaluation_generation_is_eight`
19. `j11_fifo_child_appended_during_drain`

### Proofs

- **Generation 8 evaluates, generation 9 does not**: Tests 5 and 6/7.
- **Later siblings stop**: Test 10 - `evt/c` remains in queue after `evt/b`
  rejection.
- **No engine/provider activity on rejection**: Tests 8 (callback never
  invoked for rejected events) and 9 (dispatch seam proves zero provider
  calls after gate rejection).
- **Completed follow-ups preserved**: Test 11 - `evt/first` is evaluated
  and its completed response entry is present in `follow_up_evaluations[0]`
  with exact `input_event_id`, `generation`, and `response` content;
  `apply_to_response` does not disturb a pre-existing `status` key.
- **Callback non-entry**: Tests 8 and 15 prove rejected events never invoke
  the evaluate callback.
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
- Base commit: `0e89bc79b334b67a4486504747bbbad17da94099`
- Branch: `goose/j11-runtime-admission-wiring`
