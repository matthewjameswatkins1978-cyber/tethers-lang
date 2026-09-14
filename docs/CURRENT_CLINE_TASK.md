# Tethers 0.8-B — Verification and Release Control

Task: `Tethers 0.8-B / R2`

Control contract: `1`

Status: `COMPLETE`

Task colour: `Green`

Owner: `Codex`

Route: `Make repository-owned verification authoritative, reproduce the stale-engine failure, establish current engine provenance, add the compatibility seed harness, and publish only after full evidence review.`

Base commit: `9b8b1fe86551705626785e6ed76a784f91fe6515`

Implementation checkpoint: `668648cdb3a248b803ac1cc7f04b65028de52fd8`

Worker note: `docs/worker-notes/2026-09-14-tethers-0-8-b.md`

Suggested branch:

`codex/tethers-0.8-verification-release-control`

## Objective

Make Tethers verification reproducible and honest about source identity,
toolchain prerequisites, current first-party engine provenance, required and
optional suites, compatibility evidence, and release eligibility.

## Relevant background and existing behaviour

The accepted 0.8-A baseline is `9b8b1fe86551705626785e6ed76a784f91fe6515`.
The prior Rust cross-language failures were caused by a stale or unavailable
OCaml engine environment and must not be treated as product evidence.

## Required behaviour

1. Repair the historical packet checker without invalidating accepted historical evidence.
2. Build or reject the current OCaml engine before dependent cross-language tests.
3. Make verification outcomes, compatibility seeds, warning policy, and release-control evidence explicit.

## Relevant components

- `.github/scripts/`
- `scripts/`
- `justfile`
- `tethers-0.1/host-rust/src/` test-only engine discovery helpers
- `compat/0.7/`
- `docs/VERIFICATION.md`
- `.github/workflows/`

## Frozen decisions and invariants

- No Human Tether syntax, Core semantics, Plan meaning, Trail meaning, policy, Plug, or MCP semantic changes.
- Rust remains the host and OCaml remains the semantic Core.
- No stale or arbitrary engine binary may satisfy current cross-language verification.
- No license decision is made in R2; Apache-2.0 remains an owner decision.
- Linux implementation and packaging remain R3/R5 scope.

## Acceptance criteria

1. Current and historical packet evidence are distinguished correctly.
2. Toolchain and engine prerequisites are explicit and attributable to current source.
3. `just verify` is the authoritative aggregate route and produces machine-readable evidence.
4. Required verification, compatibility seeds, CI foundations, and documentation are reviewable.
5. The complete branch is tested, reviewed, pushed, and merged before R3 begins.

## Required verification

Run the current tool diagnostic, packet tests, toolchain check, OCaml build/tests,
Rust format/check/test, cross-language tests, MCP transcripts, fixture checks,
compatibility harness, warning ratchet, documentation/link/security checks,
`git diff --check`, and the final clean-tree/publication checks.

## Forbidden changes

- No product feature work or semantic redesign.
- No Linux native parity, MCP protocol upgrade, release packaging, or license change.
- No weakening tests, force-push, history rewrite, or unrelated cleanup.

## Stop conditions

Stop before merge for a semantic regression, unverifiable current engine,
unexplained unrelated change, failed required check, suspicious PR diff,
merge conflict, or repository policy requiring human-only approval. After two
materially similar failed repair attempts, report the exact first error and stop
that approach.

## Expected pre-existing changes

None.

## Implementation scope

- `.github/scripts/check-tethers-task-packet.ps1`
- `.github/scripts/check-tethers-toolchains.ps1`
- `.github/scripts/test-check-tethers-task-packet.ps1`
- `scripts/prepare-current-engine.ps1`
- `scripts/run-rust-tests.ps1`
- `scripts/verify-tethers.ps1`
- `scripts/check-compatibility-corpus.ps1`
- `scripts/check-warning-ratchet.ps1`
- `justfile`
- `tethers-0.1/host-rust/src/engine_stdio.rs`
- `tethers-0.1/host-rust/src/host_execution.rs`
- `compat/0.7/`
- `docs/VERIFICATION.md`
- `docs/ROAD_TO_1_0.md`
- `docs/DECISIONS.md`
- `.github/workflows/tethers-verification.yml`
