# Current Implementation Task

Control contract: `1`

Task: `TETHERS CORE-9C - Canonical Core Production Cutover`

Owner: `OpenCode`

Implementation checkpoint: `227f54f70a18b80abc498f3ac8ba26edffc82465`

Status: `COMPLETE`

Task colour: `Amber`

Route: `OpenCode implementation + evidence, Lucy independent GitHub review`

Worker note: `docs/worker-notes/2026-08-12-core-9c-production-cutover.md`

Base branch: `feature/core-9b-cross-language-rehearsal`

Base commit: `adec68156481fc319de44bf686bcf8d1ef65263b`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

Rust change class: `RUST_CHANGED`

## Objective

Put the accepted canonical Core into the real production evaluation path.
After this packet there must be ONE production evaluation route:

Human request
→ explicit Core semantic environment
→ CORE-8B
→ canonical Core
→ Runtime Plan
→ existing Rust policy / replay / dispatch / Trail machinery

No legacy planning fallback.

## Relevant background and existing behaviour

CORE-9B established the cross-language rehearsal boundary with
tethers.evaluate_core as a temporary MCP tool. The MCP server currently
has tethers.evaluate on legacy evaluator and tethers.evaluate_core on Core.

## Required behaviour

1. Switch MCP tethers.evaluate to Core (tethers_core_wire)
2. Remove rehearsal tethers.evaluate_core from MCP tools/list and tools/call
3. Remove Tethers_evaluator from MCP dune module list
4. Switch standalone tethers_engine to Core via tethers_core_wire
5. Switch Rust production request construction to build_core_request_envelope
6. Remove evaluate_tether_core from EngineSession
7. Update CORE-9B tests for cutover reality (2 tools, all Core)

## Relevant components

- `tethers-0.1/engine-ocaml/bin/tethers_mcp_server.ml` -- modified
- `tethers-0.1/engine-ocaml/bin/main.ml` -- modified
- `tethers-0.1/engine-ocaml/bin/dune` -- modified
- `tethers-0.1/host-rust/src/engine_stdio.rs` -- modified
- `tethers-0.1/host-rust/src/host_execution.rs` -- modified

## Frozen decisions and invariants

- tethers.evaluate now uses Core via tethers_core_wire
- tethers.evaluate_core is removed (no dual evaluation routes)
- EngineSession::evaluate_tether_core is removed
- build_core_request_envelope is the production request builder
- Core contract digest != manifest digest; do not compare or equate
- The Runtime Plan in the response is canonical_plan.runtime_plan
- program_digest comes only from canonical_plan.program_digest
- Empty trail is compatibility scaffolding only

## Acceptance criteria

1. MCP tools/list contains tethers.evaluate + tethers.validate only (2 tools)
2. tethers.evaluate calls Tethers_core_wire
3. Standalone tethers_engine uses Core via tethers_core_wire
4. HostExecutionService::evaluate_one uses build_core_request_envelope
5. EngineSession::evaluate_tether_core is removed
6. All OCaml and Rust tests remain green
7. cargo fmt --check PASS
8. cargo check PASS
9. cargo test PASS (1448 passed)
10. git diff --check PASS (LF/CRLF warnings only)

## Required verification

1. `cargo fmt --check` -- PASS
2. `cargo check` -- PASS
3. `cargo test` -- PASS (1448 passed, 0 failed)
4. `dune build @all` -- PASS
5. `dune runtest --force` -- PASS (all OCaml tests green)
6. `git diff --check` -- PASS (LF/CRLF warnings only)
7. Diff inspection: only authorised files changed

## Forbidden changes

Do NOT delete tethers_evaluator.ml (may remain as historical reference)
Do NOT modify provider dispatch, policy, Trail semantics, replay, approval.
Do NOT derive semantic identities.
Do NOT equate Core contract digest with manifest digest.
Do NOT modify CORE-8A/8B semantics or Runtime Plan semantics.

## Stop conditions

Implementation checkpoint committed. STOP.

## Expected pre-existing changes

CORE-9B cross-language rehearsal (accepted).
