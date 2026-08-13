# Canonical V2 — Experimental Search Reduction Report

Status: `EXPERIMENTAL; KEEP AS A FOLLOW-UP CANDIDATE`

Branch: `codex/experiment-fucking-go-for-it-v2`
Base: `defb8af28c6ce1d7b5df3a07a34ba70fe877eb60`
Frozen specification: `b37ef8c`
Gold oracle: `838251d`
Exhaustive baseline: `424a18e`

## Decision

**Would I keep this? YES, as a separately reviewable experimental IR
improvement.**  The changes are exact reductions of the frozen search space,
not a new encoding or a heuristic canonicaliser.  They remove large proven
factors on qualifying shapes, expose one pre-existing unsound fact shortcut,
and retain exhaustive enumeration anywhere the proof does not apply.

This report does **not** authorise a runtime cutover.  The IR remains an
independent engine used by its test executable; the frozen encoder, oracle,
and exhaustive production canonicaliser remain unchanged.

## Non-negotiable invariant

For every valid program `P`, the required result remains exactly:

```
CanonicalPayload_V2(P) = min { Enc_V2(P, lambda) | lambda in Lambda(P) }
```

where the comparison is the existing unsigned byte-lexicographic comparison.
`Enc_V2`, its field order, integer encoding, and validation semantics were not
changed.  In particular, a generic graph canonical label is not itself an
answer unless it is converted into a V2 labelling and then re-encoded with the
frozen encoder.

## Research and architecture conclusion

The nauty/Traces and bliss work is valuable prior art for individualisation,
refinement, and automorphism-aware search.  It is not a drop-in replacement
for this problem:

- A coloured graph model would have to encode each typed/scoped Core relation
  exactly, including ordered V2 field positions and list sorting rules.
- A canonical graph label from a library need not minimise the different,
  length-prefixed byte objective of `Enc_V2`.
- Equal Weisfeiler--Leman colours prove neither an automorphism nor an
  interchangeable V2 completion.  No WL-colour pruning was introduced.

Consequently this experiment uses direct, local exchange proofs over the
actual emitted V2 bytes.  An external nauty/Traces/bliss implementation could
later be useful as a *test-only* automorphism oracle, provided the exact typed
graph translation and re-encoding check are independently verified.

Primary and technical sources consulted:

- McKay and Piperno, *Practical Graph Isomorphism, II* (nauty/Traces):
  <https://arxiv.org/abs/1301.1493>
- nauty and Traces project documentation:
  <https://users.cecs.anu.edu.au/~bdm/nauty/>
- Junttila and Kaski, *Engineering an Efficient Canonical Labeling Tool for
  Large and Sparse Graphs* (bliss):
  <https://users.aalto.fi/~tjunttil/bliss/>
- OCaml runtime tracing documentation:
  <https://ocaml.org/manual/5.5/runtime-tracing.html>
- OCaml `Gc` API, used for the allocation/collection measurements below:
  <https://ocaml.org/manual/5.5/api/Gc.html>

## Frozen-format and source audit

The format audit covered sections 1, 6, 8, 9, 12, 13, 15, and 23--25 of
`CANONICAL_FORMAT_V2_SPEC_DRAFT.md`, plus the shared encoder in
`tethers_core_canonical_v2_format.ml`.

Relevant facts established from the source, rather than assumed from the IR:

- `Enc_V2` is structural and injective; signed canonical labels are written as
  ASCII decimal integers.
- V2 list sorting is by numeric canonical label, while the final payload
  comparison is unsigned lexical bytes.
- `entry_origin` is encoded before the origin collection, and valid
  `input_facts` must all have `Evaluation_input` provenance.
- A Branch encoding emits its own label followed by its body.  Branch IDs are
  not referenced by other V2 fields.
- Branches in separate physical collections are not freely exchangeable:
  their collection positions are fixed by the encoder.

## Implemented exact reductions

### 1. Corrected and narrowed Fact shortcut

The prior shortcut sorted all `Evaluation_input` Facts globally.  That was
unsound because a declared or aggregate Fact can occur later in the format and
receive an earlier label without changing the earlier `input_facts` position.

New regression witness:

- top-level input Fact has host key `aa`;
- declared Fact has host key `z`;
- both have `Evaluation_input` provenance.

The numeric/global ordering picks the declared Fact as label 1, but frozen V2
first emits the top-level Fact.  The oracle requires the `aa` input Fact to be
label 1.  The old IR demonstrably disagreed with the oracle on this valid
program.

The shortcut now applies only when every collected Fact occurrence is exactly
one of `p.input_facts`.  On that restricted shape, all relevant Facts first
occur in one emitted list; sorting by their exact first emitted provenance
fragment gives the lexical minimum.  Ties are byte-equivalent at that point
and are still resolved by the complete frozen encoding.

### 2. Exact `entry_origin` label fixing

For a valid program with `entry_origin`, its label is the first
origin-sensitive byte.  The implementation chooses the actual minimum among
the available encoded decimal labels, not merely numeric label 1 (for example,
the lexical relation between `10;` and `1;` matters).  Fixing that label is
therefore an exchange-proof reduction of the Origin permutation factor.

The implementation explicitly retains validation first.  A tempting
counterexample with an `Origin_provenance` input Fact is invalid by the frozen
validator and is covered by a regression test; it is not silently discarded
by search optimisation.

