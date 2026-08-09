# Current Implementation Task

Control contract: `1`
Task packet: `F8-W1 — Remove Final Two Test Import Warnings`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Green`
Route: `OpenCode removes the two pre-existing test-module unused-import warnings`
Worker note: `docs/worker-notes/2026-08-09-f8-final-test-import-warnings.md`
Base branch: `foundation/f8-zero-warning-checkpoint`
Base commit: `15b792e32afa83bfd9bc2b5c64451202df15a794`
Implementation branch: `foundation/f8-final-test-import-warnings`
Implementation checkpoint: `27021c3b0ad023024eb0dfbd57f9492ca525a1be`
OCaml switch path: `N/A`
Rust toolchain: `1.97.1`
Rust change class: `RUST`

## Objective

Remove only the two pre-existing test-module unused-import warnings recorded
by the accepted F8 zero-production-warning checkpoint.

## Relevant background and existing behaviour

The accepted F8 checkpoint records zero intended production-library warnings
and exactly two pre-existing test-module unused-import diagnostics:
- `InstallationPlan` in `installation_execution_tests.rs:11`
- `InstallationApprovalRecord` in `installation_publication_mutation_tests.rs:28`

This job removes only those imports and does not activate warnings-as-errors.

## Required behaviour

1. Remove the genuinely unused `InstallationPlan` import.
2. Remove the genuinely unused `InstallationApprovalRecord` import.
3. Add no `#[allow(...)]`, suppression, or cfg tricks.
4. Change no production semantics, tests, or nearby code.
5. Do not activate warnings-as-errors or clean unrelated Clippy advice.

## Relevant components

### AUTHORISED PATHS
- `tethers-0.1/host-rust/src/installation_execution_tests.rs`
- `tethers-0.1/host-rust/src/installation_publication_mutation_tests.rs`

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-f8-final-test-import-warnings.md`

## Frozen decisions and invariants

- Accepted F8 production cleanup remains unchanged.
- No warning denial yet.
- No Clippy debt cleanup.
- No CI introduction.
- No product semantics changes.
- No dependency/toolchain changes.
- No test behaviour changes beyond removal of unused imports.

## Acceptance criteria

1. The two accepted residual compiler warnings are absent.
2. Existing all-target Cargo check reports zero warnings.
3. Affected tests pass.
4. No warning suppression was introduced.
5. Only authorised paths changed.
6. Branch pushed normally with clean status.

## Required verification

1. `cargo fmt` on authorised Rust files.
2. Locked all-target `cargo check` — zero warnings.
3. Affected test modules pass.
4. `git diff --check`.
5. Packet checker.

## Forbidden changes

- No `#[allow(...)]` or suppression.
- No other Rust files.
- No production semantics, dependency, toolchain, or CI changes.

## Stop conditions

STOP if cargo check reports any warning after edits, if formatter touches
unauthorised files, or if any required check fails.

## Expected pre-existing changes

None.
