# Current Implementation Task

Control contract: `1`
Task: `M01C4 - Application CLI import suppression cleanup`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Green`
Route: `OpenCode using HY3 for a narrow Rust import-configuration cleanup; Lucy performs independent review`
Base branch: `main`
Base commit: `966ff269ee06f6182bd6029ffe1919b0a43acda8`
Implementation branch: `opencode/m01c4-application-cli-import-suppression`
Worker note: `docs/worker-notes/2026-08-04-m01c4-application-cli-import-suppression.md`
Implementation blueprint: `docs/architecture/M01C4_APPLICATION_CLI_IMPORT_SUPPRESSION_CLEANUP.md`
Rust toolchain: exact `1.97.1`; plain Cargo; `--locked` mandatory
Agent tools: bounded `rg`, Clippy JSON, rustfmt, and ordinary Cargo through `just verify`; do not retry ineffective OpenCode LSP
OCaml switch path: `N/A`
Implementation checkpoint: `976d2519c1629c751f219a246cba0328ac90efb3`

## Objective

Remove the blanket `#[allow(unused_imports)]` attached to the CLI imports at the top of `tethers-0.1/host-rust/src/application.rs`, replacing it with an honest import layout that reflects each symbol's real production, debug-only, or test-only use.

Read `docs/architecture/M01C4_APPLICATION_CLI_IMPORT_SUPPRESSION_CLEANUP.md` completely before editing. It is authoritative.

## Relevant background and existing behaviour

M01C3 is accepted and merged on `main` at:

`40539e3084727e5357a448d9fd3cacd6fd08ce2d`

Accepted baseline:

```text
Rust             1.97.1
Cargo tests      926 passing minimum
Clippy messages  118 emitted warnings after M01C3
Cargo.lock       D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB
```

The exact current warning count must be captured before editing. The historical number is context, not a substitute for current evidence.

The target import currently suppresses unused-import diagnostics across its entire group:

```rust
#[allow(unused_imports)]
use tethers_reference_host::cli::{Cli, CliEnvelope, Command as CliCommand, OutcomeStatus};
```

The task is to encode the actual configuration boundary in the imports, not to change CLI behaviour.

## Required behaviour

1. Remove the target blanket `#[allow(unused_imports)]`.
2. Classify `Cli`, `CliEnvelope`, `CliCommand`, and `OutcomeStatus` through one bounded exact reference search in `application.rs`.
3. Keep always-compiled imports ordinary.
4. Gate test-only or debug-only imports with the narrowest truthful `#[cfg(...)]`.
5. Remove any genuinely unused import.
6. Preserve the `CliCommand` alias if it remains used.
7. Preserve all CLI parsing, command routing, debug probes, test configuration, serialization, output, exit codes, and errors.
8. Finish without a new warning or replacement suppression.

## Relevant components

- `tethers-0.1/host-rust/src/application.rs` — target import block and all local symbol uses.
- `tethers-0.1/host-rust/src/cli.rs` — defines the imported types; read only if needed to understand configuration, never edit.
- `docs/architecture/M01C4_APPLICATION_CLI_IMPORT_SUPPRESSION_CLEANUP.md` — frozen repair rules.
- `.github/scripts/check-tethers-task-packet.ps1` — packet-state checker.
- `justfile` — accepted final Cargo verification route.

## Frozen decisions and invariants

- The task changes import configuration only.
- No CLI type, enum, parser, subcommand, JSON envelope, outcome status, output text, error text, exit code, or runtime route may change.
- Debug-only probes remain available exactly where they were.
- Test-only code remains test-only.
- No replacement `#[allow]` or `#[expect]` is permitted.
- No dependency, lockfile, feature, Rust pin, tool configuration, OCaml, protocol, Plug, Trail, replay, admission, concurrency, or release change is permitted.
- OpenCode LSP is optional infrastructure that has already failed honestly in this workspace. Do not retry it. It has no veto over this task.

## Startup procedure

1. Require a clean worktree:

   ```powershell
   git status --short
   ```

   Stop only if unrelated local changes would be overwritten or make the task unsafe.

2. Fetch remote state:

   ```powershell
   git fetch origin
   ```

3. Verify the planning checkpoint is on remote main:

   ```powershell
   git merge-base --is-ancestor 16988b5b31613cece42714f32fe413c39b9ef977 origin/main
   ```

   Require exit code 0.

4. Verify accepted M01C3 is on remote main:

   ```powershell
   git merge-base --is-ancestor 40539e3084727e5357a448d9fd3cacd6fd08ce2d origin/main
   ```

   Require exit code 0.

5. Confirm the implementation branch does not already contain unrelated work. If absent, create it from current remote main:

   ```powershell
   git switch --create opencode/m01c4-application-cli-import-suppression origin/main
   ```

   If the branch already exists and is exactly this unfinished task, continue it rather than creating a second branch. Stop only if it contains unrelated or ambiguous work.

6. Update the packet Base commit to the exact `origin/main` used to create the implementation branch. Record the same base in the worker note.

7. Read completely before editing:

   - `AGENTS.md`;
   - `docs/CURRENT_CLINE_TASK.md`;
   - `docs/architecture/M01C4_APPLICATION_CLI_IMPORT_SUPPRESSION_CLEANUP.md`;
   - `tethers-0.1/host-rust/src/application.rs`;
   - `tethers-0.1/host-rust/src/cli.rs` only if symbol definitions are needed;
   - `justfile`.

8. Run the packet checker and record the lock hash:

   ```powershell
   pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
   Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256
   ```

## Reference classification

Run one bounded exact search in `application.rs` for the four names:

```powershell
rg -n --glob 'application.rs' '\b(Cli|CliEnvelope|CliCommand|OutcomeStatus)\b' tethers-0.1/host-rust/src
```

