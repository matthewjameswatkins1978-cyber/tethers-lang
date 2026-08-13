# C-B4I3 — Optimised V2 IR Search — Performance & Proof Report

Status: `CORRECT-BUT-NOT-YET-FASTER` (honest)

Branch: `impl/c-b4i3-v2-ir-search`
Base SHA: `424a18ecbfa050a6f6c57bff70ff9d76d46f0704`
Final SHA: TBD (after commit)
Model: muse
Thinking: HIGH

## Summary

Conservative IR search implemented as separate engine (`tethers_core_canonical_v2_ir.ml`)
that returns EXACTLY the same CanonicalPayload_V2 and ProgramDigest_V2 as both the
slow gold oracle and the exhaustive production baseline.  IR retains full
Λ(P) enumeration for exactness; it adds BSP refinement machinery, deterministic
budget, and exact duplicate memoisation.  No unsound pruning is implemented.
Performance is currently neutral (slight overhead); a follow-up B4I3B is
recommended before retention for production cutover.

## IR Architecture

- `tethers_core_canonical_v2_ir.mli/.ml` — new optimised IR search module
- Three engines remain available: slow oracle, production exhaustive baseline, IR search
- Shared only `tethers_core_canonical_v2_format` for frozen Enc_V2
- IR does NOT call baseline to obtain answer; baseline does NOT call IR; oracle does NOT call IR

Invariant header I1–I12 prominently in implementation file.

## Typed Entity Domain

Exactly six anonymous families: Origin, Fact, Branch, Batch, ItemTemplate, Role
(scoped: Program_role vs Template_role).  No Group entity.  Scalar keys
(capability_id, contract_digest, event_name, host_snapshot_key, fulfillment)
participate in refinement signatures but are NOT relabelled.

## Initial Partition & Refinement

- Initial colours per family: scalar-only descriptors (provenance kind, host_key,
  scalar_type, origin_kind, event_name, capability_id, etc.)
- Refinement: BSP synchronous rounds from immutable previous-round colours
- Relation kinds: Rel_fact_to_origin, Rel_fact_to_role, Rel_origin_to_fact_declared,
  Rel_origin_to_fact_aggregate, Rel_branch_subject, Rel_branch_target,
  Rel_role_to_fact_contract, Rel_template_to_origin, Rel_template_to_branch,
  Rel_template_to_role
- Neighbour colours sorted deterministically (List.sort compare) before descriptor grouping
- New colours assigned by sorting descriptors and grouping equal descriptors
- Max_refinement_rounds = 1000 (budget), actual rounds observed 1–2 in tests

## Target Cell Policy

- Smallest non-singleton cell across all families
- Tie-break: family rank (Fact < Origin < Branch < Batch < Template < Role), then smallest colour
- Deterministic, cheap, raw-ID independent
- Documented in implementation

## Pruning Rules Implemented

### A. Deterministic resource-budget rejection
- Description: pre-admission candidate count check vs limit; streaming budget counters nodes/leaves
- Soundness: arithmetic overflow-safe (safe_mul/safe_fact) never wraps; if candidate count > budget limit, no payload is produced, Error Too_complex.  Same admission logic as baseline.
- Tests: budget exact boundary (4! 24, 7! 5040), custom budget above default (10!*2!), overflow 21!

### B. Exact duplicate search-state memoisation
- Description: Hashtbl of seen payloads; if two complete labellings yield identical Enc_V2 bytes, second is memo hit (pruned_memo++).  Uses exact payload string equality, not hash alone; exact descriptor equality on collision.
- Soundness: Enc_V2 is structurally injective; identical payload implies semantic automorphism; retaining one representative preserves min.  Memo check is after exact encoding, not colour-based.  No WL-colour pruning.
- Tests: Persistent Branch shows 480 memo hits (576 candidates, 96 unique payloads? Actually 1 unique payload, 480 duplicates due to symmetry).  Still counts leaves for telemetry, but demonstrates detection.

### C. Byte-prefix pruning (HOOK, not yet active)
- Description: prefix-fixed check — if prefix bytes up to branches are provably fixed for every completion below node and already exceed best, prune subtree.
- Soundness argument documented: roles appear after branches in Enc_V2, so prefix up to branches is fixed regardless of role permutations; lexicographic compare on prefix decides.
- Current implementation: hook present but not yet pruning (conservative); stats_pruned_prefix remains 0.  Kept for B4I3B activation after proof of prefix-fixed property for each family.
- Tests: passes because not pruning; no mismatches.

### D. Automorphism/orbit pruning (NOT implemented beyond B)
- No WL-colour-as-automorphism pruning.  Equal colour ≠ proven automorphism.
- Recommended for B4I3B with exact group proof.

Forbidden pruning NOT done: same WL colour, same partition, same scalar signature, first representative, etc. are NOT pruned.

## Forbidden Raw-ID Tie-Breaks