### 3. Exact one-collection Branch ordering

When, and only when, every Branch occurs in a single physical Branch
collection, the label prefixes at collection positions are fixed to `1..N`.
No other V2 field refers to a Branch label.  The implementation encodes each
Branch body with a probe labelling, removes its own prefix, sorts the exact
bodies using the frozen unsigned-byte comparator, and assigns labels in that
order.

For any adjacent inversion, swapping two neighbouring Branch assignments
changes the first differing body at its fixed label position; putting the
smaller body first strictly improves the full V2 payload.  Repeated inversion
elimination proves the sorted order is minimal.  Equal bodies are
byte-equivalent, so either order preserves the output.

The reduction is deliberately disabled when Branches are split between program
and template collections.  A dedicated two-collection test asserts that the
search remains exhaustive there.

### 4. Fail-closed reduced-budget admission

The IR now calculates a conservative, overflow-safe candidate count after only
the three exact reductions above.  It rejects before search if that reduced
count exceeds `max_leaves`.  The public/raw candidate-count arithmetic remains
available for evidence and comparison.  No timeout, best-so-far answer, or
fallback to the exhaustive baseline is used.

## What was tried and rejected

| Candidate idea | Outcome | Why it was rejected or constrained |
| --- | --- | --- |
| Globally sort all Evaluation-input Facts | Rejected; real counterexample found | V2 placement, not just Fact provenance, determines the first byte affected by the label. |
| Prune equal WL/refinement cells | Rejected | Equal refinement colour is not a proof of automorphism or equal V2 completion. |
| Use nauty/Traces/bliss canonical label as payload | Rejected | It optimises a graph representation, not the frozen `Enc_V2` byte order. |
| Sort Branches across program/template collections | Rejected | Collection order makes the apparent exchanges invalid. |
| Use a partial prefix without a proof that all earlier fields are fixed | Rejected | Such a bound is not admissible for arbitrary remaining family permutations. |

## Tests and differential evidence

The focused test executable now covers all prior oracle/baseline differentials
plus:

- the valid input-versus-declared Fact regression above;
- validation-before-search for invalid input origin provenance;
- a multiple-Branch-collection guard (two leaves retained);
- an eight-Branch single-collection case (`8! = 40,320` raw assignments);
- a two-Branch one-collection case with distinct encoded bodies, pinned to the
  frozen oracle and exhaustive baseline;
- an eleven-Branch single-collection case (`11! = 39,916,800` raw
  assignments), which the baseline rejects at its default budget while the IR
  admits after its exact reduction;
- budget failure on eight Program Roles, where no reduction applies;
- exact leaf-count assertions for the Persistent Branch fixture.

All test-visible valid fixtures match both the frozen oracle and exhaustive
baseline whenever those engines admit the fixture.  The generated valid corpus
contains 1,000 cases with zero payload or digest mismatches.  Metamorphic
storage permutations and raw-ID renames remain byte-identical.

## Measurements

Measurements come from the focused test executable on the Windows development
host.  Baseline time is one run.  IR timing is the average of 1,000 runs, and
allocation/GC data is taken from `Gc.quick_stat`; tiny wall times should not be
treated as precise microbenchmarks.

| Fixture | Raw candidates | IR leaves | Baseline time | IR time/call | Interpretation |
| --- | ---: | ---: | ---: | ---: | --- |
| 7 distinct input Facts | 5,040 | 1 | 0.0675 s | 0.000036 s | Exact Fact ordering removes the factor. |
| 8 distinct input Facts | 40,320 | 1 | 0.5300 s | 0.000034 s | Same reduction at a larger size. |
| Persistent Branch | 576 | 6 | 0.0070 s | 0.000116 s | Entry origin plus exact Branch ordering; 570 leaves avoided. |
| 8 equal-shape Branches, one collection | 40,320 | 1 | 0.5834 s | 0.000034 s | Exact Branch-body ordering removes the factor. |
| 11 Branches, one collection | 39,916,800 | 1 | baseline rejects | admitted | Baseline limit is avoided by proof, not raised. |
| Templates / Roles | 4 | 4 | below clock resolution | below clock resolution | No unjustified reduction. |

The benchmark reports per-call IR allocation in the same run: about 5,196--
6,554 minor words and 1,011--1,041 major words for the single-leaf Facts and
Branches cases, and about 15,597 / 1,058 respectively for Persistent Branch.
These are measurements, not a claim that the IR is ready for runtime adoption.

## Files changed

- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_ir.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_ir_test.ml`
- `docs/perf/CANONICAL_V2_EXPERIMENT_FUCKING_GO_FOR_IT.md`

No frozen-format source, validation source, production exhaustive canonicaliser,
V1 path, replay/trail code, or task-control packet was changed.

## Reproduction and remaining work

From `tethers-0.1/engine-ocaml`, using the declared OCaml switch:

```
opam exec --switch="D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml" -- dune build @all
opam exec --switch="D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml" -- dune runtest --force
```

Further improvement should be a separate authorised packet.  The promising
direction is an independently specified typed coloured-digraph model used as a
differential automorphism oracle, followed by narrowly proved orbit reductions
against the real `Enc_V2` objective.  It must not promote WL colours, a graph
library label, or a heuristic lower bound into a canonical answer without that
proof.
