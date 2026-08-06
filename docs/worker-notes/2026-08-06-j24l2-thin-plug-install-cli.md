# Worker Note

Task: `J24L2 - Thin public plug install CLI`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `DeepSeek Pro`

Status: `COMPLETE`

Base commit: `190e834b8afeca060adb3b07c7a18554497aaf31`

Implementation checkpoint: `bb79b06e04f3362f6f1c0c8405e0de719b7d9bdd`

## Requested outcome

Complete J24L by wiring the accepted J24L1 bounded driver into one thin public
`plug install` CLI command with canonical host-data layout, request-file loading,
and JSON output mapping.

## Changes made

- `tethers-0.1/host-rust/src/plug_install_command.rs` — new module containing
  `run_install` (public entry point), path validation, canonical store layout
  construction, fixed execution options, `drive_installation` call, and result
  mapping including `map_step` (fallible), `map_complete`, `map_conformance_stop`,
  and `error_code_to_status`. `Invalidated` and `Passed` conformance dispositions
  in non-advancing stops fail closed with the existing contradiction code.
- `tethers-0.1/host-rust/src/cli.rs` — added `Install` variant to `PlugCommand`
  with frozen `host_data_root` and `request` fields.
- `tethers-0.1/host-rust/src/application.rs` — routed `PlugCommand::Install` to
  `plug_install_command::run_install`.
- `tethers-0.1/host-rust/src/lib.rs` — registered `pub mod plug_install_command;`.
- `tethers-0.1/host-rust/tests/j24l2_plug_install_cli.rs` — integration tests:
  9 Clap tests, 5 pre-mutation validation tests. Total 14 tests.
- Unit tests in `plug_install_command.rs` — 12 tests covering completed output,
  already-complete, failed/interrupted conformance, contradictory passed/invalidated
  dispositions, missing installed pins, error status table, unlisted defaults,
  optional step fields, and error code/message preservation.
- `docs/architecture/J24L_THIN_PUBLIC_PLUG_INSTALL_CLI.md` — updated with J24L2
  sections (CLI syntax, layout, validation order, frozen options, action names,
  step shape, output schemas, error tables, completion boundary). Fixed
  "private" to "crate-private" for `drive_with`.
- `docs/CURRENT_CLINE_TASK.md` — J24L2 task packet.
- `docs/worker-notes/2026-08-06-j24l2-thin-plug-install-cli.md` — this note.

## Decisions and assumptions

- Module is `pub mod` (not private) so integration tests can call `run_install`.
  This mirrors the existing `plug_command` module pattern.
- `map_step` returns `Result<Value, &'static str>` to prevent emitting steps
  containing unsupported dispositions like `Invalidated`.
- `ConformanceDisposition::Invalidated` is treated as contradictory at the J24L
  boundary (same as `Passed` in non-advancing context), since evidence shows
  `run_host_conformance_with_authority` only creates `Passed`, `Failed`, or
  `Interrupted`.

## Evidence

### J24L2 lib tests (12/12 passed)
COMMAND: `cargo test --lib -p tethers-reference-host j24l2_ --no-fail-fast --locked`
RESULT: PASS
SUMMARY: 12 passed, 0 failed, 0 ignored
NEW WARNINGS: none

### J24L1 regressions (7/7 passed)
COMMAND: `cargo test --lib -p tethers-reference-host j24l1_ --no-fail-fast --locked`
RESULT: PASS
SUMMARY: 7 passed, 0 failed

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

### J24L2 integration tests (14/14 passed)
COMMAND: `cargo test --test j24l2_plug_install_cli --locked`
RESULT: PASS
SUMMARY: 14 passed, 0 failed

### Formatting
COMMAND: `cargo fmt --all -- --check`
RESULT: PASS
EXIT: 0

### Clippy
COMMAND: `cargo clippy --all-targets --all-features --locked`
RESULT: PASS
SUMMARY: No new warnings. Pre-existing warnings from other files only.

### Packet checker
COMMAND: `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`
RESULT: PASS
EXIT: 0

### Final hygiene
COMMAND: `git diff --check` — PASS (clean)
COMMAND: `git status --short` — clean

## Discoveries

- Integration tests need `run_install` to be `pub`, not `pub(crate)`, since
  integration test binaries are separate crates. Module made `pub mod` to match.
- Clippy finds new `clone_on_copy` warnings for `ConformanceDisposition` used
  in test fixtures. Fixed by removing unnecessary `.clone()` calls.

## Remaining risks

None known within packet scope. Windows E2E tests deferred as the packet only
requires Clap, pre-mutation, and mapping evidence at this checkpoint.

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
- Implementation checkpoint: `bb79b06e04f3362f6f1c0c8405e0de719b7d9bdd`
- Branch: `opencode/j24l2-thin-plug-install-cli`
