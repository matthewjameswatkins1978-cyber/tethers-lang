# Current Implementation Task

Control contract: `1`

Task: `TETHERS CORE-1 — OCaml Core Type Foundation`

Owner: `OpenCode`

Status: `COMPLETE`

Task colour: `Amber`

Route: `OpenCode implementation + evidence → Lucy independent GitHub review`

Worker note: `docs/worker-notes/2026-08-11-core-1-ocaml-core-types.md`

Base branch: `feature/core-1-ocaml-core-types`

Base commit: `8bd975ae55c359ae09e30cfca3c905fdace0a01f`

Implementation checkpoint: `d03f8327a753c0b1f2380069b056db9e7cec7da7`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

Rust toolchain: read exact channel from `rust-toolchain.toml`; use plain Cargo (resolved by root pin); `--locked` mandatory

Toolchain preflight: `pwsh -NoProfile -File scripts/check-dev-tools.ps1` (run; all tools present)

Rust change class: `RUST_UNCHANGED`

## Objective

Introduce the first production representation of **Tethers Core** as a standalone OCaml semantic type module.

This packet defines types only.

It MUST NOT lower Human Tethers into Core, serialize Core, validate Core graphs, calculate `ProgramDigest`, alter planning, or affect runtime behaviour.

The purpose of CORE-1 is to establish the nominal semantic vocabulary that later packets can safely build upon.

## Relevant background and existing behaviour

Tethers currently has no Core type layer. The pipeline is:

```text
Human Tether source → parser (tether_parser.ml) → evaluator (tethers_evaluator.ml)
→ Runtime Plan JSON output.
```

The parser produces typed AST values (`tether`, `action`, `condition`, etc.) directly consumed by the evaluator. There is no intermediate canonical semantic representation (Core) between the Human Tether AST and the Runtime Plan.

The existing OCaml module graph in `bin/dune` lists two executables sharing six modules: `main`, `tethers_error`, `tethers_outcome`, `tether_parser`, `tethers_protocol`, `tethers_evaluator` for the direct engine; plus `tethers_mcp_main` and `tethers_mcp_server` for the MCP adapter. All modules currently compile and pass the full fixture/engine/MCP test suite.

A locked OCaml toolchain exists: OCaml 5.5.0, Dune 3.24.0, Yojson 2.2.2, Dune language 3.10.

The current branch (`feature/0.4-c1-together-fan-out-join`) at this base commit contains the accepted C1 Together execution work. The CORE-1 branch must be created cleanly from this base.

## Required behaviour

1. Create `tethers-0.1/engine-ocaml/bin/tethers_core.ml` and `tethers_core.mli` defining the Tethers Core semantic type vocabulary as a standalone, dormant OCaml module.
2. Define nominally distinct Core ID types with private constructors: `ProgramId`, `OriginId`, `FactId`, `RoleId`, `CapabilityId`, `BranchId`, `GroupId`, `BatchId`, `ItemTemplateId`. Each must be a distinct OCaml type (not string aliases).
3. Define Core primitive types: `terminal_outcome` (Success | Failure | Uncertain | Cancelled), `traversal_decision` (Continue | Stop), `fact_availability` (Optional | Guaranteed).
4. Define Origin Site species as a closed variant: Anchor Origin, Action Origin, Together Origin, Batch Origin. Branches and Roles must not be variants of Origin Site.
5. Define Fact with FactId, schema type description, and provenance (Origin provenance vs Role provenance proxy).
6. Define Action Origin with OriginId, CapabilityId, CapabilityContractDigest, input bindings, declared Facts, and Execution Constraints.
7. Define Input Binding types: literal value, Fact from Origin, Fact through Role, Batch Item Context. No Human Tethers wording.
8. Define Execution Constraint: Deadline (semantic duration/bound, no timer implementation).
9. Define Branch with BranchId, branch subject, and mapping from terminal outcome to semantic target.
10. Define Role with RoleId, scope (Program scope vs Item Template scope), Fact Contract, and eligible fulfillment. A Role is not an Origin.
11. Define Together Origin with GroupId, member OriginIds, and declared objective (ALL_MEMBERS_SUCCEED). Member order encodes no semantic dependency.
12. Define Batch with BatchId, collection provenance, owned ItemTemplateId, traversal policy, composite objective, and aggregate Facts.
13. Define Item Template as a scoped container independently containing Origin Sites, Branches, and Role Sites, plus an Item objective (REQUIRED_ROLE RoleId).
14. Define Batch Item Context as a static binding to the current item, structurally tied to its owning Batch/Item Template.
15. Define Program as the top-level static container with ProgramId, Core semantic version, static semantic definitions, and pinned capabilities/contracts.
16. Add `tethers_core` to both executable module lists in `bin/dune` so it compiles alongside existing modules.
17. Preserve byte-for-byte unchanged existing behaviour: no existing function consumes `Tethers_core` values, no existing module is modified, the full fixture/engine/MCP suite continues to pass.

## Relevant components

