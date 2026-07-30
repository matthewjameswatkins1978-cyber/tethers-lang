# Current Implementation Task

Control contract: `1`

Task: `J14A - public run-to-trail identity and complete positive scenario`

Owner: `Goose`

Status: `COMPLETE`

Task colour: `Amber`

Route: `Goose Medium - Amber public integration proof`

Base commit: `0c64b48d860ce2178858c4c5d8a0af38708bc7cc`

Branch: `goose/j14a-complete-local-scenario`

Worker note: `docs/worker-notes/2026-07-30-j14a-complete-local-scenario.md`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

## Objective

Create one reproducible native-Windows positive scenario using the public
operational commands check, run, and trail. Expose the host-issued execution ID
through the public run envelope.

## Relevant background and existing behaviour

J13A (`check`) validates Tether source, engine, and provider availability.
J13B (`run`) submits one explicit Anchor and Facts through the real execution
slice. J13C (`trail`) provides read-only Trail inspection. The public run
command did not expose the host-issued execution ID required by trail.

## Required behaviour

1. Public run envelope gains `data.execution_id` when a trusted replay
   admission identity exists.
2. Execution ID is never accepted from the caller, planner, or CLI layer.
3. Result Anchor schema remains unchanged.
4. Complete J14A scenario proves check, run, trail, and replay.
5. Focused Rust tests prove identity presence, absence, and spoofing protection.
6. Update J13B acceptance to prove execution_id presence/absence.

## Relevant components

- `tethers-0.1/host-rust/src/main.rs` - SharedExecutionResult, execute_boundary_impl
- `tethers-0.1/host-rust/src/host_execution.rs` - ExecutionServiceResult, map_shared_result
- `tethers-0.1/host-rust/src/run_command.rs` - map_execution_result, execution_data
- `tethers-0.1/scripts/test-j14a-complete-scenario.ps1` - new acceptance script
- `tethers-0.1/scenarios/j14-complete-local/` - scenario files
- `docs/DECISIONS.md` - decision record
- `docs/CURRENT_CLINE_TASK.md` - this task packet

## Expected pre-existing changes

None. Starting from clean `0c64b48d860ce2178858c4c5d8a0af38708bc7cc` on branch
`goose/j14a-complete-local-scenario`.

## Frozen decisions and invariants

- `replay::ExecutionId::parse` remains authoritative.
- Execution ID is obtained only from successful replay admission, never from planner.
- Result Anchor schema and serialization remain unchanged.
- Replay does not allocate a new execution identity for replay.
- Typed boundary: run_command consumes ExecutionServiceResult, not raw JSON.

## Acceptance criteria

1. Public run data exposes execution_id when trusted identity exists.
2. Execution_id absent for Deny, Ask, NoActions, Unavailable, pre-admission failures.
3. Planner-supplied fake execution_id is stripped.
4. Result Anchor contains no execution_id.
5. J14A scenario: 5 cases, check/run/trail/replay all pass.
6. All Rust tests pass (716), including 19 new j14a_ tests.
7. All regression scripts pass.
8. DECISIONS.md: additions-only diff, zero deleted lines.
9. Cargo.lock hash unchanged.
10. Packet checker and whitespace checks pass.

## Forbidden changes

No OCaml, Cargo.toml, Cargo.lock, engine, provider, replay-storage changes.
No Result Anchor schema change. No change to how execution IDs are generated.

## Stop conditions

Return BLOCKED when: origin/main differs, dirty worktree, reasoning not MEDIUM,
toolchain preflight fails, Cargo.lock changes, Result Anchor must change.

## Required verification

Full Rust test suite, fmt, clippy, build (debug+release).
J14A public scenario (5 cases).
J13A/B/C regressions, host integration scripts, engine, demo, fixtures, MCP.
Packet checker, whitespace checks, Cargo.lock hash, DECISIONS.md numstat.
