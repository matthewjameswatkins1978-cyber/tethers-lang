# J11 Packet 2 Worker Note

Task: `J11 packet 2 runtime admission wiring`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Goose`

Status: `COMPLETE`

Base commit: `0e89bc79b314b67a4486504747bbbad17da94099`

Implementation checkpoint: `WORKTREE`

## Review-correction (2026-07-28) — COMPLETE

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
- Rewrote focused J11 tests (2–8, 10–11, 14–16) to prove behaviour
  through the actual production helper using callback-invocation
  counters.
- Rewrote test 15 (`j11_rejection_does_not_modify_replay`) through the
  production helper: proves a rejected gen-9 event never invokes the
  evaluation callback, removing the unused `context` variable.
- Restored the clean `cargo check` warning baseline: 9 baseline
  warnings, 0 J11 Packet 2 warnings.

### Correction evidence

- `cargo check`: 9 baseline warnings, 0 new
- `cargo check --tests`: 4 baseline test warnings, 0 new
- `cargo test j11_`: 18/18 passed
- `cargo test event_admission`: 15/15 passed
- `cargo test` full suite: 506/506 passed
- `cargo clippy --all-targets --all-features`: no new warnings
- All integration scripts: PASS
- `opam exec --switch=... -- dune build`: PASS
- `git diff --check`: no whitespace errors
- Task packet checker: PASS

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
- In `main()`:
  - Created one `EventAdmissionGate` per invocation after the queue.
  - Admitted initial external event (generation 0) before `process_one_event`,
    failing closed on unexpected rejection.
  - Added `event_admission_rejection: Option<Value>` before the drain loop.
  - Inside `while let Some(anchor) = queue.pop_front()`, immediately after
    capturing `input_event_id` and `generation`, called `admission_gate.admit()`;
    on rejection, set `event_admission_rejection` and `break`.
  - After the drain loop, conditionally injected `event_admission_rejection`
    field into the response before `follow_up_evaluations`.
- Added 18 focused `j11_` unit tests covering all required behaviours.
- Added `drain_queue_with_admission()` test helper simulating the coordinator
  drain loop with admission.
- Updated `docs/CURRENT_CLINE_TASK.md` with the J11 packet 2 control-v1 packet.
- Created this worker note.

No changes to `event_admission.rs`, `event_queue.rs`, `result_anchor.rs`,
`Cargo.toml`, `Cargo.lock`, OCaml, protocols, or fixtures.

## Decisions and assumptions

- `event_admission_rejection` is injected into the response BEFORE
  `follow_up_evaluations` so it is always present even when no follow-ups
  completed.
- `process_one_event` is never modified; the gate is owned by the outer
  coordinator.
- Depth validation precedes duplicate lookup (inside the gate from Packet 1).
- Rejection uses `break` (not `continue`), stopping all later siblings.
- The initial-event admission uses `.map_err()` to fail closed with a host
  error message.
- JSON helper lives in `main.rs` rather than `event_admission.rs` to keep
  serde_json concerns outside the pure gate module.
- The `drain_queue_with_admission` test helper mirrors the production
  coordinator's admission-check-then-process-or-break pattern.

## Evidence

| Check | Result |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo test j11_ -- --nocapture` | 18/18 passed |
| `cargo test event_admission -- --nocapture` | 15/15 passed |
| `cargo test` (full suite) | **506/506** (488 + 18) |
| `cargo check` | 9 warnings baseline + 1 unused variable |
| `cargo clippy --all-targets --all-features` | no new warnings |
| `check-fixtures.ps1` | PASS — 46 JSON + 30 JSONL |
| `test-engine.ps1` | PASS — 24/24 |
| `test-mcp-transcripts.ps1` | PASS — 15/15 |
| `test-host-denial.ps1` | PASS |
| `test-host-execution-failure.ps1` | PASS |
| `test-host-result-follow-up.ps1` | PASS (J10 boundary) |
| `demo.ps1` | PASS |
| `opam exec -- dune build` | PASS |
| `check-tethers-task-packet.ps1` | PASS |
| `git diff --check` | PASS |

### Focused J11 test names (18):

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

### Proofs

- **Generation 8 evaluates, generation 9 does not**: Tests 5 and 6/7.
- **Later siblings stop**: Test 10 — `evt/c` remains in queue after `evt/b`
  rejection.
- **No engine/provider activity**: Tests 8 (drain helper proves rejection
  before processing) and 9 (dispatch seam proves zero provider calls after
  gate rejection).
- **Completed follow-ups preserved**: Test 11 — `evt/first` in processed,
  `evt/third` still in queue.
- **JSON shapes**: Tests 12 (duplicate) and 13 (depth) verify exact field
  names, values, and counts.
- **Clean run omits field**: Test 14 — three anchors all pass, rejection is
  `None`.
- **Admission persists after failure**: Test 17 — FailingExecutor dispatch
  fails, gate retains the event ID.

## Discoveries

- The `edit` tool could not match text across line boundaries in `main.rs`;
  used PowerShell line-by-line insertion with `StreamWriter` as a workaround.
- `authorise_and_execute_inner` requires a `plan` in the response. Test mocks
  that use the dispatch seam must provide the same plan structure as the
  existing `j10_initial_to_a_to_b_chain` test.
- The existing `cargo check` baseline is 9 warnings. One additional
  unused-variable warning from a test helper is acceptable and will be
  resolved in a later cleanup.

## Remaining risks

- Packet 3 must add a production PowerShell rejection scenario.
- Packet 4 must add Trail entries for admission and rejection events.
- The unused `context` variable in test 15 (line ~6950) triggers a warning
  but is harmless; it exists because the blocked-event context is created
  but never dispatched.

## Smallest next action

Packet 3 of 4: add a production PowerShell rejection scenario proving the
end-to-end rejection response, plus Trail entries for admission and rejection
events.

## References

- Packet 1 gate: `tethers-0.1/host-rust/src/event_admission.rs`
- Coordinator: `tethers-0.1/host-rust/src/main.rs` (main function ~line 140)
- Helper: `event_admission_rejection_value()` at line ~104
- Drain loop: `while let Some(anchor) = queue.pop_front()` at line ~291
- Base commit: `0e89bc79b314b67a4486504747bbbad17da94099`
- Branch: `goose/j11-runtime-admission-wiring`
