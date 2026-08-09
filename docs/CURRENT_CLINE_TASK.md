# Current Implementation Task

Control contract: `1`
Task: `F8a-R1 — Evidence Repair`
Owner: `OpenCode`
Model: `DeepSeek Pro HIGH`
Status: `COMPLETE`
Task colour: `Amber`
Route: `OpenCode repairs F8a evidence defects only; no production changes`
Worker note: `docs/worker-notes/2026-08-09-f8a-warning-tooling-reconciliation.md`
Base branch: `foundation/f8a-warning-tooling-reconciliation`
Base commit: `5f98c31f4bf51b806222c7f3722997d74fbe5a5b`
Implementation branch: `foundation/f8a-r1-evidence-repair`
Implementation checkpoint: `TO BE SET`
OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`
Rust toolchain: `1.97.1`

## Relevant background and existing behaviour

F8a completed a warning and tooling reconciliation audit against base `5ecf54e`.
The evidence document at `docs/foundation-pass/WARNING_TOOLING_RECONCILIATION_F8A.md`
and worker note at `docs/worker-notes/2026-08-09-f8a-warning-tooling-reconciliation.md`
were committed at branch tip `5f98c31`.

Three defects were identified:
1. OCaml commands (`dune build`, `test-engine.ps1`) were recorded as FAIL/NOT RUN
   because no global switch was set. An explicit switch exists at
   `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`. The evidence table
   and worker note contradict each other (table says FAIL, note says NOT RUN).
2. The evidence document uses `audit checkpoint` inconsistently and attempts
   to self-reference its own commit SHA. Terminology needs clarification.
3. `suspicious_open_options` (2 sites) was classified as ACTIONABLE CLEANUP
   (C24). The installation lock uses explicit non-truncation by design.

## Objective

Repair F8a evidence defects only. No production, test, fixture, build, protocol,
script, or dependency changes.

## Required behaviour

1. Reconcile OCaml command evidence: check available switches, run `dune build`
   and `test-engine.ps1` with explicit switch, record results.
2. Fix the evidence/worker-note contradiction about OCaml commands.
3. Clarify checkpoint terminology: AUDIT CHECKPOINT vs EVIDENCE CHECKPOINT.
4. Reclassify `suspicious_open_options` as INTENT REVIEW / EXPLICIT NON-TRUNCATION.
5. Do not otherwise change the warning inventory or F8 package plan.

## Frozen decisions and invariants

- No production code changes.
- No Rust changes.
- No OCaml changes.
- No test changes.
- No fixture changes.
- No build changes.
- No protocol changes.
- No script changes.
- No dependency additions.
- No F8-FMT implementation.
- No F8 cleanup work.
- The installation lock uses explicit non-truncation by design.
- Do NOT add `.truncate(true)`. A possible later cleanup is explicit
  `.truncate(false)` if focused verification proves behaviour unchanged.
- Do not start F8-FMT.

## Acceptance criteria

1. OCaml command evidence reconciled — proven
2. Evidence/worker-note OCaml contradiction eliminated — proven by diff
3. Checkpoint terminology documented — proven
4. `suspicious_open_options` reclassified to INTENT REVIEW — proven
5. Zero production/build/test/fixture changes — proven by git diff from base

## Required verification

- `git status --short`: clean
- `git diff --check`: PASS
- `git diff --name-only 5f98c31f4bf51b806222c7f3722997d74fbe5a5b..HEAD`: documentation only
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`: PASS

## Relevant components

### CLOSEOUT
- `docs/foundation-pass/WARNING_TOOLING_RECONCILIATION_F8A.md` — repaired evidence
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-f8a-warning-tooling-reconciliation.md`

## Forbidden changes

- No production code modifications
- No Rust changes
- No OCaml changes
- No test modifications
- No fixture modifications
- No build file modifications
- No script modifications
- No formatting
- No F8-FMT implementation
- No F8 cleanup packages

## Stop conditions

STOP if the audit demonstrates an actual current production correctness defect.

## Expected pre-existing changes

None — this evidence-only task starts from the exact base commit
`5f98c31f4bf51b806222c7f3722997d74fbe5a5b` with a clean tree.
