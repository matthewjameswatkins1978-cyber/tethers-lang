# Worker Note

Task: `F8-D12+D13+D14+D15 — Final Dead-Member / Test-Only Tail`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `f804759043eaa087a6f358fca9781716ac42bfb7`

Implementation checkpoint: `a029e6117846f2fbfeca78693ef2336b5f5c0317`

## Requested outcome

Resolved the final four independently classified production warning targets
without changing their live contracts.

## Changes made

- D12: removed only unread `SupervisedChild.max_line_bytes` field/assignment.
- D13: marked test-only non-creating `open_existing` and test inspection
  `root_path` with `#[cfg(test)]`.
- D14: removed only unused ordinary authority-construction wrappers, retaining
  the live injectable `_with` methods.
- D15: removed obsolete ResultAnchorKind variants/match arms and changed three
  tests to generic `Failed` with the identical external error codes.

## Decisions and assumptions

- D12 is dead storage; the captured reader-thread protocol limit and all
  LineTooLarge paths remain.
- D13 is valuable test-only architecture, not dead semantics.
- D14's live conformance path uses `_with` authority injection.
- D15 generic `Failed` preserves `capability.failed`, `provider_error`, and
  `result_validation_failed` exactly.

## Evidence

- Post-change searches — PASS: no D12 field, D14 wrappers, or D15 variants;
  D13 methods are cfg-test-only.
- Focused Result Anchor tests — PASS: 5 tests, including exact provider and
  validation codes and failure serialization.
- Focused current-trust launch tests — PASS: 2 injected-authority tests.
- Focused installation-publication tests — PASS: 81 tests including
  `open_existing` and torn/current intent evidence.
- Full-target locked cargo check — PASS: zero production-library warnings;
  only two pre-existing test-module imports remain.
- Formatter/check, `git diff --check`, and Clippy — PASS.
- `just verify-agent` — PASS (90s): packet, formatter, cargo check, full Cargo
  tests, Rust agent tools, dependency policy/advisories, and Nextest. Nextest:
  1592 passed, 2 skipped.

## Discoveries

- No standalone LineTooLarge-named test is registered, but the only active
  limit is still captured from ChildConfig into the reader thread and all full
  tests pass.

## Remaining risks

- None known within Job D scope. The next job is documentation-only.

## Smallest next action

Create the separate F8 zero-warning documentation checkpoint.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `tethers-0.1/host-rust/src/child_process.rs`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/launch_profile.rs`
- `tethers-0.1/host-rust/src/result_anchor.rs`
