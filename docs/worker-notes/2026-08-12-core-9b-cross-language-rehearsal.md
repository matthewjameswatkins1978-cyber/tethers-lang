Task: `CORE-9B Cross-Language Rehearsal`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `fe9919034a3d04cbbe3056f7c7fdc91c041032ba`

Implementation checkpoint: `b6329de0e0faeea0ba526e74b179e91a2cc0a897`

## Requested outcome

Prove that a REAL request assembled by the Rust host using the accepted
CORE-9A semantic authority can cross the retained MCP engine boundary,
enter the accepted CORE-8B request adapter, and produce a canonical
Runtime Plan.

## Changes made

### New files
- `tethers-0.1/engine-ocaml/bin/tethers_core_wire.ml` — wire adapter module
  bridging CORE-8B request adapter to PlannerResponseWire-compatible JSON
- `tethers-0.1/engine-ocaml/bin/tethers_core_wire.mli` — interface file
- `tethers-0.1/engine-ocaml/bin/tethers_core_wire_test.ml` — T1/T2/T3 tests

### Modified files
- `tethers-0.1/engine-ocaml/bin/tethers_mcp_server.ml` — added
  `tethers.evaluate_core` tool to tools/list and tools/call dispatch;
  `tethers.evaluate` remains on legacy evaluator
- `tethers-0.1/engine-ocaml/bin/dune` — added Core pipeline modules and
  digestif to MCP binary; added wire test target
- `tethers-0.1/host-rust/src/engine_stdio.rs` — added
  `EngineSession::evaluate_tether_core()` method calling
  `tethers.evaluate_core`
- `tethers-0.1/host-rust/src/host_execution.rs` — added dormant
  `build_core_request_envelope()` reusing existing request assembly
  pipeline

## Decisions and assumptions

- Wire adapter delegates entirely to CORE-8B `evaluate_request` — no
  reimplemented parsing, lowering, canonicalization, or planning
- `program_digest` is enriched into the plan object within the existing
  `Tethers_outcome.json_of_response` envelope shape
- Error codes for adapter_error use a stable "adapter_error" code with
  detail in the message, avoiding tight coupling to internal adapter error
  variants
- `build_core_request_envelope` reuses `build_request_envelope` then
  inserts core_environment — identical runtime capability projection
  guaranteed
- MCP `tethers.evaluate` tool dispatch unchanged — legacy evaluator path
  preserved exactly

## Evidence

### OCaml
- `dune build` — PASS (0 errors)
- `dune runtest --force` — PASS
  - lowerer tests: 49/49
  - validator tests: 51/51
  - plan bridge tests: 179/179
  - adapter tests: 43/43
  - request adapter tests: 89/89
  - wire tests T1/T2/T3: 3/3
  - canonical tests: PASS
- T1 PASS: wire Matched — correct envelope, program_digest, plan.id,
  action capability, idempotency_key, empty trail
- T2 PASS: wire Not_matched — correct status, plan null
- T3 PASS: wire request error — missing_core_environment stable code

### Rust
- `cargo fmt --check` — PASS
- `cargo check` — PASS (only dead_code warning on dormant method)
- `cargo test` — PASS (1442 passed, 0 failed, 5 ignored for provider exe)
- Production route unchanged: `evaluate_one` still calls
  `build_request_envelope` + `engine.evaluate_tether`
- T4 PASS: MCP tools/list contains tethers.validate, tethers.evaluate,
  tethers.evaluate_core — exactly 3 tools
- T5 PASS: Legacy tethers.evaluate still uses legacy evaluator
  (no core_environment, runtime capability name in tether source)
- T6 PASS: New tethers.evaluate_core reaches Core pipeline,
  program_digest present
- T7 PASS: EngineSession::evaluate_tether calls tethers.evaluate
  and works with historical request
- T8 PASS: EngineSession::evaluate_tether_core calls
  tethers.evaluate_core and returns Matched
- T9 PASS: No core_environment produces missing_core_environment error
- T10 PASS: Identity separation — source_name=notify,
  capability_id=cap.semantic.notify, contract_digest=CORE-CONTRACT-9B,
  runtime_name=fixture.ping; no derivation
- T11 PASS: Bridge metadata separation — core_environment has no
  manifest_digest/bridge_capability_version/bridge_provider_identity;
  top-level runtime capability has all three
- T12 PASS: Real cross-language E2E — Rust request → real OCaml MCP
  binary → tethers.evaluate_core → Tethers_core_wire → CORE-8B →
  canonical Core → Runtime Plan with correct plan.id, program_digest,
  action capability, arguments, idempotency_key, effects, and bridge
  metadata
- T13 PASS: Wrong event (fixture.other) through real flow produces
  NotMatched
- T14 PASS: Occurrence identity — same program, different evaluation_id
  produces same ProgramDigest, different plan.id, different
  idempotency keys

### Repository
- `git diff --check` — PASS (LF/CRLF warnings only)
- Implementation checkpoint: `b6329de0e0faeea0ba526e74b179e91a2cc0a897`

## Publication evidence

Implementation checkpoint committed: `b6329de0e0faeea0ba526e74b179e91a2cc0a897`
Branch: `feature/core-9b-cross-language-rehearsal`

## Discoveries

- The existing `Tethers_outcome.json_of_response` produces the exact
  historical response envelope; enriching it with `program_digest` requires
  a targeted plan-object modification rather than a separate envelope
- The MCP server's `tethers.evaluate_core` handler catches no exceptions
  because `Tethers_core_wire.evaluate_request_json` never raises — all
  outcomes are represented in the returned JSON
- The wire adapter error message for adapter_error variants provides a
  human-readable detail string while keeping the stable machine code
  generic ("adapter_error")

## Remaining risks

- The `build_core_request_envelope` Rust method is dormant — CORE-9C will
  wire it into production

## Smallest next action

Commit and push the closeout documentation, then run the task packet
checker to confirm `control-v1/COMPLETE`.

## References

- `docs/CURRENT_CLINE_TASK.md` — task packet
- `tethers-0.1/engine-ocaml/bin/tethers_core_wire.ml` — wire adapter
- `tethers-0.1/engine-ocaml/bin/tethers_core_wire.mli` — interface
- `tethers-0.1/engine-ocaml/bin/tethers_core_wire_test.ml` — tests
- `tethers-0.1/engine-ocaml/bin/tethers_mcp_server.ml` — MCP server
- `tethers-0.1/engine-ocaml/bin/dune` — build config
- `tethers-0.1/host-rust/src/engine_stdio.rs:266` — evaluate_tether_core
- `tethers-0.1/host-rust/src/host_execution.rs:644` — build_core_request_envelope
