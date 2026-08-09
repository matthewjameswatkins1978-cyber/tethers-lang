# Current Implementation Task

Control contract: `1`
Task packet: `F8-D2 — Legacy parse_provision_args Cleanup`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Green`
Route: `OpenCode removes D2 (parse_provision_args) dead parser`
Worker note: `docs/worker-notes/2026-08-09-f8-d2-parse-provision-args-cleanup.md`
Base branch: `foundation/f8-d1-provision-usage-cleanup`
Base commit: `38708d2294ccd0df4bc468c0a6edc856643ba0e4`
Implementation branch: `foundation/f8-d2-parse-provision-args-cleanup`
Implementation checkpoint: `45dba36dc4ea1b088b4cc5ba09e412046b1cc37c`
Rust change class: `RUST`

## Objective

Remove production dead-code warning D2: the unused `parse_provision_args`
function from `tethers-0.1/host-rust/src/application.rs`, and migrate the
wrong-shape rejection coverage to the live Clap parsing path.

## Relevant background and existing behaviour

`parse_provision_args` (line 87) is a legacy manual argument parser for the
`provision-replay` subcommand. It validates that the first positional arg is
exactly "provision-replay" and that exactly two args are provided with an
absolute root path. It returns usage-string errors for wrong shapes.

The actual provisioning CLI route uses Clap-based `CliCommand::ProvisionReplay`
(`cli.rs:59-63`, dispatched at `application.rs:663-695`), which performs its
own argument parsing and validation. `parse_provision_args` is never called in
production code. The legacy `run_legacy_host` function only calls
`parse_normal_args`, not `parse_provision_args`.

The only caller of `parse_provision_args` is the test
`j09_runtime_42_provisioning_wrong_shapes_are_rejected_without_mutation` at
lines 6727-6748, which tests the dead parser's error paths for three wrong
argument shapes.

Classification: **DEAD** — genuinely unused in production. The Clap-based
`ProvisionReplay` command handles all argument parsing and validation.

## Required behaviour

1. Delete `parse_provision_args` (lines 87-100).
2. Delete the test `j09_runtime_42_provisioning_wrong_shapes_are_rejected_without_mutation`
   (only tests the dead parser).
3. Migrate wrong-shape rejection coverage: add Clap-based parse tests in
   `cli.rs` proving `ProvisionReplay` rejects missing root, unknown options,
   and extra positional args.
4. Run `cargo fmt` on changed files.
5. Confirm D2 warning is gone from `cargo check --all-targets --all-features --locked`.
6. Confirm D3-D15 warnings remain otherwise unchanged.
7. Run full `just verify-agent` once.

## Frozen decisions and invariants

- Do not resolve or suppress D3-D15.
- Do not add `#[allow(dead_code)]` suppression.
- Do not rename, refactor, or opportunistically clean code outside D2.
- Preserve all runtime, replay/recovery, CLI, JSON/protocol, and compatibility
  behaviour.
- The live `ProvisionReplay` Clap dispatch must remain unchanged.
- Coverage of wrong-shape rejection must survive in the Clap path.

## Acceptance criteria

1. D2 warning absent from `cargo check` lib target.
2. `parse_provision_args` function removed from `application.rs`.
3. `rg "parse_provision_args" --type rust` returns zero Rust source matches.
4. Wrong-shape rejection covered by Clap parse tests in `cli.rs`.
5. No replacement suppression added.
6. D3-D15 warnings remain otherwise unchanged.
7. No production semantics changed.
8. `cargo fmt` only touches `application.rs` and `cli.rs`.
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
- `rg "parse_provision_args"` returns zero Rust source matches

## Relevant components

### AUTHORISED PATHS
- `tethers-0.1/host-rust/src/application.rs` — remove dead function and test
- `tethers-0.1/host-rust/src/cli.rs` — add migration coverage tests

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-f8-d2-parse-provision-args-cleanup.md`

## Forbidden changes

- No D3-D15 resolution or suppression
- No OCaml source changes
- No other Rust source changes outside authorised paths
- No Nextest configuration changes
- No CI changes
- No dependency policy changes
- No tool version changes
- No `#[allow(...)]` suppression additions
- No production dead-code cleanup beyond D2

## Stop conditions

STOP if removing `parse_provision_args` would break any live code path.
STOP if rustfmt touches any file other than `application.rs` or `cli.rs`.
STOP if migration coverage is insufficient.
STOP if verification fails.
STOP if two materially similar implementation attempts fail.

## Expected pre-existing changes

None.
