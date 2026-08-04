# Current Implementation Task

Control contract: `1`
Task: `M01B - Rust agent tooling foundation`
Owner: `OpenCode`
Status: `READY`
Task colour: `Amber`
Route: `OpenCode using DeepSeek Pro V4 for pinned Rust tooling, PowerShell verification, and OpenCode LSP configuration; Lucy performs independent review`
Base branch: `main`
Base commit: `05d0fbc6fe3cc8e05d3670cad4056f093c1c63d4`
Implementation branch: `opencode/m01b-rust-agent-tooling`
Worker note: `docs/worker-notes/2026-08-04-m01b-rust-agent-tooling.md`
Implementation blueprint: `docs/architecture/M01B_RUST_AGENT_TOOLING_FOUNDATION.md`
Rust toolchain: exact `1.97.1` from root pin; plain Cargo; `--locked` mandatory
OCaml switch path: `N/A`
Tool installation: exact M01B tools explicitly authorised

## Objective

Install, pin, configure, and prove the small Rust toolset selected to help the
active OpenCode workflow:

```text
rust-analyzer   toolchain-owned language intelligence
cargo-nextest   alternative agent test loop
cargo-deny      dependency policy and advisory gate
cargo-machete   advisory unused-dependency detector
```

M01B adds no production behaviour and performs no cleanup deletion. It creates
the diagnostic foundation that M01C will later use for warning repair and
repository pruning.

Read `docs/architecture/M01B_RUST_AGENT_TOOLING_FOUNDATION.md` completely before
editing. It freezes exact versions, installation authority, policy boundaries,
OpenCode integration, and verification.

## Relevant background and existing behaviour

M01A is accepted on `main` at
`d561b8400a1398c3d5bdde2cf670eebe661a5cc4`.

The repository now uses exact Rust 1.97.1, edition 2021, `rust-version = 1.97`,
plain Cargo commands, fail-fast Just recipes, and a byte-identical Cargo.lock.

The accepted M01B cargo-tool versions are:

```text
cargo-nextest 0.9.137
cargo-deny    0.19.7
cargo-machete 0.9.2
```

Rust-analyzer is the Rust 1.97.1 rustup component, not a separately downloaded
weekly binary.

OpenCode's repository configuration currently loads required instructions but
does not enable LSP. The active OpenCode documentation requires LSP to be enabled
in config, requires the `rust-analyzer` command for Rust, and gates direct LSP
queries behind a process-local experimental flag. The repository must prepare the
next OpenCode process honestly; a currently running process need not hot-reload.

Nextest may be slower on native Windows because it creates a process per test.
M01B measures rather than assumes performance. Ordinary `cargo test` remains the
final authority regardless of the benchmark.

Cargo-deny replaces the need for cargo-audit in this workflow. Cargo-semver-checks
remains deferred until a public Rust API compatibility promise exists.

## Startup procedure

1. Confirm the worktree is clean:

   ```powershell
   git status --short
   ```

   Stop if it is not clean.
2. Fetch remote state:

   ```powershell
   git fetch origin
   ```

3. Verify the M01B blueprint checkpoint is on remote main:

   ```powershell
   git merge-base --is-ancestor 05d0fbc6fe3cc8e05d3670cad4056f093c1c63d4 origin/main
   ```

   Require exit code 0.
4. Verify accepted M01A is on remote main:

   ```powershell
   git merge-base --is-ancestor d561b8400a1398c3d5bdde2cf670eebe661a5cc4 origin/main
   ```

   Require exit code 0.
5. Inspect the packet directly from remote main:

   ```powershell
   git show origin/main:docs/CURRENT_CLINE_TASK.md | Select-Object -First 18
   ```

   Require M01B, owner OpenCode, status READY, and branch
   `opencode/m01b-rust-agent-tooling`.
6. Verify the blueprint exists:

   ```powershell
   git cat-file -e origin/main:docs/architecture/M01B_RUST_AGENT_TOOLING_FOUNDATION.md
   ```

7. Confirm the implementation branch does not already exist:

   ```powershell
   git branch --list opencode/m01b-rust-agent-tooling
   git branch --remotes --list origin/opencode/m01b-rust-agent-tooling
   ```

   Stop without overwriting it if either command reports the branch.
