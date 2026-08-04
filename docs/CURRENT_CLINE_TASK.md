# Current Implementation Task

Control contract: `1`
Task: `M01C1 - Engine-session warning cleanup pilot`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Amber`
Route: `OpenCode using DeepSeek Pro V4 for a small language-aware Rust warning repair; Lucy performs independent review`
Base branch: `main`
Base commit: `d557d01ab41ddc881b08976de5822c2ccec53f24`
Implementation branch: `opencode/m01c1-engine-session-warning-pilot`
Worker note: `docs/worker-notes/2026-08-04-m01c1-engine-session-warning-pilot.md`
Implementation blueprint: `docs/architecture/M01C1_ENGINE_SESSION_WARNING_PILOT.md`
Rust toolchain: exact `1.97.1`; plain Cargo; `--locked` mandatory
Agent tools: rust-analyzer, cargo-nextest 0.9.140, cargo-deny 0.19.7, cargo-machete 0.9.2
OCaml switch path: `N/A`
Implementation checkpoint: `TBD`

## Objective

Use the new Rust agent toolset on one bounded, behaviour-preserving warning cluster in:

`tethers-0.1/host-rust/src/engine_stdio.rs`

The pilot must prove that OpenCode can use rust-analyzer for real reference discovery, Nextest for focused feedback, ordinary Cargo for final authority, cargo-deny for dependency policy, and cargo-machete for advisory dependency evidence.

Read `docs/architecture/M01C1_ENGINE_SESSION_WARNING_PILOT.md` completely before any edit. It is authoritative.

## Background

M01B is accepted at:

`f7e84a467bf77a02f1f1b60cd319c55644dd9bbd`

Accepted baseline:

```text
Rust             1.97.1
Cargo tests      926 passing
Nextest tests    1133 passing
Nextest retries  0
Cargo.lock       D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB
```

The target module retains a ten-second engine read timeout but currently stores timeout state that is not used by later reads. Accepted Clippy may also report path-reference linting around `EngineSession::launch`. Diagnostics captured under Rust 1.97.1 decide the exact target set.

## Startup procedure

1. Require a clean worktree:

   ```powershell
   git status --short
   ```

   Stop if it is not clean.

2. Fetch remote state:

   ```powershell
   git fetch origin
   ```

3. Verify the frozen M01C1 blueprint is on remote main:

   ```powershell
   git merge-base --is-ancestor ffb34707c6b57b708dc329061fdadd593153f650 origin/main
   ```

   Require exit code 0.

4. Verify accepted M01B is on remote main:

   ```powershell
   git merge-base --is-ancestor f7e84a467bf77a02f1f1b60cd319c55644dd9bbd origin/main
   ```

   Require exit code 0.

5. Inspect the task directly from remote main:

   ```powershell
   git show origin/main:docs/CURRENT_CLINE_TASK.md | Select-Object -First 20
   ```

   Require M01C1, owner OpenCode, status READY, and branch `opencode/m01c1-engine-session-warning-pilot`.

6. Confirm the implementation branch does not exist:

   ```powershell
   git branch --list opencode/m01c1-engine-session-warning-pilot
   git branch --remotes --list origin/opencode/m01c1-engine-session-warning-pilot
   ```

   Stop without overwriting it if either command reports the branch.

7. Create it from current remote main:

   ```powershell
   git switch --create opencode/m01c1-engine-session-warning-pilot origin/main
   ```

8. Update the packet Base commit to the exact current `origin/main` before the implementation commit. Record the same value in the worker note.

9. Read completely before editing:

   - `AGENTS.md`;
   - `docs/CURRENT_CLINE_TASK.md`;
   - `docs/architecture/M01C1_ENGINE_SESSION_WARNING_PILOT.md`;
   - `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md`;
   - `docs/TOOLCHAIN_POLICY.md`;
   - `docs/worker-notes/2026-08-04-m01b-rust-agent-tooling.md`;
   - `tethers-0.1/host-rust/src/engine_stdio.rs`;
   - `tethers-0.1/host-rust/src/check_command.rs`;
   - `tethers-0.1/host-rust/src/host_execution.rs`;
   - `.config/nextest.toml`;
   - `justfile`.

10. Run:

   ```powershell
   pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
   pwsh -NoProfile -File scripts/check-rust-agent-tools.ps1
   Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256
   ```

   Use the accepted real OpenCode CLI through `OPENCODE_BIN` or `-OpenCodePath` if the current shell does not resolve it. Do not change global PATH.

## Mandatory LSP gate

Before any source edit, use OpenCode's LSP tool to obtain:

1. definition and all references for `EngineSession::launch`;
2. definition and all references for `EngineSession::read_timeout`;
3. all call sites of private helper `read_json`.

Record the exact files and reference counts in the worker note.

Text search may confirm the result but does not replace the LSP gate.

If the current process does not expose the LSP tool, start a fresh OpenCode process through `scripts/start-opencode-lsp.ps1` and continue the task there. Do not proceed with text search alone.

## Baseline warning capture

Before editing, run ordinary Clippy and a separate machine-readable capture:

```powershell
$beforeJson = Join-Path $env:TEMP 'm01c1-clippy-before.jsonl'
$beforeErr = Join-Path $env:TEMP 'm01c1-clippy-before.stderr.txt'

