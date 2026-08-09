# Worker Note

Task: `F8-D11 — Obsolete authorise_and_execute Wrapper`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `b1bf419223a154c7f1094d1d5dd64a352095a6c4`

Implementation checkpoint: `93e50b786e48949c2e9bf6649546618a148b48be`

## Requested outcome

Removed the obsolete D11 one-shot authorisation wrapper while retaining the
live writer-aware and explicitly testable execution seams.

## Changes made

- `tethers-0.1/host-rust/src/application.rs`
  - Removed only `authorise_and_execute` (D11).
  - Retained `authorise_and_execute_with_writer`,
    `authorise_and_execute_inner`, test replay, and no-bridge-pin adapters.
  - Did not change replay admission, Result Anchor handling, dispatch proof,
    executor failure behavior, protocol behavior, or D12-D15 components.

## Decisions and assumptions

- D11 is **DEAD**: an exact Rust search found its definition but no caller.
- Live J10 event processing calls the writer-aware seam. The retained adapters
  continue to give tests explicit control of replay authority and bridge pins,
  while using the shared inner boundary.
- The remaining stale prose occurrence in `dispatch.rs` is not a definition or
  caller and is outside the authorised Job C source path.

## Evidence

- `rg "fn authorise_and_execute\\(" tethers-0.1/host-rust --type rust` — PASS: zero D11 definitions after removal.
- Retained-seam reference search — PASS: writer-aware production calls, inner
  calls, and test-adapter calls remain present.
- Focused `authorise_and_execute_` tests — PASS: 4 tests covering success,
  failed outcome, succeeded outcome, and capability-mismatch non-dispatch.
- Focused `j10_initial_event_completes_before_draining_with_no_anchor` — PASS.
- Full-target locked `cargo check` — PASS: exactly four remaining
  production-library warnings (D12-D15).
- Formatter/check, `git diff --check`, and Clippy — PASS; only pre-existing
  wider-project Clippy warnings remain.
- `just verify-agent` — PASS (93s): packet, formatter, cargo check, full Cargo
  tests, Rust agent tools, dependency policy/advisories, and Nextest. Nextest:
  1592 passed, 2 skipped.

## Discoveries

- D11 exactly duplicated production setup that the live writer-aware seam
  already owns. No test migration was needed because tests were already
  targeting explicit lower-level adapters.

## Remaining risks

- None known within Job C scope. D12-D15 remain intentionally unresolved for
  independent classification in Job D.

## Smallest next action

Begin separate D12, D13, D14, and D15 classification from this pushed tip.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `tethers-0.1/host-rust/src/application.rs`
