# Current Implementation Task

Control contract: `1`
Task: `F8a — Current Warning and Tooling Reconciliation`
Owner: `OpenCode`
Model: `DeepSeek Pro HIGH`
Status: `IN_PROGRESS`
Task colour: `Amber`
Route: `OpenCode performs evidence-only warning and tooling audit; no production changes`
Worker note: `docs/worker-notes/2026-08-09-f8a-warning-tooling-reconciliation.md`
Base branch: `foundation/f8a-warning-tooling-reconciliation`
Base commit: `5ecf54e17752096e7c553e059d014ef263cbb136`
Implementation branch: `foundation/f8a-warning-tooling-reconciliation`
Implementation checkpoint: `TO BE SET`
OCaml switch path: `N/A (no switch set)`
Rust toolchain: `1.97.1`

## Relevant background and existing behaviour

F1 originally recorded a large Clippy warning inventory. F1-R1 later observed at
accepted F5: cargo check 16 warnings, cargo clippy 81 warnings, cargo fmt FAIL
at `replay_windows.rs` ~line 3277, and `just verify`/`verify-agent` short-circuited
at formatting. These numbers are historical only and may have changed.

The project has no active OCaml switch configured. Rust toolchain is pinned to
1.97.1 via `rust-toolchain.toml`. No CI/workflow warning enforcement currently
exists. No `[lints]` configuration exists in Cargo.toml.

## Objective

Establish the exact current warning, formatting, and verification-tooling state
before any F8 cleanup. This is EVIDENCE-ONLY.

Do not fix warnings. Do not format files. Do not alter production code. Do not
alter tests. Do not alter fixtures. Do not alter scripts/tooling. Do not add
warning denial. Do not add CI enforcement. Do not start F8b.

## FOUNDATION F8 CONTRACT

F8 must:
1. reconcile the live warning/tooling inventory;
2. remove or explicitly justify warnings in bounded cleanup work;
3. reach zero INTENDED warnings;
4. record a documentation-only checkpoint proving that state;
5. only AFTER that, in a separate bounded change, activate warning denial /
   CI/tooling enforcement.

Never combine warning repair with gate activation.

## Required behaviour

1. Determine what exact warnings exist now.
2. Separate warnings by command: cargo check, cargo clippy, tests/builds.
3. Classify every distinct warning site: ACTIONABLE CLEANUP, JUSTIFIED WARNING,
   STALE/NO LONGER PRESENT, TOOLING/CONFIGURATION ISSUE, UNVERIFIED.
4. Group repeated warnings by root cause.
5. Determine exact cargo fmt failure: file, region, whether rustfmt would make
   formatting-only changes, whether any semantic/source interaction makes it
   unsafe to treat as simple formatting.
6. Determine current behaviour of just verify and just verify-agent.
7. Inspect warning/tooling configuration (read-only): Cargo.toml lint config,
   workspace lint settings, rustfmt config, clippy config, justfile verification
   commands, CI/workflow warning enforcement.
8. Identify warnings whose cleanup would require: public API change,
   protocol/Trail/replay change, visibility widening, structural redesign,
   dependency change, test weakening.
9. Produce the smallest serial F8 cleanup packages, if any.
10. Decide whether existing formatting failure should be its own tiny package,
    bundled, or deferred.

## Frozen decisions and invariants

- No production code changes.
- No Rust changes.
- No OCaml changes.
- No test changes.
- No fixture changes.
- No build changes.
- No script/tooling changes.
- No formatting changes.
- No warning denial additions.
- No CI enforcement additions.
- No F8b work.
- Do not turn Clippy preferences into architecture mandates.

## Acceptance criteria

1. Full command-result table in evidence document — proven
2. Warning counts by command — proven
3. Distinct warning/root-cause inventory with classifications — proven
4. Current rustfmt failure characterization — proven
5. just verify / verify-agent behaviour recorded — proven
6. Configuration inventory (read-only) — proven
7. Protected contracts identified — proven
8. Proposed bounded F8 packages — proven
9. Explicit non-authorisations — proven
10. Audit checkpoint committed — proven by git log

## Required verification

- All required commands run; results captured regardless of pass/fail
- Evidence document exists at `docs/foundation-pass/WARNING_TOOLING_RECONCILIATION_F8A.md`
- Worker note created
- Task packet checker passes at checkpoint and at completion
- git diff from base shows documentation only

## Relevant components

### EVIDENCE-ONLY
- `docs/foundation-pass/WARNING_TOOLING_RECONCILIATION_F8A.md`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-f8a-warning-tooling-reconciliation.md`

### READ-ONLY INSPECTION
- `justfile`
- `tethers-0.1/host-rust/Cargo.toml`
- `rust-toolchain.toml`
- CI/workflow files

## Forbidden changes

- No production code modifications
- No Rust changes
- No OCaml changes
- No test modifications
- No fixture modifications
- No build file modifications
- No script modifications
- No formatting
- No warning denial or CI enforcement additions

## Stop conditions

STOP if the audit demonstrates an actual current production correctness defect.
Flag Lucy instead.

## Expected pre-existing changes

None — this evidence-only task starts from the exact base commit
`5ecf54e17752096e7c553e059d014ef263cbb136` with a clean tree.
