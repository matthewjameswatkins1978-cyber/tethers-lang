# Current Implementation Task

Control contract: `1`
Task: `F7a — Current Test Contract Reconciliation`
Owner: `OpenCode`
Model: `DeepSeek Pro HIGH`
Status: `IN_PROGRESS`
Task colour: `Amber`
Route: `OpenCode audits test/debt inventory against current Foundation state; evidence only`
Worker note: `docs/worker-notes/2026-08-08-f7a-test-contract-reconciliation.md`
Base branch: `foundation/f1-r1-performance-baseline`
Base commit: `2a2417f5d943a5c1ca27d6c646746cfaf7b93a86`
Implementation branch: `foundation/f7a-test-contract-reconciliation`
Implementation checkpoint: `N/A` (evidence-only; audit checkpoint to be committed)
OCaml switch path: `N/A`
Rust toolchain: `1.97.1`

## Objective

Reconcile the F1 test/debt inventory against the current accepted Foundation state before any F7 test-suite modification. This is an EVIDENCE-ONLY audit.

## Relevant background and existing behaviour

The F1 baseline audit identified three ledger debts:
- M7: No OCaml-native tests exist.
- M8: Test modules inside Rust `src/` — dedicated `*_tests.rs` files under `src/` blur the production/test boundary.
- M9: Test infrastructure had `pub(crate)` visibility at the time of F1.

Since F1, Foundation pass F5 introduced stable `.mli` boundaries for `Tether_parser`, `Tethers_error`, `Tethers_outcome`, and `Tethers_evaluator`. Intervening Rust host work restructured test module declarations. This audit reconciles the current state against each debt.

## Required behaviour

1. Determine which F1 F7 debts still exist.
2. Determine which have already been resolved by intervening Foundation work.
3. Determine which were physically true but are not actually actionable maintenance debt.
4. Identify which exact test properties are genuinely duplicated.
5. Identify which properties are only indirectly evidenced and might benefit from a direct OCaml-native test after F5.
6. Characterise current Rust test failures under `--all-features`.
7. Determine whether any concrete F7 implementation package is authorised.

DO NOT consolidate, move, add, delete, rename, or rewrite tests.

## Frozen decisions and invariants

- No production code changes.
- No test changes.
- No fixture changes.
- No dependency additions.
- No F7b implementation.
- No F8 work.
- Test accessibility never justifies widening production visibility.
- Internal tests belong at the appropriate private boundary.
- Public behaviour uses public surfaces.
- Literal compatibility fixtures are independently owned evidence.
- Fixture changes require an explicit compatibility decision.
- Preserve external JSON, CLI output, exit codes, Trail shape, replay digests, and recovery semantics.

## Acceptance criteria

1. M7 (no OCaml-native tests) reconciled — proven by OCaml module audit
2. M8 (test modules inside Rust src/) reconciled — proven by source inspection
3. M9 (pub(crate) test infrastructure) reconciled — proven by lib.rs inspection
4. Exact all-features failing tests identified and classified — proven by test run
5. Duplicate-candidate table built with named properties — proven
6. Protected evidence catalogued — proven
7. OCaml direct-test candidate table built — proven
8. F7 authorisation table produced — proven
9. Output document exists with complete evidence — proven
10. Zero production/build/test/fixture changes — proven by git diff

## Required verification

- `git status --short`: clean before closeout
- `cargo test --locked`: PASS (or honest FAIL report)
- `cargo test --all-targets --all-features --locked`: exact failures recorded
- `opam exec -- dune build`: PASS
- `opam exec -- dune runtest`: result recorded
- `pwsh -NoProfile -File tethers-0.1/scripts/test-engine.ps1`: result recorded
- `pwsh -NoProfile -File tethers-0.1/scripts/test-mcp-transcripts.ps1`: result recorded
- `pwsh -NoProfile -File scripts/check-fixtures.ps1`: result recorded
- `cargo fmt --all -- --check`: observation only (known failure)
- `git diff --check`: PASS
- `check-tethers-task-packet.ps1`: PASS
- `git diff --name-only -- tethers-0.1/host-rust/`: (empty)
- `git diff --name-only -- tethers-0.1/engine-ocaml/`: (empty)
- `git diff --name-only -- tethers-0.1/protocol/`: (empty)
- `git diff --name-only HEAD~1..HEAD`: only authorised closeout files

## Forbidden changes

- No production code modifications
- No test modifications
- No fixture modifications
- No build file modifications
- No dependency additions
- No F7b implementation work
- No F8 work
- No test consolidation/move/rename

## Stop conditions

STOP if the audit demonstrates:
- an actual current production correctness defect;
- a required test consolidation would need production visibility widened;
- a compatibility fixture would need changing;
- a language/protocol/Trail/replay semantic change;
- a dependency addition;
- a production/test/build modification.

## Expected pre-existing changes

None — this evidence-only task starts from the exact base commit with a clean tree; only task-packet and documentation updates are permitted after the audit checkpoint.

## Output document

`docs/foundation-pass/TEST_CONTRACT_RECONCILIATION_F7A.md`

## Relevant components

### CLOSEOUT
- `docs/foundation-pass/TEST_CONTRACT_RECONCILIATION_F7A.md` — reconciliation evidence
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-08-f7a-test-contract-reconciliation.md`
