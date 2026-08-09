# Current Implementation Task

Control contract: `1`
Task packet: `F9-FINAL — Complete Operator Truth + Closeout Reliability Reconciliation`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Green`
Route: `OpenCode reconciles full current operational surface and fixes closeout SHA workflow`
Worker note: `docs/worker-notes/2026-08-09-f9-final-reconciliation.md`
Base branch: `foundation/f9-operator-truth`
Base commit: `74aedb9868621e1f7307665319fa80cc59d113d0`
Implementation branch: `foundation/f9-final-reconciliation`
Implementation checkpoint: `c6215c2656ac8247b494e820e46077a7d23c5efb`
OCaml switch path: `N/A`
Rust toolchain: `1.97.1`
Rust change class: `DOCS`

## Objective

Finish F9 in one bounded reconciliation by correcting every demonstrated
stale/current-operating claim in the full current operational surface, and
fix the repeated closeout-SHA workflow ambiguity without adding new tooling.

## Relevant background and existing behaviour

F9-A updated three operator-facing documents but missed false `origin/main`
claims in those same docs (F8/F9 not yet merged to main), left `CLINE_HANDOFF.md`
and `README.md` with obsolete Cline/model-specific routing, left Cline
integration files without inactive guards, and did not address the closeout
SHA workflow defect demonstrated in F8-W1-R1 and F9-A-R1.

## Required behaviour

### Part A — Foundation state truth
1. Fix false `origin/main` claims in `CURRENT_GOAL.md` and `PROJECT_DASHBOARD.md`.
2. Live `main` remains `40ec42eb2aac108901d428af3cbfe264d3edd6dc`.

### Part B — Handoff truth
3. Rewrite `CLINE_HANDOFF.md` as worker-neutral Gorilla handoff guide.

### Part C — README truth
4. Replace model-specific routing (Luna/DeepSeek/Codex Terra) with role-based routing.
5. Update CLINE_HANDOFF.md description.

### Part D — Inactive integration truth
6. Add inactive/historical guard blocks to `.github/copilot-instructions.md` and
   all four Cline integration files.
7. Update Copilot instructions to reflect OpenCode/Codex routing.

### Part E — Closeout SHA reliability
8. Update `AGENT_WORKFLOW.md` completion sequence: capture SHA from Git,
   run COMPLETE-state packet checker before closeout commit.
9. Update `TASK_PACKET_TEMPLATE.md` and `WORKER_NOTE_TEMPLATE.md` with same requirements.

## Relevant components

### AUTHORISED PATHS
- `docs/CURRENT_GOAL.md`
- `docs/PROJECT_DASHBOARD.md`
- `docs/CLINE_HANDOFF.md`
- `README.md`
- `.github/copilot-instructions.md`
- `.clinerules/00-operating-discipline.md`
- `.clinerules/20-project-workflow.md`
- `.clinerules/workflows/tethers-task.md`
- `.cline/skills/tethers-task/SKILL.md`
- `docs/AGENT_WORKFLOW.md`
- `docs/TASK_PACKET_TEMPLATE.md`
- `docs/WORKER_NOTE_TEMPLATE.md`

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-f9-final-reconciliation.md`

## Frozen decisions and invariants

- No Rust/OCaml/justfile/task-checker changes.
- No merge to main.
- No Cline-file renaming.
- No workflow redesign.
- No context-efficiency redesign.
- No new CI.
- F10 remains sole Foundation completion gate.
- `AGENTS.md` and `PROJECT_CONTROL.md` already aligned; do not edit.

## Acceptance criteria

1. Zero false `origin/main` claims for F8/F9 in current docs.
2. Live `main` SHA preserved.
3. CLINE_HANDOFF.md describes worker-neutral Gorilla handoff.
4. README.md uses role-based routing, not model names.
5. All inactive integration files have clear inactive guards.
6. No inactive integration was accidentally reactivated.
7. AGENT_WORKFLOW.md requires SHA capture from Git and COMPLETE-state checker.
8. TASK_PACKET_TEMPLATE.md and WORKER_NOTE_TEMPLATE.md encode same rules.
9. F10 remains sole Foundation completion gate.
10. No implementation/tooling tree changed.
11. Branch pushed with clean status.

## Required verification

1. Verify live `main` SHA.
2. Search for false `origin/main` claims, Cline as default worker, model names.
3. Confirm inactive guards present on all four Cline files + Copilot instructions.
4. Confirm F10 gate.
5. `git diff --check`.
6. Prove diff is docs/integration-text only.
7. Packet checker (in COMPLETE state before closeout commit).

## Forbidden changes

- No Rust/OCaml/justfile/task-checker changes.
- No merge to main.
- No Cline-file renaming.
- No workflow redesign.
- No new CI.
- No rewriting historical worker notes.

## Stop conditions

STOP if any stale claim persists, if diff touches non-docs files, or if
COMPLETE-state packet checker fails.

## Expected pre-existing changes

None.
