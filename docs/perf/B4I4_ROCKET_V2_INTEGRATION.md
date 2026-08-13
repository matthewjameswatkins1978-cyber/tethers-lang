# B4I4 — Rocket V2 Integration Report

Status: `INTEGRATION COMPLETE; READY FOR REVIEW`

Branch: `mimo/b4i4-rocket-v2-integration`
Base: `codex/c-b4i3c-canonical-v2-search` at `a1d9c3b6ad5cfbb45732f50efcca3231b21ecb4d`
Rocket candidate: `92443ac0420e377154adcf3e12c259b729d394fe`
Integration commit: `4235045cd542cbc65dad092d9bc8c4da7768c95d`

## 1. What Moved Into Production

Two files changed, byte-identical to the Rocket candidate:

| File | Change |
| --- | --- |
| `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_ir.ml` | +537 lines: 9 exact reductions, fixed-label permutation state, reduced pre-admission |
| `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_ir_test.ml` | +1139 lines: dense 5000-case corpus, tie torture, decimal boundaries, compound collapses, mixed Branch torture, performance GC instrumentation |

No other production file changed.

## 2. What Experimental Machinery Stayed Out

Three experimental research docs were excluded:

| File | Classification | Reason |
| --- | --- | --- |
| `docs/perf/C_B4I3R_ROCKET_HYBRID_TORTURE_LAB.md` | RESEARCH-ONLY | Experimental analysis; not production code |
| `docs/perf/C_B4I3R2_ROCKET_FULL_BURN_QUALIFICATION.md` | RESEARCH-ONLY | Full-burn qualification evidence; not production code |
| `docs/perf/C_B4I3R2A_ROCKET_ANCHOR_TIE_REPAIR.md` | RESEARCH-ONLY | Anchor tie repair analysis; not production code |

No telemetry, logging, debug scaffolding, or research-only code paths were present in the Rocket candidate's production files. The only non-test code is the reduction logic itself.

## 3. Why Each Accepted Reduction Is Still Sound

### 3.1 Top-level distinct Evaluation_input Fact ordering (reduction 1)

**Condition:** The entire Fact inventory is exactly the set of top-level `Evaluation_input` occurrences.

**Soundness:** The first Fact-sensitive Enc_V2 section emits facts in label order. Each fact's provenance bytes are injective. Sorting by exact encoded provenance is an adjacent-inversion proof over the first differing byte.

**Preserved:** Yes. `fact_discrete_minimal_order` unchanged.

### 3.2 entry_origin minimal label (reduction 2)

**Condition:** `entry_origin` exists and no input Fact has `Origin_provenance`.

**Soundness:** `entry_origin` is emitted after core version, input Facts, and entry guards. Valid input Facts are `Evaluation_input` only, so earlier bytes are independent of every Origin label. The entry Origin must receive the available label whose exact `encode_int` bytes are smallest.

**Preserved:** Yes. `entry_origin_minimal_label` with `fix_label_ir` applied before search.

### 3.3 Dependency-closed program Anchor body-class ordering (reduction 3)

**Condition:** `entry_origin` exists, no success continuations, no template origin_sites, all origin_sites are Anchor_origin, all declared_facts are Evaluation_input.

**Soundness:** After Fact labels and entry_origin's label are fixed, an Anchor's suffix after its own Origin label has no Origin or Role dependency. Distinct exact body classes are ordered; equal-body classes remain exhaustively permuted.

**Preserved:** Yes. `program_anchor_origins_are_dependency_closed` guards the reduction. `enumerate_program_anchor_origin_orders` sorts distinct bodies and enumerates tied bodies.

### 3.4 Exhaustive residual permutations inside equal Anchor-body classes (reduction 4)

**Condition:** Same as 3.3, plus body-class sizes computed.

**Soundness:** Equal Anchor bodies cannot be distinguished by any later Enc_V2 field that sees only their suffix. The objective can still distinguish them by the label itself. Therefore tied bodies remain live for exhaustive permutation.

**Preserved:** Yes. `free_program_anchor_body_class_sizes` computes class sizes. `program_anchor_residual_permutations` computes the product of `n!` per class.

### 3.5 One-physical-Branch-collection reduction (reduction 5)

