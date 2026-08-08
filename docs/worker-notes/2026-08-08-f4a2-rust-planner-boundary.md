# Worker Note

Task: `F4a2 — Rust Typed Planner Response Boundary`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `8a2ef5fdafb56faca59e47370c3d6e7892f5a437`

Implementation checkpoint: `WORKTREE`

## Requested outcome

Replace the Rust host's repeated interpretation of raw planner `serde_json::Value` status strings with a deliberately staged typed boundary: MCP structuredContent -> PlannerResponseWire -> host correlation/protocol validation -> PlannerOutcome -> existing execution machinery.

## Changes made

- `tethers-0.1/host-rust/src/engine_stdio.rs` — Added `PlannerResponseWire` enum (Matched, NotMatched, Error, Unknown). Added private `classify_wire_response` helper. Changed `evaluate_tether` return from `Result<Value, EngineError>` to `Result<PlannerResponseWire, EngineError>`. Added `j13b_wire_missing_or_non_string_status_is_engine_error` test.
- `tethers-0.1/host-rust/src/host_execution.rs` — Added `PlannerErrorOutcome` enum (Contextual, Request) and `PlannerOutcome` enum (Matched, NotMatched, Error). Replaced `PlannerResponseRoute` with new types. Changed `classify_planner_response` to accept `PlannerResponseWire`, return `Result<PlannerOutcome, ExecutionServiceResult>`. Replaced `route_planner_response` with `route_planner_outcome`. Inlined `validate_planner_error_correlation` into `classify_planner_response`. Updated `evaluate_one` to stage through wire. Adapted all j13b tests. Added `j13b_extra_planner_response_fields_are_tolerated` test.
- `tethers-0.1/host-rust/src/replay_windows.rs` — fmt-only line-break.

## Decisions and assumptions

- `PlannerResponseWire::Matched(Value)` — raw matched Value deliberately retained; downstream execution needs it for mutation (strip execution_id, extract proposed action, apply policy/replay/provider calls).
- No `CorrelatedPlannerResponse` wrapper — rejected per frozen design.
- No strict Serde tagged decoder — manual status interpretation preserves missing/non-string vs. unknown-string distinction.
- Unknown extra fields remain tolerated (no `deny_unknown_fields`).
- Missing/non-string status remains `EngineError::EvaluationFailed` at the engine boundary.
- Unknown string status remains `ExecutionServiceResult::InvalidData` at the host boundary.
- Exact error validation order preserved: protocol_version first, then correlation fields check, then all-or-nothing correlation, then error object/code/message extraction.
- `validate_planner_correlation` and `require_planner_field` helpers retained unchanged.

## Evidence

- `cargo test --locked` — 1331 passed, 0 failed, 2 ignored (pre-existing)
- `cargo fmt -- --check` — PASS
- `check-tethers-task-packet.ps1` — PASS (control-v1/COMPLETE, from accepted F4a1 base)
- `git diff --check` — PASS (whitespace clean)
- Production files changed: 2 (`engine_stdio.rs`, `host_execution.rs`)

### Test evidence

- `j13b_wire_missing_or_non_string_status_is_engine_error` — missing/non-string status -> `EngineError::EvaluationFailed`; unknown string -> `PlannerResponseWire::Unknown`
- `j13b_matched_response_validates_every_correlation_before_dispatch` — staged through `PlannerResponseWire::Matched` and `PlannerOutcome::Matched`
- `j13b_not_matched_is_no_actions_without_dispatch` — staged through `PlannerResponseWire::NotMatched`
- `j13b_correlated_and_minimal_planner_errors_are_distinct` — contextual (Some evaluation_id) vs request-level (None) preserved
- `j13b_every_planner_correlation_mismatch_is_invalid_data` — all correlation permutations
- `j13b_unknown_planner_status_is_invalid_data` — `PlannerResponseWire::Unknown` -> `InvalidData`
- `j13b_rejected_error_and_invalid_routes_make_zero_replay_or_provider_calls` — zero-dispatch for all terminal outcomes
- `j13b_extra_planner_response_fields_are_tolerated` — extra field passes through
- `j13b_retained_engine_*` tests — real OCaml MCP engine crosses Stage 1 boundary via `PlannerResponseWire::Matched`

## Discoveries

- `cargo fmt` corrected a pre-existing line-break in `replay_windows.rs` (unrelated assertion call).
- `wire_from_response` test helper added but not strictly needed; retained because it simplifies the correlation-mismatch loop.

## Remaining risks

- `ExecutionServiceResult::PlannerError` still carries raw `code`/`message` strings rather than an enum; that is F4b scope.
- Plan and Actions remain as raw `Value`; that is F5 scope.
- `PlannerErrorOutcome` and `PlannerOutcome` are not `pub`; they remain private to `host_execution.rs`. Public exposure is F5 scope.

## Smallest next action

F4b: clean up `ExecutionServiceResult` sizing/clarity — DO NOT BEGIN without separate packet.

## References

- `tethers-0.1/host-rust/src/engine_stdio.rs` — production
- `tethers-0.1/host-rust/src/host_execution.rs` — production
- `docs/CURRENT_CLINE_TASK.md` — F4a2 task packet
- Branch: `foundation/f4a2-rust-planner-boundary`
- F4a1 implementation checkpoint: `6326e5672b1bd34cc3054a9b42488727de61b7e1`
- Accepted base: `8a2ef5fdafb56faca59e47370c3d6e7892f5a437`
