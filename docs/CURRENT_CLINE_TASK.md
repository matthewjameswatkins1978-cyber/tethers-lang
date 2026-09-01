# Rocket V3 — R3-0 Complete Semantic Relation Inventory

Control contract: `1`

Status: `COMPLETE`

Task colour: `Red`

Owner: `Codex`

Route: `Codex evidence/design implementation in a fresh dedicated worktree; no production Rocket V3 code in R3-0`

Base commit: `5a1b461dcb95852681f269cd13a63a1e80695795`

Implementation checkpoint: `0fdcde0fd9f268513c5975062685af862955c0cd`

Worker note: `docs/worker-notes/2026-09-01-rocket-v3-r3-0.md`

Related issue: `#5 — BUG: Rocket V2 factorial search on simple sequential Action chains`

Updated: 2026-09-01

## Objective

Produce the complete, reviewable semantic-relation inventory that Rocket V3 will use as its structural canonicalisation model. R3-0 must account for every identity-bearing Core reference and every anonymous-label lookup that can influence frozen `Enc_V2` bytes, classify its direction/inverse/multiplicity/scope, and identify the exact V3 relation or structural concept needed to expose it to later canonical refinement.

R3-0 is a design/evidence gate. It MUST NOT implement Rocket V3 refinement, search, prefix pruning, automorphism pruning, component recursion, or change production canonical identity.

## Relevant background and existing behaviour

Rocket V2 is now the accepted live production semantic identity engine on post-cutover `main`. New production ProgramDigest identity uses the frozen `tethers:v2:sha256:<64 lowercase hex>` contract. V1 is not a live fallback.

Rocket V2 is exact against the V2 oracle/baseline, but its refinement relation model is incomplete. In particular, semantic relationships such as success-continuation/control-flow are not fully represented in refinement, allowing structurally simple sequential Action chains to remain artificially symmetric and enter factorial search. GitHub issue #5 records the resulting budget cliff.

Rocket V3 is intended to change how the frozen Enc_V2 minimum is found, not initially change Enc_V2 or ProgramDigest_V2. The planned architecture is a complete typed relational model followed by canonical partition refinement and later individualisation/refinement search.

The repository contains many historical Rocket branches. They are evidence only. This task starts from the exact post-cutover `main` base above and must not import implementation by rebasing, merging, cherry-picking, or copying wholesale from historical Rocket worktrees.

## Required behaviour

1. Inventory every anonymous identity family and every Core field/reference whose value can influence an Enc_V2 anonymous label, directly or transitively.
2. Inventory every `labels_for_*` / anonymous-label lookup and every relevant labelled-byte emission in the frozen Enc_V2 encoder, and map each lookup back to its originating Core semantic reference.
3. Define the proposed Rocket V3 typed relation for each semantic reference, including forward direction, inverse direction, relation discriminator, multiplicity semantics, scope semantics, and whether a structural sentinel is required.
4. Explicitly classify `Action_origin`, `Anchor_origin`, and `Together_origin` as `origin_site` constructors and anonymous Origin-family identities. Classify `Batch_site` as an `origin_site` constructor structurally, but not as an Origin-family identity: it carries the separate anonymous Batch-family `batch_id`, is excluded by `origin_id_of_site`, and is handled by `collect_batches`/`BatchMap`.
5. Explicitly classify `EntryGuard` as the existing `fact_guard` structure, not a new anonymous identity family; classify `ProgramComplete` as the existing `control_target` terminal; and classify `ProgramRoot` and `ProgramScope` as new V3 structural concepts/sentinels, not new anonymous canonical identity families.
6. Confirm or correct the proposed anonymous identity-family set: `Origin`, `Fact`, `Branch`, `Batch`, `ItemTemplate`, and `ScopedRole`. Any proposed addition/removal is a Red architectural finding and must be reported, not silently adopted.
7. Produce an explicit coverage matrix showing that every relevant Core reference and every Enc_V2 label lookup is covered exactly once or intentionally cross-referenced, with no unexplained gaps.
8. Identify the minimum R3-1 relational-model implementation surface and the tests/proofs that will be required, but do not implement it.

## Relevant components

Authorised mutation is limited to:

- `docs/CURRENT_CLINE_TASK.md`
- `docs/review/rocket-v3/R3_0_SEMANTIC_RELATION_INVENTORY.md`
- `docs/worker-notes/2026-09-01-rocket-v3-r3-0.md`

Required read-only implementation/spec evidence includes at minimum:

- `tethers-0.1/engine-ocaml/bin/tethers_core.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_validator.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_lowerer.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_format.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_format.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_reference.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_ir.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_ir.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_ir_test.ml`
- `docs/review/lucy-c-b4s-canonical-v2/CANONICAL_FORMAT_V2_SPEC_DRAFT.md`
- GitHub issue #5 and the accepted Rocket V2 cutover worker note.

## Frozen decisions and invariants

