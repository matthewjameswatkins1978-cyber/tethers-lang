# C-B4I3R2 — Rocket V2 Full-Burn Qualification

Status: `EXPERIMENTAL; ACCEPTED-BASE QUALIFICATION; EXACT REDUCTIONS ONLY`

Branch: `codex/experiment-rocket-full-burn-v2`
Base: `1da4b4e937372abf28b749234d17070d2f377076`

## Decision

The frozen objective remains the sole authority:

```text
CanonicalPayload_V2(P) = min { Enc_V2(P, lambda) | lambda in Lambda(P) }
```

under the existing unsigned byte-lexicographic comparison.  Neither
refinement, a graph library, an automorphism claim, nor a best-so-far payload
replaces it.

**Ten-year answer:** I would trust this *bounded exact design* to protect a
ProgramDigest only with the retained proof guards, validation, fail-closed
budget, and exhaustive fallback.  I would not trust a claim that it is a
general graph canonicaliser, and would reject any future pruning that lacks an
Enc_V2 earliest-difference proof or a complete typed automorphism witness.

## What survived

| Reduction | Exact condition | Why it is sound | Qualification |
| --- | --- | --- | --- |
| Top-level Facts | The entire Fact inventory is distinct top-level `Evaluation_input` | The first Fact-sensitive collection is a fixed sequence of own labels; sort exact encoded provenance suffixes | Existing adversarial and decimal tests retained |
| `entry_origin` | Validator-valid program with an entry Origin | It is the first Origin-sensitive field; choose the minimum `encode_int` byte string, not numeric `1` | 8/9/10/11/12/19/20/21 checked |
| Dependency-closed program Anchors | `entry_origin`; no success continuations; all program sites are Anchors; declared Facts are only evaluation inputs; template origin collections empty | Fact labels and entry label are fixed before the sole program-Origin collection; each remaining own-label slot differs first by its exact Anchor suffix | New, oracle/baseline projection plus hostile/reversed metamorphism |
| One physical Branch collection | All Branches are in exactly one program/template collection | Each own-label slot is fixed; sort the exact Branch body after Origins are assigned | Retained and extended through 21 labels |
| Program Roles | No program-origin `Role_proxy` Fact or `Fact_through_role` binding | No Program Role label is emitted before the Program Role list; sort exact role bodies | Retained, 6 roles `720 -> 1`, decimal matrix |
| Template Roles | No earlier in-scope role reference and pairwise-distinct exact role bodies | The template role list is earlier than its objective; pairwise-distinct bodies decide first.  A conservative pre-admission witness uses distinct length-prefixed fulfilments | New, 6 roles `720 -> 1`, tie/reference guards, decimal matrix |

All reductions are local exchange proofs over the actual emission order.  They
do not use raw IDs, colour classes, hash values, or semantic-looking graph
forms as tie-breakers.

## What broke or was deliberately rejected

| Candidate | Result | Evidence/reason |
| --- | --- | --- |
| Global Fact sort | Rejected | Input-versus-later-declared Fact location changes the earliest emitted bytes; length-prefix ordering defeats scalar intuition. |
| Unqualified Template Role sort | Rejected | Equal role-list bodies can leave the later template objective as the first difference.  The tied-body test retains both assignments. |
| Batch sort in current ordering | Rejected | Batch bodies can contain Template/Role labels still free at Batch assignment; a provisional map is not an admissible bound. |
| Template sort / broad reordered search | Rejected | Template, Batch, Role, and Origin dependencies form cycles on realistic shapes.  No fixed emission prefix was established. |
| WL-cell/orbit pruning | Rejected | Equal refinement colour is not a typed-Core automorphism witness. |
| Generic fixed-prefix B&B | Rejected | No nonempty prefix was proved fixed over a remaining free family.  `prefix_subtrees_pruned = 0`. |
| Graph canonical form as result | Rejected | A graph-library canonical label optimises its graph encoding, not frozen `Enc_V2`. |

The full burn found no payload/digest differential mismatch.  One test-harness
assumption was corrected: on this 63-bit OCaml runtime both `19!` and `20!`
fit in `int`; `21!` is the first tested overflow.

## Qualification evidence

Every tractable fixture compares exactly with the slow oracle and accepted
exhaustive baseline.  Large forms use an exhaustive smaller projection of the
same shape plus raw-ID/storage metamorphism.  The focused suite passed:

