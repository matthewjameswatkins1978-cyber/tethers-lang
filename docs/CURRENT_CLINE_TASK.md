# Current Implementation Task

Control contract: `1`

Task: `TETHERS CORE-1B — Named Inputs & Explicit Success Flow`

Owner: `OpenCode`

Status: `COMPLETE`

Task colour: `Green`

Route: `OpenCode implementation + evidence → Lucy independent GitHub review`

Worker note: `docs/worker-notes/2026-08-11-core-1b-named-inputs-success-flow.md`

Base branch: `feature/core-1-ocaml-core-types`

Base commit: `6295842688c7637172723ba46e43f128c3e86bc5`

Implementation checkpoint: `1011a644b3aa550c70643aaea33b7c2f301539b4`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

Rust toolchain: read exact channel from `rust-toolchain.toml`; use plain Cargo (resolved by root pin); `--locked` mandatory

Toolchain preflight: `pwsh -NoProfile -File scripts/check-dev-tools.ps1` (run; all tools present)

Rust change class: `RUST_UNCHANGED`

## Objective

Extend dormant `Tethers_core` so Action arguments are associated with their capability input names structurally and sequential Origin execution flow is explicitly stated, rather than derived from hidden list ordering.

CORE-1B fixes two type-level omissions discovered during CORE-2 preparation: `action_origin` stores bindings without retaining the capability input name, and Core has no explicit successful-continuation representation.

No lowering or runtime wiring occurs in this packet.

## Relevant background and existing behaviour

Tethers 0.1 names every Action argument: `file: anchor.document`, `copies: 2`. The parser emits these as a `(name * value) list` inside each `action`. The current CORE-1 `action_origin` collapses this to a bare `input_binding list`, discarding the argument name. CORE-2 cannot reconstruct the semantic association from position.

Tethers 0.1 evaluates Action Origins sequentially after the Anchor matches and all entry guards pass. Action evaluation order is source order. The current CORE-1 `program.origin_sites` is a list; without an explicit entry point and continuation structure, execution semantics are latent in list position.

The branch tip `6295842688c7637172723ba46e43f128c3e86bc5` contains the accepted CORE-1A implementation. CORE-1B continues on the same branch from that tip.

## Required behaviour

1. Introduce `capability_input_name` as a nominal type with a private constructor and `of_string`/`to_string` functions, matching the Core ID discipline.
2. Introduce `action_input = { input_name : capability_input_name; binding : input_binding }` associating each binding with its capability argument name.
3. Replace `action_origin.input_bindings : input_binding list` with `inputs : action_input list`.
4. Introduce `control_target = Origin_target of origin_id | Program_complete` as an explicit semantic continuation destination.
5. Introduce `success_continuation = { from_origin : origin_id; target : control_target }`, representing the ordinary successful path from one Origin to its successor.
6. Add `entry_origin : origin_id option` to `program`, naming the first executable semantic Origin.
7. Add `success_continuations : success_continuation list` to `program`.
8. Document in the interface that `origin_sites` ordering is representational storage only and MUST NOT determine runtime execution order.
9. Preserve byte-for-byte unchanged behaviour: no existing code consumes `Tethers_core` values, no existing OCaml module or Dune file is modified, and the full fixture/engine/MCP suite continues to pass.

## Relevant components

- `tethers-0.1/engine-ocaml/bin/tethers_core.ml` (modify: type definitions and implementations)
- `tethers-0.1/engine-ocaml/bin/tethers_core.mli` (modify: type interface with private constructors)
- `docs/CURRENT_CLINE_TASK.md` (packet, closeout scope)
- `docs/worker-notes/2026-08-11-core-1b-named-inputs-success-flow.md` (worker note, closeout scope)
- Read-only references: `tether_parser.ml` (action type shape), SPEC.md, OCaml guide

## Frozen decisions and invariants

