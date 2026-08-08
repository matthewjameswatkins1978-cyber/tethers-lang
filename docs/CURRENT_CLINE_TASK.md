# Current Implementation Task

Control contract: `1`
Task: `Control — Permanent Worker-Evidence Hardening`
Owner: `OpenCode`
Model: `DeepSeek Pro HIGH`
Status: `COMPLETE`
Task colour: `Amber`
Route: `OpenCode implements control-system hardening; no product semantics`
Worker note: `docs/worker-notes/2026-08-08-control-worker-evidence-finalization.md`
Base branch: `foundation/f4a2-rust-planner-boundary`
Base commit: `a9c2862adfd3bca5c7c253609c397ad9a59c5ac8`
Implementation branch: `foundation/control-worker-evidence-finalization`
OCaml switch path: `N/A`
Rust toolchain: `N/A`
Toolchain preflight: `pwsh -NoProfile -File scripts/check-dev-tools.ps1`

## Objective

Fix the demonstrated process defect where a COMPLETE task could pass the packet checker with `Implementation checkpoint: WORKTREE` and the workflow permitted the worker note to be written before a committed implementation checkpoint was established. Enforce: implement, commit checkpoint, verify against checkpoint, write worker note, commit closeout docs only.

## Relevant background and existing behaviour

The F4a2 task passed the packet checker with `WORKTREE` as its implementation checkpoint. The R1/R2 reconciliation tasks needed to repair stale evidence in the worker note after the fact. The checker had no ancestry or post-checkpoint closeout-scope checks.

## Relevant components

- `.github/scripts/check-tethers-task-packet.ps1` — checker enhancement
- `.github/scripts/test-check-tethers-task-packet.ps1` — NEW test script
- `docs/AGENT_WORKFLOW.md` — workflow hardening
- `docs/PROJECT_CONTROL.md` — COMPLETE definition tightening
- `docs/WORKER_NOTE_TEMPLATE.md` — checkpoint instructions

## Required behaviour

1. COMPLETE/ACCEPTED/REJECTED must have a full 40-character commit SHA as implementation checkpoint; WORKTREE is rejected
2. BLOCKED may still use WORKTREE
3. Implementation checkpoint must resolve to a real commit
4. Base commit must be an ancestor of implementation checkpoint
5. Implementation checkpoint must be an ancestor of HEAD
6. Only closeout paths may differ between implementation checkpoint and HEAD
7. Test script validates all required behaviours with independent temp repos

## Frozen decisions and invariants

- No Rust. No OCaml. No product semantics.
- BLOCKED + WORKTREE stays legal
- Closeout paths: packet, worker note, PROJECT_DASHBOARD.md
- No weakening of COMPLETE evidence requirements

## Forbidden changes

- No product code changes
- No OCaml changes
- No protocol or trust boundary changes
- No `cargo test` or `cargo fmt` requirement for control-only changes
- No beginning F4b or F5

## Stop conditions

STOP if:
- Implementing checkpoint ancestry requires modifying product code
- Checker cannot distinguish closeout paths
- Existing legitimate COMPLETE workflow depends on production edits after checkpoint
- More than listed control/workflow files become necessary

## Expected pre-existing changes

None

## Acceptance criteria

1. COMPLETE + WORKTREE checkpoint fails checker
2. BLOCKED + WORKTREE checkpoint passes checker
3. COMPLETE + nonexistent SHA fails checker
4. COMPLETE + valid checkpoint with closeout-only diff passes checker
5. COMPLETE + production changed after checkpoint fails checker
6. COMPLETE + arbitrary non-closeout doc after checkpoint fails checker
7. COMPLETE + packet/worker-note closeout passes checker
8. AGENT_WORKFLOW.md records commit-before-evidence order
9. PROJECT_CONTROL.md defines COMPLETE with committed checkpoint requirement
10. WORKER_NOTE_TEMPLATE.md distinguishes COMPLETE from BLOCKED checkpoint rules

## Required verification

```powershell
pwsh -NoProfile -File .github/scripts/test-check-tethers-task-packet.ps1
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
git diff --check
```
