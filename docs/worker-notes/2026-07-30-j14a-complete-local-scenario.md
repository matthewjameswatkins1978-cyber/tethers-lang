# Worker Note

- **Task Packet:** `J14A-R`
- **Owner:** `OpenCode`
- **Status:** `COMPLETE`
- **Base Commit:** `0c64b48d860ce2178858c4c5d8a0af38708bc7cc`
- **Implementation checkpoint:** `WORKTREE`
- **Branch / Worktree:** `goose/j14a-complete-local-scenario` / `D:\The Next Thing\Tethers Lang - Goose Integration`

The final pushed repair SHA is reported in the external completion report.

Task: `J14A-R — harden trusted execution evidence and exercise the committed scenario`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `0c64b48d860ce2178858c4c5d8a0af38708bc7cc`

Implementation checkpoint: `WORKTREE`

Original implementation owner: Goose. Repair owner: OpenCode.
Rejected candidate: `e86471ed8d160d47ba2ca70a6acbfabaf552f6ac`.

## Required Reading

- `AGENTS.md`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/PROJECT_CONTROL.md`
- `docs/AGENT_WORKFLOW.md`
- `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md`
- `docs/GIT_WORKTREES_AND_LINE_ENDINGS_FOR_AGENTS.md`
- `tethers-0.1/host-rust/src/main.rs`
- `tethers-0.1/host-rust/src/host_execution.rs`
- `tethers-0.1/host-rust/src/run_command.rs`
- `tethers-0.1/host-rust/src/replay.rs`
- `tethers-0.1/host-rust/src/replay_runtime.rs`
- `tethers-0.1/scripts/test-j14a-complete-scenario.ps1`
- `tethers-0.1/scenarios/j14-complete-local/` (all four committed files)

## Requested outcome

Create one reproducible native-Windows positive scenario exposing the host-issued
execution ID through the public run envelope, proving check-to-run-to-trail-to-replay
identity round-trip.

## Changes made

### Original Goose implementation

1. `main.rs`: SharedExecutionResult from enum to struct with `execution_id`;
   `execute_boundary_impl` stores `_host_execution_id` in response after replay admission.
2. `host_execution.rs`: ExecutionServiceResult gains `execution_id` field on
   applicable variants; `map_shared_result` threads identity; strips `execution_id`.
3. `run_command.rs`: `execution_data_with_id` helper; `map_execution_result` exposes `data.execution_id`.
4. `test-j14a-complete-scenario.ps1`: 5-case scenario script.
5. `scenarios/j14-complete-local/`: Tether, input, template, README.
6. `DECISIONS.md`: J14A decision entry.

### J14A-R repair (OpenCode)

1. `main.rs`: Added `ExecutionBoundaryEvidence`. `execute_boundary_impl` returns typed
   evidence instead of writing `_host_execution_id` into response JSON.
   `execute_shared_boundary` combines typed evidence with classified outcome.
   `SharedExecutionResult::from_response` replaced with `from_response_and_evidence`.
2. `host_execution.rs`: Strips both `execution_id` and `_host_execution_id` from planner
   response. `Denied` and `ReplayPersistenceUnavailable` gain `execution_id: Option<String>`.
   Post-admission Denied/persistence failures retain trusted ID.
3. `run_command.rs`: `map_execution_result` exposes `execution_id` for Denied and
   ReplayPersistenceUnavailable when present.
4. `test-j14a-complete-scenario.ps1`: Rewritten to exercise committed scenario files,
   Unicode+space temp path, template-based config with relative manifest_path,
   hash protection for all four committed files, provider count assertions,
   structural trail comparison.
5. Focused tests: 25 j14a_ tests proving spoofing protection, audit_failure evidence,
   post-admission identity, public ID parse.

### Authorised files

- `tethers-0.1/host-rust/src/main.rs`
- `tethers-0.1/host-rust/src/host_execution.rs`
- `tethers-0.1/host-rust/src/run_command.rs`
- `tethers-0.1/scripts/test-j14a-complete-scenario.ps1`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-07-30-j14a-complete-local-scenario.md`

## Decisions and assumptions

- Trusted execution ID is obtained only from typed `ExecutionBoundaryEvidence`, never from planner-response JSON.
- `_host_execution_id` is never written or read as trusted evidence on any production path.
- Planner-supplied `execution_id` and `_host_execution_id` are stripped before dispatch.
- Post-admission failures (intent, persistence, terminal) retain their trusted execution ID.
- Pre-admission Deny carries no execution_id; post-admission Denied retains it.
- ReplayPersistenceUnavailable carries execution_id only when admission had already established one.
- Result Anchor schema unchanged; execution_id never appears in it.
- Replay does not allocate new execution identities.

## Evidence

- **Rust tests:** 791 passed, 0 failed (40 lib + 722 bin + 29 integration)
- **j14a_ focused:** 25 passed, 0 failed
- **fmt --check:** PASS
- **clippy:** pre-existing warnings only, no new warnings
- **build (debug+release):** PASS
- **Cargo.lock SHA-256:** `d323870ea02f09391a5d0d9aa0e9a701cf686a5ac005b840ee7218e70edb5602` (unchanged)
- **J14A scenario:** 5 cases, 95 assertions, all PASS
- **J13A:** 25 PASS, **J13B:** 10 PASS, **J13C:** 19 PASS
- **Event admission:** PASS, **Event admission trail:** PASS
- **Fixtures:** PASS, **MCP transcripts:** 17 PASS

### Scenario provider counts

- Check: initialize=1, tools/list=1, tools/call=0
- First run: initialize=1, tools/list=1, tools/call=1
- Replay: initialize=2, tools/list=2, tools/call=1 (no second effect)

### Pre-existing regression not re-verified

Host denial, execution failure, result follow-up, engine, and demo scripts use a
dune build step requiring the OCaml switch from the primary Tethers worktree.
No OCaml code was changed.

## Discoveries

- The first COMPLETE report was rejected because: planner response JSON was used
  as trusted identity transport; the private-looking `_host_execution_id` was not
  stripped; post-admission persistence and Denied mappings discarded trusted evidence;
  the harness did not exercise `runtime.template.json` or `input.json`; the Unicode-path
  requirement was not met; not all scenario files were hash-protected; replay Trail
  entries were not actually compared; the evidence note incorrectly claimed no
  J14A-specific risks.
- The runtime.json `manifest_path` field must be relative to the workspace directory.
- The commit requires template placeholder substitution with correct JSON path
  escaping.

## Remaining risks

- J14B (negative integration matrix) depends on this accepted foundation.

## Smallest next action

J14B: negative public integration matrix.

## References

- `docs/CURRENT_CLINE_TASK.md` - J14A-R task packet
- `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md` - Rust guidance
- `docs/GIT_WORKTREES_AND_LINE_ENDINGS_FOR_AGENTS.md` - Git guidance
