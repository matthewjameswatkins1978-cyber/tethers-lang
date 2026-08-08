# Worker Note

Task: `F4b — Direct Typed Shared Execution Result`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `ee86b57f557516bb0ee14b52a295718d66dae2a1`

Implementation checkpoint: `0dc2f56c8262aab16cc3272086a3232a2442d982`

Branch: `foundation/f4b-direct-execution-outcome`

## Requested outcome

Remove the internal semantic round-trip where `execute_shared_boundary` called `execute_boundary_impl` to get `ExecutionBoundaryEvidence`, then reconstructed `SharedExecutionResult` by reading `response["execution_status"]`. Replace with direct typed propagation.

## Changes made

**Production file (1):** `tethers-0.1/host-rust/src/application.rs`

- `execute_boundary_impl` now returns `Result<SharedExecutionResult, Box<dyn std::error::Error>>` instead of `Result<ExecutionBoundaryEvidence, ...>`.
- Every terminal branch constructs `SharedExecutionResult { outcome, execution_id }` at the point the semantic truth is known.
- `execute_shared_boundary` performs the audit-failure override check directly on the typed result (no `from_response_and_evidence`).
- `authorise_and_execute_inner` returns `SharedExecutionResult` directly (no `map(|_| ())`).
- Removed `ExecutionBoundaryEvidence` struct (line ~2206).
- Removed `from_response_and_evidence` method (line ~2211).
- Wrapper functions retain `Result<(), ...>` return type via `.map(|_| ())`.
- Added `session_outcome` variable in `execute_boundary_impl` to carry the semantic outcome through all post-execution terminal paths.
- Comment updated: `ExecutionBoundaryEvidence` → `SharedExecutionResult` (line ~6510).

**No other production file changed.** No OCaml. No `SharedExecutionOutcome` change. No `ExecutionServiceResult` change. No `ExecutionServiceError` change. No `InternalExecutionResult`.

## Decisions and assumptions

- `SharedExecutionResult { outcome, execution_id }` is the single typed return channel. A second `{ outcome, execution_id }` type would add no invariant.
- The audit-failure override stays in `execute_shared_boundary` (the single outer point); `execute_boundary_impl` returns the execution-level semantic outcome unchanged.
- `session_outcome` variable in `execute_boundary_impl` carries the provider-outcome classification through all post-execution terminal branches rather than re-matching each time.
- `execution_status` writes continue for frozen presentation compatibility; they are never used as internal truth.
- `ReplayDispatchResult` is `Copy`, so it can be used in both `set_replay_result` and the `SharedExecutionResult` construction without cloning.

## Evidence

### execution_status inventory (application.rs)

All production references to `"execution_status"` are WRITES:

| Line | Context | Classification |
|------|---------|---------------|
| 2183 | `present_non_dispatchable_response` | WRITE (presentation) |
| 2463 | prepare_and_record failure in `execute_boundary_impl` | WRITE (presentation) |
| 2489 | deadline expiry in `execute_boundary_impl` | WRITE (presentation) |
| 2626 | trail append failure in `execute_boundary_impl` | WRITE (presentation) |
| 2692 | success path in `execute_boundary_impl` | WRITE (presentation) |
| 2754 | `set_replay_result` | WRITE (presentation) |

All reads are in test assertions verifying frozen presentation JSON. **No production path reads `execution_status` for internal semantic truth.**

### Tests changed/added/removed

**Removed (2):**
- `j14a_from_response_and_evidence_ignores_host_id_in_json` — tested removed `from_response_and_evidence`
- `j14a_from_response_and_evidence_no_id_when_evidence_is_none` — tested removed `from_response_and_evidence`

**Replaced (2):**
- `j14a_audit_failure_without_evidence_has_no_id` → `j14a_audit_failure_without_id_is_none` — constructs `SharedExecutionResult` directly
- `j14a_audit_failure_with_evidence_carries_id` → `j14a_audit_failure_carries_trusted_id` — constructs `SharedExecutionResult` directly

**Added (2):**
- `j14a_direct_result_construction_requires_no_json` — proves `SharedExecutionResult` constructed without any JSON input
- `j14a_response_execution_status_does_not_alter_typed_outcome` — proves `response["execution_status"]` value ("denied") does not alter the typed `SharedExecutionOutcome::Completed` in the result

### Preserved behaviour confirmed

- Frozen response JSON (`execution_status` values): preserved
- `execution_id` Some/None at each terminal branch: preserved
- Audit failure precedence: preserved (override in `execute_shared_boundary`)
- Replay classification: preserved (every `ReplayDispatchResult` variant mapped exactly)
- All existing J14/J14B/J09 tests: passing
- `map_shared_result` in `host_execution.rs`: unchanged

### Verification against committed checkpoint `0dc2f56`

| Check | Result |
|-------|--------|
| `cargo test --locked` | 1331 PASS, 0 FAIL, 2 ignored |
| `cargo fmt --manifest-path Cargo.toml -- --check` | PASS (application.rs clean; replay_windows.rs pre-existing discrepancy) |
| `git diff --check` | PASS |
| `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` | PASS (post-closeout) |
| OCaml untouched | confirmed |
| F4a1/F4a2 code unchanged | confirmed |
| F5 not started | confirmed |
| `SharedExecutionOutcome` unchanged | confirmed |
| `ExecutionServiceResult` unchanged | confirmed |
| `ExecutionServiceError` unchanged | confirmed |
| No `InternalExecutionResult` | confirmed |
| No other production file changed | confirmed |

## Discoveries

None.

## Remaining risks

None known within packet scope.

## Smallest next action

Lucy review and acceptance. DO NOT BEGIN F5.

## References

- `tethers-0.1/host-rust/src/application.rs` — implementation
- Branch: `foundation/f4b-direct-execution-outcome`
- Implementation checkpoint: `0dc2f56c8262aab16cc3272086a3232a2442d982`
- Base: `ee86b57f557516bb0ee14b52a295718d66dae2a1`
