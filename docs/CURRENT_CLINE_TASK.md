# Rocket V3 — R3-3B Enc_V2 Origin Canonical-Augmentation Crucible

Control contract: `1`

Status: `COMPLETE`

Task colour: `Red`

Owner: `Codex`

Route: `Standalone OCaml research crucible on a fresh dedicated worktree; exact Origin-only incremental Enc_V2 walker and differential tests. No production integration.`

Base commit: `68b33256611e05b477f1dd0eb0fa4811a2430e2a`

Branch: `feature/rocket-v3-r3-3b-origin-walker`

OCaml switch path: `D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml`

Worker note: `docs/worker-notes/2026-09-01-rocket-v3-r3-3b-origin-walker.md`

Related issue: `#5 — BUG: Rocket V2 factorial search on simple sequential Action chains`

Updated: 2026-09-01

## Objective

Build the smallest exact research crucible that can answer one question:

> Can frozen V2 Origin labels be chosen by an incremental Enc_V2-aware canonical-augmentation walker that emits only proven bytes, branches only when the next frozen bytes are genuinely unresolved, and collapses the factorial label domain in realistic structures such as the homogeneous sequential Action chain?

This task is deliberately limited to the Origin-label slice of frozen Enc_V2:

`entry_origin -> success_continuations -> origin_sites`

Do not generalise to Facts, Branches, Batches, Templates, ScopedRoles, the full encoder, production canonicalisation or planner integration.

The output of this task is experimental proof machinery plus evidence. It is allowed to STOP with a Red result if the exact walker still explodes.

## Relevant background and existing behaviour

R3-0/R3-1/R3-2 are accepted. R3-2 stable typed refinement makes the 1000 homogeneous sequential Action chain fully discrete using 6999 relation visits and no structural search.

R3-3A then proved that semantic discreteness does not determine frozen V2 numeric labels. The chain-3 leaf initially assigned entry Origin label `2` while the exact V2 oracle required `1`.

R3-3A now provides an exact small-case certificate by complete legal label-domain enumeration and frozen `encode_program` comparison. It is correct but factorial and therefore an oracle/proof baseline only.

The key frozen dual-order law is:

- many Enc_V2 collections are sorted by the **numeric integer label**;
- the same label is emitted using `encode_int n = decimal(n) ^ ";"`;
- canonical payload comparison is unsigned-byte lexicographic.

Therefore numeric order and byte order diverge at decimal-width boundaries, for example:

`9 < 10` numerically, but `"10;" < "9;"` lexicographically.

A second frozen law is lexicographic finality:

> If two legal completion families share an identical emitted prefix and the first guaranteed differing byte of family A is lower than family B, no later bytes in B can recover. B may be discarded exactly.

However this law is usable only for bytes already guaranteed by the partial assignment. The walker must never invent an order for an unresolved sorted collection.

R3-2 cell equivalence is not an automorphism proof. Same-cell vertices may guide branch order but may not be collapsed or skipped merely because refinement did not distinguish them.

## Required behaviour

1. Create a standalone experimental Origin canonical-augmentation module and focused test executable. It must not be called by production Rocket V2/V3, the planner, wire adapter or Rust host.

2. Model only the exact frozen Origin-sensitive projection needed for this crucible: optional `entry_origin`, `success_continuations` sorted by numeric `from_origin` label, and program-level `origin_sites` sorted by numeric Origin label. Batch sites, Facts, Roles, Templates and Branches are out of scope.

3. Reuse frozen V2 primitive encoding/comparison functions wherever publicly available. Any tiny test-only projection serializer that must duplicate field framing must be documented and differentially checked against the corresponding frozen Enc_V2 behaviour on every tractable fixture. Do not edit the frozen V2 format.

4. Represent a partial legal Origin label assignment explicitly as a bijection constraint: each Origin owns at most one numeric label and each numeric label belongs to at most one Origin.

