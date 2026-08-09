# Worker Note — F8-T1 Test-Only Dead Warning Cleanup

Task: `F8-T1 — Test-Only Dead Warning Cleanup`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `f6c7401f2034da79c609ff25b84e651bd001f80a`
Implementation checkpoint: `183874812e6d422cf568783f0dbc56997197d2ba`

## Requested outcome

Remove T1–T14 proven test-only dead-code warnings from 8 authorised test files.
Zero production code changes. Zero test weakening.

## Changes made

All changes are in authorised test paths only. Formatting checked clean before
and after checkpoint.

### T1 — unused `std::io::Write` — j13a_cli.rs
Removed import. No `Write` usage anywhere in file. REMOVED.

### T2 — unused `code` bindings (×3) — j13a_cli.rs
Three destructure positions where `code` was never read. Replaced with `_`
wildcard pattern (standard Rust idiom for unused destructure, not prefixed `_name`).
All three in functions: `j13a_check_missing_config_emits_error`,
`j13a_check_missing_engine_emits_error`, `j13a_hidden_commands_not_in_help`.
REMOVED.

### T3 — unused `envelope` — j13a_cli.rs
`let envelope: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();`
in `j13a_directory_engine_rejected`. `envelope` never read. Removed line.
`stdout` destructure position replaced with `_` (becomes unused after removal).
REMOVED.

### T4 — unused `serde_json::Value` — j23b_pdf_package.rs
Removed import. No `Value` usage. REMOVED.

### T5 — unused `Write`, `PathBuf`, `MAX_PDF_BYTES` — j23c3_installed_pdf_execution.rs
Three unused items. Removed `use std::io::Write`, `use std::path::PathBuf`,
and `MAX_PDF_BYTES` from pdf_tools import. REMOVED.

### T6 — unused `before` binding — j24d_plug_enable_scope_file.rs
`let before = snapshot(&root);` in `disabled_pdf_plug_is_re_enabled...`.
Only `before_json` used. Removed the `before` line. REMOVED.

### T7 — unused `canonical` helper — j24d_plug_enable_scope_file.rs
Function `fn canonical<T: serde::Serialize>(value: &T) -> Vec<u8>` never called.
Removed entire function. REMOVED.

### T8 — unused `InstallationPlanAction` / `DisabledBindingRecord` imports — mutation_tests.rs
`InstallationPlanAction` imported but never used as type or variant.
`DisabledBindingRecord` imported but never used. Both removed from import lines.
REMOVED.

### T9 — unused `error` binding — mutation_tests.rs
`let error = execute_validated_installation_recovery(...).unwrap_err()` —
`error` never read. Replaced with `let _ =` to preserve the side-effecting call.
REMOVED.

### T10 — unused `PayloadEvidence` import — preparation_tests.rs
`PayloadEvidence` imported at line 28 but never used in preparation tests.
(Confirmed used in mutation tests file — warning was for preparation file only.)
Removed; simplified import to `use crate::package;`. REMOVED.

### T11 — unused `empty_plan` helper — execution_tests.rs
Never called anywhere. `plan_with` (T12) was its only caller. Removed. REMOVED.

### T12 — unused `plan_with` helper — execution_tests.rs
Never called anywhere. Removed together with T11. REMOVED.

### T13 — unread fixture struct fields — mutation_tests.rs
Four fields in `Fixture` struct set in constructor but never read by any method
or test: `trust`, `launch`, `conformance_evidence`, `approval`. Removed from
struct definition and constructor assignments. Local variables preserved where
used in constructor logic (e.g., `trust` in `approve_with_authority`). The
`approval` local became unused after field removal; replaced `let approval =`
with `let _ =` to preserve the side-effecting store write. REMOVED.

### T14 — unread `FullFixture` struct fields — recovery_plan_tests.rs
Three fields in `FullFixture` set in constructor but never read by any method or
test: `launch`, `conformance_evidence`, `approval`. Removed from struct definition
and constructor assignments. Local variables preserved (used in constructor logic).
REMOVED.

### Items deliberately left unchanged

None. All T1–T14 items were proven dead code with zero test contribution.

## Decisions and assumptions

- `_` wildcard pattern in destructures is the standard Rust idiom for unused
  tuple positions. This is distinct from `_variable` prefixing which the task
  forbids.
- `let _ =` for side-effecting calls where the return value is unused is also
  the standard Rust idiom.
- `cargo fmt` produced zero additional changes after edits were applied.

## Evidence

| Check | Result | Detail |
| --- | --- | --- |
| cargo check warnings | 42 → 21 | 21 warnings removed (T1–T14) |
| clippy raw warnings | 139 → 119 | 20 raw warnings removed |
| `cargo test` | PASS | 1331 passed, 0 failed, 2 ignored |
| `cargo fmt --check` | PASS | Clean |
| `just verify` | PASS | All 4 steps |
| `just verify-agent` | PASS | All steps: verify, agent-tools (15/15), deps-policy (ok), deps-advisories (ok), test-agent (1589/1589) |
| `git diff --check` | PASS | Clean |
| Packet checker | PASS | control-v1/COMPLETE |
| Diff from base | 10 files | 8 authorised Rust + task packet + worker note |

## Discoveries

1. **T10** `PayloadEvidence` is used in `installation_publication_mutation_tests.rs`
   (line 506) but unused in `installation_publication_preparation_tests.rs`.
   The warning came from the preparation file, correctly cleaned there.

## Remaining risks

None. T15 `FailingResultAnchorWriter` (in `application.rs`) and all other F8
warning families remain untouched as required.

## Smallest next action

Lucy decides which F8 cleanup package proceeds next. The pipeline is verified.

## References

- Task packet: `docs/CURRENT_CLINE_TASK.md`
- Base: `f6c7401f2034da79c609ff25b84e651bd001f80a`
- Implementation checkpoint: `183874812e6d422cf568783f0dbc56997197d2ba`
- Branch: `foundation/f8-t1-test-warning-cleanup`
