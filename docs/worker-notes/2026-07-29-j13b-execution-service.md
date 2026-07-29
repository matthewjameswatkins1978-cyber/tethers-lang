# J13B Worker Note

Date: 2026-07-29
Packet: J13B Packet 1 — typed host execution service and retained execution
        sessions
Owner: Cline
Status: COMPLETE

## Changes

### New files
- `tethers-0.1/host-rust/src/executor.rs` — `CapabilityExecutor` trait extracted
  from `main.rs`
- `tethers-0.1/host-rust/src/host_execution.rs` — host execution service with:
  - `PreparedEvaluationInput`
  - `ExecutionServiceResult` (10 variants)
  - `RetainedProviderSession`
  - `ProviderSessionExecutor`
  - `HostExecutionService`
  - 14 focused `j13b_` tests

### Modified files
- `tethers-0.1/host-rust/src/main.rs`
  - Added `pub mod executor;` and `pub mod host_execution;`
  - Added `use crate::executor::CapabilityExecutor;`
  - Removed `CapabilityExecutor` trait definition (now in `executor.rs`)
  - Kept `MockExecutor`, `FailingExecutor`, and all test executors
- `tethers-0.1/host-rust/src/engine_stdio.rs`
  - Added `EvaluationFailed` error variant
  - Added `evaluate_tether()` method using `tools/call` with `tethers.evaluate`
  - Updated module comment to allow `tethers.evaluate`
- `tethers-0.1/host-rust/src/stdio_provider.rs`
  - Added public `tools_call()` method to `ManagedProvider`
- `docs/CURRENT_CLINE_TASK.md` — updated for J13B
- `docs/DECISIONS.md` — added J13B architecture decision

## Implementation choices

1. **CapabilityExecutor trait moved, not duplicated.** The trait now lives in
   `executor.rs` and is imported by `main.rs` (for test executors) and used by
   `host_execution.rs` (for `ProviderSessionExecutor`).

2. **EngineSession extended, not replaced.** `evaluate_tether` uses the same
   retained session, protocol reader, and timeout infrastructure. A Tethers
   response with `status: "error"` in `structuredContent` is treated as valid
   planner data.

3. **RetainedProviderSession wraps ManagedProvider.** Request IDs start at 3
   (after initialize=1 and tools/list=2) and increment monotonically. Each
   provider is launched, initialized and listed exactly once per service run.

4. **All existing gates preserved.** The service applies: capability resolution,
   policy evaluation (Deny/Ask/Unavailable/Allow), replay admission, durable
   intent recording, armed boundary, deadline check, provider invocation,
   outcome classification, durable outcome, and terminal state.

5. **Service returned in same order as inputs.** Results match input index.

## Evidence

- cargo fmt --check: PASS
- cargo check: PASS
- cargo check --tests: PASS
- cargo test j12_ -- --nocapture: 99 passed, 0 failed
- cargo test j13a_ -- --nocapture: 74 passed, 0 failed
- cargo test j13b_ -- --nocapture: 14 passed, 0 failed
- cargo test: 709 passed, 0 failed
- cargo clippy --all-targets --all-features: PASS (no errors)
- cargo build: PASS
- cargo build --release: PASS
- git diff --check: PASS
- check-tethers-task-packet.ps1: PASS
- check-fixtures.ps1: PASS
- test-mcp-transcripts.ps1: PASS (15 cases)
- test-j13a-check.ps1: PASS (25/25)
- test-engine.ps1: NOT RUN (opam switch not set in this worktree)
- opam exec -- dune build: NOT RUN (opam switch not set in this worktree)

## J13B test coverage

1. j13b_deny_produces_zero_provider_calls — Deny policy with valid bridge pins
2. j13b_ask_produces_zero_provider_calls — Ask requires availability
3. j13b_unavailable_produces_zero_provider_calls — Empty store → Unavailable
4. j13b_result_variants_constructable — All 10 result variants exist
5. j13b_evaluation_input_all_fields_explicit — Input fields are explicit
6. j13b_missing_bridge_pins_deny — Missing bridge pins fail closed
7. j13b_capability_executor_trait_works — Trait compiles and works
8. j13b_replay_state_variants_exist — Succeeded/Failed/Uncertain exist
9. j13b_logical_execution_key_deterministic — Same inputs = same key
10. j13b_prepare_error_deny_vs_ask — PrepareError variants distinct
11. j13b_prepare_error_variants_distinct — All three variants distinct
12. j13b_retained_session_monotonic_ids — Design verification
13. j13b_service_does_not_use_cli_envelope — No CliEnvelope in service
14. j13b_no_evaluation_id_derivation — evaluation_id must be explicit

## Remaining risks

- No real-provider integration test (fixture provider not available in this
  worktree).  The service architecture supports it but the test relies on
  deterministic unit boundaries.
- `test-engine.ps1` and `opam exec -- dune build` not run due to environment.
  These are OCaml engine operations not changed by this packet.
- The host execution service is not yet wired to a CLI command — that is J13B
  Packet 2.

## Forbidden items — confirmed absent

- No public `run` command
- No evaluation-ID derivation rule
- No J13C or J14 acceptance claim
- No `main` branch modification
- No new operating-system calls
- No Tethers language syntax or protocol change
