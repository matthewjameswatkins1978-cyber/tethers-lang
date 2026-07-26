# Task Queue

Updated: 2026-07-26

## Current State

Latest accepted implementation checkpoint:

`d5ed278d4a2cae5e9ab8a3e1d8700fdcba7ae851`

- [x] Tethers 0.1 semantic and protocol baseline signed off.
- [x] OCaml MCP planner and validation surface complete.
- [x] Manifest parsing, canonicalisation, digesting, and trusted store complete.
- [x] Configured local stdio provider discovery and fail-closed admission complete.
- [x] Deterministic capability projection complete.
- [x] Planner-to-dispatch manifest, version, and provider pinning complete.
- [x] Intent-first dispatch proof boundary complete.
- [x] Executor output validation complete.
- [x] Known-outcome Result Anchors complete.
- [x] Four-outcome effective policy complete:
  `allow`, `ask`, `deny`, and `unavailable`.
- [x] J04a stale-digest and unestablished-scope correction accepted.

The rejected J04 attempt remains in worker notes and Git history. It is not
current project state.

## Active Gate

- [ ] J09 Red implementation: durable replay protection.

J05 and J06 are accepted. The historical J07 deadline/uncertainty and J08
uncertain-Result-Anchor outcomes were absorbed into accepted J06; their entries
remain in the roadmap as provenance and are not separate active jobs. J09 is
the next milestone. Its frozen authority is `docs/J09_DURABLE_REPLAY_DESIGN.md`.

Expected route:

- Lucy: architecture and acceptance.
- Codex: Red implementation and local verification.
- Cline: no J09 work unless Lucy later compiles and routes a bounded subtask.

## Remaining 0.2 Queue

### Exact Approval

- [x] J05 exact one-shot Ask resolution accepted.

### Honest Execution

- [x] J06 deadline and truthful outcome classification accepted.
- [x] J07 historical deadline and `uncertain` outcome intent absorbed into J06.
- [x] J08 historical `capability.uncertain` Result Anchor intent absorbed into J06.
- [ ] J09 durable replay protection — next active milestone; design frozen.

### Event Continuation

- [ ] J10 queue generated Result Anchors serially.
- [ ] J11 reject duplicate event IDs and enforce causal depth eight.

### Operable Runtime

- [ ] J12 freeze the minimal runnable Tether Set and configuration boundary.
- [ ] J13 implement local `check`, `run`, and `trail` routes.
- [ ] J14 prove one complete real local scenario and required negative cases.

### Hardening And Release

- [ ] J15 consolidate the 0.2 failure matrix.
- [ ] J16 prove clean-checkout, restart, and replay behaviour.
- [ ] J17 independently sign off and tag 0.2.0.

The detailed dependency and acceptance map lives in `docs/ROAD_TO_0_2.md`.
Executable tasks are compiled just in time into `docs/CURRENT_CLINE_TASK.md`.

## Gorilla Coding Route 🦄

```text
Lucy inspects and compiles
-> Matthew routes to Cline or Codex
-> one worker implements and verifies
-> worker writes the note and concise report
-> Matthew pastes the report to Lucy
-> Lucy accepts, corrects, or escalates
```

- Cline is the default Green and Amber implementation owner.
- Codex handles Red work, difficult local failure, Git/environment/recovery, and
  machine-required verification.
- Copilot is not part of the active workflow.
- Cline does not compile or begin the next task.

## Completed Foundations

### Toolchain And Verification

- [x] Native Windows OCaml 5.5.0, Dune, Yojson, Rust, and PowerShell 7 workflows.
- [x] Fixture validation, OCaml build, Rust tests, golden engine tests, MCP
  transcripts, host denial/failure tests, demo, and whitespace checks.
- [x] Deterministic-repeat and focused failure-branch coverage.

### Tethers 0.1

- [x] Parser, protocol helpers, deterministic evaluator, ordered Plans, and causal
  evaluation Trails.
- [x] Correlated evaluation and planning errors.
- [x] Version rejection, indentation, argument uniqueness, type, missing Fact,
  unknown Capability, and operator fixtures.
- [x] Host authorisation, execution, idempotency identity, and execution Trail.

### MCP And Capability Bridge

- [x] M0-M7 MCP planning and authoring direction.
- [x] `tethers.evaluate` and `tethers.validate` over stdio.
- [x] Trusted manifest and provider binding design.
- [x] Exact capability projection and opaque digest pass-through.

### Columbo

- [x] C1 manifest parsing, duplicate-key rejection, RFC 8785/JCS canonicalisation,
  SHA-256 digesting, and semantic validation.
- [x] C2 verified manifest admission and identity/digest indexes.

## Deferred Beyond 0.2

- Lantern Keeper provider integration until it exposes a small stable capability
  surface.
- Safe retry until idempotency is proved end to end.
- Additional providers and automatic discovery.
- Remote transports, OAuth, and network listeners.
- HQ, package management, marketplace, scheduling, and adapters.
- General plugin or AI-agent framework features.
- Cosmetic rename of `tethers-0.1/` while the local opam switch remains
  path-bound.

## Working Rule

The ten-minute implementation-step limit is a runaway brake, not a deadline.
Stop at a coherent recoverable point and return exact evidence rather than rush,
repeat attempts blindly, or invent missing decisions.
