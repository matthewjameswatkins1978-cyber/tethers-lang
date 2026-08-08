# Current Implementation Task

Control contract: `1`
Task: `F4a2 — Rust Typed Planner Response Boundary`
Owner: `OpenCode`
Model: `DeepSeek Pro HIGH`
Status: `COMPLETE`
Task colour: `Amber`
Route: `OpenCode implements F4a2 Rust typed planner response boundary; do not begin F4b or F5`
Worker note: `docs/worker-notes/2026-08-08-f4a2-rust-planner-boundary.md`
Base branch: `foundation/f4a1-ocaml-evaluation-outcome`
Base commit: `8a2ef5fdafb56faca59e47370c3d6e7892f5a437`
F4a1 implementation checkpoint: `6326e5672b1bd34cc3054a9b42488727de61b7e1`
Implementation branch: `foundation/f4a2-rust-planner-boundary`
OCaml switch path: `N/A`
Rust toolchain: read exact channel from `rust-toolchain.toml`; use plain Cargo (resolved by root pin); `--locked` mandatory
Toolchain preflight: `pwsh -NoProfile -File scripts/check-dev-tools.ps1`

## Objective

Replace the Rust host's repeated interpretation of raw planner `serde_json::Value` status strings with a deliberately staged typed boundary: MCP structuredContent -> PlannerResponseWire -> host correlation/protocol validation -> PlannerOutcome -> existing execution machinery. Rust counterpart to accepted F4a1.

## Relevant background and existing behaviour

`EngineSession::evaluate_tether` returned raw `serde_json::Value` and required status to exist as a string. `HostExecutionService::classify_planner_response` switched on raw status strings ("matched", "not_matched", "error") and performed correlation validation inline. `PlannerResponseRoute` had two variants: Matched(Value) and Terminal(ExecutionServiceResult).

## Relevant components

- `tethers-0.1/host-rust/src/engine_stdio.rs` — Stage 1 wire boundary
- `tethers-0.1/host-rust/src/host_execution.rs` — Stage 2 validated outcome boundary

## Required behaviour

1. Introduce PlannerResponseWire enum in engine_stdio.rs
2. Change evaluate_tether to return PlannerResponseWire
3. Add missing/non-string status proof at engine boundary
4. Introduce PlannerOutcome and PlannerErrorOutcome enums in host_execution.rs
5. classify_planner_response stages from wire to validated outcome
6. route_planner_outcome exhaustively routes all variants
7. Preserve exact correlation semantics
8. Preserve error validation order
9. Preserve unknown extra-field tolerance
10. Adapt all existing tests without weakening assertions

## Frozen decisions and invariants

- No strict Serde tagged decoder
- No deny_unknown_fields
- Raw matched Value deliberately retained
- No CorrelatedPlannerResponse wrapper
- No ExecutionServiceResult redesign
- No OCaml changes (F4a1 accepted)
- Missing/non-string status stays at engine boundary
- Unknown string status stays at host boundary

## Forbidden changes

- No production changes outside engine_stdio.rs and host_execution.rs
- No OCaml changes
- No Strict Serde decoding
- No fully typing Plan or Actions
- No beginning F4b or F5

## Stop conditions

STOP if:
- Missing-status behaviour conflicts with wire enum
- Exact correlation semantics cannot be retained
- New module becomes necessary
- Production changes spread beyond two files
- Two materially similar attempts fail

## Expected pre-existing changes

None

## Acceptance criteria

1. PlannerResponseWire enum with four variants (Matched, NotMatched, Error, Unknown)
2. PlannerOutcome enum with three variants (Matched, NotMatched, Error)
3. PlannerErrorOutcome enum with Contextual and Request variants
4. evaluate_tether returns PlannerResponseWire
5. classify_planner_response returns Result<PlannerOutcome, ExecutionServiceResult>
6. missing/non-string status -> EngineError::EvaluationFailed proved at engine boundary
7. unknown string status -> InvalidData proved at host boundary
8. Exact correlation semantics preserved for matched/not_matched/error
9. Extra-field tolerance maintained
10. All 1331 host tests pass; formatting clean in all F4a2 production files (accepted-base `replay_windows.rs` line 3280 has a proven pre-existing rustfmt discrepancy unrelated to F4a2)

## Required verification

```powershell
cargo test --locked --manifest-path tethers-0.1/host-rust/Cargo.toml
cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml -- --check
  # Accepted-base replay_windows.rs:3280 has a proven pre-existing rustfmt
  # discrepancy not caused by F4a2. The check may report it; that is expected.
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
git diff --check
git diff <accepted-base>..HEAD -- tethers-0.1/host-rust/src/replay_windows.rs
  # Must be empty — replay_windows.rs has zero net F4a2 diff
```
