Task: `CORE-9B Cross-Language Rehearsal`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `fe9919034a3d04cbbe3056f7c7fdc91c041032ba`

Implementation checkpoint: `c79db5caf096d8a3037476ee422d4ba25cdeab42`

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
  `tethers.evaluate_core`; added `list_tools()` method
- `tethers-0.1/host-rust/src/host_execution.rs` — added dormant
  `build_core_request_envelope()` reusing existing request assembly
  pipeline; added real PreparedRuntime test helpers and T9-T14 builder tests

## Decisions and assumptions

- Wire adapter delegates entirely to CORE-8B `evaluate_request` — no
  reimplemented parsing, lowering, canonicalization, or planning
- `program_digest` is a sibling of `plan` in the response envelope (not
  inside the plan object); comes only from `canonical_plan.program_digest`
- Error codes for adapter_error use a stable "adapter_error" code with
  detail in the message, avoiding tight coupling to internal adapter error
  variants
- `build_core_request_envelope` reuses `build_request_envelope` then
  inserts core_environment — identical runtime capability projection
  guaranteed
- MCP `tethers.evaluate` tool dispatch unchanged — legacy evaluator path
  preserved exactly
- T9-T14 tests use real PreparedRuntime built through accepted runtime
  configuration/preparation path with fixture-ping manifest

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
- T1 PASS: wire Matched — correct envelope, program_digest at top level
  (sibling of plan), absent from plan, plan.id, action capability,
  idempotency_key, empty trail
- T2 PASS: wire Not_matched — correct status, plan null
- T3 PASS: wire request error — missing_core_environment stable code

### Rust
- `cargo fmt --check` — PASS
- `cargo check` — PASS (only dead_code warning on dormant method)
- `cargo test` — PASS (1448 passed, 0 failed)
- Production route unchanged: `evaluate_one` still calls
  `build_request_envelope` + `engine.evaluate_tether`
- T4 PASS: MCP tools/list contains tethers.validate, tethers.evaluate,
  tethers.evaluate_core — exactly 3 tools
- T5 PASS: Legacy tethers.evaluate still uses legacy evaluator
  (no core_environment, runtime capability name in tether source)
- T6 PASS: New tethers.evaluate_core reaches Core pipeline,
  program_digest at top level, absent from plan
- T7 PASS: EngineSession::evaluate_tether calls tethers.evaluate
  and works with historical request
- T8 PASS: EngineSession::evaluate_tether_core calls
  tethers.evaluate_core and returns Matched
- T9 PASS: build_core_request_envelope fails with InvalidData when
  core_environment is absent (tests Rust builder directly)
- T10 PASS: Builder output identity separation — source_name=notify,
  capability_id=cap.semantic.notify, contract_digest=CORE-CONTRACT-9B,
  runtime_name=fixture.ping; no derivation
- T11 PASS: Builder output bridge metadata separation — core_environment
  has no manifest_digest/bridge_capability_version/bridge_provider_identity;
  top-level runtime capability has real values; CORE-CONTRACT-9B != manifest digest
- T12 PASS: Real Rust-built cross-language E2E — PreparedRuntime →
  build_core_request_envelope → EngineSession::evaluate_tether_core →
  tethers.evaluate_core → Tethers_core_wire → CORE-8B → canonical Core →
  Runtime Plan with correct plan.id, program_digest (top level, absent from
  plan), action capability, arguments, idempotency_key, effects, and
  bridge metadata
- T13 PASS: Builder with wrong event (fixture.other) produces NotMatched
- T14 PASS: Builder with two evaluation IDs — same ProgramDigest, different
  plan.id, different idempotency keys

### Repository
- `git diff --check` — PASS (LF/CRLF warnings only)
- Implementation checkpoint: `c79db5caf096d8a3037476ee422d4ba25cdeab42`

## Publication evidence

Implementation checkpoint committed: `c79db5caf096d8a3037476ee422d4ba25cdeab42`
Branch: `feature/core-9b-cross-language-rehearsal`

## Discoveries

- The existing `Tethers_outcome.json_of_response` produces the exact
  historical response envelope; `program_digest` is added as a sibling
  of `plan` at the top level of the envelope
- The MCP server's `tethers.evaluate_core` handler catches no exceptions
  because `Tethers_core_wire.evaluate_request_json` never raises — all
  outcomes are represented in the returned JSON
- The wire adapter error message for adapter_error variants provides a
  human-readable detail string while keeping the stable machine code
  generic ("adapter_error")
- Two OCaml engine binaries exist (Goose Integration and Tethers Lang
  workspaces); Rust tests require the one in the Goose Integration workspace

## Remaining risks

- The `build_core_request_envelope` Rust method is dormant — CORE-9C will
  wire it into production

## Smallest next action

Push and stop for independent review.

## References

- `docs/CURRENT_CLINE_TASK.md` — task packet
- `tethers-0.1/engine-ocaml/bin/tethers_core_wire.ml` — wire adapter
- `tethers-0.1/engine-ocaml/bin/tethers_core_wire.mli` — interface
- `tethers-0.1/engine-ocaml/bin/tethers_core_wire_test.ml` — tests
- `tethers-0.1/engine-ocaml/bin/tethers_mcp_server.ml` — MCP server
- `tethers-0.1/engine-ocaml/bin/dune` — build config
- `tethers-0.1/host-rust/src/engine_stdio.rs:266` — evaluate_tether_core
- `tethers-0.1/host-rust/src/host_execution.rs:644` — build_core_request_envelope
