# Rocket V3 — R3-3B3 Exact Rooted Success-Tree Canonisation

Control contract: `1`

Status: `READY`

Task colour: `Red`

Owner: `Codex`

Route: `Fresh dedicated worktree; derive and prove an exact canonicaliser for one connected acyclic Origin success tree rooted at ProgramComplete. No disconnected forest, cross-family or production integration.`

Base commit: `64d1557603366f2b8b934f987bfdef87e2b4ec0e`

OCaml switch path: `D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml`

Worker note: `docs/worker-notes/2026-09-02-rocket-v3-r3-3b3-success-tree-canon.md`

Related issue: `#5 — BUG: Rocket V2 factorial search on simple sequential Action chains`

Updated: 2026-09-02

## Objective

Generalise the proven R3-3B2 single-path result to one connected rooted success tree without returning to factorial Origin permutation search.

Supported success shape:

```
                 ProgramComplete
                 /      |      \
                A       B       C
               / \              |
              D   E             F
```

Edges above are the reverse view of Core `source -> target` success continuations.

For every supported Origin:

- exactly one success continuation exists;
- its target is another Origin or `ProgramComplete`;
- cycles are forbidden;
- following targets from every Origin eventually reaches `ProgramComplete`;
- therefore adding the fixed `ProgramComplete` root makes one connected rooted tree.

The task must compute the exact frozen V2-minimal Origin assignment for this supported tree without enumerating complete Origin permutations.

Do not generalise to disconnected success forests in this task.

## Relevant background and existing behaviour

R3-3A is the exact small-case frozen identity oracle but enumerates legal label assignments.

R3-3B1 repaired the next-observable-byte theorem and proved exact chain-10/11 results, but general label search remained combinatorial.

R3-3B2 then solved the complete single-path case directly in label space.

R3-3B2 exact evidence includes:

- chains 1..11 equal the frozen exhaustive oracle byte-for-byte and digest-for-digest;
- chain-11 exact labels `[10,9,8,7,6,5,4,3,2,1,11]`;
- decimal boundaries 9/10/11/12/99/100/999/1000;
- chain-1000 completes with `complete_permutations_enumerated=0`;
- chain-1000 requires only 1003 candidate target checks and 1003 feasibility checks.

The next structural class is not a general graph. Validator guarantees at most one success continuation per `from_origin` and rejects success cycles. Under this packet's additional connected-completion restriction, reverse success edges form an ordinary rooted tree with fixed root `ProgramComplete`.

Frozen top-level order remains:

`entry_origin -> success_continuations -> origin_sites -> later fields`

Therefore canonicalisation is lexicographically hierarchical:

1. minimise the frozen entry field;
2. among assignments tied there, minimise the complete frozen success-continuation block;
3. among assignments tied through both earlier blocks, minimise the supported Origin-site block;
4. later fixed bytes cannot rescue a loser in any earlier block.

For this task, supported Origin bodies are intentionally restricted so secondary Origin-site comparison is local and exact:

- Anchor Origin with no declared Facts;
- Action Origin with no inputs and no declared Facts;
- Action capability/contract and execution constraints may differ;
- Together Origin is excluded from B3 because its member-label references create another cross-Origin dependency;
- Batch/Facts/Branches/Roles/Templates are excluded.

Thus a supported Origin has a frozen body descriptor independent of raw ID and independent of other Origin labels except its own slot label.

The entry Origin is a distinguished semantic vertex and must remain distinguished during tree canonisation.

## Required behaviour

1. Start from exact base `64d1557603366f2b8b934f987bfdef87e2b4ec0e`.

2. Preserve all R3-3A/B/B1/B2 evidence unchanged.

3. Add a new isolated B3 success-tree module and focused test executable. Do not modify the B2 path canonicaliser.

4. Implement an exact supported-shape predicate for the connected rooted success-tree class described in this packet.

5. Supported shape must require every program Origin to have exactly one success continuation whose transitive target chain ends at `ProgramComplete`.

6. Supported shape must reject any cycle, disconnected/no-continuation component, duplicate source continuation, Batch site, Together Origin, Fact-bearing Anchor/Action, Action input, Branch, Role, ItemTemplate or other cross-family structure.

7. Treat `ProgramComplete` as a fixed external tree root, never as an anonymous labelled Origin.

8. Treat `entry_origin` as a distinguished tree vertex whose frozen entry-field label is selected by exact `encode_int` byte order before the continuation block is optimised.

9. Derive an exact rooted-tree representation from success continuations using only semantic edges and supported frozen body descriptors; raw IDs may be lookup handles only.

10. Define a canonical subtree signature/code for the supported rooted tree. The signature must be invariant under raw-ID and storage permutation and must include the distinguished-entry marker.

11. If subtree/body signatures are used to collapse sibling alternatives, equality must be a proven isomorphism certificate for the complete supported subtree state being collapsed, not merely R3-2 refinement equivalence.

12. Preserve the frozen primary/secondary objective exactly: continuation-block bytes dominate all Origin-site body bytes; body bytes may break ties only among assignments with byte-identical entry + continuation blocks.

