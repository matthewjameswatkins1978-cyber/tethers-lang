# Current Implementation Task

Control contract: `1`
Task: `F1-R1 — Missing Performance Baseline Reconciliation`
Owner: `OpenCode`
Model: `DeepSeek Pro HIGH`
Status: `COMPLETE`
Task colour: `Amber`
Route: `OpenCode measures historical and current-F5 build/test timings; no production changes; evidence only`
Worker note: `docs/worker-notes/2026-08-08-f1-r1-performance-baseline.md`
Base branch: `foundation/f5-ocaml-boundaries`
Base commit: `ea7426dbeb1934cf336673d03ae2abf76146ea7d`
Implementation branch: `foundation/f1-r1-performance-baseline`
Implementation checkpoint: `41242a45963c8a7f751b6f6a93f1ad3fe2ae7320`
OCaml switch path: `N/A`
Rust toolchain: `1.97.1`

## Objective

Produce reproducible performance/operational-cost measurements sufficient to decide whether either F1 performance hypothesis becomes an actual F6 optimisation candidate. No optimisation, no production changes, no test changes, no fixture changes, no dependency additions.

## Relevant background and existing behaviour

The F1 baseline reported two unmeasured F6 optimisation candidates:
- **P1:** `application.rs` compile-time hypothesis (large file may slow compilation)
- **P2:** `result_large_err` hypothesis (Clippy reports ~160+ byte Err variant)

Both were classified as unmeasured hypotheses requiring measurement before F6 could begin.

## Required behaviour

1. Establish separate detached worktrees for historical baseline (`24428139`) and current F5 (`ea7426d`).
2. Collect cold and warm timings for `cargo check`, `cargo test`, `cargo clippy`, `just verify`, `just verify-agent`.
3. Record machine environment (OS, CPU, RAM, filesystem, Rust/Cargo/PowerShell versions, cargo cache state).
4. Gather P1 evidence: application.rs line counts, cold/warm check timings, attribute or note inability to attribute per-file compile cost.
5. Gather P2 evidence: locate `result_large_err` sites, note whether still present at F5, classify hot vs cold path.
6. Classify each hypothesis: MEASURED COST, UNATTRIBUTED COST, UNMEASURED HYPOTHESIS, NO MATERIAL COST, or UNVERIFIED.
7. Produce F6 authorisation table.
8. No production/build/test/fixture file changes.
9. Document results in `docs/foundation-pass/PERFORMANCE_BASELINE_R1.md`.

## Relevant components

### NEW
- `docs/foundation-pass/PERFORMANCE_BASELINE_R1.md` — performance evidence document

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-08-f1-r1-performance-baseline.md`

## Frozen decisions and invariants

- No production code changes.
- No test changes.
- No fixture changes.
- No dependency additions (no benchmarking crates, no unstable compiler features).
- Measurement worktrees are temporary; no repository mutations to either measurement target.
- Rust toolchain locked at 1.97.1.

## Acceptance criteria

1. Historical timings collected for cargo check, cargo test, cargo clippy, just verify, just verify-agent — proven by raw timing table
2. Current-F5 timings collected for cargo check, cargo test, cargo clippy, just verify, just verify-agent — proven by raw timing table
3. Environment recorded — proven by environment table
4. P1 evidence gathered (application.rs line counts, cold/warm timings) — proven
5. P2 evidence gathered (result_large_err sites, hot/cold path assessment) — proven
6. Each hypothesis classified with honest causal limits — proven
7. F6 authorisation table produced — proven
8. Zero production/build/test/fixture changes — proven by git diff
9. PERFORMANCE_BASELINE_R1.md exists with complete evidence — proven

## Required verification

- `git diff --check`: PASS
- `check-tethers-task-packet.ps1`: PASS
- `git diff --name-only -- tethers-0.1/host-rust/`: (empty)
- `git diff --name-only -- tethers-0.1/engine-ocaml/`: (empty)
- `git diff --name-only -- tethers-0.1/protocol/`: (empty)
- `git diff --name-only HEAD~1..HEAD`: only authorised closeout files

## Forbidden changes confirmed not made

- No production code modifications
- No test modifications
- No fixture modifications
- No build file modifications
- No dependency additions
- No benchmarking crate installations
- No refactoring to test hypotheses
- No F6 optimisation work

## Stop conditions

NONE triggered.

## Expected pre-existing changes

None. All changes are documentation/evidence only. Implementation checkpoint is WORKTREE because this task produces no implementation commits.
