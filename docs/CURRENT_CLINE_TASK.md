# Current Implementation Task

Control contract: `1`
Task packet: `F9-A — Current Operator Truth Reconciliation`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Green`
Route: `OpenCode updates stale operator-facing documentation`
Worker note: `docs/worker-notes/2026-08-09-f9-operator-truth.md`
Base branch: `foundation/f8-warning-enforcement`
Base commit: `5e616357963e70b86f59c870f6c00b7fbc94cb0a`
Implementation branch: `foundation/f9-operator-truth`
Implementation checkpoint: `3dc85fe989ca53619f17e525431c5726e6d73079`
OCaml switch path: `N/A`
Rust toolchain: `1.97.1`
Rust change class: `DOCS`

## Objective

Update the small set of current human/operator-facing documents that are
demonstrably stale after the completed Foundation work through F8.

## Relevant background and existing behaviour

F1-F8 are complete through warning enforcement at
`5e616357963e70b86f59c870f6c00b7fbc94cb0a`. Three current/operator-facing
documents still describe F3c/F3d as the active phase and Cline as the
normal implementation worker — both are obsolete.

## Required behaviour

1. `docs/CURRENT_GOAL.md` — update to reflect F9 truth (F1-F8 complete,
   F8 warning enforcement is latest accepted checkpoint).
2. `docs/PROJECT_DASHBOARD.md` — update milestone, checkpoint, active task,
   and remove obsolete F3 drift notes.
3. `docs/GORILLA_CODING_QUICK_CARD.md` — update to OpenCode/Lucy/Codex/Matthew
   operating reality; remove Cline as active worker.
4. F10 remains the sole Foundation completion gate.

## Relevant components

### AUTHORISED PATHS
- `docs/CURRENT_GOAL.md`
- `docs/PROJECT_DASHBOARD.md`
- `docs/GORILLA_CODING_QUICK_CARD.md`

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-f9-operator-truth.md`

## Frozen decisions and invariants

- No Rust, OCaml, justfile, task-checker, architecture, or workflow changes.
- No speculative post-Foundation plans.
- No rewriting historical worker notes.
- No polishing unrelated documentation.

## Acceptance criteria

1. Current goal reflects F9 rather than F3.
2. Dashboard reflects the live Foundation state.
3. Gorilla quick card reflects OpenCode/Codex/Lucy/Matthew operating reality.
4. No abandoned context/workflow redesign introduced.
5. F8 warning enforcement described accurately where relevant.
6. F10 remains sole Foundation completion gate.
7. No implementation/tooling tree changed.
8. Branch pushed with clean status.

## Required verification

1. Inspect all authorised document diffs.
2. Search updated docs for stale claims (F3c accepted, F3d active, Cline active).
3. Confirm new F8 checkpoint SHA is exact.
4. Confirm F10 remains Foundation completion gate.
5. Packet checker.
6. `git diff --check`.
7. Prove final diff is docs-only.

## Forbidden changes

- No Rust/OCaml/justfile/task-checker changes.
- No architecture/workflow redesign.
- No Cline-file renaming.
- No GitHub Actions.
- No speculative plans.
- No rewriting historical worker notes.

## Stop conditions

STOP if any stale claim persists in updated documents, if diff touches
non-docs files, or if packet checker fails.

## Expected pre-existing changes

None.
