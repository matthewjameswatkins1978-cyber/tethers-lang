# Current Implementation Task

Control contract: `1`
Task: `TETHERS-0.3-P1-R1C-FIX — Close Canonical Directory Proof`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Amber`
Route: `OpenCode implements bounded correction`
Worker note: `docs/worker-notes/2026-08-09-0.3-p1-r1c-canonical-directory.md`
Base branch: `feature/0.3-p1-r1b-scope-validation`
Base commit: `f18d211523de953d260417e67abbadf766412037`
Implementation branch: `feature/0.3-p1-r1c-canonical-directory`
Implementation checkpoint: `eae2e708c52cc1739113d5b2079239169541403e`
Original R1C checkpoint: `bb4ba228f8812703bb06bcf9970de42f4a9eee44`
Previous closeout HEAD: `942282e98761ee39f7d683d696ba72941b259835`
OCaml switch path: `resolve from existing machine state only`
Rust toolchain: `1.97.1`
Rust change class: `AMBER_ARCHITECTURE_CORRECTION`

## Relevant background and existing behaviour

R1C was architecturally accepted at `bb4ba22` and closeout-pushed at `942282e`. Two proof gaps remained: a dead_code warning introduced by R1C when `validate_operational_scope` became test-only, and missing explicit test coverage for schema-valued `additionalProperties` canonicalisation.

## Objective

Close two small proof gaps in the accepted R1C implementation:

1. Eliminate the dead_code warning by making `validate_operational_scope` `#[cfg(test)]`.
2. Add explicit test proof for `additionalProperties` canonicalisation.

## Required behaviour

1. Mark `validate_operational_scope` with `#[cfg(test)]` — not `#[allow(dead_code)]`.
2. Add `r1c_additional_properties_schema_canonicalised` test.
3. Update the worker note with corrected truth (warning not pre-existing, remote equality confirmed).

## Frozen decisions and invariants

1. No `#[allow(dead_code)]` suppression.
2. No production validation semantics changed.
3. No new schema features.
4. Overall P1 remains `completion repair in progress`.

## Acceptance criteria

1. `cargo check` returns 0 warnings. — DONE
2. additionalProperties canonicalisation test passes. — DONE
3. All 15 R1C tests pass. — DONE
4. All 14 R1B regression tests pass. — DONE
5. `cargo fmt --all -- --check` clean. — DONE
6. `git diff --check` clean. — DONE
7. Branch pushed, remote == local, clean worktree. — DONE

## Relevant components

### AUTHORISED PATHS
- `tethers-0.1/host-rust/src/validation.rs`

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-0.3-p1-r1c-canonical-directory.md`

## Required verification

1. `cargo check --all-targets --all-features --locked` — 0 warnings
2. `cargo test r1c` — 15/15 passed
3. `cargo test r1b` — 14/14 passed
4. `cargo fmt --all -- --check` — clean
5. `git diff --check` — clean

## Forbidden changes

- No Tethers language change
- No concurrency
- No plug pack/inspect/conform implementation
- No registry, marketplace, HTTP/WebSocket/gRPC, SDK, secret store, OAuth, OS sandbox
- No dependency update
- No physical extraction into `reference-plugs/`
- No synthetic Plug
- No provider changes
- No conformance repair, P2, migration tool
- No `just verify-agent`, engine fixtures, MCP transcripts, fixture validator

## Stop conditions

None. All acceptance criteria verified.

## Expected pre-existing changes

None.
