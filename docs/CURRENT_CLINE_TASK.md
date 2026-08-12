# Current Implementation Task

Control contract: `1`

Task: `TETHERS CORE-8A - Human Request → Canonical Evaluation Adapter`

Owner: `OpenCode`

Implementation checkpoint: `TBD`

Status: `IN_PROGRESS`

Task colour: `Amber`

Route: `OpenCode implementation + evidence, Lucy independent GitHub review`

Worker note: `docs/worker-notes/2026-08-12-core-8a-evaluation-adapter.md`

Base branch: `feature/core-7b-anchor-reception`

Base commit: `97eb5e637b9c4cfcba729d8ced71360784922e7b`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

Rust change class: `RUST_UNCHANGED`

## Objective

Create one pure OCaml adapter that turns a Human-world evaluation input into
the already-accepted canonical Core evaluation path:

Human source → parser → lowerer → canonicalize → canonical reception →
guards → Runtime Plan

The adapter MUST receive explicit semantic lowering identities from its caller.

Do NOT switch the existing production `tethers.evaluate` MCP tool to this path yet.

Do NOT invent capability identity from runtime capability names.

## Required behaviour

1. New module: `tethers_core_evaluation_adapter.ml` / `.mli`
2. Adapter environment with explicit semantic identities (capability_binding, input_fact_binding, environment)
3. Human-world evaluation_input type
4. One-call evaluate pipeline: parse → lower → canonicalize → map facts → build context → evaluate_canonicalized
5. Runtime Fact mapping through environment.input_facts with deterministic error handling
6. Capability mapping producing both lowerer and plan projections from same binding
7. Typed adapter_error preserving layer ownership
8. ProgramDigest emerges only from canonicalisation
9. evaluation_id passes through unchanged
10. Legacy evaluator remains unchanged

## Relevant components

- `tethers-0.1/engine-ocaml/bin/tethers_core_evaluation_adapter.ml` -- new
- `tethers-0.1/engine-ocaml/bin/tethers_core_evaluation_adapter.mli` -- new
- `tethers-0.1/engine-ocaml/bin/tethers_core_evaluation_adapter_test.ml` -- new
- `tethers-0.1/engine-ocaml/bin/dune` -- modified (test stanza)

## Frozen decisions and invariants

- Caller provides explicit semantic lowering identities
- CapabilityId is Core semantic identity, NOT derived from capability name
- Contract digest is explicitly supplied, NOT derived
- ProgramDigest emerges only from canonicalisation
- evaluation_id passes through unchanged
- No coercion, no case folding, no normalisation for event name matching
- Legacy evaluator unchanged

## Acceptance criteria

1. T1: Minimal unguarded Human Tether → Matched
2. T2: Full guarded Anchor-value Human flow → Matched with correct arguments
3. T3: Wrong event → Not_matched
4. T4: Guard false → Not_matched
5. T5: Missing required Fact → Error (Planning_error (Missing_fact_snapshot ...))
6. T6: Unknown supplied runtime Fact name → Unknown_runtime_fact_name
7. T7: Duplicate supplied runtime Fact name → Duplicate_runtime_fact_name
8. T8: Ambiguous environment Fact source name → Ambiguous_runtime_fact_name
9. T9: Capability source-name resolution
10. T10: Capability source name differs from Core CapabilityId
11. T11: Wrong capability projection identity cannot substitute
12. T12: ProgramDigest invariant across occurrence data
13. T13: ProgramId changes do not alter occurrence identity
14. T14: evaluation_id changes occurrence only
15. T15: Existing low-level Core tests remain green
16. E2E: One-call adapter proof (no manual parser/lowerer/canonical/plan calls)

## Required verification

1. OCaml build: `dune build @all` -- PASS (exit 0)
2. All tests: `dune runtest --force` -- PASS
3. Whitespace: `git diff --check` -- PASS
4. Diff inspection: only authorised files changed
5. Git status: clean worktree
6. Task-packet checker at closeout: `control-v1/COMPLETE`
7. Push branch to origin and confirm local HEAD == remote HEAD

## Forbidden changes

No production evaluator changes, no main.ml changes, no MCP protocol changes,
no Rust changes, no Human syntax changes, no parser semantic changes, no
lowerer semantic changes, no Core vocabulary changes, no validator semantic
changes, no canonicalisation semantic changes, no Runtime Plan representation
changes, no new external dependencies.

## Stop conditions

Commit CORE-8A implementation checkpoint. STOP. Do NOT begin CORE-8B or any
production evaluator wiring.

## Expected pre-existing changes

None.
