Task: `M01B - Rust agent tooling foundation`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `BLOCKED`
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

## Decisions and assumptions

The observed local PowerShell directory was prepended only to verification
processes, never to user or machine PATH. The desktop OpenCode executable was
accepted as a discovered candidate only after it was required to execute CLI
commands; its GUI-only behaviour is treated as a failure rather than a proxy for
`debug config` proof.

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
- Cargo.lock SHA-256 remained
  `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB`.

## Discoveries

The only locally discoverable OpenCode executable is
`C:\Users\Matmus\AppData\Local\Programs\@opencode-aidesktop\OpenCode.exe`,
version 1.18.12. It starts the desktop UI for `debug config`; it does not expose
the required console command or a child exit code. No separate `opencode` CLI
was found in PATH, the running process, the desktop data directory, or the
local npm locations inspected.

## Remaining risks

M01B cannot claim OpenCode LSP acceptance until a console-capable OpenCode CLI
is supplied or installed under separate authority. The repository configuration
and wrapper are ready, but the real `debug config` command cannot be proved on
this machine without that executable.

## Smallest next action

Provide the installed console OpenCode CLI path (or explicitly authorise its
installation), then rerun the checker and LSP launcher with `-OpenCodePath` and
change the packet state only if their real effective configuration proof passes.

## References

- Packet: `docs/CURRENT_CLINE_TASK.md`
- Implementation commit: `27caeae7c0513603a53b4ceaebc99f9fe11628f8`
- Checker: `scripts/check-rust-agent-tools.ps1`
- Launcher: `scripts/start-opencode-lsp.ps1`
