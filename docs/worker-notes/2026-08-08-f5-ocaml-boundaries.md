# F5 — OCaml Semantic and Error Boundary Extraction

**Status:** COMPLETE
**Date:** 2026-08-08
**Owner:** OpenCode

## Git references

- Accepted base SHA: `9b5fdd47a885309ac04575065ba7cb0e6cf48693`
- Branch: `foundation/f5-ocaml-boundaries`
- Implementation checkpoint: `bcd0e09d4384b61d74cce4f5a5b823a237618eeb`

## Changed files

### NEW (6)
- `tethers-0.1/engine-ocaml/bin/tethers_error.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_error.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_outcome.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_outcome.mli`
- `tethers-0.1/engine-ocaml/bin/tether_parser.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_evaluator.mli`

### MODIFIED (6)
- `tethers-0.1/engine-ocaml/bin/tether_parser.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_protocol.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_evaluator.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_mcp_server.ml`
- `tethers-0.1/engine-ocaml/bin/main.ml`
- `tethers-0.1/engine-ocaml/bin/dune`

## Interfaces

### Tethers_error.mli
```ocaml
exception Tethers_error of string * string
val fail : string -> string -> 'a
```

### Tethers_outcome.mli
Transparent semantic model with `error_details`, `planned_action`, `trail_entry`, `plan`, `evaluation_context`, `status_payload`, `contextual_result`, `response`.
Exposes `error_response` and `json_of_response`.

### Tether_parser.mli
Transparent AST: `value`, `operator`, `condition`, `action`, `tether`.
Exposes `drop_prefix` and `parse_tether`.
No longer exposes `Tethers_error` or `fail`.

### Tethers_evaluator.mli
Single entrypoint: `val evaluate_request : Yojson.Safe.t -> Tethers_outcome.response`

## External parser-symbol inventory

Symbols used outside `tether_parser.ml`:

| Symbol | Used by |
|--------|---------|
| `value` type + constructors | protocol, evaluator |
| `operator` type + constructors | evaluator |
| `condition` type + fields | evaluator |
| `action` type + fields | evaluator |
| `tether` type + fields | evaluator, MCP server |
| `parse_tether` | evaluator, MCP server |
| `drop_prefix` | evaluator |
| `Tethers_error` exception | (moved to Tethers_error) |
| `fail` | (moved to Tethers_error) |

All required symbols exposed in `.mli`. No additional symbols needed.

## Ownership proofs

- **Shared Tethers_error ownership:** `tethers_error.ml` is the single definition site. Grep confirms only `tethers_error.ml:1` and `tethers_error.mli:1` declare `exception Tethers_error`. Parser no longer owns it.
- **`fail` ownership:** Single definition at `tethers_error.ml:3`.
- **Outcome type ownership:** `Tethers_outcome` is the single definition site for all response types.
- **JSON encoder ownership:** `json_of_response` defined only in `tethers_outcome.ml:38`.
- **process_line ownership:** Only in `main.ml:1`.
- **Evaluator entrypoint:** `.mli` has exactly 1 line: `evaluate_request`.

## Preserved invariants

- Evaluator input remains complete `Yojson.Safe.t` request JSON
- `evaluation_id` semantics unchanged
- `plan.id` generation unchanged (`evaluation_id ^ "/plan"`)
- `idempotency_key` generation unchanged (`evaluation_id ^ "/" ^ action_id`)
- Outcome types remain transparent (no abstract types, no smart constructors)
- `response` name preserved
- `json_of_response` name preserved
- `error_response` name preserved
- `process_line` moved to `main.ml`, exact behaviour preserved
- No `tethers_evaluation` module created
- No `tethers_types` module created
- No `tethers_protocol.mli` created
- No typed evaluator-input redesign

## Compatibility tests harvested

1. `pwsh -NoProfile -File scripts/check-fixtures.ps1` — validates 46 JSON + 30 JSONL fixture files
2. `cargo test --locked` — 1331 Rust host tests exercise engine integration
3. `dune build` — compiles both executables
4. `dune runtest` — 0 OCaml native tests (recorded honestly)

## Verification results

| Check | Result |
|-------|--------|
| `opam exec -- dune build` | PASS |
| `opam exec -- dune runtest` | PASS (0 native tests) |
| `check-fixtures.ps1` | PASS (46 JSON, 30 JSONL) |
| `cargo test --locked` | 1331 PASS, 0 FAIL, 2 ignored |
| `git diff --check` | PASS |
| `git diff --name-only -- tethers-0.1/host-rust/` | (empty) |
| `git diff --name-only -- tethers-0.1/protocol/` | (empty) |
| `rg "exception Tethers_error"` (in bin/) | Only `tethers_error.ml` + `.mli` |
| `rg "let fail "` (in bin/) | Only `tethers_error.ml` |
| `rg "process_line"` (in bin/) | Only `main.ml` |
| `rg "let json_of_response"` (in bin/) | Only `tethers_outcome.ml` |
| `rg "let error_response"` (in bin/) | Only `tethers_outcome.ml` |

## UNVERIFIED properties

None. All 13 acceptance criteria have hard proof.

## Defects found

None.

## Later Foundation phases

Not started. F6, F7, F8, F9, F10 not touched.

## Deferred decision recorded

"Typed/purified evaluator input was considered during architecture review and deliberately deferred because evaluation_id participates directly in deterministic plan/action identity generation."

## Smallest next action

Lucy reviews F5 evidence. F6 awaits compilation.
