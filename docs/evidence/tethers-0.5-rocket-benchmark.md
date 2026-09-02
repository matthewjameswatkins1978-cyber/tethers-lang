# Tethers 0.5 Rocket evidence

The measurements below are descriptive runtime evidence. They are not part of
canonical bytes, digests, parent vectors, or semantic ordering.

## Portfolio differential corpus

The focused portfolio executable reported `41/41` checks passed. The bounded
corpus included path sizes 1–5, raw-ID-renamed paths, reversed storage order,
the empty case, explicit reference mode, and a zero optimisation budget. It
reported zero payload mismatches and zero payload-derived digest mismatches.

The zero-budget case selected the exhaustive reference fallback and preserved
the same payload as the normal exact route. An optimisation budget therefore
changes work performed and the reported backend only; it does not change
identity.

The existing V2 differential suite also reported a deterministic generated
corpus of `5000` valid programs with `0` mismatches across `16` archetypes,
plus the existing adversarial symmetry and metamorphic cases with no failures.

## Common path

The Rocket success-path chain benchmark for `1000` actions reported:

```text
path_size=1000
successor_slots=1000
candidate_targets=1003
feasibility_checks=1003
rejected_infeasible_choices=3
committed_choices=1000
complete_permutations_enumerated=0
max_partial_components=1000
```

This is the intended common-case profile: exact path construction without
enumerating complete permutations.

## Hard and symmetry-heavy cases

The full V2 IR evidence remained green. Representative output included:

```text
Persistent Branch: raw_candidates=576 IR_nodes=31 IR_leaves=6
  dup_hits=5 leaves_avoided=570
N=8: raw_candidates=40320 IR_nodes=6 IR_leaves=1
  leaves_avoided=40319
high-symmetry 8 branches: raw_candidates=40320 IR_nodes=6 IR_leaves=1
  leaves_avoided=40319
```

The permanent exhaustive oracle remains the bounded correctness reference.
Research-only R3-3B3A/B3B/B3C claims are not used as production theorems.