5. The incremental walker must emit bytes only while those bytes are identical for every legal completion under the current partial assignment. When that stops being true, it must return a typed decision point rather than fabricate an ordering.

6. Support at minimum two typed decision forms:
   - `NeedLabel(origin)`: the next frozen bytes require the label of a fixed semantic Origin;
   - `NeedOwnerOfNumericSlot(slot)`: the next frozen collection element depends on which unbound Origin owns a particular numeric sort position.
   Exact OCaml names may differ.

7. Implement the exact forced-label rule: when the next guaranteed differing bytes are solely `encode_int(label)` for one fixed Origin and all preceding bytes are fixed, only the remaining label with byte-minimal exact encoding can survive. Record this as a forced assignment, not a heuristic branch.

8. Do not apply the forced-label rule when assigning a numeric value changes which collection member is emitted before the current byte position. In that case the walker must block on numeric-slot ownership or another explicit unresolved decision.

9. For an unresolved decision, enumerate every legal alternative unless a lower exact emitted prefix has already killed that branch. No semantic-signature, cell, raw-ID or traversal-order pruning is authorised.

10. Maintain an incumbent complete payload when one exists. Prefix pruning is permitted only when the branch has emitted an exact byte prefix guaranteed for every completion below that branch and that prefix is already lexicographically greater than the incumbent prefix at the first differing byte.

11. R3-2 semantic/refinement evidence may be used only to choose deterministic branch exploration order in this task. Same-cell membership is not an automorphism certificate and must not eliminate alternatives.

12. Branch exploration order must not influence the final canonical payload. Provide at least three deterministic branch-order policies and prove identical results on all oracle-sized fixtures.

13. Build an independent exhaustive mini-Origin oracle for small cases by enumerating every legal Origin-label bijection and selecting the exact unsigned-byte minimum of the same projection. Do not call the new walker from the oracle or vice versa.

14. Differentially prove walker == exhaustive mini-Origin oracle for homogeneous success chains sizes 1 through 7 and for all other small fixtures within the chosen exhaustive limit.

15. Add exact dual-order boundary fixtures involving family sizes/remaining domains around 8/9/10/11/12. Avoid requiring 12! enumeration: pre-bind or constrain all but a tractable residual set where necessary, and exhaustively compare those residual legal assignments.

16. Add adversarial Origin-only fixtures at minimum:
   - two structurally symmetric/twin Origins;
   - three unresolved anonymous Origins;
   - disconnected Actions with distinct fixed body bytes;
   - disconnected Actions with equal fixed body bytes;
   - two independent success chains;
   - a branching or converging continuation shape if valid under Core constraints.
   Do not collapse equal-refinement entities without exhaustive proof.

17. Add raw-ID renaming and source/storage permutation metamorphic variants for every fixture class. Exact canonical projection bytes and deterministic search statistics must remain invariant for the same branch-order policy.

18. Instrument deterministic solver statistics at minimum:
   - emitted_bytes;
   - forced_assignments;
   - decision_points;
   - branches_explored;
   - prefix_prunes;
   - completed_candidates;
   - max_depth.
   No wall-clock value may influence the result.

19. Run the homogeneous Action chain at sizes 10, 100 and 1000 through the walker. Do not use the factorial oracle at those sizes. Record all solver statistics and demonstrate that the implementation does not pre-enumerate the full permutation domain.

20. For the 1000-chain, if the walker exceeds 1,000,000 explored branches or 100,000 completed candidates, STOP as a Red performance finding. Do not add heuristic pruning to make the gate pass.

21. Do not generalise the walker beyond Origins in this task, even if the design appears obvious. The purpose is to prove the blocked-walker and dual-order mechanics before exposing them to cross-family dependencies and scoped roles.

22. Preserve all accepted R3-3A code and evidence as the independent exact small-case authority. No R3-3A correctness weakening, production cutover, general graph I/R implementation or release work is authorised.

