# Rocket V3 — R3-2 Stable Typed Partition Refinement

Control contract: `1`

Status: `COMPLETE`

Task colour: `Red`

Owner: `Codex`

Route: `Codex implementation in a fresh dedicated worktree; partition/refinement engine and proofs only`

Base commit: `546c778425386dd61ec91422cf01cddb1e40bfbe`

Implementation checkpoint: `1b32cab71ddc472f0a5c97549f8657872a45a6e0`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

OCaml toolchain contract: use this exact external directory switch with explicit `--switch`; run Dune against the current R3-2 worktree source tree. Do not create, copy, move, select globally, or substitute another installed switch. For repository scripts that invoke opam without `--switch`, set `OPAMSWITCH` only process-locally to this exact path.

Worker note: `docs/worker-notes/2026-09-01-rocket-v3-r3-2-refinement.md`

Related issue: `#5 — BUG: Rocket V2 factorial search on simple sequential Action chains`

Design authorities:

- `docs/review/rocket-v3/R3_0_SEMANTIC_RELATION_INVENTORY.md`
- accepted R3-1 model at base `546c778425386dd61ec91422cf01cddb1e40bfbe`

Updated: 2026-09-01

## Objective

Implement deterministic typed partition refinement over the accepted immutable Rocket V3 semantic model.

R3-2 must compute the unique stable/equitable refinement induced by vertex kind/scalar descriptors and the complete typed directed relation multigraph. It must use an incremental worklist/splitter design with smaller-half scheduling rather than repeated factorial search or raw-ID tie-breaking.

The decisive proof target is the issue-#5 family: a homogeneous sequential Action chain whose Actions have identical scalar payloads must become discrete by semantic refinement alone for sizes 1, 10, 50, 100, 250, 500 and 1000.

R3-2 does not individualize vertices and does not search. A non-singleton stable cell is a truthful statement of remaining ambiguity, not permission to invent an ordering.

## Relevant background and existing behaviour

R3-1 is now on `main` and supplies:

- exactly six anonymous identity families;
- fixed ProgramRoot, ProgramScope, ProgramComplete and BranchStop structural vertices;
- deterministic scalar descriptors;
- complete typed forward and reverse adjacency;
- exact relation discriminators and payloads;
- multiplicity and scope;
- validation-first construction.

The model exposes internal integer vertex handles only as implementation handles. They are not semantic identity and MUST NOT be used to split otherwise equivalent vertices.

Standard individualisation/refinement canonicalisation starts from an invariant initial colouring, computes an equitable partition, then only searches if non-singleton cells remain. R3-2 implements only that root refinement stage.

## Required behaviour

1. Add an abstract partition module over R3-1 model vertices. Initialise the partition solely from explicit semantic base keys: `vertex_kind` plus `vertex_scalar`. Fixed structural vertices must begin distinguishable by kind. Internal vertex numbers and input order must not affect grouping.
2. Add an incremental refinement module that reaches a stable typed equitable partition using worklist/splitter processing. Do not implement repeated whole-program canonical encoding or any permutation search.
3. Treat an edge channel as the combination of direction, `relation_kind`, `relation_discriminator` and exact relation payload. Forward and inverse incidence must remain distinguishable. Multiplicity to/from a splitter cell is counted exactly.
4. Split a candidate cell whenever two vertices have different multiplicity counts for any typed edge channel into the active splitter cell. Zero incidence is a real count and must distinguish zero from one or more.
5. Use smaller-half worklist scheduling when a non-active cell is split: enqueue all resulting parts except one deterministic largest part. If the old cell is already active, replace/update its active work consistently so no required splitter is lost. Equal-size choices must be resolved by semantic/invariant subgroup order, never raw vertex number.
6. Make refinement scheduling deterministic: splitter selection, channel processing, affected-cell processing and subgroup ordering must not depend on hash iteration, raw IDs, source collection order, pointer identity or wall clock.
7. Expose only partition/refinement evidence needed by later phases and tests: cell count, cell membership queries, cell sizes, discreteness, stable-state indication and deterministic work statistics. Cell handles/colour numbers are refinement handles, not canonical labels.
8. Record deterministic work statistics at minimum: `relation_visits`, `splitter_pops`, `cell_splits`, `max_worklist` and final cell count. No wall-clock value may influence the result.
9. Prove the stable result against an independent slow test-only reference refinement on a deterministic generated corpus of small valid Core programs. Compare the induced equivalence relation/cell partition, not incidental internal cell numbers.
10. Add homogeneous sequential Action-chain fixtures where every Action has the same capability, contract digest, inputs, facts and constraints, differing only in raw identity and position in the root/success/complete structure. Sizes 1, 10, 50, 100, 250, 500 and 1000 must refine to singleton Action cells with no individualisation/search.
11. Prove that genuine unresolved symmetry is not broken artificially. Structurally indistinguishable twins/symmetric valid fixtures must remain in the same stable cell unless semantic relations distinguish them.
12. Integrate only the new partition/refinement modules and focused tests into Dune. Do not wire Rocket V3 into production canonicalisation, planning, wire, Rust host or ProgramDigest.

