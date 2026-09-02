# R3-3B3B ListIso / Matching Reduction Investigation

## Scope and result boundary

This is a research-only investigation of the connected rooted success-tree
completion predicate.  It does not change Frozen Enc_V2, ProgramDigest V2,
the B2 path canonicaliser, the B3A oracle, Core validation, or any production
call path.

The exact question is whether a partial frozen parent/target vector can be
completed by a compact List Restricted Tree Isomorphism (ListIso) or exact
matching/tree-DP state.  A candidate is acceptable only if it agrees with the
independent B3A brute-force `Completable` oracle on every required prefix.

The two inherited negative results are prerequisites, not implementation
details:

* `parents = [6,2,3,6,5,6,-1]` has the historical rank target `6` but exact
  target `5` at continuation slot 7; the frozen payload first differs at byte
  55 (`0x36` versus `0x35`).
* `parents = [1,2,-1]`, entry semantic vertex `0`, prefix
  `[2, ProgramComplete]` is accepted by the old local-capacity candidate but
  rejected by exact brute force.

## Formal problem

Let `T` be the semantic rooted tree over Origins.  `parent_T(u)` is either
another Origin or the fixed external `ProgramComplete` root.  Let `e` be the
distinguished entry Origin and let `k` be its fixed numeric label.

For a prefix `q`, numeric slots are `1..N`.  A supplied entry is either
`q[i] = Complete` or `q[i] = j`.  Define `Completable(T,e,k,q)` to mean that
there is a bijection `L : Origins(T) -> {1..N}` with `L(e) = k` and:

```text
q[i] = Complete  => parent_T(L^-1(i)) = ProgramComplete
q[i] = j         => parent_T(L^-1(i)) = L^-1(j)
```

The second rule is a binary placement constraint.  It relates the semantic
vertex occupying slot `i` to the semantic vertex occupying slot `j`; it is not
just a property of either endpoint independently.

An equivalent partial-structure formulation is useful.  Build `P_q` with one
numeric vertex for every slot mentioned by the processed sources and their
targets, and one directed edge for every supplied `q[i] = j`.  A `Complete`
entry is an edge to the fixed external root.  Then `Completable` is equivalent
to the existence of an injective map from `P_q` into `T` plus the external
root, fixing slot `k` to `e`, preserving every supplied parent edge.  Slots
not mentioned by `q` are isolated and can receive the unused semantic Origins
after the constrained map is chosen.

This is a partial forest-embedding problem with simultaneous injectivity.  It
is not ordinary rooted-tree isomorphism because unmentioned parent edges are
unknown rather than absent.

## Direct standard-ListIso reduction attempt

The natural attempted construction is:

* `G = T`, with its external `ProgramComplete` root represented explicitly;
* `H =` a complete numeric target tree whose vertices are slots `1..N` plus
  the external root;
* the ListIso allowed-image list for a semantic vertex `u` contains exactly
  the numeric slots to which `u` may be assigned, including the entry list
  `{k}` for `e`.

This cannot be completed from a partial `q` without first choosing the entire
unknown target tree `H`.  For `q[i] = j`, the required statement is:

```text
if L(u) = i, then L(parent_T(u)) = j
```

That is a binary relation between the images of `u` and `parent_T(u)`.  A
standard ListIso instance supplies only unary lists `u -> allowed slots`;
those lists cannot express that the image chosen for `u` and the image chosen
for its parent must be the particular pair `(i,j)`.  Replacing the binary
condition by the unary condition `u may occupy i` loses the parent constraint.

### Smallest obstruction in the inherited corpus

Take the semantic path `0 -> 1 -> 2 -> ProgramComplete`, entry `0`, and the
prefix:

```text
slot 1 -> slot 2
slot 2 -> ProgramComplete
```

The entry restriction fixes slot 1 to semantic vertex 0.  The first edge then
forces slot 2 to semantic vertex 1, because it is the semantic parent of 0.
The second edge simultaneously requires that same semantic vertex 1 have
`ProgramComplete` as parent, which is false: its parent is semantic vertex 2.

