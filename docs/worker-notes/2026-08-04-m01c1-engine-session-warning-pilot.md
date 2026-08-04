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

The original implementation preceded native LSP evidence and used `rg`; that history is retained rather than rewritten. Correction-time native LSP verification was attempted through a fresh console OpenCode 1.18.12 process launched by `scripts/start-opencode-lsp.ps1` with the accepted npm executable. `OPENCODE_EXPERIMENTAL_LSP_TOOL=true` and `OPENCODE_DISABLE_LSP_DOWNLOAD=true` were set only within that wrapped process.

- OpenCode native `lsp` `goToDefinition` and `findReferences` calls for `EngineSession::launch`, `EngineSession::read_timeout`, and `read_json` all returned `No results found`.
- Native `lsp` `hover` for `EngineSession::launch` returned `[null]`; `documentSymbol` and `workspaceSymbol` returned no results.
- A direct wrapped `opencode debug lsp document-symbols file:///D:/The%20Next%20Thing/Tethers%20Lang%20-%20Goose%20Integration/tethers-0.1/host-rust/src/engine_stdio.rs` returned `[]` with exit code zero.
- A second fresh wrapped session was instructed to query zero-based positions at real call sites; it timed out after 244 seconds without usable LSP results.

Therefore the LSP trial is recorded as ineffective tooling and was not retried. The existing verified `rg` fallback remains the usable discovery proof: `EngineSession::launch` has one definition and seven references (two non-test call sites and five test calls); `read_json` has one definition and three call sites; `read_timeout` is traced from its declaration through initialization and retained-session uses. The accepted console LSP operation is exposed but does not initialise or index this Rust workspace under the frozen configuration.

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

## Evidence

- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` while `IN_PROGRESS` — PASS (`control-v1/IN_PROGRESS`).
- `pwsh -NoProfile -File scripts/test-check-rust-agent-tools.ps1` — PASS (10 passed, 0 failed).
- `pwsh -NoProfile -File scripts/check-rust-agent-tools.ps1 -OpenCodePath C:\Users\Matmus\AppData\Roaming\npm\opencode.cmd` — PASS (15 passed, 0 failed); OpenCode 1.18.12 and effective LSP configuration confirmed.
- `cargo fmt --check` and locked `cargo clippy --all-targets --all-features` — PASS; `engine_stdio.rs` has 0 warnings, with no new warnings elsewhere.
- locked `cargo test --all-targets --all-features` — PASS: 926 passed, 0 failed.
- locked `cargo nextest run` with `.config/nextest.toml` — PASS: 1133 passed, 0 failed, 0 retries.
- `cargo deny` licences/bans/sources and advisories — PASS; `cargo machete --with-metadata` — PASS (no unused dependencies).
- `just verify` and `just verify-agent` — PASS.
- Cargo.lock SHA-256: `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB` (unchanged).
- `git diff --check` — PASS; final status before the documentation commit listed only these two permitted documentation files.

## Test and policy evidence

- Cargo test: 926 passed, 0 failed.
- Nextest: 1133 passed, 0 failed, 0 retries (root config sets `retries = 0`).
- Cargo-deny licences/bans/sources: passed; advisories: passed.
- Cargo-machete: no unused dependencies.
- Rustfmt: clean; Clippy: target warnings 3 to 0, with no suppression and no new warnings outside the target.
- `just verify`: passed. `just verify-agent`: passed.

## Tool usefulness assessment

| Tool | Used | Useful | Notes |
|------|------|--------|-------|
| rust-analyzer / OpenCode LSP | Attempted | No | Native operation was exposed but returned no symbols, definitions, references, or hover data for the Rust workspace; it cannot supply usable reference proof. |
| cargo-nextest | Yes | Yes | Clear per-test reporting, isolated process execution, simple filter by test name |
| cargo-deny | Yes | Yes | Single-pass licence/ban/source/advisory verification |
| cargo-machete | Yes | Yes | Confirmed zero unused dependencies |

## Discoveries

- The packet checker required the six missing control-v1 sections; they were added accurately and passed while the task was `IN_PROGRESS`.
- Fresh wrapped OpenCode native LSP is configured and callable, but it returned no Rust workspace symbols or navigation results. The direct document-symbol debug command with a correct file URI also returned an empty array.
- `PathBuf` import was needed only by the test module after the signature change, requiring a distinct import placement.
- Process-local `$PSHOME` restoration made `pwsh.exe` available to the verification processes; no user or machine PATH was changed.

## Remaining risks

None specific to M01C1. The LSP trial was ineffective, but the verified `rg` fallback and all required acceptance checks are recorded.

## Smallest next action

Lucy reviews the pushed M01C1 evidence and decides acceptance or the next separately compiled task.

## References

- Packet: `docs/CURRENT_CLINE_TASK.md`
- Blueprint: `docs/architecture/M01C1_ENGINE_SESSION_WARNING_PILOT.md`
- Implementation commit: `1083e7be5bef5fca78ec9d33fe725b6709f46636`
- M01B worker note: `docs/worker-notes/2026-08-04-m01b-rust-agent-tooling.md`
