# Worker Note

Task: `Rocket V3 — R3-3B2 Exact Success-Path Canonisation`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `3034117dffa16366fa73c7befd1cccbf0bb86033`

Implementation checkpoint: `e82ef8ce885fcc1060980974ecd9128aa2f55ce2`

## Requested outcome

Implement an isolated exact canonicaliser for the supported Origin-only case in which every program Origin occurs exactly once on one acyclic success path from `entry_origin` to `ProgramComplete`. The implementation must minimise the frozen Enc_V2 success-continuation bytes without enumerating complete Origin permutations, while leaving frozen V2, Core, R3-1, R3-2 and prior Rocket evidence unchanged.

## Changes made

- Added `tethers_core_rocket_v3_success_path.ml/.mli`.
- Added the focused `tethers_core_rocket_v3_success_path_test.ml` and its Dune test stanza.
- Changed the packet state from `READY` to `IN_PROGRESS` during the authorised implementation, then to `COMPLETE` after the checkpoint.
- Added supported-shape validation for a complete single Origin success path, including Anchor/Action/Together Origin sites, and deterministic rejection of disconnected or non-Origin/cross-family structures.
- Added label-space successor-table construction. The fixed entry label is selected by exact unsigned-byte comparison of frozen `encode_int` output. Numeric source labels remain the collection order; encoded target bytes remain the lexicographic comparison order.
- Added an exact partial-table feasibility predicate backed by rollback disjoint-set state. It rejects duplicate predecessors, predecessor-to-entry, self-loop/cycle, multiple terminal, closed-entry-with-other-component and incomplete final states.
- Added final mapping from the winning numeric path back to semantic Origin IDs and delegated payload emission to the existing frozen `Tethers_core_canonical_v2_format.encode_program`.
- No complete Origin permutation enumeration is performed by the new canonicaliser; the reported statistic is always `complete_permutations_enumerated = 0`.

## Decisions and assumptions

The semantic-path/label-path bijection is the key proof: a legal labelling of a single semantic path induces one rooted Hamiltonian successor table over numeric slots, and a legal rooted Hamiltonian successor table maps back uniquely by following successors from the fixed entry slot. The completion predicate is exact for these partial tables because every accepted Origin edge joins two distinct open path components; the only additional obstruction is closing the entry component while other components remain. Greedy choice is therefore lexicographic choice over the actual frozen target bytes subject to exact feasibility, not a heuristic assignment.

Stable partition order is not used as label authority. No raw ID, source/storage order, internal vertex number or refinement cell/colour number participates in a canonical choice. The implementation remains restricted to the simple-path crucible and does not implement forests, cross-family canonicalisation, general I/R search, prefix/orbit pruning or production integration.

## Evidence

- Startup was performed in `D:\The Next Thing\Tethers Lang - Rocket V3 R3-3B2 Success Path Canon` on branch `feature/rocket-v3-r3-3b2-success-path-canon`, starting from the requested base. The worktree was clean before mutation; the packet checker reported `control-v1/READY` before implementation.
- Exact switch verified: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`; OCaml `5.5.0`; Dune `3.24.0`.
- `scripts/check-dev-tools.ps1`: passed.
- Focused path suite: `rocket-v3-success-path: 69/69 checks passed`.
- Focused inherited suites: R3-1 `214/214`, R3-2 `4807/4807`, R3-3A `39/39`.
- Full `dune runtest --force`: passed. This included the V2 reference, production and IR suites, validator/lowerer/adapter/wire suites, and the generated V2 differential corpus `seed=308386 total=5000 valid=5000 mismatches=0`.
- `dune build @all`: passed.
- Exact chain differential coverage: chains 1 through 11 match the independent frozen Enc_V2 oracle byte-for-byte and digest-for-digest. Chain 11 produces `[10; 9; 8; 7; 6; 5; 4; 3; 2; 1; 11]`.
- Decimal structural boundary coverage: chains 9, 10, 11, 12, 99, 100, 999 and 1000 all complete with zero complete permutations enumerated. The same suite covers storage reversal, raw-ID renaming, distinct body bytes, Anchor/Action, Together, three deterministic choice orders and repeatable statistics.
- Chain-1000 statistics: `path_size=1000`, `successor_slots_processed=1000`, `candidate_targets_considered=1003`, `feasibility_checks=1003`, `rejected_infeasible_choices=3`, `committed_choices=1000`, `complete_permutations_enumerated=0`, `max_partial_components=1000`.
- `git diff --check`: passed before checkpoint.
- Final authorised-path proof was checked before checkpoint; only the packet, Dune stanza and new success-path implementation/interface/test paths were staged.

## Discoveries

The simple-path problem is directly solvable in label space. The frozen serializer exposes successor slots in numeric source-label order but compares target labels as decimal bytes, so numeric ordering and byte ordering must remain separate. The feasibility predicate was checked against an independent exhaustive table oracle for every partial prefix of the size-4 label-space domain. This gives a bounded proof of the greedy commits used by the path canonicaliser rather than relying only on final differential examples.

## Remaining risks

This is not a general Rocket V3 canonicaliser. The feasibility proof and implementation apply only to one complete acyclic success path with no Facts, Branches, Batches, Templates, Roles or other cross-family fields. General success forests and full Enc_V2 interactions remain future work. No production engine wiring was changed.

## Smallest next action

Keep this checkpoint as the R3-3B2 evidence boundary. Any forest or cross-family extension must be separately specified and authorised; do not begin it as part of this task.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_format.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_success_path.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_success_path_test.ml`
- `e82ef8ce885fcc1060980974ecd9128aa2f55ce2`
