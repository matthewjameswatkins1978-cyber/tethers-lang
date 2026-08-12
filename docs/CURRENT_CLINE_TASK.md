# Current Implementation Task

Control contract: `1`

Task: `TETHERS CORE-8B2 — Truly Total JSON Shape Validation`

Owner: `OpenCode`

Implementation checkpoint: `203393ae0715d53122ce98da47e3d4d31079919f`

Status: `COMPLETE`

Task colour: `Amber`

Route: `OpenCode implementation + evidence, Lucy independent GitHub review`

Worker note: `docs/worker-notes/2026-08-12-core-8b2-truly-total-json-validation.md`

Base branch: `feature/core-8b-request-boundary`

Base commit: `10605078138ab7eab3e3cda8a5d4d14eec53d243`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

Rust change class: `RUST_UNCHANGED`

## Objective

Finish the total-request invariant from CORE-8B1. evaluate_request must
return request_error for malformed structural JSON. No Yojson.Safe.Util.Type_error
or other structural exception may escape.

## Relevant background and existing behaviour

CORE-8B1 removed `raise Exit` paths and added `core_env_string` helper.
But `json_string`, `json_list`, `json_member`, and `core_env_string` still
call `Yojson.Safe.Util.member` on potentially non-object values. The root
request, tether, event, and capability-list items were not validated as
objects before field extraction.

## Required behaviour

1. Replace Yojson.Util.member-based helpers with object-safe extraction
   that proves `Assoc` before accessing fields
2. Validate root request is object; tether is object; event is object
3. Validate each capability list item is object before parse_capability
4. Validate each core_environment capability binding is object
5. Validate each input_facts declaration is object
6. Require schema_description as mandatory string field (not optional "")
7. Add Q1-Q12 regression tests

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
- schema_description is required (not optional)

## Acceptance criteria

1. Q1: `evaluate_request `Null` → Invalid_request, no exception
2. Q2: `evaluate_request (`String "oops")` → Invalid_request, no exception
3. Q3: tether = `String "oops"` → Invalid_request
4. Q4: event = `List []` → Invalid_request
5. Q5: capabilities = [`Int 42] → Invalid_request
6. Q6: core_environment.capabilities = [`Int 42] → Invalid_core_environment
7. Q7: core_environment.input_facts = [`String "oops"] → Invalid_core_environment
8. Q8: Fact declaration missing schema_description → Invalid_core_environment
9. Q9: Fact declaration schema_description = `Int 7 → Invalid_core_environment
10. Q10: program_id wrong type → Invalid_core_environment
11. Q11: core_version wrong type → Invalid_core_environment
12. Q12: all previous tests remain green
13. dune build @all PASS
14. dune runtest --force PASS
15. git diff --check PASS
16. task-packet checker PASS

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

Commit CORE-8B2 implementation checkpoint. STOP.

## Expected pre-existing changes

CORE-8B request boundary and CORE-8B1 total-parsing fixes (accepted).
