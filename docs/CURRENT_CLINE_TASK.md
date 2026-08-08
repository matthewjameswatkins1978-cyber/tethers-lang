# Current Implementation Task

Control contract: `1`
Task: `F4b — Direct Typed Shared Execution Result`
Owner: `OpenCode`
Model: `DeepSeek Pro HIGH`
Status: `COMPLETE`
Task colour: `Amber`
Route: `OpenCode implements shared-boundary semantic hardening; no new types`
Worker note: `docs/worker-notes/2026-08-08-f4b-direct-execution-outcome.md`
Base branch: `foundation/control-worker-evidence-finalization`
Base commit: `ee86b57f557516bb0ee14b52a295718d66dae2a1`
Implementation branch: `foundation/f4b-direct-execution-outcome`
Implementation checkpoint: `0dc2f56c8262aab16cc3272086a3232a2442d982`
OCaml switch path: `N/A`
Rust toolchain: `1.97.1`

## Objective

Remove the remaining internal semantic round-trip in the shared Rust execution boundary. `authorise_and_execute_inner` now returns `SharedExecutionResult` directly. `execute_boundary_impl` constructs the typed outcome at each terminal branch. Presentation JSON continues to be written for compatibility but is never read back for internal semantic truth.

## Relevant background and existing behaviour

F4a2 established the typed planner boundary. F4b finishes Foundation F4 by removing the last internal JSON string reconstruction (`from_response_and_evidence` reading `execution_status`). The JSON remains frozen as presentation/wire state.

Previously, `execute_shared_boundary` called `execute_boundary_impl` which returned `ExecutionBoundaryEvidence` (only the execution_id), then used `from_response_and_evidence` to read `response["execution_status"]` and reconstruct `SharedExecutionOutcome`. This internal JSON round-trip was the last dependency on JSON-as-semantic-truth in the shared execution boundary.

## Required behaviour

1. `authorise_and_execute_inner` returns `SharedExecutionResult` directly.
2. `execute_boundary_impl` constructs `SharedExecutionResult { outcome, execution_id }` at every terminal branch.
3. `execute_shared_boundary` performs audit-failure override on typed result.
4. `ExecutionBoundaryEvidence` and `from_response_and_evidence` are removed from production.
5. Frozen response JSON (`execution_status`) is still written for presentation compatibility but never read for semantic truth.
6. `SharedExecutionOutcome` unchanged.
7. `ExecutionServiceResult` and `ExecutionServiceError` untouched.
8. No `InternalExecutionResult` or equivalent wrapper introduced.
9. All existing `execution_id` Some/None semantics preserved at every terminal branch.

## Relevant components

`tethers-0.1/host-rust/src/application.rs` — 1 production file (122 insertions, 120 deletions)

Key changes:

- `execute_boundary_impl` return type: `ExecutionBoundaryEvidence` → `SharedExecutionResult`
- Every terminal branch constructs precise `SharedExecutionOutcome` at point of semantic truth
- `execute_shared_boundary`: audit-failure override on typed result (no JSON read-back)
- `authorise_and_execute_inner` returns `SharedExecutionResult` directly (no `map(|_| ())`)
- Removed `ExecutionBoundaryEvidence` struct
- Removed `from_response_and_evidence` method
- Wrapper functions (`authorise_and_execute`, etc.) retain `Result<(), ...>` with `.map(|_| ())`

## Frozen decisions and invariants

- `SharedExecutionOutcome` variant set is frozen: Completed, Failed, Uncertain, Unattempted, Denied, AuditFailed, Replay(ReplayDispatchResult).
- `SharedExecutionResult { outcome, execution_id }` is the single typed internal return channel.
- `ExecutionServiceResult` and `ExecutionServiceError` are not redesigned.
- No `InternalExecutionResult` or equivalent.
- Frozen response JSON continues to be written but never read for internal semantic truth.
- No other production file changed.
- No OCaml changed.

## Acceptance criteria

1. All terminal branches in `execute_boundary_impl` return `SharedExecutionResult` with correct outcome and execution_id — verified by `cargo test --locked` (1331 PASS).
2. `from_response_and_evidence` removed from production — confirmed by grep (zero production references).
3. `ExecutionBoundaryEvidence` removed — confirmed by grep (zero production references, only one comment updated).
4. No production path reads `response["execution_status"]` for semantic truth — confirmed by execution_status inventory (all production references are writes).
5. Frozen JSON presentation preserved — all existing `response["execution_status"]` writes retained; all test assertions on presentation JSON pass.
6. `SharedExecutionOutcome` unchanged — structure identical to base.
7. `ExecutionServiceResult` and `ExecutionServiceError` untouched — confirmed via diff.
8. No OCaml changed — confirmed via diff.
9. No F5 started — confirmed.

## Required verification

- `cargo test --locked`: 1331 PASS, 0 FAIL, 2 ignored
- `cargo fmt --manifest-path Cargo.toml -- --check`: application.rs clean; replay_windows.rs:3277 pre-existing discrepancy (proven, not F4b-introduced)
- `git diff --check`: PASS
- `check-tethers-task-packet.ps1`: PASS (post-closeout)
- `execution_status` inventory: zero production reads for semantic truth

## Forbidden changes

- No `SharedExecutionOutcome` variant changes
- No `ExecutionServiceResult` redesign
- No `ExecutionServiceError` redesign
- No `InternalExecutionResult` or equivalent
- No OCaml changes
- No F4a1/F4a2 code changes (beyond the F4b semantic boundary change)
- No F5 structural extraction
- No new dependencies
- No `replay_windows.rs` changes

## Stop conditions

STOP if:
- Exact JSON compatibility requires keeping semantic read-back
- Removing read-back changes replay classification
- Removing read-back changes execution identity behaviour
- Audit failure cannot preserve current precedence
- Another production module must change
- ExecutionServiceResult must change
- SharedExecutionOutcome must change
- More than two genuinely new focused tests appear necessary
- OCaml changes appear necessary

NONE triggered.

## Expected pre-existing changes

1. Pre-existing `cargo fmt` discrepancy in `replay_windows.rs:3277` (proven, not F4b-introduced).
2. Implementation checkpoint `0dc2f56c` covers all production/test changes.
3. Closeout docs (`CURRENT_CLINE_TASK.md`, worker note) are the only files after checkpoint.