## Relevant components

Authorised mutation is limited to:

- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-09-01-rocket-v3-r3-2-refinement.md`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_partition.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_partition.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_refine.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_refine.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_refine_test.ml`
- `tethers-0.1/engine-ocaml/bin/dune`

Read-only implementation authorities include:

- `tethers_core_rocket_v3_model.ml/.mli`
- `tethers_core_rocket_v3_model_test.ml`
- `tethers_core.ml/.mli`
- `tethers_core_validator.ml/.mli`
- `tethers_core_canonical_v2_format.ml/.mli`
- V2 oracle/production/IR modules and tests for regression evidence only.

## Frozen decisions and invariants

- The R3-1 model is semantic input authority for R3-2. Do not duplicate/reinterpret Core relations independently in the refinement engine.
- Frozen Enc_V2 and `tethers:v2:sha256:` ProgramDigest semantics do not change.
- Refinement may prove vertices distinguishable. Equal stable cells do not prove automorphism or canonical identity.
- No raw ID, raw model vertex handle, collection position or current V2 heuristic may split a cell.
- Relation direction, kind, discriminator, payload and multiplicity are all observable refinement information.
- A stable partition must be equitable for every typed directed edge channel.
- Initial partition keys are semantic base descriptors only. Do not seed refinement with V2 canonical labels, raw IDs, Enc_V2 labels or search results.
- Fixed ProgramRoot, ProgramScope, ProgramComplete and BranchStop are structural colours, not anonymous identity families.
- Batch remains a distinct anonymous family, never an Origin shortcut.
- The complete refinement result must be independent of valid splitter-processing order as an equivalence relation; the implementation schedule itself must nevertheless be deterministic for reproducible statistics.
- Smaller-half scheduling is an efficiency mechanism, not identity authority.
- No external graph dependency.
- No wall-clock timeout or budget in R3-2.
- No V1 fallback.
- No individualisation, search, canonical label assignment or candidate encoding.

## Acceptance criteria

1. Initial partition construction groups vertices only by semantic `vertex_kind + vertex_scalar`, with each fixed structural kind distinguishable and no use of raw vertex number.
2. The refinement algorithm terminates with a stable partition in which vertices sharing a cell have identical typed incoming/outgoing multiplicity counts to every final cell.
3. Direction, relation kind, discriminator and payload independently affect refinement; removing/changing any one in focused fixtures changes the expected split behaviour.
4. Multiplicity is preserved: zero/one/two-or-more incidences can split cells where semantically present, and no relation occurrence is silently set-collapsed.
5. Worklist split updates implement the smaller-half rule for non-active split cells and preserve correctness when an already-active cell splits.
6. Repeated runs over identical input report identical final partition evidence and identical deterministic work statistics.
7. Renaming every raw nominal ID and permuting all representation collections leaves the final partition equivalence structure and deterministic statistics unchanged for paired fixtures.
8. Internal model vertex numbering/insertion perturbation cannot create a refinement distinction. Tests must not use raw vertex IDs as semantic sort keys.
9. The independent slow reference refinement and incremental R3-2 refinement induce the same stable equivalence relation over every case in the deterministic generated small-program corpus.
10. Homogeneous Action chains of 1, 10, 50, 100, 250, 500 and 1000 Actions finish with every Action in a singleton cell; the 1000-Action case performs no search because no search exists in R3-2.
11. The homogeneous-chain test proves scalar equality first: all Action vertices in the fixture share the same initial semantic key before control-flow refinement.
12. At least one valid symmetric/twin fixture remains non-discrete after stable refinement, proving R3-2 does not manufacture identity from handles/order.
13. ProgramRoot propagation, ProgramComplete propagation and directed success-next/success-prev structure are each necessary/observable in focused mutation tests.
14. Batch, role-scope, Together-member, Branch outcome/Stop, Action-binding and multiplicity fixtures all refine using the R3-1 typed relation channels without special-case raw Core logic in the refinement module.
15. Statistics include relation visits, splitter pops, cell splits, max worklist and final cell count. They are deterministic and are evidence only, never Enc_V2 bytes or identity.
16. The 1000-Action homogeneous-chain relation-visit count is recorded in the worker note and demonstrates bounded incremental behaviour; any unexpectedly quadratic/explosive result is a Red performance finding and must be reported rather than hidden behind a larger budget.
17. The public R3-2 API contains no individualise/search/canonical-label/digest/candidate-emission operation.
18. `dune build @all`, the focused R3-2 test executable, `dune runtest --force`, `git diff --check` and task-packet consistency all pass.
19. Existing R3-1 model tests and V2 oracle/production/IR regression suites remain green, including the existing 5,000-case corpus.
20. Final diff contains only authorised paths, the implementation checkpoint is committed before closeout documentation, local HEAD equals remote HEAD and the worktree is clean.

