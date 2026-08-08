# Current Implementation Task

Control contract: `1`
Task: `F5 — OCaml Semantic and Error Boundary Extraction`
Owner: `OpenCode`
Model: `DeepSeek Pro HIGH`
Status: `COMPLETE`
Task colour: `Amber`
Route: `OpenCode implements OCaml module ownership extraction; no semantic redesign`
Worker note: `docs/worker-notes/2026-08-08-f5-ocaml-boundaries.md`
Base branch: `foundation/f4b-direct-execution-outcome`
Base commit: `9b5fdd47a885309ac04575065ba7cb0e6cf48693`
Implementation branch: `foundation/f5-ocaml-boundaries`
Implementation checkpoint: `bcd0e09d4384b61d74cce4f5a5b823a237618eeb`
OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`
Rust toolchain: `N/A`

## Objective

Perform the bounded Foundation F5 structural extraction: make existing ownership boundaries visible in the OCaml module structure. No product capability, no semantic redesign, no protocol migration, no Rust changes.

## Relevant background and existing behaviour

F2-F4 stabilised the Tethers 0.1 semantic contracts: evaluation, response JSON, Trail, plan identity, and idempotency_key generation. F5 is a pure structural extraction from that stable base.

Previously `tether_parser.ml` owned the engine-wide `exception Tethers_error` and `fail` helper, creating incidental dependency on a leaf parsing module. `tethers_evaluator.ml` owned evaluation, outcome domain types, JSON encoding, error construction, and stdin transport — too many distinct responsibilities.

F5 extracts the stable error boundary to `Tethers_error` and the stable outcome boundary to `Tethers_outcome`, adds `.mli` interfaces to enforce ownership, and moves transport to `main.ml`. No semantic changes.

## Required behaviour

1. Create `Tethers_error` module owning the engine-wide exception and fail helper, extracted from `Tether_parser`.
2. Create `Tethers_outcome` module owning response types, JSON encoder, and error_response, extracted from `Tethers_evaluator`.
3. Remove error ownership from `Tether_parser`.
4. Remove outcome ownership from `Tethers_evaluator`.
5. Move `process_line` from `Tethers_evaluator` to `main.ml`.
6. Create `Tether_parser.mli` exposing parser AST + parse_tether + drop_prefix.
7. Create `Tethers_evaluator.mli` exposing only evaluate_request.
8. Update `tethers_protocol.ml` to use `Tethers_error` for fail.
9. Update `tethers_mcp_server.ml` to use `Tethers_outcome` for error_response and json_of_response.
10. Update `dune` to include new modules.
11. Preserve all existing JSON/output/error semantics.
12. Zero Rust changes.
13. Zero fixture changes.

## Relevant components

### NEW
- `tethers-0.1/engine-ocaml/bin/tethers_error.ml` — engine-wide exception + fail
- `tethers-0.1/engine-ocaml/bin/tethers_error.mli` — interface
- `tethers-0.1/engine-ocaml/bin/tethers_outcome.ml` — response types + JSON encoder + error_response
- `tethers-0.1/engine-ocaml/bin/tethers_outcome.mli` — transparent interface
- `tethers-0.1/engine-ocaml/bin/tether_parser.mli` — transparent AST surface
- `tethers-0.1/engine-ocaml/bin/tethers_evaluator.mli` — single-entrypoint interface

### MODIFIED
- `tethers-0.1/engine-ocaml/bin/tether_parser.ml` — removed exception/fail, opens Tethers_error
- `tethers-0.1/engine-ocaml/bin/tethers_protocol.ml` — opens Tethers_error
- `tethers-0.1/engine-ocaml/bin/tethers_evaluator.ml` — removed outcome types/encoder/transport, opens Tethers_outcome + Tethers_error
- `tethers-0.1/engine-ocaml/bin/tethers_mcp_server.ml` — uses Tethers_outcome.error_response and Tethers_outcome.json_of_response
- `tethers-0.1/engine-ocaml/bin/main.ml` — now owns process_line
- `tethers-0.1/engine-ocaml/bin/dune` — adds tethers_error and tethers_outcome to both executables

## Frozen decisions and invariants

- `Tethers_error` is the shared owner of engine-wide exception and fail.
- `Tethers_outcome` is the shared owner of response types and JSON encoder.
- `Tethers_evaluator` exposes only `evaluate_request` in its interface.
- No typed evaluator-input redesign; `evaluation_id` semantics preserved.
- `plan.id` and `idempotency_key` generation unchanged.
- Outcome types remain transparent; no abstract types or smart constructors.
- `json_of_response` and `error_response` names preserved.
- No `tethers_evaluation`, `tethers_response`, `tethers_types` modules.
- No functors, module types, or abstraction layers.
- Zero Rust changes; zero fixture changes.

## Acceptance criteria

1. New OCaml interfaces compile — `dune build` PASS
2. Parser no longer defines `Tethers_error` — grep confirms only `tethers_error.ml`/`.mli`
3. `Tethers_error` is shared owner — single definition site
4. `Tethers_outcome` owns response types — single definition site
5. `Tethers_outcome` owns JSON encoder — `json_of_response` only in `tethers_outcome.ml`
6. `Tethers_evaluator` exposes only `evaluate_request` — `.mli` has 1 line
7. `process_line` no longer in evaluator — grep confirms only in `main.ml`
8. Legacy line engine output unchanged — all fixtures valid (46 JSON + 30 JSONL)
9. MCP output unchanged — all fixtures valid
10. Response JSON expectations unchanged — zero fixture diffs
11. Rust host tests pass — `cargo test --locked` 1331 PASS, 0 FAIL, 2 ignored
12. No Rust file changed — diff confirms zero
13. No compatibility fixture changed — diff confirms zero

## Required verification

- `opam exec -- dune build`: PASS
- `opam exec -- dune runtest`: PASS (0 OCaml native tests; engine behaviour covered by integration scripts + Rust host tests)
- `pwsh -NoProfile -File scripts/check-fixtures.ps1`: 46 JSON + 30 JSONL valid
- `cargo test --locked`: 1331 PASS, 0 FAIL, 2 ignored
- `git diff --check`: PASS
- `check-tethers-task-packet.ps1`: pending closeout

## Forbidden changes confirmed not made

- No Tethers language syntax changes
- No evaluator semantics changes
- No request/response JSON changes
- No JSON field ordering changes
- No error code/message changes
- No variant/field renames
- No `tethers_evaluation`, `tethers_response`, `tethers_types` modules
- No functors, module types, or abstraction
- No typed evaluator-input redesign (evaluation_id semantics preserved)
- No Rust changes
- No fixture changes
- No F6+ work started

## Stop conditions

NONE triggered.

## Expected pre-existing changes

1. Implementation checkpoint `bcd0e09d` covers all production/build changes.
2. Closeout docs (`CURRENT_CLINE_TASK.md`, worker note) are the only files after checkpoint.
