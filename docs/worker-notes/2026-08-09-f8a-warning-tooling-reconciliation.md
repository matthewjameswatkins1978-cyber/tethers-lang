# Worker Note — F8a Warning and Tooling Reconciliation

Task: `F8a — Current Warning and Tooling Reconciliation`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `5ecf54e17752096e7c553e059d014ef263cbb136`
Implementation checkpoint: `74904309d9af04024cd1a0b60c4cf654b8617481`

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

- **No OCaml switch:** Recorded as "TOOLING/CONFIGURATION ISSUE" rather than a blocker. The OCaml tooling state is documented; F8 is Rust-focused.
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

Commands NOT run (dependencies failed):
- `test-engine.ps1` — depends on `dune build` (no OCaml switch)
- `just verify-agent` sub-tools — unreachable due to fmt failure in verify
- `opam exec -- dune build` — no switch set

## Discoveries

1. **No CI/workflow enforcement exists.** No `.github/workflows/` directory. No automated warning gates.
2. **No `[lints]` configuration in Cargo.toml.** Warnings are entirely driven by default compiler/Clippy settings.
3. **No rustfmt.toml or clippy.toml exists.** All formatting and linting uses defaults.
4. **Single fmt failure site.** Only `replay_windows.rs:3277` — a whitespace-only change in test code. Everything else is properly formatted.
5. **verify/verify-agent are fragile.** A single formatting failure blocks the entire pipeline. The fmt check should ideally be a separate gate or `verify` should continue and report, but this is a design choice for F8b.
6. **OCaml tooling is unavailable.** Not needed for F8 Rust cleanup but recorded for completeness.
7. **33 check warnings vs historical 16.** The count has grown since F5, likely due to accumulated dead code from Foundation work.
8. **81 raw Clippy emissions vs historical 81.** The raw count is unchanged from F5, but distinct count is 45 (36 duplicates from shared compilation units).

## Remaining risks

- **verify-agent sub-tools state unknown:** `cargo nextest`, `cargo deny`, `cargo machete` not exercised. Expected to work but unverified.
- **OCaml engine tests not run:** No switch available. Recorded as TOOLING/CONFIGURATION.
- **Production dead code (D1-D15):** Removal of `pub(crate)` items requires per-item judgement. May break external callers or future features. Not authorised in F8a.
- **items_after_test_module (P5):** Code reordering in `candidate_preparation.rs` and `installation_publication_mutation.rs` could introduce merge conflicts if deferred.

## Smallest next action

Apply F8-FMT: single `cargo fmt` on `replay_windows.rs:3277`. This unblocks `just verify` and `just verify-agent`, enabling all subsequent F8 cleanup packages to be verified end-to-end.

## References

- Evidence document: `docs/foundation-pass/WARNING_TOOLING_RECONCILIATION_F8A.md`
- Task packet: `docs/CURRENT_CLINE_TASK.md`
- Base: `5ecf54e17752096e7c553e059d014ef263cbb136`
- Audit checkpoint: `74904309d9af04024cd1a0b60c4cf654b8617481`
- Branch: `foundation/f8a-warning-tooling-reconciliation`
