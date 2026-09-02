# Rocket V3 R3-3B3C Complexity Boundary

## Result

This investigation establishes an exact reduction of the Rocket partial
parent-vector predicate to a restricted rooted spanning-forest extension
problem.  It does not establish NP-hardness or NP-completeness for Rocket
`Completable`: the available cited hardness results do not preserve Rocket's
fixed entry mapping and external-terminal contract, and no pin-preserving
gadget has been proved here.

The strongest defensible result is:

* Rocket `Completable` is in NP.
* A forest-compatible Rocket prefix is exactly a spanning subforest of the
  relabelled semantic success tree, with fixed parent directions, one fixed
  external `ProgramComplete` root, a contiguous source prefix, and a pinned
  entry image.
* The contiguous-prefix condition is not itself an obstruction: every rooted
  forest can be normalized by placing its non-root vertices in slots `1..m`
  and its component roots afterwards.
* The known rooted spanning-forest FPT parameter maps exactly to the number of
  components in the Rocket prefix including `ProgramComplete`, namely
  `c = N + 1 - m` for an acyclic length-`m` prefix.
* A direct frozen-payload canonicaliser is a separate optimization problem;
  hardness of a general prefix decision oracle would not prove hardness of
  computing the final minimum payload.

## Exact Rocket predicate

Let `T` be the semantic success tree over `N` Origins.  Each Origin has one
success parent, either another Origin or the fixed external terminal
`ProgramComplete`.  Let `e` be the semantic entry Origin, let `k` be its fixed
numeric label, and let `q = (q[1], ..., q[m])` be a Rocket prefix.

The exact decision predicate is:

```text
Completable(T, e, k, q)
```

iff there is a bijection `L` from the semantic Origins to labels `1..N` such
that `L(e) = k` and, for every processed source slot `i`:

```text
q[i] = j         => parent_T(L^-1(i)) = L^-1(j)
q[i] = Complete  => parent_T(L^-1(i)) = ProgramComplete.
```

The labels are numeric source positions.  This is a placement constraint on
the images of both endpoints of an edge, not an independent unary label list.

## The induced partial structure `F_q`

Define a graph with vertex set:

```text
V(F_q) = {1, 2, ..., N} union {C}
```

where `C` is a single fixed external vertex representing
`ProgramComplete`.  For each processed source `i <= m`, add exactly one
directed edge:

```text
i -> j  when q[i] = j
i -> C  when q[i] = ProgramComplete.
```

Slots greater than `m` have no supplied outgoing edge.  They are isolated only
with respect to their own outgoing edge; they may still be targets of already
supplied edges.

The distinction between syntactic and structural validity matters:

* Every prefix has outdegree at most one at every numeric slot because sources
  are processed once.
* A prefix containing a directed/undirected cycle cannot be completed to the
  Rocket tree.
* A prefix with more than one `i -> C` edge cannot be completed because the
  semantic tree has exactly one Origin whose parent is `ProgramComplete`.
* An acyclic prefix with at most one external edge is a spanning forest
  candidate, but it is not automatically `Completable`.  The B3A false
  positive `parents = [1,2,-1]`, entry `0`, prefix
  `[2, ProgramComplete]` is the smallest repository witness: it is locally
  forest-like but has no semantic embedding satisfying both fixed edges.

Thus the precise statement is:

> A Rocket prefix that is acyclic and extendible induces a rooted directed
> spanning forest over all numeric slots plus `C`.  Completion adds only the
> missing outgoing parent edges of the unprocessed slots, and the result must
> be the semantic host tree with `C` fixed.

An arbitrary syntactically valid prefix need not satisfy the first sentence;
the forest interpretation is a necessary structural condition, not a shortcut
for semantic feasibility.

## Forward implication: Rocket to forest completion

Assume `Completable(T,e,k,q)` and let `L` be a witnessing bijection.  Extend
the inverse map with `phi(C) = ProgramComplete` and define:

```text
phi(i) = L^-1(i)  for every numeric slot i.
```

The completed labelled Rocket graph is the semantic tree `T` with its Origin
vertices renamed by `L`.  Every edge supplied by `q` is an edge of that
completed tree by the definition of `Completable`; `i -> C` maps to the unique
semantic terminal edge.  Therefore `F_q` is a spanning subforest of a tree
isomorphic to `T` with its fixed external root, and `phi(k) = e`.

Because all slots are already present in `F_q`, no vertex is inserted during
completion.  Only missing parent edges are added.  A processed source cannot
receive another outgoing parent edge; an unprocessed source receives its one
remaining parent edge.

## Reverse implication: forest completion to Rocket

Assume a rooted directed completion `H` of `F_q` exists on the same `N+1`
vertices, `H` is isomorphic to `T` with `C` fixed, and the isomorphism maps
slot `k` to semantic entry `e`.  Define `L` by the inverse of that
isomorphism on Origins.

Every supplied `i -> j` edge remains a parent edge in `H`, so its semantic
image satisfies `parent_T(L^-1(i)) = L^-1(j)`.  Every supplied `i -> C` edge
has semantic image `ProgramComplete`.  The fixed image of `k` gives
`L(e) = k`.  Hence `L` is a legal Rocket labelling and witnesses
`Completable(T,e,k,q)`.

The two directions are therefore equivalent for the following restricted
problem, which is the exact Rocket abstraction:

```text
Pinned externally-rooted directed spanning-forest extension with
contiguous processed sources and frozen outgoing-parent edges.
```

This is stronger than a visual analogy, but deliberately narrower than an
unqualified standard forest-completion problem.  A standard undirected
forest instance does not, by itself, carry the Rocket requirement that each
already processed source has exactly its one frozen outgoing parent edge.

## Contiguous-prefix normalization

The prefix-source restriction does not prevent encoding an arbitrary rooted
forest's specified parent edges.

Take a rooted forest `F` on Origin vertices.  Every non-root vertex has one
specified parent edge; every component root has none.  Let `m` be the number of
non-root vertices.  Assign labels `1..m` to the non-roots and assign later
labels to the component roots.  For each non-root vertex `u` in slot `i`, set:

```text
q[i] = slot(parent_F(u)).
```

The parent may be a later root slot or another prefix slot.  Every specified
parent-edge source is now in the contiguous prefix, and every component root
is later.  This construction is linear in the forest size and uses no
semantic canonical ordering; it is only a reduction encoding.

The normalization is exact for rooted forest edges because a rooted forest has
one outgoing parent edge at precisely its non-roots.  It would not be exact
for an arbitrary undirected partial graph whose specified edges do not admit
one parent orientation per non-root.  That distinction is why the reduction
must name the rooted directed forest variant explicitly.

The B3C research checker mechanically constructs this normalization and checks
the resulting prefix shape.  Its deterministic construction order is not
used as Rocket identity evidence.

## Entry pin

Rocket requires `L(e) = k`.  Under the exact forest formulation this is a
prescribed image pair `(e,k)`, or equivalently a pinned vertex/slot in the
forest isomorphism.

If a source forest instance already supplies a distinguished vertex and
prescribed image, the normalization above preserves it: place that vertex in
the requested slot, subject to whether it is a non-root or a component root.
If the vertex is a non-root, choose its requested slot inside `1..m`; if it is
a root, choose its requested slot after `m`.  The other vertices can be
placed around it without changing any forest edge.

This does not prove that the unpinned NP-hard problem remains NP-hard after an
arbitrary prescribed image is imposed.  A reduction from an unpinned source
would need either a pin-preserving hardness theorem or a proved polynomial
gadget.  Neither is present in the repository or established here.  It would
be an overclaim to cite unpinned hardness and silently treat the Rocket entry
pin as free.

## ProgramComplete and the external root

