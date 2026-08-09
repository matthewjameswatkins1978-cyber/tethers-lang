# Worker Note

Task: `F8-D11 — Obsolete authorise_and_execute Wrapper`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `IN_PROGRESS`

Base commit: `b1bf419223a154c7f1094d1d5dd64a352095a6c4`

Implementation checkpoint: `PENDING`

## Requested outcome

Classify and remove the obsolete D11 one-shot wrapper only if the live
writer-aware and test execution seams retain every real contract.

## Changes made

- No implementation changes yet; packet and classification only.

## Decisions and assumptions

- Exact Rust search currently finds D11 only at its definition. The retained
  writer-aware production path has current J10 callers, and test adapters call
  the shared inner boundary directly.

## Evidence

- `rg "\\bauthorise_and_execute\\(" tethers-0.1/host-rust --type rust` found
  only the D11 definition plus a stale dispatch comment, not a production or
  test caller.

## Discoveries

- The D11 body duplicates the production clock, file replay authority, Result
  Anchor writer, and initial input context setup already represented by the
  live writer-aware seam.

## Remaining risks

- Focused execution-boundary tests must confirm no direct D11 test dependency
  was missed before source removal.

## Smallest next action

Remove D11 only and run the named boundary checks.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `tethers-0.1/host-rust/src/application.rs`
