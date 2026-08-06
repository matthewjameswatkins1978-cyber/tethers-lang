# Worker Note

Task: `J24L2 - Thin public plug install CLI`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `DeepSeek Pro`

Status: `COMPLETE`

Base commit: `190e834b8afeca060adb3b07c7a18554497aaf31`

Implementation checkpoint: `d9cfeb21c08a29d0e115b9daa243f665ee84a4b8`

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
  `error_code_to_status`.
- `tethers-0.1/host-rust/src/cli.rs` — added `Install` variant to `PlugCommand`
  with frozen `host_data_root` and `request` fields.
- `tethers-0.1/host-rust/src/application.rs` — routed `PlugCommand::Install` to
  `plug_install_command::run_install`.
- `tethers-0.1/host-rust/src/lib.rs` — registered `mod plug_install_command;`
  (private, not `pub mod`).

### Tests

- Unit tests in `plug_install_command.rs` `#[cfg(test)]` — 17 tests (12 mapping + 5 pre-mutation).
- `tethers-0.1/host-rust/tests/j24l2_plug_install_cli.rs` — 10 integration tests:
  - 9 Clap parsing tests.
  - 1 `#[cfg(windows)]` E2E test with exact byte-level conformance snapshot
    comparison proving no retry.

### Documentation

- `docs/architecture/J24L_THIN_PUBLIC_PLUG_INSTALL_CLI.md` — updated with J24L2 sections and "crate-private" fix.
- `docs/CURRENT_CLINE_TASK.md` — J24L2 task packet.
- `docs/worker-notes/2026-08-06-j24l2-thin-plug-install-cli.md` — this note.

## Decisions and assumptions

- Module is `mod` (private) with `pub(crate) fn run_install`. Integration tests invoke real binary.
- Pre-mutation tests in `#[cfg(test)]` module where crate-private access is available.
- `map_conformance_stop` uses exhaustive fail-closed match (all four `ConformanceDisposition` variants).
- E2E test uses `conformance_snapshot` with SHA256 content hashing for byte-level
  equality check (detects added, removed, and changed evidence files).
- Integration test uses `host_binary()` pattern consistent with j24f, j24a, etc.

## Evidence

### J24L2 lib tests (17/17 passed)
COMMAND: `cargo test --lib -p tethers-reference-host j24l2_ --no-fail-fast --locked`
RESULT: PASS — 17 passed, 0 failed

### J24L1 regressions (7/7 passed)
COMMAND: `cargo test --lib -p tethers-reference-host j24l1_ --no-fail-fast --locked`
RESULT: PASS — 7 passed, 0 failed

### J24L2 integration tests (10/10 passed)
COMMAND: `cargo test --test j24l2_plug_install_cli --no-fail-fast --locked`
RESULT: PASS — 10 passed, 0 failed (including 1 #[cfg(windows)] E2E)

### All plug tests (32/32 passed)
COMMAND: `cargo test -p tethers-reference-host plug_ --no-fail-fast --locked`
RESULT: PASS — 32 passed, 0 failed

### J24K3f regressions (10/10 passed)
COMMAND: `cargo test --lib -p tethers-reference-host j24k3f --no-fail-fast --locked`
RESULT: PASS — 10 passed, 0 failed

### J24K2 regressions (26/26 passed)
COMMAND: `cargo test --lib -p tethers-reference-host j24k2 --no-fail-fast --locked`
RESULT: PASS — 26 passed, 0 failed

### J24J planner regressions (24/24 passed)
COMMAND: `cargo test --test j24j_installation_reconciliation --locked`
RESULT: PASS — 24 passed, 0 failed

### Formatting
COMMAND: `cargo fmt --all -- --check`
RESULT: PASS — EXIT 0

### Clippy
COMMAND: `cargo clippy --all-targets --all-features --locked`
RESULT: PASS — no new J24L2 warnings

### Release build
COMMAND: `cargo build --release --locked`
RESULT: PASS

### Full serial verification
COMMAND: `$env:RUST_TEST_THREADS="1"; just verify`
RESULT: PASS — 1254 passed, 0 failed, 2 ignored

### Packet checker
COMMAND: `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`
RESULT: PASS

### Cargo.lock unchanged
COMMAND: `git diff --exit-code -- tethers-0.1/host-rust/Cargo.lock`
RESULT: PASS — no changes

### Diff check
COMMAND: `git diff --check`
RESULT: PASS — clean

### Git status
COMMAND: `git status --short --branch`
RESULT: PASS — clean after final commit

## Discoveries

- `installation-intent/current.json` persists after successful publication (StoreRoot::open creates dir).
  The absence of `current.json` proves intent was consumed.
- Conformance store uses M3 store with content-hash filenames that remain stable
  across re-opens, making byte-level snapshot comparison a reliable no-retry proof.
- Prior implementation checkpoint was `f0f0c84` (abbreviated); the full SHA is
  `f0f0c841cf77bca63a0b916d03fdaad45160ccb5`.

## Remaining risks

None known within J24L2 scope.

## Smallest next action

Lucy reviews and accepts. Matthew may merge the J24L branch into main.

## References

- `docs/architecture/J24L_THIN_PUBLIC_PLUG_INSTALL_CLI.md`
- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `docs/architecture/J24G_INSTALLATION_REQUEST_CONTRACT.md`
- `tethers-0.1/host-rust/src/installation_driver.rs`
- `tethers-0.1/host-rust/src/plug_install_command.rs`
- Prior correction checkpoint: `f0f0c841cf77bca63a0b916d03fdaad45160ccb5`
- Implementation checkpoint: `d9cfeb21c08a29d0e115b9daa243f665ee84a4b8`
- Branch: `opencode/j24l2-thin-plug-install-cli`
