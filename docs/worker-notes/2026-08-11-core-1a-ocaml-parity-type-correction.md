# Worker Note — TETHERS CORE-1A

Task: `TETHERS CORE-1A — Current-Language Parity Type Correction`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `5e6a9826cfede4646fbea82a0d310ed0b3f5e60b`

Implementation checkpoint: `c82e93604f10abad389d2ee17d34e8618f4d8383`

## Requested outcome

Extend the dormant `Tethers_core` OCaml vocabulary so the typed literal values,
immutable host-supplied evaluation-input Facts and Conditions over them, and
structured `anchor.*` event-data bindings already supported by Tethers 0.1 each
have a lossless static type home in Core. CORE-1A adds types only: no lowering,
no guard evaluation, no Anchor path resolution, no evaluator wiring, and no
behavioural change.

## Changes made

- `tethers-0.1/engine-ocaml/bin/tethers_core.ml` — added `host_snapshot_key`
  with `host_snapshot_key_of_string` / `string_of_host_snapshot_key`; added
  `core_scalar_type` and `core_value` variants; extended `fact_provenance`
  with `Evaluation_input of host_snapshot_key * core_scalar_type`; added
  `comparison_operator` and `fact_guard`; changed `input_binding` so
  `Literal_value` carries `core_value` and added
  `Anchor_value of origin_id * string list`; added `input_facts : fact list`
  and `entry_guards : fact_guard list` to `program`.
- `tethers-0.1/engine-ocaml/bin/tethers_core.mli` — mirror declarations with
  private `host_snapshot_key` constructor and conversion functions; documented
  the CORE-1A extension in the module header.
- `docs/CURRENT_CLINE_TASK.md` — replaced the CORE-1 packet with the CORE-1A
  packet; set to `IN_PROGRESS`, then `COMPLETE` at closeout.
- `docs/worker-notes/2026-08-11-core-1a-ocaml-parity-type-correction.md` — this
  note.

No existing OCaml module other than `tethers_core.ml` and `tethers_core.mli`
was modified. `bin/dune` was not modified.

## Decisions and assumptions

- `core_value` uses constructors `String_value | Integer_value | Boolean_value`
  mirroring the parser's value constructors but as Core's own nominal type;
  integers and booleans are never stringified.
- The evaluation-input form is a `fact_provenance` constructor carrying a
  nominal `host_snapshot_key` and a `core_scalar_type`, so an evaluation-input
  Fact remains a `fact` with a `fact_id` while being statically distinct from
  Origin provenance and Role proxy.
- `fact_guard` mirrors the 0.1 Condition shape (`fact_id`, `operator`,
  `expected : core_value`) using Core vocabulary; `comparison_operator`
  `Equals | Contains | Greater_than | Greater_than_or_equal` corresponds to
  0.1 `is | contains | greater_than | greater_than_or_equal`.
- `Anchor_value of origin_id * string list` stores event-data path components
  structurally (e.g. `["customer"; "id"]`), drops the `anchor.` textual prefix,
  and stores no resolved runtime value.
- `program` gained `input_facts` (explicit immutable evaluation-input Fact
  declarations) and ordered `entry_guards`; field placement in the record is
  not semantically significant.

## Evidence

- Packet checker at start: `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` — PASS (`control-v1/IN_PROGRESS`, base `5e6a982`, HEAD `5e6a982`).
- OCaml build: `opam exec --switch="D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" -- dune build` — PASS (no output, exit 0).
- Fixture suite: `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\check-fixtures.ps1` — PASS (64 JSON files, 32 JSONL files).
- Engine suite: `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-engine.ps1` — PASS (32 fixture cases plus happy-path, together, and line-ending determinism checks).
- MCP transcript suite: `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-mcp-transcripts.ps1` — PASS (16 cases).
- Whitespace check: `git diff --check` — PASS (no whitespace errors; only informational LF-to-CRLF working-copy warnings).
- Rust formatter check (RUST_UNCHANGED, read-only): `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check` — PASS (exit 0).
- `git diff --stat` — only the two implementation files plus the packet; zero Rust/dependency changes.
- Working tree implementation files match checkpoint `c82e93604f10abad389d2ee17d34e8618f4d8383` exactly (empty `git diff` for the two files).
- Packet checker at closeout: `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` — PASS (`control-v1/COMPLETE`).

## Publication evidence

Branch pushed: `origin/feature/core-1-ocaml-core-types`. Full remote HEAD SHA:
`c82e93604f10abad389d2ee17d34e8618f4d8383` (resolved after the normal push).
Local `HEAD == remote HEAD`: confirmed. Final `git status --short --branch`:
clean.

## Discoveries

- None within packet scope. The parser/evaluator/protocol modules were not
  inspected for implementation purposes beyond confirming the existing 0.1
  value, operator, and Condition shapes used as parity references.

## Remaining risks

- None known within packet scope. `fact_guard` evaluation, Anchor path
  resolution, Core validation, JSON encoding, and `ProgramDigest` remain
  explicitly deferred and are not partially implemented here.

## Smallest next action

CORE-2: lower the Human Tether AST into these Core types (parser → Core
`program`), reusing `core_value`, `Evaluation_input`, `fact_guard`, and
`Anchor_value` without changing the existing parser/evaluator runtime path.

## References

- `tethers-0.1/engine-ocaml/bin/tethers_core.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core.mli`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/OCAML_GUIDE_FOR_AGENTS.md`
- `tethers-0.1/SPEC.md` sections 4–5 (scalar values, operators, `anchor.*`)
- Implementation checkpoint commit `c82e93604f10abad389d2ee17d34e8618f4d8383`
- Base commit `5e6a9826cfede4646fbea82a0d310ed0b3f5e60b` (CORE-1 closeout)
