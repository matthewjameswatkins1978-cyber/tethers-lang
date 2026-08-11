# Current Implementation Task

Control contract: `1`

Task: `TETHERS CORE-5B — Runtime Plan Contract and Terminal-Path Correction`

Owner: `OpenCode`

Implementation checkpoint: `b0e194b2da9331ef2455674a30bd427a5d1873d8`

Status: `COMPLETE`

Task colour: `Amber`

Route: `OpenCode implementation + evidence → Lucy independent GitHub review`

Worker note: `docs/worker-notes/2026-08-11-core-5b-runtime-plan-contract.md`

Base branch: `feature/core-5-runtime-plan-bridge`

Base commit: `a28bdf483db7959f5471b4b950802e863093d9f8`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

Rust change class: `RUST_UNCHANGED`

## Objective

CORE-5A is ACCEPTED WITH CORRECTION. Do not redesign Core. Correct the two independently reviewed Runtime Plan boundary defects (success paths must terminate explicitly; runtime occurrence identity must not be ProgramId), then bring the bridge into the existing Runtime Plan Action contract using an explicit runtime planning context and approved, digest-pinned capability projections.

## Relevant background and existing behaviour

CORE-5A (`tethers_core_plan.ml/.mli`) walks `entry_origin → success_continuation` and currently treats a missing continuation as successful completion. It sets `plan.id = program_id`, emits a nonstandard `capability_contract_digest` action field, and has no runtime occurrence context or capability projections. The existing Runtime Plan Action vocabulary lives in `tethers_evaluator.ml` (`action_id`, `idempotency_key`, `capability`, `capability_version`, `arguments`, `effects`, plus optional `manifest_digest`/`bridge_capability_version`/`bridge_provider_identity`) and `tethers_protocol.ml` (`capability`). `docs/CAPABILITY_BRIDGE.md` defines the manifest/digest pinning contract. `Tethers_outcome.plan` remains the single Runtime Plan model.

## Required behaviour

1. `Incomplete_success_path of origin_id`: every reachable sequential path must reach `Program_complete` explicitly; running out of continuation is incomplete meaning, not completion
2. `plan.id` derives from the runtime occurrence context (`evaluation_id ^ "/plan"`), never from `program_id`; `program_id` remains Core logical identity only
3. Every planned Action carries the existing Runtime Plan Action contract fields: `action_id`, `idempotency_key` (`evaluation_id ^ "/action_N"`), `capability`, `capability_version`, `arguments`, `effects`, plus the projection's bridge fields when present; do not invent missing values
4. Introduce the smallest clear typed runtime planning context carrying the runtime occurrence `evaluation_id` and approved capability projections
5. Add an explicit trusted `runtime_capability_projection` keyed and pinned by Core `CapabilityId` + `CapabilityContractDigest`, carrying the runtime capability name/version, effects, and manifest/bridge metadata where applicable; reuse existing types where they fit cleanly
6. Planning fails closed when a projection is missing, its CapabilityId does not match, its pinned digest does not match, or required runtime capability metadata is unavailable; never silently substitute another capability version or contract
7. Populate `required_effects` from the planned capabilities using the existing deterministic first-occurrence uniqueness behaviour; do not infer effects from capability names
8. Preserve all CORE-5A fail-closed behaviour (Together, Batch, Branch, Fact_through_role, Anchor_value, Fact_from_origin, Deadline, ItemTemplate, invalid Core) without broadening support

## Relevant components

- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.ml` — modified (bridge implementation)
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.mli` — modified (bridge interface)
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan_test.ml` — modified (T1..T13)
- `tethers-0.1/engine-ocaml/bin/dune` — modified (plan test module list gains `tethers_protocol tether_parser tethers_error`)
- `docs/CURRENT_CLINE_TASK.md` — updated to CORE-5B

## Frozen decisions and invariants

- The bridge consumes Core meaning plus runtime occurrence context plus approved host Capability projections; it never reinterprets Core, executes Actions, authorises, repairs invalid Core, infers missing semantics, or uses AI
- Core defines the program; Runtime instantiates an occurrence. Occurrence identity (`plan.id`, idempotency keys) derives from `evaluation_id`, never `program_id`
- Execution order is semantic control flow only; `origin_sites` order is representational storage and must not affect the plan
- `CapabilityId` and `CapabilityContractDigest` are semantic atoms used to key and verify approved projections; they are not derived from human syntax
- Projection verification is fail-closed: missing projection, identity mismatch, digest mismatch, or incomplete runtime metadata all return precise typed errors; no silent substitution of another capability version or contract
- `required_effects` aggregates planned capability effects with the existing deterministic first-occurrence uniqueness behaviour
- No placeholder strings, `"TODO"` values, fabricated evaluation IDs, or invented runtime semantics may enter a valid plan
- The bridge remains a dormant sidecar; no evaluator, protocol, runtime, canonicalisation, parser, lowerer, Rust, or dispatch wiring changes

## Acceptance criteria

1. T1 — `A → Program_complete` plans one Action with correct occurrence id and capability identity
2. T2 — `A → B` with B having no continuation returns `Incomplete_success_path(B)`
3. T3 — `evaluation_id = eval_123`, `program_id = MY_PROGRAM` yields `plan.id = eval_123/plan` and not `MY_PROGRAM`
4. T4 — equivalent Programs with different ProgramIds and identical occurrence context yield the same occurrence-derived `plan.id`
5. T5 — one literal Action carries `action_id`, `idempotency_key`, `capability`, `capability_version`, `arguments`, `effects`, and pinned digest/bridge fields exactly
6. T6 — a missing capability projection fails explicitly
7. T7 — a capability identity match with a mismatched pinned digest fails explicitly
8. T8 — two Actions with overlapping effects yield deterministic unique `required_effects`
9. T9 — two Actions under `evaluation_id = eval_X` yield `eval_X/action_1` and `eval_X/action_2`
10. T10 — reversed `origin_sites` with identical control flow and context produce equal Runtime Plans
11. T11 — Together, Batch, Branch, Fact_through_role, Anchor_value, Fact_from_origin, Deadline, ItemTemplate, and invalid Core all fail closed exactly as in CORE-5A
12. T12 — a contract digest approved only under a different CapabilityId fails with the identity-mismatch error
13. T13 — an approved projection with incomplete runtime metadata fails closed

## Required verification

1. OCaml build: `dune build @all` — PASS (exit 0)
2. All tests: `dune runtest` — PASS (lowerer 49/49, validator 51/51, plan bridge 43/43)
3. Whitespace: `git diff --check` — PASS
4. Cargo fmt: `cargo fmt --check` — PASS (RUST_UNCHANGED)
5. Diff inspection: only authorised files changed
6. Git status: clean worktree
7. Task-packet checker at closeout: `control-v1/COMPLETE`
8. Push branch to origin and confirm local HEAD == remote HEAD

## Forbidden changes

No Core type changes, no validator semantic changes, no evaluator/protocol/outcome changes, no runtime wiring, no production dispatch, no Rust changes, no new dependencies. Do not broaden unsupported-construct support. Do not modify Core Validator semantics in this packet.

## Stop conditions

Commit CORE-5B implementation checkpoint. STOP. Do NOT begin CORE-5C or any runtime wiring.

## Expected pre-existing changes

None.
