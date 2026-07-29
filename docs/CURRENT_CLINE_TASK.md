# Current Implementation Task

Control contract: `1`

Task: `J13B Packet 1 — typed host execution service and retained execution sessions`

Status: `COMPLETE`

Task colour: `Red`

Owner: `Codex`

Route: `Codex - J13B Packet 1 independent Red review correction`

Worker note: `docs/worker-notes/2026-07-29-j13b-execution-service.md`

Base branch: `main`

Base commit: `982039fd3673bb2a65fe8ed63180c3082af658b8`

Branch: `goose/j13b-execution-service`

## Objective

Extract the existing host execution machinery from `main.rs` into a typed Rust
application service that can later be called by a thin public `run` command.
Extend the retained OCaml engine and MCP provider sessions.

No public `run` command. No evaluation-ID derivation rule. No J13C or J14
acceptance claim.

## Relevant background and existing behaviour

J12 Packet 2 completed strict local runtime configuration parsing, validation,
and materialisation. The PreparedRuntime provides immutable runtime state with
verified manifests, provider launch plans, and capability-to-provider bindings.

J13A added clap CLI parsing, Windows Job Object process supervision, MCP engine
session management, and the check command coordinator.

## Required behaviour

1. Add a focused application-service module `host_execution.rs`.
2. Move `CapabilityExecutor` trait to shared `executor.rs` module.
3. Extend `EngineSession` with `evaluate_tether` using `tools/call`.
4. Add retained provider sessions with monotonically increasing request IDs.
5. Preserve all existing capability resolution, scope, policy, replay,
   durable-intent and dispatch boundaries.
6. Return typed results without constructing CLI envelopes.
7. No public `run` command. No evaluation-ID derivation rule.
8. All existing J12 and J13A tests continue to pass.

## Relevant components

- `tethers-0.1/host-rust/src/host_execution.rs` — new service module
- `tethers-0.1/host-rust/src/executor.rs` — CapabilityExecutor trait
- `tethers-0.1/host-rust/src/engine_stdio.rs` — EngineSession with evaluate_tether
- `tethers-0.1/host-rust/src/stdio_provider.rs` — ManagedProvider with tools_call
- `tethers-0.1/host-rust/src/main.rs` — updated imports, removed trait

## Frozen decisions and invariants

- Engine validation uses tools/call, not a direct method.
- Engine evaluation uses tools/call with tethers.evaluate.
- Tethers response with status "error" is valid planner data.
- Job Object is unnamed and assignment failure is fatal.
- Provider close takes the child and calls shutdown; Drop is emergency fallback.
- Envelope status and exit_code always match.
- No public `run` command in this packet.
- No evaluation-ID derivation rule.

## Acceptance criteria

1. cargo fmt --check passes
2. cargo check and cargo check --tests pass
3. cargo test j12_ -- --nocapture: 99 pass
4. cargo test j13a_ -- --nocapture: 74 pass, 0 fail
5. cargo test j13b_ -- --nocapture: 26 pass, 0 fail
6. cargo test: 708 pass, 0 fail
7. cargo clippy --all-targets --all-features: zero errors
8. cargo build and cargo build --release succeed
9. J13A public acceptance: 25 pass, 0 fail
10. Task packet checker passes

## Required verification

```powershell
cargo fmt --check
cargo check && cargo check --tests
cargo test j12_ -- --nocapture
cargo test j13a_ -- --nocapture
cargo test j13b_ -- --nocapture
cargo test
cargo clippy --all-targets --all-features
cargo build && cargo build --release
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
pwsh -NoProfile -File tethers-0.1/scripts/check-fixtures.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-engine.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-mcp-transcripts.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-j13a-check.ps1
opam exec -- dune build
git diff --check
```

## Forbidden changes

Do not:
- add the public `run` command;
- define the public event-and-facts JSON shape;
- derive or generate evaluation IDs;
- add the public `trail` command;
- alter Tethers 0.1 language syntax or protocol semantics;
- alter manifest format or replay identity;
- add GUI, TUI or Shell implementation;
- port to Linux or macOS;
- merge into or push directly to `main`;
- force-push.

## Stop conditions

Stop and report rather than guessing if:
- extraction requires changing the Tethers 0.1 request or response protocol;
- a public evaluation-ID derivation rule becomes necessary;
- a provider call cannot be routed through replay admission and durable intent;
- the existing J13A public acceptance changes;
- two substantially identical implementation attempts fail.

## Expected pre-existing changes

None. Starting from clean `main` at `982039fd3673bb2a65fe8ed63180c3082af658b8`.
