# Current Implementation Task

Control contract: `1`
Task: `M01C2 - Event-queue contract warning cleanup`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Green`
Route: `OpenCode using HY3 for a narrow Rust test-and-comment correction; Lucy performs independent review`
Base branch: `main`
Base commit: `2bbbe3c84d65d5610dd417b00e0a8c711904ecf7`
Implementation branch: `opencode/m01c2-event-queue-contract-warning`
Worker note: `docs/worker-notes/2026-08-04-m01c2-event-queue-contract-warning.md`
Implementation blueprint: `docs/architecture/M01C2_EVENT_QUEUE_CONTRACT_WARNING_CLEANUP.md`
Rust toolchain: exact `1.97.1`; plain Cargo; `--locked` mandatory
Agent tools: cargo-nextest 0.9.140 and accepted Rust tool checker; do not retry ineffective OpenCode LSP
OCaml switch path: `N/A`
Implementation checkpoint: `b3fdc1cd6f34874e95c0ebc413d8d99a4343c4e4`

## Objective

Remove the warning in `tethers-0.1/host-rust/src/event_queue.rs` by replacing a misleading non-proving `!Send` test with a truthful compile-time assertion, while preserving the actual J10 contract: FIFO, coordinator-driven serial evaluation, no recursion, no retry, and no parallel worker.

Read `docs/architecture/M01C2_EVENT_QUEUE_CONTRACT_WARNING_CLEANUP.md` completely before editing. It is authoritative.

## Relevant background and existing behaviour

M01C1 is accepted on `main` at:

`2fbebfc14c8e2c55990f2bdfc8e85830da339b65`

Accepted baseline:

```text
Rust             1.97.1
Cargo tests      926 passing
Nextest tests    1133 passing
Nextest retries  0
Cargo.lock       D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB
```

J10 established serial follow-up evaluation through an explicit FIFO queue and coordinator drain. It did not establish a type-level `!Send` guarantee. The current `queue_cannot_be_moved_across_threads` test defines an `assert_send<T: Send>()` helper but does not call it, and its comments claim a guarantee the type does not enforce.

OpenCode LSP has already been tested and found ineffective for this Rust workspace. Do not retry it. This job uses exact Clippy diagnostics and `rg` only where textual confirmation is useful.

## Required behaviour

1. Capture the actual Rust 1.97.1 Clippy warning baseline before editing.
2. Identify every warning whose primary span is `src/event_queue.rs`.
3. Replace the false/non-proving `queue_cannot_be_moved_across_threads` test with a truthful compile-time `Send` assertion for the current representation.
4. State clearly that `Send` means the value may be moved between threads, while Tethers seriality remains a coordinator policy and execution design.
5. Keep the event-queue test count unchanged.
6. Remove every warning whose primary span is `src/event_queue.rs` when caused by this test.
7. Preserve production queue code and all runtime behaviour exactly.
8. Use focused Nextest during the edit loop and ordinary Cargo through `just verify` as final authority.
9. Skip irrelevant dependency scans because dependency and policy files are forbidden from changing.

## Relevant components

- `tethers-0.1/host-rust/src/event_queue.rs` — target queue and colocated tests.
- `tethers-0.1/host-rust/src/result_anchor.rs` — queue payload definition, read-only for understanding only.
- `docs/worker-notes/2026-07-27-j10-result-event-queue.md` — accepted serial queue contract.
- `.config/nextest.toml` — committed zero-retry focused test configuration.
- `.github/scripts/check-tethers-task-packet.ps1` — control-v1 checker.
- `scripts/check-rust-agent-tools.ps1` — accepted tool presence/configuration checker.
- `justfile` — final ordinary Cargo verification route.

## Frozen decisions and invariants

- FIFO remains `push_back` and `pop_front`.
- The queue remains process-local and in-memory.
- Follow-up events remain coordinator-driven and one-at-a-time.
- No recursion, retry, thread, async runtime, channel, lock, worker pool, parallel evaluation, persistence, or scheduler is added.
- Do not force `ResultEventQueue` to become `!Send` with marker fields or non-`Send` payloads.
- Result Anchor identity, causation, correlation, generation, admission and serialization remain unchanged.
- No public CLI, language, Plug, Trail, capability, protocol, dependency, tool or configuration behaviour changes.

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

3. Verify the M01C2 blueprint is on remote main:

   ```powershell
   git merge-base --is-ancestor 09ee5ab32f3f34c237b247a3bafbbb573325dadc origin/main
   ```

   Require exit code 0.

4. Verify accepted M01C1 is on remote main:

   ```powershell
   git merge-base --is-ancestor 2fbebfc14c8e2c55990f2bdfc8e85830da339b65 origin/main
   ```

   Require exit code 0.

5. Inspect the packet directly from remote main:

   ```powershell
   git show origin/main:docs/CURRENT_CLINE_TASK.md | Select-Object -First 20
   ```

   Require M01C2, owner OpenCode, status READY, colour Green, and branch `opencode/m01c2-event-queue-contract-warning`.

6. Confirm the implementation branch does not exist:

   ```powershell
   git branch --list opencode/m01c2-event-queue-contract-warning
   git branch --remotes --list origin/opencode/m01c2-event-queue-contract-warning
   ```

   Stop without overwriting it if either command reports the branch.

7. Create it from current remote main:

   ```powershell
   git switch --create opencode/m01c2-event-queue-contract-warning origin/main
   ```

8. Update the packet Base commit to the exact current `origin/main` before the implementation commit. Record the same value in the worker note.

9. Read completely before editing:

   - `AGENTS.md`;
   - `docs/CURRENT_CLINE_TASK.md`;
   - `docs/architecture/M01C2_EVENT_QUEUE_CONTRACT_WARNING_CLEANUP.md`;
   - `docs/worker-notes/2026-07-27-j10-result-event-queue.md`;
   - `docs/worker-notes/2026-08-04-m01c1-engine-session-warning-pilot.md`;
   - `tethers-0.1/host-rust/src/event_queue.rs`;
   - `tethers-0.1/host-rust/src/result_anchor.rs`;
   - `.config/nextest.toml`;
   - `justfile`.

10. Restore PowerShell resolution process-locally if required:

   ```powershell
   $pwshExe = Join-Path $PSHOME 'pwsh.exe'
   if (-not (Test-Path -LiteralPath $pwshExe -PathType Leaf)) {
       throw "pwsh.exe not found under PSHOME: $PSHOME"
   }
   $env:PATH = "$PSHOME;$env:PATH"
   Get-Command pwsh.exe -CommandType Application -ErrorAction Stop
   ```

   Do not change user or machine PATH.

11. Run startup checks:

   ```powershell
   pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
   pwsh -NoProfile -File scripts/check-rust-agent-tools.ps1
   Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256
   ```

## Baseline warning capture

Before editing:

```powershell
$beforeJson = Join-Path $env:TEMP 'm01c2-clippy-before.jsonl'
$beforeErr = Join-Path $env:TEMP 'm01c2-clippy-before.stderr.txt'

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