8. Create it from current remote main:

   ```powershell
   git switch --create opencode/m01b-rust-agent-tooling origin/main
   ```

9. Read completely before editing:

   - `AGENTS.md`;
   - `docs/CURRENT_CLINE_TASK.md`;
   - `docs/architecture/M01B_RUST_AGENT_TOOLING_FOUNDATION.md`;
   - `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md`;
   - `docs/TOOLCHAIN_POLICY.md`;
   - `docs/worker-notes/2026-08-04-m01a-rust-toolchain-refresh.md`;
   - `rust-toolchain.toml`;
   - `justfile`;
   - `opencode.json`;
   - `scripts/check-dev-tools.ps1`;
   - `.github/scripts/check-tethers-toolchains.ps1`.

10. Run the packet checker.
11. Capture:

    ```powershell
    git rev-parse origin/main
    Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256
    rustc --version
    opencode --version
    cargo nextest --version
    cargo deny --version
    cargo machete --version
    rust-analyzer --version
    ```

    Missing commands are expected inventory results, not permission to choose
    different tools.
12. Update the packet Base commit to the exact current `origin/main` before the
    implementation commit and keep the worker note identical to that value.

## Installation authority

Matthew explicitly authorises only these machine changes:

- add rust-analyzer to exact Rust 1.97.1;
- install cargo-nextest exactly 0.9.137;
- install cargo-deny exactly 0.19.7;
- install cargo-machete exactly 0.9.2.

Installation must be performed through the repository installer created by this
task. Do not update the global Rust default, PATH, OpenCode, Cargo configuration,
other Cargo tools, or any unrelated package.

## Required behaviour

1. Add `tools/rust-agent-tools.json` with the exact schema and versions frozen in
   the blueprint. Executable scripts read this file rather than copying versions.

2. Add rust-analyzer to the existing Rust 1.97.1 component list in
   `rust-toolchain.toml` without changing the channel, profile, rustfmt, or Clippy.

3. Add `scripts/install-rust-agent-tools.ps1` with exact, idempotent installation
   behaviour and no global default, PATH, Cargo-config, or unrelated mutation.

4. Add read-only `scripts/check-rust-agent-tools.ps1` that validates config,
   exact versions, rust-analyzer ownership, OpenCode availability/config, and
   required repository policy files.

5. Expose the checker's callable function and add
   `scripts/test-check-rust-agent-tools.ps1` covering missing, malformed,
   wrong-schema, impossible-version, real-success, and non-mutation paths.

6. Preserve `opencode.json` instructions and add only LSP enablement plus LSP-tool
   permission.

7. Add `scripts/start-opencode-lsp.ps1` that process-locally enables the
   experimental LSP tool, disables OpenCode LSP downloads, forwards arguments,
   returns the child exit code, and restores both environment values.

8. Prove `opencode debug config` sees the repository LSP configuration. Stop
   rather than claiming integration if the installed OpenCode build cannot
   provide the frozen current behaviour.

9. Add minimal `.config/nextest.toml` requiring 0.9.137 with retries zero and
   fail-fast true. Add no flaky-test exceptions or retry override.

10. Add a concise reviewed root `deny.toml` implementing the frozen licence,
    advisory, source, wildcard, and duplicate-version policy with no advisory
    ignores.

11. Extend `justfile` with fail-fast `agent-tools`, `test-agent`, `deps-policy`,
    `deps-advisories`, `deps-unused`, and `verify-agent` recipes exactly as
    described by the blueprint.

12. Keep `deps-unused` advisory and outside `verify-agent`. Never run
    cargo-machete with `--fix` and remove no dependency.

13. Update only live agent/tool guidance in `AGENTS.md`, the Rust engineering
    guide, and toolchain policy with the frozen tool roles and authority limits.

14. Benchmark three warm complete runs each of ordinary Cargo and nextest on
    native Windows. Record all durations and medians without changing machine
    security or scheduling settings.

15. Run cargo-deny licences, bans, sources, and advisories. Stop on an advisory or
    licence outside the frozen list rather than inventing an exception.

16. Preserve Cargo.lock byte-for-byte and all production source, tests, OCaml,
    edition, compiler channel, dependency graph, and Tethers behaviour.