13. Build a supported Origin body descriptor from frozen encoding primitives with the Origin's own numeric label treated as a slot-local field. Do not invent a different semantic body order.

14. Produce an exact canonical numeric successor table for the rooted tree, respecting frozen numeric source-slot sorting and exact encoded target bytes.

15. The algorithm may use canonical subtree ordering, dynamic programming, canonical augmentation, exact feasibility, or another proved tree method, but it must not enumerate all complete Origin permutations.

16. Any greedy or locally committed choice must carry an exact proof that no lexicographically smaller legal rooted-tree completion exists.

17. Preserve a compact representation of exact ties when the continuation block has automorphisms; do not choose arbitrarily among tied sibling subtrees if later Origin-site body bytes can distinguish them.

18. Resolve exact continuation ties using the frozen Origin-site block only after the entire earlier continuation block is known equal.

19. Map the final numeric tree labelling back onto semantic Origins and delegate final full payload/digest emission to existing frozen `Tethers_core_canonical_v2_format.encode_program`.

20. Prove that every complete result is a legal bijection over Origin labels 1..N and preserves exactly the original semantic success tree.

21. Prove B2 path compatibility: every B2-supported path fixture must produce the same exact labels/payload/digest under B3.

22. Differentially compare B3 to an independent exhaustive frozen oracle for all generated supported rooted trees up to a tractable size, at minimum every non-isomorphic/generated fixture through N=7 or an equally strong exhaustive labelled corpus.

23. Include explicit small structures: star, balanced binary tree, unbalanced tree, comb/path, repeated identical sibling subtrees, repeated structurally identical siblings with different body descriptors, and asymmetric trees.

24. Include entry-position variants: entry at a leaf, internal node, and direct child of ProgramComplete where semantically valid.

25. Add raw-ID renaming and storage-order permutation metamorphic variants for every structural fixture family.

26. Add decimal-width tree fixtures crossing 9/10, 10/11, 99/100 and 999/1000 without factorial oracle requirements at large sizes.

27. Instrument at minimum: tree_size, tree_height, subtree_signatures_built, exact_tie_classes, symmetry_collapses, candidate_assignments_considered, feasibility_or_dp_states, committed_label_choices, complete_permutations_enumerated, and max_frontier_or_depth.

28. Require `complete_permutations_enumerated = 0` for the new B3 canonicaliser.

29. After exact small-case proof, run at least path-1000, star-1000, balanced-tree approximately 1000 Origins, and a repeated-subtree adversarial tree approximately 1000 Origins. Record deterministic work statistics.

30. Stop after B3 evidence. Do not begin disconnected forest canonisation, Facts/Together/cross-family work, generic I/R, production integration, R3-3C or R3-4.

## Relevant components

Authorised mutation is limited to:

- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-09-02-rocket-v3-r3-3b3-success-tree-canon.md`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_success_tree.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_success_tree.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_success_tree_test.ml`
- `tethers-0.1/engine-ocaml/bin/dune`

Read-only authorities include:

- `tethers_core_canonical_v2_format.ml/.mli`
- `tethers_core_canonical_v2_reference.ml/.mli`
- `tethers_core_rocket_v3_encode.ml/.mli`
- `tethers_core_rocket_v3_origin_walk.ml/.mli`
- `tethers_core_rocket_v3_success_path.ml/.mli`
- validator/Core/planner
- R3-1 model
- R3-2 partition/refinement
- all prior R3-3 worker notes/tests.

If exact implementation requires editing a read-only authority, STOP.

## Frozen decisions and invariants

- Frozen Enc_V2 and ProgramDigest V2 do not change.
- Numeric success-continuation source order does not change.
- `encode_int` byte representation does not change.
- Entry bytes dominate continuation bytes.
- Complete continuation bytes dominate Origin-site bytes.
- Origin-site bytes dominate later fixed fields in this supported projection.
- ProgramComplete is fixed, external and unlabeled.
- Entry Origin is distinguished.
- Supported reverse success structure is one rooted tree.
- Raw IDs/internal vertices/storage order are never label authority.
- Same R3-2 cell is not an automorphism proof.
- Proven complete supported-subtree isomorphism may be used as a symmetry certificate.
- Body descriptors may break only exact continuation ties, never overturn a smaller continuation block.
- No heuristic pruning.
- No complete Origin permutation search.
- No V1 fallback.
- No arbitrary size limit used to define identity.

## Acceptance criteria

1. Work starts from exact base `64d1557603366f2b8b934f987bfdef87e2b4ec0e`.

2. Prior R3 evidence is unchanged.

3. B3 implementation/tests are isolated and B2 implementation is unchanged.

4. Supported-shape predicate accepts connected rooted success trees.

5. Every accepted Origin has exactly one continuation and reaches ProgramComplete.

6. Unsupported/cross-family/Together/disconnected/cyclic shapes reject deterministically.

7. ProgramComplete is never assigned an Origin label.

8. Entry Origin is distinguished and receives the exact frozen minimal entry-field label.

9. Tree representation uses only semantic success edges and supported body semantics.

10. Canonical subtree signatures are raw-ID/storage invariant and entry-aware.