Require exit code 0. Record:

- total emitted warnings;
- warning counts by lint code;
- unique warnings whose primary span is `src/event_queue.rs`;
- warnings outside the target file.

Trust the captured diagnostics. If `event_queue.rs` has no warning, stop as `BLOCKED` with the exact evidence rather than inventing work.

## Required implementation

In `event_queue.rs` only:

1. Replace `queue_cannot_be_moved_across_threads` with a truthfully named test.
2. Use a real compile-time assertion:

   ```rust
   fn assert_send<T: Send>() {}
   assert_send::<ResultEventQueue>();
   ```

3. Explain in the test comments that current `Send` capability does not create parallel evaluation. Seriality is enforced by the coordinator and explicit mutable queue drain.
4. Keep the number of event-queue tests unchanged.
5. Remove misleading claims that `ResultAnchor` or `ResultEventQueue` is intentionally `!Send`.
6. Do not modify production queue fields, methods, visibility, or implementation.

## Permitted files

Only:

- `tethers-0.1/host-rust/src/event_queue.rs`;
- `docs/CURRENT_CLINE_TASK.md` for state and checkpoint;
- `docs/worker-notes/2026-08-04-m01c2-event-queue-contract-warning.md`.

Stop before changing another path.

## Forbidden changes

Do not modify production queue code, `ResultAnchor`, application/coordinator code, dispatch, replay, admission, Cargo.toml, Cargo.lock, dependencies, features, Rust pins, tool versions, tool configuration, deny policy, Nextest policy, Just recipes, OpenCode configuration, PowerShell tooling, OCaml, fixtures, Plug installation, J24J, CLI contracts, Trail, Anchor, release, tag or publication state.

Do not add:

