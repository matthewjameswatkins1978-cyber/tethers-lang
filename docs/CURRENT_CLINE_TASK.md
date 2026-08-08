# Current Implementation Task

Control contract: `1`
Task: `F4a1 — OCaml Typed Evaluation Outcome Boundary`
Owner: `OpenCode`
Model: `DeepSeek Pro HIGH`
Status: `COMPLETE`
Task colour: `Amber`
Route: `OpenCode implements F4a1 OCaml typed evaluation outcome boundary; do not begin F4a2`
Worker note: `docs/worker-notes/2026-08-08-f4a1-ocaml-evaluation-outcome.md`
Base branch: `main`
Base commit: `5a3ce59a2d6840fb083b5b6ec1a405962e9cddd2`
Implementation branch: `foundation/f4a1-ocaml-evaluation-outcome`
OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`
Rust toolchain: read exact channel from `rust-toolchain.toml`; use plain Cargo (resolved by root pin); `--locked` mandatory
Toolchain preflight: `pwsh -NoProfile -File scripts/check-dev-tools.ps1`

## Objective

Replace direct construction of semantically significant Tethers planner JSON inside the OCaml evaluator with a small typed evaluation-result model and ONE exhaustive JSON encoder.

Preserve the exact existing Tethers 0.1 protocol behaviour.

## Relevant background and existing behaviour

The current `tethers_evaluator.ml` constructs JSON response envelopes directly via inner functions `response` and `contextual_error_response`. These construct `Yojson.Safe.t` using string statuses (`"matched"`, `"not_matched"`, `"error"`) within evaluation branches. The `evaluate_request` function returns `Yojson.Safe.t` directly, and `error_response` separately builds request-level error JSON. This means the evaluator leaves its typed OCaml domain at evaluation boundaries, permitting impossible semantic combinations.

## Relevant components

- `tethers-0.1/engine-ocaml/bin/tethers_evaluator.ml` — primary edit target
- `tethers-0.1/engine-ocaml/bin/tethers_mcp_server.ml` — caller update for new typed interface
- `tethers-0.1/scripts/test-engine.ps1` — protocol fixture validation
- `tethers-0.1/scripts/test-mcp-transcripts.ps1` — MCP transcript validation

## Required behaviour

1. Introduce semantic types (error_details, planned_action, trail_entry, plan, evaluation_context, status_payload, contextual_result, response) in tethers_evaluator.ml
2. Create json_of_response encoder
3. evaluate_request returns response type
4. error_response returns response type
5. Update process_line to use json_of_response
6. Update MCP server to use json_of_response
7. Preserve exact wire shapes for all four response variants
8. Preserve field ordering
9. Preserve empty-array rule
10. Preserve exception topology (protocol/language version checks still raise Tethers_error)

## Frozen decisions and invariants

- Do not change protocol wire shapes
- Do not modify tethers_protocol.ml or tether_parser.ml
- Do not change fixture files
- Do not begin Rust-side decoder
- Preserve exception topology

## Forbidden changes

- No production code changes outside tethers_evaluator.ml and tethers_mcp_server.ml
- No fixture changes
- No Rust changes
- No new dependencies

## Stop conditions

STOP if:
- Protocol fixtures fail after correct implementation
- Two materially similar attempts fail
- A contradiction between frozen wire contract and type model emerges

## Expected pre-existing changes

None

## Acceptance criteria

1. Semantic types (error_details, planned_action, plan, evaluation_context, status_payload, contextual_result, response) compile and are used in tethers_evaluator.ml
2. json_of_response encoder exists and is the single encoding path from response to Yojson.Safe.t
3. evaluate_request returns response type, not Yojson.Safe.t
4. error_response returns response type, not Yojson.Safe.t
5. process_line calls json_of_response on the typed response
6. MCP server calls Tethers_evaluator.json_of_response on the typed response
7. All four response variants produce exact wire shapes matching the frozen protocol contract
8. Field ordering matches the current JSON output
9. Empty arrays (required_effects, actions) serialize as `[]` not `null`
10. Protocol/language version mismatches still raise Tethers_error, caught as Request_error by callers

## Required verification

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
git diff --check
powShell -ExecutionPolicy Bypass -File tethers-0.1\scripts\test-engine.ps1
```