## Relevant components

Authorised mutation is limited to:

- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-09-01-rocket-v3-r3-3b-origin-walker.md`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_origin_walk.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_origin_walk.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_origin_walk_test.ml`
- `tethers-0.1/engine-ocaml/bin/dune`

Read-only authorities include:

- `tethers_core_canonical_v2_format.ml/.mli`
- `tethers_core_canonical_v2_reference.ml/.mli`
- `tethers_core_rocket_v3_encode.ml/.mli` (R3-3A exact certificate)
- `tethers_core_rocket_v3_model.ml/.mli`
- `tethers_core_rocket_v3_partition.ml/.mli`
- `tethers_core_rocket_v3_refine.ml/.mli`
- accepted R3-0/R3-1/R3-2/R3-3A worker notes and tests.

If the exact focused implementation cannot be completed without editing a read-only authority, STOP and report why.

## Frozen decisions and invariants

- Frozen Enc_V2 bytes and ProgramDigest V2 semantics do not change.
- Numeric label order and encoded-byte order are distinct and must both be modelled.
- A partial prefix is authoritative only if every legal completion below that state emits exactly that prefix.
- First guaranteed differing byte decides lexicographic order permanently.
- R3-2 discreteness identifies semantic entities; it is not numeric-label authority.
- R3-2 same-cell equivalence is not automorphism proof.
- Raw IDs and internal vertex numbers are lookup handles only, never canonical ordering/pruning evidence.
- Prefix pruning must be justified solely by exact emitted bytes.
- The exhaustive mini-Origin oracle and R3-3A remain independent correctness authorities.
- No wall-clock decision, random branch order or hash iteration may affect canonical output.
- No production integration or V1 fallback.

## Acceptance criteria

1. A standalone Origin-only experimental walker and focused tests exist and no production call path references them.

2. The crucible faithfully models `entry_origin`, numerically sorted `success_continuations`, and numerically sorted Origin sites, with all other families excluded.

3. Frozen primitive encoding/comparison is reused where possible; any duplicated projection framing is documented and differentially validated without changing frozen V2.

4. Partial assignments enforce a valid Origin-label bijection at every state.

5. The walker never emits a byte that differs across legal completions beneath the current partial state; unresolved emission returns a typed decision.

6. Tests exercise both fixed-entity label decisions and numeric-slot-owner decisions.

7. At least one fixture proves a label choice is forced solely by the first guaranteed differing `encode_int` bytes.

8. At least one dual-order fixture proves the walker does not incorrectly force the byte-smallest label when that numeric assignment would change the earlier collection member.

9. Every unresolved legal alternative is explored unless eliminated by a proven exact-prefix comparison.

10. Every prefix prune has an exact first-differing-byte witness and no heuristic/cell-based prune exists.

11. R3-2 evidence affects branch order only; no same-cell candidate is skipped without exact byte proof.

12. Three deterministic branch-order policies return identical canonical projection bytes on all focused and exhaustive fixtures.

13. The exhaustive mini-Origin oracle is implementation-independent from the walker and enumerates the complete legal small-case Origin label domain.

14. Chains 1–7 match the exhaustive mini-Origin oracle exactly.

15. 8/9/10/11/12 boundary fixtures match exhaustive residual-domain references and demonstrate numeric-order/byte-order divergence correctly.

16. Symmetric/disconnected/multi-chain/valid branch-shape adversarial fixtures match the exhaustive oracle wherever tractable.

17. Raw-ID and storage-order metamorphic variants produce identical canonical bytes and same-policy statistics.

18. Required deterministic statistics are present, repeatable and excluded from identity.

19. Chains 10, 100 and 1000 complete without pre-enumerating their factorial domains, and exact solver statistics are recorded.

20. The 1000-chain remains below the explicit Red gate of 1,000,000 explored branches and 100,000 completed candidates; otherwise the task stops rather than adding heuristic pruning.