- 10,000/10,000 valid deterministic generated programs:
  `oracle == baseline == Rocket` for payload and digest;
- high bytes, NUL, empty strings, length-prefix traps, hostile IDs, reversed
  origin/branch/outcome/role/fact storage, multiplicity, Together, mixed
  Batch/Template scopes, split Branch collections, same raw Role IDs in
  different scopes, and structurally symmetric non-automorphic cases;
- exact negative guards for continuations, Together origins, earlier Role
  references, tied Template Role bodies, split Branch collections, and invalid
  input Origin provenance;
- decimal counts 8/9/10/11/12/19/20/21 for Facts, Origins, Branches, Program
  Roles, and Template Roles;
- semantic mutation test: changing a host snapshot key changes the frozen
  payload; representation-only reversal does not.

## Factorial transformations

| Shape | Raw candidates | Rocket leaves |
| --- | ---: | ---: |
| 8 distinct top-level Facts | 40,320 | 1 |
| 21 distinct top-level Facts | 21! | 1 |
| 8 one-collection Branches | 40,320 | 1 |
| 21 one-collection Branches | 21! | 1 |
| 21 dependency-closed program Anchors | 21! | 1 |
| 6 Program Roles | 720 | 1 |
| 6 distinct dependency-closed Template Roles | 720 | 1 |
| A: 8 Facts x 8 Branches | 1,625,702,400 | 1 |
| B: 6 Program Roles x 8 Branches | 29,030,400 | 1 |
| C: 8 Facts x 8 Branches x 6 Roles | 1,170,505,728,000 | 1 |
| Template Role tie or earlier reference | 2 | 2 |
| Split program/template Branch collections | 2 | 2 |

For the compound shapes, 3x3, 3x3, and 3x3x3 exhaustive projections were
checked against both oracle and baseline before the large metamorphic runs.

## Measured run

OCaml 5.5.0 / Dune 3.24.0, Windows host.  Baseline timing is one exhaustive
run; Rocket timing is a 1,000-call average.  Allocation figures are
`Gc.quick_stat` observations, not a cross-machine promise.

| Fixture | Baseline time | Rocket time/call | Rocket leaf count |
| --- | ---: | ---: | ---: |
| 7 distinct Facts (5,040) | 0.0600 s | 0.000034 s | 1 |
| 8 distinct Facts (40,320) | 0.5330 s | 0.000039 s | 1 |
| Persistent Branch (576) | 0.0070 s | 0.000032 s | 1 |
| 8 high-symmetry Branches (40,320) | 0.6000 s | 0.000036 s | 1 |

Typical one-leaf Fact/Branch runs allocated roughly 5,600--6,800 minor words
and 1,010--1,045 major words per call.  No prefix or orbit pruning was
reported; all saved leaves come from the explicit local proofs.

## Research result

McKay and Piperno describe the individualisation/refinement paradigm and the
practical role of nauty/Traces.  nauty documents both automorphism groups and
canonical labelling; its guide defines orbits through actual automorphisms.
That supports using a typed graph representation for diagnostics, witness
search, or a test-only oracle, but not treating a refinement cell as an orbit
or returning a library label as V2 output.

- McKay & Piperno, *Practical Graph Isomorphism, II*:
  <https://arxiv.org/abs/1301.1493>
- nauty/Traces: <https://users.cecs.anu.edu.au/~bdm/nauty/>
- nauty/Traces User's Guide 2.9.3:
  <https://users.cecs.anu.edu.au/~bdm/nauty/nug29.pdf>
- bliss: <https://users.aalto.fi/~tjunttil/bliss/>

## Architecture to keep / throw away

Keep frozen bytes as authority; validate before search; retain typed
scope-aware refinement as diagnostic machinery; maintain a small proof ledger
with negative guards; pre-admit only the reduced count justified by a proof;
and fail closed without a partial digest.

Throw away colour-as-orbit logic, graph-canonical-form substitution, provisional
label sorting, raw-string ordering, timeout answers, and any B&B claim without
a fixed frozen prefix.

The worst remaining factorial families are non-entry/relational Origins,
Batches, ItemTemplates, split Branch collections, referenced/tied Template
Roles, and Program Roles referenced before their own collection.  They remain
exhaustive by design.

## Files

- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_ir.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_ir_test.ml`
- `docs/perf/C_B4I3R2_ROCKET_FULL_BURN_QUALIFICATION.md`
