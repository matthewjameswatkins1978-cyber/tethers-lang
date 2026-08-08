# Current Implementation Task

Control contract: `1`
Task: `F7a-R1 — Evidence Repair`
Owner: `OpenCode`
Model: `DeepSeek Pro HIGH`
Status: `IN_PROGRESS`
Task colour: `Amber`
Route: `OpenCode repairs F7a evidence defects only; no production changes`
Worker note: `docs/worker-notes/2026-08-08-f7a-r1-evidence-repair.md`
Base branch: `foundation/f7a-test-contract-reconciliation`
Base commit: `532126810ad51dfbf6d75472854c9cb49d8d0811`
Implementation branch: `foundation/f7a-r1-evidence-repair`
Implementation checkpoint: `N/A`
OCaml switch path: `N/A`
Rust toolchain: `1.97.1`

## Objective

Repair F7a evidence/closeout defects only. No production, test, fixture, build, protocol, script, or dependency changes.

## Relevant background and existing behaviour

Lucy has independently reviewed F7a. F7 DECISION: NO TEST CONSOLIDATION AUTHORISED. M7 is DEFERRED. F7 will close as NO-OP if this evidence repair passes.

F7a had three evidence defects:
1. The fixture verification path was incorrectly recorded as `scripts/check-fixtures.ps1` — correct path is `tethers-0.1/scripts/check-fixtures.ps1`.
2. The all-features failure analysis claimed the six F1-R1 failures were "resolved by intervening Foundation work" — the correct finding is "CURRENTLY NOT REPRODUCED — PRIOR CAUSE UNVERIFIED" since `ea7426d..2a2417f5` contains only documentation changes.
3. The F7 authorisation table recommended F7b for limited OCaml native tests — Lucy's decision is M7 DEFER, NO F7b AUTHORISED.

## Required behaviour

1. Prepare F7a-R1 repaired evidence document.
2. Correct fixture verification path everywhere.
3. Correct all-features failure analysis to "CURRENTLY NOT REPRODUCED — PRIOR CAUSE UNVERIFIED".
4. Record F7a final HEAD `532126810ad51dfbf6d75472854c9cb49d8d0811` in the evidence document.
5. Amend F7 authorisation: M7 DEFER, NO F7b AUTHORISED, F7 COMPLETES AS NO-OP.

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
- No F7b implementation.
- No F8 work.
- F7 DECISION: NO TEST CONSOLIDATION AUTHORISED. M7 DEFERRED.
- The `ea7426d..2a2417f5` range contains documentation only — no production/test change can causally explain changed all-features results.

## Acceptance criteria

1. Fixture verification path corrected everywhere — proven by grep
2. All-features failure analysis corrected to UNVERIFIED — proven
3. Final HEAD recorded in evidence document — proven
4. F7 authorisation amended: M7 DEFER, NO F7b — proven
5. Task packet is a valid F7a-R1 packet — proven by checker
6. Zero production/build/test/fixture changes — proven by git diff

## Required verification

- `git status --short`: clean
- `cargo test --locked`: PASS
- `cargo test --all-targets --all-features --locked`: PASS
- `pwsh -NoProfile -File tethers-0.1/scripts/check-fixtures.ps1`: PASS
- `git diff --check`: PASS
- `git diff --name-only 532126810ad51dfbf6d75472854c9cb49d8d0811..HEAD`: documentation only
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`: PASS at R1 checkpoint
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`: PASS at COMPLETE

## Relevant components

### CLOSEOUT
- `docs/foundation-pass/TEST_CONTRACT_RECONCILIATION_F7A.md` — repaired evidence
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-08-f7a-r1-evidence-repair.md`

## Forbidden changes

- No production code modifications
- No Rust changes
- No OCaml changes
- No test modifications
- No fixture modifications
- No build file modifications
- No protocol changes
- No script modifications
- No dependency additions
- No F7b implementation work
- No F8 work

## Stop conditions

STOP if the audit demonstrates an actual current production correctness defect.

## Expected pre-existing changes

None — this evidence-only task starts from the exact base commit `532126810ad51dfbf6d75472854c9cb49d8d0811` with a clean tree.