- CORE-1B defines types and documentation only. No lowering, ID generation, success-edge validation, Branch validation, canonicalisation, ProgramDigest, JSON encoding, evaluator integration, Runtime Plan generation, Rust changes, Trail changes, Together execution, Roles, Batch behaviour, or Deadline behaviour.
- The module remains dormant: no existing code consumes `Tethers_core` values.
- Action meaning = Capability identity + named input bindings + contract digest + facts + constraints. Never "first item at position N probably means field N."
- Sequential meaning = explicit entry origin + explicit success continuation graph. Never "things in a list probably run in order."
- `success_continuation` defines only the ordinary successful path (SUCCESS). FAILURE, UNCERTAIN, CANCELLED are not represented here; their handling belongs to Branches or later semantics.
- Branch remains the explicit alternative-routing construct; CORE-1B does not replace Branch with generic outcome edges.
- Together semantics and internal member scheduling are not modified.
- Only `tethers_core.ml`, `tethers_core.mli`, packet, and worker note may change.
- `bin/dune` must not be modified unless an unforeseen compile issue genuinely requires it.

## Acceptance criteria

1. `capability_input_name` exists as a private nominal type with `of_string`/`to_string`.
2. `action_input` exists with `input_name : capability_input_name` and `binding : input_binding`.
3. `action_origin` has `inputs : action_input list` (no longer `input_bindings : input_binding list`).
4. `control_target` exists with `Origin_target` and `Program_complete` constructors.
5. `success_continuation` exists with `from_origin : origin_id` and `target : control_target`.
6. `program` has `entry_origin : origin_id option`.
7. `program` has `success_continuations : success_continuation list`.
8. Interface comments explicitly state that `origin_sites` list order carries no execution semantics.
9. `dune build` succeeds with the modified module.
10. Fixture suite passes: 64 JSON + 32 JSONL.
11. Engine suite passes: 32 cases.
12. MCP transcript suite passes: 16 cases.
13. `git diff --check` reports no whitespace issues.
14. Zero Rust or dependency changes confirmed by `git diff --stat`.
15. `git status --short` shows only authorised files changed.
16. No existing OCaml module other than `tethers_core.ml` and `tethers_core.mli` is modified.

## Required verification

1. Packet checker at start (`control-v1/IN_PROGRESS`):
   `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`
2. OCaml build:
   `opam exec --switch="D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" -- dune build`
3. Fixture suite:
   `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\check-fixtures.ps1`
4. Engine suite:
   `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-engine.ps1`
5. MCP transcript suite:
   `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-mcp-transcripts.ps1`
6. Whitespace check:
   `git diff --check`
7. Complete diff inspection and `git status --short`.
8. On closeout: packet checker `control-v1/COMPLETE`.

Rust suite is not required: Rust source and dependencies remain untouched.

Expected baselines: fixtures 64 JSON + 32 JSONL, engine 32 cases, MCP 16 cases.

## Forbidden changes

DO NOT modify any existing OCaml module other than `tethers_core.ml` and `tethers_core.mli` (`tether_parser.ml`, `tether_parser.mli`, `tethers_evaluator.ml`, `tethers_evaluator.mli`, `tethers_protocol.ml`, `tethers_protocol.mli`, `tethers_outcome.ml`, `tethers_outcome.mli`, `tethers_error.ml`, `main.ml`, `tethers_mcp_main.ml`, `tethers_mcp_server.ml`).

DO NOT modify `bin/dune` unless an unforeseen compile issue genuinely requires it.

DO NOT modify any Rust source, Cargo files, dependencies, opam files, toolchain configuration, fixtures, grammar, or protocol specifications.

DO NOT implement Human AST → Core lowering, automatic ID generation, success-edge validation, Branch validation, canonicalisation, ProgramDigest, JSON encoding, evaluator integration, Runtime Plan generation, Rust changes, Trail changes, Together execution, Roles, Batch behaviour, or Deadline behaviour.

## Stop conditions

- After committing CORE-1B, stop. Do not begin CORE-2.
- If the base commit does not match the current branch's ancestor relationship, stop.
- If two materially similar implementation attempts fail on the same underlying problem, stop.
- If a required verification command produces unexpected output that cannot be resolved locally, stop.

## Expected pre-existing changes

None.
