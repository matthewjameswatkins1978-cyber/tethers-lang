Task: `J13A local process supervision and check command`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `Codex`
Status: `COMPLETE`
Base commit: `f100689a35c9b7032193abd4f737c3203815fa4c`
Implementation checkpoint: `9de4a99444dab20e7b016cd339ced5cc0873197c`

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

## Partial-evidence closure and public acceptance (2026-07-29)

### Exact files changed

- `tethers-0.1/host-rust/src/check_command.rs`
- `tethers-0.1/scripts/test-j13a-check.ps1`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-07-28-j13a-process-check.md`

### Partial-evidence result

Provider checking now returns accumulated provider values plus a typed
`CheckFailure`. `run_check` builds the canonical data and creates the final
envelope. Invalid Tethers, provider launch/initialize/tools-list failures,
capability mismatch, later-provider failure, and interruption retain all
completed evidence. Status, embedded exit code, and process exit code derive
from the same `OutcomeStatus`.

Before correction, the missing-tool public output was:

```json
{"schema":"tethers.cli/1","command":"check","status":"unavailable","exit_code":4,"data":{},"error":{"code":"PROVIDER_CAPABILITY_UNAVAILABLE","message":"provider 0 (tethers-stdio-fixture): capability unavailable","field":"/providers/0"}}
```

After correction, it was:

```json
{"schema":"tethers.cli/1","command":"check","status":"unavailable","exit_code":4,"data":{"config":{"provider_count":1,"tether_count":1,"tether_set_id":"fixture.acceptance","tether_set_version":"1"},"providers":[{"capabilities":[{"error":"MCP protocol error: tools/list did not contain trusted binding tool 'fixture_ping'","name":"fixture.ping","status":"unavailable","version":1}],"identity":"tethers-stdio-fixture","index":0,"status":"unavailable"}],"tethers":[{"id":"fixture-valid","index":0,"status":"valid","version":"1"}]},"error":{"code":"PROVIDER_CAPABILITY_UNAVAILABLE","message":"provider 0 (tethers-stdio-fixture): capability unavailable","field":"/providers/0"}}
```

### Public-boundary acceptance

`test-j13a-check.ps1`: 25 passed, 0 failed.

1. Valid check returns ok and exit 0: PASS
2. Exactly one JSON document: PASS
3. Config and engine paths containing spaces: PASS
4. Caller CWD differs from config directory: PASS
5. Provider CWD equals canonical config directory: PASS
6. Provider initialize marker exactly once: PASS
7. Provider tools/list marker exactly once: PASS
8. Provider marker contains no tools/call: PASS
9. Engine marker records one validate per Tether and no evaluate: PASS
10. Invalid Tether returns invalid_data and launches no provider: PASS
11. Missing tool retains prior valid-Tether evidence: PASS
12. Later provider failure retains earlier successful provider: PASS
13. Engine initialize hang is bounded: PASS (12037 ms)
14. Provider initialize hang is bounded: PASS (12237 ms)
15. Provider tools/list hang is bounded: PASS (12562 ms)
16. Provider stdout contamination fails closed: PASS
17. Oversized provider line fails closed: PASS
18. Ctrl+C during blocked reading returns interrupted/10: PASS (106 ms)
19. Direct child cleanup: PASS (PID 18536 gone)
20. Descendant cleanup: PASS (PID 25840 gone)
21. Check creates no Trail or replay state: PASS (0 changes)
22. Unknown command returns exit 2: PASS
23. Misspelled `runn` returns exit 2: PASS
24. Hidden `__legacy` remains reachable: PASS
25. Envelope contains no timestamp: PASS

Real-boundary proof:

- Config: generated `valid primary config.json` using the current J12 runtime
  schema under a temporary `Tethers J13A Ω acceptance with spaces ...`
  directory.
- Engine:
  `tethers-0.1/engine-ocaml/_build/default/bin/tethers_mcp_main.exe`
- Manifest:
  `tethers-0.1/protocol/capability-manifests/fixture-ping.json`
- Manifest digest:
  `sha256:01fed7a4b877dd82abe91a1b6cfcd476b02e4c115489e70cbb285b8bf2d32d8b`
- Provider:
  `pwsh.exe -NoProfile -File scripts/tethers-stdio-fixture.ps1`
- Engine marker: `initialize,tools/call:tethers.validate`
- Provider marker: `initialize,tools/list`

### Final verification

- `cargo fmt --check`: PASS
- `cargo check`: PASS
- `cargo check --tests`: PASS
- `cargo test j13a_ -- --nocapture`: 74 passed, 0 failed
  (31 lib + 14 bin + 29 integration)
- `cargo test j12_ -- --nocapture`: 99 passed, 0 failed
- `cargo test`: 695 passed, 0 failed (31 lib + 635 bin + 29 integration)
- `cargo clippy --all-targets --all-features`: PASS, 0 errors
- `cargo build`: PASS
- `cargo build --release`: PASS
- `test-engine.ps1`: 24 fixture cases plus deterministic repeat PASS
- `test-mcp-transcripts.ps1`: 15 cases PASS
- `test-host-denial.ps1`: PASS
- `test-host-execution-failure.ps1`: PASS
- `test-host-result-follow-up.ps1`: PASS
- `test-host-event-admission.ps1`: PASS
- `test-host-event-admission-trail.ps1`: PASS
- `demo.ps1`: PASS
- `check-fixtures.ps1`: 46 JSON and 30 JSONL valid
- `check-tethers-task-packet.ps1`: PASS
  (`control-v1/COMPLETE`, base `f100689`, HEAD `9de4a99`)
- `opam exec -- dune build`: PASS
- `git diff --check`: exit 0; stdout empty; stderr contained only:
  - `warning: in the working copy of 'docs/CURRENT_CLINE_TASK.md', LF will be replaced by CRLF the next time Git touches it`
  - `warning: in the working copy of 'docs/worker-notes/2026-07-28-j13a-process-check.md', LF will be replaced by CRLF the next time Git touches it`
- Skipped required checks: 0
- Unrun required checks: 0

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