The two constraints are individually plausible placement restrictions but are
jointly inconsistent.  This is exactly the B3A false positive.  A direct
ListIso encoding would need a gadget or a target-tree enumeration that carries
the pairwise placement relation.  Neither is standard unary-list ListIso, and
no sound-and-complete tree gadget compilation has been established here.

Therefore the direct reduction is **not established**.  The precise
obstruction is the unresolved target tree together with binary constraints
between numeric-slot images, not a failure of ordinary complete-tree ListIso.

## Matching/tree-DP state required for a valid generalisation

Any exact extension must retain the partial pattern `P_q`, not only degrees or
connected-component counts.  A sound candidate state needs at least:

1. the current partial-pattern vertices and exact parent edges;
2. possible semantic images for every numeric pattern vertex;
3. the fixed entry pair `(slot k, semantic e)`;
4. the external-root/`Complete` constraint;
5. injective resource use across all simultaneously placed pattern vertices;
6. for every pattern vertex with children, a bipartite matching between its
   pattern children and distinct host children;
7. a global disjointness state for separate partial-pattern components that
   may be placed in the same host tree without overlapping.

The first six items describe ordinary bottom-up tree matching.  Item 7 is the
part that the B3A local-capacity state omitted: two locally valid component
placements may compete for the same semantic vertices, and a choice at one
numeric slot can determine the only legal image for another slot.

For a connected partial component, the local recurrence is conceptually:

```text
CanMap(pattern_node p, host_node h) iff
  every required terminal condition agrees, and
  the required pattern children of p can be matched injectively
  to distinct host children of h whose CanMap states are true.
```

This is a genuine matching condition, not a degree comparison.  It is sound
and complete for one fully specified connected component.  It is not by
itself a completion theorem for a partial forest: the embeddings of multiple
components must also be chosen disjointly in one global host tree.

## Why the current bounded result stops here

The B3A oracle can answer the full predicate by enumerating complete
label-assignment bijections.  Reusing that enumeration inside a proposed
candidate would merely rename the forbidden factorial search and would not be
a ListIso/matching reduction.

The direct unary-list construction is disproved by the binary coupled
constraint above.  A connected-component matching recurrence is exact only
for its stated connected state, while the general prefix can contain several
partially specified components whose host placements must be globally
disjoint.  No compact polynomial state that captures that global packing was
established in this bounded investigation.

Consequently this document records a precise research frontier rather than
claiming a production-ready theorem.  The new research module is restricted
to the explicit partial-pattern and matching state described above; it must
remain separate from the B3A brute-force authority and must fail closed when
that state cannot certify a result.

## The required seven-node distinction

For `[6,2,3,6,5,6,-1]`, the historical rank vector chooses target 6 at source
slot 7 while the exact oracle chooses target 5.  Both choices can look locally
compatible with subtree descriptions.  The exact prefix state must preserve
which semantic vertex is occupying every already-fixed slot and which parent
that semantic vertex requires.  Both complete vectors are legal tree
placements, but at the first differing frozen continuation target the encoded
target `5` is smaller than `6`, so target 5 wins the exact byte objective.
Matching is what certifies the coupled placement constraints while the frozen
byte comparison selects between the surviving completions.  A subtree rank or
local capacity count does not retain this coupled matching information.

## Remaining theorem gaps

The following are deliberately unresolved and block any B3 constructor:

* a sound-and-complete compilation of partial binary parent constraints into
  standard unary-list Tree ListIso;
* a polynomial global matching/tree-DP recurrence for disjoint partial
  component placement in one host tree;
* a proof that the state remains exact when decimal-width byte ordering is
  reintroduced;
* a production integration boundary.

No production B3 canonicaliser, forest solver, cross-family support, R3-3C or
R3-4 work is authorised by this result.
