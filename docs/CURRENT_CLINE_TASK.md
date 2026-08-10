# Current Implementation Task

Control contract: `1`
Task: `TETHERS-0.3-P1-R1G-RERUN — Final P1 Acceptance Gate (Rerun)`
Owner: `OpenCode`
Status: `IN_PROGRESS`
Task colour: `Amber`
Route: `OpenCode runs final P1 verification and closeout`
Worker note: `docs/worker-notes/2026-08-10-0.3-p1-r1g-final-gate.md`
Base branch: `feature/0.3-p1-r1g-fix-j23c3-scope-digest`
Base commit: `a0bdead29b89f76b41f3350d014e02f5f060e9a9`
Implementation branch: `feature/0.3-p1-r1g-final-gate-rerun`
Implementation checkpoint: `unset — gate in progress`
OCaml switch path: `not applicable`
Rust toolchain: `1.97.1`
Rust change class: `VERIFICATION_AND_CLOSEOUT_ONLY`

## Objective

Rerun the final P1 acceptance gate after the bounded J23C3 stale-test correction (R1G-FIX). Verification + closeout only. No implementation changes.

## Relevant background

- First R1G run was BLOCKED on stale J23C3 assertion (`assert_eq!` at line 226 expected equal digests for different scope content)
- R1G-FIX corrected two stale expectations: `assert_eq!` → `assert_ne!` and "enablement scope does not match supplied scope" → "enablement pins are stale"
- R1G-FIX accepted at `a0bdead29b89f76b41f3350d014e02f5f060e9a9`
- This rerun gates the same acceptance criteria from the corrected base

## Required behaviour

1. Verify clean starting state (no untracked file from previous BLOCKED run)
2. No-knowledge gate: zero generic-provider references in production `src/`
3. Retired-delivery gate: zero retired subject-specific paths in generic production
4. Dependency gate: no dependency change vs `c0fd57780156bee023d8dcff884737ea470d096c`
5. `cargo clippy --all-targets --all-features --locked` — PASS
6. `just verify-agent` — PASS (full suite)
7. `git diff --check` — PASS
8. Reconcile 14 P1 acceptance criteria
9. Closeout: worker note, task packet COMPLETE, push, remote == local, genuinely clean worktree

## Reused valid evidence

- Engine fixtures: 29 PASS (no OCaml files changed during P1)
- MCP transcripts: 15 PASS (R1G-FIX changed only assertions/messages in J23C3 + docs)
- Fixture validator: 46 JSON + 30 JSONL PASS (no fixture data or production behaviour changed)

## Frozen decisions and invariants

1. No production code changes
2. No test changes
3. No implementation
4. P1 only — no P2

## Relevant components

### Authorised paths

- `docs/CURRENT_CLINE_TASK.md` (update)
- `docs/worker-notes/2026-08-10-0.3-p1-r1g-final-gate.md` (new, replaces removed BLOCKED version)
- P1/goal/dashboard documentation

## Acceptance criteria

1. All 8 mandatory gates PASS
2. 14/14 P1 criteria reconciled YES
3. Task packet checker `control-v1/COMPLETE`
4. Branch pushed, local == remote, genuinely clean worktree (no modified, staged, or untracked files)

## Required verification

1. `git diff --check` — PASS
2. Task packet checker — `control-v1/COMPLETE`
3. `git push` + remote SHA + local == remote + clean status

## Forbidden changes

- No production code changes
- No test changes
- No implementation of any kind
- No P2 work

## Stop conditions

- Any mandatory gate fails
- Production or test changes needed
- After two materially similar failed attempts

## Expected pre-existing changes

None. HEAD must equal `a0bdead29b89f76b41f3350d014e02f5f060e9a9`. Working tree must be genuinely clean.
