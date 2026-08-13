# C-B4I3R — Rocket Hybrid Torture Lab

Status: `EXPERIMENTAL; EXACT REDUCTIONS ONLY`

Branch: `codex/experiment-rocket-hybrid-v2`
Base: `a1d9c3b6ad5cfbb45732f50efcca3231b21ecb4d`

## Answer

**Would I trust this design to protect a ProgramDigest for ten years?**

**YES for the frozen objective and the reductions actually implemented; NO for
any claim that it is already a general graph canonicaliser.**  Every retained
shortcut is an exchange proof over frozen `Enc_V2` bytes.  Every remaining
family is enumerated exactly.  No colour, graph-library label, cached hash, or
best-so-far candidate becomes semantic authority.

The frozen law remains unchanged:

```
CanonicalPayload_V2(P) = min { Enc_V2(P, lambda) | lambda in Lambda(P) }
```

with the existing unsigned byte-lexicographic comparator.

## What survived

### 1. Top-level distinct Evaluation-input Facts

The Fact shortcut is active only if the collected Fact inventory is exactly the
top-level `input_facts` collection and every Fact has a distinct
`Evaluation_input` encoded provenance.  The first label-sensitive fields are
then that one sorted V2 list.  Sorting the exact emitted provenance fragments
and removing adjacent inversions gives the byte minimum.

The pre-existing wider shortcut was disproved: an input Fact with host key `aa`
and a later declared Evaluation-input Fact with host key `z` demonstrates that
global Fact sorting is wrong under length-prefixed bytes (`1:z < 2:aa`) because
the input location is emitted first.  Rocket retains exhaustive Fact labels on
that shape.

### 2. `entry_origin`

The entry Origin is the first Origin-sensitive V2 field after only the version,
input Facts, and guards.  Validator rules require input Facts to have
`Evaluation_input` provenance, so those earlier fields contain no Origin label.
The entry Origin is fixed to the available label whose *encoded decimal bytes*
are smallest.  This deliberately compares `encode_int` rather than assuming
numeric label 1; label counts crossing decimal boundaries are covered.

### 3. One physical Branch collection

Branch labels occur only as each Branch's own prefix.  If all Branches occur in
one physical program or one template Branch collection, the emitted numeric
prefix sequence is fixed (`1..N`); the remaining exact Branch body may be
sorted under the frozen byte comparator.  An adjacent inversion makes the
first differing body larger, proving the sorted assignment is minimal.

The body is recomputed after Origin assignments, because it contains subject
and `Continue_to` Origin labels.  The shortcut is disabled for split
program/template Branch collections because their global label space couples
two different V2 sections.

### 4. Unreferenced Program Roles

Program Roles can similarly be body-sorted only when no program Origin site
contains a `Role_proxy` Fact or `Fact_through_role` binding.  Otherwise a role
label is visible before the program Role collection and the local exchange
proof does not apply.  Template Roles remain exhaustive: their own template
body can expose them earlier.

This is a useful non-graph reduction: six unreferenced Program Roles give
`720 -> 1`; an earlier `Role_proxy` guard retains `2 -> 2`.

## What broke or was thrown away

| Candidate | Result | Reason |
| --- | --- | --- |
| Global Evaluation-input Fact sort | Rejected | Actual valid input-versus-declared counterexample. |
| One-collection Batch sort | Rejected | Batch bodies can contain Template and Role labels not yet fixed in the existing search order. A provisional map would not be an admissible proof. |
| WL/refinement-cell pruning | Rejected | Equal colours are not automorphism witnesses. |
| Graph-library canonical label as V2 result | Rejected | It optimises a graph form, not frozen length-prefixed `Enc_V2` bytes. |
| General fixed-prefix branch-and-bound | Rejected | No nontrivial prefix was proved fixed across all remaining family assignments. |
| Generic orbit pruning | Rejected | No exact typed-Core automorphism witness plus Enc_V2 preservation proof was implemented. |

## Research conclusion

nauty/Traces and bliss confirm that individualisation/refinement plus verified
automorphism information is the standard practical machinery for graph
canonisation.  They are useful models and possible *test-only* oracle tools,
not replacements for the frozen V2 objective:

