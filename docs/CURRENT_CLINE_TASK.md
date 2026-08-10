# Current Implementation Task

Control contract: `1`
Task: `TETHERS-0.3-P2B-FIX — Cleanup Authority + Task Closeout`
Owner: `OpenCode`
Status: `IN_PROGRESS`
Task colour: `Amber`
Route: `OpenCode implements bounded cleanup correction`
Worker note: `docs/worker-notes/2026-08-10-0.3-p2b-fix-cleanup-authority.md`
Base branch: `feature/0.3-p2b-public-plug-conform`
Base commit: `532550c296efb6384c67023efeca63bac26a7bdd`
Implementation branch: `feature/0.3-p2b-fix-cleanup-authority`
Implementation checkpoint: `WORKTREE`
OCaml switch path: `not applicable`
Rust toolchain: `1.97.1`
Rust change class: `PRODUCTION_AND_TEST`

## Objective

Close the final P2B acceptance gaps found during Lucy's independent review.
Do NOT redesign public conform. Do NOT begin P2C.

## Required behaviour

### FIX 1 — Cleanup failure must block clean success

1. Always attempt supervised scratch cleanup after conformance execution.
2. Always attempt final whole-workspace cleanup regardless of conform pass/fail/interruption/scratch-cleanup failure.
3. Capture cleanup failures.
4. If either scratch or workspace cleanup fails, return `conformance_cleanup_failed` / `failed` / exit 6.
5. Safe message: "ephemeral conform state could not be completely removed"
6. No path, temp dir, or raw filesystem error exposure in public output.
7. Cleanup failure takes precedence over an otherwise successful conform result.

### FIX 2 — Current task packet

Replace stale P1-R1G-RERUN task packet with actual bounded P2B-FIX task.

### FIX 3 — Run three required commands

- `cargo test --locked --lib conformance::tests`
- `cargo test --locked --lib launch_profile::tests`
- `cargo test --locked --lib installation_trust::tests`

Report PASS — 0 matched if applicable. Do NOT report NOT RUN.

## Frozen decisions and invariants

1. No conformance case changes
2. No supervised launch semantics changes
3. No exact-candidate trust changes
4. No public approval semantics changes
5. No dependencies added
6. No P2C work

## Authorised paths

- `src/plug_conform.rs` (production change)
- `tests/p2b_plug_conform_cli.rs` (existing, no changes expected)
- `docs/CURRENT_CLINE_TASK.md` (update)
- `docs/worker-notes/2026-08-10-0.3-p2b-fix-cleanup-authority.md` (new)
- `docs/worker-notes/2026-08-10-0.3-p2b-public-plug-conform.md` (optional addendum)

## Acceptance criteria

1. Cleanup failure overrides conformance success: `conformance_cleanup_failed` / `failed` / exit 6
2. Cleanup failure message is safe: no path, no temp dir, no raw error
3. Both scratch and workspace cleanup attempted regardless of failures
4. Focused unit tests prove all three outcome classes
5. `cargo test --locked --test p2b_plug_conform_cli` — 13/13 PASS
6. `cargo test --locked --test p2a_plug_pack_cli` — 7/7 PASS
7. `cargo test --locked --test j24a_plug_inspect_cli` — 3/3 PASS
8. `cargo clippy --all-targets --all-features --locked` — 0 new warnings
9. `cargo fmt --all -- --check` — PASS
10. `git diff --check` — PASS
11. Task packet checker `control-v1/COMPLETE`
12. Branch pushed, remote == local, genuinely clean worktree

## Required verification

1. Full focused test suite as listed above
2. Task packet checker — `control-v1/COMPLETE`
3. `git push` + remote SHA + local == remote + clean status

## Forbidden changes

- No conformance case changes
- No supervised launch semantics changes
- No exact-candidate trust changes
- No public approval semantics changes
- No dependencies
- No P2C work
- No `just verify-agent`

## Stop conditions

- Any mandatory gate fails
- Production or test changes needed beyond `src/plug_conform.rs`
- After two materially similar failed attempts

## Expected pre-existing changes

None. HEAD equals `532550c296efb6384c67023efeca63bac26a7bdd`. Working tree currently has uncommitted P2B-FIX changes.
