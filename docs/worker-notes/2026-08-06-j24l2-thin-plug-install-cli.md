# Worker Note

Task: `J24L2 - Thin public plug install CLI`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `DeepSeek Pro`

Status: `COMPLETE`

Base commit: `190e834b8afeca060adb3b07c7a18554497aaf31`

Implementation checkpoint: `f0f0c84ad0003d0af23e65dfb54f612175156177`

## Requested outcome

Complete J24L by wiring the accepted J24L1 bounded driver into one thin public
`plug install` CLI command with canonical host-data layout, request-file loading,
and JSON output mapping.

## Changes made

### Production code

- `tethers-0.1/host-rust/src/plug_install_command.rs` — new module containing
  `pub(crate) run_install`, path validation, canonical store layout construction,
  fixed execution options, `drive_installation` call, and result mapping including
  `map_step` (fallible), `map_complete`, `map_conformance_stop` (exhaustive
  fail-closed match, no `unreachable!()`), `contradict_non_advancing`, and
  `error_code_to_status`. `Invalidated` and `Passed` conformance dispositions
  in non-advancing stops fail closed with the existing contradiction code.
- `tethers-0.1/host-rust/src/cli.rs` — added `Install` variant to `PlugCommand`
  with frozen `host_data_root` and `request` fields.
- `tethers-0.1/host-rust/src/application.rs` — routed `PlugCommand::Install` to
  `plug_install_command::run_install`.
- `tethers-0.1/host-rust/src/lib.rs` — registered `mod plug_install_command;`
  (private, not `pub mod`).

### Tests

- Unit tests in `plug_install_command.rs` `#[cfg(test)]` — 17 tests:
  - 12 mapping tests covering completed output, already-complete, failed/
    interrupted conformance, contradictory passed/invalidated dispositions,
    missing installed pins, error status table, unlisted defaults, optional
    step fields, and error code/message preservation.
  - 5 pre-mutation validation tests (moved from integration test file):
    relative/missing paths, malformed requests create no lifecycle state.
- `tethers-0.1/host-rust/tests/j24l2_plug_install_cli.rs` — 10 integration tests:
  - 9 Clap parsing tests (valid, reordered, missing, duplicate, unknown,
    no package/candidate, equal sign).
  - 1 `#[cfg(windows)]` E2E test: stages package via binary, installs, verifies
    4 steps in exact order, proves disabled record + destination, no pending
    intent, clean conformance scratch, empty enablements, plug list shows one
    disabled Plug, snapshots state, re-installs with 1 AlreadyComplete step,
    proves record/destination unchanged, no conformance retry or second
    publication.

### Documentation

- `docs/architecture/J24L_THIN_PUBLIC_PLUG_INSTALL_CLI.md` — updated with J24L2
  sections (CLI syntax, layout, validation order, frozen options, action names,
  step shape, output schemas, error tables, completion boundary). Fixed
  "private" to "crate-private" for `drive_with`.
- `docs/CURRENT_CLINE_TASK.md` — J24L2 task packet with explicit module privacy
  and exhaustive match invariants.
- `docs/worker-notes/2026-08-06-j24l2-thin-plug-install-cli.md` — this note.

## Decisions and assumptions

- Module is `mod` (private) with `pub(crate) fn run_install`. Integration tests
  invoke the real binary via `CARGO_BIN_EXE_tethers-reference-host` rather than
  calling `run_install` through a widened Rust API.
- Pre-mutation validation tests moved into `#[cfg(test)]` module where
  crate-private access is available.
- `map_conformance_stop` uses exhaustive fail-closed match (all four
  `ConformanceDisposition` variants handled explicitly, no `unreachable!()`).
- E2E test uses the real `pdf_tools_provider` binary for conformance execution.
- Integration test uses `host_binary()` pattern consistent with j24f, j24a, etc.

## Evidence

### J24L2 lib tests (17/17 passed)
COMMAND: `cargo test --lib -p tethers-reference-host j24l2_ --no-fail-fast --locked`
RESULT: PASS
SUMMARY: 17 passed, 0 failed, 0 ignored
NEW WARNINGS: none

### J24L1 regressions (7/7 passed)
COMMAND: `cargo test --lib -p tethers-reference-host j24l1_ --no-fail-fast --locked`
RESULT: PASS
SUMMARY: 7 passed, 0 failed

### J24L2 integration tests (10/10 passed)
COMMAND: `cargo test --test j24l2_plug_install_cli --no-fail-fast --locked`
RESULT: PASS
SUMMARY: 10 passed, 0 failed (including 1 #[cfg(windows)] E2E)

### All plug tests (32/32 passed)
COMMAND: `cargo test -p tethers-reference-host plug_ --no-fail-fast --locked`
RESULT: PASS
SUMMARY: 32 passed, 0 failed

### J24K3f regressions (10/10 passed)
COMMAND: `cargo test --lib -p tethers-reference-host j24k3f --no-fail-fast --locked`
RESULT: PASS
SUMMARY: 10 passed, 0 failed

### J24K2 regressions (26/26 passed)
COMMAND: `cargo test --lib -p tethers-reference-host j24k2 --no-fail-fast --locked`
RESULT: PASS
SUMMARY: 26 passed, 0 failed

### J24J planner regressions (24/24 passed)
COMMAND: `cargo test --test j24j_installation_reconciliation --locked`
RESULT: PASS
SUMMARY: 24 passed, 0 failed

### Formatting
COMMAND: `cargo fmt --all -- --check`
RESULT: PASS
EXIT: 0

### Clippy
COMMAND: `cargo clippy --all-targets --all-features --locked`
RESULT: PASS (no new J24L2 warnings; pre-existing warnings from other files only)

### Release build
COMMAND: `cargo build --release --locked`
RESULT: PASS

### Full serial verification (just verify)
COMMAND: `$env:RUST_TEST_THREADS="1"; just verify`
RESULT: PASS
SUMMARY: 1254 passed, 0 failed, 2 ignored

### Packet checker
COMMAND: `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`
RESULT: PASS

### Cargo.lock unchanged
COMMAND: `git diff --exit-code -- tethers-0.1/host-rust/Cargo.lock`
RESULT: PASS (no changes)

### Final hygiene
COMMAND: `git diff --check` — PASS (clean)
COMMAND: `git status --short` — clean after commit

## Discoveries

- `installation-intent/` directory persists after successful publication
  (created by `StoreRoot::open`). The presence of `current.json` indicates a
  pending intent; its absence indicates the intent was consumed. E2E test
  asserts `current.json` does not exist after install.
- Integration tests that exercise conformance need a real executable provider
  binary (not just arbitrary bytes).
- Clippy `needless_borrow` caught one instance in the moved pre-mutation test
  code. Fixed in implementation checkpoint.

## Remaining risks

None known within J24L2 scope. All 8 corrections applied and verified.

## Smallest next action

This is the final J24L package. Lucy reviews and accepts. Matthew may merge
the J24L branch into main.

## References

- `docs/architecture/J24L_THIN_PUBLIC_PLUG_INSTALL_CLI.md`
- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `docs/architecture/J24G_INSTALLATION_REQUEST_CONTRACT.md`
- `tethers-0.1/host-rust/src/installation_driver.rs`
- `tethers-0.1/host-rust/src/installation_execution.rs`
- `tethers-0.1/host-rust/src/plug_install_command.rs`
- Implementation checkpoint: `f0f0c84ad0003d0af23e65dfb54f612175156177`
- Branch: `opencode/j24l2-thin-plug-install-cli`
