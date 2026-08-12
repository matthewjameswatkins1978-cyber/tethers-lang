# Current Implementation Task

Control contract: `1`

Task: `TETHERS CORE-7A - Canonical Entry Guard Evaluation`

Owner: `OpenCode`

Implementation checkpoint: `c9cfc20fefae80a02ee9658317af65aaf699bfe2`

Status: `COMPLETE`

Task colour: `Amber`

Route: `OpenCode implementation + evidence, Lucy independent GitHub review`

Worker note: `docs/worker-notes/2026-08-12-core-7a-entry-guard-evaluation.md`

Base branch: `feature/core-7a-entry-guards`

Base commit: `29e5cc72b1e71fdae2cccef4478f075878d0e288`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

Rust change class: `RUST_UNCHANGED`

## Objective

Add deterministic runtime evaluation of canonical Core entry guards before
Runtime Plan creation, establishing: canonical Core + runtime Fact snapshots ->
evaluate entry guards -> MATCHED produces Runtime Plan, NOT MATCHED produces
no plan, ERROR produces typed failure.

## Relevant background and existing behaviour

Core already defines evaluation-input Facts as `Evaluation_input of
host_snapshot_key * core_scalar_type` and entry guards as `fact_guard` records
with `fact_id`, `comparison_operator`, and expected `core_value`. Operators
are Equals, Contains, Greater_than, Greater_than_or_equal. CORE-4 implements
canonicalisation; CORE-6A implements the Core to Runtime Plan bridge; CORE-6B
adds the canonical planning entry point. Entry guards are not yet evaluated
by any planning path.

## Required behaviour

1. Add `fact_snapshot` type to `tethers_core_plan` (key: `host_snapshot_key`,
   value: `Yojson.Safe.t`)
2. Extend `planning_context` with `facts : fact_snapshot list`
3. Resolve each guard's canonical FactId to its `Evaluation_input` declaration
   in `canonical_program.input_facts`
4. Resolve runtime snapshot by exactly `HostSnapshotKey` (0 = error, 1 = continue,
   2+ = error)
5. Runtime type checking: String_type -> JSON string only, Integer_type -> JSON
   integer only, Boolean_type -> JSON boolean only
6. Guard comparison: Equals, Contains, Greater_than, Greater_than_or_equal
7. Add `evaluate_canonicalized` returning `Matched of canonical_plan | Not_matched`
8. `plan()` and `plan_canonicalized()` fail with `Unresolved_entry_guards` when
   entry_guards is non-empty
9. ProgramDigest preserved across different runtime Fact values
10. evaluation_id unaffected by Fact values

## Relevant components

- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.ml` -- modified (fact_snapshot, guard evaluation, evaluate_canonicalized, Unresolved_entry_guards, plan_internal)
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.mli` -- modified (fact_snapshot, facts field, canonical_evaluation, evaluate_canonicalized, new error constructors)
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan_test.ml` -- modified (G1-T1 through G1-T17, E2E, adversarial)

## Frozen decisions and invariants

- Runtime Facts are keyed by HostSnapshotKey, NOT canonical FactId
- FactId is internal Core identity and may be rewritten by canonicalisation
- HostSnapshotKey is the external semantic key through which runtime data enters
- Resolution by HostSnapshotKey must be deterministic: never first-match
- No coercion: "42" -> 42, 1 -> true, null -> missing are all forbidden
- Same canonical program + different runtime Facts: ProgramDigest stays identical
- evaluation_id remains the runtime occurrence identity regardless of Fact values
- plan() and plan_canonicalized() fail closed on guarded programs

## Acceptance criteria

1. G1-T1: Equals string matches
2. G1-T2: Equals string false -> Not_matched
3. G1-T3: Integer Greater_than
4. G1-T4: Integer Greater_than_or_equal
5. G1-T5: String Contains
6. G1-T6: Boolean Equals
7. G1-T7: Multiple guards AND together (3+ guards)
8. G1-T8: Missing runtime Fact -> Missing_fact_snapshot
9. G1-T9: Wrong HostSnapshotKey -> still Missing_fact_snapshot
10. G1-T10: Duplicate HostSnapshotKey -> Ambiguous_fact_snapshot
11. G1-T11: Reversed duplicate order -> same error
12. G1-T12: Runtime type mismatch -> Fact_snapshot_type_mismatch
13. G1-T13: Invalid comparison typing -> Invalid_guard_comparison
14. G1-T14: Low-level guard bypass blocked -> Unresolved_entry_guards
15. G1-T15: plan_canonicalized guard bypass blocked -> Unresolved_entry_guards
16. G1-T16: Unguarded existing behaviour preserved
17. G1-T17: ProgramDigest invariant across runtime facts
18. E2E: Human -> parser -> lowerer -> canonicalize -> evaluate_canonicalized
19. Adversarial: Canonical identity independence across different temporary FactIds

## Required verification

1. OCaml build: `dune build @all` -- PASS (exit 0)
2. All tests: `dune runtest --force` -- PASS (136/136 plan bridge tests)
3. Whitespace: `git diff --check` -- PASS
4. Cargo fmt: `cargo fmt --check` -- PASS (RUST_UNCHANGED)
5. Diff inspection: only authorised files changed
6. Git status: clean worktree
7. Task-packet checker at closeout: `control-v1/COMPLETE`
8. Push branch to origin and confirm local HEAD == remote HEAD

## Forbidden changes

No Core type changes, no validator semantic changes, no evaluator/protocol/outcome
changes, no runtime wiring, no production dispatch, no Rust changes, no new
dependencies. Do not change Human syntax or lowerer semantics. Do not change
canonicalisation semantics.

## Stop conditions

Commit CORE-7A implementation checkpoint. STOP. Do NOT begin CORE-7B or any
runtime wiring.

## Expected pre-existing changes

None.
