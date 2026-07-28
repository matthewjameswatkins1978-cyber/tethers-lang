# Current Implementation Task

Control contract: `1`

Task: `J13A local process supervision and check command - bounded correction`

Status: `COMPLETE`

Task colour: `Green`

Owner: `Codex`

Route: `Codex - J13A partial-evidence closure and public acceptance`

Correction summary:
  - Replaced provider-side envelope construction with one typed internal
    failure description.
  - Canonical check data now retains completed Tether and provider evidence on
    every post-preparation failure.
  - Completed an independent 25-case public-boundary acceptance against the
    real J12 config schema, OCaml MCP engine, reviewed fixture manifest, and
    PowerShell stdio provider.

Worker note: `docs/worker-notes/2026-07-28-j13a-process-check.md`

Base branch: `main`

Base commit: `f100689a35c9b7032193abd4f737c3203815fa4c`

Branch: `goose/j13a-process-check`

## Expected pre-existing changes

Starting from accepted J13A candidate
`2717c584c219bfebb33e776605fea4ef16c4fd5f`.
Partial-evidence implementation checkpoint:
`9de4a99444dab20e7b016cd339ced5cc0873197c`.

## Relevant background and existing behaviour

J12 Packet 2 completed strict local runtime configuration parsing, validation,
and materialisation. The PreparedRuntime provides immutable runtime state with
verified manifests, provider launch plans, and capability-to-provider bindings.

J13A initial implementation added clap CLI parsing, Windows Job Object process
supervision, MCP engine session management, and the check command coordinator.

## Objective

Close the confirmed partial-evidence blocker without altering accepted engine,
provider trust, process supervision, or successful-check semantics, then prove
the full J13A public boundary with real Windows processes.

## Required behaviour

1. Canonical data contains config identity/counts, ordered Tethers, and ordered
   providers.
2. Invalid Tether evidence includes the failing Tether and launches no provider.
3. Provider launch, initialize, tools/list, and capability failures retain all
   completed evidence.
4. Provider-check interruption retains completed evidence and returns
   interrupted/10.
5. Stable machine codes and field pointers remain unchanged.
6. Envelope and process exit codes derive from the same OutcomeStatus.
7. Acceptance uses a generated real J12-schema config named
   `valid primary config.json` under a temporary path containing spaces and
   Unicode.
8. Acceptance uses the real OCaml MCP engine, the reviewed
   `protocol/capability-manifests/fixture-ping.json` manifest
   (`sha256:01fed7a4b877dd82abe91a1b6cfcd476b02e4c115489e70cbb285b8bf2d32d8b`),
   and `pwsh.exe -NoProfile -File scripts/tethers-stdio-fixture.ps1`.

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
4. cargo test j13a_ -- --nocapture: 74 pass, 0 fail
5. cargo test: 695 pass, 0 fail
6. cargo clippy --all-targets --all-features: zero errors
7. cargo build and cargo build --release succeed
8. J13A public acceptance: 25 pass, 0 fail; every required PowerShell script passes
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

Only the four authorised Rust, acceptance-script, task, and worker-note files
may change. No J13B, J13C, or J14 behaviour.
No provider tools/call. No tethers.evaluate. No Trail or replay creation.

## Completion evidence

- Real engine:
  `tethers-0.1/engine-ocaml/_build/default/bin/tethers_mcp_main.exe`
- Engine marker: `initialize,tools/call:tethers.validate`
- Provider marker: `initialize,tools/list`
- Timeout evidence: engine initialize 12037 ms; provider initialize 12237 ms;
  provider tools/list 12562 ms
- Ctrl+C: interrupted/10 in 106 ms; direct PID 18536 gone; descendant PID
  25840 gone
- Trail/replay changes: 0
- Skipped required checks: 0
- Unrun required checks: 0

## Stop conditions

- Task packet checker fails
- Any mandatory script produces unexpected results
- Git status not clean after expected changes
- Branch cannot be pushed or remote SHA does not match local
