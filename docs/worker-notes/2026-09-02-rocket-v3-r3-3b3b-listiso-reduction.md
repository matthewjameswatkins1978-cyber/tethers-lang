# Worker Note

- **Task Packet:** `docs/CURRENT_CLINE_TASK.md` — Rocket V3 R3-3B3B ListIso / Matching Reduction Crucible
- **Owner:** `Codex`
- **Status:** `BLOCKED`
- **Base Commit:** `eae11c5fd2bb964c0f586c48823f406d2472dccf`
- **Implementation Checkpoint:** `0fdef0ec5bcf66b99dbb15f0c9ecfb034887e472`
- **Branch / Worktree:** `feature/rocket-v3-r3-3b3b-listiso-reduction` / `D:\The Next Thing\Tethers Lang - Rocket V3 R3-3B3B ListIso Reduction`

## Files Modified

- `docs/CURRENT_CLINE_TASK.md`
- `docs/review/rocket-v3/R3_3B3B_LISTISO_REDUCTION.md`
- `docs/worker-notes/2026-09-02-rocket-v3-r3-3b3b-listiso-reduction.md`
- `tethers-0.1/engine-ocaml/bin/dune`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_tree_listiso.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_tree_listiso.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_tree_listiso_test.ml`

## Behavioural Result

The research module models the partial numeric parent vector as an explicit
partial pattern.  For one connected partial component it performs exact
bottom-up child matching with injective augmenting-path matching, entry pinning
and explicit ProgramComplete-root handling.  For multiple partial components
it returns `Unknown_global_packing` rather than claiming that independent
component matches are globally disjoint.

The standard unary-list Tree ListIso reduction was not established.  The
prefix relation `q[i] = j` is a binary constraint between the semantic images
of numeric slots `i` and `j`; the three-node B3A false positive demonstrates
that independent unary placement lists lose this coupling.  No compact
sound-and-complete global packing state was established in this bounded task,
so no constructor or production B3 implementation was added.

## Invariants Preserved

- Frozen Enc_V2, ProgramDigest V2, Core, validator, B2, R3-1, R3-2 and the
  B3A brute-force oracle were not modified.
- No raw IDs, internal handles, storage order or refinement cells influence
  the candidate verdict.
- Matching is injective and child placements are simultaneous.
- The research module performs no complete Origin permutation enumeration.
- No production call path, forest solver or cross-family support was added.

## Negative Tests Added or Updated

- B3A tree-completion authority — reproduced the seven-node rank failure:
  target `6` versus exact target `5`, payload byte 55 `0x36` versus `0x35`.
- B3A tree-completion authority — reproduced the three-node local-capacity
  false positive: `[2, Complete]` accepted by the old candidate and rejected
  by exact brute force.
- `tethers_core_rocket_v3_tree_listiso_test.ml` — exact matching rejects the
  three-node coupled-placement false positive.
- `tethers_core_rocket_v3_tree_listiso_test.ml` — seven-node target `5` and
  target `6` are both legal completions; the frozen exact oracle chooses `5`.
- `tethers_core_rocket_v3_tree_listiso_test.ml` — disconnected partial
  components return explicit `Unknown_global_packing`.
- `tethers_core_rocket_v3_tree_listiso_test.ml` — semantic storage renaming
  preserves connected feasibility.

## Commands Executed

- `pwsh -NoProfile -File scripts/check-dev-tools.ps1` — `PASS`
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` —
  `PASS control-v1/READY` before mutation
- explicit switch inspection at
  `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml` — `PASS`
  (OCaml 5.5.0, Dune 3.24.0, Yojson 2.2.2)
- `opam exec --switch="D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" -- dune exec ./bin/tethers_core_rocket_v3_tree_completion_test.exe` —
  `PASS 47634/47634`
- `opam exec --switch="D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" -- dune exec ./bin/tethers_core_rocket_v3_tree_listiso_test.exe` —
  `PASS 10/10`; deterministic stats:
  `partial_prefixes_checked=9 candidate_states=43 candidate_pairs=35 matching_instances=40 matching_vertices=60 matching_edges=35 matching_failures=11 candidate_targets_considered=35 committed_targets=0 exact_oracle_complete_assignments=8 complete_permutations_enumerated=0`
- `opam exec --switch="D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" -- dune exec ./bin/tethers_core_rocket_v3_success_path_test.exe` —
  `PASS 69/69`
- `opam exec --switch="D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" -- dune build @all` — `PASS`
- `opam exec --switch="D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" -- dune runtest --force` — `PASS`; included B3B3 `10/10`, B3A `47634/47634`, R3-3A `39/39`, R3-1 `214/214`, R3-2 `4807/4807`, V2 suites and generated corpus `valid=5000 mismatches=0`.
- `git diff --cached --check` — `PASS` at implementation checkpoint
- staged authorised-path inspection — `PASS`

## Unrun Checks and Reason

- `git diff --check` after terminal documentation — pending until the closeout
  documentation commit is staged.
- terminal packet checker, normal push and local/remote equality — pending
  until the packet is marked `BLOCKED` and the worker note is committed.

## Discoveries

- Standard Tree ListIso assumes two complete trees and unary allowed-image
  lists.  The Tethers prefix instead leaves the numeric target tree unknown and
  constrains parent-image pairs.
- Connected-component matching is a useful exact sub-state, but independent
  component feasibility does not prove simultaneous global disjoint placement.
- The seven-node historical target-6 vector is itself a legal completion; it
  loses to target 5 by the frozen byte objective, not because it is infeasible.

## Remaining Risks

- No compact sound-and-complete global matching/tree-DP theorem was established
  for arbitrary partial forests.  Treating the B3A oracle as the candidate
  would reintroduce forbidden complete-label enumeration.

## Recommended Next Action

Specify and independently review a global disjoint-placement state for partial
forest embeddings, then rerun the B3A differential corpus.  Do not begin a
production B3 canonicaliser from this blocked result.
