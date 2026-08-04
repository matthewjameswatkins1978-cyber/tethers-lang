# Current Implementation Task

Control contract: `1`
Task: `M01B - Rust agent tooling foundation`
Owner: `OpenCode`
Status: `READY`
Task colour: `Amber`
Route: `OpenCode using DeepSeek Pro V4 for pinned Rust tooling, PowerShell verification, and OpenCode LSP configuration; Lucy performs independent review`
Base branch: `main`
Base commit: `57e709f7c3fd0a85fdf52d5f027bbd4bdf9af5bf`
Implementation branch: `opencode/m01b-rust-agent-tooling`
Worker note: `docs/worker-notes/2026-08-04-m01b-rust-agent-tooling.md`
Implementation blueprint: `docs/architecture/M01B_RUST_AGENT_TOOLING_FOUNDATION.md`
Rust toolchain: exact `1.97.1` from root pin; plain Cargo; `--locked` mandatory
OCaml switch path: `N/A`
Tool installation: exact M01B tools explicitly authorised
Status: `COMPLETE`
Implementation checkpoint: `TBD`

## Objective

Install, pin, configure, and prove the small Rust toolset selected to help the
active OpenCode workflow:

```text
rust-analyzer   Rust 1.97.1 toolchain component
cargo-nextest   0.9.140 alternative agent test loop
cargo-deny      0.19.7 dependency policy and advisory gate
cargo-machete   0.9.2 advisory unused-dependency detector
```

M01B adds no production behaviour and performs no cleanup deletion. It creates
the diagnostic foundation that M01C will later use for warning repair and
repository pruning.

Read `docs/architecture/M01B_RUST_AGENT_TOOLING_FOUNDATION.md` completely before
editing. It is authoritative for exact versions, installation authority, policy
boundaries, OpenCode integration, and verification.

## Relevant background and existing behaviour

M01A is accepted on `main` at
`d561b8400a1398c3d5bdde2cf670eebe661a5cc4`.

The repository uses exact Rust 1.97.1, edition 2021, `rust-version = 1.97`, plain
Cargo commands, fail-fast Just recipes, and a committed Cargo.lock that M01B must
preserve byte-for-byte.

Rust-analyzer must belong to the Rust 1.97.1 rustup toolchain rather than arriving
as an unrelated weekly binary. Nextest may be slower on native Windows, so this
task measures it rather than assuming it is faster. Ordinary `cargo test` remains
the final completion authority.

Cargo-deny replaces cargo-audit in this workflow. Cargo-semver-checks remains
deferred until Tethers explicitly promises compatibility for a public Rust
library API. Cargo-machete is advisory only and may not modify dependencies.

OpenCode currently loads repository instructions but has no explicit LSP setting.
M01B must enable and verify the repository configuration honestly. A currently
running OpenCode process need not hot-reload; the next process launched through
the repository wrapper is the acceptance target.

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

4. Verify accepted M01A is on remote main:

   ```powershell
   git merge-base --is-ancestor d561b8400a1398c3d5bdde2cf670eebe661a5cc4 origin/main
   ```

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

7. Confirm the implementation branch does not exist locally or remotely:

   ```powershell
   git branch --list opencode/m01b-rust-agent-tooling
   git branch --remotes --list origin/opencode/m01b-rust-agent-tooling
   ```

   Stop without overwriting it if either command reports the branch.
8. Create the branch from current remote main:

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
11. Capture current `origin/main`, Cargo.lock SHA-256, Rust/OpenCode versions,
    and whether each selected tool is already installed.
12. Update this packet's Base commit to the exact current `origin/main` before
    the implementation commit, and use the identical value in the worker note.

## Installation authority

Matthew explicitly authorises only these machine changes:

- add rust-analyzer to exact Rust 1.97.1;
- install cargo-nextest exactly 0.9.140;
- install cargo-deny exactly 0.19.7;
- install cargo-machete exactly 0.9.2.