`C` is not one of the `N` anonymous Origin identities and receives no numeric
label.  It is a fixed vertex in `F_q` and in the semantic host.

For comparison with a rooted target tree whose ordinary root is `r`, add a
fresh external vertex `C` and the single edge `r -> C` in the Rocket parent
orientation.  Add `C` as an isolated fixed vertex to the forest side.  A
completion then adds exactly one edge to `C`, and the unique Origin incident to
that edge must map to `r`.

This transformation preserves arbitrary branching below `r`; it only makes
the Rocket terminal explicit.  It does not permit several independent
components to terminate at `C`.  A forest with several components must join
all but one component through Origin-to-Origin parent edges before the one
remaining root takes the external edge.  A prefix that already has two
external edges is therefore a forest structurally but not a Rocket-completable
forest.

The external-root treatment is exact for rooted, externally anchored target
instances.  It is not permission to replace the Rocket terminal by an
ordinary high-degree root in a hardness proof.

## Complexity conclusion

### Membership in NP

Rocket `Completable` is in NP when `T`, `e`, `k` and `q` are input objects.
The certificate is the `N`-entry bijection `L` (or the slot-to-Origin
permutation).  In polynomial time, a verifier checks:

1. `L` is a bijection;
2. `L(e)=k`;
3. every supplied `q[i]=j` agrees with the corresponding semantic parent;
4. every supplied `q[i]=Complete` agrees with the unique external parent;
5. the semantic input and fixed-root conditions are well formed.

This is a decision-membership result only.

### No unsupported NP-hardness claim

Garey and Johnson's `SUBFOREST ISOMORPHISM` theorem establishes NP-completeness
for the general problem in which a forest must occur as a subforest of a tree.
The 2026 Liu, Chen, Zheng, Wang and Shi paper states that rooted spanning
forest isomorphism on a tree is NP-hard and fixed-parameter tractable in the
number of forest components, with an algorithm of approximately
`O(4^c c^2 n^2 + n^3)`.

Those results support the relevance of the forest boundary.  They do not,
without an explicit reduction, establish NP-hardness for Rocket.  The exact
Rocket-to-restricted-forest equivalence above preserves more structure than
the unqualified statement: a pinned entry image, one fixed external terminal,
directed outgoing-parent edges, and the prefix-normalized source domain.  The
bounded work therefore records the strongest justified conclusion as:

```text
Rocket Completable is in NP and is exactly a restricted pinned
rooted-spanning-forest extension problem.  Rocket NP-hardness is not proved
by the available unpinned citations.
```

This is a boundary theorem, not a claim that Rocket is polynomial or that it
is easy in general.

## Exact component parameter mapping

Count components in `F_q` including the fixed external vertex `C`.
For an acyclic prefix of length `m`, every processed source contributes one
edge and every edge merges two distinct components.  Since there are `N+1`
vertices:

```text
c(q) = (N + 1) - m.
```

The same number is the number of parent edges still missing from a full
Rocket tree.  If a prefix has a cycle, this identity is not a valid completion
parameter because the state is already rejected.  If one instead counts only
Origin-slot components and removes `C`, then a prefix with `t` supplied
`ProgramComplete` edges has `c_origin = N - (m-t)`; the inclusive definition
above is the clean parameter matching the rooted host with its fixed terminal.

For an extendible prefix, adding the next supplied edge has three cases:

* it joins two components and decreases `c` by one;
* it closes a cycle and is rejected;
* it is a second external terminal and is rejected by Rocket's one-terminal
  rule.

Thus `c` evolves monotonically from `N+1` at the empty prefix to `1` at a
complete tree.  Early prefixes have a large forest-component parameter; late
prefixes have a small one.  The parameter is therefore a plausible late-prefix
exact fallback, not evidence that all early prefix queries are cheap.

## FPT assessment

The cited rooted spanning-forest algorithm makes FPT completion research the
most direct exact fallback direction.  A Rocket adaptation would need to pass
the fixed external root, entry pin and frozen parent directions as explicit
constraints, then prove that the algorithm's `c` is the `c(q)` above rather
than a different count.

