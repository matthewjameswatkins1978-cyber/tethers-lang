# Current Implementation Task

Control contract: `1`
Task packet: `F8-D11 — Obsolete authorise_and_execute Wrapper`
Owner: `Codex`
Status: `IN_PROGRESS`
Task colour: `Amber`
Route: `Codex classified the unused one-shot authorisation wrapper before removal`
Worker note: `docs/worker-notes/2026-08-09-f8-d11-authorise-wrapper-cleanup.md`
Base branch: `foundation/f8-d7-d8-d9-local-notification-cleanup`
Base commit: `b1bf419223a154c7f1094d1d5dd64a352095a6c4`
Implementation branch: `foundation/f8-d11-authorise-wrapper-cleanup`
Implementation checkpoint: `PENDING`
OCaml switch path: `N/A`
Rust toolchain: `1.97.1`
Rust change class: `RUST`

## Objective

Remove only the obsolete D11 one-shot `authorise_and_execute` wrapper while
retaining the live writer-aware path, test adapters, shared execution boundary,
and all authorization, replay, Result Anchor, dispatch, and zero-call failure
contracts.

## Relevant background and existing behaviour

The Job B closeout at `b1bf419223a154c7f1094d1d5dd64a352095a6c4` left five
production-library warnings: D11-D15. The current J10 event coordinator calls
`authorise_and_execute_with_writer`, which creates the production clock and
file replay authority before delegating to `authorise_and_execute_inner`. The
D11 wrapper independently performs the same one-shot setup, but complete Rust
searches find no caller. Existing `#[cfg(test)]` adapters directly exercise the
shared inner boundary with explicit replay or bridge-pin choices.

## Required behaviour

1. Remove D11 `authorise_and_execute` without replacement or suppression.
2. Retain `authorise_and_execute_with_writer`,
   `authorise_and_execute_inner`, the explicit test adapters, and their
   existing callers.
3. Preserve replay admission, Result Anchor generation, dispatch-proof
   boundary, and executor zero-call-on-failure guarantees.
4. Reduce the intended production-library warning count from five to four,
   leaving D12-D15 unresolved.
5. Run focused execution-boundary checks and exactly one final
   `just verify-agent` after the implementation checkpoint.

## Relevant components

### AUTHORISED PATHS
- `tethers-0.1/host-rust/src/application.rs` — remove D11 only

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-f8-d11-authorise-wrapper-cleanup.md`

## Frozen decisions and invariants

- D11 is dead only because its exact Rust caller search is empty; the live
  writer-aware J10 route remains retained.
- Preserve all current `_with_writer`, `_inner`, test-replay, and
  without-bridge-pins adapter behavior.
- No unrelated execution refactor, D12-D15 cleanup, dead-code suppression,
  protocol change, dependency/toolchain change, or source outside the
  authorised Rust path.

## Acceptance criteria

1. Exact Rust search has zero `fn authorise_and_execute(` definition/caller
   matches after removal.
2. The retained writer-aware and inner execution seams remain reachable from
   their existing production and test callers.
3. Focused execution-boundary tests preserve dispatch, replay, Result Anchor,
   and failure zero-call evidence.
4. Full-target locked cargo check reports exactly four remaining
   production-library warnings and no D11 warning.
5. Formatter, whitespace check, and Clippy pass; the one final umbrella
   verification passes after the implementation checkpoint.

## Required verification

1. Exact Rust searches for D11 and all retained authorisation seams.
2. Named focused `authorise_and_execute` / J10 tests selected from
   `application.rs`.
3. Full-target locked `cargo check`, formatter diff/check, `git diff --check`,
   and Clippy.
4. One final `just verify-agent`, then complete range-diff, remote equality,
   and clean-status evidence.

## Forbidden changes

- No removal or modification of the writer-aware, inner, or explicit test
  execution seams; no D12-D15 cleanup or lint suppression.
- No OCaml, fixture, protocol, dependency, CI, lint-policy, merge, amend,
  tag, force-push, direct `main`, or pull-request change.

## Stop conditions

STOP if an actual caller or sole live-contract representation is found, a test
migration would weaken evidence, an architectural choice is required,
formatter output leaves the authorised Rust path, or verification is
untrustworthy. Do not begin Job D without a verified and pushed Job C tip.

## Expected pre-existing changes

None.
