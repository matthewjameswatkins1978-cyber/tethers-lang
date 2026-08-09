# Current Implementation Task

Control contract: `1`
Task packet: `F8-D12+D13+D14+D15 — Final Dead-Member / Test-Only Tail`
Owner: `Codex`
Status: `COMPLETE`
Task colour: `Amber`
Route: `Codex resolved the independently classified final warning tail without changing live contracts`
Worker note: `docs/worker-notes/2026-08-09-f8-d12-d15-final-warning-tail.md`
Base branch: `foundation/f8-d11-authorise-wrapper-cleanup`
Base commit: `f804759043eaa087a6f358fca9781716ac42bfb7`
Implementation branch: `foundation/f8-d12-d15-final-warning-tail`
Implementation checkpoint: `a029e6117846f2fbfeca78693ef2336b5f5c0317`
OCaml switch path: `N/A`
Rust toolchain: `1.97.1`
Rust change class: `RUST`

## Objective

Resolve the remaining D12-D15 production-library warnings only where each
independent classification preserves the existing product and test contract.

## Relevant background and existing behaviour

Job C left four warnings. D12 is an unread `SupervisedChild` copy of the
protocol line limit; the reader thread uses a separate local capture. D13
methods are exercised solely by cfg-test modules and preserve important
non-creating/test-inspection semantics. D14 wrappers only construct the
ordinary trust authority; real conformance calls the injectable `_with`
methods. D15 variants are only matched by `ResultAnchor::new` and constructed
by local tests; the generic `Failed { code, message }` already serializes the
same `capability.failed`, `provider_error`, and `result_validation_failed`
external contract.

## Required behaviour

1. Remove only D12's redundant stored field and constructor assignment, while
   preserving the configured reader-thread line limit and LineTooLarge paths.
2. Mark D13 `open_existing` and `root_path` cfg-test-only, retaining all their
   semantics and tests.
3. Remove only D14's unused non-injectable wrappers; retain current-trust and
   launch `_with` methods and their authority-injection behavior.
4. Replace D15 test constructions with generic `Failed` values, then remove
   the obsolete variants and constructor match arms without changing serialized
   failed result-anchor codes or event names.
5. Demonstrate zero intended production-library warnings, run focused checks
   for each component, and run exactly one final `just verify-agent`.

## Relevant components

### AUTHORISED PATHS
- `tethers-0.1/host-rust/src/child_process.rs`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/launch_profile.rs`
- `tethers-0.1/host-rust/src/result_anchor.rs`

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-f8-d12-d15-final-warning-tail.md`

## Frozen decisions and invariants

- D12 retains `ChildConfig.max_protocol_line_bytes`, the local reader-thread
  capture, and LineTooLarge behavior.
- D13 remains available to cfg-test code; no open_existing behavior is changed
  into creating behavior.
- D14 retains `_with` authority injection, candidate pinning, launch
  environment, suspended Job assignment, and process/memory/protocol limits.
- D15 retains generic Failed serialization with exact existing external codes
  and `capability.failed` event name.
- No suppression, unrelated refactor, protocol, dependency/toolchain, CI,
  merge, amend, tag, force-push, direct-main, or pull-request change.

## Acceptance criteria

1. D12 field/assignment are absent and focused child-process limit evidence
   passes.
2. D13 methods are cfg-test-only and existing no-create/current.json tests
   pass.
3. D14 wrappers are absent while `_with` callers/tests remain and pass.
4. D15 obsolete variants/match arms are absent; tests directly prove the exact
   provider and validation serialized codes remain.
5. Full-target locked cargo check reports zero production-library warnings;
   formatter, whitespace, Clippy, and final umbrella verification pass.

## Required verification

1. Per-target Rust reference searches after each change.
2. Focused child-process, installation-publication, current-trust/launch, and
   result-anchor tests.
3. Full-target locked cargo check, formatter diff/check, `git diff --check`,
   and Clippy.
4. One final `just verify-agent`, then complete diff/range, remote equality,
   and clean status checks.

## Forbidden changes

- No change to current-trust, process limit, publication-intent, Result Anchor
  public protocol, queue, external error-code, or event-name semantics.
- No dead-code suppression or work outside listed paths and closeout docs.

## Stop conditions

STOP the affected item if a real production caller, sole live representation,
weakened test contract, or architectural choice is found. Do not use another
item's classification as justification. Do not begin Job E without a verified
and pushed Job D tip.

## Expected pre-existing changes

None.
