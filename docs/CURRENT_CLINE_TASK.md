# Current Implementation Task

Control contract: `1`

Task: `TETHERS CORE-6A1 — Human → Core → Plan Proof`

Owner: `OpenCode`

Implementation checkpoint: `7586b29d20133879af47ca8fd0d22878c85710de`

Status: `COMPLETE`

Task colour: `Amber`

Route: `OpenCode implementation + evidence → Lucy independent GitHub review`

Worker note: `docs/worker-notes/2026-08-11-core-6a-anchor-snapshot-binding.md`

Base branch: `feature/core-6-anchor-snapshot-binding`

Base commit: `6e1aba9f2ade3c24c43badc77d20b2094e791f3a`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

Rust change class: `RUST_UNCHANGED`

## Objective

Add the end-to-end test requested by CORE-6A: prove the actual Human → parser → Core lowerer → planner chain works correctly for Anchor snapshot binding. The previous report said this was not possible without parser/lowerer integration changes. Independent review found that it is possible entirely inside the test layer. `Tether_parser.parse_tether` and `Tethers_core_lowerer.lower` are already public APIs. A Dune test-module dependency change is authorised.

## Relevant background and existing behaviour

Human lowering already converts an anchor reference conceptually like `anchor.document.title` into Core `Anchor_value (anchor_origin_id, ["document"; "title"])`. CORE-6A should make that existing Core meaning executable. Do not change the lowerer semantics. The existing Runtime Plan Action vocabulary lives in `tethers_evaluator.ml` (`action_id`, `idempotency_key`, `capability`, `capability_version`, `arguments`, `effects`, plus optional `manifest_digest`/`bridge_capability_version`/`bridge_provider_identity`) and `tethers_protocol.ml` (`capability`). `Tethers_outcome.plan` remains the single Runtime Plan model. CORE-5B introduced `planning_context` with `evaluation_id` and `capabilities` fields.

## Required behaviour

1. Extend `planning_context` with typed Anchor snapshot data
2. For `Anchor_value (O_anchor, ["document"; "title"])`, find the snapshot for exactly `O_anchor`, traverse the path, produce the resolved concrete Runtime Plan argument
3. Anchor snapshot lookup must be deterministic: 0 snapshots → explicit missing-snapshot error; 1 snapshot → use it; 2+ snapshots → explicit ambiguous-snapshot error
4. Support ordered object traversal; fail explicitly when a component is missing, traversal attempts to continue through a non-object, or the terminal value cannot be represented faithfully by the existing Runtime Plan argument vocabulary
5. CORE-6A supports string, integer, boolean terminal values; anything else fails closed with explicit typed error
6. Existing `Literal_value` planning behaviour must remain unchanged; an Action may contain a mixture of `Literal_value` and `Anchor_value`
7. Do NOT add support for `Fact_from_origin`, `Fact_through_role`, `Batch_item_context`; they retain existing fail-closed errors
8. If planning context contains data for `O_other_anchor` but Core requests `O_anchor`, planning must fail; do not fall back to "the only available anchor"

## Relevant components

- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.ml` — modified (anchor snapshot resolution)
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.mli` — modified (anchor snapshot types)
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan_test.ml` — modified (T1..T12)
- `tethers-0.1/engine-ocaml/bin/dune` — modified if test module dependencies require it
- `docs/CURRENT_CLINE_TASK.md` — updated to CORE-6A

## Frozen decisions and invariants

- The bridge consumes Core meaning plus runtime occurrence context plus approved host Capability projections plus runtime Anchor snapshot data; it never reinterprets Core, executes Actions, authorises, repairs invalid Core, infers missing semantics, or uses AI
- Core defines the program; Runtime instantiates an occurrence. Occurrence identity (`plan.id`, idempotency keys) derives from `evaluation_id`, never from `program_id`
- Execution order is semantic control flow only; `origin_sites` order is representational storage and must not affect the plan
- `CapabilityId` and `CapabilityContractDigest` are semantic atoms used to key and verify approved projections; they are not derived from human syntax
- Projection verification is fail-closed: missing projection, identity mismatch, digest mismatch, or incomplete runtime metadata all return precise typed errors; no silent substitution of another capability version or contract
- `required_effects` aggregates planned capability effects with the existing deterministic first-occurrence uniqueness behaviour
- Anchor snapshot lookup is deterministic: identity-based, not first-match, not order-dependent
- Anchor path traversal is ordered semantic data; fail explicitly on missing components, non-object traversal, or unsupported terminal values
- No placeholder strings, `"TODO"` values, fabricated evaluation IDs, or invented runtime semantics may enter a valid plan
- The bridge remains a dormant sidecar; no evaluator, protocol, runtime, canonicalisation, parser, lowerer, Rust, or dispatch wiring changes

## Acceptance criteria

1. T1 — Nested string resolution: snapshot `{"document":{"title":"Tethers"}}`, Core `Anchor_value(O_anchor, ["document"; "title"])`, expected Action argument `"title": "Tethers"`
2. T2 — Integer resolution: resolve nested integer correctly
3. T3 — Boolean resolution: resolve nested boolean correctly
4. T4 — Mixed literal + anchor inputs: one Action contains `Literal_value` and `Anchor_value`; both appear correctly
5. T5 — Missing snapshot: Core references `O_anchor`, no matching snapshot exists, planning fails explicitly
6. T6 — Wrong anchor does not substitute: context contains one snapshot for another OriginId, planning still fails
7. T7 — Duplicate snapshot ambiguity: two snapshots for the same Anchor OriginId, planning fails explicitly
8. T8 — Reversed duplicate snapshot order: reverse those duplicate snapshots, expected identical ambiguity error
9. T9 — Missing path component: snapshot does not contain requested component, planning fails explicitly
10. T10 — Non-object traversal: `{"document":"hello"}` with `["document"; "title"]`, planning fails explicitly
11. T11 — Unsupported terminal JSON: object, array or null at terminal path, planning fails explicitly rather than coercing
12. T12 — Existing fail-closed behaviour: prove `Fact_from_origin` remains unsupported; do not broaden scope

## Required verification

1. OCaml build: `dune build @all` — PASS (exit 0)
2. All tests: `dune runtest` — PASS
3. Whitespace: `git diff --check` — PASS
4. Cargo fmt: `cargo fmt --check` — PASS (RUST_UNCHANGED)
5. Diff inspection: only authorised files changed
6. Git status: clean worktree
7. Task-packet checker at closeout: `control-v1/COMPLETE`
8. Push branch to origin and confirm local HEAD == remote HEAD

## Forbidden changes

No Core type changes, no validator semantic changes, no evaluator/protocol/outcome changes, no runtime wiring, no production dispatch, no Rust changes, no new dependencies. Do not broaden unsupported-construct support. Do not modify Core Validator semantics in this packet. Do not change Human syntax or lowerer semantics.

## Stop conditions

Commit CORE-6A implementation checkpoint. STOP. Do NOT begin CORE-6B or any runtime wiring.

## Expected pre-existing changes

None.