## Required verification

- Use a fresh dedicated worktree tracking `origin/feature/rocket-v3-r3-2-refinement`.
- Read `AGENTS.md`, the OCaml guide, this packet, R3-0 inventory and R3-1 model interface/tests before mutation.
- Confirm branch, exact base `546c778425386dd61ec91422cf01cddb1e40bfbe`, READY state and clean initial worktree.
- Run `pwsh -NoProfile -File scripts/check-dev-tools.ps1`.
- Run `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` and require `control-v1/READY`.
- Verify the exact authorised OCaml switch with explicit `--switch`.
- Implement partition/refinement and focused tests only in authorised paths.
- Run the focused R3-2 test executable throughout implementation.
- Run `opam exec --switch="D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" -- dune build @all` from the current worktree's `tethers-0.1/engine-ocaml`.
- Run the focused R3-2 executable and record exact checks/statistics, including the 1000 homogeneous-Action-chain relation visits.
- Run `opam exec --switch="D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" -- dune runtest --force`.
- Run `git diff --check`.
- Inspect the complete base-to-HEAD diff and prove only authorised paths changed.
- Commit the implementation/test checkpoint and capture its exact full SHA.
- Write the worker note from actual evidence and mark the task `COMPLETE`; no implementation/test mutation after the recorded checkpoint.
- Run the packet checker again and require `control-v1/COMPLETE`.
- Push normally to `origin/feature/rocket-v3-r3-2-refinement`.
- Confirm local HEAD equals remote HEAD and worktree is clean.
- Report exact evidence and STOP.

## Forbidden changes

- No edits to the accepted R3-1 model implementation/interface/tests.
- No Core, validator, lowerer, Enc_V2, V2 oracle/production/IR semantic changes.
- No individualisation of a non-singleton cell.
- No I/R search tree.
- No canonical-label assignment from partition cell order.
- No Enc_V2 candidate generation or ProgramDigest production.
- No prefix pruning, automorphism/orbit pruning or component recursion.
- No undo trail/search-state checkpointing yet.
- No V3 search/resource budgets or wall-clock cutoffs.
- No production adapter/planner/wire/Rust-host integration.
- No new dependency.
- No raw-ID, vertex-handle or storage-order tie-breaker.
- No historical Rocket branch merge/cherry-pick/rebase/copy as implementation authority.
- Do not begin R3-3 automatically after completion.

## Stop conditions

- Correct refinement requires semantic information not exposed by the accepted R3-1 model.
- A stable partition cannot be made invariant to raw IDs/storage/internal numbering without inventing canonical search.
- The incremental algorithm disagrees with the independent slow reference on the same valid model after two materially different diagnoses/repairs.
- The homogeneous Action chain does not become discrete from root/success/complete semantic structure.
- The 1000-Action chain shows unexpectedly explosive/quadratic work that defeats the intended smaller-half architecture.
- Smaller-half scheduling cannot be implemented without changing semantic output.
- Work requires modifying R3-1 model/Core/V2 files, adding dependencies, or beginning search.
- Checkout/branch/base/packet state differs after fetching origin.

## Expected pre-existing changes

None.
