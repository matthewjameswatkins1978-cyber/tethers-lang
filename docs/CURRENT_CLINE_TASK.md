# Current Implementation Task

Control contract: `1`

Task: `J11 packet 2 runtime admission wiring`

Status: `COMPLETE`

Task colour: `Red`

Owner: `Goose`

Route: `Goose — J11 packet 2 in fenced local worktree`

Worker note: `docs/worker-notes/2026-07-28-j11-runtime-admission-wiring.md`

Base branch: `goose/j11-event-admission-foundation`

Base commit: `0e89bc79b314b67a4486504747bbbad17da94099`

Branch: `goose/j11-runtime-admission-wiring`

## Objective

Wire the accepted `EventAdmissionGate` from Packet 1 into the real host
invocation and FIFO follow-up coordinator so that duplicate event IDs and
events exceeding generation 8 are visibly stopped before evaluation. The gate
blocks the rejected event and all later queued siblings while preserving
completed follow-ups and clean J10 behaviour.

## Relevant background and existing behaviour

Packet 1 delivered `EventAdmissionGate` in `event_admission.rs`: a pure, tested
admission component that accepts each exact event ID once per invocation and
rejects generations 9+. It was not wired into any runtime path.

J10 introduced the outer FIFO coordinator in `main()`: an initial
`process_one_event` call followed by a `while let Some(anchor) =
queue.pop_front()` drain loop that builds follow-up requests and processes each
queued Result Anchor serially. `follow_up_evaluations` collects completed
follow-up responses. No admission or deduplication exists in the coordinator
today.

`InputEventContext::for_initial` produces a generation-0 context. Queued anchors
inherit their generation from the Anchor's causal fields.

The accepted Packet 1 checkpoint is
`0e89bc79b314b67a4486504747bbbad17da94099`.

## Required behaviour

1. One `EventAdmissionGate` is created per host invocation and lives for the
   entire invocation.

2. The initial external event is admitted before `process_one_event` is called,
   using the external event ID and generation 0.

3. An unexpected initial-event admission rejection fails closed as a host error.

4. Each queued Result Anchor is admitted immediately after `pop_front()` and
   before `InputEventContext::from_result_anchor`, `build_follow_up_request`,
   or any evaluation, replay, policy, dispatch, provider, or Trail activity.

5. A duplicate event ID in the queue is rejected and not evaluated.

6. A queued event with generation 9 or greater is rejected and not evaluated.

7. The first queued-event rejection stops the entire remaining drain: no later
   siblings are processed, no retry, no reinsertion, and no `continue`.

8. Completed follow-up evaluations that finished before the rejection remain in
   `follow_up_evaluations`.

9. A rejection produces exactly one top-level `event_admission_rejection` field
   with frozen shape. The field is omitted when no rejection occurs.

10. Rejected events cause zero engine calls, planner evaluations, replay
    lookups, policy evaluations, provider calls, or Trail activity.

11. The initial event ID stays admitted even when initial evaluation fails.

12. An admitted queued event ID stays admitted even when its evaluation fails.

13. No generation-9 or greater event is ever evaluated.

14. Clean J10 behaviour (no duplicate, no over-depth) is unchanged.

## Relevant components

- `tethers-0.1/host-rust/src/event_admission.rs` — Packet 1 gate (not modified)
- `tethers-0.1/host-rust/src/main.rs` — coordinator, `process_one_event`,
  `build_follow_up_request`, outer drain loop, test module
- `tethers-0.1/host-rust/src/event_queue.rs` — queue (not modified)
- `tethers-0.1/host-rust/src/result_anchor.rs` — anchor type (not modified)

## Frozen decisions and invariants

- One `EventAdmissionGate` per invocation.
- Initial event admitted before evaluation; failure is a host error.
- Queued admission happens before request construction, engine, or any
  side-effecting component.
- First queued rejection stops the drain via `break`; no `continue`, no skip,
  no retry.
