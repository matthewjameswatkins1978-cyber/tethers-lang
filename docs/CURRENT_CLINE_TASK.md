# Current Implementation Task

Control contract: `1`

Task: `J11 packet 1 event admission foundation`

Status: `COMPLETE`

Task colour: `Red`

Owner: `Goose`

Route: `Goose — J11 packet 1 in fenced local worktree`

Worker note: `docs/worker-notes/2026-07-28-j11-event-admission-foundation.md`

Base branch: `goose/j10-integration-rehearsal`

Base commit: `6950c4328f4df83701bbc4fc4287c96eee1e2386`

Branch: `goose/j11-event-admission-foundation`

## Objective

Implement a host-local event admission gate that accepts each exact event ID
once per host invocation, rejects duplicate event IDs, accepts causal
generations 0 through 8 inclusive, rejects generation 9 or greater, mutates no
state when admission is rejected, and is fully unit-tested. This packet creates
and tests the pure admission component only. Runtime wiring into the coordinator
belongs to Packet 2.

## Relevant background and existing behaviour

J10 introduced `ResultEventQueue`: a host-owned FIFO queue of generated Result
Anchors. The outer coordinator loop drains one anchor at a time through
`process_one_event`. `InputEventContext` carries causal context (event ID,
correlation ID, generation) through each evaluation. Initial external events
carry generation 0; queued anchors carry their generation from the Anchor's
causal fields.

Before this packet, no admission gate exists: every queued event is processed
unconditionally. There is no duplicate-event-ID rejection and no causal-depth
limit. J10's frozen decisions explicitly defer these to J11.

The accepted base is the J10 integration rehearsal checkpoint at
`6950c4328f4df83701bbc4fc4287c96eee1e2386`.

## Required behaviour

1. A fresh `EventAdmissionGate` has zero admitted event IDs.
2. A unique generation-0 event ID is accepted.
3. Distinct exact event IDs are independently accepted.
4. A second admission of the same exact event ID is rejected as duplicate.
5. Duplicate matching uses exact, case-sensitive string equality.
6. Generation 8 is accepted as a valid causal depth.
7. Generation 9 is rejected as `CausalDepthExceeded`.
8. Any generation greater than 9 is also rejected as `CausalDepthExceeded`.
9. A depth-rejected event does not reserve the event ID; a subsequent
   lower-generation admission of the same ID succeeds.
10. A duplicate-rejected event does not change the admitted count.
11. Depth validation precedes duplicate lookup so an event beyond the causal
    limit always reports depth violation regardless of prior admission.
12. An accepted event ID stays recorded for the lifetime of the gate with no
    removal surface.
13. The gate performs no logging, Trail writing, queue draining, evaluation, or
    dispatch.
14. The gate is process-local, in-memory, non-persistent, and not durable replay
    protection.

## Relevant components

- `tethers-0.1/host-rust/src/event_admission.rs` (new) — the pure admission gate
- `tethers-0.1/host-rust/src/main.rs` — module declaration only
- `tethers-0.1/host-rust/src/event_queue.rs` — existing queue (not modified)
- `tethers-0.1/host-rust/src/result_anchor.rs` — existing anchor type (not modified)

## Frozen decisions and invariants

- The gate is process-local and in-memory with no durable storage.
- It lives for one future host invocation and is not durable replay protection.
- It does not replace J09 execution replay protection.
- Event IDs use exact, case-sensitive string equality.
- Generations `0..=8` are valid; `9` and greater are rejected.
- Depth validation happens before duplicate lookup.
- An event ID is inserted only after every admission check passes.
- A rejected event never changes the admitted-ID set.
- Once an event ID is admitted, it remains admitted even when later evaluation
  or execution fails.
- The component performs no logging, Trail writing, queue draining, evaluation,
  or dispatch.
- No retry or automatic correction exists.
- Empty event-ID validation is outside this packet; upstream assumptions are
  preserved unchanged.
- Runtime wiring belongs to Packet 2 and must not appear in this packet.

## Acceptance criteria

1. A fresh gate reports zero admitted events.
2. A unique generation-0 event is accepted without error.
3. Two distinct exact event IDs are both independently accepted.
4. The second admission of the same exact event ID returns
   `EventAdmissionRejection::DuplicateEventId` with the matching `event_id`.
5. Duplicate matching distinguishes `"EventA"` from `"eventa"` as distinct IDs.
6. Generation 8 is accepted without error.
7. Generation 9 is rejected with `EventAdmissionRejection::CausalDepthExceeded`
   carrying `event_id`, `generation: 9`, and `maximum_generation: 8`.
8. Generation 10 is rejected with the same variant and correct fields.
9. After a depth-rejected admission at generation 9, the same ID at generation 0
   is still accepted, proving the ID was not reserved.
10. A duplicate rejection does not change `admitted_count()`; the count remains
    stable before and after.
11. When both depth and duplicate conditions apply, depth rejection is returned,
    not duplicate.
12. An accepted event ID remains recorded with no removal or retry surface.
13. The `event_admission` module is declared in `main.rs` but not wired into
    `main()`, `process_one_event`, the queue, or any coordinator function.
14. No other source file is modified.

## Required verification

- `cargo fmt --check` passes with no formatting errors.
- `cargo check` passes with no compilation errors.
- `cargo test event_admission -- --nocapture` passes all focused tests.
- `cargo test` passes all tests (expected +12 from this packet).
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

- No wiring of the admission gate into `main()`, `process_one_event`, or any
  runtime coordinator function.
- No modification of J09 replay behaviour.
- No modification of J10 queue behaviour.
- No addition of Trail records or public response fields.
- No changes to OCaml engine, protocols, or fixtures.
- No changes to `Cargo.toml` or `Cargo.lock`.
- No implementation of Packets 2, 3, or 4.
- No beginning of J12.
- No modification of existing code for style only.
- No push, merge, rebase, reset, amend, squash, cherry-pick, stash, or clean.

## Stop conditions

- Any J09 or J10 test regresses.
- The gate is wired into runtime execution.
- `main.rs` changes beyond the module declaration.
- Any other source file is modified.
- A dependency needs to be added.
- A test fails that cannot be fixed within the packet scope.
- The checker fails after honest documentation.
- The worktree deviates from the four authorised paths.

## Expected pre-existing changes

None. The worktree is clean at base commit
`6950c4328f4df83701bbc4fc4287c96eee1e2386`.
