Task: `M01C1 - Engine-session warning cleanup pilot`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `d557d01ab41ddc881b08976de5822c2ccec53f24`
Implementation checkpoint: `1083e7be5bef5fca78ec9d33fe725b6709f46636`

## Requested outcome

Use the new Rust agent toolset on one bounded, behaviour-preserving warning cluster in `tethers-0.1/host-rust/src/engine_stdio.rs`. Remove every warning whose primary span is in the target file without behaviour or protocol change, exercise LSP discovery, Nextest feedback, Cargo final authority, cargo-deny policy gates, and cargo-machete evidence.

## Changes made

- `tethers-0.1/host-rust/src/engine_stdio.rs`: changed `EngineSession::launch` signature from `&PathBuf` to `&Path` for both parameters (`clippy::ptr_arg` fix). Added `DEFAULT_ENGINE_READ_TIMEOUT` constant (10 seconds). Changed `read_json` to accept an explicit `timeout: Duration` parameter. Initialization, validation, and evaluation reads now all use the constant or `self.read_timeout` respectively, eliminating the dead-code warning on the field. Moved `PathBuf` import into the test module since it is only used there after the signature change.
- `tethers-0.1/host-rust/src/host_execution.rs`: updated the single call site (line 431) to pass `self.engine_path` directly instead of `&self.engine_path.to_path_buf()`.
- `docs/CURRENT_CLINE_TASK.md`: updated Base commit to current `origin/main` (`d557d01ab41ddc881b08976de5822c2ccec53f24`).

## Decisions and assumptions

- LSP was enabled in configuration (`lsp: true`, `permission.lsp: "allow"`) but the current process does not expose the native LSP tool. Text search (`rg`) was used as the best available reference discovery and confirmed the expected call sites (`check_command.rs:132`, `host_execution.rs:431`, plus 5 test calls in `engine_stdio.rs`). The `launch.rsp` helper was moved because this process was started without the `start-opencode-lsp.ps1` launcher.
- `check_command.rs` required no change because `&PathBuf` auto-coerces to `&Path`.
- `ptr_arg` was applied to both parameters as Clippy suggested two separate warnings.

## LSP evidence

Derived via text search (LSP-enabled process not available):
- `EngineSession::launch`: definition at `engine_stdio.rs:83`; 7 references (2 non-test call sites, 5 test calls).
- `EngineSession::read_timeout`: field declared at `engine_stdio.rs:78`, assigned at `engine_stdio.rs:146`.
- `read_json`: defined at `engine_stdio.rs:275`; 3 call sites (initialize at line 109, validate at line 175, evaluate at line 240).

## Warning inventory before/after

| Metric | Before | After |
|--------|--------|-------|
| Total warnings (JSON) | 126 | 120 |
| `src/engine_stdio.rs` warnings | 3 (1 × dead_code, 2 × ptr_arg) | 0 |
| New warnings outside target | 0 | 0 |
| New suppression attributes | 0 | 0 |

Warning detail for target file:

Before:
- `dead_code` on field `EngineSession::read_timeout` (line 78)
- `clippy::ptr_arg` on parameter `engine_path: &PathBuf` (line 83)
- `clippy::ptr_arg` on parameter `working_dir: &PathBuf` (line 83)

After: none.

## Test and policy evidence

- Cargo test: 921 passed, 5 failed (pre-existing `execution_environment` failures, identical on base commit).
- Nextest: 1128 passed, 5 failed, 0 retries.
- Focused engine-session tests: 5 passed, 0 failed (`cargo nextest run -E 'test(engine_stdio::)'`).
- Cargo-deny licences/bans/sources: passed.
- Cargo-deny advisories: passed.
- Cargo-machete: no unused dependencies.
- Cargo.lock SHA-256: `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB` (unchanged).
- Rustfmt: clean.
- `git diff --check`: clean.

## Tool usefulness assessment

| Tool | Used | Useful | Notes |
|------|------|--------|-------|
| rust-analyzer / LSP | Not available | N/A | Config enabled but process lacks native tool; text search used |
| cargo-nextest | Yes | Yes | Clear per-test reporting, isolated process execution, simple filter by test name |
| cargo-deny | Yes | Yes | Single-pass licence/ban/source/advisory verification |
| cargo-machete | Yes | Yes | Confirmed zero unused dependencies |

## Discoveries

- The packet checker (`check-tethers-task-packet.ps1`) requires additional sections (`Relevant background and existing behaviour`, `Required behaviour`, `Relevant components`, `Frozen decisions and invariants`, `Stop conditions`, `Expected pre-existing changes`) that the current M01C1 packet was compiled without. This causes `just verify` to fail on a pre-existing format issue.
- Five `execution_environment` tests fail on this Windows machine with both the base commit and the implementation branch. These are pre-existing PowerShell process infrastructure failures, not caused by M01C1 changes.
- `PathBuf` import was needed only by the test module after the signature change, requiring a distinct import placement.

## Remaining risks

None specific to M01C1. The 5 pre-existing `execution_environment` test failures should be investigated separately.

## Smallest next action

Lucy inspects pushed evidence and decides accept, correct, or escalate. Matthew routes the next task to the appropriate worker.

## References

- Packet: `docs/CURRENT_CLINE_TASK.md`
- Blueprint: `docs/architecture/M01C1_ENGINE_SESSION_WARNING_PILOT.md`
- Implementation commit: `1083e7be5bef5fca78ec9d33fe725b6709f46636`
- M01B worker note: `docs/worker-notes/2026-08-04-m01b-rust-agent-tooling.md`
