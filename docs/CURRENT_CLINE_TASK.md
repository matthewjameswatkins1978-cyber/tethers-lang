# Current Implementation Task

Control contract: `1`

Task: `J13B Packet 1 — typed host execution service and retained execution sessions`

Status: `COMPLETE`

Task colour: `Green`

Owner: `Cline`

Route: `Cline - J13B Packet 1 — typed host execution service`

## Objective

Extract the existing host execution machinery from `main.rs` into a typed Rust
application service that can later be called by a thin public `run` command.
Extend the retained OCaml engine and MCP provider sessions.

No public `run` command. No evaluation-ID derivation rule. No J13C or J14
acceptance claim.

## Worker note

`docs/worker-notes/2026-07-29-j13b-execution-service.md`

## Implementation summary

- Added `pub mod executor;` and `pub mod host_execution;` to `main.rs`
- Moved `CapabilityExecutor` trait from `main.rs` to new `executor.rs` module
- Extended `EngineSession` with `evaluate_tether()` method using `tools/call`
  with `tethers.evaluate`
- Added public `tools_call()` method to `ManagedProvider`
- Created `host_execution.rs` with:
  - `PreparedEvaluationInput` — typed evaluation input with explicit fields
  - `ExecutionServiceResult` — typed result enum (10 variants)
  - `RetainedProviderSession` — retained MCP provider session wrapper
  - `ProviderSessionExecutor` — CapabilityExecutor using retained sessions
  - `HostExecutionService` — main service orchestrating validation, evaluation,
    and dispatch through all existing gates
  - 14 focused `j13b_` tests

## Base

Base branch: `main`
Base commit: `982039fd3673bb2a65fe8ed63180c3082af658b8`
Branch: `goose/j13b-execution-service`

## Expected pre-existing changes

None. Starting from clean `main` at `982039fd3673bb2a65fe8ed63180c3082af658b8`.

## Verification

```powershell
cargo fmt --check         # PASS
cargo check               # PASS
cargo check --tests       # PASS
cargo test j12_ -- --nocapture  # 99 passed, 0 failed
cargo test j13a_ -- --nocapture # 74 passed, 0 failed
cargo test j13b_ -- --nocapture # 14 passed, 0 failed
cargo test                      # 709 passed, 0 failed
cargo clippy --all-targets --all-features  # PASS (no errors)
cargo build                       # PASS
cargo build --release             # PASS
git diff --check                  # PASS
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1  # PASS
pwsh -NoProfile -File tethers-0.1/scripts/check-fixtures.ps1         # PASS
pwsh -NoProfile -File tethers-0.1/scripts/test-engine.ps1            # NOT RUN (opam switch env)
pwsh -NoProfile -File tethers-0.1/scripts/test-mcp-transcripts.ps1   # PASS (15 cases)
pwsh -NoProfile -File tethers-0.1/scripts/test-j13a-check.ps1        # PASS (25/25)
opam exec -- dune build  # NOT RUN (opam switch env)
```

## Confirmation

- No public `run` command was added
- No evaluation-ID rule was invented
- `main` was not changed (all work on `goose/j13b-execution-service` branch)
- J13A public acceptance unchanged (25/25)
