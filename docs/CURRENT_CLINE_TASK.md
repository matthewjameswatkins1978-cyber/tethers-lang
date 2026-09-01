# Tethers Agent Essentials — Phase C Coding Essentials

Control contract: `1`

Status: `COMPLETE`

Task colour: `Red`

Owner: `Lucy`

Route: `Codex direct implementation in dedicated review worktree; do not merge to main`

Base commit: `8d869c09a8c43b1d674c16b5274026f4b61d07df`

Implementation checkpoint: `5b2182848cdb2b6fcbe0eef48116e71f9ff7e714`

Worker note: `docs/worker-notes/2026-09-01-agent-essentials-phase-c.md`

Updated: 2026-09-01

## Objective

Extend the existing trusted Plug/provider seam with the Phase C coding
essentials: structured local Git operations, bounded argv-only process
execution, and named verification checks. Preserve Core semantics, capability
resolution, policy, scope, Trail, replay, Together concurrency, and the Phase B
workspace provider unchanged except for additive integration where required.

## Relevant background and existing behaviour

The repository already has host-owned Plug installation, conformance, trust,
operational scope delivery, supervised MCP execution, deterministic manifests,
the Phase B workspace provider, and the `tethers.cli/1` envelope. Phase C must
use those paths. Git and process operations are provider capabilities, not shell
shortcuts or hidden host commands.

## Required behaviour

1. Add structured Git capabilities for status, diff, log, show, branch listing,
   current branch, add, branch creation, checkout, and commit. No arbitrary Git
   passthrough, reset-hard, history deletion, force push, or remote mutation in
   this phase.
2. Add `process.execute` with an argv array, scoped executable/cwd/runtime/
   environment/output limits, timeout classification, truncation flags, and
   structured exit results. Shell interpolation is forbidden.
3. Add `verification.run` for explicitly configured named checks such as tests,
   lint, format, build, and typecheck; no arbitrary hidden command strings in
   capability input.
4. Add complete trusted manifests, exact MCP tool schemas, provider bindings,
   conservative effects/scopes/confirmation/retry metadata, and reproducible
   author package material for the Phase C provider.
5. Attack Git/process/verification boundaries with adversarial tests and prove
   the new package passes pack, inspect, and conformance without changing the
   frozen M4 or Phase B contracts.

## Relevant components

* `tethers-0.1/host-rust/src/agent_workspace.rs`
* `tethers-0.1/host-rust/src/bin/agent_workspace_provider.rs`
* `tethers-0.1/host-rust/src/file_tools.rs`
* `tethers-0.1/host-rust/src/host_execution.rs`
* `tethers-0.1/host-rust/src/manifest.rs`
* `tethers-0.1/host-rust/src/plug_pack.rs`
* `tethers-0.1/host-rust/src/conformance.rs`
* `reference-plugs/tethers-agent-workspace/`

## Frozen decisions and invariants

* Do not redesign the language, Core, planner, policy, Plug lifecycle, trust,
  scope evidence, Trail, replay, or Together concurrency.
* Extend the existing native provider seam; do not create a second registry,
  policy engine, scheduler, daemon, server, database, or agent framework.
* Git commands must be structured and allow-listed. Process execution must use
  argv without shell interpretation and must fail closed on scope violations.
* Verification names resolve only through host configuration; callers cannot
  supply arbitrary command text or bypass the configured check.
* No ambient credentials, hidden network access, remote Git mutation, force
  push, reset-hard, or destructive history operation is added.
* Phase B package identities, schemas, behavior, and compatibility fixtures
  remain intact.

## Acceptance criteria

1. Structured Git read and approved local mutation operations execute only
   through the existing trusted provider path and return typed results.
2. Process execution enforces argv, executable, cwd, timeout, environment, and
   output bounds and exposes timeout/truncation classification.
3. Named verification checks reject unknown or caller-supplied commands and
   return standardized check results.
4. Phase C manifests and package material pass exact discovery/schema
   conformance, adversarial tests, and deterministic packaging checks.
5. Existing M4 and Phase B tests pass with no frozen artifact or contract
   changes.

## Required verification

* `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml -- --check`
* `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked`
* focused Phase C unit and integration tests
* existing `m4_file_tools` tests
* Phase B workspace tests and provider smoke
* package pack, inspect, and supervised conformance
* `git diff --check`
* task-packet checker

## Forbidden changes

* No network, archive, SQLite, system inspection, plan, Trail query, or Linux
  package implementation in this packet.
* No arbitrary Git command or shell escape hatch.
* No direct host shortcut around Plug trust, policy, scope, or provider binding.
* No weakening or rewriting of frozen fixtures, M4 package identities, or Phase
  B manifests.
* No merge to `main`.

## Stop conditions

Stop and report if structured output cannot be kept stable, if a requested
operation would require hidden authority, if existing M4/Phase B behavior must
change, if configuration cannot bind a named verification check precisely, or
if the current package format cannot carry the provider without an unrelated
format redesign.

## Expected pre-existing changes

The baseline host suite had known missing-engine and concurrency stress failures
before Phase B. Preserve those results and distinguish them from Phase C
regressions. Build output under ignored `target/` and generated local package
artifacts under ignored `dist/` are not source changes.