**Condition:** All Branch occurrences live in exactly one physical collection (program branches or one template's branches, not both).

**Soundness:** Branch IDs are not referenced by any V2 field. When all Branches are in one collection, every candidate emits the fixed numeric label sequence 1..N and differs only in the suffix after each label. Sorting those exact suffixes is an adjacent-inversion proof.

**Preserved:** Yes. `all_branches_in_one_collection` guards the reduction. `assign_single_collection_branch_order` sorts and assigns.

### 3.6 Program Role body ordering with guard (reduction 6)

**Condition:** No earlier program origin_site uses a `Role_proxy` or `Fact_through_role` binding.

**Soundness:** Program Roles are emitted after every program Origin site. If no site references a Role label, the Role collection is the first role-sensitive field. Exact encoded-body ordering is sound.

**Preserved:** Yes. `program_roles_unreferenced_before_own_collection` guards. `program_role_body_order` sorts and assigns via `fix_label_ir`.

### 3.7 Template Role distinct-body reduction with guard (reduction 7)

**Condition:** Template roles are unreferenced before their own collection AND have pairwise-distinct exact fulfilment strings.

**Soundness:** A template's role list can be locally ordered only when role labels have no earlier occurrence in that template AND the later objective cannot observe tied bodies. Different fulfilment strings yield different length-prefixed suffixes irrespective of all free labels.

**Preserved:** Yes. `template_roles_unreferenced_before_own_collection` and `template_roles_have_distinct_fulfillments` guard. `template_role_body_order_if_distinct` returns `None` for tied bodies, falling back to exhaustive.

### 3.8 Synchronized reduced pre-admission accounting (reduction 8)

**Condition:** All reductions above applied consistently.

**Soundness:** `reduced_candidate_count_within_budget_ir` computes the exact leaf budget after all proven reductions. This is the deterministic budget gate, not the raw Lambda(P) count.

**Preserved:** Yes. `raw_candidate_count` computed with `limit:max_int` for telemetry. `reduced_candidate_count_within_budget_ir` with `budget.max_leaves` for the production gate.

### 3.9 Exhaustive fallback everywhere else

**Condition:** Default path.

**Soundness:** When no reduction applies, the full permutation enumeration runs. Equal colours are never an automorphism certificate. The exhaustive baseline remains independently callable.

**Preserved:** Yes. Every `else` branch falls through to `assign_next_ir`.

## 4. Exact Residual Fallback Behaviour

| Family | When fallback activates |
| --- | --- |
| Facts | Not all top-level Evaluation_input, or duplicate provenance bytes |
| Origins | entry_origin absent, or Origin_provenance input Fact, or non-Anchor sites, or success continuations, or template origin_sites |
| Branches | Multiple physical collections (program + template branches coexist) |
| Program Roles | Earlier site uses Role_proxy or Fact_through_role |
| Template Roles | Earlier site uses role reference, or tied fulfilment strings |
| Batches | Always (no batch reduction proposed) |
| Templates | Always (no template reduction proposed) |

## 5. Frozen Vector Result

**PASS.** All frozen payload/digest vectors remain byte-identical:

- `known frozen vectors` — PASS
- `frozen simple anchor vector` — PASS
- `frozen persistent branch vector` — PASS

## 6. Differential Result

**PASS.** Dense 5000-case corpus:

```
Dense generated corpus: seed=308386 total=5000 valid=5000 mismatches=0 archetypes=16
```

oracle == baseline == production-Rocket for all 5000 cases, every payload and digest.

## 7. Fail-Closed Result

**PASS.**

| Gate | Result |
| --- | --- |
| `max_leaves` budget rejection | PASS (8 template roles at budget=100) |
| `max_nodes` budget rejection | Covered by budget fail-closed test |
| `max_refinement_rounds` | Covered by refinement fail-closed test |
| Factorial overflow (21!) | PASS |
| No digest on failure | PASS (Error case returns no payload/digest) |
| 11-Branch reduced pre-admission | PASS (39,916,800 raw → 1 leaf, 39,916,799 avoided) |

## 8. Performance Result

### Baseline vs Rocket (production-integrated)

| Case | Raw candidates | Baseline time | IR leaves | IR time/call | Leaves avoided |
| --- | --- | --- | --- | --- | --- |
| N=7 distinct facts | 5,040 | 0.057s | 1 | 0.000034s | 5,039 |
| N=8 distinct facts | 40,320 | 0.546s | 1 | 0.000038s | 40,319 |
| Persistent Branch | 576 | 0.007s | 6 | 0.000126s | 570 |
| High-symmetry 4 origins | 24 | ~0s | 6 | 0.000061s | 18 |
| High-symmetry 8 branches | 40,320 | 0.610s | 1 | 0.000036s | 40,319 |
| Mixed small | 2 | ~0s | 1 | 0.000021s | 1 |
| Templates/roles | 4 | ~0s | 4 | 0.000066s | 0 |

### Key observations

- **N=8 facts:** Baseline takes 0.546s. Rocket takes 0.000038s per call. Speedup: ~14,000x.
- **8 branches (single collection):** Baseline takes 0.610s. Rocket takes 0.000036s per call. Speedup: ~17,000x.
- **Persistent Branch:** 576→6 leaves. Rocket is faster per call despite encoding 6 leaves.
- **No allocation regressions observed.** GC pressure is minimal per call.

### Cases still hitting factorial fallback

- Templates/roles (4 candidates): Template roles with tied fulfilment strings fall through to exhaustive. This is correct — the later objective can distinguish tied bodies.

## 9. Remaining Worst Factorial Families

| Family | Worst case | Current behaviour |
| --- | --- | --- |
| 8 distinct Program Roles | 8! = 40,320 | Reduced to 1 when no earlier role reference |
| 8 distinct Template Roles (tied fulfilment) | 8! = 40,320 | Falls through to exhaustive (correct) |
| 8 distinct Facts (not top-level Evaluation_input) | 8! = 40,320 | Falls through to exhaustive (correct) |
| 12 Anchor origins (non-dependency-closed) | 12! = 479,001,600 | Budget rejection at default |

## 10. Research Code Classification

| Machinery | Classification | Ships? |
| --- | --- | --- |
| `fixed` field on `perm_state_ir` | SHIP | Yes — needed for all label-fixing reductions |
| `fix_label_ir` | SHIP | Yes — deterministic label assignment |
| `entry_origin_minimal_label` | SHIP | Yes — reduction 2 |
| `program_anchor_origins_are_dependency_closed` | SHIP | Yes — reduction 3 guard |
| `free_program_anchor_body_class_sizes` | SHIP | Yes — reduction 4 class sizes |
| `program_anchor_residual_permutations` | SHIP | Yes — reduction 4 residual count |
| `all_branches_in_one_collection` | SHIP | Yes — reduction 5 guard |
| `assign_single_collection_branch_order` | SHIP | Yes — reduction 5 implementation |
| `program_roles_unreferenced_before_own_collection` | SHIP | Yes — reduction 6 guard |
| `program_role_body_order` | SHIP | Yes — reduction 6 implementation |
| `template_roles_unreferenced_before_own_collection` | SHIP | Yes — reduction 7 guard |
| `template_roles_have_distinct_fulfillments` | SHIP | Yes — reduction 7 body guard |
| `template_role_body_order_if_distinct` | SHIP | Yes — reduction 7 implementation |
| `enumerate_program_anchor_origin_orders` | SHIP | Yes — reduction 3/4 implementation |
| `reduced_candidate_count_within_budget_ir` | SHIP | Yes — reduction 8 pre-admission |
| `raw_candidate_count` (telemetry) | SHIP | Yes — leaves_avoided computation |
| `stats_leaves_avoided` (removed) | N/A | Was replaced by `raw_candidate_count` computation |
| Dense 5000-case generator | SHIP | Yes — production test gate |
| GC instrumentation in bench | SHIP | Yes — performance evidence |
| `test_single_collection_branch_shortcut` | SHIP | Yes — regression test |
| `test_dependency_closed_program_anchor_origins` | SHIP | Yes — regression test |
| `test_anchor_tie_repair_minimal` | SHIP | Yes — correctness test |
| `test_anchor_tie_torture` | SHIP | Yes — adversarial test |
| `test_branch_label_count_boundaries` | SHIP | Yes — 8/9/10/11/12/19/20/21 boundary test |
| `test_decimal_label_family_boundaries` | SHIP | Yes — 9/10/11 decimal boundary test |
| `test_compound_factor_collapses` | SHIP | Yes — multi-factor collapse test |
| `test_mixed_branch_torture` | SHIP | Yes — mixed adversarial test |

No research-only or test-only code was found in the production files. All code is either reduction logic or production test evidence.

## 11. Remaining Risks

1. **No new optimisations introduced.** The Rocket candidate contains only the9 accepted reductions. No speculative graph/IR/orbit/B&B machinery.
2. **Tied Template Role bodies remain exhaustive.** This is correct — the later objective can distinguish equal fulfilment strings by label. No reduction is proposed.
3. **Batches have no reduction.** This is correct — no batch reduction has been proposed or accepted.
4. **Multiple physical Branch collections remain exhaustive.** This is correct — shared global label space couples earlier and later sections.

## 12. Production Cutover Recommendation

**YES.**

This is a boring production implementation of the already-approved Rocket mathematics. Every frozen vector is byte-identical. Every adversarial test passes. The5000-case dense differential shows oracle == baseline == production-Rocket. Fail-closed gates are verified. Performance is dramatically improved for the families Rocket reduces, with no regression for families it does not.

The diff is minimal: exactly2 code files, exactly the9 accepted reductions, no experimental scaffolding.

## 13. Final State

| Item | Value |
| --- | --- |
| Integration branch | `mimo/b4i4-rocket-v2-integration` |
| Base SHA | `a1d9c3b6ad5cfbb45732f50efcca3231b21ecb4d` |
| Final SHA | `4235045cd542cbc65dad092d9bc8c4da7768c95d` |
| Changed files | 2 |
| `cargo fmt --check` | PASS |
| `cargo check` | PASS |
| `cargo test` | PASS (1451 passed, 0 failed) |
| `dune build @all` | PASS |
| `dune runtest --force` | PASS (all OCaml tests green) |
| `git diff --check` | PASS |
| Frozen vectors | PASS (byte-identical) |
| Dense differential | PASS (5000/5000) |
| Fail-closed | PASS |
| Performance | Improved (see table) |