- `tethers-0.1/engine-ocaml/bin/tethers_core.ml` (new: Core type definitions and implementations)
- `tethers-0.1/engine-ocaml/bin/tethers_core.mli` (new: Core type interface with private constructors)
- `tethers-0.1/engine-ocaml/bin/dune` (modify: add `tethers_core` to both executables)
- `docs/CURRENT_CLINE_TASK.md` (packet, closeout scope)
- `docs/worker-notes/2026-08-11-core-1-ocaml-core-types.md` (worker note, closeout scope)
- Read-only references: all existing `bin/*.ml` modules, OCaml guide, specification

## Frozen decisions and invariants

- CORE-1 defines types only. No lowering, validation, serialization, `ProgramDigest`, or planning.
- The module remains dormant: no existing code consumes `Tethers_core` values.
- Nominal IDs must be distinct OCaml types, not string aliases. The invariant `OriginId ≠ RoleId ≠ FactId ≠ BatchId` must be enforced by the type system.
- Branches and Roles are separate semantic species, not Origin variants.
- Item Templates keep Origins, Branches, and Roles as structurally distinct lists (not collapsed into one).
- Existing externally observable behaviour must remain byte-for-byte unchanged.
- No OCaml modules other than `tethers_core.ml`, `tethers_core.mli`, and `dune` may change.
- No Rust, dependency, toolchain, fixture, or protocol changes.
- The terminal outcome type must be clearly separate from the existing `Tethers_outcome` module.
- Construction of nominal IDs requires explicit functions; accidental interchanging must be a type error.

## Acceptance criteria

1. `tethers_core.ml` and `tethers_core.mli` exist and compile as part of both executables.
2. Nine nominal ID types exist with private constructors and explicit `of_string`/`to_string` functions.
3. Each nominal ID is a distinct OCaml type; `origin_id = role_id` is a type error.
4. `terminal_outcome`, `traversal_decision`, and `fact_availability` types are defined and clearly separate from `Tethers_outcome`.
5. Origin Site is a closed variant with Anchor, Action, Together, and Batch constructors.
6. Branch and Role types exist independently; neither is a variant of Origin Site.
7. Fact type carries FactId, schema description, and provenance distinguishing Origin from Role proxy.
8. Action Origin carries OriginId, CapabilityId, CapabilityContractDigest, input bindings, Facts, and Execution Constraints.
9. Input Binding structurally distinguishes literal, fact-from-origin, fact-through-role, and batch-item-context.
10. Execution Constraint has a Deadline constructor.
11. Branch maps terminal outcomes to semantic targets.
12. Role carries RoleId, scope, Fact Contract, and eligible fulfillment.
13. Together Origin carries GroupId, member OriginIds, and ALL_MEMBERS_SUCCEED objective.
14. Batch carries BatchId, provenance, ItemTemplateId, traversal policy, composite objective, and aggregate Facts.
15. Item Template independently contains Origin Sites list, Branches list, and Role Sites list plus a REQUIRED_ROLE objective.
16. Batch Item Context is structurally tied to Item Template context.
17. Program is the top-level container with ProgramId, version, definitions, and capabilities.
18. `dune build` succeeds with the new module included.
19. Fixture suite passes: 64 JSON + 32 JSONL.
20. Engine suite passes: 32 cases.
21. MCP transcript suite passes: 16 cases.
22. `git diff --check` reports no whitespace issues.
23. Zero Rust or dependency changes confirmed by `git diff --stat`.
24. `git status --short` shows only authorised files changed.

## Required verification

1. Packet checker at start (`control-v1/IN_PROGRESS`):
   `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`
2. Rust formatter check (RUST_UNCHANGED; read-only):
   `cargo fmt --all -- --check`
3. OCaml build:
   `opam exec --switch="D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" -- dune build`
4. Fixture suite:
   `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\check-fixtures.ps1`
5. Engine suite:
   `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-engine.ps1`
6. MCP transcript suite:
   `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-mcp-transcripts.ps1`
7. Whitespace check:
   `git diff --check`
8. Complete diff inspection and `git status --short`.
9. On closeout: packet checker `control-v1/COMPLETE`.

Expected baselines: fixtures 64 JSON + 32 JSONL, engine 32 cases, MCP 16 cases.

## Forbidden changes

DO NOT modify any existing OCaml module (`tether_parser.ml`, `tether_parser.mli`, `tethers_evaluator.ml`, `tethers_evaluator.mli`, `tethers_protocol.ml`, `tethers_protocol.mli`, `tethers_outcome.ml`, `tethers_outcome.mli`, `tethers_error.ml`, `main.ml`, `tethers_mcp_main.ml`, `tethers_mcp_server.ml`).

DO NOT modify any Rust source, Cargo files, dependencies, opam files, toolchain configuration, fixtures, grammar, or protocol specifications.

DO NOT implement lowering, validation, serialization, `ProgramDigest`, canonicalisation, fingerprints, JSON encoding/decoding, Rust structs, batch execution, role runtime binding, or deadline runtime behaviour.

## Stop conditions

- After committing the implementation, stop. Do not begin CORE-2.
- If the base commit does not match the current branch's ancestor relationship, stop.
- If two materially similar implementation attempts fail on the same underlying problem, stop.
- If a required verification command produces unexpected output that cannot be resolved locally, stop.

## Expected pre-existing changes

None.
