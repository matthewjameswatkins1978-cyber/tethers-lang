# Current Implementation Task

Control contract: `1`
Task: `M01A - Rust toolchain refresh and verification cleanup`
Owner: `OpenCode`
Status: `READY`
Task colour: `Amber`
Route: `OpenCode using DeepSeek Pro V4 for a cross-file toolchain and PowerShell maintenance change; Lucy performs independent review`
Base branch: `main`
Base commit: `c9b24b3987b92092aa7800f28e1147e719c70b57`
Implementation branch: `opencode/m01a-rust-toolchain-refresh`
Worker note: `docs/worker-notes/2026-08-04-m01a-rust-toolchain-refresh.md`
Implementation blueprint: `docs/architecture/M01A_RUST_TOOLCHAIN_REFRESH.md`
OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`
Rust toolchain: current `1.89.0`; target exact `1.97.1`; plain Cargo after root pin; `--locked` mandatory
Toolchain preflight: required

## Objective

Upgrade the repository-owned Rust toolchain to exact stable Rust 1.97.1 and
clean the live verification plumbing so future commands derive compiler truth
from repository pins instead of repeating `1.89.0` throughout scripts and
instructions.

M01A is deliberately separate from dependency updates, edition migration,
production refactoring, warning removal, file deletion, and the Plug installation
sequence.

Read `docs/architecture/M01A_RUST_TOOLCHAIN_REFRESH.md` completely before any
edit. It is authoritative for version semantics, allowed machine installation,
required files, and verification.

## Relevant background and existing behaviour

J24I is accepted on `main` at
`88d8ab2e5c65052401b3860d8a7d68f3ccb06265`. The installation sequence pauses
before J24J while this maintenance milestone refreshes the build floor.

Current live Rust authority is fragmented:

- root `rust-toolchain.toml` selects 1.89.0;
- host `Cargo.toml` declares `rust-version = "1.89"`;
- the `justfile`, environment probe, toolchain checker, task template, and Rust
  guide repeat 1.89.0 directly.

The current `justfile` also joins some commands with PowerShell semicolons, so a
later successful command can obscure the status of an earlier failed command.

Rust 1.97.1 is the exact approved target. It supersedes 1.97.0 because the point
release fixes an LLVM miscompilation.

OCaml 5.5.0, Dune 3.24.0, and Yojson 2.2.2 remain the accepted locked OCaml
baseline. M01A must verify but not update them.

The existing Cargo lock, dependencies, Tethers source, tests, and product
semantics are not part of this task.

## Startup procedure

1. Confirm the current worktree is clean:

   ```powershell
   git status --short
   ```

   Stop if it is not clean.
2. Fetch remote state:

   ```powershell
   git fetch origin
   ```

3. Verify the M01A blueprint checkpoint is an ancestor of current remote main:

   ```powershell
   git merge-base --is-ancestor c9b24b3987b92092aa7800f28e1147e719c70b57 origin/main
   ```

   Require exit code 0.
4. Verify accepted corrected J24I is an ancestor of remote main:

   ```powershell
   git merge-base --is-ancestor 88d8ab2e5c65052401b3860d8a7d68f3ccb06265 origin/main
   ```

   Require exit code 0.
5. Inspect the packet directly from `origin/main` and require M01A, OpenCode,
   `READY`, and branch `opencode/m01a-rust-toolchain-refresh`:

   ```powershell
   git show origin/main:docs/CURRENT_CLINE_TASK.md | Select-Object -First 18
   ```

6. Verify the blueprint exists on remote main:

   ```powershell
   git cat-file -e origin/main:docs/architecture/M01A_RUST_TOOLCHAIN_REFRESH.md
   ```

7. Confirm the implementation branch does not exist locally or remotely:

   ```powershell
   git branch --list opencode/m01a-rust-toolchain-refresh
   git branch --remotes --list origin/opencode/m01a-rust-toolchain-refresh
   ```

   Stop without overwriting it if either reports a branch.
8. Create the implementation branch from current remote main:

   ```powershell
   git switch --create opencode/m01a-rust-toolchain-refresh origin/main
   ```

9. Read completely:

   - `AGENTS.md`;
   - `docs/CURRENT_CLINE_TASK.md`;
   - `docs/architecture/M01A_RUST_TOOLCHAIN_REFRESH.md`;
   - `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md`;
   - `docs/worker-notes/2026-07-30-toolchain-baseline-01.md`;
   - `rust-toolchain.toml`;
   - `tethers-0.1/host-rust/Cargo.toml`;
   - `justfile`;
   - `.github/scripts/check-tethers-toolchains.ps1`;
   - `.github/scripts/test-check-tethers-toolchains.ps1`;
   - `scripts/check-tethers-environment.ps1`.

10. Run the packet checker before editing.
11. Capture the current Cargo.lock SHA-256 and the current Rust 1.89.0 full-suite
    and warning baseline if 1.89.0 remains installed.
12. Check whether exact Rust 1.97.1, rustfmt, and Clippy are installed. Matthew
    explicitly authorises only this exact installation if missing:

    ```powershell
    rustup toolchain install 1.97.1 --profile minimal --component rustfmt --component clippy
    ```

    Do not update the global default, uninstall 1.89.0, modify PATH, or install
    anything else.

## Required behaviour

1. Pin root `rust-toolchain.toml` to exact Rust `1.97.1`, minimal profile, with
   rustfmt and Clippy.

2. Change only `rust-version` in host `Cargo.toml` from `1.89` to `1.97` while
   preserving edition 2021, package version, dependencies, features, and
   `Cargo.lock` byte-for-byte.

3. Replace explicit `+1.89.0` Cargo selectors and `Push-Location` semicolon
   chains in `justfile` with plain Cargo plus `--manifest-path`, keeping `--locked`
   and making every multi-step recipe fail immediately on its first failure.

4. Remove explicit `+1.89.0` arguments from
   `scripts/check-tethers-environment.ps1`; rely on the root pin and preserve its
   read-only, offline, machine-readable capability-report contract.

5. Refactor `.github/scripts/check-tethers-toolchains.ps1` to derive the exact
   Rust channel from `rust-toolchain.toml` and Rust edition/MSRV from
   `Cargo.toml`, while retaining explicit OCaml-switch verification and the
   process-local rustup auto-install guard.

6. Make the checker require exact rustc 1.97.1, Cargo major/minor 1.97, installed
   rustfmt and Clippy components, and successful rustfmt/Clippy version commands
   without inventing non-contractual point-version equality.

7. Update `.github/scripts/test-check-tethers-toolchains.ps1` without weakening
   its existing negative paths, guard restoration, real-switch, no-fallback, and
   repository-non-mutation evidence.

8. Update `docs/TASK_PACKET_TEMPLATE.md` so future packets read the root exact
   pin, use plain Cargo, and require `--locked` rather than copying a compiler
   version into the template.

9. Update live Rust guidance in `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md` to
   Rust 1.97.1, edition 2021, and `rust-version = 1.97`; direct agents to the
   repository pins rather than floating latest documentation or remembered
   versions.

10. Add a concise `docs/TOOLCHAIN_POLICY.md` covering exact pins, monthly or
    milestone review, prompt security/soundness point releases, separation of
    compiler/dependency/edition jobs, locked builds, OCaml lock authority, and
    preservation of historical evidence.

11. Preserve all historical worker notes, completed packets, release notes, and
    architecture evidence exactly as historical records even when they mention
    Rust 1.89.0.

12. Record evidence-backed candidates for M01B in the worker note only. Do not
    delete or rename files during M01A.

13. Preserve all Tethers language, Plug lifecycle, provider, trust, execution,
    Anchor, Trail, package, and runtime behaviour.

14. Follow the DeepSeek exact-edit recovery rule: after one `oldString` failure,
    reread the current file and create a smaller fresh patch; never retry the
    identical edit and stop after two materially different failed attempts.

## Relevant components

- `rust-toolchain.toml`
- `tethers-0.1/host-rust/Cargo.toml`
- `tethers-0.1/host-rust/Cargo.lock`
- `justfile`
- `scripts/check-tethers-environment.ps1`
- `.github/scripts/check-tethers-toolchains.ps1`
- `.github/scripts/test-check-tethers-toolchains.ps1`
- `docs/TASK_PACKET_TEMPLATE.md`
- `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md`
- `docs/TOOLCHAIN_POLICY.md`
- `docs/architecture/M01A_RUST_TOOLCHAIN_REFRESH.md`
- `docs/worker-notes/2026-07-30-toolchain-baseline-01.md`
- `docs/CURRENT_CLINE_TASK.md`

Read-only references include:

- `AGENTS.md`;
- `docs/OCAML_GUIDE_FOR_AGENTS.md`;
- `tethers-0.1/engine-ocaml/tethers_engine.opam`;
- `tethers-0.1/engine-ocaml/tethers_engine.opam.locked`;
- `tethers-0.1/engine-ocaml/dune-project`.

## Frozen decisions and invariants

- Exact active Rust channel is `1.97.1`, not floating `stable`.
- Rust edition remains 2021.
- Declared Rust minimum becomes `1.97`; no separate older MSRV lane is claimed.
- Toolchain, dependency, warning-cleanup, file-pruning, and edition migration are
  separate jobs.
- `Cargo.lock` remains byte-identical and ordinary Cargo verification is locked.
- Plain Cargo commands inherit the exact root toolchain pin.
- Cargo's point version is not required to equal rustc's point-release number.
- Existing explicit OCaml switch and lock authority remain unchanged.
- Toolchain checks are read-only and restore `RUSTUP_AUTO_INSTALL` exactly.
- Historical evidence is not rewritten to look current.
- No production source or test changes are authorised.
- No inactive configuration or document is deleted in M01A.
- M01B handles warning cleanup and safe repository pruning after this compiler
  refresh is accepted.
- No Plug-installation work proceeds until M01A is accepted.

## Acceptance criteria

1. `rust-toolchain.toml` selects exact 1.97.1 with minimal profile, rustfmt, and
   Clippy.
2. Host `Cargo.toml` remains edition 2021 and declares `rust-version = "1.97"`.
3. Cargo dependencies, features, package version, and `Cargo.lock` are unchanged.
4. No explicit `+1.89.0` remains in active `justfile` or environment-probe
   commands.
5. Every Just verification recipe stops at its first failing command and uses
   the host manifest explicitly.
6. The environment probe resolves the root-pinned compiler and remains
   read-only/offline where previously required.
7. The toolchain checker derives the exact Rust channel and Cargo metadata from
   repository files rather than copied constants.
8. Exact rustc 1.97.1, Cargo 1.97 major/minor, rustfmt, and Clippy are verified
   without false point-version assumptions.
9. All prior toolchain-checker negative paths and environment-restoration tests
   remain green.
10. The real explicit OCaml switch still passes with OCaml 5.5.0, Dune 3.24.0,
    and Yojson 2.2.2.
11. The task template no longer fossilises a specific Rust version.
12. The Rust guide and new toolchain policy describe the accepted live baseline
    and maintenance cadence accurately.
13. Historical evidence mentioning Rust 1.89.0 remains unchanged.
14. Rust formatting, check, Clippy, and complete all-target/all-feature tests run
    under exact Rust 1.97.1 with `--locked` where supported.
15. Before/after evidence identifies any new warnings or failures introduced by
    1.97.1; none are silently dismissed.
16. `just --list`, `just fmt`, `just check`, `just test-rust`, and `just verify`
    all succeed.
17. Packet checker, toolchain checker tests, real preflight, and
    `git diff --check` pass.
18. No production Rust/OCaml source, production test, dependency, lock, runtime,
    lifecycle, or language behaviour changes.
19. The worker note records exact tool versions, Cargo.lock hash equality,
    changed files, warning comparison, M01B candidates, and a verified real
    implementation checkpoint.

## Required verification

Run in this order after changes:

```powershell
rustup run 1.97.1 rustc --version
rustup component list --toolchain 1.97.1 --installed
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check
cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked
cargo clippy --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked
cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked
just --list
just fmt
just check
just test-rust
pwsh -NoProfile -File .github/scripts/test-check-tethers-toolchains.ps1 -OcamlSwitchPath 'D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml'
pwsh -NoProfile -File .github/scripts/check-tethers-toolchains.ps1 -OcamlSwitchPath 'D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml'
just verify
git diff --check
git status --short
```

Also prove:

```powershell
Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256
rg -n --fixed-strings '+1.89.0' justfile scripts/check-tethers-environment.ps1 .github/scripts/check-tethers-toolchains.ps1 docs/TASK_PACKET_TEMPLATE.md docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md
```

The final `rg` must return no active copied command or guidance pin. A historical
reference inside the blueprint explaining the migration is allowed; historical
worker notes and release evidence are outside this active-file search.

## Permitted changes

Only these files may change:

- `rust-toolchain.toml`
- `tethers-0.1/host-rust/Cargo.toml`
- `justfile`
- `scripts/check-tethers-environment.ps1`
- `.github/scripts/check-tethers-toolchains.ps1`
- `.github/scripts/test-check-tethers-toolchains.ps1`
- `docs/TASK_PACKET_TEMPLATE.md`
- `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md`
- `docs/TOOLCHAIN_POLICY.md` (new)
- `docs/worker-notes/2026-08-04-m01a-rust-toolchain-refresh.md` (new)
- `docs/CURRENT_CLINE_TASK.md` only for status transitions and final checkpoint

Stop before changing any other file.

## Forbidden changes

Do not:

- modify `Cargo.lock`, dependencies, features, package version, production source,
  production tests, fixtures, or generated evidence;
- change OCaml, opam, Dune, Yojson, OCaml locks, or Dune language;
- migrate to Rust edition 2024;
- suppress, allow, or repair existing warnings in production code;
- delete or rename `.clinerules`, `.clineignore`, scripts, guides, roadmaps,
  worker notes, release notes, control files, or any historical evidence;
- modify `AGENTS.md`, `docs/OCAML_GUIDE_FOR_AGENTS.md`, CI, release, version tag,
  Tethers Core, Plug installation, trust, provider, runtime, Anchor, Trail, or
  application code;
- update the global Rust default, uninstall 1.89.0, modify PATH, install another
  tool, or change global/user Cargo configuration;
- amend, reset, rebase, cherry-pick, force-push, merge into main, tag, or publish.

## Stop conditions

Stop and return exact evidence plus the smallest unresolved question if:

- the implementation branch already exists;
- the blueprint or corrected J24I is not on current remote main;
- the worktree is dirty;
- the explicit OCaml switch path is absent, resolves elsewhere, or fails the
  accepted no-fallback contract;
- Rust 1.97.1 cannot be installed using only the authorised rustup command;
- the upgrade requires a dependency, Cargo.lock, production code, test, OCaml,
  edition, or forbidden-file change;
- Cargo 1.97 cannot consume the existing lock without mutation;
- a new compiler failure cannot be repaired strictly within the permitted
  tooling/document files;
- checker compatibility requires a wider environment-system redesign;
- an exact-edit replacement fails twice after rereading and using materially
  different anchors;
- branch-specific verification fails after two materially different attempts.

## Expected pre-existing changes

None.

## Git and completion contract

Use normal commits and normal push only.

After every check passes:

1. Create `docs/worker-notes/2026-08-04-m01a-rust-toolchain-refresh.md`.
2. Set this packet to `COMPLETE`.
3. Create the implementation commit normally.
4. Obtain its real full SHA from Git.
5. Verify it exists:

   ```powershell
   git cat-file -e <REAL_SHA>^{commit}
   ```

6. Record that exact SHA in the packet and worker note.
7. Create completion-documentation commit separately.
8. Push normally.

Return branch, remote tip, verified implementation checkpoint, exact changed
files, exact tool versions, Cargo.lock before/after hashes, toolchain-checker
assertion count, Rust full-suite result, Just recipe results, warning comparison,
M01B cleanup candidates, worker-note path, and explicit confirmation that M01A
changed no dependency, production source, OCaml lock, edition, or Tethers
behaviour.
