Task: `J13A local process supervision and check command`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `Goose`
Status: `COMPLETE`
Base commit: `f100689a35c9b7032193abd4f737c3203815fa4c`
Implementation checkpoint: `98a09f0fbd9315aa6b3ff96a81925b50bdaf5570`

## Requested outcome

Correct the prior J13A implementation with nine bounded corrections: real MCP
tools/call protocol, bounded protocol reads with timeout/interrupt, correct
Job Object ownership, proper provider shutdown, envelope consistency, no
skip-as-pass tests, comprehensive acceptance script, all required gates,
and control-v1 documentation.

## Changes made

1. Engine MCP protocol: Changed from direct `tethers.validate` method to `tools/call` with `name: "tethers.validate"` and `arguments: {source: "..."}`. Parses `result.structuredContent.valid` and `result.structuredContent.error.{code,message}`.

2. Bounded protocol reads: Replaced direct `BufReader::read_line` with a dedicated stdout reader thread using `mpsc::sync_channel`. `read_protocol_line` uses `recv_timeout` with configurable timeout and checks the interrupt flag on each poll cycle. Enforces 8 MiB line limit before unbounded allocation. Requires strict UTF-8 (rejects NUL bytes, rejects non-UTF-8 via `String::from_utf8`).

3. Windows Job Object ownership: Changed from named to unnamed (`CreateJobObjectW(NULL, NULL)`). Assignment failure is now fatal (kills child, waits, closes handle, returns `JobObjectFailed`). Handle closed exactly once on every shutdown and Drop path. Stored reader-thread `JoinHandle`s and joins them on shutdown.

4. Provider shutdown: `ManagedProvider` owns `Option<SupervisedChild>`. `close()` takes the child via `Option::take()` and calls `SupervisedChild::shutdown()`. Drop remains emergency fallback only.

5. Envelope consistency: `OutcomeStatus` enum with explicit `exit_code()` mapping. Every `CliEnvelope` stores `status` as enum (serialized as string) and `exit_code` as matching integer. Added `error_with_data` constructor for partial evidence on failure.

6. Real engine tests: Tests use correct Tether syntax matching the OCaml engine's expected format. Engine binary required; tests panic if not found.

## Decisions and assumptions

- Engine advertises "tools" capability, not "tethers" (relaxed the initialize check)
- Valid Tether syntax follows the protocol/request.json format with sections
- `SyncSender` bound of 16 lines is sufficient for the check command's sequential usage
- Poll interval of 100ms balances responsiveness with CPU usage for interrupt checking
- `read_until_newline` handles EOF without trailing newline gracefully

## Evidence

- All 99 J12 tests pass
- All 63 J13A tests pass (31 lib + 3 bin + 29 integration)
- cargo fmt --check: PASS
- cargo clippy --all-targets --all-features: 25 warnings, 0 errors
- cargo build --release: PASS
- test-j13a-check.ps1: 8/8 PASS
- check-fixtures.ps1: 46 JSON + 30 JSONL valid
- check-tethers-task-packet.ps1: PASS (after documentation fix)

## Discoveries

- The OCaml engine expects tether syntax with "tether", "anchor", "when", "do" sections
- PowerShell `$Args` automatic variable shadows function parameters - renamed to `$ArgList`
- `BufReader::fill_buf()` borrow checker requires scoped blocks for consume pattern
- Windows test environment may restrict process creation within job hierarchies

## Remaining risks

- Stderr capture test timing sensitivity on slow/loaded machines
- Integration with J12 runtime config requires valid config JSON (not yet in test fixtures)
- The `SyncSender` bound of 16 may be insufficient for high-throughput scenarios (not relevant for check command)
- Some existing stdio_provider tests require pwsh.exe and may fail in restricted environments

## Probe compatibility correction (2026-07-28)

### Original failing checks

- `test-host-event-admission.ps1` — line 258: "missing scenario : output contained JSON when it should have been an error"
- `test-host-event-admission-trail.ps1` — line 287: "relative path must fail"

### Corrected root cause

Two compatibility faults:

1. The PowerShell negative tests expected old unstructured usage text and rejected JSON output. J13A deliberately emits one JSON error envelope per invocation.

2. `run_event_admission_trail_probe_clap` lost the requirement that the Trail path be absolute, allowing relative paths to potentially create directories before the error was returned.

### Exact files changed

- `tethers-0.1/host-rust/src/main.rs` — added `!trail_path.is_absolute()` guard before `fs::create_dir_all`; added `j13a_relative_trail_path_rejected` and `j13a_absolute_trail_path_accepted` tests
- `tethers-0.1/scripts/test-host-event-admission.ps1` — updated `Test-AdmissionProbeFailure` to parse JSON envelopes, verify schema/status/error fields
- `tethers-0.1/scripts/test-host-event-admission-trail.ps1` — updated all six negative scenarios to parse JSON envelopes; added filesystem non-effect proof for relative path
- `docs/CURRENT_CLINE_TASK.md` — status transitions

### Exact rerun results

- `cargo test j13a_` — 65 passed (31 lib + 5 bin + 29 integration), 0 failed
- `cargo test j12_` — 99 passed, 0 failed
- `cargo test` — 686 passed, 0 failed
- `cargo clippy --all-targets --all-features` — 0 errors (pre-existing warnings only)
- `cargo build` / `cargo build --release` — PASS
- `test-host-event-admission.ps1` — all 7 cases PASS
- `test-host-event-admission-trail.ps1` — all 11 cases PASS
- `test-j13a-check.ps1` — 8/8 PASS
- `test-engine.ps1` — 24 cases PASS
- `test-mcp-transcripts.ps1` — 15 cases PASS
- `test-host-denial.ps1` — PASS
- `test-host-execution-failure.ps1` — PASS (with opam env)
- `test-host-result-follow-up.ps1` — PASS
- `demo.ps1` — PASS
- `check-fixtures.ps1` — 46 JSON + 30 JSONL valid
- `check-tethers-task-packet.ps1` — PASS
- `opam exec -- dune build` — PASS

### Skipped required checks: 0
### Unrun required checks: 0

## Smallest next action

J13B: extract typed host execution service and implement the `run` command with
event evaluation, policy decision, and Action dispatch through the retained
provider sessions established by J13A.

## References

- Starting SHA: f100689a35c9b7032193abd4f737c3203815fa4c
- Implementation SHA: 995c649ba43520418ed44a107a1370ac41e97fb6
- Documentation SHA: 98a09f0fbd9315aa6b3ff96a81925b50bdaf5570
- Correction HEAD: 3b22e765ad23709a4d97c15b86aeba9735b5fbcb
- Branch: goose/j13a-process-check
