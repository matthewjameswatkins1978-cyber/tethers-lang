# Rocket-Only Runtime V2 Cutover Worker Note

Task: `Rocket-Only Runtime V2 Cutover`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Lucy`

Status: `COMPLETE`

Base commit: `b32d1940a36ddd92ac7048f87787d7ca7ff7d63a`

Implementation checkpoint: `c7b30ecd40ecad52007eca16ac64cb8a93b59008`

## Requested outcome

Finish the pre-1.0 Canonical V2 production migration so live Human-source
evaluation uses Rocket V2 as the sole producer of new semantic ProgramDigest
identity, with no V1 fallback, while preserving runtime semantics and the
existing frozen Canonical V2 identity.

## Changes made

The live OCaml evaluation adapter now calls
`Tethers_core_canonical_v2_ir.canonicalize_ir` and never invokes the legacy V1
canonicalizer. The abstract Rocket token now binds the exact validated Core
program to the V2 canonical payload/preimage/digest from which it was computed.

The planner consumes the Rocket V2 token directly rather than a V1 rewritten
Core value. Runtime Action indices are derived from semantic control-flow
traversal, and Together operational group/member handles are derived from
semantic member Action positions rather than raw storage IDs.

The OCaml wire returns the full current
`tethers:v2:sha256:<64 lowercase hex>` ProgramDigest directly. The Rust host
validates that domain on matched responses and rejects legacy bare
`sha256:<hex>` values as non-current ProgramDigest identity.

The current Core benchmark was moved from V1 to Rocket V2. The live/current
module graph no longer requires V1 canonicalization. V1 remains only as
historical/reference/forensic evidence where separately named.

A dedicated Windows verification workflow was added because the full reference
host and real-engine rehearsal are native Windows verification surfaces. The
workflow builds OCaml 5.5.0 and Rust in one workspace, runs the cross-language
tests against the freshly built `tethers_mcp_main.exe`, verifies that live
modules do not call V1, checks branch whitespace against fetched main history,
and runs the repository task-packet checker.

## Decisions and assumptions

Canonical identity and operational Core identity are deliberately separate.
Rocket V2 computes semantic ProgramDigest identity without rewriting operational
Core IDs. Tests that previously expected V1-style rewritten Origin IDs were
updated to assert the current architecture instead.

There is no fallback to V1 when Rocket exceeds its deterministic budget. The
sequential Action-chain factorial-search weakness remains a separately tracked
problem in GitHub issue #5 and is the principal motivation for Rocket V3.

The frozen Enc_V2 bytes and `tethers:v2:sha256:` ProgramDigest contract remain
unchanged by this cutover.

## Evidence

Exact-head GitHub Actions run `33493864974` passed on implementation checkpoint
`c7b30ecd40ecad52007eca16ac64cb8a93b59008`.

The run passed:

- OCaml 5.5.0 dependency setup.
- `dune build @all`.
- `dune runtest --force`, including the frozen V2 vectors, Rocket V2 suite,
  Persistent Branch coverage, and the generated differential corpus.
- Static proof that the live evaluation/planner/wire modules do not reference
  `Tethers_core_canonical.canonicalize`.
- `cargo fmt --all -- --check`.
- `cargo check --all-targets --all-features --locked` on the native Windows
  reference-host platform.
- Focused real-engine CORE-9B ProgramDigest tests against the Dune-built engine.
- `git diff --check origin/main...HEAD`.
- Repository task-packet consistency in the pre-closeout state.

Earlier CI failures were diagnosed and corrected without changing the frozen
semantic objective: obsolete V1-ID-rewrite test assumptions, a Rust formatting
line, Linux verification of the Windows reference host, and shallow checkout
history for the branch-diff gate.

## Discoveries

The previous documentation phrase “Rocket V2 production cutover” had referred
to the accepted V2 implementation itself, not a completed live runtime migration.
The live adapter/planner/wire had remained coupled to V1 until this task.

Removing V1 rewriting exposed accidental runtime dependence on raw/storage
identity in Action/Together handle construction. The live planner now derives
those operational handles from semantic positions instead.

Rocket V2 refinement still does not understand enough of the complete Core
relation graph to collapse long sequential Action chains. Issue #5 records that
a simple sequence can reach factorial search and the deterministic budget around
11 Actions. This is not accepted as a user-facing Tether size limit.

## Remaining risks

GitHub issue #5 remains open and must not be treated as closed by this cutover.
Rocket V3 is expected to address the structural canonical-search weakness while
preserving exact Enc_V2/ProgramDigest_V2 identity unless a later explicit
pre-1.0 decision changes the identity format.

Historical V1 and Rocket research branches still exist remotely. Repository
hygiene issue #6 and the merged branch-safety controls prevent them from being
implicit bases for new tasks, but physical branch deletion remains separate
maintenance.

## Smallest next action

Merge the verified Rocket-only cutover to current `main`. Then create a fresh
Rocket V3 R3-0 branch from that exact post-merge `main` SHA and authorise only
the semantic relation inventory/design task before any V3 implementation.

## References

- GitHub PR #7 — Finish Rocket V2 production identity cutover
- GitHub issue #5 — Rocket V2 factorial search on simple sequential Action chains
- GitHub issue #6 — repository branch hygiene
- `docs/review/lucy-c-b4s-canonical-v2/CANONICAL_FORMAT_V2_SPEC_DRAFT.md`
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_v2_ir.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_evaluation_adapter.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_wire.ml`
- `tethers-0.1/host-rust/src/engine_stdio.rs`
