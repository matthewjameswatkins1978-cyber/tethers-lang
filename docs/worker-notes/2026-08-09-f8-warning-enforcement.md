# Worker Note

Task: `F8-W2 — Enforce Zero Rust Compiler Warnings`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `26a93cb26f48a88018c6f621624fed78969f1db1`

Implementation checkpoint: `bec1024fe96cec194207c3d2cce5196ac34bed90`

## Requested outcome

Make future Rust compiler warnings fatal during `just verify` (and therefore
`just verify-agent`) by denying warnings in the all-target Cargo check, now
that the accepted F8-W1 tree is warning-free.

## Changes made

- `justfile` — added `$env:RUSTFLAGS="-D warnings";` prefix to the `check`
  recipe so that `cargo check --all-targets --all-features --locked` treats
  Rust compiler warnings as errors. The `verify` recipe composes `check`, so
  `just verify` and `just verify-agent` both inherit strict enforcement.

## Decisions and assumptions

- `RUSTFLAGS="-D warnings"` was chosen over a `[lints]` Cargo.toml setting
  because it is:
  1. scoped to the just invocation (session-local env var);
  2. does not affect `cargo clippy` runs outside the verification path;
  3. does not require any Cargo manifest changes.
- The existing `invoke-timed.ps1` timing wrapper is preserved unchanged.
- No new script or additional tooling file was needed.

## Evidence

- Pre-change `cargo check --all-targets --all-features --locked`: zero warnings.
- `$env:RUSTFLAGS="-D warnings"; cargo check --all-targets --all-features --locked`:
  PASS, zero warnings.
- `just check`: PASS (TIME cargo-check 0.3s PASS).
- Negative proof: added `use std::io;` to `installation_execution_tests.rs` →
  `just check` failed with `error: unused import: std::io` and exit code 1.
  File restored byte-for-byte; diff returned to justfile + task-packet only.
- `just verify-agent` (once, at implementation checkpoint `bec1024`):
  - TIME task-packet 0.9s PASS
  - TIME cargo-fmt 1.1s PASS
  - TIME cargo-check 9.7s PASS (with strict warning denial)
  - TIME cargo-test 49.4s PASS
  - TIME agent-tools 3.2s PASS
  - TIME deps-policy 0.6s PASS
  - TIME deps-advisories 1.3s PASS
  - TIME nextest 39.7s PASS (1592 passed, 2 skipped)
  - All 8 steps PASS.
- `git diff --check`: clean.
- Packet checker: PASS.
- Final diff: `justfile` (1 change) + closeout docs only.

## Discoveries

None.

## Remaining risks

None known within packet scope. No Clippy, dependency, CI, or Rust source
changes were made.

## Smallest next action

Lucy accepts this enforcement baseline. No further follow-up required unless
new warnings are introduced in future work (they would now fail `just verify`).

## References

- `docs/CURRENT_CLINE_TASK.md`
- `justfile`
- `docs/worker-notes/2026-08-09-f8-final-test-import-warnings.md`
