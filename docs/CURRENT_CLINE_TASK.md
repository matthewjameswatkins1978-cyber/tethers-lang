# Current Implementation Task

Control contract: `1`

Task: `TETHERS CORE-9A — Rust Semantic Environment Authority`

Owner: `OpenCode`

Implementation checkpoint: `c1a46c26815cfeb3999a97a6bd0e51e16cbdd87f`

Status: `COMPLETE`

Task colour: `Amber`

Route: `OpenCode implementation + evidence, Lucy independent GitHub review`

Worker note: `docs/worker-notes/2026-08-12-core-9a-rust-semantic-environment.md`

Base branch: `feature/core-8b-request-boundary`

Base commit: `81722867840c3adf03794cbaeff761f414a96301`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

Rust change class: `RUST_CHANGED`

## Objective

Extend the Rust runtime configuration/preparation layer so each configured
Tether MAY carry the explicit semantic environment required by the accepted
CORE-8B request boundary. Authority + preparation only; no production
evaluation injection yet.

## Relevant background and existing behaviour

CORE-8B established the core_environment JSON wire shape in the OCaml
request adapter. The Rust host must now have an honest, typed source for
ProgramId, CoreVersion, capability identities, and fact identities.

## Required behaviour

1. Add CoreEnvironmentConfig, CoreCapabilityBindingConfig,
   CoreInputFactBindingConfig, CoreScalarType types
2. Add optional core_environment to TetherRef
3. Validate structural host-owned invariants when present (9A.5)
4. Validate runtime_name join (0 or 2+ matches fail closed) (9A.6)
5. Add PreparedCoreEnvironment, PreparedCoreCapabilityBinding,
   PreparedCoreInputFactBinding to configured_runtime.rs
6. Carry core_environment through prepare_runtime()
7. Add core_environment_json() pure serializer
8. Add T1-T15 + adversarial tests

## Relevant components

- `tethers-0.1/host-rust/src/runtime_config.rs` -- modified
- `tethers-0.1/host-rust/src/configured_runtime.rs` -- modified
- `tethers-0.1/host-rust/src/host_execution.rs` -- modified (field init)

## Frozen decisions and invariants

- core_environment is optional; absent means no Core semantic authority
- Never derive program_id from tether.id
- Never derive capability_id from source_name, runtime_name, or manifest
- Never derive contract_digest from pinned_digest or manifest digest
- Never derive fact_id or host_snapshot_key from source_name
- Core contract digest != manifest digest; do not compare or equate
- scalar_type is exactly one of: string, integer, boolean
- runtime_name join: 0 matches or 2+ matches fails closed
- Preserve configured array order; no sorting, dedup, or canonicalisation
- core_environment is dormant; not injected into production requests yet

## Acceptance criteria

1. T1: existing config without core_environment remains valid
2. T2: explicit environment parses with exact typed values
3. T3: program_id not derived from tether.id
4. T4: four capability identities differ and survive unchanged
5. T5: contract_digest distinct from manifest digest
6. T6: explicit Fact identities survive unchanged
7. T7: all three scalar types accepted
8. T8: invalid scalar type rejected
9. T9: missing runtime_name target fails closed
10. T10: ambiguous runtime_name fails closed
11. T11: empty semantic identities rejected
12. T12: exact CORE-8B JSON projection
13. T13: missing environment has no JSON
14. T14: serializer preserves configured order
15. T15: all existing Rust tests stay green
16. Adversarial test: no accidental derivation
17. cargo fmt --check PASS
18. cargo check PASS
19. cargo test PASS (1431 passed)
20. git diff --check PASS
21. OCaml unchanged
22. host_execution.rs prepared with core_environment: None

## Required verification

1. `cargo fmt --check` -- PASS
2. `cargo check` -- PASS
3. `cargo test` -- PASS (1431 passed, 0 failed)
4. `git diff --check` -- PASS (LF/CRLF warnings only)
5. Diff inspection: only authorised files changed
6. Git status: clean worktree after push
7. OCaml: zero diff against base

## Forbidden changes

No production evaluator, no main.ml, no MCP, no OCaml request adapter
semantics, no lowerer, no Core, no validator, no canonicalisation, no planner
semantics, no policy changes, no Trail changes, no provider execution changes.

## Stop conditions

Commit CORE-9A implementation checkpoint. STOP.

## Expected pre-existing changes

CORE-8B request boundary and CORE-8B2 total-parsing fixes (accepted).