17. Record every cargo-machete finding for M01C with no automatic conclusion that
    it is truly unused.

18. Follow the exact-edit recovery rule: reread after one `oldString` failure,
    make a fresh smaller patch, never retry the identical edit, and stop after two
    materially different failures.

## Relevant components

- `rust-toolchain.toml`
- `tools/rust-agent-tools.json`
- `scripts/install-rust-agent-tools.ps1`
- `scripts/check-rust-agent-tools.ps1`
- `scripts/test-check-rust-agent-tools.ps1`
- `scripts/start-opencode-lsp.ps1`
- `opencode.json`
- `.config/nextest.toml`
- `deny.toml`
- `justfile`
- `AGENTS.md`
- `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md`
- `docs/TOOLCHAIN_POLICY.md`
- `docs/architecture/M01B_RUST_AGENT_TOOLING_FOUNDATION.md`
- `docs/CURRENT_CLINE_TASK.md`

Read-only references include Cargo.toml, Cargo.lock, production source/tests,
OCaml files, M01A evidence, and inactive files reserved for M01C review.

## Frozen decisions and invariants

- Exact tool versions are frozen; no floating latest install is permitted.
- Rust-analyzer belongs to exact Rust 1.97.1.
- LSP assists navigation; compiler, Clippy, tests, and contracts remain authority.
- Direct OpenCode LSP queries are opt-in and process-local.
- OpenCode may not download a second rust-analyzer when using the wrapper.
- Nextest never retries and never replaces final ordinary Cargo testing.
- Native Windows nextest performance is measured, not assumed.
- Cargo-deny is the one accepted dependency/advisory gate.
- Cargo-audit and cargo-semver-checks remain absent.
- Cargo-machete is advisory only and never removes dependencies automatically.
- No advisory ignore or new licence is invented by the worker.
- Machine installation is exact, bounded, idempotent, and separate from verify.
- Cargo.lock, dependencies, edition, compiler channel, production behaviour, and
  OCaml remain unchanged.
- M01C performs actual warning cleanup and deletion review.

## Acceptance criteria

1. The root JSON contains only the frozen schema and exact tool versions.
2. Rust 1.97.1 owns installed rust-analyzer while its other components remain.
3. The installer installs only missing/mismatched accepted versions and a second
   run performs no installation.
4. The checker reports every exact version and changes no repository state.
5. All focused checker negative and success tests pass.
6. OpenCode config retains instructions, enables LSP, and permits the LSP tool.
7. The opt-in launcher sets and restores both environment variables on success
   and a forced child-command failure.
8. `opencode debug config` confirms the effective repository LSP configuration.
9. Nextest accepts its repository config, runs with zero retries, and completes
   the supported Rust test graph.
10. Ordinary Cargo still completes the authoritative all-target/all-feature test
    graph.
11. Three-run timing evidence reports honest Cargo and nextest medians.
12. Cargo-deny licence, bans, and sources checks pass with only the frozen policy.
13. Cargo-deny advisories pass with no ignores.
14. Cargo-machete runs with metadata and no `--fix`; all findings are recorded.
15. New Just recipes are fail-fast and `verify-agent` retains ordinary Cargo
    verification.
16. Live agent/tool guidance accurately describes tool roles and limitations.
17. Cargo.lock before/after hashes are identical.
18. No dependency, source, test, OCaml, edition, Rust channel, Plug lifecycle, or
    Tethers behaviour changes.
19. Packet checker, Rustfmt, ordinary `just verify`, `verify-agent`, and
    `git diff --check` pass.
20. The worker note records installed versions, idempotency, LSP evidence,
    benchmarks, deny results, machete findings, lock hash, exact changed files,
    and a verified real implementation checkpoint.

## Required verification

