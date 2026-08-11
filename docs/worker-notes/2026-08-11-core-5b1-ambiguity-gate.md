# Worker Note

Task: `TETHERS CORE-5B1 — Capability Projection Ambiguity Gate`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `d7c05c52daaac44e3169bd43c3c2e91f3ecfad04`

Implementation checkpoint: `12f266c106942e9f7bd4a76d18623df27ee893d1`

## Requested outcome

Remove storage-order-dependent selection of duplicate runtime Capability
projections.  When more than one projection matches an exact
`(CapabilityId, ContractDigest)` pair, fail closed with a new typed error
instead of silently picking the first list entry.

## Changes made

- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.ml` — added
  `Ambiguous_capability_projection of capability_id` to `planning_error`;
  changed `projection_of` from `List.find_opt` (first match) to
  `List.filter` + length check (fail closed on >1 exact match)
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.mli` — added documented
  `Ambiguous_capability_projection` error variant
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan_test.ml` — added
  `Ambiguous_capability_projection` to `string_of_planning_error`; added
  tests B1-T1 (duplicate exact projection fails), B1-T2 (reversed duplicates
  fail identically), B1-T3 (distinct contracts for one CapabilityId remain
  selectable)

## Decisions and assumptions

1. **Fail closed on ambiguity.** Two projections with identical
   `(CapabilityId, ContractDigest)` but different runtime metadata produce
   `Ambiguous_capability_projection`.  The host must deduplicate projections
   before supply.

2. **Distinct contracts are fine.** `(A, Digest1)` and `(A, Digest2)` can
   coexist; only exact pair duplicates are rejected.

3. **No sorting.** The gate does not sort or deduplicate projections.  It
   detects multiple exact matches and rejects them.

## Evidence

All commands ran against implementation checkpoint
`12f266c106942e9f7bd4a76d18623df27ee893d1`.

| Command | Result |
| --- | --- |
| `dune build @all` | PASS (exit 0) |
| `dune runtest` | PASS — plan bridge 48/48 |
| `git diff --check` | PASS (LF/CRLF normalisation warnings only) |
| `RUST_UNCHANGED` | Yes |

**New tests:** 3 test functions, 7 assertions:

- B1-T1 `test_duplicate_projection_fails` — two projections with identical
  `(CapabilityId, ContractDigest)` fail with `Ambiguous_capability_projection`
- B1-T2 `test_reversed_duplicates_fail` — reversed projection list produces
  the same typed error, proving storage order cannot choose meaning
- B1-T3 `test_distinct_contracts_coexist` — `(A, Digest1)` and `(A, Digest2)`
  coexist; exact Core-requested pair resolves correctly

**Commands not run:**

- Fixture suite, MCP transcript suite, Rust host tests: NOT RUN — CORE-5B1 is
  a dormant sidecar bridge change; no evaluator, protocol, MCP, or Rust code
  changed.

## Publication evidence

Branch `feature/core-5-runtime-plan-bridge` pushed normally to `origin`.
Remote HEAD resolved and confirmed equal to local HEAD; `git status --short
--branch` clean.

## Remaining risks

- None within packet scope.  The bridge remains dormant.

## References

- Branch: `feature/core-5-runtime-plan-bridge`
- Base: `d7c05c52daaac44e3169bd43c3c2e91f3ecfad04` (CORE-5B closeout HEAD)
- Implementation checkpoint: `12f266c106942e9f7bd4a76d18623df27ee893d1`
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan_test.ml`
