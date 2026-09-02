# Rocket V3 — R3-3B3C Complexity Boundary

Control contract: `1`

Status: `READY`

Task colour: `Red`

Owner: `Codex`

Route: `Fresh dedicated worktree; proof-first complexity investigation only. Determine whether exact partial frozen parent-vector completion is equivalent/reducible to rooted spanning-forest completion on a tree, establish the resulting complexity boundary, and identify the correct next algorithmic route. No production B3 implementation.`

Base commit: `ef22f861ebfce6ed6341b5e0043baf53b153aab3`

Accepted production frontier: `64d1557603366f2b8b934f987bfdef87e2b4ec0e`

OCaml switch path: `D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml`

Worker note: `docs/worker-notes/2026-09-02-rocket-v3-r3-3b3c-complexity-boundary.md`

Related issue: `#5 — BUG: Rocket V2 factorial search on simple sequential Action chains`

Updated: 2026-09-02

## Objective

Establish the exact computational-complexity boundary of the B3 parent-vector completion predicate before any further solver design.

The central predicate remains:

`Completable(T, e, k, q)`

where:

- `T` is the semantic rooted success tree over N Origins;
- ProgramComplete is the fixed external root and receives no Origin label;
- `e` is the distinguished semantic entry Origin;
- `k` is its fixed numeric label;
- `q` is a prefix of the numeric parent/target vector;
- a completion exists iff there is a bijection from semantic Origins to labels `1..N`, fixing `e -> k`, whose labelled parent relation agrees with every supplied prefix edge and whose completed tree is isomorphic to `T`.

The task must answer:

1. Is this predicate exactly, or by polynomial reduction, a rooted spanning-forest isomorphism/completion problem on a tree, possibly with one pinned vertex?
2. If so, is general `Completable` NP-complete under the supported Rocket tree shape?
3. If not, what structural restriction of Rocket prefixes prevents the known hardness reduction?
4. What algorithmic strategy follows from the answer: direct-objective canonisation, fixed-parameter completion, or another exact special case?

This task is not another attempt to invent a polynomial `Completable` recurrence.

## Relevant background and existing behaviour

### Accepted frontier

R3-3B2 exactly solves the simple connected success-path case and scales to chain 1000 with zero complete Origin permutation enumeration.

Accepted production frontier:

`64d1557603366f2b8b934f987bfdef87e2b4ec0e`

### B3 negative theorem

Canonical subtree rank is not sufficient for frozen V2 global label order.

Counterexample:

`parents = [6,2,3,6,5,6,-1]`

The rank candidate emits target `6` where the exact frozen oracle prefers target `5`.

### B3A negative theorem

Local edge validity, acyclicity, degree/root capacity and entry terminal-kind are not sufficient for exact completion.

Counterexample:

`parents = [1,2,-1]`

entry = `0`

prefix = `[2, ProgramComplete]`

The local candidate accepts; exact brute force rejects.

### B3B result

R3-3B3B established that exact bottom-up matching works for one connected partial component, including injective child matching, entry pinning and ProgramComplete handling.

It did not establish a global theorem for multiple partial components.

The unresolved state is:

> several individually embeddable partial components must be placed simultaneously and vertex-disjointly into one semantic host tree.

B3B final research evidence:

`ef22f861ebfce6ed6341b5e0043baf53b153aab3`

### Relevant known complexity results

The research lead to examine is the family of problems commonly called:

- Subforest Isomorphism;
- Spanning Forest Isomorphism on Tree;
- Rooted Spanning Forest Isomorphism on Tree;
- Tree Assembly / forest completion by adding edges.

Known literature reports:

- Subforest Isomorphism is NP-complete even though ordinary subtree isomorphism for two trees is polynomial.
- Rooted Spanning Forest Isomorphism on Tree is NP-hard and fixed-parameter tractable in the number `c` of forest components.
- A 2026 result gives an FPT runtime of approximately `O(4^c c^2 n^2 + n^3)` for the rooted spanning-forest isomorphism problem.

Relevant sources to inspect and cite in the research note:

1. Garey and Johnson, Subforest Isomorphism / Computers and Intractability, theorem identifying NP-completeness of forest-into-tree embedding.
2. Theoretical Computer Science 1061 (2026), "Parameterized algorithms for the spanning forest isomorphism and containment on tree", DOI 10.1016/j.tcs.2025.115652.

Do not assume these results apply to Rocket until an exact reduction is established.

## Required behaviour

1. Start from exact B3B research base `ef22f861ebfce6ed6341b5e0043baf53b153aab3`.

2. Treat B2 `64d1557603366f2b8b934f987bfdef87e2b4ec0e` as the accepted production frontier.

3. Preserve B3A brute-force `Completable` and B3B matching code as read-only research authorities.

4. Formalise the partial numeric pattern `F_q` induced by a prefix `q` over all N numeric slots, treating unmentioned/unprocessed slots as isolated vertices and every supplied `q[i]=j` as a directed parent edge.

5. State precisely how `q[i]=ProgramComplete` is represented relative to the fixed external root.

