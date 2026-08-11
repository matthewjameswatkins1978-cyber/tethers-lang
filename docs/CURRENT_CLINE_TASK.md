# Current Implementation Task

Control contract: `1`

Task: `TETHERS CORE-1A — Current-Language Parity Type Correction`

Owner: `OpenCode`

Status: `COMPLETE`

Task colour: `Green`

Route: `OpenCode implementation + evidence → Lucy independent GitHub review`

Worker note: `docs/worker-notes/2026-08-11-core-1a-ocaml-parity-type-correction.md`

Base branch: `feature/core-1-ocaml-core-types`

Base commit: `5e6a9826cfede4646fbea82a0d310ed0b3f5e60b`

Implementation checkpoint: `c82e93604f10abad389d2ee17d34e8618f4d8383`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

Rust toolchain: read exact channel from `rust-toolchain.toml`; use plain Cargo (resolved by root pin); `--locked` mandatory

Toolchain preflight: `pwsh -NoProfile -File scripts/check-dev-tools.ps1` (run; all tools present)

Rust change class: `RUST_UNCHANGED`

## Objective

Extend the dormant `Tethers_core` semantic vocabulary so every currently supported sequential Tether construct has a lossless semantic home in Core.

CORE-1A is a packet/design correction: CORE-1 omitted three semantics already present in Tethers 0.1 — typed literal values, immutable host-supplied Facts and Conditions over them, and structured `anchor.*` event-data bindings used by Action arguments.

This task adds static Core types only. No lowering. No evaluator wiring. No behavioural changes.

## Relevant background and existing behaviour

Tethers 0.1 supports three scalar values in Conditions and Action arguments: quoted strings, integers, and `true`/`false`. Conditions are evaluated against an immutable host-supplied Fact snapshot using four operators: `is`, `contains`, `greater_than`, `greater_than_or_equal`. Action arguments may reference structured event-data fields through dotted `anchor.*` paths (for example `anchor.customer.id`).

The current CORE-1 `Tethers_core` types cannot represent these semantics without loss:

- `input_binding` uses `Literal_value of string`, stringifying every literal;
- `fact_provenance` has only Origin provenance and Role proxy, with no evaluation-input form;
- there is no Condition/guard vocabulary in Core;
- `input_binding` has no structured Anchor event-data reference.

The existing parser (`tether_parser.ml`) defines `value = String_value of string | Int_value of int | Bool_value of bool | Reference of string` and `operator = Is | Contains | Greater_than | Greater_than_or_equal`; the evaluator resolves `anchor.*` references against the supplied event envelope. CORE-1A must make these existing semantics statically representable in Core.

The branch tip `5e6a9826cfede4646fbea82a0d310ed0b3f5e60b` contains the accepted CORE-1 implementation. CORE-1A continues on the same branch from that tip.

## Required behaviour

1. Replace the string-only Core literal with a nominally typed `core_value` = `String_value of string | Integer_value of int | Boolean_value of bool`, and change `input_binding`'s `Literal_value` to carry `core_value`.
2. Extend `fact_provenance` with an explicit evaluation-input form that carries a host snapshot key and a typed scalar declaration, keeping Origin provenance and Role proxy distinct.
3. Add a `host_snapshot_key` nominal type with a private constructor and `of_string`/`to_string` functions, matching the Core ID discipline.
4. Add `core_scalar_type` = `String_type | Integer_type | Boolean_type` as the static type declaration for evaluation-input Facts.
5. Add `comparison_operator` = `Equals | Contains | Greater_than | Greater_than_or_equal` and a `fact_guard` record carrying `fact_id`, `operator`, and `expected : core_value`.
6. Add `input_facts : fact list` to `program` as the explicit home for immutable evaluation-input Fact declarations.
7. Add `entry_guards : fact_guard list` to `program` as an ordered list preserving declared evaluation order.
8. Extend `input_binding` with `Anchor_value of origin_id * string list`, storing event-data path components structurally without the literal `anchor.` prefix and without any resolved runtime value.
9. Preserve byte-for-byte unchanged behaviour: no existing code consumes `Tethers_core` values, no existing OCaml module or Dune file is modified, and the full fixture/engine/MCP suite continues to pass.

