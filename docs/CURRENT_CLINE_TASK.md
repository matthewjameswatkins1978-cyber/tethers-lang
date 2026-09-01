# Rocket V3 — R3-3B2 Exact Success-Path Canonisation

Control contract: `1`

Status: `COMPLETE`

Task colour: `Red`

Owner: `Codex`

Route: `Fresh dedicated worktree; derive and prove a direct exact canonicaliser for the Origin-only simple success-path case. No general forest, cross-family or production work.`

Base commit: `3034117dffa16366fa73c7befd1cccbf0bb86033`

OCaml switch path: `D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml`

Worker note: `docs/worker-notes/2026-09-01-rocket-v3-r3-3b2-success-path-canon.md`

Related issue: `#5 — BUG: Rocket V2 factorial search on simple sequential Action chains`

Updated: 2026-09-01

## Objective

Replace factorial/permutation search for the single simple success-path case with a direct exact canonicalisation algorithm derived from frozen Enc_V2.

The task must answer:

> Given a valid Origin-only program whose complete success-continuation structure is one acyclic path from entry_origin through every Origin exactly once to ProgramComplete, can the exact frozen V2-minimum Origin label assignment be constructed without enumerating Origin permutations?

Prove the theorem first, then implement it as an isolated R3-3B2 path canonicaliser and demonstrate exact equality with exhaustive authorities on tractable cases.

Do not generalise to trees/forests in this task.

## Relevant background and existing behaviour

R3-3A established exact frozen identity by complete legal label-domain enumeration. It is exact but factorial.

R3-3B introduced an incremental Origin walker but had an unsound target-forcing rule across decimal-width boundaries.

R3-3B1 repaired that exactness theorem. Exact chain-10 parity covered 362,880 residual candidates; exact chain-11 parity covered 3,628,800 residual candidates and eliminated the historical byte-23 mismatch.

However R3-3B1 correctly stopped on deterministic performance: chain-100 reached 27,000 branches and 195,471,123 emitted bytes before the authorised stop. Chain-1000 was not attempted.

The validated success-continuation relation has at most one continuation per from_origin and rejects success cycles. This task restricts further to one complete path:

`entry -> Origin -> ... -> Origin -> ProgramComplete`

Every program Origin must appear exactly once on that path.

Relevant frozen Enc_V2 order is:

`entry_origin -> success_continuations -> origin_sites -> later fields`

Therefore entry_origin dominates the continuation block, and the complete continuation block dominates all later Origin-site bytes.

Continuation elements are sorted by numeric from_origin label, while labels are emitted by `encode_int n = decimal(n) ^ ";"` and compared by unsigned-byte lexicographic order. Numeric and byte order therefore diverge at decimal-width boundaries.

For a simple path of N Origins, a complete legal labelling induces a numeric successor table over slots 1..N. Legal semantic path labellings correspond to legal rooted Hamiltonian successor paths over those numeric slots once the entry label is fixed. The task must prove and exploit that reduction instead of permuting semantic Origin objects.

## Required behaviour

1. Start from exact base `3034117dffa16366fa73c7befd1cccbf0bb86033`.

2. Preserve all R3-3A/B/B1 evidence and the B1 BLOCKED result.

3. Add an explicit supported-shape predicate for valid Origin-only single-path programs.

4. Supported shape requires entry_origin, every program Origin reachable exactly once from entry, final Origin targeting ProgramComplete, no disconnected Origins and no missing continuation on the path.

5. Unsupported/non-path structures must return a deterministic experimental unsupported result; do not silently invoke the old factorial walker.

6. Prove the bijection between semantic path Origin labellings and legal numeric-label successor tables rooted at the assigned entry label.

7. Preserve the frozen exact entry rule: entry Origin gets the legal label whose exact encode_int bytes are lexicographically minimal.

8. Minimise the complete success-continuation block in the exact numeric source-slot order used by frozen Enc_V2.

9. Do not use raw IDs, source storage order, internal vertex numbers or R3-2 cell numbers to choose canonical labels.

10. If using greedy construction, every committed choice must be justified by an exact legal-completion feasibility proof.

11. Implement an exact feasibility predicate for partial successor tables.

12. Feasibility must reject duplicate predecessor assignment.

13. Feasibility must reject multiple successors from one source.

14. Feasibility must reject a predecessor into the fixed entry slot.

