# Rocket V3 — R3-3B3A Exact Parent-Vector Completion Theorem

Control contract: `1`

Status: `BLOCKED`

Task colour: `Red`

Owner: `Codex`

Route: `Fresh dedicated worktree; proof-first Origin-only rooted-tree research crucible. Establish an exact partial parent-vector completion theorem before any scalable tree canonicaliser is attempted.`

Base commit: `64d1557603366f2b8b934f987bfdef87e2b4ec0e`

Implementation checkpoint: `2234438f8fa03811bc33788d26744023bef6495e`

OCaml switch path: `D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml`

Worker note: `docs/worker-notes/2026-09-02-rocket-v3-r3-3b3a-parent-vector-completion.md`

Related issue: `#5 — BUG: Rocket V2 factorial search on simple sequential Action chains`

Updated: 2026-09-02

## Objective

Determine whether exact frozen Enc_V2 canonicalisation of a connected rooted success tree can be reduced to lexicographic construction of its numeric parent/target vector using an exact partial-completion feasibility predicate.

Do not build the general B3 tree canonicaliser yet.

The central question is:

> Given the original semantic rooted success tree, a fixed exact entry label, and a prefix of numeric source slots whose frozen continuation targets have been chosen, can we decide exactly whether that prefix extends to at least one complete bijective Origin labelling isomorphic to the original tree?

If the answer admits a compact exact tree-specific algorithm, prove it and implement only the bounded research oracle/feasibility machinery required to establish the theorem.

If no such exact reduction is established, STOP with a Red/BLOCKED mathematical finding.

## Relevant background and existing behaviour

R3-3A is the exact frozen small-case oracle by complete legal label enumeration.

R3-3B1 repaired the next-observable-byte law.

R3-3B2 solved the complete simple success path directly in label space. Chain-1000 completes with zero complete Origin permutation enumeration.

A subsequent uncommitted B3 experiment attempted to generalise path canonisation to rooted success trees by assigning canonical subtree ranks. That experiment correctly stopped before commit/push after an independent exact oracle disproved the ordering rule.

Mandatory counterexample from that experiment:

`parents = [6,2,3,6,5,6,-1]`

For the generated supported tree, canonical subtree-rank ordering selected continuation target `6` where the independent exact frozen oracle selected `5`.

The B3 attempt had 1793/1793 focused checks before this counterexample, plus green build/regressions, demonstrating that the failure is not basic tree construction. The failed theorem was:

> canonical subtree rank order implies frozen Enc_V2-optimal global label order.

That theorem is false.

This task must not repair the failure by adding more heuristic ranking fields.

### Important simplification

For trees with fewer than 10 Origin labels, numeric label order and `encode_int` byte order coincide.

Therefore the first proof domain should deliberately stay below the decimal-width boundary. This isolates the real tree-label placement problem from the separate decimal encoding issue.

Within that domain, the frozen success-continuation objective reduces to the lexicographically smallest legal numeric target/parent vector after the entry label is fixed.

### Supported research shape

For this task only:

- program Origin sites only;
- Anchor or Action Origins;
- no Facts;
- no Action inputs;
- no Together;
- no Batch;
- no Branches;
- no Roles;
- no ItemTemplates;
- each Origin has exactly one success continuation;
- every Origin reaches ProgramComplete;
- no cycle;
- one connected reverse-success rooted tree;
- entry_origin is distinguished.

ProgramComplete is a fixed external root and never receives an Origin label.

## Required behaviour

1. Start from exact accepted B2 base `64d1557603366f2b8b934f987bfdef87e2b4ec0e`; do not inherit the uncommitted B3 implementation as authority.

2. Reconstruct the reported B3 failing tree from `parents = [6,2,3,6,5,6,-1]` or an exactly equivalent explicit semantic fixture, and independently reproduce the rank-order target `6` versus exact-oracle target `5` mismatch before proposing a replacement theorem.

3. Record the first frozen continuation position/bytes at which the heuristic candidate loses to the exact candidate.

4. Formalise the supported rooted-tree canonicalisation objective as lexicographic minimisation of the complete numeric success target/parent vector after the frozen entry label is fixed.

5. For the primary proof corpus use N <= 9 so numeric label order equals exact encoded integer order; do not mix decimal-width effects into the initial theorem.

6. Define a partial parent-vector state over processed numeric source slots, including enough semantic information to state what remains unassigned without using raw IDs as canonical evidence.

7. Define the exact extension predicate:
   `Completable(original_tree, fixed_entry, partial_parent_vector)`.

8. The predicate must mean existence of at least one complete bijective Origin labelling whose induced rooted success tree is isomorphic to the original semantic tree, respects the distinguished entry vertex, and has exactly the supplied parent-vector prefix.

9. Build an independent brute-force completion oracle for small N. It must not call the proposed feasibility algorithm.

10. Exhaustively compare the proposed feasibility predicate against brute-force existence for every reachable partial prefix of every generated supported tree through at least N=7, or another demonstrably equivalent exhaustive corpus.