- McKay and Piperno, *Practical Graph Isomorphism, II*:
  <https://arxiv.org/abs/1301.1493>
- nauty/Traces documentation:
  <https://users.cecs.anu.edu.au/~bdm/nauty/>
- bliss documentation on canonical labels and automorphism groups:
  <https://users.aalto.fi/~tjunttil/bliss/>

A future internal graph model must encode every typed relation, scope, and
ordered emission dependency exactly.  A library-produced label would still
need to be mapped back to a V2 labelling and verified by frozen `Enc_V2`.

## Torture evidence

All tractable fixtures compare exactly against the slow oracle and exhaustive
baseline.  Larger factorial cases use smaller exhaustive projections of the
same shape and raw-ID/storage metamorphism.

Added coverage includes:

- Branch counts 9, 10, 11, and 12, including decimal label boundaries;
- distinct Branch bodies, different subjects, `Continue_to` targets, multiple
  outcomes in hostile order, and late differences;
- reversed storage, reversed outcome storage, hostile raw IDs, high-byte and
  length-prefix scalar strings;
- Together, Batch plus template, template-scoped Role, and mixed Origin/Batch;
- a split program/template Branch collection guard;
- an input `Origin_provenance` validation-before-search guard;
- unreferenced Program Role sorting and the earlier-role-reference guard;
- existing same-raw scoped Roles, multiplicity, symmetry/non-automorphism,
  generated 1,000-case corpus, and metamorphic suites.

No payload or ProgramDigest differential mismatch occurred.

## Measured transformations

The final focused run used OCaml `Gc.quick_stat` for allocation/collection
observations.  Baseline time is one run; IR time is 1,000-run average and is
not a precision microbenchmark.

| Fixture | Raw candidates -> leaves | Evidence |
| --- | --- | --- |
| 7 distinct input Facts | `5,040 -> 1` | baseline 0.0625 s; IR 0.000035 s/call |
| 8 distinct input Facts | `40,320 -> 1` | baseline 0.5611 s; IR 0.000038 s/call |
| Persistent Branch | `576 -> 6` | entry Origin plus exact Branch sort; 570 leaves avoided |
| 8 one-collection Branches | `40,320 -> 1` | baseline 0.6300 s; IR 0.000038 s/call |
| 11 one-collection Branches | `39,916,800 -> 1` | baseline rejects default 5M cap; Rocket admits by proof |
| 6 unreferenced Program Roles | `720 -> 1` | exact role-body ordering |
| earlier Program `Role_proxy` | `2 -> 2` | guard preserves exhaustive search |
| template-scoped Roles | `4 -> 4` | deliberately no unproven reduction |

The focused performance run observed approximately 5,374--6,554 minor words
and 1,016--1,035 major words per one-leaf Fact/Branch call, and approximately
16,122 / 1,054 respectively for Persistent Branch.  These values are evidence
for this host and run, not a portability guarantee.

## Architecture to keep

- Frozen `Enc_V2` as the only identity authority.
- Validation before any search reduction.
- Typed, scope-aware individualisation/refinement as diagnostic/search
  machinery, never semantic authority.
- A small ledger of local byte-order exchange proofs, each with an explicit
  negative guard and differential test.
- Raw candidate counts separate from reduced deterministic pre-admission;
  failure remains `Canonicalisation_too_complex`, with no fallback answer.

## Architecture to throw away

- Treating WL cells as orbits.
- Comparing raw identifiers or scalar strings instead of frozen encoded bytes.
- Provisional-label sorting when later label families are still free.
- Calling a generic graph canonical form the V2 payload.
- Timeout-based or best-so-far canonicalisation.

## Worst remaining factorial families

Origins not fixed by `entry_origin`, Batches, ItemTemplates, split Branch
collections, Template Roles, and Program Roles referenced from earlier program
Origin fields remain factorial.  That is intentional: Rocket has no proof yet
that their candidates can be discarded.

## Files

- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_ir.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_ir_test.ml`
- `docs/perf/C_B4I3R_ROCKET_HYBRID_TORTURE_LAB.md`
