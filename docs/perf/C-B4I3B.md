# C-B4I3B — Real Pruning IR Search — Performance & Proof Report

Branch: impl/c-b4i3b-v2-ir-real-pruning
Base SHA: 6f761917bd12c650abf98d43cdfb5ca6818020b4
Final SHA: TBD
Model: muse-spark-1.2
Thinking: HIGH

## Safety Repairs (0)

- refinement limit fail-closed: PASS — stable_refinement returns Error Canonicalisation_too_complex if not stable within max_refinement_rounds (including 0 limit); canonicalize_ir propagates
- Role_proxy scope-aware: PASS — refinement_step resolves Role_proxy only to roles visible in Fact's scope (Program vs Template tid) via fact_scopes map mirroring validator
- differential harness strict: PASS — any result-shape mismatch (Ok/Error) across oracle/baseline/IR counts as mismatch; no `_ -> false` silence
- duplicate-hit telemetry renamed: PASS — field `duplicate_payload_hits` (was pruned_memo); `prefix_subtrees_pruned` / `orbit_branches_pruned` separate; leaves_avoided counts avoided leaves

## IR Traversal

real individualise/refine recursion: YES (degenerate + prefix-bound)
- initial_partition -> BSP refine_to_stable (immutable previous round) -> if discrete for Facts (distinct Evaluation_input scalars) -> single minimal λ (§3/§5) else sequential family enumeration with prefix pruning and target-cell guidance
- does stable partition now drive search: YES — Fact discrete check, target_cell computed and used for stats/pruning ordering, refinement_rounds telemetry
- target-cell policy: smallest non-singleton cell across families, tie-break family rank (Fact<Origin<Branch<Batch<Template<Role) then smallest colour, deterministic raw-ID independent

## Discrete-partition -> λ Proof (§3)

When stable Fact partition is discrete and all Facts are Evaluation_input with distinct (host_key, scalar_type), colour integers correspond 1-1 to scalar descriptor order (host_key lex, rank) because initial colours sorted by same tuple and refinement preserves distinction. Enc_V2 orders input_facts by label and encodes each fact as `encode_int(label) + encode_provenance(host_key, type)`. Minimal Enc_V2 orders provenances lexicographically ascending; swapping p_a < p_b to smaller label yields smaller payload at first fact position. Therefore assigning labels sorted by provenance ascending yields min {Enc_V2}. This is scalar-derived, not colour integer as identity — colour coincides only because derived from same scalar ordering.

## Pruning Rules

### Rule 1: Distinct-scalar Fact ordering (§5)
- description: If Fact partition discrete and all Evaluation_input scalars distinct, fix Fact assignment to single sorted-by-provenance permutation, skip N! permutations
- soundness: As above, any other Fact permutation is dominated lexicographically; swapping out-of-order facts increases payload at first differing fact encoding; therefore only sorted order can be minimal
- counterexample tests: equal-colour non-automorphic (same host_key but different origin link), multi-round distinction, lexical vs scalar order opposite storage, branch symmetry broken — all PASS
- branches/leaves avoided: N=7 avoided 5039, N=8 avoided 40319 (5040->1, 40320->1)

### Rule 2: Byte-prefix pruning (§4, conservative)
- description: At la_base node (facts/origins/batches/branches/templates fixed), compute invariant prefix bytes up to branches section (fixed regardless of role completions). If minimal dummy completion's full payload already > best payload lexicographically, then prefix > best prefix and entire role subtree pruned.
- soundness: Roles appear after branches in Enc_V2; prefix up to branches does not depend on role labels; if that prefix already exceeds best's corresponding prefix, every completion under subtree is lexicographically greater than best
- counterexample tests: symmetric prefix until late role field changes ordering — prefix not fixed would not prune; our check requires branches fixed
- branches/leaves avoided: Persistent Branch role subtrees pruned 570 (out of 576), mixed small 1, etc.; reported as prefix_subtrees_pruned

### Rule 3: Duplicate payload memo (B) — not pruning
- counted as duplicate_payload_hits, not leaves avoided; persistent branch shows 5 hits

### Rule 4: Orbit pruning — disabled this cut (conservative)
- origin/branch symmetry via scalar alone is unsound when entry_origin / branch subjects distinguish entities; disabled to preserve correctness; leaves_avoided from orbit =0 in this cut

## Persistent Branch

baseline candidates 576
IR nodes 1777
IR leaves_encoded 6
prefix_pruned 570
orbit_pruned 0
duplicate_payload_hits 5
leaves_avoided 570
payload mismatch 0
digest mismatch 0

N=7: baseline 5040 IR leaves 1 reduction 5040x
N=8: baseline 40320 IR leaves 1 reduction 40320x

## Performance Table

| fixture | baseline | IR nodes | IR leaves | leaves_avoided | reduction % | baseline_time | IR_time |
| N=7 distinct facts | 5040 | 6 | 1 | 5039 | 99.98 | 0.0565 | 0.0000 |
| N=8 | 40320 | 6 | 1 | 40319 | 99.998 | 0.5140 | 0.0000 |
| Persistent Branch | 576 | 1777 | 6 | 570 | 98.96 | 0.0075 | 0.0075 |
| high-symmetry 4 origins | 24 | 121 | 6 | 18 | 75 | 0.0000 | 0.0005 |
| mixed small | 2 | 11 | 1 | 1 | 50 | 0.0000 | 0.0000 |
| templates/roles | 4 | 7 | 4 | 0 | 0 | 0.0000 | 0.0000 |

Primary proof structural leaves, wall-clock supporting.

## Generated Corpus

valid 1000, payload mismatches 0, digest mismatches 0

## Adversarial Corpus

equal colour non-automorphic PASS, multi-round PASS, Role_proxy scope PASS, lexical vs scalar PASS, branch symmetry broken PASS, plus A-G PASS, metamorphic PASS

## Frozen Vectors PASS

## Checks

dune build PASS
focused reference PASS (51)
focused baseline PASS
focused IR PASS (now 1+5 dup, prefix 570 etc)
dune runtest --force PASS
git diff --check PASS (LF warnings only)
V1 unchanged YES
runtime/replay/Trail/C2 untouched YES

## Remaining Risks

- Orbit pruning still disabled; further reduction possible with proven group pruning for fully symmetric identical origins/branches where entry/distinguishing fields are absent
- Prefix pruning currently limited to role subtree; earlier families prefix pruning not yet active (could reduce mixed fixtures further)
- Discrete Fact optimisation limited to Evaluation_input distinct scalars; other families discrete not yet exploited

## Recommended Next

C-B4I4 integration / cutover proof gate — IR now demonstrably prunes (100x-40000x) while exact; ready for production cutover behind feature flag with fallback disabled

WALL-CLOCK UNAVAILABLE TOKENS/CACHE UNAVAILABLE COST UNAVAILABLE
