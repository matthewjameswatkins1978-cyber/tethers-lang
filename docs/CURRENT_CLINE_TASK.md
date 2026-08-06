# Current Implementation Task

Control contract: `1`
Task: `J24L2 - Thin public plug install CLI`
Owner: `DeepSeek Pro`
Model: `DeepSeek Pro`
Status: `COMPLETE`
Task colour: `Amber`
Route: `OpenCode using DeepSeek Pro for one bounded Rust CLI and context-assembly package; Lucy performs independent review and any later merge`
Base branch: `main`
Base commit: `190e834b8afeca060adb3b07c7a18554497aaf31`
Implementation branch: `opencode/j24l2-thin-plug-install-cli`
Parent branch: `opencode/j24l1-bounded-installation-driver`
Parent tip: `f5fecb02276b1aa0937126d730c367c9333ac203`
Worker note: `docs/worker-notes/2026-08-06-j24l2-thin-plug-install-cli.md`
Rust toolchain: `1.97.1`

## Objective

Complete J24L by wiring the accepted J24L1 bounded driver into one thin public
`plug install` CLI command with canonical host-data layout, request-file loading,
and JSON output mapping.

## Relevant background and existing behaviour

J24L1 is accepted. The crate-private `installation_driver.rs` provides
`drive_installation` with a four-call maximum. J24K's public API is
`execute_next_installation_action(request, context, options) -> Result<InstallationStepResult>`.
J24G defines the installation request loading and validation. Existing `plug stage`
creates candidates and quarantine roots that `plug install` must reuse.

## Required behaviour

1. Add `Install` variant to `PlugCommand` in `cli.rs` with frozen fields.
2. Create `plug_install_command.rs` with `run_install`, path assembly, validation
   order, frozen options, driver call, and result mapping.
3. Route `Install` through `application.rs`.
4. Register the new module in `lib.rs`.
5. Preserve all J24L1, J24K, and J24J behaviour unchanged.
6. Implement comprehensive unit, mapping, and Windows E2E tests.

## Frozen decisions and invariants

- Maximum four J24K calls from the J24L1 driver.
- Frozen CLI shape: `plug install --host-data-root <ABSOLUTE_PATH> --request <ABSOLUTE_JSON_PATH>`.
- Frozen options: `tethers-reference-host-cli` authority, 30-second wall time, compile-time build identity.
- Candidate must already be staged by `plug stage`.
- No package, candidate, retry, or recovery arguments.
- `Invalidated` and `Passed` in non-advancing conformance are contradictory at the J24L boundary.
- Error status mapping follows the explicit table.

## Relevant components

- `tethers-0.1/host-rust/src/plug_install_command.rs` (new)
- `tethers-0.1/host-rust/src/cli.rs` (modified)
- `tethers-0.1/host-rust/src/application.rs` (modified)
- `tethers-0.1/host-rust/src/lib.rs` (modified)
- `tethers-0.1/host-rust/tests/j24l2_plug_install_cli.rs` (new)
- `docs/architecture/J24L_THIN_PUBLIC_PLUG_INSTALL_CLI.md` (modified)
- `docs/CURRENT_CLINE_TASK.md` (replacement)
- `docs/worker-notes/2026-08-06-j24l2-thin-plug-install-cli.md` (new)

## Acceptance criteria

1. Clap: exact valid parse, reordered options, all missing/duplicate/unknown
   rejections, no package/candidate option. Evidence: Clap tests pass.
2. Pre-mutation validation: relative/missing paths, malformed requests create no
   lifecycle state. Evidence: integration tests pass.
3. Pure mapping: completed, already-complete, failed/interrupted/contradictory
   conformance, missing installed pins, error status table, unlisted defaults.
   Evidence: unit/mapping tests pass.
4. Windows E2E: fresh install completes in 4 steps, disabled record + destination
   present, no intent, scratch clean, enablements/ empty, plug list reports one
   disabled Plug, second invocation succeeds as already complete with one step,
   no conformance retry or second publication. Evidence: `#[cfg(windows)]` tests pass.
5. Architecture document updated with J24L2 sections and "crate-private" fix.
6. All J24L1, J24K3f, J24K2, J24J regressions pass.
7. No dead-code warnings for driver types or new J24L2 code.
8. Formatting and Clippy clean.
9. Packet checker passes.

## Required verification

```
cargo test --lib -p tethers-reference-host j24l1_ --no-fail-fast --locked
cargo test --lib -p tethers-reference-host j24l2_ --no-fail-fast --locked
cargo test --test j24l2_plug_install_cli --no-fail-fast --locked
cargo test --lib -p tethers-reference-host j24k3f --no-fail-fast --locked
cargo test --lib -p tethers-reference-host j24k2 --no-fail-fast --locked
cargo test --test j24j_installation_reconciliation --locked
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked
$env:RUST_TEST_THREADS="1"; just verify; Remove-Item Env:RUST_TEST_THREADS
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
git diff --check
git status --short
```

## Forbidden changes

Do not change J24J, J24K, J24L1 semantics, add a fifth executor call, retry
conformance, stage a package during install, enable the installed Plug, add
recovery execution to CLI, or change Cargo.toml/Cargo.lock.

## Stop conditions

Stop on base/main mismatch, dirty/unexplained tree, contradictory
requirements, or two materially similar failed attempts.

## Expected pre-existing changes

None.

## Checkpoint procedure

1. Require READY packet checker pass.
2. Change to IN_PROGRESS.
3. Implement production code and tests.
4. Commit implementation.
5. Run all verification.
6. Complete worker note, mark COMPLETE.
7. Commit verification docs, push, report SHAs. Do not merge.
