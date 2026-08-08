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

## Relevant background

F4a2 established the typed planner boundary. F4b finishes Foundation F4 by removing the last internal JSON string reconstruction (`from_response_and_evidence` reading `execution_status`). The JSON remains frozen as presentation/wire state.

## Files changed

`tethers-0.1/host-rust/src/application.rs` — 1 production file (122 insertions, 120 deletions)

## Key changes

- `execute_boundary_impl` return type: `ExecutionBoundaryEvidence` → `SharedExecutionResult`
- Every terminal branch constructs precise `SharedExecutionOutcome` at point of semantic truth
- `execute_shared_boundary`: audit-failure override on typed result (no JSON read-back)
- `authorise_and_execute_inner` returns `SharedExecutionResult` directly (no `map(|_| ())`)
- Removed `ExecutionBoundaryEvidence` struct
- Removed `from_response_and_evidence` method
- Removed `execution_status` read-back from production shared execution flow
- Wrapper functions (`authorise_and_execute`, etc.) retain `Result<(), ...>` with `.map(|_| ())`
- `SharedExecutionOutcome` unchanged
- `ExecutionServiceResult` and `ExecutionServiceError` untouched
- `map_shared_result` in `host_execution.rs` unchanged (still receives typed `SharedExecutionResult`)

## Tests

- Removed: `j14a_from_response_and_evidence_ignores_host_id_in_json`, `j14a_from_response_and_evidence_no_id_when_evidence_is_none` (tested removed function)
- Replaced: `j14a_audit_failure_without_evidence_has_no_id` → `j14a_audit_failure_without_id_is_none`, `j14a_audit_failure_with_evidence_carries_id` → `j14a_audit_failure_carries_trusted_id` (no longer use `ExecutionBoundaryEvidence`)
- Added: `j14a_direct_result_construction_requires_no_json` (SharedExecutionResult constructed without any JSON), `j14a_response_execution_status_does_not_alter_typed_outcome` (proves JSON execution_status does not alter typed outcome)

## execution_status inventory

No production path reads `execution_status` for semantic truth. All production references are WRITES for frozen presentation. Test reads are all assertions on presentation JSON.

## Verification

- `cargo test --locked`: 1331 PASS, 0 FAIL, 2 ignored
- `cargo fmt --manifest-path Cargo.toml -- --check`: application.rs clean; replay_windows.rs:3277 pre-existing discrepancy (proven, not F4b-introduced)
- `git diff --check`: PASS
- `check-tethers-task-packet.ps1`: PASS (post-closeout)
- OCaml untouched
- F5 not started
- `SharedExecutionOutcome` unchanged
- `ExecutionServiceResult` untouched
- `ExecutionServiceError` untouched
- No `InternalExecutionResult` or equivalent wrapper
- No other production file changed
