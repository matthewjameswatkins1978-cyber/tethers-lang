# Current Implementation Task

Control contract: `1`

Task: `TETHERS CORE-4B — Collision-Free Canonical Refinement`

Owner: `OpenCode`

Implementation checkpoint: `31b614727c1f42e2fcab975341c09be35508eefb`

Status: `COMPLETE`

Task colour: `Amber`

Route: `OpenCode implementation + evidence → Lucy independent GitHub review`

Worker note: `docs/worker-notes/2026-08-11-core-4-canonicalisation.md`

Base branch: `feature/core-3-static-validator`

Base commit: `7efec4b1eb69c37c98b3e6b71a7b2e1d8a9260f5`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

Rust change class: `RUST_UNCHANGED`

## Objective

Implement the first authoritative canonical semantic identity layer for Tethers Core: semantic projection, structural fingerprinting, canonical ordering, internal ID assignment, reference rewriting, canonical byte encoding, SHA-256, and ProgramDigest. Same semantic meaning must produce same canonical bytes and ProgramDigest regardless of temporary IDs or storage order.

## Relevant background and existing behaviour

CORE-3 provides a static Core validator. CORE-2 provides the lowering pipeline. Neither provides canonical semantic identity. The Core types (`program`, `origin_site`, `fact`, `role`, etc.) represent semantic meaning but carry temporary internal IDs that are not canonical. No canonical byte representation exists.

## Required behaviour

1. Validate with `Tethers_core_validator.validate` before canonicalising; fail closed on invalid Core
2. Compute structural fingerprints for every semantic entity using iterative refinement over the typed Core graph, excluding raw temporary IDs, behaviour-neutral metadata, and storage position
3. Determine canonical semantic ordering from structural fingerprints; never from raw IDs or list position
4. Assign fresh canonical IDs (O1.., F1.., R1.., B1.., G1.., BA1.., IT1..) in structural order
5. Rewrite every internal reference to use canonical IDs
6. Sort every semantically unordered collection by canonical ID (or fixed semantic order for outcomes)
7. Encode the canonical program as deterministic bytes with version prefix `TETHERS_CORE_CANON_V1\0`
8. Compute SHA-256 over the canonical bytes; produce `sha256:<hex>` ProgramDigest
9. Handle genuinely symmetric structures (identical Together siblings) without falling back to raw IDs
10. Exclude `program_id`, `fact.schema_description`, and `capability_contract.schema_description` from canonical bytes
11. Include `core_version`, all external identities, opaque strings, and all semantic scalar fields in canonical bytes
12. Never reconstruct or manufacture a ProgramDigest outside canonicalisation

## Relevant components

- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical.ml` — new (implementation)
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical.mli` — new (interface)
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_test.ml` — new (28 tests)
- `tethers-0.1/engine-ocaml/bin/dune` — modified (test executable)
- `tethers-0.1/engine-ocaml/tethers_engine.opam` — modified (digestif dep)
- `tethers-0.1/engine-ocaml/tethers_engine.opam.locked` — modified (digestif 1.3.1 + eqaf 0.10)

## Frozen decisions and invariants

- Canonicalisation receives Core meaning; it does not reinterpret Human Tethers, execute, repair invalid Core, infer missing semantics, or use AI
- Incoming/pre-canonical IDs must not influence canonical order
- ProgramDigest = exact semantic-content identity; ProgramId = logical/source identity
- External semantic identities (CapabilityId, CapabilityContractDigest, HostSnapshotKey, CoreVersion, CapabilityInputName) are semantic atoms — never renumbered
- Opaque string-backed placeholders (Deadline, role_fulfillment, batch fields) are exact atoms — not reinterpreted
- Canonicalisation normalises representation; it does not invent semantic equivalences
- No graph-isomorphism theatre: use typed semantic structure, not general isomorphism engines
- CORE-4 remains semantically dormant apart from its tests — not wired into evaluator/runtime

## Acceptance criteria

1. Valid CORE-2 sequential program canonicalises successfully
2. Repeated canonicalisation of identical input produces identical canonical bytes and ProgramDigest
3. Renaming every OriginId consistently produces the same ProgramDigest
4. Renaming all internal IDs (Origin, Fact, Role, Branch, Group, Batch, ItemTemplate) consistently produces the same ProgramDigest
5. Shuffling unordered collections produces the same ProgramDigest
6. Reordering named Action inputs produces the same ProgramDigest
7. Reordering Together members produces the same ProgramDigest
8. Two structurally identical Together siblings survive temporary-ID renaming and storage reversal with identical canonical bytes; multiplicity preserved
9. Reordering outcome branches produces the same ProgramDigest
10. Changing `fact.schema_description` or `capability_contract.schema_description` leaves ProgramDigest unchanged
11. Changing only `program_id` leaves ProgramDigest unchanged
12. Changing an Action literal value produces a different ProgramDigest
13. Changing CapabilityId produces a different ProgramDigest
14. Changing CapabilityContractDigest produces a different ProgramDigest
15. Changing Anchor event_name or path content/order produces a different ProgramDigest
16. Changing comparison operator or expected value produces a different ProgramDigest
17. Changing a success continuation target produces a different ProgramDigest
18. Changing Branch Outcome routing produces a different ProgramDigest
19. Consistent RoleId renaming preserves digest; changing fulfillment semantics changes digest
20. Consistent ItemTemplateId renaming preserves digest; changing objective/template semantics changes digest
21. Changing one opaque Batch semantic field produces a different ProgramDigest
22. Changing CoreVersion produces a different ProgramDigest
23. One semantic member canonicalises differently from two semantic members
24. Invalid Core returns `Invalid_core` and produces no bytes or digest
25. Programs with lexically reversed temporary IDs produce identical canonical bytes

## Required verification

1. OCaml build: `dune build @all` — PASS (exit 0)
2. All tests: `dune runtest` — PASS (28 new + existing validator/lowerer)
3. Whitespace: `git diff --check` — PASS
4. Cargo fmt: `cargo fmt --check` — PASS (RUST_UNCHANGED)
5. Diff inspection: only authorised files changed
6. Git status: clean worktree
7. Task-packet checker at closeout: `control-v1/COMPLETE`
8. Push branch to origin and confirm local HEAD == remote HEAD

## Forbidden changes

No evaluator/protocol/outcome/CORE-2/CORE-3 changes. No Rust changes. No runtime wiring. No Core type changes. No Human Tethers changes.

## Stop conditions

Commit CORE-4 implementation checkpoint. STOP. Do NOT begin CORE-5.

## Expected pre-existing changes

None.
