# Current Implementation Task

Control contract: `1`

Task: `TETHERS CORE-6B — Canonical Core → Runtime Plan Boundary`

Owner: `OpenCode`

Implementation checkpoint: `dac6cce92287b2ad853b3f435063c96359c8d1e5`

Status: `COMPLETE`

Task colour: `Amber`

Route: `OpenCode implementation + evidence → Lucy independent GitHub review`

Worker note: `docs/worker-notes/2026-08-11-core-6b-canonical-planning-boundary.md`

Base branch: `feature/core-6b-canonical-planning`

Base commit: `534abc763938f573fa799619ffa22193206e3b15`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

Rust change class: `RUST_UNCHANGED`

## Objective

Make the canonical Core → Runtime Plan boundary explicit and tested: add a
`plan_canonicalized` entry point requiring already-canonicalised Core, preserve
the existing `plan` function as the lower-level API, and prove identity
invariance across ProgramId, temporary OriginId, and storage order variations.

## Relevant background and existing behaviour

CORE-4 implements deterministic canonicalisation: structural fingerprinting,
canonical ordering, internal ID assignment, reference rewriting, canonical byte
encoding, SHA-256, and ProgramDigest. CORE-6A added the Core → Runtime Plan
bridge with anchor snapshot resolution. CORE-6B proves that Runtime planning
is performed from canonical Core, with canonical internal identities, while
preserving the existing Runtime Plan model.

## Required behaviour

1. Add `canonical_plan` type wrapping `ProgramDigest` + existing `Tethers_outcome.plan`
2. Add `plan_canonicalized` entry point requiring `Tethers_core_canonical.canonicalized`
3. Preserve existing `plan` function as lower-level API
4. Anchor snapshot lookup must use canonical OriginId, not pre-canonical
5. Prove Human → parser → lowerer → canonicalise → planner chain works end-to-end
6. Prove ProgramId variation leaves digest and occurrence plan unchanged
7. Prove pre-canonical temporary ID/storage variation canonicalises to equal plans
8. Prove stale pre-canonical Anchor OriginId fails with existing typed error
9. Prove existing CORE-6A planner tests remain green through both paths

## Relevant components

- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.ml` — modified (canonical_plan type + plan_canonicalized)
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.mli` — modified (canonical_plan + plan_canonicalized interface)
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan_test.ml` — modified (CB-T1..CB-T8)
- `tethers-0.1/engine-ocaml/bin/dune` — modified (tethers_core_canonical + digestif added to plan-test stanza)
- `docs/CURRENT_CLINE_TASK.md` — updated to CORE-6B

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
- `ProgramDigest` = semantic program identity; `evaluation_id` = runtime occurrence identity. Do not conflate them.

## Acceptance criteria

1. CB-T1 — Canonicalized entry point produces Runtime Plan
2. CB-T2 — Returned ProgramDigest equals canonicalized ProgramDigest
3. CB-T3 — Human → parser → lowerer → canonicalize → planner Anchor_value proof
4. CB-T4 — ProgramId variation leaves digest and occurrence plan unchanged
5. CB-T5 — Pre-canonical temporary ID/storage variation canonicalises to equal plans
6. CB-T6 — Anchor snapshot keyed by canonical OriginId resolves
7. CB-T7 — Stale pre-canonical Anchor OriginId does not silently substitute
8. CB-T8 — Existing CORE-6A planner tests remain green (low-level + canonical)
9. No unsafe canonical_plan constructor exists; canonicalized values are only obtainable from Tethers_core_canonical.canonicalize

## Required verification

1. OCaml build: `dune build @all` — PASS (exit 0)
2. All tests: `dune runtest` — PASS (101/101 plan bridge tests)
3. Whitespace: `git diff --check` — PASS
4. Cargo fmt: `cargo fmt --check` — PASS (RUST_UNCHANGED)
5. Diff inspection: only authorised files changed
6. Git status: clean worktree
7. Task-packet checker at closeout: `control-v1/COMPLETE`
8. Push branch to origin and confirm local HEAD == remote HEAD

## Forbidden changes

No Core type changes, no validator semantic changes, no evaluator/protocol/outcome changes, no runtime wiring, no production dispatch, no Rust changes, no new dependencies. Do not broaden unsupported-construct support. Do not modify Core Validator semantics in this packet. Do not change Human syntax or lowerer semantics. Do not change canonicalisation semantics.

## Stop conditions

Commit CORE-6B implementation checkpoint. STOP. Do NOT begin CORE-6C or any runtime wiring.

## Expected pre-existing changes

None.