Installation must use the repository installer created by this task. Do not
update the global Rust default, PATH, OpenCode, Cargo configuration, another
Cargo tool, or any unrelated package.

## Required behaviour

1. Add `tools/rust-agent-tools.json` with schema 1 and exactly the four frozen
   tool/version declarations. Installation and checking scripts must read it.

2. Add rust-analyzer to the existing Rust 1.97.1 component list in
   `rust-toolchain.toml` without changing the channel, profile, rustfmt, or
   Clippy.

3. Add `scripts/install-rust-agent-tools.ps1`. It must validate repository and
   config, install only missing or mismatched exact tools, use `--locked`, use
   `--force` only for a genuine version mismatch, avoid global configuration,
   and be idempotent.

4. Add read-only `scripts/check-rust-agent-tools.ps1` plus focused
   `scripts/test-check-rust-agent-tools.ps1`. The checker must expose an
   in-process function and prove exact versions, rust-analyzer ownership,
   OpenCode/config readiness, required policy files, and repository non-mutation.

5. Preserve the existing `opencode.json` instruction list and add only LSP
   enablement plus permission for the LSP tool.

6. Add `scripts/start-opencode-lsp.ps1`. It must process-locally set
   `OPENCODE_EXPERIMENTAL_LSP_TOOL=true` and
   `OPENCODE_DISABLE_LSP_DOWNLOAD=true`, forward arbitrary OpenCode arguments,
   return the child exit code, and restore both previous environment values on
   success and failure.

7. Prove `opencode debug config` sees the repository LSP configuration. If the
   installed OpenCode build cannot provide the frozen behaviour, stop and report
   the exact version and limitation rather than claiming success.

8. Add `.config/nextest.toml` requiring nextest 0.9.140 with retries zero and
   fail-fast true. Add no flaky-test exception or retry override.

9. Add a concise reviewed root `deny.toml` implementing the blueprint's frozen
   licence, advisory, source, wildcard, and duplicate-version policy. Do not add
   advisory ignores or autonomous licence exceptions.

10. Extend `justfile` with fail-fast `agent-tools`, `test-agent`, `deps-policy`,
    `deps-advisories`, `deps-unused`, and `verify-agent`. Keep ordinary Cargo
    inside normal `verify`; keep advisory `deps-unused` outside `verify-agent`.

11. Update only live guidance in `AGENTS.md`,
    `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md`, and
    `docs/TOOLCHAIN_POLICY.md` with the frozen tool roles and authority limits.

12. Benchmark three warm complete runs each of ordinary Cargo and nextest on
    native Windows. Record all durations and medians without changing machine
    security, scheduling, PATH, or cache state.

13. Run cargo-deny licences, bans, sources, and advisories. Stop on an advisory
    or licence outside the frozen policy rather than inventing an exception.

14. Run cargo-machete with metadata and never `--fix`. Record every finding for
    M01C without concluding automatically that it is truly unused.

15. Preserve Cargo.toml, Cargo.lock, dependencies, source, tests, OCaml, edition,
    Rust channel, Plug lifecycle, and Tethers behaviour.

16. Follow the exact-edit recovery rule: after one `oldString` failure, reread
    the current file, make a fresh smaller patch, never retry the identical edit,
    and stop after two materially different failures.

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

Cargo.toml, Cargo.lock, production source/tests, OCaml files, M01A evidence, and
inactive files reserved for M01C are read-only.

## Frozen decisions and invariants

