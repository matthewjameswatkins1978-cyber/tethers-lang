# Current Implementation Task

Control contract: `1`
Task packet: `PRE-F10 — Final Gate Consistency Repair`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Green`
Route: `OpenCode fixes justfile verify warning gate and reconciles current-state docs`
Worker note: `docs/worker-notes/2026-08-09-pre-f10-gate-consistency.md`
Base branch: `foundation/f9-final-reconciliation`
Base commit: `fc33dba435a87833a6f0f53642326697a246694b`
Implementation branch: `foundation/pre-f10-gate-consistency`
Implementation checkpoint: `2a69d71bff9e01d53f8f785573ff795b2057d00f`
OCaml switch path: `N/A`
Rust toolchain: `1.97.1`
Rust change class: `DOCS`

## Objective

Correct every remaining demonstrated inconsistency found by the independent pre-F10 sweep:
1. make `just verify` / `just verify-agent` enforce zero Rust compiler warnings;
2. reconcile current goal/dashboard truth now that F9-FINAL has completed.

## Relevant background and existing behaviour

F8-W2 added `RUSTFLAGS="-D warnings"` to the canonical `just check` recipe, but
`just verify` runs its own separate non-strict Cargo check invocation, bypassing
the warning gate. `CURRENT_GOAL.md` still describes F9 as the active documentation
phase. `PROJECT_DASHBOARD.md` contains a false "accepted and merged" claim for F8.

## Required behaviour

### Part A — Fix the warning gate
1. In `justfile`, replace `just verify`'s separate Cargo-check invocation with
   `just check`, so that `just verify` and `just verify-agent` genuinely enforce
   zero Rust compiler warnings.
2. Preserve ordering: task-packet check, formatting check, strict `just check`,
   Cargo tests.
3. No justfile redesign, no new scripts, no Clippy policy change.

### Part B — Current state truth
4. Update `CURRENT_GOAL.md`: F9-FINAL completed, pre-F10 consistency repair active,
   F10 remains sole gate, live main unchanged.
5. Update `PROJECT_DASHBOARD.md`: remove false "accepted and merged" claim for F8,
   record pre-F10 repair as active task, Foundation work on branch lineage.
6. At closeout: update dashboard to pre-F10 COMPLETE, pending Lucy review.

## Relevant components

### AUTHORISED PATHS
- `justfile`
- `docs/CURRENT_GOAL.md`
- `docs/PROJECT_DASHBOARD.md`

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-pre-f10-gate-consistency.md`
- final closeout update to `docs/PROJECT_DASHBOARD.md`

## Frozen decisions and invariants

- No Rust source change in final tree.
- No OCaml change.
- No dependency or lockfile change.
- No Clippy cleanup.
- No new script.
- No new CI.
- No workflow redesign.
- No merge to main.
- No F10 verification yet.
- F10 remains sole Foundation completion gate.

## Acceptance criteria

1. `just verify` uses canonical strict `just check`.
2. A deliberate temporary warning causes `just verify` to fail before Cargo tests.
3. Clean `just check` passes with zero warnings.
4. Temporary Rust change is restored exactly.
5. Dashboard contains no false "F8 merged" claim.
6. F9 is no longer described as IN_PROGRESS.
7. F10 remains the sole completion gate.
8. COMPLETE-state packet checker passes.
9. Branch is pushed and clean.

## Required verification

### Positive proof
1. Inspect `just verify` recipe and prove it invokes canonical `just check`.
2. Run `just check`; require zero warnings / PASS.

### Negative proof
3. Temporarily introduce one harmless unused Rust import.
4. Run `just verify`; require failure at strict `just check` stage (unused-import
   warning promoted to error; Cargo tests not reached).
5. Restore source file byte-for-byte; prove no Rust diff remains.
6. Do not commit the temporary warning.

### Cheap final checks
7. `just fmt`
8. `git diff --check`
9. Inspect full diff; prove only authorised tooling/docs remain.
10. Do NOT run `just verify-agent`.
11. Do NOT rerun the full Rust suite.

### Closeout
12. COMPLETE-state packet checker must report `control-v1/COMPLETE`.

## Forbidden changes

- No Rust source change in final tree.
- No OCaml change.
- No dependency or lockfile change.
- No Clippy cleanup.
- No new script.
- No new CI.
- No workflow redesign.
- No merge to main.
- No F10 verification yet.

## Stop conditions

STOP if any stale claim persists, if diff touches non-authorised files, or if
COMPLETE-state packet checker fails.

## Expected pre-existing changes

None.
