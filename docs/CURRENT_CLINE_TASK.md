# Current Implementation Task

Control contract: `1`

Task: `TETHERS CORE-9B - Rust to Canonical Core Cross-Language Rehearsal`

Owner: `OpenCode`

Implementation checkpoint: `c79db5caf096d8a3037476ee422d4ba25cdeab42`

Status: `COMPLETE`

Task colour: `Amber`

Route: `OpenCode implementation + evidence, Lucy independent GitHub review`

Worker note: `docs/worker-notes/2026-08-12-core-9b-cross-language-rehearsal.md`

Base branch: `feature/core-9a-rust-semantic-environment`

Base commit: `fe9919034a3d04cbbe3056f7c7fdc91c041032ba`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

Rust change class: `RUST_CHANGED`

## Objective

Prove that a REAL request assembled by the Rust host using the accepted
CORE-9A semantic authority can cross the retained MCP engine boundary,
enter the accepted CORE-8B request adapter, and produce a canonical
Runtime Plan.

## Relevant background and existing behaviour

CORE-8B established the core pipeline modules (request adapter, evaluation
adapter, plan, canonical, validator, lowerer) with 400+ OCaml tests.
CORE-9A added CoreEnvironmentConfig and PreparedCoreEnvironment to the Rust
host. The MCP server currently uses only the legacy Tethers_evaluator.

## Required behaviour

1. Add `tethers_core_wire.ml`/`.mli` — narrow OCaml module bridging
   CORE-8B request adapter to existing PlannerResponseWire-compatible JSON
2. Add `tethers.evaluate_core` MCP tool (9B.5) keeping `tethers.evaluate`
   on legacy evaluator
3. Update dune to include Core modules in MCP binary (9B.6)
4. Add `evaluate_tether_core` Rust engine method (9B.7)
5. Add dormant `build_core_request_envelope` Rust helper (9B.8)
6. Add OCaml wire tests T1-T3
7. Verify production route unchanged (9B.9)

## Relevant components

- `tethers-0.1/engine-ocaml/bin/tethers_core_wire.ml` -- new
- `tethers-0.1/engine-ocaml/bin/tethers_core_wire.mli` -- new
- `tethers-0.1/engine-ocaml/bin/tethers_core_wire_test.ml` -- new
- `tethers-0.1/engine-ocaml/bin/tethers_mcp_server.ml` -- modified
- `tethers-0.1/engine-ocaml/bin/dune` -- modified
- `tethers-0.1/host-rust/src/engine_stdio.rs` -- modified
- `tethers-0.1/host-rust/src/host_execution.rs` -- modified

## Frozen decisions and invariants

- `tethers.evaluate` MUST remain on the legacy evaluator
- `tethers.evaluate_core` uses the Core pipeline via tethers_core_wire
- No Action dispatch in this packet
- No production route change (CORE-9C will switch)
- Core contract digest != manifest digest; do not compare or equate
- The Runtime Plan in the response is canonical_plan.runtime_plan
- program_digest comes only from canonical_plan.program_digest
- Empty trail is compatibility scaffolding only

## Acceptance criteria

1. T1: OCaml wire Matched produces correct envelope with program_digest
2. T2: OCaml wire Not_matched produces correct envelope with plan null
3. T3: OCaml wire request error produces stable error code
4. MCP tools/list contains tethers.evaluate, tethers.evaluate_core, tethers.validate
5. Legacy tethers.evaluate still uses legacy evaluator
6. New tethers.evaluate_core calls Tethers_core_wire
7. EngineSession::evaluate_tether_core calls tethers.evaluate_core
8. build_core_request_envelope fails when core_environment is None
9. Production evaluate_one unchanged (legacy route)
10. All existing OCaml and Rust tests remain green
11. cargo fmt --check PASS
12. cargo check PASS
13. cargo test PASS (1431 passed)
14. git diff --check PASS (LF/CRLF warnings only)

## Required verification

1. `cargo fmt --check` -- PASS
2. `cargo check` -- PASS
3. `cargo test` -- PASS (1431 passed, 0 failed)
4. `dune build` -- PASS
5. `dune runtest --force` -- PASS (all OCaml tests green)
6. `git diff --check` -- PASS (LF/CRLF warnings only)
7. Diff inspection: only authorised files changed
8. Production route unchanged: evaluate_one still calls evaluate_tether

## Forbidden changes

Do NOT switch tethers.evaluate to Core.
Do NOT change HostExecutionService::evaluate_one to Core.
Do NOT execute the resulting Action.
Do NOT modify provider dispatch, policy, Trail semantics, replay, approval.
Do NOT derive semantic identities.
Do NOT equate Core contract digest with manifest digest.
Do NOT modify CORE-8A/8B semantics or Runtime Plan semantics.

## Stop conditions

Implementation checkpoint committed. STOP.

## Expected pre-existing changes

CORE-9A semantic environment authority (accepted).