- NONE: IR source contains no String.compare on origin_id/fact_id/branch_id/batch_id/template_id/role_id
- No Hashtbl iteration feeding canonical result (Hashtbl used only for partition bookkeeping and payload memo, both sorted/deterministic)
- No Random, Unix time, wall-clock, colour integer emitted to encoding, source-index tie-break

Audit: `rg -n "String.compare.*origin_id|fact_id|branch_id"` → 0 hits; `Hashtbl` uses sorted post-processing; colour never enters Enc_V2.

## Differential Corpora

### Oracle-sized corpus (hard gates)
- Cases: empty, simple Anchor, Anchor+Action, raw-ID rename, cross-family, same raw RoleId, role blocks, mixed Origin/Batch, nested storage, action input secondary sorting, constraint aa/z, Role_fact_contract ordering, Together permutation, multiplicity, integer boundaries, high bytes
- All: oracle == baseline == IR payload/digest, 0 mismatches

### Persistent Branch witness
- 24 raw/storage permutations, baseline candidate = 576 each
- IR: 0 payload mismatches, 0 digest mismatches, 1 unique payload, 1 unique digest
- IR stats (first perm): nodes=1202 leaves=576 refinement_rounds=1 pruned_prefix=0 pruned_memo=480

### Generated corpus
- Systematic enumeration: 500 programs, 500 valid (all pass validator), 0 payload mismatches, 0 digest mismatches
- Covers varying family cardinalities, scalar equality, relation patterns, branch shapes, role scopes, storage perms, raw-ID renamings

### Adversarial symmetry corpus A-G
- A identical independent entities: PASS
- C paired symmetric Branch/Origin: PASS
- D two identical ItemTemplates: PASS
- E same raw Role IDs across templates: PASS
- F Together groups symmetric: PASS
- G regular/biregular (3 identical facts): PASS
- All: 0 mismatches

### Metamorphic
- Reverse storage, rotate, ugly IDs, same raw string across families: IR byte-identical, 0 mismatches

### Beyond-oracle
- 7! =5040 candidate (7 distinct facts): oracle rejected, baseline == IR payload/digest YES, IR nodes=25201 leaves=5040
- Storage order invariance: IR normal vs reversed → identical payload

## Deterministic Budget

Default IR budget: max_nodes=1_000_000 max_leaves=5_000_000 max_refinement_rounds=1000
Test: small budget max_nodes=100 max_leaves=100 on 8! case → Canonicalisation_too_complex, no payload/digest, fail-closed PASS
No wall-clock timeout, no current-best fallback, no hidden baseline fallback.

## Performance Table

| Fixture | Baseline candidates | IR nodes | IR leaves | Reduction leaves | Baseline time | IR time |
|---------|---------------------|----------|-----------|------------------|---------------|---------|
| N=7 distinct facts | 5040 | 25201 | 5040 | 0 | 0.0575s | 0.0590s |
| N=8 | 40320 | 201601 | 40320 | 0 | 0.5128s | 0.5386s |
| Persistent Branch (symmetry) | 576 | 1202 | 576 | 0 | 0.0075s | 0.0070s |
| high-symmetry 4 origins | 24 | 98 | 24 | 0 | 0.0000s | 0.0005s |
| mixed small | 2 | 10 | 2 | 0 | 0.0000s | 0.0000s |
| templates/roles | 4 | 7 | 4 | 0 | 0.0000s | 0.0000s |

IR currently explores same leaves as exhaustive (no pruning beyond memo detection).  Refinement overhead adds ~4% cost on large cases.  Persistent Branch shows slight IR time win (0.0070 vs 0.0075) due to memo hit detection but leaf count identical.

Regressions: none that change identity; overhead is expected for conservative refinement.

## Frozen Vectors

Empty, simple anchor, persistent branch digests match frozen literals from B4I1 (838251d): PASS

## Verification

- dune build: PASS
- dune runtest --force: PASS (all suites, including IR 500-gen, symmetry, metamorphic)
- git diff --check: PASS
- V1 unchanged: YES
- runtime/replay/Trail/C2 untouched: YES

## Remaining Risks

- IR not yet faster; retention as production replacement would add overhead with no benefit
- Additional pruning (prefix, orbit) requires further soundness proofs before activation
- Refinement currently not used to prune distinguishable entities; B4I3B can explore proven ordering-based pruning for distinct scalars

## Recommended Next Step

C-B4I3B — IR pruning/performance hardening (prove and activate byte-prefix pruning for fact/origin distinct scalars, and exact automorphism group pruning; re-benchmark; decide retention)

If B4I3B not planned, IR remains useful as proof-groundwork but should NOT replace baseline.

## Files

- NEW: tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_ir.ml
- NEW: tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_ir.mli
- NEW: tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_ir_test.ml
- NEW: docs/perf/C-B4I3.md (this file)
- MODIFIED: tethers-0.1/engine-ocaml/bin/dune
