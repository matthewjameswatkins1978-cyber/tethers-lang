# Rocket V3 — R3-3B1 Next-Observable-Byte Correctness Repair

Control contract: `1`

Status: `READY`

Task colour: `Red`

Owner: `Codex`

Route: `Fresh dedicated worktree; repair and re-prove the Origin-only Enc_V2 canonical-augmentation crucible. No cross-family generalisation or production integration.`

Base commit: `c3d136dc4217059d4434f8d39a273fa398c4e64d`

OCaml switch path: `D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml`

Worker note: `docs/worker-notes/2026-09-01-rocket-v3-r3-3b1-next-observable-byte.md`

Related issue: `#5 — BUG: Rocket V2 factorial search on simple sequential Action chains`

Updated: 2026-09-01

## Objective

Repair the R3-3B Origin-only canonical-augmentation walker so that a label is forced only when its encoded value is genuinely the next unresolved label-sensitive bytes in frozen Enc_V2.

The current R3-3B forcing rule is unsound across decimal label-width boundaries because it can force a continuation target while lower numeric source slots remain unresolved.

Prove the corrected rule against exact exhaustive references before making any further 100/1000-Action scaling claim.

Do not begin R3-3C.

## Relevant background and existing behaviour

R3-3B introduced an Origin-only incremental walker covering:

`entry_origin -> success_continuations -> origin_sites`

It models the frozen dual-order law:

- numeric labels determine collection order;
- `encode_int(label)` bytes determine lexicographic payload order.

R3-3B passed small exhaustive chain tests through size 7, but independent review found a decimal-boundary correctness counterexample.

Homogeneous 11-Action chain:

current R3-3B forcing sequence:
`[10, 11, 1, 2, 3, 4, 5, 6, 7, 8, 9]`

exact exhaustive minimum:
`[10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 11]`

The flaw is that an occupied numeric source slot does not imply its target label is the next unresolved label-sensitive bytes. If lower numeric source slots remain unresolved, they may serialize earlier.

Correct law:

> A label may be forced by first-difference reasoning only when all preceding bytes are invariant across every legal completion beneath the current state.

Equivalent operational rule:

> The occurrence must be the next unresolved label-sensitive point in the actual frozen serialization order.

Call this the next-observable-byte law.

R3-3A remains the exact small-case general certificate. R3-3B remains useful historical evidence but is not accepted as a correct large-chain result until this repair passes.

## Required behaviour

1. Start from exact base `c3d136dc4217059d4434f8d39a273fa398c4e64d` in a fresh dedicated worktree.

2. Preserve the Origin-only scope: `entry_origin`, numerically sorted `success_continuations`, and numerically sorted program Origin sites.

3. Reproduce the chain-11 counterexample before changing forcing logic.

4. Add a focused regression test that captures the incorrect R3-3B chain-11 assignment and the exact exhaustive minimum.

5. Replace source-slot-based target forcing with a proof-based next-observable-byte eligibility check.

6. A continuation target may be forced only when every continuation element capable of serializing before that target occurrence is already resolved enough to determine its exact preceding bytes.

7. If any lower numeric source slot remains unresolved and could own a continuation that serializes first, do not force the later target.

8. In that blocked state, expose or retain an explicit ownership decision instead of emitting speculative bytes.

9. Preserve the exact entry-origin rule: when entry_origin is the first Origin-label-sensitive frozen field and preceding bytes are fixed, assign the remaining label whose exact `encode_int` bytes are minimal.

10. Keep numeric collection order and encoded-byte order as separate concepts throughout the implementation.

11. Do not substitute numeric label order for encoded-byte order or encoded-byte order for numeric collection order.

12. Prefix pruning is allowed only for a prefix proven identical for every legal completion beneath the branch up to the comparison point.

13. If exact next bytes are not proven, the walker must block and branch rather than serialize a guessed collection order.

14. Preserve at least three deterministic branch-order policies and prove final canonical payload independence from branch order.

15. Extend the independent exhaustive Origin reference so decimal-width continuation structure is tested, not merely primitive `encode_int` comparisons.

16. Require exact full-payload differential proof for homogeneous chain size 10.

17. Require exact full-payload differential proof for homogeneous chain size 11.

18. For chain-10/11 exhaustive reference, it is permitted to first prove/fix the entry Origin from frozen byte law and then exhaustively enumerate every remaining legal Origin-label assignment.

19. Only after chain-10 and chain-11 exact parity passes, rerun chains 12, 100 and 1000 and record corrected solver statistics.

20. Do not preserve the former zero-branch claim by adding heuristic pruning. If the corrected 1000-chain search becomes expensive, report the real counters and STOP.

21. Preserve R3-3A and R3-3B evidence. Document this task as a correctness repair; do not rewrite/delete the failed R3-3B result.

22. Do not generalise to Facts, Branches, Batches, Templates, ScopedRoles, generic graph I/R, full Enc_V2 or production wiring.

## Relevant components

Authorised mutation is limited to:

- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-09-01-rocket-v3-r3-3b1-next-observable-byte.md`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_origin_walk.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_origin_walk.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_origin_walk_test.ml`
- `tethers-0.1/engine-ocaml/bin/dune`

Read-only authorities:

- `tethers_core_canonical_v2_format.ml/.mli`
- `tethers_core_canonical_v2_reference.ml/.mli`
- `tethers_core_rocket_v3_encode.ml/.mli`
- R3-1 model
- R3-2 partition/refinement
- accepted R3-3A worker note/tests
- completed R3-3B worker note/tests

If correctness requires editing a read-only authority, STOP.

## Frozen decisions and invariants

