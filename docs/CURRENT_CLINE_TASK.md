# Current Implementation Task

Control contract: `1`

Task: `J13A local process supervision and check command - bounded correction`

Status: `COMPLETE`

Task colour: `Green`

Owner: `Goose`

Route: `Goose - J13A final probe-compatibility repair (narrow correction)`

Correction summary:
  - Restored absolute-path invariant in run_event_admission_trail_probe_clap.
  - Updated both event-admission probe scripts to accept JSON error envelopes.
  - Probes remain explicit hidden Clap commands (event-admission-probe,
    event-admission-trail-probe), not legacy positional commands.

Worker note: `docs/worker-notes/2026-07-28-j13a-process-check.md`

Base branch: `main`

Base commit: `f100689a35c9b7032193abd4f737c3203815fa4c`

Branch: `goose/j13a-process-check`

## Expected pre-existing changes

Starting from prior J13A implementation at `2c1ed6f99c180283456c1dfa4273500b4962e499`.
Previous commits on this branch: implementation at `cb3690d7`, documentation at `2c1ed6f9`.

## Relevant background and existing behaviour

J12 Packet 2 completed strict local runtime configuration parsing, validation,
and materialisation. The PreparedRuntime provides immutable runtime state with
verified manifests, provider launch plans, and capability-to-provider bindings.

J13A initial implementation added clap CLI parsing, Windows Job Object process
supervision, MCP engine session management, and the check command coordinator.

## Objective

Correct the prior J13A implementation with nine bounded corrections:
1. Real MCP protocol via tools/call with tethers.validate
2. Bounded protocol reads with timeout and interrupt checking
3. Correct Windows Job Object ownership (unnamed, fatal assignment failure)
4. Correct provider shutdown (Option-based, close takes child)
5. Envelope consistency with matching status/exit_code
6. No skip-as-pass tests (real engine execution required)
7. Comprehensive PowerShell acceptance script
8. All required gates (scripts, dune build)
9. Control-v1 documentation restoration

## Required behaviour

1. Engine uses tools/call with name "tethers.validate", arguments {"source": "..."}
2. Persistent BufReader with mpsc channel for timeout-aware reads
3. Unnamed Job Object (CreateJobObjectW(NULL, NULL)), assignment failure is fatal
4. ManagedProvider owns child in Option, close() takes it and calls shutdown
5. Every envelope's status and exit_code match process exit code
6. At least 40 genuinely executed j13a_ tests
7. Acceptance script tests real engine and provider fixture
8. All gates pass: dune build, all PowerShell scripts, clippy, fmt

## Relevant components

- `tethers-0.1/host-rust/src/child_process.rs` - Job Object, bounded reads, channels
- `tethers-0.1/host-rust/src/engine_stdio.rs` - tools/call protocol, real engine tests
- `tethers-0.1/host-rust/src/stdio_provider.rs` - Option-based child, graceful close
- `tethers-0.1/host-rust/src/check_command.rs` - envelope consistency, partial evidence
- `tethers-0.1/host-rust/src/cli.rs` - OutcomeStatus with matching exit codes
- `tethers-0.1/host-rust/src/main.rs` - thin entry boundary
- `tethers-0.1/host-rust/tests/j13a_cli.rs` - CLI integration tests
- `tethers-0.1/scripts/tethers-stdio-fixture.ps1` - extended test modes
- `tethers-0.1/scripts/test-j13a-check.ps1` - acceptance script

## Frozen decisions and invariants

- Engine validation uses tools/call, not a direct method
- parse result.structuredContent.valid for validation outcome
- Job Object is unnamed and assignment failure is fatal
- Provider close takes the child and calls shutdown; Drop is emergency fallback
- Envelope status and exit_code always match
- Interruption uses exit 10, status "interrupted"
- No tethers.evaluate during check
- No provider tools/call during check
- No Trail or replay creation during check

## Acceptance criteria

1. cargo fmt --check passes
2. cargo check and cargo check --tests pass
3. cargo test j12_ -- --nocapture: 99 pass
4. cargo test j13a_ -- --nocapture: 63+ pass, 0 fail
5. cargo test: all pass
6. cargo clippy --all-targets --all-features: zero errors
7. cargo build and cargo build --release succeed
8. All PowerShell scripts pass
9. opam exec -- dune build succeeds
10. Task packet checker passes

## Required verification

```powershell
cargo fmt --check
cargo check && cargo check --tests
cargo test j12_ -- --nocapture
cargo test j13a_ -- --nocapture
cargo test
cargo clippy --all-targets --all-features
cargo build && cargo build --release
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
pwsh -NoProfile -File tethers-0.1/scripts/check-fixtures.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-engine.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-mcp-transcripts.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-j13a-check.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-host-denial.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-host-execution-failure.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-host-result-follow-up.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-host-event-admission.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-host-event-admission-trail.ps1
pwsh -NoProfile -File tethers-0.1/scripts/demo.ps1
opam exec -- dune build
```

## Forbidden changes

Only 12 authorised files may change. No J13B, J13C, or J14 behaviour.
No provider tools/call. No tethers.evaluate. No Trail or replay creation.

## Stop conditions

- Task packet checker fails
- Any mandatory script produces unexpected results
- Git status not clean after expected changes
- Branch cannot be pushed or remote SHA does not match local