## Relevant components

- `tethers-0.1/engine-ocaml/bin/tethers_core.ml` (modify: Core type definitions and implementations)
- `tethers-0.1/engine-ocaml/bin/tethers_core.mli` (modify: Core type interface with private constructors)
- `docs/CURRENT_CLINE_TASK.md` (packet, closeout scope)
- `docs/worker-notes/2026-08-11-core-1a-ocaml-parity-type-correction.md` (worker note, closeout scope)
- Read-only references: `tether_parser.ml`, `tethers_evaluator.ml`, SPEC.md, OCaml guide

## Frozen decisions and invariants

- CORE-1A defines types only. No lowering, guard evaluation, Anchor path resolution, Core validation, type-checking algorithms, Definite Fact Availability, provenance DAG validation, JSON encoding, canonicalisation, `ProgramDigest`, or planning.
- The module remains dormant: no existing code consumes `Tethers_core` values.
- Core must clearly distinguish, and none may be an ambiguous raw string: literal value, evaluation-input Fact, Fact from Origin, Fact through Role, Anchor event-data binding, Batch Item Context.
- Nominal IDs and semantic key types remain distinct OCaml types with private constructors; accidental interchanging is a type error.
- Entry guards preserve declared order because current evaluation and Trail behaviour are ordered.
- `core_value` never stringifies integers or booleans.
- No Human Tethers source wording is carried into Core.
- Only `tethers_core.ml`, `tethers_core.mli`, the packet, and the worker note may change.
- `bin/dune` must not be modified unless an unforeseen compile issue genuinely requires it.
- Existing advanced placeholders (`batch_collection_provenance`, `batch_traversal_policy`, `batch_objective`, `role_fulfillment`) remain untouched and are not authorised as final semantic encodings.

## Acceptance criteria

1. `core_value` exists with `String_value`, `Integer_value`, and `Boolean_value` constructors.
2. `Literal_value` carries `core_value`; integers and booleans are not stringified.
3. `fact_provenance` distinguishes evaluation input, Origin provenance, and Role proxy.
4. The evaluation-input form carries a host snapshot key and a typed scalar declaration.
5. `host_snapshot_key` is a distinct nominal type with a private constructor and `of_string`/`to_string` functions.
6. `core_scalar_type` exists with `String_type`, `Integer_type`, and `Boolean_type` constructors.
7. `comparison_operator` exists with `Equals`, `Contains`, `Greater_than`, and `Greater_than_or_equal` constructors.
8. `fact_guard` exists and carries `fact_id`, `operator`, and `expected : core_value`.
9. `program` carries `input_facts : fact list`.
10. `program` carries `entry_guards : fact_guard list` (ordered).
11. `input_binding` carries `Anchor_value of origin_id * string list`; no `anchor.` prefix is preserved and no resolved runtime value is stored.
12. `dune build` succeeds with the modified module.
13. Fixture suite passes: 64 JSON + 32 JSONL.
14. Engine suite passes: 32 cases.
15. MCP transcript suite passes: 16 cases.
16. `git diff --check` reports no whitespace issues.
17. Zero Rust or dependency changes confirmed by `git diff --stat`.
18. `git status --short` shows only authorised files changed.
19. No existing OCaml module other than `tethers_core.ml` and `tethers_core.mli` is modified.

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

DO NOT implement Human AST → Core lowering, guard evaluation, Anchor path resolution, Core validation, type-checking algorithms, Definite Fact Availability, provenance DAG validation, JSON encoding, canonicalisation, `ProgramDigest`, Rust ingestion, Runtime Plan changes, Trail changes, Roles, Batch behaviour, Deadline behaviour, or C2.

## Stop conditions

- After committing the CORE-1A implementation, stop. Do not begin CORE-2.
- If the base commit does not match the current branch's ancestor relationship, stop.
- If two materially similar implementation attempts fail on the same underlying problem, stop.
- If a required verification command produces unexpected output that cannot be resolved locally, stop.

## Expected pre-existing changes

None.