- R3-0 is evidence/design only. No production OCaml/Rust implementation changes.
- Rocket V3 initially targets the exact frozen Enc_V2 byte minimum and existing `tethers:v2:sha256:` ProgramDigest contract.
- Raw Core IDs, source storage order, hash iteration order, search selector, wall-clock timing, and machine architecture are not semantic identity.
- Every semantic directed reference considered by V3 refinement must account for information flow in both directions. The inventory must record explicit forward and inverse relation meaning even if a later compact implementation stores them in shared adjacency structures.
- Relation multiplicity is semantic data where repeated references are permitted; it must never be collapsed to set membership without proof.
- Different refinement colours may prove distinguishability. Equal refinement colours never prove automorphism or canonical identity.
- No greedy canonical label assignment from a non-singleton refinement cell is authorised in R3-0 or implied for R3-1.
- Prefix pruning, automorphism pruning, and component recursion are later optimisation phases and are not prerequisites for the first correct V3 relational model/search.
- No V1 fallback.
- No new dependency.
- No Human Tether grammar, Core semantics, policy, Plug, provider, Trail, replay, Together execution, or runtime-authority redesign.
- Historical Rocket branches are evidence only and are not implementation bases.

## Acceptance criteria

1. The inventory names all six current anonymous identity families, with repository evidence for each, and separately lists structural/non-anonymous concepts.
2. Every anonymous-label lookup in the frozen Enc_V2 encoder is represented in the coverage matrix with source location, identity family, semantic owner/reference, and V3 relation mapping.
3. Every Core field/reference that can feed those label lookups is represented with forward and inverse relation meaning, discriminator, multiplicity, and scope.
4. The report explicitly records `Action_origin`, `Anchor_origin`, and `Together_origin` as `origin_site` constructors and Origin-family identities, and records `Batch_site` as a structural `origin_site` constructor belonging to the separate anonymous Batch identity family.
5. The report explicitly records `EntryGuard = fact_guard`, `ProgramComplete = control_target terminal`, and `ProgramRoot`/`ProgramScope` as V3 structural concepts rather than anonymous identity families.
6. Any mismatch between the proposed six-family model and actual Core/Enc_V2 evidence is surfaced as a blocking architectural finding instead of being papered over.
7. The final matrix has no unexplained Core-reference or Enc_V2-label-lookup gaps; any intentionally excluded scalar/non-anonymous field states why it cannot require an anonymous V3 relation.
8. The report ends with a bounded R3-1 implementation surface and proof/test list sufficient for Lucy to authorise or reject the first V3 relational-model code task.
9. `git diff --check` passes and the final diff contains only the three authorised documentation paths.
10. The task-packet checker passes in `control-v1/COMPLETE` state with a worker note that cites the exact implementation/evidence checkpoint and final repository state.

## Required verification

- Complete the AGENTS.md startup report from a fresh worktree created from `origin/feature/rocket-v3-r3-0`.
- Run `pwsh -NoProfile -File scripts/check-dev-tools.ps1`.
- Run `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` before mutation and record the `control-v1/READY` result.
- Use repository search plus direct source reading to enumerate every anonymous-label lookup in `tethers_core_canonical_v2_format.ml` and every corresponding Core reference.
- Cross-check the inventory independently against the V2 reference/baseline/Rocket modules so the matrix is not derived from one implementation alone.
- Inspect the complete branch diff.
- Run `git diff --check`.
- Confirm only the authorised documentation files changed.
- Commit the evidence/report checkpoint, capture its exact full SHA, then create the worker note and mark the packet `COMPLETE`.
- Run the packet checker again and require `control-v1/COMPLETE`.
- Push normally to `origin/feature/rocket-v3-r3-0`, confirm local `HEAD == remote HEAD`, and report clean status.

No OCaml/Rust build is required for a documentation-only R3-0 task unless the investigation discovers that executable evidence is necessary to resolve a specific ambiguity. Do not modify code merely to create such evidence.

## Forbidden changes

- No edits outside the three authorised documentation paths.
- No Rocket V3 production modules.
- No changes to Rocket V2, Enc_V2, ProgramDigest, canonical vectors, Core types, validator, lowerer, planner, wire, Rust host, Dune, Cargo, dependencies, CI, or language syntax.
- No cherry-pick, merge, rebase, reset, force-push, or code copying from historical Rocket branches.
- No implementation of 1-WL/partition refinement, I/R search, prefix pruning, automorphism/orbit pruning, component recursion, deterministic search budgets, or external graph-canonicaliser integration in R3-0.
- No assumption that a proposed relation is complete merely because Rocket V2 currently models it.
- No treating raw IDs, array/list position, or current V2 heuristic colour rank as semantic authority.
- Do not begin R3-1 automatically after completion.

## Stop conditions

- Actual Core or Enc_V2 evidence contradicts the proposed six anonymous identity families in a way that changes the V3 architecture.
- A label-bearing semantic reference cannot be classified without changing frozen Enc_V2 semantics.
- Two materially similar inventory approaches still leave the same unexplained coverage gap.
- The checkout, branch, HEAD/base, or packet state differs from this READY task after fetching origin.
- Completing the inventory would require production code mutation, dependency changes, or importing historical branch implementation.
- A consequential ambiguity remains over scope identity, multiplicity, structural sentinels, or direction/inverse semantics that cannot be resolved from current authoritative repository evidence.

## Expected pre-existing changes

None.
