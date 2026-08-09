# Current Implementation Task

Control contract: `1`
Task packet: `F8-T15 — Final Test Dead-Code Warning Cleanup`
Owner: `OpenCode`
Status: `IN_PROGRESS`
Task colour: `Green`
Route: `OpenCode removes last F8 test dead-code item`
Worker note: `docs/worker-notes/2026-08-09-f8-t15-test-warning-cleanup.md`
Base branch: `foundation/f8-elapsed-evidence`
Base commit: `9d19bc5e121d00da65b27d167183a7c6fe99e5b0`
Implementation branch: `foundation/f8-t15-test-warning-cleanup`
Implementation checkpoint: `db6dbcc76cd24856324ca2bcdbd0737d67318abd`
Rust change class: `RUST`

## Objective

Remove the final known F8 test dead-code item: `FailingResultAnchorWriter` from
`tethers-0.1/host-rust/src/application.rs`.

## Relevant background and existing behaviour

`FailingResultAnchorWriter` is a `#[cfg(test)]` struct with a
`ResultAnchorWriter` implementation that always returns `Err(())`. It was used
to prove that a failed Anchor write enqueues nothing. It is never constructed
in any test or production code. It contributes one `dead_code` warning (T15),
the last remaining F8 test dead-code item.

## Required behaviour

1. Search repository for `FailingResultAnchorWriter` references.
2. Prove the struct and its impl are the only code occurrences.
3. Delete the struct and its impl.
4. Run `cargo fmt` on the changed file only.
5. Confirm zero `FailingResultAnchorWriter` occurrences in Rust source.
6. Confirm the T15 warning is gone from `cargo check`.
7. Run full `just verify-agent` once.

## Frozen decisions and invariants

- Do not touch production dead-code D1-D15.
- Do not add `#[allow(...)]` suppression.
- Do not rename or refactor.
- Do not change any test.
- Preserve all runtime, Result Anchor, approval, event admission, queue, Trail,
  replay/recovery, CLI, JSON/protocol, and compatibility behaviour.

## Acceptance criteria

1. Pre-change search proves T15 genuinely unused in code.
2. `FailingResultAnchorWriter` declaration removed.
3. Its implementation removed.
4. No replacement suppression added.
5. No tests weakened.
6. No production semantics changed.
7. T15 warning absent afterward.
8. `cargo fmt` only touches `application.rs`.
9. `just verify-agent` passes once.
10. Branch pushed and local == remote.

## Required verification

- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`
- `git diff --check`
- Packet checker
- `just verify-agent` (full regression)
- Repository search for `FailingResultAnchorWriter` returns zero Rust source matches

## Relevant components

### AUTHORISED PATHS
- `tethers-0.1/host-rust/src/application.rs`

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-f8-t15-test-warning-cleanup.md`

## Forbidden changes

- No other Rust source changes
- No OCaml source changes
- No test changes
- No Nextest configuration changes
- No CI changes
- No dependency policy changes
- No tool version changes
- No `#[allow(...)]` suppression additions
- No production dead-code cleanup

## Stop conditions

STOP if `FailingResultAnchorWriter` has any live code use.
STOP if rustfmt touches any file other than `application.rs`.
STOP if verification fails.
STOP if two materially similar implementation attempts fail.

## Expected pre-existing changes

None.
