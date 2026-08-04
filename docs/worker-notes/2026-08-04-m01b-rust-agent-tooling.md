Task: `M01B - Rust agent tooling foundation`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `57e709f7c3fd0a85fdf52d5f027bbd4bdf9af5bf`
Implementation checkpoint: `27caeae7c0513603a53b4ceaebc99f9fe11628f8`

## Requested outcome

Correct M01B’s control state and tooling proof: fail closed without OpenCode,
use a local executable without altering PATH, make Cargo-deny and both Rust test
graphs pass, retain one explicit Nextest configuration, and preserve Cargo.lock.

## Changes made

- Restored one packet status and checkpoint field, then recorded the correction
  implementation SHA.
- Added strict tool-JSON validation, OpenCode resolution by explicit path,
  `OPENCODE_BIN`, then PATH, and fail-closed configuration checks.
- Updated the LSP launcher to use the same resolution order and to restore its
  two process-local environment variables.
- Added only `publish = false` to the host package metadata; removed the false
  licence exception; kept one root Nextest configuration and explicit locked
  Just recipes.
- Serialized Nextest because its parallel Windows processes race the inherited
  handle-isolation regression; retries remain zero.
- Verified the console OpenCode CLI installed by the user at
  `C:\Users\Matmus\AppData\Roaming\npm\node_modules\opencode-ai\node_modules\opencode-windows-x64\bin\opencode.exe`
  (1.18.12) through both the checker and the LSP launcher.

## Decisions and assumptions

The observed local PowerShell directory was prepended only to verification
processes, never to user or machine PATH. The desktop OpenCode executable was
accepted as a discovered candidate only after it was required to execute CLI
commands; its GUI-only behaviour is treated as a failure rather than a proxy for
`debug config` proof. The user-installed console CLI is supplied explicitly (or
via `OPENCODE_BIN`), so the running shell does not need a refreshed global PATH.

## Evidence

- `cargo test --all-targets --all-features --locked`: 926 passed, 0 failed.
- `cargo nextest run --config-file .config/nextest.toml ... --locked`: 1133
  passed, 0 failed with zero retries.
- `cargo deny --locked ... check licenses bans sources` and `check advisories`:
  passed.
- `cargo machete --with-metadata`: no unused dependencies.
- Focused checker tests: 10 passed, 0 failed; missing and invalid OpenCode
  paths fail closed, and an explicit test executable proves effective LSP
  configuration handling.
- Real CLI proof: checker and `just agent-tools` each reported 15 passed,
  including OpenCode 1.18.12 and effective configuration with `lsp: true` and
  `permission.lsp: "allow"`; the launcher produced the same `debug config`
  result and returned successfully.
- `just verify-agent` completed successfully with the real console CLI: 926
  ordinary Cargo tests and 1133 Nextest tests passed, followed by locked
  Cargo-deny licence/source/ban/advisory gates.
- Cargo.lock SHA-256 remained
  `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB`.

## Discoveries

The desktop executable at
`C:\Users\Matmus\AppData\Local\Programs\@opencode-aidesktop\OpenCode.exe`
remains GUI-only for `debug config`. The separately installed npm console CLI at
the explicit path above is version 1.18.12 and exposes the required command.
Repository-local explicit resolution and `OPENCODE_BIN` work immediately; a
pre-existing process may still need a new shell before its PATH sees the npm
shim.

## Remaining risks

No M01B acceptance blocker remains. During repeated diagnostics, the pre-existing
Windows inherited-handle isolation test was intermittent; no source, test, retry,
or exception change was made. The required final unmodified aggregate completed
with 926 Cargo and 1133 Nextest tests passing.

## Smallest next action

Review and merge the completed branch normally. Supply `-OpenCodePath` or
`OPENCODE_BIN` in a process that predates the npm installation until a new shell
inherits the updated PATH.

## References

- Packet: `docs/CURRENT_CLINE_TASK.md`
- Implementation commit: `27caeae7c0513603a53b4ceaebc99f9fe11628f8`
- Checker: `scripts/check-rust-agent-tools.ps1`
- Launcher: `scripts/start-opencode-lsp.ps1`
