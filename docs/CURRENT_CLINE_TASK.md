# Current Implementation Task

Control contract: `1`

Task: `TETHERS CORE-5A — Minimal Core → Runtime Plan Bridge`

Owner: `OpenCode`

Implementation checkpoint: `WORKTREE`

Status: `IN_PROGRESS`

Task colour: `Amber`

Route: `OpenCode implementation + evidence → Lucy independent GitHub review`

Worker note: `docs/worker-notes/2026-08-11-core-5a-runtime-plan-bridge.md`

Base branch: `feature/core-4-canonicalisation-program-digest`

Base commit: `b29b0d348d057cec19faf544f64b64989111fa09`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

Rust change class: `RUST_UNCHANGED`

## Objective

Create the first executable sidecar bridge from validated Tethers Core to the existing Runtime Plan representation (`Tethers_outcome.plan`). Prove the architecture with the smallest useful vertical slice: `Core Program → validate → Core → Runtime Plan bridge → sequential executable plan`. Do not replace the legacy evaluator and do not wire production dispatch.

## Relevant background and existing behaviour

CORE-1..CORE-4 provide Core types, lowering, validation, and canonicalisation. The legacy evaluator (`tethers_evaluator.ml`) plans directly from the parser AST into `Tethers_outcome.plan` (`plan`, `group_plan`, `planned_action` as Yojson, plus `Matched`/`Not_matched`/`Evaluation_error` payloads). No bridge exists from Core meaning to that plan vocabulary. `Tethers_outcome.plan` is the existing Runtime Plan model; it must be reused, not duplicated.

## Required behaviour

1. `plan : Tethers_core.program -> (Tethers_outcome.plan, planning_error) result` validates Core with `Tethers_core_validator.validate` first; invalid Core returns an explicit error and never produces a plan
2. Sequential execution order derives from semantic control flow (`entry_origin` then `success_continuation` edges, stopping at `Program_complete`), never from `origin_sites` storage order
3. Every planned Action preserves `CapabilityId` and `CapabilityContractDigest` exactly, without resolving capability meaning from human syntax or inventing host authorisation
4. Literal Action inputs translate to concrete plan argument values; any binding that cannot be represented faithfully by the existing plan layer returns an explicit typed error
5. `Together_origin`, `Batch_site`, branch-driven control flow, `Fact_through_role`, `Role_proxy` facts, item templates, and execution constraints fail closed with precise typed errors and never partially plan
6. Reuse `Tethers_outcome.plan` as the Runtime Plan representation; do not create a second competing Runtime Plan model
7. No placeholder strings, `"TODO"` values, fabricated evaluation IDs, or invented runtime semantics may enter a valid plan
8. Remain a dormant sidecar: no evaluator, protocol, runtime, canonicalisation, parser, lowerer, Rust, or dispatch wiring changes

## Relevant components

- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.ml` — new (bridge implementation)
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.mli` — new (bridge interface)
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan_test.ml` — new (T1..T8 plus fail-closed branch tests)
- `tethers-0.1/engine-ocaml/bin/dune` — modified (new test executable stanza)

## Frozen decisions and invariants

- The bridge consumes Core meaning; it does not reinterpret Human Tethers, execute Actions, authorise, repair invalid Core, infer missing semantics, or use AI
- Execution order is semantic control flow only; `origin_sites` order is representational storage and must not affect the plan
- `CapabilityId` and `CapabilityContractDigest` are semantic atoms preserved exactly on every planned Action
- The existing Runtime Plan carries concrete resolved argument values and has no event-data vocabulary; therefore `Anchor_value` and `Fact_from_origin` bindings cannot be faithfully represented by this bridge and must return explicit typed errors (representation incompatibility, not silent skip)
- Execution constraints (e.g. `Deadline`) have no existing runtime-plan vocabulary; their presence returns an explicit typed error
- `plan.id` uses the program's logical identity (`program_id`); no evaluation ID is fabricated
- The bridge must never construct a valid plan containing a placeholder; every field is either real Core content or absent
- CORE-5A remains a sidecar; it is not wired into the evaluator, MCP, or runtime

## Acceptance criteria

1. T1 — Minimal Action: `Anchor → Action A → Program_complete` plans exactly one executable Action with correct capability identity and inputs
2. T2 — Sequential Two Actions: `Anchor → A → B → Program_complete` plans A then B in control-flow order
3. T3 — Storage Order Independence: identical control-flow graphs with reversed `origin_sites` storage produce identical Runtime Plans
4. T4 — Capability Digest Preservation: the planned Action carries `CapabilityId` and `CapabilityContractDigest` exactly
5. T5 — Unsupported Together: a valid Core program containing `Together_origin` returns an explicit unsupported error and plans no partial siblings
6. T6 — Unsupported Batch: a valid Core program containing `Batch_site` returns an explicit unsupported error
7. T7 — Unsupported Role binding: a valid Core Action using `Fact_through_role` returns an explicit unsupported error
8. T8 — Invalid Core: an invalid Core program returns `Error` and never a plan
9. T9 — An `Anchor_value` binding returns an explicit representation-incompatibility error
10. T10 — A `Fact_from_origin` binding returns an explicit representation-incompatibility error
11. T11 — Branch semantics returns an explicit unsupported error
12. T12 — An execution constraint (Deadline) returns an explicit unsupported error
13. T13 — Item templates return an explicit unsupported error
14. T14 — A program with no `entry_origin` returns an explicit missing-entry error

## Required verification

1. OCaml build: `dune build @all` — PASS (exit 0)
2. All tests: `dune runtest` — PASS (T1..T14 plus existing validator/lowerer/canonical)
3. Whitespace: `git diff --check` — PASS
4. Cargo fmt: `cargo fmt --check` — PASS (RUST_UNCHANGED)
5. Diff inspection: only authorised files changed
6. Git status: clean worktree
7. Task-packet checker at closeout: `control-v1/COMPLETE`
8. Push branch to origin and confirm local HEAD == remote HEAD

## Forbidden changes

No evaluator/protocol/outcome/CORE-1..CORE-4 changes. No Rust changes. No runtime wiring or production dispatch. No Core type changes. No Human Tether, parser, lowerer, or canonicalisation changes. No new dependencies. No engine entry-point changes.

## Stop conditions

Commit CORE-5A implementation checkpoint. STOP. Do NOT begin CORE-5B or any runtime wiring.

## Expected pre-existing changes

None.
