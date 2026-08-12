Task: `CORE-9C Canonical Core Production Cutover`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base branch: `feature/core-9b-cross-language-rehearsal`

Base commit: `adec68156481fc319de44bf686bcf8d1ef65263b`

Implementation checkpoint: `227f54f70a18b80abc498f3ac8ba26edffc82465`

## Requested outcome

Put the accepted canonical Core into the real production evaluation path.
After this packet there must be ONE production evaluation route.

## Changes made

### Modified files
- `tethers-0.1/engine-ocaml/bin/tethers_mcp_server.ml` -- switched
  tethers.evaluate to Core (tethers_core_wire), removed tethers.evaluate_core
  from tools/list and tools/call
- `tethers-0.1/engine-ocaml/bin/main.ml` -- switched standalone tethers_engine
  to Core via tethers_core_wire
- `tethers-0.1/engine-ocaml/bin/dune` -- removed Tethers_evaluator from MCP
  and standalone module lists, added Core pipeline modules and digestif
- `tethers-0.1/host-rust/src/engine_stdio.rs` -- removed evaluate_tether_core,
  updated all tests to use evaluate_tether (2 tools, all Core)
- `tethers-0.1/host-rust/src/host_execution.rs` -- switched evaluate_one to
  build_core_request_envelope, updated builder tests to use evaluate_tether

## Decisions and assumptions

- tethers.evaluate now uses Core via tethers_core_wire (single evaluation route)
- tethers.evaluate_core is removed (no dual evaluation routes)
- EngineSession::evaluate_tether_core is removed
- build_core_request_envelope is the production request builder
- Tethers_evaluator source remains in repository as historical reference
- All tests updated to reflect cutover reality (2 tools, all Core)

## Evidence

### OCaml
- `dune build @all` -- PASS
- `dune runtest --force` -- PASS (all tests green, T1-T3 included)

### Rust
- `cargo fmt --check` -- PASS
- `cargo check` -- PASS
- `cargo test` -- PASS (1448 passed, 0 failed, 2 ignored)
- All engine_stdio tests pass (17/17)
- All host_execution tests pass

### Repository
- `git diff --check` -- PASS (LF/CRLF warnings only)
- Implementation checkpoint: `227f54f70a18b80abc498f3ac8ba26edffc82465`

## Publication evidence

Implementation checkpoint committed: `227f54f70a18b80abc498f3ac8ba26edffc82465`
Branch: `feature/core-9c-production-cutover`

## Discoveries

- The MCP server's tethers.evaluate now routes to Core wire adapter
- Standalone tethers_engine now uses Core wire adapter directly
- Production HostExecutionService::evaluate_one now uses
  build_core_request_envelope which requires core_environment
- All legacy tests that used evaluate_tether_core have been updated to
  use evaluate_tether with Core requests

## Remaining risks

- The validate_core_environment LSP error in runtime_config.rs is pre-existing
  and not from this task

## Smallest next action

Push and stop for independent review.

## References

- `docs/CURRENT_CLINE_TASK.md` -- task packet
- `tethers-0.1/engine-ocaml/bin/tethers_mcp_server.ml` -- MCP server
- `tethers-0.1/engine-ocaml/bin/main.ml` -- standalone engine
- `tethers-0.1/engine-ocaml/bin/dune` -- build config
- `tethers-0.1/host-rust/src/engine_stdio.rs` -- engine session
- `tethers-0.1/host-rust/src/host_execution.rs` -- host execution service