- `PhantomData`, `Rc`, `Cell`, raw pointers or another artificial non-`Send` marker;
- `#[allow(...)]` or `#[expect(...)]`;
- underscore renaming, dummy use, unreachable use or `black_box`;
- threads, async code, channels, locks, workers, retries or sleeps;
- source-text tests pretending to prove type behaviour.

## Stop conditions

- Stop if the captured warning is not in `event_queue.rs` or is unrelated to the false test.
- Stop if the compile-time `Send` assertion does not compile.
- Stop if fixing the warning would require production source or an out-of-scope file.
- Stop if focused Nextest or final `just verify` has any failure.
- After two materially different failed edits, stop with exact evidence rather than rewriting the file wholesale.

## Expected pre-existing changes

None.

## Edit recovery

After an exact replacement reports `oldString` was not found:

1. do not repeat the same edit;
2. reread the current file;
3. create a smaller patch against stable surrounding text;
4. stop after two materially different failures.

## Focused feedback loop

After the single coherent edit:

```powershell
cargo fmt `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all -- --check

cargo nextest list `
  --config-file .config/nextest.toml `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all-targets `
  --all-features `
  --locked `
  -E 'test(event_queue::)'

cargo nextest run `
  --config-file .config/nextest.toml `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all-targets `
  --all-features `
  --locked `
  -E 'test(event_queue::)'
```

Require zero failures and zero retries. Record the exact listed and passed test counts.

## Final warning accounting

```powershell
$afterJson = Join-Path $env:TEMP 'm01c2-clippy-after.jsonl'
$afterErr = Join-Path $env:TEMP 'm01c2-clippy-after.stderr.txt'

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

- zero warnings whose primary span is `src/event_queue.rs`;
- no new or changed warning outside the target;
- no suppression attribute;
- lower total emitted warning count if the expected warning was present.

Explain duplicate emitted messages when the same unique source warning is compiled for multiple targets.

## Required verification

Run only these evidence-bearing checks:

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
pwsh -NoProfile -File scripts/check-rust-agent-tools.ps1

cargo fmt `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all -- --check

cargo clippy `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all-targets `
  --all-features `
  --locked

just verify

Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256
git diff --check
git status --short
```

Required final Cargo floor:

```text
926 passed
0 failed
```

Do not run full Nextest, cargo-deny, cargo-machete, `just verify-agent`, OCaml tests, or unrelated scripts. They cannot add relevant evidence to a permitted test-and-comment-only diff. If an unexpected dependency, policy, configuration or runtime file changes, stop instead of broadening verification.

## Acceptance criteria

1. The exact `event_queue.rs` warning is captured before editing.
2. The misleading non-proving `!Send` test is replaced by a real current-representation `Send` assertion.
3. Comments correctly separate type mobility from serial coordinator policy.
4. Event-queue test count is unchanged.
5. Focused Nextest passes with zero retries.
6. `event_queue.rs` has zero warnings afterward.
7. No warning outside the target changes.
8. No suppression or fake use is added.
9. `just verify` passes with 926 Cargo tests and zero failures.
10. Cargo.lock hash is unchanged.
11. Only the three permitted files change.
12. No runtime, dependency, protocol, CLI, language, Plug, Trail, replay, admission or concurrency behaviour changes.

## Completion contract

After every acceptance condition passes:

1. Create `docs/worker-notes/2026-08-04-m01c2-event-queue-contract-warning.md` with the control-v1 header and sections:
   - Requested outcome
   - Changes made
   - Decisions and assumptions
   - Warning evidence before and after
   - Focused Nextest evidence
   - Final Cargo evidence
   - Tool usefulness
   - Discoveries
   - Remaining risks
   - Smallest next action
   - References
2. Set the single packet status to `COMPLETE`, colour to `Green`, and checkpoint to `TBD`.
3. Make one normal implementation commit.
4. Verify its real SHA:

   ```powershell
   git cat-file -e <REAL_SHA>^{commit}
   ```

5. Record that exact SHA in the packet and worker note.
6. Make completion documentation a separate normal commit.
7. Push normally.

Do not amend, reset, rebase, cherry-pick, force-push, merge into main, tag or publish.

Return:

- branch and remote tip;
- implementation checkpoint;
- exact changed files;
- unique warning and emitted-message counts before and after;
- focused Nextest listed/passed totals and zero retries;
- final Cargo total;
- Cargo.lock hashes;
- confirmation that dependency scans were intentionally skipped because no dependency or policy path changed;
- worker-note path;
- confirmation that no production source or runtime behaviour changed.
