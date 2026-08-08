# Worker Note — F5 OCaml Semantic and Error Boundary Extraction

Task: `F5 — OCaml Semantic and Error Boundary Extraction`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `9b5fdd47a885309ac04575065ba7cb0e6cf48693`
Implementation checkpoint: `bcd0e09d4384b61d74cce4f5a5b823a237618eeb`

## Requested outcome

Extract the stable error boundary to `Tethers_error` and the stable outcome boundary to `Tethers_outcome`, add `.mli` interfaces to enforce ownership, move transport to `main.ml`, with no product capability, no semantic redesign, no protocol migration, no Rust changes.

## Changes made

### New modules (6)
- `tethers_error.ml` + `.mli` — engine-wide `exception Tethers_error` and `fail` helper
- `tethers_outcome.ml` + `.mli` — response types (`error_details`, `planned_action`, `trail_entry`, `plan`, `evaluation_context`, `status_payload`, `contextual_result`, `response`), `json_of_response`, `error_response`
- `tether_parser.mli` — transparent AST: `value`, `operator`, `condition`, `action`, `tether`, `drop_prefix`, `parse_tether`
- `tethers_evaluator.mli` — single line: `val evaluate_request : Yojson.Safe.t -> Tethers_outcome.response`

### Modified modules (6)
- `tether_parser.ml` — removed `exception Tethers_error` and `fail`; added `open Tethers_error`; `fail` now resolved from shared module
- `tethers_protocol.ml` — added `open Tethers_error` for `fail`
- `tethers_evaluator.ml` — removed outcome types (37 lines), `json_of_response` (50 lines), `error_response` (1 line), `process_line` (14 lines); added `open Tethers_outcome` and `open Tethers_error`; internal types `condition_result` and `action_planning_result` preserved
- `tethers_mcp_server.ml` — added `open Tethers_error`; `Tethers_evaluator.error_response` → `Tethers_outcome.error_response`; `Tethers_evaluator.json_of_response` → `Tethers_outcome.json_of_response`
- `main.ml` — now owns `process_line` with exact same catch logic; calls `Tethers_evaluator.evaluate_request`, uses `Tethers_outcome.*`
- `dune` — added `tethers_error` and `tethers_outcome` to both executable module lists

## Decisions and assumptions

- Evaluator input model preserved: complete `Yojson.Safe.t` -> `Tethers_outcome.response`. No typed evaluator-input redesign. Reason: `evaluation_id` participates directly in deterministic `plan.id` and `idempotency_key` generation.
- Outcome types remain transparent (no abstract types, no smart constructors). The evaluator is the legitimate producer; exhaustive variants and structural records are useful compiler-visible contracts.
- `condition_result` and `action_planning_result` remain internal to the evaluator — they are implementation detail of `check_conditions` and `plan_actions`, not the public outcome contract.
- No `tethers_protocol.mli` created. Compilation contradiction did not arise.
- No OCaml native tests exist in the repository. Engine and MCP behaviour is covered by `test-engine.ps1` (23 cases + determinism + line-ending), `test-mcp-transcripts.ps1` (15 cases), and Rust host tests (1331 cases).

## Evidence

| Check | Result |
|-------|--------|
| `opam exec -- dune build` | PASS |
| `opam exec -- dune runtest` | PASS (0 native tests) |
| `test-engine.ps1` | PASS (23 fixture cases + determinism repeat + LF/CRLF/mixed validation) |
| `test-mcp-transcripts.ps1` | PASS (15 MCP transcript cases) |
| `check-fixtures.ps1` | PASS (46 JSON + 30 JSONL; fixture-integrity only) |
| `cargo test --locked` | 1331 PASS, 0 FAIL, 2 ignored |
| `git diff --check` | PASS |
| `rg "exception Tethers_error"` in bin/ | Only `tethers_error.ml` + `.mli` |
| `rg "let fail "` in bin/ | Only `tethers_error.ml` |
| `rg "process_line"` in bin/ | Only `main.ml` |
| `rg "let json_of_response"` in bin/ | Only `tethers_outcome.ml` |
| `rg "let error_response"` in bin/ | Only `tethers_outcome.ml` |
| `tethers_evaluator.mli` line count | 1 line |
| `git diff --name-only -- tethers-0.1/host-rust/` | (empty) |
| `git diff --name-only -- tethers-0.1/protocol/` | (empty) |

`check-fixtures.ps1` validates that JSON/JSONL fixtures are syntactically well-formed; it does not execute the engine or MCP server. `test-engine.ps1` and `test-mcp-transcripts.ps1` provide direct behavioural evidence that the extracted engine/MCP code produces identical output.

All 13 acceptance criteria have hard proof. No expected JSON changed. No fixtures changed. No Rust changed.

## Discoveries

- The OCaml module system handles transparent type re-exports cleanly — opening `Tethers_outcome` in the evaluator brings all variant constructors (`Contextual`, `Request_error`, `Matched`, `Not_matched`, `Evaluation_error`) into scope without qualification changes in `evaluate_request`.
- `drop_prefix` is used by the evaluator for `anchor.*` reference resolution and must remain in the parser's public interface.
- `unique` helper stayed in the evaluator — it is internal to plan construction.

## Remaining risks

None. Pure structural extraction with zero semantic or output changes, proven by identical fixture and test results.

## Smallest next action

Lucy reviews F5 evidence. F6 awaits compilation.

## References

- Implementation branch: `foundation/f5-ocaml-boundaries`
- Implementation checkpoint: `bcd0e09d4384b61d74cce4f5a5b823a237618eeb`
- Base: F4b accepted tip `9b5fdd47a885309ac04575065ba7cb0e6cf48693`
- F5 task specification: inline user instructions
