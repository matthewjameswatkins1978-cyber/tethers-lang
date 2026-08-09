# Current Implementation Task

Control contract: `1`
Task: `TETHERS-0.3-P1-R1B — Enforce Operational Scope Schema`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Amber`
Route: `OpenCode implements bounded correction`
Worker note: `docs/worker-notes/2026-08-09-0.3-p1-r1b-scope-validation.md`
Base branch: `origin/main`
Base commit: `809a245e83b77604788853ee6e18a3ecaa9e1f30`
Implementation branch: `feature/0.3-p1-r1b-scope-validation`
Implementation checkpoint: `pending`
OCaml switch path: `resolve from existing machine state only`
Rust toolchain: `1.97.1`
Rust change class: `AMBER_ARCHITECTURE_CORRECTION`

## Relevant background and existing behaviour

R1A pinned the operational-scope schema as verified package evidence through the full evidence chain. `run_enable()` reads the scope-schema digest from the installed record but does not validate the supplied scope against the schema. The schema is carried inertly through inspection, candidate, and installed records but is not enforced at enablement time.

## Objective

Complete one missing P1 correctness property:

> A scope supplied to `plug enable` must conform to the exact operational-scope schema pinned in the installed Plug evidence.

Do not implement path canonicalisation.

## Required behaviour

1. In `run_enable()`, validate the supplied scope against the installed `operational_scope_schema`.
2. Require both `operational_scope_schema` and `operational_scope_schema_digest` as a coherent pair.
3. Add a small dedicated `validate_operational_scope()` wrapper using the existing schema-validation machinery.
4. Recognise `x-tethers-path` as an operational-scope annotation (ignore semantics, not a rejection).
5. Do NOT permit arbitrary `x-tethers-*`.
6. Fix scope-file parsing so duplicate keys are rejected at every depth, including inside `scope`.

## Frozen decisions and invariants

1. Existing conservative JSON Schema validator reused.
2. `x-tethers-path` keyword recognised but not interpreted (R1C).
3. No arbitrary `x-tethers-*`.
4. No new dependency.
5. No reopening `plug.json`.
6. No fallback schema.
7. No path canonicalisation, filesystem checks, symlink/reparse work.
8. No synthetic Plug.
9. No conformance repair, P2, migration tool.
10. Overall P1 remains `completion repair in progress`.

## Acceptance criteria

1. Supplied scope validated against exact installed operational-scope schema. — DONE (10 focused tests pass, validation in run_enable)
2. Wrong types refused. — DONE (r1b_wrong_property_type_fails)
3. Required fields enforced. — DONE (r1b_missing_required_property_fails)
4. Additional properties enforced. — DONE (r1b_unknown_property_fails_when_additional_properties_false)
5. Numeric bounds enforced. — DONE (r1b_numeric_minimum_fails, r1b_numeric_maximum_fails)
6. Unsupported schema keywords fail closed. — DONE (r1b_unsupported_schema_keyword_fails)
7. Nested duplicate JSON keys refused. — DONE (r1b_nested_duplicate_scope_key_fails)
8. No schema rediscovered from disk. — DONE (reads from installed record)
9. No dependency changes. — PRESERVED
10. Focused checks pass. — PENDING (focused tests pass, full checks needed)
11. Branch pushed, remote == local, clean worktree. — PENDING

## Stop conditions

Already resolved: scope schema enforcement at enablement time.

## Expected pre-existing changes

None.

## Relevant components

### AUTHORISED PATHS
- `tethers-0.1/host-rust/src/plug_command.rs`
- `tethers-0.1/host-rust/src/validation.rs`

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-0.3-p1-r1b-scope-validation.md`

## Required verification

1. 10 focused R1B tests pass.
2. Full lib test suite passes.
3. `cargo fmt --all -- --check` clean.
4. `cargo check --all-targets --all-features --locked` clean.
5. `cargo clippy --all-targets --all-features --locked` passes.
6. `git diff --check` clean.

## Forbidden changes

- No Tethers language change
- No concurrency
- No plug pack/inspect/conform implementation
- No registry, marketplace, HTTP/WebSocket/gRPC, SDK, secret store, OAuth, OS sandbox
- No dependency update
- No physical extraction into `reference-plugs/`
- No path canonicalisation
- No filesystem checks, symlink/reparse work
- No synthetic Plug
- No provider changes except to keep existing focused tests compiling
- No conformance repair, P2, migration tool
- No `just verify-agent`, engine fixtures, MCP transcripts, fixture validator