Read the surrounding configuration gates and classify every actual use as:

- always compiled;
- `debug_assertions` only;
- test only;
- both debug and test;
- unused.

Record the classification in the worker note. Do not retry LSP, search unrelated repositories, or broaden into CLI redesign.

## Baseline warning capture

Before editing, run one machine-readable locked Clippy capture:

```powershell
$beforeJson = Join-Path $env:TEMP 'm01c4-clippy-before.jsonl'
$beforeErr = Join-Path $env:TEMP 'm01c4-clippy-before.stderr.txt'

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
- any warning whose primary span is the target import block;
- the warning set outside the target for comparison.

The current suppression may mean there is no target warning before editing. That is expected and is not a blocker.

## Required implementation

Implement only the frozen blueprint:

1. Delete the target `#[allow(unused_imports)]`.
2. Arrange imports according to the observed use classification.
3. Prefer a small number of clear imports over scattered fully qualified names.
4. Keep `Command as CliCommand` if the alias is used.
5. If all four symbols are genuinely required in all target configurations, remove only the redundant attribute.
6. If a symbol is unused everywhere, remove it.
7. Do not move functions, change configuration gates around functions, or alter code beyond the import block unless rustfmt changes whitespace mechanically.

## Permitted files

Only these may change:

- `tethers-0.1/host-rust/src/application.rs`;
- `docs/CURRENT_CLINE_TASK.md` for state and checkpoint;
- `docs/worker-notes/2026-08-04-m01c4-application-cli-import-suppression.md`.

Stop before changing another path.

## Forbidden changes

Do not modify `cli.rs`, Cargo.toml, Cargo.lock, dependencies, features, Rust pins, tool versions, tool configuration, Just recipes, PowerShell tooling, Nextest policy, deny policy, OCaml, protocol, request or response JSON, command routing, exit codes, debug-probe availability, tests outside the permitted file, Plug behaviour, Trail, replay, admission, concurrency, release, tag, or publication state.

Do not add any suppression, dummy use, underscore import, `black_box`, unreachable reference, or source-text guard pretending to prove runtime behaviour.

## Stop conditions

Stop as `BLOCKED` only when:

- the branch contains unrelated work that cannot be safely separated;
- removing the suppression exposes a real compile problem that requires an out-of-scope behavioural or configuration change;
- `just verify` exposes a real failure caused by this edit that cannot be corrected inside the permitted import-only scope;
- completing the task would require changing another production file.

Do not stop merely because an optional tool is unavailable, an LSP result is empty, or the exact current warning count differs from the historical note. Record those facts and continue using the compiler-backed path.

After two materially different failed edit attempts, stop with exact evidence and the smallest unresolved question rather than repeating the same action.

## Expected pre-existing changes

None.

## Edit recovery

If an exact replacement fails:

1. reread the current import block;
2. make one fresh small patch against current content;
3. do not rewrite the full file;
4. stop only after two materially different failed attempts.

## Focused feedback

After the coherent import edit:

1. run rustfmt check;
2. run ordinary locked Clippy once and inspect the import diagnostics;
3. run a narrow CLI/application test filter only if the reference classification exposes an existing meaningful filter;
4. otherwise record that focused tests were skipped because import configuration is fully checked by compilation and the final Cargo graph.

Do not invent ceremonial focused tests.

## Final warning accounting

Capture final machine-readable Clippy output:

```powershell
$afterJson = Join-Path $env:TEMP 'm01c4-clippy-after.jsonl'
$afterErr = Join-Path $env:TEMP 'm01c4-clippy-after.stderr.txt'

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

- no warning caused by the new import layout;
- no new or changed warning outside the target import block;
- total emitted warnings unchanged or lower;
- the target blanket suppression absent;
- no replacement suppression added.

Record an exact before/after table in the worker note.

## Required verification

Run only these evidence-bearing checks:

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1

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

Do not run full Nextest, cargo-deny, cargo-machete, `just verify-agent`, OCaml tests, LSP diagnostics, or unrelated scripts.

Expected Cargo floor remains 926 passed and 0 failed. If the repository's authoritative total is higher, record the actual total; no test may disappear because of this task.

## Acceptance criteria

1. The target `#[allow(unused_imports)]` is removed.
2. No new `allow` or `expect` replaces it.
3. All four symbols are classified by their actual configuration uses.
4. The final import layout exactly reflects those uses.
5. Locked all-target Clippy exits zero with no new warning.
6. Total emitted warnings are unchanged or lower.
7. `just verify` passes with no missing test.
8. Cargo.lock hash is unchanged.
9. Only the three permitted files change.
10. No CLI, debug-probe, test, JSON, exit-code, protocol, or runtime behaviour changes.

## Completion contract

After every acceptance condition passes:

1. Create `docs/worker-notes/2026-08-04-m01c4-application-cli-import-suppression.md` with:
   - Requested outcome
   - Changes made
   - Decisions and assumptions
   - Evidence
   - Exact symbol-use classification
   - Before/after warning table
   - Focused-test decision
   - Final Cargo evidence
   - Cargo.lock hash
   - Remaining risks
   - Smallest next action
   - References
2. Record the real implementation commit as `Implementation checkpoint` in both packet and worker note.
3. Set packet and worker note status to `COMPLETE` only after verification passes.
4. Commit normally and push the implementation branch normally.
5. Return a concise handoff containing:
   - outcome;
   - branch and remote tip;
   - implementation checkpoint;
   - changed files;
   - exact import classification and final layout;
   - before/after warning totals;
   - final Cargo total;
   - unchanged Cargo.lock hash;
   - any honest remaining risk.

Do not merge `main`; Lucy performs independent review and, once accepted, has standing permission to fast-forward and push `main` with `force=false`.
