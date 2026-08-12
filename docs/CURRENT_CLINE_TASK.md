# Current Implementation Task

Control contract: `1`

Task: `TETHERS CORE-8B1 — Total Request Parsing + Fact Fidelity Proof`

Owner: `OpenCode`

Implementation checkpoint: `c333812c1092f196654cf6c0556e156ae2adb3cc`

Status: `COMPLETE`

Task colour: `Amber`

Route: `OpenCode implementation + evidence, Lucy independent GitHub review`

Worker note: `docs/worker-notes/2026-08-12-core-8b1-total-parsing-fact-fidelity.md`

Base branch: `feature/core-8b-request-boundary`

Base commit: `dcdc6dbe5e568b863f86dde29d05b6bf80a9b3a5`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

Rust change class: `RUST_UNCHANGED`

## Objective

Close four narrow review findings in CORE-8B:

1. Make evaluate_request total over request JSON (no escape through
   Exit, Type_error, Failure, Match_failure)
2. Preserve top-level fact occurrence data exactly (no filter_map)
3. Fix T3 reception-before-guard proof
4. Strengthen T7 and T13 with exact key/idempotency assertions

Plus 8 new regression tests (R1-R8).

## Relevant background and existing behaviour

CORE-8B created the request boundary module. Review found four gaps:
- `raise Exit` paths in `resolve_one_capability` and `parse_one_fact`
- `filter_map` dropping non-scalar fact values before CORE-8A
- T3 not proving reception-before-guard semantics
- T7 and T13 assertions too shallow

## Required behaviour

1. Remove all `raise Exit` paths from `resolve_one_capability` and `parse_one_fact`
2. Use Result-returning `core_env_string` helper for core_environment field extraction
3. Validate `core_environment` is an object before parsing
4. Require `facts` field to be an object (missing/null = Invalid_request)
5. Pass occurrence facts pairs through unchanged (no filter_map)
6. Strengthen T3 with guarded tether, wrong event, no facts
7. Strengthen T7 with exact HOST_KEY_771 key assertion
8. Strengthen T13 with exact idempotency key assertions
9. Add R1-R8 regression tests

## Relevant components

- `tethers-0.1/engine-ocaml/bin/tethers_core_request_adapter.ml` -- modified
- `tethers-0.1/engine-ocaml/bin/tethers_core_request_adapter.mli` -- unchanged
- `tethers-0.1/engine-ocaml/bin/tethers_core_request_adapter_test.ml` -- modified

## Frozen decisions and invariants

- evaluate_request is total: no structural parsing exceptions escape
- Fact occurrence data passes through unchanged to CORE-8A
- core_environment must be an object (not Null, not string, etc.)
- facts must be an object (missing/null = Invalid_request)
- Malformed core_environment fields return Invalid_core_environment
- Malformed core_environment fields return Invalid_core_environment

## Acceptance criteria

1. All T1-T16 tests pass (strengthened T3, T7, T13)
2. R1: malformed core capability field → typed error
3. R2: malformed Fact declaration → typed error
4. R3: malformed core_environment structural type → typed error
5. R4: facts missing/non-object → Invalid_request
6. R5: non-scalar occurrence Fact preserved → type mismatch (not missing)
7. R6: guarded wrong-event → Not_matched
8. R7: exact HOST_KEY_771 assertion
9. R8: exact idempotency key assertions
10. dune build @all PASS
11. dune runtest --force PASS
12. git diff --check PASS
13. task-packet checker PASS

## Required verification

1. OCaml build: `dune build @all` -- PASS (exit 0)
2. All tests: `dune runtest --force` -- PASS
3. Whitespace: `git diff --check` -- PASS
4. Diff inspection: only authorised files changed
5. Git status: clean worktree
6. Task-packet checker at closeout: `control-v1/COMPLETE`
7. Push branch to origin and confirm local HEAD == remote HEAD

## Forbidden changes

No production evaluator, no main.ml, no MCP, no Rust, no CORE-8A adapter
semantics, no lowerer, no Core, no validator, no canonicalisation, no planner
semantics.

## Stop conditions

Commit CORE-8B1 implementation checkpoint. STOP.

## Expected pre-existing changes

CORE-8B request boundary (accepted).
