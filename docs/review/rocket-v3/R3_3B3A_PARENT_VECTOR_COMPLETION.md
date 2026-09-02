# R3-3B3A Parent-Vector Completion Investigation

## Scope

This is a research-only Origin success-tree investigation.  The model uses
dense fixture positions as construction coordinates, not as canonical evidence.
It does not modify frozen Enc_V2, Core validation, R3-1/R3-2, R3-3A, B1 or B2,
and it does not introduce a production canonicaliser.

## Historical B3 counterexample

The reported semantic tree is:

```text
parents = [6, 2, 3, 6, 5, 6, -1]
```

where `parent[i]` is the success target of dense Origin `i`, and `-1` is
`ProgramComplete`.  Entry is dense Origin `0`.

The historical rank candidate assigns slots:

```text
[0, 6, 5, 3, 4, 2, 1]
```

The independent exact frozen parent-vector oracle assigns:

```text
[0, 6, 3, 5, 2, 4, 1]
```

The first continuation-vector difference is numeric source slot `7`: the rank
candidate emits target `6`, while the exact candidate emits target `5`.  The
focused test also compares both complete payloads through the existing frozen
`encode_program` and records their first differing payload byte.

The failed B3 theorem was therefore not about tree construction.  A static
subtree rank describes a local rooted shape, but discards the global placement
of that subtree's parent and the effects of all previously assigned numeric
source slots.  Equal or ordered subtree descriptions do not prove that two
partial label-allocation states are interchangeable under the frozen parent
vector objective.

## Exact predicate

For a rooted semantic tree `T`, distinguished semantic entry `e`, fixed entry
label `k`, and partial vector `q`, define:

```text
Completable(T, e, k, q)
iff
there exists a bijection L from semantic Origins to numeric labels such that
L(e) = k, every supplied q[i] equals the frozen target of the Origin at label i,
and the complete labelled success relation is isomorphic to T.
```

The implementation contains an independent brute-force oracle for this
definition.  It enumerates all bijections consistent with the fixed entry and
checks the supplied prefix only at complete assignments.  It does not call a
candidate feasibility routine.

## Candidate state investigated

The first bounded candidate retains only:

* the fixed prefix edges;
* local self-edge and directed-cycle validity;
* the number of fixed children of the external Complete root;
* maximum semantic Origin child degree;
* whether the fixed entry edge has the correct terminal/non-terminal kind.

This is exposed as `local_capacity_candidate`, deliberately not as
`Completable`.  It is a necessary-condition state, not an asserted theorem.

## Result

The candidate is disproved by the three-node path:

```text
parents = [1, 2, -1], entry = 0
partial vector = [2, ProgramComplete]
```

The fixed prefix is locally acyclic, has valid target labels and fits all local
degree capacities.  It is nevertheless impossible: fixing label `1` to the
entry and making label `2` the terminal Origin would require the entry's parent
to be terminal while also requiring that same label `2` to have a non-terminal
parent.  The brute-force oracle rejects it while the local candidate accepts it.

This is the missing information in the attempted compact state: local degree
and component facts do not retain the coupled semantic placement of the fixed
source slots and their future parent slots.

The test exhaustively checks every reachable prefix generated from all dense
single-root parent arrays through size five and the required seven-node B3
counterexample, and includes the negative three-node state above.  That proves
the oracle machinery and rejects this candidate recurrence; it does not prove a
replacement compact polynomial recurrence.

## Boundary decision

No compact exact `Completable` theorem has been established in this bounded
investigation.  The task must therefore stop before a left-to-right constructor:

* no heuristic subtree ranking is revived;
* no complete permutation constructor is added;
* no B3B, forest, cross-family, R3-3C or R3-4 work begins.

The smallest next architectural question is whether the missing coupled
placement information admits a separately specified exact tree-automaton or
matching recurrence.  That decision is outside R3-3B3A.
