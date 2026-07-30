# Worker Note

- **Task Packet:** `J14A - public run-to-trail identity and complete positive scenario`
- **Owner:** `Goose`
- **Status:** `COMPLETE`
- **Base Commit:** `0c64b48d860ce2178858c4c5d8a0af38708bc7cc`
- **Implementation checkpoint:** `WORKTREE`
- **Branch / Worktree:** `goose/j14a-complete-local-scenario` / `D:\The Next Thing\Tethers Lang - Goose Integration`

Task: `J14A - public run-to-trail identity and complete positive scenario`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Goose`

Status: `COMPLETE`

Base commit: `0c64b48d860ce2178858c4c5d8a0af38708bc7cc`

Implementation checkpoint: `WORKTREE`

## Requested outcome

Create one reproducible native-Windows positive scenario exposing the host-issued
execution ID through the public run envelope, proving check-to-run-to-trail-to-replay
identity round-trip.

## Changes made

1. `main.rs`: SharedExecutionResult from enum to struct with `execution_id`;
   `execute_boundary_impl` stores `_host_execution_id` in response after replay
   admission.
2. `host_execution.rs`: ExecutionServiceResult gains `execution_id` field on
   applicable variants; `map_shared_result` threads identity; spoofing protection
   strips planner-supplied `execution_id`.
3. `run_command.rs`: `execution_data_with_id` helper; `map_execution_result`
   exposes `data.execution_id`.
4. `test-j13b-run.ps1`: execution_id presence/absence assertions.
5. `test-j14a-complete-scenario.ps1`: new 5-case, 85-assertion scenario script.
6. `scenarios/j14-complete-local/`: Tether, input, template, README.
7. `DECISIONS.md`: J14A decision entry.

## Decisions and assumptions

- Trusted execution ID is obtained only from replay admission, never from planner.
- Result Anchor schema unchanged; execution_id never appears in it.
- `SharedExecutionResult` now carries typed optional evidence.
- Replay does not allocate new execution identities.

## Evidence

- Rust: 716 tests pass (19 new j14a_ tests)
- fmt --check: PASS
- clippy: pre-existing warnings only
- build (debug+release): PASS
- Cargo.lock SHA-256: `d323870ea02f09391a5d0d9aa0e9a701cf686a5ac005b840ee7218e70edb5602` (unchanged)
- J14A scenario: 5 cases, 85 assertions, all PASS
- J13A: 25 PASS, J13B: 10 PASS, J13C: 19 PASS
- Host denial, execution failure, result follow-up, event admission, event-admission-trail: all PASS
- Fixtures, MCP transcripts, engine, demo: all PASS
- DECISIONS.md numstat: additions-only, zero deletions
- git diff --check: PASS (LF/CRLF advisory only)

## Discoveries

- The J14A scenario works best with programmatic config construction (J13B-style)
  rather than template substitution with absolute paths.

## Remaining risks

- J14B (negative integration matrix) depends on this accepted foundation.
- No J14A-specific risks.

## Smallest next action

J14B: negative public integration matrix (malformed manifest, unavailable
provider, Ask, Deny, stale pin, intent failure, executor failure, invalid output,
uncertain timeout, duplicate replay, loop depth).

## References

- `docs/DECISIONS.md` - J14A decision
- `docs/ROAD_TO_0_2.md` - release route
- `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md` - Rust guidance