cargo clippy `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all-targets `
  --all-features `
  --locked

cargo clippy `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all-targets `
  --all-features `
  --locked `
  --message-format=json `
  1> $beforeJson `
  2> $beforeErr
```

Require both commands to exit zero.

Parse and record:

- total warnings;
- warning counts by lint code;
- every warning whose primary span is `src/engine_stdio.rs`;
- all warnings outside the target file for later comparison.

Do not assume a warning exists merely because the blueprint predicts it.

## Required implementation

Implement only the frozen blueprint.

Required shape:

1. Remove every warning whose primary span is `src/engine_stdio.rs` when the cause can be repaired without protocol or behaviour change.
2. Establish one named ten-second default engine read duration.
3. Use that same default during initialize and store it in the retained session.
4. Make validation and evaluation reads use the stored session timeout.
5. Pass an explicit timeout into the private `read_json` helper rather than leaving the retained field decorative.
6. If accepted Clippy emits `ptr_arg` for `EngineSession::launch`, change `&PathBuf` parameters to `&Path` and update only LSP-proven call sites.
7. Preserve all request JSON, MCP version, method names, IDs, error text, validation, shutdown and stderr-tail behaviour.
8. Add or adjust focused tests only where necessary to prove unchanged behaviour.

Forbidden repair techniques:

- no new `#[allow(...)]` or `#[expect(...)]`;
- no underscore rename to conceal use;
- no dummy read, unreachable use or black-box consumption;
- no sleeps or retries;
- no dependency or configuration change;
- no broad refactor.

## Permitted files

Only these may change:

- `tethers-0.1/host-rust/src/engine_stdio.rs`;
- `tethers-0.1/host-rust/src/check_command.rs` only if the `&Path` signature requires a direct call-site adjustment;
- `tethers-0.1/host-rust/src/host_execution.rs` only for the same reason;
- focused tests already colocated in those files if necessary;
- `docs/CURRENT_CLINE_TASK.md` for state and checkpoint;
- `docs/worker-notes/2026-08-04-m01c1-engine-session-warning-pilot.md`.

Stop before changing another path.

## Forbidden changes

Do not modify Cargo.toml, Cargo.lock, dependencies, features, edition, rust-version, `publish`, Rust pins, tool versions, tool configuration, Just recipes, OpenCode configuration, PowerShell tooling, deny policy, Nextest policy, OCaml, fixtures, production modules outside the permitted list, event-queue Send semantics, Plug installation, J24J, CLI contracts, Trail, Anchor, provider policy, release, tag or publication state.

## Edit recovery

After an exact replacement reports that `oldString` was not found:

1. do not repeat it unchanged;
2. reread the current file;
3. make a fresh smaller patch against the latest content;
4. stop after two materially different failed attempts rather than rewriting the file wholesale.

## Focused feedback loop

After the first coherent edit:

1. run Rustfmt;
2. use Nextest to list and run the narrowest engine-session-related tests with the root config, `--locked`, and zero retries;
3. run ordinary Clippy and inspect target warnings;
4. correct on the same branch if needed.

Do not claim Nextest is faster. Its value here is clearer focused feedback and independent execution.

## Final warning accounting

Capture final machine-readable Clippy output:

```powershell
$afterJson = Join-Path $env:TEMP 'm01c1-clippy-after.jsonl'
$afterErr = Join-Path $env:TEMP 'm01c1-clippy-after.stderr.txt'

cargo clippy `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all-targets `
  --all-features `
  --locked `
  --message-format=json `
  1> $afterJson `
  2> $afterErr
```

Require:

- zero warnings whose primary span is `src/engine_stdio.rs`;
- total warnings lower than before;
- no new warning code or warning instance outside the target file;
- no added suppression attribute.

Record the exact before/after table in the worker note.

## Required verification

Every command below must succeed:

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
pwsh -NoProfile -File scripts/test-check-rust-agent-tools.ps1
pwsh -NoProfile -File scripts/check-rust-agent-tools.ps1

cargo fmt `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all -- --check

cargo clippy `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all-targets `
  --all-features `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all-targets `
  --all-features `
  --locked

cargo nextest run `
  --config-file .config/nextest.toml `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all-targets `
  --all-features `
  --locked

cargo deny --locked `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  check licenses bans sources

cargo deny --locked `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  check advisories

cargo machete --with-metadata tethers-0.1/host-rust

just verify
just verify-agent
just deps-unused

Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256
git diff --check
git status --short
```

Expected full-suite floors remain:

- Cargo: 926 passed, 0 failed;
- Nextest: 1133 passed, 0 failed;
- retries: zero.

If an intentionally added focused test increases a total, record and explain the exact increase. No test may disappear.

## Acceptance criteria

1. LSP evidence precedes editing and is recorded.
2. Exact before/after warning inventories are recorded.
3. `src/engine_stdio.rs` emits no warning.
4. No warning outside the target file is added or altered unexpectedly.
5. Total warning count decreases.
6. Ten seconds remains the effective default for initialize, validate and evaluate reads.
7. No warning suppression or fake use is introduced.
8. Cargo and Nextest complete with zero failures and zero retries.
9. Cargo-deny and cargo-machete complete successfully.
10. Cargo.lock hash is unchanged.
11. No dependency, configuration, OCaml, protocol, CLI, lifecycle, concurrency or Tethers behaviour changes.
12. The worker note records what each new tool contributed and whether its result was useful.

## Completion contract

After every acceptance condition passes:

1. Create `docs/worker-notes/2026-08-04-m01c1-engine-session-warning-pilot.md` with control-v1 header and sections:
   - Requested outcome
   - Changes made
   - Decisions and assumptions
   - LSP evidence
   - Warning inventory before/after
   - Test and policy evidence
   - Tool usefulness assessment
   - Discoveries
   - Remaining risks
   - Smallest next action
   - References
2. Set the single packet status field to `COMPLETE` and checkpoint to `TBD`.
3. Make one normal implementation commit.
4. Obtain and verify its real SHA:

   ```powershell
   git cat-file -e <REAL_SHA>^{commit}
   ```

5. Record that exact SHA in the packet and worker note.
6. Make a separate completion-documentation commit.
7. Push normally.

Do not amend, reset, rebase, cherry-pick, force-push, merge into main, tag or publish.

Return branch, remote tip, verified implementation checkpoint, exact changed files, LSP reference results, warning before/after table, focused Nextest command/result, complete Cargo and Nextest totals, deny and machete results, Cargo.lock hashes, tool usefulness assessment, worker-note path, and confirmation that no forbidden behaviour changed.
