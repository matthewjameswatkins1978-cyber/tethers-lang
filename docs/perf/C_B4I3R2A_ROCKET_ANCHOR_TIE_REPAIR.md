# C-B4I3R2A — Rocket Anchor Tie Repair and Dense Differential Generator

Status: `EXPERIMENTAL CORRECTNESS REPAIR; EXACT ENC_V2 AUTHORITY`

Branch: `codex/experiment-rocket-anchor-tie-repair-v2`

Base: `55099152a10e87c844041a13cb43630e3696e224`

## Verdict

The hostile review was correct. Commit `55099152` was unsound for equal
dependency-closed Program Anchor bodies. The defect was reproduced before the
algorithm was changed, then repaired by ordering distinct exact body classes
while exhaustively enumerating every permutation inside an equal-body class.

No frozen V2 semantic, encoder, domain separator, digest rule, baseline,
oracle, or validator behavior changed.

## 1. Exact counterexample

The minimal valid witness has three Program Anchors:

- `a0` is `entry_origin` and has event `entry`;
- `a1` and `a2` both have event `tie` and no declared Facts;
- there are no success continuations;
- one later Branch has `branch_subject = a2`.

With source order `[a0; a1; a2]`, `55099152` fixed the entry label, sorted the
two equal Anchor bodies, and broke the equality by source index. It therefore
assigned `a1 = 2`, `a2 = 3` and encoded only one leaf. Oracle and baseline
assigned `a2 = 2`, `a1 = 3` because the Program-Origin collection is unchanged
by that swap and the later Branch is then smaller.

The exact first differing Branch-section bytes are:

```text
oracle/baseline: 1;1;2;1;0:1:
55099152 Rocket: 1;1;3;1;0:1:
                         ^ branch_subject Origin label
```

The reproduced digests were:

```text
oracle/baseline tethers:v2:sha256:2ebeeb6a465d755b1bf04443d881a08020f26c3c5c8b22ca8df8b075c7f04806
55099152 Rocket tethers:v2:sha256:0daaf08ff3a422f7f440e5e0aa9781c64e2376b5c5dbcaad79b767e34968b89a
```

The raw candidate count was `3! = 6`; unsound Rocket encoded one leaf.

## 2. Why `55099152` was unsound

The adjacent-exchange proof decides the order of *different* exact bodies.
It proves nothing inside an equal-body class. `55099152` nevertheless used the
source index as the tie-breaker and discarded every other assignment. Source
index did not enter `Enc_V2` at the Program-Origin collection, but later Origin
references still observed the chosen labels. Thus a search-only ordering was
accidentally promoted to semantic authority.

Reversing storage demonstrated the same flaw from the invariance direction:
the arbitrary survivor changed even though the Core structure did not.

## 3. Exact repair

After Fact labels and the entry Origin label are fixed, Rocket computes each
free Anchor's exact frozen body with its own Origin prefix removed. It then:

1. sorts the distinct exact body classes by unsigned frozen byte order;
2. assigns each class its corresponding ordered set of free Origin-label slots;
3. enumerates every permutation of those labels within the class;
4. takes the ordinary full `Enc_V2` minimum across all residual leaves.

No raw ID or storage index resolves an equality. Source index remains only an
irrelevant container handle while the residual permutation enumerator visits
all tied assignments.

Examples:

```text
five Origins total, entry fixed, free classes [A] [B,B,B]
full raw search:       5! = 120
after entry reduction: 4! = 24
after body classes:    3! = 6 residual leaves
```

The previous four-identical-Anchor Persistent Branch fixture is now correctly
`576 -> 6`, not the unsound `576 -> 1`.

## 4. Equal-body Origin classes

Equal classes are live. A class of size `k` contributes exactly `k!` residual
Origin assignments. Multiple classes multiply independently. A class of size
one contributes one.

Runtime equality is equality of the actual exact body bytes. Under the narrow
dependency-closed guard, pre-admission can determine the same equality classes
without choosing labels: the Anchor body begins with `event_name`, followed by
the declared Fact multiset under an injective Fact labelling. Two bodies can be
equal only when their event strings and declared Fact-ID multisets are equal.
Raw Fact IDs are used only to establish reference equality inside one program,
never to order bodies or break a tie.

## 5. Pre-admission accounting

Pre-admission now multiplies the factorial of every free equal-body class.
It no longer returns Origin factor `1` merely because the structural
dependency-closed guard passes.

The 12-Origin boundary witness has one tied pair and otherwise distinct bodies:

```text
raw Lambda Origin factor: 12! = 479,001,600
entry-fixed factor:        11! = 39,916,800
proved residual factor:     2! = 2
```

A `max_leaves = 2` budget admits it and produces the exact answer. A
`max_leaves = 1` budget rejects it fail-closed before search.

## 6. Tie torture evidence

The repaired algorithm agrees exactly with oracle and exhaustive baseline on
the minimal witness and its small projections for:

- later `branch_subject` observation;
- later `Continue_to` observation;
- multiple Branches, multiple outcomes, and references to both tied Origins;
- forward, reversed, and hostile raw-ID storage;
- identical high-byte/NUL bodies;
- almost-identical bodies differing late;
- a three-way equal class (`3!` residual);
- counts 9, 10, 11, and 12, including the `encode_int` decimal transition.

Count 9 is directly exhaustive at the boundary. Counts 10--12 use the same
small exhaustive observer projections plus large boundary metamorphism; the
accepted exhaustive baseline's global budget cannot execute 11!/12! raw
spaces, while repaired Rocket executes exactly the proved residual two leaves.

## 7. Dense differential generator

The old generator varied only tiny cardinalities and mostly produced shallow
near-duplicates. It has been replaced with 16 deterministic valid archetypes
and 5,000 cases (seed `308386`). Each case has no more than 720 raw Lambda
candidates, so oracle, exhaustive baseline, and Rocket must all return an
answer; invalid cases and `Oracle_too_large` are generator failures rather than
silently skipped data.

The archetypes and their deterministic variants combine:

- top-level and declared Facts;
- valid `Evaluation_input` and `Origin_provenance`;
- Anchors, Actions, Together, and Batches;
- Branch subjects, `Continue_to`, multiple outcomes, and split Branch storage;
- success continuations;
- Program Roles, Template Roles, `Role_proxy`, and `Fact_through_role`;
- templates and two-template cases;
- identical raw Role IDs in Program and Template scopes;
- high bytes, NUL, empty strings, length-prefix traps, hostile identifiers,
  reversals, and alternate relation choices;
- the repaired equal-Anchor observer shapes.

Mismatch handling prints the deterministic seed, case number, and archetype,
then raises immediately. The generator does not count and continue.

Final differential result:

```text
seed=308386
total=5000
valid=5000
oracle payload == baseline payload == Rocket payload: 5000/5000
oracle digest  == baseline digest  == Rocket digest:  5000/5000
mismatches=0
```

## 8. Performance impact

Focused Windows run using OCaml 5.5.0 and Dune 3.24.0:

| Fixture | Baseline | Corrected Rocket | Transformation |
| --- | ---: | ---: | ---: |
| Persistent Branch | 0.0070 s | 0.000125 s/call | `576 -> 6` |
| Four identical Origins | <0.0005 s | 0.000057 s/call | `24 -> 6` |
| Eight distinct Facts | 0.5386 s | 0.000037 s/call | `40,320 -> 1` |
| Eight distinct Branches | 0.5954 s | 0.000036 s/call | `40,320 -> 1` |

Persistent Branch now allocates about 16,908 minor words per call, versus the
unsound one-leaf run's roughly 6,160. This is the necessary price of retaining
the six observable tied assignments. It remains roughly two orders of
magnitude below the exhaustive baseline wall time on this fixture.

Distinct Anchor bodies retain the full exchange reduction, including the
existing large decimal-boundary metamorphic cases. Only genuine equality
classes pay residual factorial cost.

## 9. Is the corrected Anchor reduction worth keeping?

Yes. Realistic Anchors commonly differ by event name or declared Fact set, so
distinct classes still remove large factors. Symmetric identical Anchors pay
exactly the unresolved class factorial, which is the information-theoretically
honest search until later bytes decide it. The proof boundary is now explicit,
testable, and mirrored by pre-admission.

## 10. B4I4 nomination

**Yes: nominate the corrected branch for B4I4 independent review, not
`55099152`.** The nomination is justified by the reproduced counterexample,
proof-shaped residual repair, synchronized pre-admission, complete existing
Rocket rerun, and dense 5,000-case three-engine differential result.

This is a nomination, not self-acceptance. B4I4 should independently review
the body-equivalence argument and residual enumerator, and should preserve the
same stop-the-line standard.

The governing lesson is now encoded in implementation, tests, and admission:

```text
DISTINCT BODIES MAY BE DECIDED.
TIED BODIES REMAIN LIVE WHEN LATER BYTES CAN OBSERVE THEIR LABELS.
```