- Exact tool versions are frozen; no floating latest installation is permitted.
- Rust-analyzer belongs to exact Rust 1.97.1.
- LSP assists navigation; compiler, Clippy, tests, and contracts remain authority.
- Direct OpenCode LSP queries are opt-in and process-local.
- OpenCode may not download a second rust-analyzer through the wrapper.
- Nextest retries remain zero and it never replaces final ordinary Cargo tests.
- Native Windows performance is measured, not assumed.
- Cargo-deny is the single accepted dependency/advisory gate.
- Cargo-audit and cargo-semver-checks remain absent.
- Cargo-machete is advisory and never removes dependencies automatically.
- No advisory ignore or new licence is invented by the worker.
- Installation is exact, bounded, idempotent, and separate from verification.
- Cargo.lock and all product behaviour remain unchanged.
- M01C performs actual warning cleanup and deletion review.

## Acceptance criteria

1. Repository JSON contains exactly the frozen schema and tool declarations.
2. Rust 1.97.1 owns installed rust-analyzer while existing components remain.
3. The installer installs only missing or mismatched accepted versions.
4. A second installer run performs no installation and succeeds.
5. The read-only checker reports exact versions and changes no repository state.
6. All checker negative and real-success tests pass.
7. OpenCode config retains instructions, enables LSP, and permits the LSP tool.
8. The launcher sets and restores both environment variables on success and a
   forced child failure.
9. `opencode debug config` confirms effective repository LSP configuration.
10. Nextest accepts its config, uses zero retries, and completes the test graph.
11. Ordinary Cargo completes the authoritative all-target/all-feature graph.
12. Three-run timing evidence reports honest Cargo and nextest medians.
13. Cargo-deny licence, bans, sources, and advisories pass with no ignores.
14. Cargo-machete runs with metadata and no fix; findings are recorded only.
15. New Just recipes are fail-fast and `verify-agent` retains ordinary Cargo
    verification.
16. Live agent/tool guidance accurately describes roles and limits.
17. Cargo.lock before and after hashes are identical.
18. No dependency, source, test, OCaml, edition, Rust channel, lifecycle, or
    Tethers behaviour changes.
19. Packet checker, Rustfmt, ordinary `just verify`, `verify-agent`, and
    `git diff --check` pass.
20. The worker note records versions, idempotency, LSP evidence, benchmarks, deny
    results, machete findings, lock hash, changed files, and a verified real
    implementation checkpoint.

## Required verification

Run every applicable command:

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

`cargo machete` and `just deps-unused` may return their documented finding exit
code. This does not authorise ignoring output or changing a dependency. Record
the exact result and M01C candidates. Every other required command must succeed.

Test the OpenCode launcher with a fast child invocation and environment
sentinels. For the benchmark, warm once and then use `Measure-Command` for three
complete runs of each test runner without clearing caches.

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

- No Cargo.toml, Cargo.lock, dependency, feature, package, edition, or MSRV
  change.
- No production source, test, fixture, generated lifecycle evidence, or OCaml
  change.
- No Rust channel change or unrelated rustup component.
- No OpenCode update, global environment edit, PATH edit, or Cargo config edit.
- No cargo-audit, cargo-semver-checks, cargo-binstall, or other tool installation.
- No nextest retries or replacement of ordinary Cargo completion tests.
- No cargo-deny advisory ignore or autonomous licence expansion.
- No cargo-machete `--fix`, dependency removal, source deletion, warning cleanup,
  inactive-agent-file deletion, roadmap pruning, or M01C work.
- No Plug installation, J24J, CLI, runtime, provider, Anchor, Trail, release, tag,
  or publication work.
- No amend, reset, rebase, cherry-pick, force-push, or merge into main.

## Stop conditions

Stop and return exact evidence when:

- the implementation branch already exists;
- current origin/main lacks accepted M01A or the M01B blueprint;
- exact installation requires another package or global configuration change;
- OpenCode effective config cannot provide the frozen LSP behaviour;
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

Return branch, remote tip, verified implementation checkpoint, changed files,
installed versions, second-install no-op evidence, checker results, OpenCode/LSP
evidence, Cargo/nextest benchmark, deny results, machete findings, Cargo.lock
hashes, ordinary and nextest test results, Just results, worker-note path, and
explicit confirmation that no dependency or product behaviour changed.
