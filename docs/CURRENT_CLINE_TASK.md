# Rocket V3 — R3-3B3B ListIso / Matching Reduction Crucible

Control contract: `1`

Status: `BLOCKED`

Task colour: `Red`

Owner: `Codex`

Route: `Fresh dedicated worktree; research-only rooted-tree theorem crucible. Determine whether exact partial frozen parent-vector completion reduces to tree ListIso or an exact matching/tree-DP generalisation. No production canonicaliser until the reduction is proved.`

Base commit: `eae11c5fd2bb964c0f586c48823f406d2472dccf`

Research evidence base: `eae11c5fd2bb964c0f586c48823f406d2472dccf`

Accepted production frontier: `64d1557603366f2b8b934f987bfdef87e2b4ec0e`

Implementation checkpoint: `0fdef0ec5bcf66b99dbb15f0c9ecfb034887e472`

OCaml switch path: `D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml`

Worker note: `docs/worker-notes/2026-09-02-rocket-v3-r3-3b3b-listiso-reduction.md`

Related issue: `#5 — BUG: Rocket V2 factorial search on simple sequential Action chains`

Updated: 2026-09-02

## Objective

Determine exactly whether the supported connected rooted success-tree completion problem can be solved by:

1. a direct reduction to List Restricted Tree Isomorphism (ListIso), or
2. a closely related exact polynomial matching/tree-DP formulation that preserves the same simultaneous-placement constraints.

The task must not implement a production B3 tree canonicaliser unless and until the completion theorem itself is established. In this packet, even a successful theorem stops at a research-only constructor/oracle and design result.

The central predicate remains:

`Completable(T, e, k, q)`

where:

- `T` is the semantic rooted success tree over Origins;
- ProgramComplete is a fixed external root and is not labelled;
- `e` is the distinguished semantic entry Origin;
- `k` is the fixed numeric label assigned to entry;
- `q` is a supplied prefix of the frozen numeric parent/target vector;
- a completion exists iff there is a bijection from semantic Origins to labels `1..N` that assigns `e -> k` and induces exactly the supplied prefix `q`.

The research question is not whether trees are easy to canonise in the ordinary sense. It is whether these partial numeric placement constraints admit an exact compact completion test.

## Relevant background and existing behaviour

### Accepted B2 frontier

R3-3B2 exactly solves the simple connected success-path crucible in label space.

It matches exhaustive frozen authority on tractable cases and scales to chain 1000 with approximately:

- successor_slots_processed = 1000
- candidate_targets_considered = 1003
- feasibility_checks = 1003
- complete_permutations_enumerated = 0

B2 remains the accepted production frontier.

### B3 theorem already disproved

A later uncommitted B3 experiment attempted to order rooted subtrees by canonical subtree rank.

That theorem is false.

Historical counterexample:

`parents = [6,2,3,6,5,6,-1]`

The rank candidate and exact oracle differ at numeric source slot 7:

- rank candidate target = 6
- exact target = 5

Frozen payload first differs at byte offset 55:

- rank candidate = `0x36`
- exact oracle = `0x35`

Therefore:

> ordinary rooted-subtree canonical rank does not imply frozen global label-placement optimality.

### B3A local-capacity recurrence already disproved

R3-3B3A built an independent exact brute-force completion oracle and tested a compact local-capacity candidate state.

The candidate retained local edge validity, acyclicity, root capacity, child-degree capacity and entry terminal/non-terminal kind.

It is not sufficient.

Minimal false positive:

`parents = [1,2,-1]`

entry semantic vertex = `0`

partial numeric parent vector:

`[2, ProgramComplete]`

The local candidate accepts; exact brute force rejects.

Why: the numeric slot forced to be the entry's parent is simultaneously required to represent a semantic vertex whose own parent kind is incompatible with the prefix. The missing information is coupled semantic placement, not just local degree/component capacity.

B3A exact evidence:

- 47,634 focused checks passed;
- 23,814 partial prefixes checked;
- 1,465,731 complete assignments examined by the bounded brute-force oracle;
- B2 69/69 green;
- R3-3A 39/39 green;
- R3-1 214/214 green;
- R3-2 4807/4807 green;
- full Dune/V2 5000-case corpus green.

### New research lead: ListIso on trees

List Restricted Graph Isomorphism asks whether an isomorphism exists subject to per-vertex allowed-image lists.

Klavík, Knop and Zeman show that ListIso is polynomial-time on trees.

Their tree algorithm processes rooted trees bottom-up. For a candidate mapping `u -> w`, the children of `u` must be simultaneously assignable to the children of `w`; this is tested using bipartite perfect matching over already-feasible child-image pairs.

Relevant references:

- Pavel Klavík, Dušan Knop, Peter Zeman, "Graph Isomorphism Restricted by Lists", Theoretical Computer Science 860 (2021), 51–71.
- DOI: 10.1016/j.tcs.2021.01.027
- arXiv: 1607.03918

This simultaneous matching mechanism is directly relevant to the coupling that B3A's local-capacity recurrence lost.

However, ordinary ListIso receives two complete trees plus unary allowed-image lists.

Our numeric target tree is only partially specified by the supplied parent-vector prefix. Therefore direct applicability is not assumed.

## Supported research shape

For this task only:

- one connected reverse-success rooted tree;
- ProgramComplete is one fixed external root;
- every Origin has exactly one success continuation;
- every Origin reaches ProgramComplete;
- no cycles;
- entry Origin distinguished;
- Origin bodies limited to the same B3A research shape;
- no Facts;
- no Action inputs;
- no Together;
- no Batch;
- no Branches;
- no Roles;
- no ItemTemplates;
- no cross-family labels.

Initial exact theorem work should use N <= 9 so ordinary numeric order and `encode_int` byte order do not introduce an unnecessary decimal-width confounder. Targeted 10/11 boundary work comes only after the structural theorem.

## Required behaviour

1. Start from exact research evidence base `eae11c5fd2bb964c0f586c48823f406d2472dccf`.

2. Treat accepted production B2 `64d1557603366f2b8b934f987bfdef87e2b4ec0e` as the production frontier. This branch is research evidence only and must not be merged as a production B3 implementation.

3. Preserve the B3A exact brute-force completion oracle as read-only correctness authority. Do not weaken it or replace it with the candidate algorithm under test.

4. Re-run and confirm both mandatory negative fixtures before implementing a new theorem:
   - seven-node rank failure `[6,2,3,6,5,6,-1]`;
   - three-node local-capacity false positive `[1,2,-1]`, entry `0`, prefix `[2, Complete]`.

5. Formalise the completion problem as a constraint problem over a bijection `L : semantic Origins -> numeric slots`.

6. Express each processed prefix slot exactly:
   - if `q[i] = Complete`, then the semantic vertex mapped to numeric slot `i` must have ProgramComplete as parent;
   - if `q[i] = j`, then the semantic vertex mapped to numeric slot `i` must have as parent the semantic vertex mapped to numeric slot `j`.

7. Distinguish unary placement restrictions from binary relational restrictions. Do not pretend a binary prefix edge constraint is already a ListIso unary list.

8. Attempt a direct reduction to standard tree ListIso.

9. If proposing a direct reduction, explicitly construct the two complete rooted trees and every allowed-image list.

10. Prove both directions for that reduction:
    - every legal parent-vector completion induces a list-compatible tree isomorphism;
    - every list-compatible tree isomorphism induces a legal parent-vector completion.

11. If a direct reduction is impossible because the target numeric tree is only partially specified, identify the exact obstruction with a minimal counterexample.

12. Then investigate an exact matching/tree-DP generalisation rather than abandoning the matching idea.

13. Candidate states may include possible semantic images for numeric slots, possible numeric slots for semantic vertices, fixed parent constraints, subtree compatibility classes and matching feasibility. Derive the necessary state from correctness; do not adopt this list blindly.