This task does not implement that adaptation.  The B3A oracle remains the
bounded truth authority, and B3B's connected matching remains valid only for
its stated one-component boundary.  Feeding B3A brute force into an FPT-shaped
wrapper would merely disguise factorial enumeration and is not a result.

## Prefix completion versus final canonical payload

The frozen identity computation is an optimization problem:

```text
min { Enc_V2(P, L) | L is a legal complete labelling }.
```

`Completable(T,e,k,q)` is a decision predicate about one partial prefix.  Even
if a future proof established that arbitrary Rocket prefix completion is
NP-hard, that would not by itself establish that directly computing the
minimum frozen payload is NP-hard.  A direct algorithm may exploit the exact
serialization objective, eliminate many labels without asking arbitrary
prefix questions, or solve special structures (as B2 does for a single
success path).  Conversely, a generic decision oracle could support a
self-reduction of some optimization tasks, but that is an additional
construction and not an automatic equivalence.

Therefore the correct research separation is:

```text
prefix feasibility complexity  !=  direct canonical-payload complexity.
```

No result in this task authorizes replacing direct objective research with a
hardness claim or changing frozen Enc_V2.

## Bounded repository validation

The research-only checker
`Tethers_core_rocket_v3_tree_complexity` independently constructs `F_q`,
counts its components, normalizes rooted forests into prefix form, and checks
whether a forest embedding witness exists.  It does not enter a production
call path and does not modify B3A or B3B.

The focused checker passed `31/31` checks, including:

* acyclic prefix, cycle and multiple-terminal classification;
* path, star, balanced and multi-branch semantic trees;
* the seven-node B3 counterexample shape;
* Rocket witness equivalence against the independent B3A oracle on bounded
  reachable prefixes;
* non-root-first normalization of a genuine multi-component rooted forest.

The inherited authorities also passed unchanged:

* B3A: `47634/47634`;
* B3B connected matching: `10/10`;
* B2 success-path suite: `69/69`;
* R3-3A: `39/39`;
* R3-1: `214/214`;
* R3-2: `4807/4807`;
* V2 generated corpus: `5000` valid, `0` mismatches.

Raw-ID/storage-order invariance remains an inherited V2/Rocket invariant.  The
research checker uses dense fixture coordinates only to construct bounded
witnesses; it never treats those coordinates as canonical ordering evidence.

## Recommendation

**FPT COMPLETION RESEARCH**

The exact Rocket restriction is now clear enough to stop searching for an
unproved general polynomial `Completable` recurrence.  The next bounded
research task should adapt the rooted-spanning-forest FPT state to Rocket's
fixed external root, pinned entry and directed prefix constraints, while
keeping direct frozen-payload canonisation as a separate question.  This
recommendation is recorded only; no such task is started here.

## References

* Garey and Johnson, *Computers and Intractability*, Theorem 4.6,
  `SUBFOREST ISOMORPHISM`:
  [public PDF](https://perso.limos.fr/~palafour/PAPERS/PDF/Garey-Johnson79.pdf).
* J. Liu, X. Chen, Y. Zheng, J. Wang and F. Shi, “Parameterized algorithms for
  the spanning forest isomorphism and containment on tree,” *Theoretical
  Computer Science* 1061 (2026), 115652,
  [ScienceDirect record](https://www.sciencedirect.com/science/article/pii/S0304397525005894),
  DOI [10.1016/j.tcs.2025.115652](https://doi.org/10.1016/j.tcs.2025.115652).
* Repository authorities:
  `docs/review/rocket-v3/R3_3B3A_PARENT_VECTOR_COMPLETION.md`,
  `docs/review/rocket-v3/R3_3B3B_LISTISO_REDUCTION.md`,
  `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_tree_completion.ml`,
  `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_tree_listiso.ml`.
