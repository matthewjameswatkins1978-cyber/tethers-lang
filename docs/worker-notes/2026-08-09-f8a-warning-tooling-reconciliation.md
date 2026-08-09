# Worker Note — F8a Warning and Tooling Reconciliation

Task: `F8a — Current Warning and Tooling Reconciliation`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `5ecf54e17752096e7c553e059d014ef263cbb136`
Implementation checkpoint: `5f98c31f4bf51b806222c7f3722997d74fbe5a5b`

## Requested outcome

Evidence-only audit of current warning, formatting, and tooling state. No production changes. All commands run against unchanged source at HEAD `5ecf54e`.

## Changes made

1. Created branch `foundation/f8a-warning-tooling-reconciliation` from base `5ecf54e`.
2. Wrote F8a task packet (`docs/CURRENT_CLINE_TASK.md`) with full F8 contract and required sections.
3. Committed as audit checkpoint (`cb0e151`).
4. Ran all required diagnostic commands (cargo check, test, clippy, fmt, just verify, just verify-agent, test-mcp-transcripts, check-fixtures, git diff --check, packet checker).
5. Wrote evidence document (`docs/foundation-pass/WARNING_TOOLING_RECONCILIATION_F8A.md`).
6. Wrote this worker note.

Zero production, test, fixture, build, script, or tooling source files changed.

## Decisions and assumptions

- **No OCaml switch:** F8a recorded "TOOLING/CONFIGURATION ISSUE" because `opam exec -- dune build` without `--switch` failed. F8a-R1 corrects: switch exists at `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`; `dune build` passes with explicit `--switch`; `test-engine.ps1` passes all 28 cases when switch is active.
- **fmt failure is single-site, formatting-only:** Verified the diff is whitespace-only in test code at `replay_windows.rs:3277`. Safe to fix as a standalone package.
- **permissions_set_readonly_false classified as JUSTIFIED:** This lint is Unix-specific; the 13 sites are all Windows test helpers that legitimately need writable permissions.
- **too_many_arguments / type_complexity / result_large_err classified as preferences:** Not defects. These represent honest domain signatures. Restructuring would be architectural work not authorised by F8.
- **F8-FMT recommended as first package:** Unblocks the verify/verify-agent pipeline for all subsequent work.
- **Historical F1/F5 numbers confirmed stale:** Check: 33 vs historical 16; Clippy distinct: 45 vs historical 81 raw.

## Evidence

1. `cargo check` — 33 warnings (15 lib + 18 test). All sites documented in §4.1-4.2.
2. `cargo test` — 1331 passed, 0 failed, 2 ignored. All green.
3. `cargo clippy` — 81 emitted (36 duplicates), 45 distinct. All classified in §4.3, §8.
4. `cargo fmt --check` — FAIL at `replay_windows.rs:3277`. Characterized in §5.
5. `just verify` — FAIL at fmt (step 2/4). Behaviour in §6.
6. `just verify-agent` — FAIL at verify (fmt). Behaviour in §6.
7. `test-mcp-transcripts.ps1` — PASS (15 cases).
8. `check-fixtures.ps1` — PASS (46 JSON, 30 JSONL).
9. `check-tethers-task-packet.ps1` — PASS.
10. `git diff --check` — PASS (clean).
11. Configuration inventory in §7 — no lint config, no CI workflows.
12. Protected contracts in §9 — identified items needing per-item judgement.
13. Proposed F8 packages in §10 — 5 packages + F8-FMT.

Commands NOT run (blocked by dependencies or environment):
- `just verify-agent` sub-tools — unreachable due to `cargo fmt` failure in `verify`
- `opam exec -- dune build` without `--switch` — fails because no global switch is set; passes with explicit `--switch` (F8a-R1 correction)

Commands re-run in F8a-R1:
- `opam exec --switch "D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" -- dune build` — PASS
- `test-engine.ps1` (with switch active) — PASS (28 cases)

## Discoveries

1. **No CI/workflow enforcement exists.** No `.github/workflows/` directory. No automated warning gates.
2. **No `[lints]` configuration in Cargo.toml.** Warnings are entirely driven by default compiler/Clippy settings.
3. **No rustfmt.toml or clippy.toml exists.** All formatting and linting uses defaults.
4. **Single fmt failure site.** Only `replay_windows.rs:3277` — a whitespace-only change in test code. Everything else is properly formatted.
5. **verify/verify-agent are fragile.** A single formatting failure blocks the entire pipeline. The fmt check should ideally be a separate gate or `verify` should continue and report, but this is a design choice for F8b.
6. **OCaml engine is available** with explicit switch `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`. F8a incorrectly recorded it as unavailable. F8a-R1 corrects: `dune build` passes, `test-engine.ps1` passes 28/28.
7. **33 check warnings vs historical 16.** The count has grown since F5, likely due to accumulated dead code from Foundation work.
8. **81 raw Clippy emissions vs historical 81.** The raw count is unchanged from F5, but distinct count is 45 (36 duplicates from shared compilation units).

## Remaining risks

- **verify-agent sub-tools state unknown:** `cargo nextest`, `cargo deny`, `cargo machete` not exercised. Expected to work but unverified.
- **OCaml engine:** Available but requires explicit `--switch`. `test-engine.ps1` passes all 28 cases. F8a-R1 corrects this.
- **Production dead code (D1-D15):** Removal of `pub(crate)` items requires per-item judgement. May break external callers or future features. Not authorised in F8a.
- **items_after_test_module (P5):** Code reordering in `candidate_preparation.rs` and `installation_publication_mutation.rs` could introduce merge conflicts if deferred.

## Smallest next action

Apply F8-FMT: single `cargo fmt` on `replay_windows.rs:3277`. This unblocks `just verify` and `just verify-agent`, enabling all subsequent F8 cleanup packages to be verified end-to-end.

## References

- Evidence document: `docs/foundation-pass/WARNING_TOOLING_RECONCILIATION_F8A.md`
- Task packet: `docs/CURRENT_CLINE_TASK.md`
- F8a-R1 evidence repair: `foundation/f8a-r1-evidence-repair`
- Base: `5ecf54e17752096e7c553e059d014ef263cbb136`
- Audit checkpoint (F8a): `5f98c31f4bf51b806222c7f3722997d74fbe5a5b`
- F8a branch: `foundation/f8a-warning-tooling-reconciliation`