- Rejection JSON uses exact snake_case field names: `event_admission_rejection`,
  `kind`, `event_id`, `generation`, `maximum_generation`, `processing`.
- `processing` is always `"stopped"`.
- `kind` is `"duplicate_event_id"` or `"causal_depth_exceeded"`.
- `event_admission_rejection` omitted when no rejection occurs.
- `follow_up_evaluations` contains only evaluations completed before rejection.
- `process_one_event` never owns the gate, pops the queue, calls itself, or
  decides whether the drain continues.
- No new Trail records, command-line flags, environment bypasses, or protocol
  changes.
- J09 replay and J10 FIFO semantics preserved unchanged.

## Acceptance criteria

1. Initial external event admitted before initial `process_one_event`; unexpected
   rejection fails closed.
2. Clean unique follow-up evaluated normally and appears in
   `follow_up_evaluations`.
3. Queued event reusing the initial external event ID is rejected as duplicate.
4. Duplicate queued sibling not evaluated twice.
5. Generation-8 queued event evaluated normally.
6. Generation-9 queued event not evaluated.
7. Generation greater than 9 not evaluated.
8. Rejected event causes zero engine calls.
9. Rejected event causes zero provider calls.
10. Rejection stops later queued siblings.
11. Completed follow-ups before rejection preserved in `follow_up_evaluations`.
12. Duplicate rejection JSON exactly matches frozen shape.
13. Depth rejection JSON exactly matches frozen shape.
14. Clean run (no rejection) omits `event_admission_rejection`.
15. Rejection does not modify J09 replay state.
16. Rejection does not produce or enqueue another Result Anchor.
17. Admission persists when a later evaluation fails.
18. Maximum evaluation generation observed is 8, never 9.

## Required verification

- `cargo fmt --check` passes.
- `cargo check` passes with no new warnings.
- `cargo test j11_ -- --nocapture` passes all focused tests.
- `cargo test event_admission -- --nocapture` passes 15/15.
- `cargo test` full suite passes.
- `cargo clippy --all-targets --all-features` passes with no new warnings.
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` passes.
- `pwsh -NoProfile -File tethers-0.1/scripts/check-fixtures.ps1` passes.
- `pwsh -NoProfile -File tethers-0.1/scripts/test-engine.ps1` passes.
- `pwsh -NoProfile -File tethers-0.1/scripts/test-mcp-transcripts.ps1` passes.
- `pwsh -NoProfile -File tethers-0.1/scripts/test-host-denial.ps1` passes.
- `pwsh -NoProfile -File tethers-0.1/scripts/test-host-execution-failure.ps1` passes.
- `pwsh -NoProfile -File tethers-0.1/scripts/test-host-result-follow-up.ps1` passes.
- `pwsh -NoProfile -File tethers-0.1/scripts/demo.ps1` passes.
- `opam exec -- dune build` in engine-ocaml passes.
- `git diff --check` reports no whitespace errors.

## Forbidden changes

- No modification of `event_admission.rs` semantics.
- No modification of J09 replay behaviour.
- No modification of J10 FIFO ordering.
- No `continue` skipping a rejected item.
- No evaluation after the first rejection.
- No retry, reinsertion, parallelism, or recursion.
- No new Trail schema or Trail records.
- No command-line flags or environment-variable bypasses.
- No production PowerShell rejection scenario (Packet 3).
- No changes to Cargo.toml, Cargo.lock, OCaml, protocols, or fixtures.
- No push, merge, rebase, reset, amend, squash, or cherry-pick.

## Stop conditions

- Any existing J09, J10, or event_admission test regresses.
- A rejected event is evaluated.
- The drain loop continues after a rejection.
- `process_one_event` gains admission responsibility.
- The gate is recreated per-event.
- A rejection produces an unstructured Rust error instead of structured JSON.
- `main.rs` diff exceeds the bounded coordinator changes.
- A dependency needs to be added.

## Expected pre-existing changes

None. The worktree is clean at base commit
`0e89bc79b314b67a4486504747bbbad17da94099`.