15. Feasibility must reject more than one ProgramComplete terminal.

16. Feasibility must reject premature directed cycles.

17. Feasibility must reject partial states that cannot still be completed into one Hamiltonian path.

18. Feasibility must accept every partial state that has at least one legal complete success-path completion.

19. Do not enumerate complete Origin permutations in the new path canonicaliser.

20. Do not call R3-3A or R3-3B1 search implementation from the new path implementation; they remain test authorities only.

21. Produce the final Origin label assignment by mapping the winning numeric successor path back onto the semantic Origin path.

22. Feed the final assignment through existing frozen Enc_V2 machinery for final payload/digest; do not create a new identity format.

23. Prove byte-for-byte parity against exhaustive authority for homogeneous chains 1 through 11.

24. Chain-10 must match the previously proven 362,880-residual-candidate minimum.

25. Chain-11 must match the previously proven 3,628,800-residual-candidate minimum and known exact label sequence `[10,9,8,7,6,5,4,3,2,1,11]`.

26. Add structural decimal-width cases crossing 9/10, 10/11, 99/100 and 999/1000.

27. Add path fixtures with non-identical Origin body bytes, including distinct Action capabilities/contracts and Anchor/Action mixtures where supported.

28. Prove that once entry plus the complete continuation block uniquely determine the label assignment, later Origin-site bytes cannot overturn the winner.

29. Add raw-ID renaming and storage-order permutation metamorphic variants.

30. Add at least three deterministic implementation traversal/choice-order perturbations; all must produce identical labels/payload/digest.

31. Instrument path_size, successor_slots_processed, candidate_targets_considered, feasibility_checks, rejected_infeasible_choices, committed_choices, complete_permutations_enumerated and max_partial_components.

32. Require `complete_permutations_enumerated = 0`, then after exact 1–11 parity run chain-12, chain-100, chain-1000, and chain-5000 only if still comfortably bounded. Do not generalise beyond simple paths.

## Relevant components

Authorised mutation is limited to:

- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-09-01-rocket-v3-r3-3b2-success-path-canon.md`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_success_path.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_success_path.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_success_path_test.ml`
- `tethers-0.1/engine-ocaml/bin/dune`

Read-only authorities:

- `tethers_core_canonical_v2_format.ml/.mli`
- `tethers_core_canonical_v2_reference.ml/.mli`
- `tethers_core_rocket_v3_encode.ml/.mli`
- `tethers_core_rocket_v3_origin_walk.ml/.mli`
- R3-1 model
- R3-2 partition/refinement
- validator/Core/planner
- R3-3A/B/B1 worker notes/tests

If exact implementation requires changing a read-only authority, STOP.

## Frozen decisions and invariants

- Frozen Enc_V2 remains unchanged.
- ProgramDigest V2 remains unchanged.
- Numeric continuation sorting remains unchanged.
- encode_int byte representation remains unchanged.
- Entry field outranks the entire continuation block.
- The complete continuation block outranks all Origin-site/later bytes.
- A smaller earlier frozen block cannot be rescued by a later block.
- Semantic path order is structure, not canonical numeric label order.
- Raw IDs remain non-semantic.
- No arbitrary Action-count limit.
- No search budget may alter identity.
- No V1 fallback.
- No heuristic pruning.
- Numeric order must never be confused with encoded-byte order.
- This task proves only the simple-path case.

## Acceptance criteria

1. Work starts from exact base `3034117dffa16366fa73c7befd1cccbf0bb86033`.

2. All prior R3-3 evidence remains preserved.

3. Supported-shape detection accepts intended complete single paths.

4. Supported-shape detection rejects disconnected/incomplete/non-path structures deterministically.

5. Unsupported shapes do not silently invoke factorial search.

6. Semantic path labellings to numeric successor tables are documented/tested as a bijection.

7. Entry label is selected solely by frozen exact byte law.

8. Continuation minimisation operates in numeric source-slot serialization order.

9. No raw-ID/storage/internal-vertex/R3-2-cell ordering influences labels.

10. Every committed choice has an exact completion-feasibility justification.

11. A standalone exact feasibility predicate exists.

12. Duplicate-predecessor states are rejected.

13. Multiple-successor states are rejected.

14. Predecessor-to-entry states are rejected.

15. Invalid/multiple-terminal states are rejected.

16. Premature cycles are rejected.

17. Uncompletable disconnected partial states are rejected.

