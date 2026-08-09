# Current Implementation Task

Control contract: `1`
Task: `TETHERS-0.3-P1-R1C — Canonical Operational Scope Directories`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Amber`
Route: `OpenCode implements bounded correction`
Worker note: `docs/worker-notes/2026-08-09-0.3-p1-r1c-canonical-directory.md`
Base branch: `feature/0.3-p1-r1b-scope-validation`
Base commit: `f18d211523de953d260417e67abbadf766412037`
Implementation branch: `feature/0.3-p1-r1c-canonical-directory`
Implementation checkpoint: `bb4ba228f8812703bb06bcf9970de42f4a9eee44`
OCaml switch path: `resolve from existing machine state only`
Rust toolchain: `1.97.1`
Rust change class: `AMBER_ARCHITECTURE_CORRECTION`

## Relevant background and existing behaviour

R1A pinned the operational-scope schema as verified package evidence. R1B enforced the schema against the supplied scope at enablement time, using `validate_operational_scope`. The `x-tethers-path` annotation was recognised but left inert — validation succeeded against the cleaned schema, but no path hardening was performed. OperationalScopeEvidence stored the supplied scope values verbatim.

## Objective

Implement the actual meaning of `x-tethers-path: canonical-directory`:

> The host must turn an authorised directory path into exact, hardened, canonical scope evidence before enablement.

## Required behaviour

1. `validate_and_canonicalize_operational_scope()` replaces `validate_operational_scope()` in `run_enable()`.
2. The function validates against the cleaned schema, canonicalises `x-tethers-path` fields, then re-validates the canonical result.
3. Reuses `m3_store::verify_chain()` and `m3_store::reject_reparse()` for path hardening.
4. Supports nested canonical-directory annotations in `properties`, `items`, and schema-valued `additionalProperties`.
5. The canonical scope (not the original) is stored in `OperationalScopeEvidence`.
6. The schema and schema digest remain unchanged.

## Frozen decisions and invariants

1. Existing conservative JSON Schema validator reused.
2. `x-tethers-path` keyword value must be `"canonical-directory"`.
3. No arbitrary `x-tethers-*`.
4. No new dependency.
5. No reopening `plug.json`.
6. No fallback schema.
7. No path canonicalisation of non-annotated values.
8. No synthetic Plug.
9. No conformance repair, P2, migration tool.
10. Overall P1 remains `completion repair in progress`.

## Acceptance criteria

1. Supplied scope validated against exact installed operational-scope schema. — DONE (first validation pass)
2. `x-tethers-path: canonical-directory` triggers real filesystem hardening. — DONE (14 focused tests pass)
3. Only absolute existing directories accepted. — DONE (relative, nonexistent, file-all refuse)
4. Symlink/reparse paths and ancestors refused. — DONE (junction + ancestor-via-junction tests pass on Windows)
5. Canonical filesystem path replaces supplied value. — DONE (evidence stores canonical)
6. Canonical result validated again. — DONE (second validation pass)
7. OperationalScopeEvidence stores only canonical scope. — DONE (r1c_evidence_stores_canonical_path_not_original)
8. Schema/schema digest remain exact and unchanged. — DONE (r1c_schema_digest_unchanged_by_canonicalization)
9. No ordinary capability-schema semantics broadened. — DONE (validate_against_schema unchanged)
10. Focused verification passes. — PASS (14 R1C tests, full lib: 1371/0/2, clippy pass, fmt clean)
11. Branch pushed, remote == local, clean worktree. — DONE (see worker note)

## Stop conditions

None. All acceptance criteria verified.

## Expected pre-existing changes

None.

## Relevant components

### AUTHORISED PATHS
- `tethers-0.1/host-rust/src/plug_command.rs`
- `tethers-0.1/host-rust/src/validation.rs`

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-0.3-p1-r1c-canonical-directory.md`

## Required verification

1. 14 focused R1C tests pass.
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
- No synthetic Plug
- No provider changes except to keep existing focused tests compiling
- No conformance repair, P2, migration tool
- No `just verify-agent`, engine fixtures, MCP transcripts, fixture validator
