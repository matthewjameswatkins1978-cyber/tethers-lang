# Current Implementation Task

Control contract: `1`

Task: `TETHERS CORE-7B - Canonical Anchor Reception and Runtime Event Binding`

Owner: `OpenCode`

Implementation checkpoint: `c5e37618d5114af13d153d82c0685756631667f7`

Status: `IN_PROGRESS`

Task colour: `Amber`

Route: `OpenCode implementation + evidence, Lucy independent GitHub review`

Worker note: `docs/worker-notes/2026-08-12-core-7b-anchor-reception.md`

Base branch: `feature/core-7b-anchor-reception`

Base commit: `28426eed6ed145791e51793fc6954e9dc1a5b173`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

Rust change class: `RUST_UNCHANGED`

## Objective

Make the high-level canonical evaluation path consume an actual runtime event
and prove that the event matches the Core Anchor before evaluating guards or
planning Actions.

## Relevant background and existing behaviour

CORE-7A implemented entry guard evaluation. The existing `evaluate_canonicalized`
takes a `planning_context` with manually-supplied `anchors` and `facts`. The
caller must know the canonical Anchor OriginId to construct the anchor snapshot.
CORE-7B adds the reception gate: the caller supplies a runtime event (name +
data), the evaluator finds the single canonical Anchor_origin, matches the event
name exactly, and internally derives the anchor snapshot from the event data.

## Required behaviour

1. Add `runtime_event` type: `{ name : string; data : Yojson.Safe.t }`
2. Add `evaluation_context` type: `{ evaluation_id; event; capabilities; facts }`
3. Add `Missing_reception_anchor` and `Ambiguous_reception_anchor` errors
4. Implement anchor reception in `evaluate_canonicalized`:
   - Find single top-level Anchor_origin (0 = error, 2+ = error)
   - Exact event name match (no normalisation)
   - Mismatch = `Ok Not_matched` (not an error)
   - Match = bind event data to canonical Anchor OriginId internally
5. Evaluation order: reception → guards → plan
6. Wrong event + missing/malformed Fact → `Not_matched` (not guard error)
7. ProgramDigest invariant across different events
8. evaluation_id preserved (plan.id = evaluation_id + "/plan")
9. Caller must NOT supply canonical Anchor OriginId

## Relevant components

- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.ml` -- modified
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.mli` -- modified
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan_test.ml` -- modified

## Frozen decisions and invariants

- Runtime event name/data are occurrence inputs, NOT program identity
- ProgramDigest stays identical across different events for same canonical program
- Reception happens before guard evaluation (ordering is semantic)
- Exact string equality for event name matching
- Caller must not know canonical Anchor OriginId
- No coercion, no case folding, no normalisation

## Acceptance criteria

1. T1: Exact event match → Matched
2. T2: Event mismatch → Not_matched
3. T3: Matching is exact (case, space, prefix)
4. T4: Reception before missing Fact → Not_matched
5. T5: Reception before malformed Fact → Not_matched
6. T6: Matched event then missing Fact → Missing_fact_snapshot
7. T7: Matched event then guard false → Not_matched
8. T8: Matched event + guard true → Matched
9. T9: Event data resolves Anchor_value
10. T10: Event mismatch prevents Anchor path error
11. T11: Event match exposes Anchor path error
12. T12: Missing reception Anchor → Missing_reception_anchor
13. T13: Multiple reception Anchors → Ambiguous_reception_anchor
14. T14: ProgramDigest invariant across events
15. T15: evaluation_id preserved
16. E2E A: Human → canonical → reception → guards → plan (full match)
17. E2E B: Wrong event → Not_matched
18. E2E C: Right event, wrong condition → Not_matched
19. Adversarial: Canonical identity independence across temporary Anchor OriginIds

## Required verification

1. OCaml build: `dune build @all` -- PASS (exit 0)
2. All tests: `dune runtest --force` -- PASS (179/179 plan bridge tests)
3. Whitespace: `git diff --check` -- PASS
4. Cargo fmt: `cargo fmt --check` -- PASS (RUST_UNCHANGED)
5. Diff inspection: only authorised files changed
6. Git status: clean worktree
7. Task-packet checker at closeout: `control-v1/COMPLETE`
8. Push branch to origin and confirm local HEAD == remote HEAD

## Forbidden changes

No Human syntax changes, no parser semantic changes, no lowerer semantic changes,
no Core vocabulary changes, no validator semantic changes, no canonicalisation
semantic changes, no Runtime Plan representation changes, no production evaluator
changes, no main.ml changes, no MCP protocol changes, no Rust changes, no new
dependencies.

## Stop conditions

Commit CORE-7B implementation checkpoint. STOP. Do NOT begin CORE-7C or any
runtime wiring.

## Expected pre-existing changes

None.