6. Prove or refute that a valid acyclic prefix `q` induces a rooted forest spanning all N numeric slots, with completion corresponding to adding only missing parent edges.

7. Prove or refute the forward equivalence:
   a Rocket completion of `q` induces an isomorphism between a completion of `F_q` and semantic tree `T`.

8. Prove or refute the reverse equivalence:
   a rooted spanning-forest completion of `F_q` to a tree isomorphic to `T`, respecting the entry pin and external root, induces a legal Rocket labelling satisfying `q`.

9. Distinguish exact equivalence from merely similar-looking problems. Record any mismatch explicitly.

10. Investigate the prefix-order restriction: only numeric source slots `1..m` have specified outgoing parent edges in a length-m prefix.

11. Determine whether an arbitrary rooted forest instance can be relabelled in polynomial time so that every vertex with a specified parent edge occupies an initial contiguous label prefix, while component roots/isolated vertices occupy later labels.

12. If such relabelling is possible, prove it. If not, provide the smallest obstruction.

13. Handle the distinguished entry pin `L(e)=k` rigorously. Determine whether pinning leaves hardness intact, can be forced by a polynomial structural gadget, or materially restricts the problem.

14. Handle ProgramComplete rigorously. Do not silently discard the external-root condition.

15. If the exact Rocket predicate is shown equivalent/reducible to a known NP-hard rooted spanning-forest problem, prove membership in NP and state the strongest justified complexity result for Rocket `Completable`.

16. Do not claim NP-hardness or NP-completeness solely from analogy or citation; the Rocket-specific reduction must be explicit.

17. Build a small research-only reduction/witness checker if useful, separate from B3A/B3B authorities.

18. Mechanically validate the proposed equivalence/reduction on bounded generated forests and Rocket prefixes through at least N=7 against B3A brute-force truth.

19. Include paths, stars, balanced trees, multiple nontrivial components, isolated slots, entry inside/outside a nontrivial component, and prefixes with/without a ProgramComplete edge.

20. Include raw-ID and construction/storage-order metamorphic variants.

21. If hardness is established, stop searching for a general polynomial `Completable` recurrence in this task.

22. If hardness is not established because Rocket prefixes have extra exploitable structure, identify that structure exactly and propose the narrowest theorem that remains plausible.

23. Map the known FPT component parameter `c` to Rocket prefix state. Derive `c` exactly from `F_q` rather than guessing.

24. Determine how `c` evolves as a valid prefix grows and whether early/late prefix regimes differ materially.

25. Assess whether the known FPT direction could be useful as an exact fallback or late-prefix solver without proposing production integration yet.

26. Separately analyse whether NP-hardness of arbitrary prefix completion implies hardness of computing the final frozen V2 canonical payload. Do not assume it does.

27. State explicitly whether a direct canonical-labelling algorithm could still avoid arbitrary `Completable` queries even if the prefix predicate is NP-hard.

28. Produce a next-step recommendation choosing exactly one of:
   - DIRECT OBJECTIVE RESEARCH;
   - FPT COMPLETION RESEARCH;
   - EXPLOIT ROCKET-SPECIFIC PREFIX STRUCTURE;
   - STOP GENERAL B3 TREE WORK.

29. Preserve all inherited regressions and production call paths unchanged.

30. Stop after the complexity theorem and recommendation. Do not begin the recommended next task.

## Relevant components

Authorised mutation is limited to:

- `docs/CURRENT_CLINE_TASK.md`
- `docs/review/rocket-v3/R3_3B3C_COMPLEXITY_BOUNDARY.md`
- `docs/worker-notes/2026-09-02-rocket-v3-r3-3b3c-complexity-boundary.md`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_tree_complexity.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_tree_complexity.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_tree_complexity_test.ml`
- `tethers-0.1/engine-ocaml/bin/dune`

Read-only authorities include:

- B3A `tree_completion` implementation/tests/review/note;
- B3B `tree_listiso` implementation/tests/review/note;
- R3-3B2 success-path implementation/tests;
- R3-3A exhaustive frozen oracle;
- frozen V2 format/reference;
- Core/validator/planner;
- R3-1/R3-2.

The new complexity module is research-only and must not enter a production call path.

## Frozen decisions and invariants

- Frozen Enc_V2 and ProgramDigest V2 do not change.
- ProgramComplete is a fixed external root and receives no Origin label.
- Entry Origin is distinguished and has a fixed numeric label.
- Numeric source slots in a prefix are exactly the initial contiguous slots `1..m`.
- Raw IDs, storage order and traversal order are non-semantic.
- B3A brute force remains bounded truth authority.
- B3B connected matching remains exact only within its stated connected-component boundary.
- NP-hardness of a general completion oracle does not automatically prove hardness of direct canonical labelling.
- A literature theorem applies to Rocket only after an explicit polynomial reduction/equivalence is proved.
- No heuristic identity decision is permitted.
- No change to V1/V2 identity law is permitted.

## Acceptance criteria

1. Exact B3B research base, branch and startup preflight pass.

2. B2 remains explicitly identified as the production frontier.

3. B3A and B3B research authorities remain unmodified.

4. `F_q` is formally defined over all N numeric slots.

5. ProgramComplete representation in `F_q` is explicit.

6. The spanning-forest interpretation of valid prefixes is proved or precisely refuted.

7. Forward Rocket-completion to forest-completion implication is proved or precisely refuted.

8. Reverse forest-completion to Rocket-completion implication is proved or precisely refuted.

9. Any mismatch with standard rooted spanning-forest formulations is documented exactly.

10. The contiguous prefix-source restriction is analysed explicitly.

11. Arbitrary forest relabelling into the required prefix form is proved polynomial or refuted by a counterexample.

12. The relabelling argument, if used, is mechanically/boundedly validated.

13. Entry pinning is handled rigorously in the reduction/obstruction.

14. ProgramComplete/external-root semantics are handled rigorously in the reduction/obstruction.

15. Any NP-hard/NP-complete claim includes a Rocket-specific proof and membership-in-NP argument where applicable.

16. No complexity claim rests only on analogy or citation.

17. Any research reduction checker is isolated from production and existing authorities.

18. Bounded generated equivalence checks agree with B3A truth through the required corpus.

19. Required structural/prefix fixture families pass.

20. Raw-ID/storage/construction metamorphics remain invariant.

21. If hardness is established, the packet does not continue searching for a general polynomial `Completable` recurrence.

22. If hardness is not established, the exact Rocket-specific structural escape hatch is identified.

23. Forest-component parameter `c` is derived exactly from Rocket prefix state.

24. Evolution of `c` across prefix growth is analysed.

25. FPT applicability is assessed without production integration.

26. The distinction between prefix-completion hardness and direct canonical-payload complexity is explicit.

27. The possibility of a direct objective algorithm is assessed separately from `Completable`.

28. Exactly one next-step recommendation is selected and justified.

29. All inherited tests/regressions remain green and production call paths untouched.

30. Task stops at the complexity/recommendation boundary with no next-stage implementation.

## Required verification

- Use a fresh dedicated worktree tracking `origin/feature/rocket-v3-r3-3b3c-complexity-boundary`.
- Confirm branch, exact remote READY HEAD, Base commit and clean worktree.
- Run `pwsh -NoProfile -File scripts/check-dev-tools.ps1`.
- Run packet checker and require `control-v1/READY`.
- Verify exact authorised OCaml switch.
- Read this packet, B3A review/note/oracle, B3B review/note/matching code, B2 and frozen V2 authority before mutation.
- Write `docs/review/rocket-v3/R3_3B3C_COMPLEXITY_BOUNDARY.md` with:
  - exact Rocket predicate;
  - induced forest construction;
  - known problem definition;
  - reduction/equivalence proof or obstruction;
  - entry/root treatment;
  - prefix-contiguity treatment;
  - complexity conclusion;
  - FPT parameter mapping;
  - direct-objective distinction;
  - one next-step recommendation.
- If a bounded research checker is useful, implement it only in the authorised `tree_complexity` files.
- Differentially validate the reduction/equivalence on generated small cases against B3A brute force.
- Run B3C focused suite.
- Run B3B focused suite.
- Run B3A `47634/47634`.
- Run B2 `69/69`.
- Run R3-3A `39/39`.
- Run R3-1 `214/214`.
- Run R3-2 `4807/4807`.
- Run V2 suites and generated 5,000-case corpus.
- Run `opam exec --switch="D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml" -- dune build @all`.
- Run `opam exec --switch="D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml" -- dune runtest --force`.
- Run `git diff --check`.
- Inspect full base-to-HEAD diff and prove authorised paths only.
- Commit research/tests and record full implementation checkpoint.
- Write worker note and transition packet to `COMPLETE` or `BLOCKED` based on evidence.
- No research implementation/test mutation after the recorded checkpoint.
- Run packet checker requiring matching terminal state.
- Push normally, prove local HEAD == remote HEAD, require clean worktree, then STOP.

## Forbidden changes

- No frozen V2/ProgramDigest/Core/validator changes.
- No B2 production changes.
- No B3A oracle changes.
- No B3B matching changes.
- No heuristic subtree ranking.
- No new general polynomial `Completable` implementation attempt.
- No generic graph I/R.
- No SAT/SMT/general CSP dependency.
- No external graph/matching dependency.
- No production B3 integration.
- No forest production solver.
- No cross-family support.
- No Together/Facts/Branches/Batches/Templates/Roles support.
- No V1 fallback.
- No R3-3C/R3-4/release work.

## Stop conditions

- The proposed forest equivalence fails on a bounded exact counterexample.
- Arbitrary rooted forest instances cannot be encoded because of a genuine Rocket prefix restriction.
- Entry pinning or ProgramComplete semantics invalidate the claimed hardness reduction.
- A complexity claim cannot be made without assumptions stronger than the supported Rocket shape.
- The task begins drifting into solver implementation rather than complexity proof.
- Any required correctness change would modify frozen V2/Core/B2.
- Bounded reduction/equivalence checks disagree with B3A truth.
- No defensible complexity theorem can be stated from the gathered evidence.

A BLOCKED result is valid if it precisely identifies which reduction step fails.

## Expected pre-existing changes

None.
