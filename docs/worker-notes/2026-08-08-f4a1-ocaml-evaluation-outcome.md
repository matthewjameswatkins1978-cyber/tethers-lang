# Worker Note

Task: `F4a1 — OCaml Typed Evaluation Outcome Boundary`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `5a3ce59a2d6840fb083b5b6ec1a405962e9cddd2`

Implementation checkpoint: `6326e5672b1bd34cc3054a9b42488727de61b7e1`

## Requested outcome

Replace direct construction of semantically significant Tethers planner JSON inside the OCaml evaluator with a small typed evaluation-result model and ONE exhaustive JSON encoder. Preserve exact existing Tethers 0.1 protocol behaviour.

## Changes made

- `tethers-0.1/engine-ocaml/bin/tethers_evaluator.ml` — Introduced typed evaluation outcome model (error_details, planned_action, plan, evaluation_context, status_payload, contextual_result, response). Replaced inner JSON-constructing `response` and `contextual_error_response` functions with direct construction of typed `response` values. Changed `evaluate_request` return from `Yojson.Safe.t` to `response`. Changed `error_response` return from `Yojson.Safe.t` to `response`. Added `json_of_response` as the single exhaustive encoder from `response` to `Yojson.Safe.t`. Updated `process_line` to compose through `json_of_response`.
- `tethers-0.1/engine-ocaml/bin/tethers_mcp_server.ml` — Updated `handle_tools_call` to call `Tethers_evaluator.json_of_response` on the typed `response` before serialization.
- `docs/CURRENT_CLINE_TASK.md` — Updated to F4a1 task packet.
- `docs/worker-notes/2026-08-08-f4a1-ocaml-evaluation-outcome.md` — This worker note.

## Decisions and assumptions

- Kept `condition_result` and `action_planning_result` intermediate types unchanged.
- `type trail_entry = Yojson.Safe.t` and the existing `let trail_entry` function coexist; OCaml has separate namespaces for type names and value names. The frozen F4a1 `trail_entry` alias is now implemented and `contextual_result.trail` uses `trail_entry list`.
- Preserved exception topology exactly: protocol/language version checks still raise `Tethers_error` caught as `Request_error` by callers.
- Field ordering in `json_of_response` matches the frozen protocol contract (contextual fields before error, trail last, etc.).

## Evidence

- `dune build` — PASS (no warnings)
- `test-engine.ps1` — PASS (28/28 cases, including deterministic repeat, LF/CRLF)
- `test-mcp-transcripts.ps1` — PASS (15/15 cases)
- `check-tethers-task-packet.ps1` — PASS (control-v1/COMPLETE)
- `git diff --check` — PASS (whitespace clean)
- Committed implementation checkpoint `6326e5672b1bd34cc3054a9b42488727de61b7e1` contains 4 changed files from the accepted F3 base: 2 production (`tethers_evaluator.ml`, `tethers_mcp_server.ml`) and 2 documentation (`CURRENT_CLINE_TASK.md`, worker note)

## Discoveries

- None.

## Remaining risks

- None known within packet scope.

## Smallest next action

F4a2: Rust-side typed response decoder boundary (DO NOT BEGIN without separate packet).

## References

- `tethers-0.1/engine-ocaml/bin/tethers_evaluator.ml` — modified
- `tethers-0.1/engine-ocaml/bin/tethers_mcp_server.ml` — modified
- `docs/CURRENT_CLINE_TASK.md` — F4a1 task packet
- `docs/worker-notes/2026-08-08-f4a1-ocaml-evaluation-outcome.md` — this note
- Branch: `foundation/f4a1-ocaml-evaluation-outcome`
- Implementation checkpoint: `6326e5672b1bd34cc3054a9b42488727de61b7e1`
- Base commit: `5a3ce59a2d6840fb083b5b6ec1a405962e9cddd2`
