# Worker Note

Task: `Rocket V3 — R3-3B3B ListIso / Matching Reduction Crucible`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `Codex`
Status: `BLOCKED`
Base commit: `eae11c5fd2bb964c0f586c48823f406d2472dccf`
Implementation checkpoint: `0fdef0ec5bcf66b99dbb15f0c9ecfb034887e472`

## Requested outcome

Determine whether partial frozen parent-vector completion for the supported
connected rooted success-tree shape has an exact standard Tree ListIso or
matching/tree-DP reduction, without beginning production B3 work.

## Changes made

- Added the research document
  `docs/review/rocket-v3/R3_3B3B_LISTISO_REDUCTION.md`.
- Added the research-only module, interface and test:
  `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_tree_listiso.ml`,
  `.mli` and `_test.ml`.
- Added the focused Dune test stanza.
- Changed the packet to `IN_PROGRESS`, recorded the implementation checkpoint,
  and changed it to `BLOCKED` after the bounded theorem investigation.
- Did not modify Frozen Enc_V2, ProgramDigest V2, Core, validator, B2, R3-1,
  R3-2 or the B3A oracle.

## Decisions and assumptions

The exact predicate remains the B3A definition: a bijection from semantic
Origins to numeric slots, fixed entry label, and exact satisfaction of every
supplied parent-vector relation.  A supplied `q[i] = j` is a binary placement
constraint between the images of slots `i` and `j`, not a unary allowed-image
list.

The research module implements exact bottom-up child matching for one
connected partial component, including injective augmenting-path matching,
entry pinning and explicit ProgramComplete-root handling.  Multiple partial
components return `Unknown_global_packing`; independent local matches are not
promoted to a global completion theorem.

## Evidence

- Fresh worktree and branch matched the requested READY packet at
  `9b2f6a47f6559e06ae2ec5234ef6c875609b8b66`; research base ancestry matched
  `eae11c5fd2bb964c0f586c48823f406d2472dccf`.
- `scripts/check-dev-tools.ps1` passed.
- The packet checker passed `control-v1/READY` before mutation.
- The explicit switch
  `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml` resolved to
  OCaml `5.5.0`, Dune `3.24.0` and Yojson `2.2.2`.
- B3A authority reproduced the historical seven-node rank failure: target `6`
  versus exact target `5`, first frozen payload difference byte `55`,
  `0x36` versus `0x35`.
- B3A authority reproduced the three-node local-capacity false positive:
  `[2, Complete]` was accepted by the old candidate and rejected by exact
  brute force.
- B3B3 focused test passed `10/10`.  Deterministic candidate statistics were:
  `partial_prefixes_checked=9 candidate_states=43 candidate_pairs=35`
  `matching_instances=40 matching_vertices=60 matching_edges=35`
  `matching_failures=11 candidate_targets_considered=35 committed_targets=0`
  `exact_oracle_complete_assignments=8 complete_permutations_enumerated=0`.
- The matching candidate rejected the three-node coupled-placement false
  positive.  On the seven-node case it accepted both target `5` and target
  `6` as legal full placements, while the frozen exact oracle selected target
  `5` by byte comparison.
- Disconnected partial components returned explicit
  `Unknown_global_packing`.
- `tethers_core_rocket_v3_tree_completion_test.exe` passed `47634/47634`.
- `tethers_core_rocket_v3_success_path_test.exe` passed `69/69`.
- `dune build @all` passed.
- `dune runtest --force` passed, including B3B3 `10/10`, B3A `47634/47634`,
  R3-3A `39/39`, R3-1 `214/214`, R3-2 `4807/4807`, V2 suites, and generated
  corpus `valid=5000 mismatches=0`.
- The implementation checkpoint is
  `0fdef0ec5bcf66b99dbb15f0c9ecfb034887e472`.

## Discoveries

Standard Tree ListIso requires two complete trees plus unary allowed-image
lists.  The Tethers numeric target tree is only partially specified, and its
prefix imposes binary parent-image relations.  The three-node case is the
smallest inherited obstruction to treating those relations as independent
lists.

The connected matching recurrence is exact for the connected partial state it
represents.  It does not solve the general partial forest because separate
components may have individually valid but globally overlapping placements.
The seven-node target-6 vector is a legal completion; it loses to target 5 by
the frozen byte objective, rather than being infeasible.

## Remaining risks

No compact sound-and-complete global matching/tree-DP state was established for
arbitrary partial forests.  Using the B3A oracle as that state would merely
reintroduce forbidden complete-label enumeration.

## Smallest next action

Specify and independently review a global disjoint-placement state for partial
forest embeddings, then rerun the B3A differential corpus.  Do not begin a
production B3 canonicaliser from this blocked result.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/review/rocket-v3/R3_3B3B_LISTISO_REDUCTION.md`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_tree_completion.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_tree_completion_test.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_tree_listiso.ml`
- `0fdef0ec5bcf66b99dbb15f0c9ecfb034887e472`
