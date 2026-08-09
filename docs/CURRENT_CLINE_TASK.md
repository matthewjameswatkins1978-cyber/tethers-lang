# Current Implementation Task

Control contract: `1`
Task packet: `F8-D1 — First Production Dead-Code Warning Cleanup`
Owner: `OpenCode`
Status: `IN_PROGRESS`
Task colour: `Green`
Route: `OpenCode removes D1 (PROVISION_USAGE) dead-code constant`
Worker note: `docs/worker-notes/2026-08-09-f8-d1-provision-usage-cleanup.md`
Base branch: `foundation/f8-t15-test-warning-cleanup`
Base commit: `66da0492d0bc681defefa66a92cdb40287dcb05c`
Implementation branch: `foundation/f8-d1-provision-usage-cleanup`
Implementation checkpoint: `TBD`
Rust change class: `RUST`

## Objective

Remove production dead-code warning D1: the unused `PROVISION_USAGE` constant
from `tethers-0.1/host-rust/src/application.rs`.

## Relevant background and existing behaviour

`PROVISION_USAGE` (line 24) is a legacy manual-parser usage string for the
`provision-replay` subcommand. The actual provisioning CLI route uses Clap-based
`CliCommand::ProvisionReplay` (`cli.rs:59-63`, dispatched at `application.rs:661-694`),
which generates its own usage strings. The constant is only referenced inside
`parse_provision_args` (D2, line 89), itself never called in production code.
Both survive only in one test at line 6726-6743.

The legacy manual-parser architecture (`parse_provision_args` / `run_legacy_host`)
predates the Clap-based CLI refactor. The constant and its parser are completely
routed around in production.

### Evidence summary
- `PROVISION_USAGE` defined at `application.rs:24-25`
- Used at `application.rs:91, 95` inside `parse_provision_args` (D2)
- Asserted against at `application.rs:6741` in test `j09_runtime_42_provisioning_wrong_shapes_are_rejected_without_mutation`
- Zero other Rust source references
- Actual provisioning route: `Cli::parse()` → `CliCommand::ProvisionReplay { root }` → `replay_windows::provision_replay(&root)` (lines 661-694)
- Classification: **DEAD** — genuinely unused in production

## Required behaviour

1. Delete the `PROVISION_USAGE` constant (lines 24-25).
2. Replace the two inline references inside `parse_provision_args` (D2) with
   the string literal (consequential adaptation, not D2 resolution).
3. Replace the test assertion reference (line 6741) with the string literal
   (consequential adaptation; test behaviour identical).
4. Run `cargo fmt` on the changed file only.
5. Confirm D1 warning is gone from `cargo check --all-targets --all-features --locked`.
6. Confirm D2-D15 warnings remain otherwise unchanged in count and identity.
7. Run full `just verify-agent` once.

## Frozen decisions and invariants

- Do not resolve or suppress D2-D15.
- Do not add `#[allow(dead_code)]` suppression.
- Do not rename, refactor, or opportunistically clean code outside D1.
- Preserve all runtime, replay/recovery, CLI, JSON/protocol, and compatibility
  behaviour.
- `parse_provision_args` (D2) must still compile and produce identical errors.
- The test at line 6726 must still pass with identical semantic assertions.

## Acceptance criteria

1. D1 warning absent from `cargo check` lib target.
2. `PROVISION_USAGE` constant removed.
3. `parse_provision_args` (D2) still compiles and returns identical errors.
4. Test `j09_runtime_42_provisioning_wrong_shapes_are_rejected_without_mutation`
   passes with identical behaviour.
5. No replacement suppression added.
6. D2-D15 warnings remain otherwise unchanged.
7. No production semantics changed.
8. `cargo fmt` only touches `application.rs`.
9. `just verify-agent` passes once.
10. Branch pushed and local == remote.

## Required verification

- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`
- `cargo check --all-targets --all-features --locked`
- `cargo clippy --all-targets --all-features --locked`
- `cargo test --all-targets --all-features --locked`
- `git diff --check`
- Packet checker
- `just verify-agent` (full regression)
- `rg PROVISION_USAGE` returns zero Rust source matches

## Relevant components

### AUTHORISED PATHS
- `tethers-0.1/host-rust/src/application.rs`

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-f8-d1-provision-usage-cleanup.md`

## Forbidden changes

- No D2-D15 resolution or suppression
- No OCaml source changes
- No other Rust source changes outside consequential adaptations
- No Nextest configuration changes
- No CI changes
- No dependency policy changes
- No tool version changes
- No `#[allow(...)]` suppression additions
- No production dead-code cleanup beyond D1

## Stop conditions

STOP if removing D1 would silently break `parse_provision_args` behaviour.
STOP if rustfmt touches any file other than `application.rs`.
STOP if the test `j09_runtime_42` fails.
STOP if verification fails.
STOP if two materially similar implementation attempts fail.

## Expected pre-existing changes

None.