11. Include the known failing B3 tree and its partial prefixes in the exhaustive feasibility corpus.

12. Determine what information the failed canonical-subtree-rank rule discarded. Record this explicitly in the worker note/design evidence.

13. Investigate exact tree-specific formulations such as entry-aware coloured rooted-tree isomorphism, canonical component matching, dynamic programming over child-isomorphism classes, or another exact construction. Do not assume any candidate formulation is sufficient until differential proof passes.

14. If subtree isomorphism classes are used, distinguish clearly between:
    - proof that two complete supported subtrees are isomorphic;
    - proof that two partial label-allocation states are interchangeable under the frozen parent-vector objective.
    The former alone must not imply the latter.

15. Determine whether the completion predicate has a compact polynomial/bounded state representation. State the state variables and recurrence/decision rule explicitly.

16. If a valid exact completion predicate is established, implement a research-only lexicographic parent-vector constructor that processes source slots left-to-right and selects the smallest target whose prefix remains completable.

17. The constructor must use exact feasibility, not heuristic subtree ranking, and must enumerate zero complete Origin permutations.

18. Differentially prove the constructor against the full frozen exhaustive oracle for all supported generated trees through at least N=7 and targeted fixtures through N=8/9 where tractable.

19. Require exact equality of parent vector, final Origin assignment, frozen payload and digest for all tractable fixtures.

20. Include stars, balanced trees, unbalanced trees, combs, repeated identical sibling subtrees, repeated structurally identical siblings with different supported body descriptors, asymmetric trees, and entry at leaf/internal/root-child positions where valid.

21. Add raw-ID renaming and storage-order permutation metamorphic variants.

22. Only after the N<=9 theorem passes, reintroduce exact `encode_int` byte ordering at N=10/11 in targeted bounded cases and prove that the feasibility theorem itself is unchanged while candidate ordering uses frozen bytes.

23. Instrument at minimum: partial_prefixes_checked, brute_force_completions_considered, feasibility_states, isomorphism_checks, exact_tie_states, candidate_targets_considered, committed_targets, complete_permutations_enumerated, and max_state_width.

24. Do not attempt N=100/1000 scaling unless a compact exact completion theorem has first been established and validated. If established, one modest scale probe up to N=100 is permitted only as diagnostic evidence, not as acceptance authority.

25. Preserve B2 and all prior accepted regressions. No production path may reference B3A.

26. Stop after the theorem/research result. Do not begin scalable B3B implementation, disconnected forests, cross-family canonicalisation, R3-3C or R3-4.

## Relevant components

Authorised mutation is limited to:

- `docs/CURRENT_CLINE_TASK.md`
- `docs/review/rocket-v3/R3_3B3A_PARENT_VECTOR_COMPLETION.md`
- `docs/worker-notes/2026-09-02-rocket-v3-r3-3b3a-parent-vector-completion.md`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_tree_completion.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_tree_completion.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_tree_completion_test.ml`
- `tethers-0.1/engine-ocaml/bin/dune`

Read-only authorities include:

- frozen V2 format/reference;
- R3-3A exact certificate;
- R3-3B1 Origin walker;
- R3-3B2 success-path canonicaliser;
- Core/validator/planner;
- R3-1/R3-2;
- all prior committed worker notes/tests.

Do not modify or import uncommitted B3 experimental files into the accepted codebase. They may be inspected locally as historical research only if available.

## Frozen decisions and invariants

- Frozen Enc_V2 and ProgramDigest V2 do not change.
- Entry field is minimised before success-continuation bytes.
- Success continuations are emitted in numeric source-label order.
- Target labels are compared by exact frozen encoded bytes.
- ProgramComplete is fixed and unlabeled.
- Entry Origin is distinguished.
- Raw IDs/internal vertex numbers/storage order are non-semantic.
- Same R3-2 cell is not automorphism proof.
- Complete rooted-subtree isomorphism does not automatically prove interchangeability under a partial global label allocation.
- No heuristic subtree ranking may prune or commit identity decisions.
- Exact feasibility means existence of a full legal completion, not local plausibility.
- The initial N<=9 proof intentionally removes decimal-width ordering as a confounder.
- No complete Origin permutation enumeration in any proposed scalable constructor.
- No V1 fallback or identity budget.

## Acceptance criteria

1. Work starts from exact base `64d1557603366f2b8b934f987bfdef87e2b4ec0e`.

2. The known B3 counterexample is reconstructed independently.

3. The target-6 versus target-5 oracle mismatch is reproduced and its first frozen differing continuation position is recorded.

4. The exact parent-vector objective is formally documented.

5. Initial proof fixtures remain within N<=9 except explicitly separated decimal-boundary tests.

6. Partial state has no raw-ID-derived canonical ordering.

7. `Completable` is defined precisely.

8. `Completable` includes exact semantic-tree isomorphism, distinguished entry and exact prefix requirements.

9. Independent brute-force existence oracle exists.

10. Feasibility agrees with brute-force existence over the exhaustive required partial-prefix corpus.

11. The historical failing tree is included in that corpus.

12. The information missing from canonical subtree rank is explicitly identified.

13. Any chosen tree-specific formulation is justified by tests/proof, not assumption.

14. Complete subtree isomorphism and partial-state interchangeability are treated as separate claims.

15. A compact exact state/recurrence is either established explicitly or the task stops BLOCKED with evidence.

16. If established, an exact lexicographic constructor exists.

17. Constructor decisions use only exact completion feasibility and enumerate zero complete permutations.

18. Constructor matches exhaustive exact parent vectors throughout the supported small-tree corpus.

19. Final labels/payload/digest match frozen exhaustive authority.

20. Required structural/adversarial/entry fixtures pass.

21. Raw-ID/storage metamorphic variants are invariant.

22. Targeted N=10/11 tests correctly reintroduce exact encoded-byte candidate ordering without changing the feasibility theorem.

23. Required deterministic statistics are present and repeatable.

24. No large-scale claim is made before theorem proof; any permitted N<=100 probe is clearly diagnostic only.

25. B2 and full inherited regressions remain green and no production path references B3A.

26. Task ends COMPLETE only if the theorem is established; otherwise ends BLOCKED with the smallest precise unresolved mathematical obstruction. No B3B/R3-3C/R3-4 work begins.

## Required verification

- Use a fresh dedicated worktree tracking `origin/feature/rocket-v3-r3-3b3a-parent-vector-completion`.
- Confirm exact branch, remote READY HEAD, base and clean worktree.
- Run `pwsh -NoProfile -File scripts/check-dev-tools.ps1`.
- Run packet checker and require `control-v1/READY`.
- Verify exact authorised OCaml switch.
- Read frozen top-level encoder, B2 implementation/tests/note, R3-3A oracle and validator success rules.
- Reproduce the B3 counterexample before implementing the replacement feasibility theorem.
- Write/update `docs/review/rocket-v3/R3_3B3A_PARENT_VECTOR_COMPLETION.md` with the objective, counterexample, state definition, recurrence/decision theorem and proof limits.
- Build independent brute-force small-tree/prefix oracle first.
- Differentially validate feasibility over exhaustive partial prefixes.
- Only if feasibility passes, build the left-to-right lexicographic constructor.
- Differentially validate final constructor against frozen exhaustive payload/digest authority.
- Run targeted N=10/11 exact encoded-byte tests only after N<=9 proof passes.
- Run B3A focused suite.
- Run B2 focused suite.
- Run R3-3A `39/39`.
- Run R3-1 `214/214`.
- Run R3-2 `4807/4807`.
- Run V2 suites and 5,000-case corpus.
- Run `opam exec --switch="D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml" -- dune build @all`.
- Run `opam exec --switch="D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml" -- dune runtest --force`.
- Run `git diff --check`.
- Inspect full base-to-HEAD diff and prove authorised paths only.
- Commit implementation/research/tests and record full implementation checkpoint.
- Write worker note and transition packet to `COMPLETE` or `BLOCKED` according to evidence.
- No implementation/test mutation after recorded checkpoint.
- Run packet checker requiring matching terminal state.
- Push normally for COMPLETE or BLOCKED evidence, prove local HEAD == remote HEAD, require clean worktree, then STOP.

## Forbidden changes

- No frozen V2/ProgramDigest/Core/validator changes.
- No R3-1/R3-2/R3-3A/B1/B2 changes.
- No import of uncommitted B3 heuristic implementation as accepted code.
- No heuristic subtree-rank ordering.
- No refinement-cell-as-automorphism assumption.
- No subtree-isomorphism-implies-global-interchangeability shortcut.
- No raw-ID/internal/storage ordering.
- No generic graph I/R.
- No disconnected forest solver.
- No Together/Facts/Branches/Batches/Templates/Roles support.
- No complete-permutation production algorithm.
- No new dependency.
- No wall-clock identity decision.
- No production integration.
- No V1 fallback.
- No B3B/R3-3C/R3-4/release work.

## Stop conditions

- The B3 counterexample cannot be reproduced from the reported semantic fixture.
- Proposed feasibility disagrees with brute-force existence on any valid exhaustive partial prefix.
- Required exact state grows into unrestricted complete permutation enumeration.
- No compact exact completion theorem can be stated after bounded investigation.
- Any proposed collapse relies only on subtree rank/refinement rather than exact partial-state equivalence.
- Different traversal/raw-ID/storage order changes an exact result.
- N=10/11 reveals that the feasibility theorem itself depended incorrectly on numeric rather than structural state.
- Correctness requires changing frozen V2/Core/R3-2.
- Two materially similar theorem attempts fail without a genuinely new state/reduction.

A BLOCKED mathematical result is valid and should be pushed as evidence.

## Expected pre-existing changes

None.
