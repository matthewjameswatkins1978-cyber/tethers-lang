# Rocket V3 R3-3B Origin Walker Worker Note

## Result

Status: `COMPLETE`

Branch: `feature/rocket-v3-r3-3b-origin-walker`

Base: `68b33256611e05b477f1dd0eb0fa4811a2430e2a`

Implementation checkpoint: `5a53e5307903d6b02e364904d67300a359a2541e`

Owner: Codex

Task colour: Red

The implementation remains an Origin-only standalone research crucible. It
does not call production Rocket V2/V3, alter frozen Enc_V2, alter R3-3A, or
implement general graph I/R, automorphism pruning, component recursion or
search budgets.

## Startup evidence

- Fresh worktree: `D:\The Next Thing\Tethers Lang - Rocket V3 R3-3B Origin Walker`
- Branch and initial HEAD matched the READY packet.
- Initial worktree was clean; the packet declared no pre-existing changes.
- `scripts/check-dev-tools.ps1`: passed.
- Packet checker: `control-v1/READY` at base `68b3325`, HEAD `fd09830`.
- Exact switch: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`.
- `ocamlc -version`: `5.5.0`.
- `dune --version`: `3.24.0`.

The READY packet preserved the R3-3A finding that the old discrete-leaf
bridge emitted entry label `2` while the slow oracle emitted `1` at payload
byte 13. The clean R3-3B worktree contained no old diagnostic edits to
discard; that historical mismatch is retained as packet evidence and the
new tests use the frozen oracle directly.

## Implemented proof surface

`tethers_core_rocket_v3_origin_walk.ml/.mli` implements a typed, reversible
Origin label walk over the legal bijection between Origin entities and labels
`1..N`. It models only:

- optional `entry_origin`;
- success continuations and `ProgramComplete`;
- program-level Anchor, Action and Together Origin sites;
- fixed non-Origin framing needed to compare the complete frozen payload.

Batch sites are excluded from the Origin domain. Facts, Branches, Roles,
Templates and other cross-family semantics are rejected as outside this
crucible. Raw IDs are construction lookups only. The semantic branch order is
derived from site shape and continuation distance; no raw ID, internal array
index or storage order is used as canonical label authority.

The first known continuation target is forced only after its source numeric
slot is occupied. The exact frozen byte comparator chooses the minimum
remaining `encode_int` byte sequence. This is valid because the target label is
the next unresolved byte after a fixed prefix; unresolved numeric-slot owners
remain explicit branch decisions. All legal residual alternatives are still
walked, and prefix pruning is enabled only for a guaranteed frozen prefix.

The complete candidate payload is assembled with the existing frozen format
primitives and compared against an independent test-only exhaustive oracle
that calls `Tethers_core_canonical_v2_format.encode_program` for every small
Origin permutation. No alternate Enc_V2 encoder exists.

## Focused evidence

Focused command:

`opam exec --switch="D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml" -- dune exec bin/tethers_core_rocket_v3_origin_walk_test.exe`

Result: `rocket-v3-origin-walk: 100/100 checks passed`.

The focused checks covered:

- homogeneous success chains 1 through 7 against the independent exact
  oracle;
- Anchor, Action and Together Origin site encodings;
- three structurally symmetric Origin anchors;
- disconnected Origin owner-slot cases and disconnected distinct Actions;
- typed initial decisions for entry label and numeric owner slot;
- raw-ID renaming and storage reversal, including same-policy statistics;
- all three branch-order policies with identical payloads;
- unsigned-byte boundaries 8/9, 9/10, 10/11, 11/12 and 12/2;
- empty-domain fail-closed behaviour.

The scaling statistics printed by the focused executable were:

| chain | emitted_bytes | forced_assignments | decision_points | branches_explored | prefix_prunes | completed_candidates | max_depth |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10 | 1436 | 10 | 0 | 0 | 0 | 2 | 0 |
| 12 | 1704 | 12 | 0 | 0 | 0 | 2 | 0 |
| 100 | 13506 | 100 | 0 | 0 | 0 | 2 | 0 |
| 1000 | 139516 | 1000 | 0 | 0 | 0 | 2 | 0 |

The 1000-chain is below the Red limits of 1,000,000 explored branches and
100,000 completed candidates. It exhibits no factorial pre-enumeration.

## Regression evidence

- `dune build @all`: passed.
- `dune runtest --force`: passed.
- Rocket V3 model: `214/214`.
- Rocket V3 refinement: `4807/4807`; existing chain-1000 refinement evidence
  remained `6999` relation visits.
- R3-3A exact Stage-A suite: `39/39`.
- V2 reference, production and IR suites: passed.
- Existing generated differential corpus: `5000` valid cases, `0` mismatches.
- `git diff --check`: passed.
- Authorised-path proof: only the packet, worker note, Dune stanza and the
  three new Origin-walker files changed.

No R3-3C/R3-4 or general Rocket V3 search work was started.
