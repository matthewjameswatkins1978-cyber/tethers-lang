# Worker Note

Task: `F8-D2 — Legacy parse_provision_args Cleanup`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `38708d2294ccd0df4bc468c0a6edc856643ba0e4`

Implementation checkpoint: `45dba36dc4ea1b088b4cc5ba09e412046b1cc37c`

## Requested outcome

Remove production dead-code warning D2: the unused `parse_provision_args` legacy
manual argument parser from `application.rs`. Migrate wrong-shape rejection
coverage to the live Clap parsing path in `cli.rs`.

## Changes made

- `tethers-0.1/host-rust/src/application.rs`: Deleted `parse_provision_args`
  function (14 lines, old lines 87-100). Deleted the test
  `j09_runtime_42_provisioning_wrong_shapes_are_rejected_without_mutation`
  (22 lines, old lines 6727-6748) which tested only the dead parser.
- `tethers-0.1/host-rust/src/cli.rs`: Added 4 Clap-based unit tests for
  `ProvisionReplay` argument validation:
  - `j13a_provision_replay_valid_parse` — valid `C:\host-data` root parses
  - `j13a_provision_replay_missing_root_rejected` — no root arg rejected
  - `j13a_provision_replay_extra_positional_rejected` — extra arg rejected
  - `j13a_provision_replay_unknown_option_rejected` — unknown `--host-data-root` option rejected

## Decisions and assumptions

- Classification: **DEAD**. `parse_provision_args` was a legacy manual parser
  for `provision-replay`, completely bypassed by the Clap-based
  `CliCommand::ProvisionReplay { root }` dispatch at `application.rs:663-695`.
  Confirmed `run_legacy_host` only calls `parse_normal_args`, never
  `parse_provision_args`.
- The test `j09_runtime_42` tested only the dead parser's error paths. Its
  behavioral coverage (wrong argument shapes rejected) was migrated to Clap
  parse tests in `cli.rs`, following the existing `parse_cli` helper pattern.
- The existing integration test `j13a_provision_replay_hidden_accessible` in
  `tests/j13a_cli.rs` continues to cover the live binary path.
- D3-D15 remain entirely unchanged (13 lib warnings).

## Evidence

- Pre-change references: `parse_provision_args` at `application.rs:87`
  (definition) and `application.rs:6744` (test call). Zero other Rust references.
- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check` — PASS
- `cargo check --all-targets --all-features --locked` — 13 lib warnings (D3-D15), D2 absent. 9 lib test warnings (unchanged).
- `cargo clippy --all-targets --all-features --locked` — PASS (no errors)
- `cargo test --all-targets --all-features --locked` — 1592 passed (1334 lib + 258 integration), 0 failed, 2 skipped
- `git diff --check` — PASS
- Packet checker: PASS
- `just verify-agent` — PASS (1592 passed, 0 failed, 2 skipped; nextest 39.1s)
- `rg parse_provision_args` in Rust source — Zero matches

## Publication evidence

Branch `foundation/f8-d2-parse-provision-args-cleanup` pushed to `origin`.

## Discoveries

None.

## Remaining risks

None known within packet scope. D3-D15 remain unresolved.

## Smallest next action

Resolve D3 (`run_event_admission_probe`) or another remaining dead-code item
as prioritised by Lucy.

## References

- `tethers-0.1/host-rust/src/application.rs` — function and test removed
- `tethers-0.1/host-rust/src/cli.rs` — Clap parse tests added (lines 403-428)
- `tethers-0.1/host-rust/tests/j13a_cli.rs` — existing live integration test line 311
