# Rocket V3 R3-3B Origin Walker Worker Note

Task: `Rocket V3 — R3-3B Enc_V2 Origin Canonical-Augmentation Crucible`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `68b33256611e05b477f1dd0eb0fa4811a2430e2a`

Implementation checkpoint: `5a53e5307903d6b02e364904d67300a359a2541e`

## Requested outcome

Provide a standalone exact Origin-only canonical-augmentation crucible for
the frozen Enc_V2 projection, without beginning general Rocket V3 search or
altering V2, R3-3A or production wiring.

## Changes made

- Added `tethers_core_rocket_v3_origin_walk.ml/.mli`.
- Added its independent focused test executable and Dune stanza.
- Added an explicit Origin bijection state, typed initial decisions,
  completion-invariant frozen-prefix emission, exact unsigned-byte label
  comparison and exhaustive residual branching.
- The walker validates Core first, excludes Batch/Facts/Branches/Templates/
  Roles from this Origin-only slice, and uses raw IDs only for construction
  lookup.
- The complete candidate payload is emitted through the existing frozen
  format primitives; no alternate Enc_V2 encoder was added.
- Updated the packet to `COMPLETE`.

## Decisions and assumptions

The READY packet’s historical R3-3A evidence records the original chain-3
diagnostic: the old discrete-leaf bridge emitted entry label `2`, while the
slow oracle emitted `1` at payload byte 13. The fresh R3-3B worktree was
clean, as required by the packet, so no old diagnostic source edits existed
locally to replay or discard.

The exact forced assignment is applied only after a known numeric source slot
has fixed the next continuation field and the target label is the next
unresolved byte. The minimum is selected by the actual frozen
`encode_int`/unsigned-byte comparator, not numeric order. If collection member
ownership is unresolved, the walker branches over every legal owner.

Semantic site shape and continuation distance affect exploration order only.
They are not label authority, and no raw ID, internal vertex number, source
order, storage order or refinement colour is used as canonical evidence.

## Evidence

Startup:

- Worktree: `D:\The Next Thing\Tethers Lang - Rocket V3 R3-3B Origin Walker`.
- Branch: `feature/rocket-v3-r3-3b-origin-walker`.
- Initial HEAD matched READY HEAD `fd098304801ad11497233d5fc7ad98f757745b09`.
- Initial worktree was clean and packet pre-existing changes were `None`.
- `scripts/check-dev-tools.ps1`: passed.
- Initial packet checker: `control-v1/READY`.
- Exact switch: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`.
- `ocamlc -version`: `5.5.0`; `dune --version`: `3.24.0`.

Focused command:

`opam exec --switch="D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml" -- dune exec bin/tethers_core_rocket_v3_origin_walk_test.exe`

Result: `rocket-v3-origin-walk: 100/100 checks passed`.

The focused suite covered chains 1–7 against an independent exhaustive
Origin permutation oracle; Anchor, Action and Together sites; symmetric and
disconnected Origin cases; typed entry/owner decisions; raw-ID renaming;
storage reversal; three branch policies; and encoded integer boundaries
8/9, 9/10, 10/11, 11/12 and 12/2.

Scaling statistics:

| chain | emitted_bytes | forced_assignments | decision_points | branches_explored | prefix_prunes | completed_candidates | max_depth |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10 | 1436 | 10 | 0 | 0 | 0 | 2 | 0 |
| 12 | 1704 | 12 | 0 | 0 | 0 | 2 | 0 |
| 100 | 13506 | 100 | 0 | 0 | 0 | 2 | 0 |
| 1000 | 139516 | 1000 | 0 | 0 | 0 | 2 | 0 |

The 1000-chain is below the Red thresholds of 1,000,000 explored branches
and 100,000 completed candidates, with no factorial pre-enumeration.

Regression:

- `dune build @all`: passed.
- `dune runtest --force`: passed.
- R3-1 model: `214/214`.
- R3-2 refinement: `4807/4807`; existing chain-1000 evidence remained
  `6999` relation visits.
- R3-3A exact Stage-A suite: `39/39`.
- V2 reference, production and IR suites: passed.
- Existing generated differential corpus: `5000` valid, `0` mismatches.
- `git diff --check`: passed.
- Authorised-path proof: only the packet, this worker note, Dune and the
  three new Origin-walker files changed.

## Discoveries

R3-2 discreteness does not choose frozen V2 numeric labels. The chain witness
is resolved because the frozen continuation list exposes the known
predecessor’s target label before later collection members; the exact byte
minimum can therefore force the target owner. This is a byte-law proof, not
an execution-chain special case.

The focused chain-1000 run has zero decision points and zero explored
branches. The only completed candidates are the greedy incumbent and the
unpruned exact leaf, so the result remains independently checked rather than
being returned from a heuristic shortcut.

## Remaining risks

This is deliberately not a general Rocket V3 engine. Cross-family Facts,
Branches, Batches, Templates, ScopedRoles, generic I/R, automorphism
pruning, component recursion and production integration remain later work.
The tested Origin-only projection rejects those structures closed rather than
silently projecting them.

## Smallest next action

Stop this task. Do not begin R3-3C, R3-4 or general Rocket V3 search from this
worktree.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_format.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_reference.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_origin_walk.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_origin_walk_test.ml`