11. Any symmetry collapse is backed by proven complete supported-subtree isomorphism, not refinement-cell equality.

12. Continuation block remains the primary objective and Origin-site body bytes are secondary only on exact continuation ties.

13. Supported body descriptor is frozen-byte-faithful and independent of raw ID.

14. Numeric successor table reproduces frozen numeric source sorting and encoded target-byte comparison exactly.

15. New B3 implementation enumerates no complete Origin permutations.

16. Every local commit/choice has an exact global-minimality justification.

17. Continuation automorphism ties are represented without arbitrary semantic selection.

18. Different bodies break only exact earlier-block ties.

19. Final payload/digest are emitted by existing frozen format machinery.

20. Final labels form one legal Origin bijection preserving the original success tree.

21. All B2 path fixtures produce identical B2/B3 labels, payload and digest.

22. Exhaustive/generated small supported trees match independent frozen oracle exactly.

23. Star/balanced/unbalanced/comb/repeated-sibling/asymmetric fixtures all pass.

24. Leaf/internal/root-child entry variants pass.

25. Raw-ID and storage metamorphic variants remain payload/digest identical.

26. Decimal tree boundary fixtures 9/10, 10/11, 99/100 and 999/1000 pass.

27. Required deterministic B3 statistics are present and repeatable.

28. `complete_permutations_enumerated = 0`.

29. Path/star/balanced/repeated-subtree scale fixtures around N=1000 complete without factorial search or else task stops with exact evidence.

30. Full regressions, authorised-path proof, checkpoint/packet closeout, push, remote/local equality and clean worktree pass; no B4/R3-3C/R3-4 work begins.

## Required verification

Before mutation:

- use a fresh dedicated worktree tracking `origin/feature/rocket-v3-r3-3b3-success-tree-canon`;
- confirm exact base and READY remote HEAD;
- require clean worktree;
- run `pwsh -NoProfile -File scripts/check-dev-tools.ps1`;
- run packet checker and require `control-v1/READY`;
- verify exact authorised OCaml switch;
- read frozen top-level encoder, validator success rules, B2 implementation/tests/note, R3-3A oracle and R3-2 interfaces.

Proof-first stage:

- write down why the accepted reverse-success relation is a rooted tree;
- define the entry-aware canonical subtree state;
- define the exact primary continuation objective and secondary Origin-body tie objective;
- state exactly when a subtree symmetry may be collapsed.

Oracle stage:

- keep the independent oracle implementation separate from B3;
- exhaustively/differentially compare small supported rooted trees;
- include automorphism-heavy and body-tie cases.

Compatibility stage:

- run the complete B2 focused path corpus through both B2 and B3 and require identical labels/payload/digest.

Scale stage only after exact small proof:

- path ~1000;
- star ~1000;
- balanced tree ~1000;
- repeated-subtree adversarial tree ~1000;
- decimal boundaries through 1000.

Record all required deterministic statistics.

Regression:

- B3 focused suite;
- B2 focused suite;
- R3-3A `39/39`;
- R3-1 `214/214`;
- R3-2 `4807/4807`;
- V2 suites;
- 5,000-case corpus;
- `dune build @all`;
- `dune runtest --force`;
- `git diff --check`.

Closeout:

- inspect full base-to-HEAD diff and prove authorised paths only;
- commit implementation/tests and record full implementation checkpoint SHA;
- write worker note;
- transition packet to `COMPLETE`;
- no implementation/test mutation after checkpoint;
- packet checker must report `control-v1/COMPLETE`;
- push normally;
- prove local HEAD == remote HEAD;
- require clean worktree;
- STOP.

## Forbidden changes

- No frozen V2/ProgramDigest/Core/validator changes.
- No R3-1/R3-2 changes.
- No R3-3A/B/B1/B2 implementation changes.
- No disconnected success-forest implementation.
- No Together support.
- No Facts/Branches/Batches/Templates/Roles work.
- No generic graph I/R.
- No refinement-cell-as-automorphism assumption.
- No raw-ID/internal-vertex/storage ordering.
- No complete Origin permutation search.
- No heuristic-only sibling ordering.
- No body-byte influence before an exact continuation tie.
- No new dependency.
- No wall-clock identity decision.
- No production integration.
- No V1 fallback.
- No R3-3C/R3-4/release work.

## Stop conditions

- Accepted shape is not provably a rooted tree.
- B3 disagrees with B2 on any path fixture.
- B3 disagrees with the exhaustive frozen oracle on any supported small tree.
- A proposed symmetry collapse lacks a complete supported-subtree isomorphism proof.
- Body bytes are required to choose between continuation blocks that are not byte-identical.
- Exact rooted-tree canonisation still requires complete factorial Origin permutations.
- Raw IDs/internal handles become necessary for identity.
- Decimal boundaries expose an unmodelled dual-order dependency.
- Scale fixtures become combinatorially explosive and no new exact structural reduction is established.
- Correctness requires changing frozen V2/Core/R3-2.
- Two materially similar failed approaches recur without a new diagnosis.

A Red theorem/performance finding is valid. Do not weaken exactness to complete the task.

## Expected pre-existing changes

None.