18. Known completable partial states are accepted.

19. New canonicaliser enumerates zero complete Origin permutations.

20. R3-3A/B1 are test authorities only, not implementation dependencies.

21. Winning successor table maps back to one complete legal semantic Origin label assignment.

22. Final payload/digest use existing frozen Enc_V2 machinery.

23. Chains 1–9 match exhaustive frozen authority exactly.

24. Chain-10 matches the exact 362,880-residual-candidate result.

25. Chain-11 matches the exact 3,628,800-residual-candidate result and known label sequence.

26. Decimal width crossings 9/10, 10/11, 99/100 and 999/1000 are exercised.

27. Distinct-body Origin path fixtures retain exact identity.

28. Later origin_sites bytes are proven unable to overturn a uniquely minimal earlier continuation block.

29. Raw-ID and storage-order metamorphic variants remain byte/digest identical.

30. Three deterministic traversal/choice perturbations return identical results.

31. Required deterministic statistics are present and repeatable.

32. `complete_permutations_enumerated = 0`; chain-12, 100 and 1000 complete without factorial search or the task STOPS with an exact mathematical/performance finding before any generalisation.

## Required verification

- Use a fresh dedicated worktree tracking `origin/feature/rocket-v3-r3-3b2-success-path-canon`.
- Confirm exact base `3034117dffa16366fa73c7befd1cccbf0bb86033`, branch and clean worktree.
- Run `pwsh -NoProfile -File scripts/check-dev-tools.ps1`.
- Run packet checker and require `control-v1/READY`.
- Verify the exact authorised OCaml switch.
- Read frozen encode_program, validator success-continuation invariants, R3-3B1 implementation/worker note and chain-10/11 exact tests before mutation.
- Write down the semantic-path/numeric-successor-table proof and exact partial completion predicate before large-chain claims.
- Test feasibility independently.
- Differentially prove chains 1–9, exact chain-10 and exact chain-11.
- Exercise decimal boundaries, distinct body shapes, raw-ID/storage metamorphics and traversal-order perturbations.
- Only after exact parity run chain-12, chain-100 and chain-1000; optionally chain-5000 if comfortably bounded.
- Record all required deterministic statistics.
- Run the focused path suite.
- Run R3-3A `39/39`.
- Run R3-1 `214/214`.
- Run R3-2 `4807/4807`.
- Run V2 suites and 5,000-case corpus.
- Run `opam exec --switch="D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml" -- dune build @all`.
- Run `opam exec --switch="D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml" -- dune runtest --force`.
- Run `git diff --check`.
- Inspect full base-to-HEAD diff and prove authorised paths only.
- Commit implementation/tests and record full implementation checkpoint SHA.
- Write worker note and transition packet to `COMPLETE`; no implementation mutation after checkpoint.
- Run packet checker requiring `control-v1/COMPLETE`.
- Push normally, prove local HEAD == remote HEAD, require clean worktree, report evidence and STOP.

## Forbidden changes

- No frozen V2 changes.
- No ProgramDigest change.
- No Core/validator changes.
- No R3-1/R3-2 changes.
- No R3-3A/B/B1 modification.
- No full permutation search in the new path canonicaliser.
- No heuristic-only greedy rule.
- No unproved feasibility shortcut.
- No raw-ID/internal-vertex ordering.
- No automorphism/orbit machinery.
- No generic graph I/R.
- No success-tree/forest generalisation.
- No Facts/Branches/Batches/Templates/Roles work.
- No production integration.
- No new dependency.
- No wall-clock identity decision.
- No V1 fallback.
- No R3-3C/R3-4/release work.

## Stop conditions

- Semantic-path to numeric-successor-table bijection is false.
- An exact bounded completion-feasibility predicate cannot be established.
- Any chain 1–11 result disagrees with exhaustive frozen authority.
- Chain-11 known exact sequence is not reproduced.
- Correctness depends on later Origin-site bytes before the continuation minimum is determined.
- Path canonicaliser requires complete factorial permutation enumeration.
- Traversal-order perturbations change identity.
- Decimal-width crossings expose an unmodelled frozen-order dependency.
- Correctness requires frozen V2/Core/R3-2 changes.
- Two materially similar approaches fail without a new diagnosis.

A Red mathematical finding is a valid result. Do not weaken the theorem for speed.

## Expected pre-existing changes

None.