Run every applicable command in this order:

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
pwsh -NoProfile -File scripts/install-rust-agent-tools.ps1
pwsh -NoProfile -File scripts/install-rust-agent-tools.ps1
pwsh -NoProfile -File scripts/test-check-rust-agent-tools.ps1
pwsh -NoProfile -File scripts/check-rust-agent-tools.ps1
rustc --version
rust-analyzer --version
cargo nextest --version
cargo deny --version
cargo machete --version
opencode --version
opencode debug config
cargo nextest show-config version
cargo nextest run --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features
cargo deny --manifest-path tethers-0.1/host-rust/Cargo.toml check licenses bans sources
cargo deny --manifest-path tethers-0.1/host-rust/Cargo.toml check advisories
cargo machete --with-metadata tethers-0.1/host-rust
just --list
just agent-tools
just test-agent
just deps-policy
just deps-advisories
just deps-unused
just verify
just verify-agent
cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check
cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked
Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256
git diff --check
git status --short
```

`cargo machete` and `just deps-unused` may return its documented finding exit
code. That does not authorise ignoring the output or changing a dependency. The
worker note must record the exact result and M01C candidates. All other required
commands must succeed.

Test the OpenCode launcher with a fast child invocation and environment sentinels
as defined in the blueprint. Record whether the currently running OpenCode
process exposed direct LSP queries; completion is based on the next process and
effective config, not hot reload.

For the benchmark, warm once and then use `Measure-Command` for three complete
runs of each test runner. Do not clear caches between runs.

## Permitted changes

Only these paths may change:

- `rust-toolchain.toml`
- `tools/rust-agent-tools.json`
- `scripts/install-rust-agent-tools.ps1`
- `scripts/check-rust-agent-tools.ps1`
- `scripts/test-check-rust-agent-tools.ps1`
- `scripts/start-opencode-lsp.ps1`
- `opencode.json`
- `.config/nextest.toml`
- `deny.toml`
- `justfile`
- `AGENTS.md`
- `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md`
- `docs/TOOLCHAIN_POLICY.md`
- `docs/worker-notes/2026-08-04-m01b-rust-agent-tooling.md`
- `docs/CURRENT_CLINE_TASK.md` only for control state and checkpoint

Stop before changing another path.

## Forbidden changes

- No Cargo.toml, Cargo.lock, dependency, feature, package, edition, or MSRV change.
- No production source, test, fixture, generated lifecycle evidence, or OCaml
  change.
- No Rust channel change or unrelated rustup component.
- No OpenCode update, global environment edit, PATH edit, or Cargo config edit.
- No cargo-audit, cargo-semver-checks, cargo-binstall, or other tool installation.
- No nextest retries or replacement of ordinary Cargo completion tests.
- No cargo-deny advisory ignore or autonomous licence expansion.
- No cargo-machete `--fix`, dependency removal, source deletion, warning cleanup,
  inactive-agent-file deletion, roadmap pruning, or M01C work.
- No Plug installation, J24J, CLI, runtime, provider, Anchor, Trail, or release
  work.
- No amend, reset, rebase, cherry-pick, force-push, merge to main, tag, or
  publication.

## Stop conditions

Stop and return exact evidence when:

- the implementation branch already exists;
- current origin/main lacks accepted M01A or the M01B blueprint;
- exact accepted tool installation would require another package or global
  configuration change;
- OpenCode effective config cannot enable the frozen LSP behaviour;
- cargo-deny reports an advisory or licence outside the frozen policy;
- nextest requires retries or a production-test change;
- Cargo.lock, Cargo.toml, source, tests, OCaml, Rust channel, or a forbidden path
  appears necessary;
- a cargo-machete finding cannot be recorded without acting on it;
- two materially different edit attempts fail after rereading current content.

## Expected pre-existing changes

None.

## Git and return contract

Use ordinary commits and normal push only.

After all required evidence passes:

1. Create the worker note at the exact packet path.
2. Set packet status to `COMPLETE` and implementation checkpoint to `TBD`.
3. Make one normal implementation commit.
4. Obtain and verify its real full SHA:

   ```powershell
   git cat-file -e <REAL_SHA>^{commit}
   ```

5. Record that exact SHA in packet and worker note.
6. Make a separate completion-documentation commit.
7. Push normally.

Return branch, remote tip, verified implementation checkpoint, exact changed
files, exact installed versions, second-install no-op evidence, checker results,
OpenCode/LSP evidence, Cargo/nextest benchmark, deny results, machete findings,
Cargo.lock hashes, full ordinary and nextest test results, Just results, worker
note path, and explicit confirmation that no dependency or product behaviour
changed.
