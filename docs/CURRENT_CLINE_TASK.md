# Current Implementation Task

Control contract: `1`
Task: `F8-FMT — Single Formatting Repair`
Owner: `OpenCode`
Model: `DeepSeek Pro HIGH`
Status: `COMPLETE`
Task colour: `Green`
Route: `OpenCode applies single formatting-only rustfmt fix to replay_windows.rs`
Worker note: `docs/worker-notes/2026-08-09-f8-fmt-formatting-repair.md`
Base branch: `foundation/f8a-r1-evidence-repair`
Base commit: `bfb47ced813d8ec227f8828bbf66c7ecd1110d2f`
Implementation branch: `foundation/f8-fmt`
Implementation checkpoint: `109acae33aecd3c070b06bd42c8c45e2e35f5247`
OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`
Rust toolchain: `1.97.1`

## Relevant background and existing behaviour

F8a identified a single `cargo fmt --check` failure at
`tethers-0.1/host-rust/src/replay_windows.rs:3277`. The failure is a
whitespace-only formatting issue: a chained `.with_file_name(...).exists()`
assert expression exceeds the line length. rustfmt wants to split it across
lines. The change is semantically identical; no logic, control flow, or
behaviour changes.

This single failure blocks `just verify` (step 2/4) and `just verify-agent`
(verify is first dependency).

## Objective

Apply `cargo fmt` to fix the single formatting failure. Unblock the
verification pipeline.

## Required behaviour

1. Apply `cargo fmt` to `replay_windows.rs:3277` only.
2. Verify no other Rust source file is changed.
3. Commit the formatting repair as the implementation checkpoint.
4. Run post-repair verification: `cargo fmt --check`, `just verify`,
   `just verify-agent`.
5. Record results honestly. If a later verification step fails for a new
   reason, record it but do not repair it.

## Frozen decisions and invariants

- Change only `replay_windows.rs` — formatting only.
- No warning cleanup.
- No other Rust source changes.
- No test changes.
- No fixture changes.
- No build changes.
- No F8b gate activation.
- The rustfmt change is whitespace-only; zero semantic impact.

## Acceptance criteria

1. `cargo fmt --all -- --check` passes — proven
2. Only `replay_windows.rs` changed — proven by git diff
3. `just verify` passes or reaches a new step past fmt — proven
4. `just verify-agent` runs past verify — proven
5. Packet checker passes — proven
6. Zero other file changes — proven by git diff from base

## Required verification

- `cargo fmt --all -- --check`: PASS
- `just verify`: record result
- `just verify-agent`: record result and furthest step reached
- `git diff --check`: PASS
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`: PASS

## Relevant components

### PRODUCTION
- `tethers-0.1/host-rust/src/replay_windows.rs:3277` — single formatting fix

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-f8-fmt-formatting-repair.md`

## Forbidden changes

- No other Rust source modifications
- No warning cleanup
- No clippy --fix
- No formatting of other files
- No F8b work
- No F8-PACKAGE work

## Stop conditions

STOP if `cargo fmt` changes any Rust file other than `replay_windows.rs`.
STOP if any verification step reveals a pre-existing correctness defect.

## Expected pre-existing changes

None — starts from exact base `bfb47ced813d8ec227f8828bbf66c7ecd1110d2f`
with a clean tree.
