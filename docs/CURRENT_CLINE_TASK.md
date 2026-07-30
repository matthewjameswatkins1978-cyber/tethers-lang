# Current Implementation Task

Control contract: `1`

Task: `J14A-R — harden trusted execution evidence and exercise the committed scenario`

Owner: `OpenCode`

Status: `COMPLETE`

Task colour: `Amber`

Route: `OpenCode implementation — Amber repair, Lucy independent review`

Base commit: `0c64b48d860ce2178858c4c5d8a0af38708bc7cc`

Rejected candidate: `e86471ed8d160d47ba2ca70a6acbfabaf552f6ac`

Branch: `goose/j14a-complete-local-scenario`

Worker note: `docs/worker-notes/2026-07-30-j14a-complete-local-scenario.md`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

## Objective (J14A-R repair)

Correct three J14A acceptance defects:

1. Trusted execution identity currently travels through planner-response JSON
   using the private-looking field `_host_execution_id`.
2. Post-admission persistence and intent-recording failures may discard a
   trusted execution identity.
3. The public scenario harness reconstructs its input and runtime instead of
   exercising the committed scenario artefacts.

J14A remains unaccepted until this repair passes independent review.

## Relevant background and existing behaviour

J13A (`check`) validates Tether source, engine, and provider availability.
J13B (`run`) submits one explicit Anchor and Facts through the real execution
slice. J13C (`trail`) provides read-only Trail inspection. The public run
command did not expose the host-issued execution ID required by trail.

## Required behaviour (J14A-R repair)

1. `execute_boundary_impl` returns typed `ExecutionBoundaryEvidence` instead of
   writing `_host_execution_id` into mutable response JSON.
2. `execute_shared_boundary` combines typed evidence with classified
   `SharedExecutionOutcome`; never reads execution ID from response JSON.
3. `SharedExecutionResult` receives execution_id from typed evidence, not from
   `from_response` parsing.
4. `dispatch_matched_response` strips both `execution_id` and `_host_execution_id`
   from planner-supplied data.
5. Post-admission failures (intent, persistance, terminal) retain their trusted
   execution ID.
6. `Denied` and `ReplayPersistenceUnavailable` carry optional execution_id for
   post-admission paths.
7. Public scenario harness materialises committed `runtime.template.json` (not
   reconstructed JSON), copies committed `input.json`, and uses Unicode+space
   temp path.

## Relevant components (J14A-R repair)

- `tethers-0.1/host-rust/src/main.rs` - ExecutionBoundaryEvidence, execute_boundary_impl, execute_shared_boundary, SharedExecutionResult
- `tethers-0.1/host-rust/src/host_execution.rs` - ExecutionServiceResult, dispatch_matched_response, map_shared_result
- `tethers-0.1/host-rust/src/run_command.rs` - map_execution_result
- `tethers-0.1/scripts/test-j14a-complete-scenario.ps1` - rewritten acceptance script
- `docs/CURRENT_CLINE_TASK.md` - this task packet
- `docs/worker-notes/2026-07-30-j14a-complete-local-scenario.md` - evidence note

## Expected pre-existing changes

None. Starting from clean `0c64b48d860ce2178858c4c5d8a0af38708bc7cc` on branch
`goose/j14a-complete-local-scenario`.

## Frozen decisions and invariants

- `replay::ExecutionId::parse` remains authoritative.
- Execution ID is obtained only from successful replay admission, never from planner.
- Result Anchor schema and serialization remain unchanged.
- Replay does not allocate a new execution identity for replay.
- Typed boundary: run_command consumes ExecutionServiceResult, not raw JSON.

## Acceptance criteria (J14A-R repair)

1. `_host_execution_id` is never written into response JSON by any production path.
2. `SharedExecutionResult` receives execution_id through typed `ExecutionBoundaryEvidence`, not response parsing.
3. Planner-supplied `execution_id` and `_host_execution_id` are stripped before dispatch.
4. Post-admission intent/persistence failure retains its trusted execution ID.
5. Pre-admission Deny has no execution_id; post-admission Denied retains it.
6. ReplayPersistenceUnavailable carries execution_id only when admission already established one.
7. Result Anchor contains no execution_id.
8. J14A scenario: committed template, input, and Tether exercised; Unicode+space temp path.
9. All Rust tests pass, including 15+ j14a_ tests.
10. Cargo.lock hash: `d323870ea02f09391a5d0d9aa0e9a701cf686a5ac005b840ee7218e70edb5602`.
11. All regressions pass. Packet checker passes.

## Forbidden changes (J14A-R repair)

No OCaml, Cargo.toml, Cargo.lock, scenario source files, DECISIONS.md, Result Anchor,
replay storage, identity-generation, provider fixtures, or manifest changes.
Do not rebase, amend, squash, reset, or force-push.

## Stop conditions

Return BLOCKED when: origin/main differs, dirty worktree, reasoning not MEDIUM,
toolchain preflight fails, Cargo.lock changes, Result Anchor must change.

## Required verification (J14A-R repair)

Full Rust test suite, fmt, clippy, build (debug+release).
J14A public scenario, J13A/B/C regressions, host integration scripts, engine,
demo, fixtures, MCP, packet checker, whitespace checks.
Cargo.lock hash unchanged. Repository git status clean.
