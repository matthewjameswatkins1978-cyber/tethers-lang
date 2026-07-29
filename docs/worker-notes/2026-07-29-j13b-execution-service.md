# J13B Worker Note

Task: `J13B Packet 1 — typed host execution service and retained execution sessions`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Cline`

Status: `COMPLETE`

Base commit: `982039fd3673bb2a65fe8ed63180c3082af658b8`

Implementation checkpoint: `c19c729e06cc5ae63a018c287aa5f0f7eb917866`

## Requested outcome

Extract host execution machinery from main.rs into a typed Rust application
service. Extend retained OCaml engine and MCP provider sessions. No public run
command. No evaluation-ID derivation rule.

## Changes made

### New files
- `tethers-0.1/host-rust/src/executor.rs` — CapabilityExecutor trait
- `tethers-0.1/host-rust/src/host_execution.rs` — host execution service with
  PreparedEvaluationInput, ExecutionServiceResult (10 variants),
  RetainedProviderSession, ProviderSessionExecutor, HostExecutionService,
  and 14 focused j13b_ tests

### Modified files
- `tethers-0.1/host-rust/src/main.rs` — added executor and host_execution
  modules, moved CapabilityExecutor trait
- `tethers-0.1/host-rust/src/engine_stdio.rs` — added evaluate_tether method
- `tethers-0.1/host-rust/src/stdio_provider.rs` — added public tools_call method
- `docs/CURRENT_CLINE_TASK.md` — updated for J13B
- `docs/DECISIONS.md` — added J13B architecture decision

## Decisions and assumptions

1. CapabilityExecutor trait moved to executor.rs, not duplicated.
2. EngineSession extended with evaluate_tether using retained session.
3. RetainedProviderSession wraps ManagedProvider with request IDs starting at 3.
4. All existing gates preserved: capability resolution, policy, replay, intent.
5. No public run command or evaluation-ID rule in this packet.

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
- check-tethers-task-packet.ps1: PASS (after format fix)
- check-fixtures.ps1: PASS
- test-mcp-transcripts.ps1: PASS (15 cases)
- test-j13a-check.ps1: PASS (25/25)
- test-engine.ps1: NOT RUN (opam switch not set)
- opam exec -- dune build: NOT RUN (opam switch not set)

## Discoveries

The evaluate_effective_policy function requires bridge pins (manifest_digest,
bridge_capability_version, bridge_provider_identity) in the ProposedAction.
Tests without these produce Deny via MissingBridgePin rather than the expected
policy decision.

## Remaining risks

- No real-provider integration test in this worktree.
- test-engine.ps1 and dune build not run due to environment.
- Service not yet wired to a CLI command (that is J13B Packet 2).

## Smallest next action

J13B Packet 2: add public run command with event-and-facts input and
evaluation-ID assignment, wiring the HostExecutionService to the CLI.

## References

- Branch: goose/j13b-execution-service
- Commit: c19c729e06cc5ae63a018c287aa5f0f7eb917866