- Frozen Enc_V2 bytes do not change.
- ProgramDigest V2 semantics do not change.
- `success_continuations` remain sorted by numeric `from_origin` label.
- Origin sites remain sorted by numeric Origin label.
- `encode_int` remains decimal text plus `;`.
- Numeric order and encoded-byte order are distinct.
- First-difference pruning is valid only after all preceding bytes are completion-invariant.
- Known semantic identity alone does not force a numeric label.
- Known source-slot occupancy alone does not force a target label.
- The relevant occurrence must be next-observable in frozen serialization.
- Same R3-2 cell is not automorphism proof.
- Raw IDs/internal vertex handles are non-semantic.
- No heuristic pruning or wall-clock identity decision.
- R3-3A remains exact small-case authority.
- R3-3B remains historical evidence including its flaw.

## Acceptance criteria

1. Fresh work starts from exact base `c3d136dc4217059d4434f8d39a273fa398c4e64d`.

2. Existing chain-11 failure is reproduced before repair.

3. Test evidence records both the former incorrect chain-11 assignment and the exact exhaustive minimum.

4. The first complete-payload difference between the incorrect and exact chain-11 results is captured.

5. Target forcing explicitly checks next-observable-byte eligibility rather than source-slot occupancy alone.

6. A target behind unresolved lower numeric source slots is not forced.

7. At least one focused fixture proves that blocked behaviour.

8. Ownership/branch resolution occurs when preceding collection ownership is unresolved.

9. Entry-origin remains exactly forceable by frozen first-difference law.

10. Numeric collection ordering and encoded-byte ordering remain separately represented.

11. Real continuation fixtures cross the 9/10/11 decimal-width boundary.

12. Prefix pruning compares only completion-invariant frozen prefixes.

13. No speculative unresolved collection order is serialized.

14. Three deterministic branch policies return identical canonical payloads.

15. The exhaustive reference remains independent from walker search logic.

16. Chain-10 repaired walker payload matches exhaustive reference byte-for-byte.

17. Chain-11 repaired walker payload matches exhaustive reference byte-for-byte.

18. Chain-10/11 digest parity is proven using the complete frozen payload where applicable.

19. Chains 12, 100 and 1000 are rerun only after chain-10/11 exact proof passes.

20. Corrected 1000-chain solver statistics are recorded without assuming zero branching.

21. Existing R3-3A, R3-1, R3-2 and V2 regression suites remain green.

22. Final diff stays strictly within authorised R3-3B1 paths with no cross-family/full Rocket generalisation.

## Required verification

- Use a fresh dedicated worktree tracking `origin/feature/rocket-v3-r3-3b1-next-observable-byte`.
- Confirm exact base `c3d136dc4217059d4434f8d39a273fa398c4e64d`, branch and clean worktree.
- Run `pwsh -NoProfile -File scripts/check-dev-tools.ps1`.
- Run packet checker and require `control-v1/READY`.
- Verify exact authorised OCaml switch.
- Read current R3-3B walker/worker note/tests and frozen continuation ordering before mutation.
- Reproduce and record the chain-11 failure first.
- Implement the smallest exact repair justified by next-observable-byte law.
- Run existing chains 1–7, exact chain-10, exact chain-11, decimal-boundary structural cases, symmetric/disconnected/multi-chain cases, three branch policies, raw-ID renaming and storage permutation.
- Only after chain-10/11 parity passes, run chain 12, 100 and 1000.
- Record `emitted_bytes`, `forced_assignments`, `decision_points`, `branches_explored`, `prefix_prunes`, `completed_candidates`, and `max_depth`.
- Run Origin-walker focused suite.
- Run R3-3A `39/39`.
- Run R3-1 `214/214`.
- Run R3-2 `4807/4807`.
- Run V2 suites and the 5,000-case differential corpus.
- Run `opam exec --switch="D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml" -- dune build @all`.
- Run `opam exec --switch="D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml" -- dune runtest --force`.
- Run `git diff --check`.
- Inspect base-to-HEAD diff and prove authorised paths only.
- Commit implementation/tests and record exact 40-character implementation checkpoint.
- Write worker note and mark packet `COMPLETE`; no implementation/test mutation after checkpoint.
- Run packet checker and require `control-v1/COMPLETE`.
- Push normally, prove local HEAD == remote HEAD, require clean worktree, report evidence and STOP.

## Forbidden changes

- No frozen V2 modification.
- No ProgramDigest modification.
- No Core/validator/lowerer changes.
- No R3-1/R3-2 changes.
- No R3-3A changes.
- No cross-family solver.
- No generic graph I/R.
- No automorphism/orbit pruning.
- No assumption that refinement equivalence means interchangeability.
- No raw-ID/internal-vertex ordering.
- No speculative prefix.
- No heuristic first-occurrence assignment.
- No “known source means force target” rule.
- No memo/component pruning.
- No randomised search.
- No wall-clock cutoff.
- No new dependency.
- No production integration.
- No V1 fallback.
- No R3-3C/R3-4/release/version work.

## Stop conditions

- Repaired walker disagrees with exhaustive chain-10 or chain-11.
- No exact condition can determine when a continuation target becomes next-observable.
- Exact prefix construction requires speculative unresolved collection ordering.
- Different branch policies produce different canonical payloads.
- Decimal-width behaviour reveals another unmodelled ordering dependency.
- Correctness requires changing frozen V2, Core, R3-2 or R3-3A.
- Correctness requires heuristic pruning.
- Chain-1000 becomes combinatorially explosive after correctness repair.
- Two materially similar failures recur without a new diagnosis.
- Checkout/branch/base/packet state differs after fetch.

## Expected pre-existing changes

None.