14. Any bottom-up candidate relation `CanMap(u, slot/state)` must preserve simultaneous child-placement constraints. Local degree counts alone are forbidden as sufficiency evidence.

15. Where child assignments are mutually dependent, use an exact bipartite matching condition or prove an equivalent exact criterion.

16. Matching must be genuinely injective: two semantic children may not consume the same numeric child/placement resource.

17. Entry and ProgramComplete must be represented explicitly in the theorem/state. Do not special-case them only in the final constructor.

18. Build a research-only exact candidate feasibility predicate separate from the brute-force oracle.

19. Exhaustively differential-test candidate feasibility against brute-force `Completable` over every reachable prefix for all generated supported trees through at least N=6, plus all required N=7 counterexamples/targeted fixtures.

20. If tractable, extend the exhaustive corpus to N=7; if not, document exact combinatorial limit and use exhaustive N=6 plus strong generated N=7 differential samples.

21. Required generated shapes include:
    - paths;
    - stars;
    - balanced trees;
    - combs;
    - highly asymmetric trees;
    - repeated identical sibling subtrees;
    - repeated isomorphic subtrees under different global placements;
    - entry at leaf;
    - entry at internal vertex;
    - entry adjacent to ProgramComplete.

22. Include raw-ID renaming and storage/construction-order metamorphic variants.

23. The new predicate must reject the three-node false positive for the correct coupled-placement reason, not because of a fixture-specific branch.

24. The new theorem/state must explain conceptually why target 5 remains feasible/preferred while target 6 loses in the seven-node B3 counterexample.

25. If exact feasibility is established, implement a research-only left-to-right parent-vector constructor:
    - process numeric source slots in frozen order;
    - try targets in exact frozen candidate order;
    - commit the first target for which exact candidate feasibility remains true.

26. Such a constructor must enumerate zero complete Origin permutations.

27. Differentially compare final constructed parent vectors against the existing exhaustive frozen oracle for all tractable supported fixtures.

28. Require exact agreement of parent vector, semantic-to-numeric assignment where uniquely determined, frozen payload and digest for tractable fixtures.

29. Only after the N<=9 structural theorem passes, reintroduce exact `encode_int` byte ordering with targeted N=10/11 cases.

30. Prove that decimal-width ordering changes candidate order only, not the structural matching/completion theorem.

31. Instrument at minimum:
    - partial_prefixes_checked;
    - candidate_states;
    - candidate_pairs;
    - matching_instances;
    - matching_vertices;
    - matching_edges;
    - matching_failures;
    - candidate_targets_considered;
    - committed_targets;
    - exact_oracle_complete_assignments;
    - complete_permutations_enumerated.

32. If the exact matching theorem is established and small exhaustive parity passes, one diagnostic scale phase is allowed on N=100 and N=1000 path/star/balanced/repeated-subtree trees.

33. Scale evidence is secondary. Do not use performance to excuse an unproved state transition.

34. Do not introduce a generic graph individualisation/refinement engine.

35. Do not introduce general-purpose SAT/SMT/CSP dependencies.

36. Do not add an external matching dependency. A deterministic research-local augmenting-path or Hopcroft-Karp implementation is sufficient if matching is required.

37. No heuristic sibling ordering may make identity decisions.

38. Stop at the theorem/research result. Do not integrate into production Rocket, forests, cross-family canonicalisation, R3-3C or R3-4.

## Relevant components

Authorised mutation is limited to:

- `docs/CURRENT_CLINE_TASK.md`
- `docs/review/rocket-v3/R3_3B3B_LISTISO_REDUCTION.md`
- `docs/worker-notes/2026-09-02-rocket-v3-r3-3b3b-listiso-reduction.md`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_tree_listiso.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_tree_listiso.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_tree_listiso_test.ml`
- `tethers-0.1/engine-ocaml/bin/dune`

Read-only authorities include:

- `tethers_core_rocket_v3_tree_completion.ml/.mli/_test.ml` from B3A;
- B3A review and worker note;
- frozen V2 format/reference;
- R3-3A exhaustive frozen certificate;
- R3-3B2 path canonicaliser;
- Core/validator/planner;
- R3-1/R3-2;
- all inherited production tests.

Do not modify the B3A tree_completion files. Their exact oracle and counterexample corpus are research authority for this packet.

## Frozen decisions and invariants

- Frozen Enc_V2 and ProgramDigest V2 do not change.
- Entry field dominates success-continuation bytes.
- Success continuations are emitted in numeric source-label order.
- Target labels compare by exact frozen encoded bytes.
- ProgramComplete is fixed and unlabeled.
- Entry Origin is distinguished.
- Raw IDs/internal vertex numbers/storage order are non-semantic.
- Same refinement cell is not automorphism proof.
- Complete subtree isomorphism is not partial-global-state interchangeability.
- Local degree/capacity compatibility is not sufficient completion evidence.
- Unary image lists and binary parent constraints are different mathematical objects until an exact reduction proves otherwise.
- Any matching state must preserve injectivity and simultaneous placement.
- No heuristic rank may prune exact candidates.
- No complete Origin permutation enumeration in a proposed scalable constructor.
- B3A brute force remains bounded correctness authority only.
- Initial N<=9 theorem work intentionally removes decimal-width ordering as a confounder.

## Acceptance criteria

1. Exact research base and branch preflight pass.

2. Both historical counterexamples reproduce unchanged.

3. Exact `Completable` semantics are restated without raw-ID ordering.

4. Prefix constraints are represented as exact relational constraints.

5. Unary and binary constraints are distinguished explicitly.

6. Direct standard-ListIso reducibility is either proved both ways or refuted by a precise obstruction.

7. If direct reduction fails, the packet proceeds to an exact matching/tree-DP formulation rather than silently treating ListIso as solved.

8. Candidate state is documented mathematically before production-like coding.

9. Simultaneous child placement is enforced by perfect matching or a proved equivalent exact condition.

10. Matching/resource use is injective.

11. Entry and ProgramComplete participate in the theorem/state explicitly.

12. Candidate feasibility is implemented separately from brute-force authority.

13. Candidate feasibility agrees with brute-force `Completable` over the required exhaustive N<=6 corpus.

14. Required N=7 historical fixtures pass exactly.

15. Required structural/adversarial generated shapes pass.

16. Raw-ID/storage/construction-order metamorphics are invariant.

17. The three-node B3A false positive is rejected for a general theorem reason.

18. The seven-node target-5/target-6 distinction is explained and reproduced.

19. If a constructor is built, every commitment is justified only by exact candidate feasibility.

20. Any constructor enumerates zero complete Origin permutations.

21. Final constructed parent vectors match exhaustive authority throughout tractable fixtures.

22. Frozen payload/digest parity holds throughout tractable fixtures.

23. N=10/11 boundary tests use exact frozen encoded-byte target order.

24. The structural theorem remains unchanged across decimal-width boundaries.

25. Required deterministic matching/oracle statistics are recorded.

26. No generic graph I/R, SAT/SMT/CSP solver or external matching dependency is introduced.

27. B2, R3-3A, R3-1, R3-2, V2 and full inherited regressions remain green.

28. Production call paths remain untouched.

29. Task finishes COMPLETE only if an exact reduction/state theorem is established and differentially proved; otherwise BLOCKED with the smallest precise obstruction.

30. No forest, cross-family, R3-3C, R3-4 or production B3 work begins.

31. All required deterministic matching/oracle counters are emitted and repeatable.

32. Any N=100/N=1000 scale diagnostics run only after the exact small-case theorem and parity gates pass.

33. Performance evidence never substitutes for exact correctness or differential parity.

34. No generic graph individualisation/refinement engine is introduced.

35. No SAT, SMT or general CSP solver/dependency is introduced.

36. Matching, if required, is implemented research-locally with no external matching dependency.

37. No heuristic sibling ordering or rank is used to make identity decisions.

38. The task stops at the research theorem/result boundary and does not begin production B3, forest, cross-family, R3-3C or R3-4 work.

## Required verification

- Use a fresh dedicated worktree tracking `origin/feature/rocket-v3-r3-3b3b-listiso-reduction`.
- Confirm exact remote READY HEAD, branch, research evidence base and clean worktree.
- Run `pwsh -NoProfile -File scripts/check-dev-tools.ps1`.
- Run packet checker and require `control-v1/READY`.
- Verify exact authorised OCaml switch.
- Read B3A review, worker note and exact tree_completion oracle/test code.
- Read B2 implementation/tests and frozen V2 top-level encoding order.
- Reproduce both mandatory historical counterexamples before writing the new candidate theorem.
- Write `docs/review/rocket-v3/R3_3B3B_LISTISO_REDUCTION.md` first with:
  - formal problem;
  - direct-ListIso reduction attempt;
  - proof or obstruction;
  - matching/tree-DP state;
  - soundness sketch;
  - completeness sketch;
  - complexity.
- Implement independent research module only after the state is explicit.
- Differentially validate candidate feasibility against B3A brute-force `Completable`.
- If candidate feasibility fails on any prefix, reduce to smallest counterexample before attempting repair.
- Two materially similar failed repairs without a genuinely new state/reduction trigger BLOCKED.
- Only after feasibility parity may a lexicographic constructor be implemented.
- Only after small exact parity may N=10/11 byte-order tests and optional N=100/1000 diagnostics run.
- Run B3B focused suite.
- Run B3A focused suite.
- Run B2 `69/69`.
- Run R3-3A `39/39`.
- Run R3-1 `214/214`.
- Run R3-2 `4807/4807`.
- Run V2 suites and generated 5,000-case corpus.
- Run `opam exec --switch="D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml" -- dune build @all`.
- Run `opam exec --switch="D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml" -- dune runtest --force`.
- Run `git diff --check`.
- Inspect full base-to-HEAD diff and prove authorised paths only.
- Commit implementation/research/tests and record full implementation checkpoint.
- Write worker note and transition packet to `COMPLETE` or `BLOCKED`.
- No implementation/test mutation after recorded checkpoint.
- Run packet checker requiring matching terminal state.
- Push normally, prove local HEAD == remote HEAD, require clean worktree, then STOP.

## Forbidden changes

- No frozen V2/ProgramDigest/Core/validator changes.
- No accepted B2 production changes.
- No B3A exact-oracle mutation.
- No heuristic subtree-rank resurrection.
- No local-capacity-as-sufficiency resurrection.
- No raw-ID/internal/storage ordering.
- No generic graph I/R.
- No SAT/SMT/CSP dependency.
- No external graph/matching dependency.
- No disconnected forest solver.
- No Together/Facts/Branches/Batches/Templates/Roles support.
- No production integration.
- No complete-permutation scalable algorithm.
- No V1 fallback.
- No B3 production release claim.
- No R3-3C/R3-4/release work.

## Stop conditions

- A claimed direct ListIso reduction fails either proof direction.
- Candidate feasibility disagrees with brute-force authority on any valid required prefix.
- Matching state collapses only to local degree/capacity information already disproved.
- Exact state requires unrestricted complete labelling enumeration.
- The only way to represent prefix constraints is to enumerate all complete target trees.
- Different raw IDs/storage/traversal change exact feasibility.
- Correctness requires changing frozen V2/Core/B2.
- Two materially similar matching-state attempts fail without a new mathematical abstraction.
- No compact exact theorem can be stated after bounded investigation.

A precise BLOCKED mathematical result is valid and must be pushed as evidence.

## Expected pre-existing changes

The branch intentionally starts from terminal B3A research evidence `eae11c5fd2bb964c0f586c48823f406d2472dccf`.

Those inherited B3A files are expected and remain read-only except for the packet itself.
