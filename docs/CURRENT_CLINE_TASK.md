# Current Implementation Task

Control contract: `1`
Task packet: `F8-W2 — Enforce Zero Rust Compiler Warnings`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Green`
Route: `OpenCode adds warning denial to the existing just verify path`
Worker note: `docs/worker-notes/2026-08-09-f8-warning-enforcement.md`
Base branch: `foundation/f8-final-test-import-warnings`
Base commit: `26a93cb26f48a88018c6f621624fed78969f1db1`
Implementation branch: `foundation/f8-warning-enforcement`
Implementation checkpoint: `bec1024fe96cec194207c3d2cce5196ac34bed90`
OCaml switch path: `N/A`
Rust toolchain: `1.97.1`
Rust change class: `TOOLING`

## Objective

Make future Rust compiler warnings fail the repository's normal verification
path now that the accepted all-target Cargo check is warning-free.

## Relevant background and existing behaviour

The accepted F8-W1 checkpoint removed the two residual test-module unused-import
warnings. The locked all-target `cargo check --all-targets --all-features --locked`
is now warning-free.

The existing `just verify` recipe runs `cargo check` (without warning denial),
`cargo fmt --check`, `cargo test`, and the packet checker via
`scripts/invoke-timed.ps1`.

`just verify-agent` extends `verify` with agent tools, dependency policy,
dependency advisories, and Nextest.

## Required behaviour

1. Add `RUSTFLAGS="-D warnings"` to the `check` recipe in `justfile` so
   Rust compiler warnings are fatal during `cargo check`.
2. `just verify` and `just verify-agent` inherit this enforcement.
3. No Clippy lint-policy changes.
4. No Rust source changes.
5. No GitHub Actions or CI introduction.

## Relevant components

### AUTHORISED PATHS
- `justfile`

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-f8-warning-enforcement.md`

## Frozen decisions and invariants

- No Clippy cleanup or denial.
- No global `[lints]` in Cargo.toml.
- No Rust source changes.
- No dependency, toolchain, or CI changes.
- No warning suppression.
- No weakened existing verification.

## Acceptance criteria

1. `RUSTFLAGS="-D warnings" cargo check` passes on clean tree.
2. Deliberate temporary warning proves the gate fails.
3. `just verify` uses the strict check.
4. `just verify-agent` passes once at implementation checkpoint.
5. Existing Clippy policy unchanged.
6. Final diff contains only authorised tooling/docs.
7. Branch pushed normally with clean status.

## Required verification

1. `git diff --check`.
2. Packet checker.
3. Negative proof (temporary warning → gate fails → restore → clean).
4. `just verify-agent` once.

## Forbidden changes

- No Clippy cleanup or denial.
- No global lint-policy redesign.
- No Rust source change in final tree.
- No OCaml change.
- No GitHub Actions.
- No new dependency.
- No warning suppression.
- No unrelated justfile cleanup.

## Stop conditions

STOP if strict check does not pass on clean tree, if negative proof fails,
or if `just verify-agent` fails.

## Expected pre-existing changes

None.