21. Final diff remains Origin-crucible-only and does not generalise to Facts/Branches/Batches/Templates/Roles or full Enc_V2 integration.

22. R3-3A, R3-1, R3-2, V2 suites, `dune build @all`, `dune runtest --force`, `git diff --check` and packet checker remain green; checkpoint/closeout/push/local==remote/clean requirements are satisfied.

## Required verification

- Use a fresh dedicated worktree tracking `origin/feature/rocket-v3-r3-3b-origin-walker`.
- Confirm exact branch HEAD and base `68b33256611e05b477f1dd0eb0fa4811a2430e2a`.
- Confirm clean initial worktree and `Expected pre-existing changes: None`.
- Run `pwsh -NoProfile -File scripts/check-dev-tools.ps1`.
- Run the task packet checker and require `control-v1/READY`.
- Verify the exact authorised OCaml switch with explicit `--switch`.
- Read the frozen V2 top-level encoder, numeric collection sort rules, R3-3A certificate, R3-2 interfaces and relevant Core continuation invariants before mutation.
- Implement the mini exhaustive oracle first or alongside the walker, with no shared search implementation.
- Prove chains 1–7 and adversarial small fixtures against exhaustive reference before attempting large chains.
- Run the decimal-boundary residual-domain cases before large-chain claims.
- Run chain 10, then 100, then 1000. If the explicit 1000 Red gate is exceeded, STOP and record exact counters.
- Run focused Origin-walker tests and record exact check count plus key solver statistics.
- Run existing R3-3A focused tests.
- Run existing R3-1 and R3-2 suites.
- Run existing V2 suites and generated differential corpus.
- Run `opam exec --switch="D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml" -- dune build @all`.
- Run `opam exec --switch="D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml" -- dune runtest --force`.
- Run `git diff --check`.
- Inspect full base-to-HEAD diff and prove only authorised paths changed.
- Commit implementation/tests and capture the exact 40-character implementation checkpoint.
- Write the worker note from evidence and transition this packet to `COMPLETE`. No implementation/test mutation after the checkpoint.
- Run packet checker and require `control-v1/COMPLETE`.
- Push normally; require local HEAD == remote HEAD and a clean worktree; report evidence and STOP.

## Forbidden changes

- No edit to frozen V2 format/oracle/production/IR.
- No edit to R3-1 model, R3-2 partition or refinement.
- No edit to R3-3A certificate implementation.
- No Core/validator/lowerer change.
- No production planner/wire/Rust-host integration.
- No generic graph I/R implementation.
- No automorphism/orbit pruning.
- No assumption that same R3-2 cell means interchangeable.
- No semantic-signature pruning.
- No raw-ID/internal-vertex ordering or pruning.
- No memo pruning.
- No component decomposition.
- No unproved branch-and-bound.
- No search budget used to return a non-minimal answer.
- No wall-clock cutoff.
- No randomised search.
- No new dependency.
- No V1 fallback.
- No release/version work.
- Do not begin cross-family/full Enc_V2 generalisation automatically.

## Stop conditions

- The mini walker and exhaustive oracle disagree on any valid tractable fixture.
- A partial prefix cannot be proven completion-invariant but the proposed algorithm requires emitting it.
- Numeric collection ordering cannot be represented without changing frozen Enc_V2.
- Correctness requires treating R3-2 cell equivalence as automorphism.
- Correctness requires raw IDs/internal handles as ordering evidence.
- Any branch-order policy changes the final canonical bytes.
- A decimal-boundary fixture reveals an unmodelled numeric-vs-byte ordering dependency.
- The 1000-chain exceeds 1,000,000 explored branches or 100,000 completed candidates.
- Exactness requires editing frozen V2, Core, R3-2 or R3-3A.
- Two materially similar failures recur without a new diagnosis.
- Checkout/branch/base/packet state differs after fetch.

## Expected pre-existing changes

None.
