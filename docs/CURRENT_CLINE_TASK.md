# Current Implementation Task

Control contract: `1`
Task packet: `F8-T1 — Test-Only Dead Warning Cleanup`
Owner: `OpenCode`
Status: `IN_PROGRESS`
Task colour: `Green`
Route: `OpenCode removes proven test-only dead-code warnings T1–T14`
Worker note: `docs/worker-notes/2026-08-09-f8-t1-test-warning-cleanup.md`
Base branch: `foundation/f8-worker-lifecycle-carry`
Base commit: `f6c7401f2034da79c609ff25b84e651bd001f80a`
Implementation branch: `foundation/f8-t1-test-warning-cleanup`
Implementation checkpoint: `TO BE SET`
Rust change class: `RUST_CHANGING`

## Objective

Remove only the proven test-only compiler/dead-code warnings T1–T14 from the
F8a inventory without weakening any test. No production code changes.

## Relevant background and existing behaviour

Current baseline: ~33 cargo check warnings, ~45 distinct Clippy warnings.
T1–T14 are test-only unused imports, unused bindings, unread struct fields,
and unused helper functions in the 8 authorised test files. None contribute
assertions, setup, cleanup, or failure-path evidence.

## Required behaviour

1. Remove T1–T14 warnings from the 8 authorised test files only.
2. For each item, confirm the unused element contributes no test assertion,
   setup, cleanup, compatibility evidence, or failure-path evidence.
3. Leave unchanged any item whose intent is uncertain.
4. Run `cargo fmt` before the implementation checkpoint.
5. All existing tests must continue to pass.

## Frozen decisions and invariants

- T1–T14 are proven test-only dead code. No redesign is needed.
- Do not touch T15 `FailingResultAnchorWriter` in `src/application.rs`.
- Do not touch production code, `src/application.rs`, `src/bin/*`, or lint config.
- Do not weaken any test.
- Do not blindly prefix with `_` — remove genuinely unused items.

## Acceptance criteria

1. Cargo check warning count decreases from baseline
2. Clippy distinct warning count decreases from baseline
3. `cargo test --all-targets --all-features --locked` passes with same test count
4. `cargo fmt --all -- --check` passes
5. `git diff --check` passes
6. Packet checker passes
7. Diff touches only the 8 authorised Rust paths + task packet + worker note
8. No production files changed

## Required verification

- `cargo fmt --all -- --check`
- `cargo check --all-targets --all-features --locked`
- `cargo test --all-targets --all-features --locked`
- `cargo clippy --all-targets --all-features --locked -- -W clippy::all`
- `just verify`
- `just verify-agent`
- `git diff --check`
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`

## Relevant components

### AUTHORISED RUST PATHS
- `tethers-0.1/host-rust/tests/j13a_cli.rs`
- `tethers-0.1/host-rust/tests/j23b_pdf_package.rs`
- `tethers-0.1/host-rust/tests/j23c3_installed_pdf_execution.rs`
- `tethers-0.1/host-rust/tests/j24d_plug_enable_scope_file.rs`
- `tethers-0.1/host-rust/src/installation_publication_mutation_tests.rs`
- `tethers-0.1/host-rust/src/installation_publication_preparation_tests.rs`
- `tethers-0.1/host-rust/src/installation_execution_tests.rs`
- `tethers-0.1/host-rust/src/installation_recovery_plan_tests.rs`

### TARGET WARNINGS
- T1: unused `std::io::Write` — j13a_cli.rs
- T2: unused `code` bindings (3) — j13a_cli.rs
- T3: unused `envelope` — j13a_cli.rs
- T4: unused `serde_json::Value` — j23b_pdf_package.rs
- T5: unused `Write` / `PathBuf` / `MAX_PDF_BYTES` — j23c3_installed_pdf_execution.rs
- T6: unused `before` — j24d_plug_enable_scope_file.rs
- T7: unused `canonical` helper — j24d_plug_enable_scope_file.rs
- T8: unused `InstallationPlanAction` / `DisabledBindingRecord` imports
- T9: unused `error` binding
- T10: unused `PayloadEvidence` import
- T11: unused `empty_plan` helper
- T12: unused `plan_with` helper
- T13: unread fixture struct fields
- T14: unread `FullFixture` struct fields

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-f8-t1-test-warning-cleanup.md`

## Forbidden changes

- No production code changes
- No `src/application.rs`
- No `src/bin/*`
- No `suspicious_open_options`
- No preference lints
- No Clippy architecture changes
- No lint configuration / CI / warning gates
- No T15 `FailingResultAnchorWriter`
- No other F8 warning families

## Stop conditions

STOP if `cargo fmt` changes any Rust file outside the 8 authorised paths.
STOP if a test fails after cleanup.
STOP if an unused item appears to be intentional test evidence.
STOP if two materially similar cleanup attempts fail.

## Expected pre-existing changes

None.
