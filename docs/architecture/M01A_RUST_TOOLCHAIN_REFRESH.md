# M01A Rust Toolchain Refresh

Status: frozen implementation blueprint

## Purpose

Refresh the repository-owned Rust build toolchain from 1.89.0 to the latest
verified stable point release, Rust 1.97.1, without combining the compiler move
with dependency updates, edition migration, production refactoring, or Plug
installation work.

This is the first half of the repository spring clean. M01A updates live
build authority and removes stale version duplication. M01B will separately
review obsolete files, inactive agent configuration, duplicated guidance, and
safe deletions.

## Verified release decision

The accepted target is exactly:

```text
Rust 1.97.1
```

Rust 1.97.1 is preferred over 1.97.0 because the point release fixes an LLVM
miscompilation. The repository must pin the exact point release rather than the
floating `stable` channel.

The Rust edition remains 2021. Edition migration is not part of M01A.

The package `rust-version` becomes `1.97`. Tethers is not currently maintaining
a separately tested older MSRV lane, so the declared minimum and active compiler
must not pretend to be independent support promises.

## Current live duplication

Rust 1.89.0 is currently repeated in:

- `rust-toolchain.toml`;
- `tethers-0.1/host-rust/Cargo.toml`;
- `justfile` command arguments;
- `scripts/check-tethers-environment.ps1` command arguments;
- `.github/scripts/check-tethers-toolchains.ps1` checks and messages;
- `docs/TASK_PACKET_TEMPLATE.md`;
- `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md`.

After M01A, the exact toolchain version is owned by `rust-toolchain.toml`.
Commands run plain `cargo`, `rustc`, `rustfmt`, and `clippy` from the repository
root or below it so rustup resolves the pinned toolchain automatically.

`Cargo.toml` retains the separate Cargo-standard `rust-version` declaration,
but the toolchain checker must prove its major/minor equals the exact root pin.

## Required repository changes

### 1. Root Rust pin

Change `rust-toolchain.toml` to:

```toml
[toolchain]
channel = "1.97.1"
profile = "minimal"
components = ["rustfmt", "clippy"]
```

Do not add targets or use the floating `stable`, `beta`, or `nightly` channels.

### 2. Cargo package metadata

In `tethers-0.1/host-rust/Cargo.toml`:

```toml
edition = "2021"
rust-version = "1.97"
```

No dependency, feature, package-version, or lockfile change is permitted.
`Cargo.lock` must remain byte-identical.

### 3. Fail-fast `justfile`

Remove every explicit `+1.89.0` selector.

Use plain Cargo commands with `--manifest-path
tethers-0.1/host-rust/Cargo.toml` instead of `Push-Location` plus semicolon
chains.

Each logical verification step must be a separate Just recipe line so Just
stops on the first non-zero command. In particular:

- `verify` must not continue after a failed packet check, format, check, or test;
- milestone recipes must not continue from a failed first suite to a later
  suite;
- every Cargo build/test/check command keeps `--locked` where Cargo supports it;
- formatting keeps `cargo fmt --all -- --check`.

The PowerShell shell selection may remain unchanged in M01A. Cross-platform
PowerShell naming is reviewed in M01B.

### 4. Environment probe

In `scripts/check-tethers-environment.ps1`:

- remove explicit `+1.89.0` arguments;
- invoke plain Cargo under the root pin;
- retain `--locked` and `--offline` semantics already present;
- add or retain machine-readable evidence that the resolved `rustc --version`
  is Rust 1.97.1;
- do not install tools or mutate repository state.

Do not change the environment-report schema unless required to add one clearly
named Rust-version probe. Existing consumers must remain compatible.

### 5. Toolchain checker

Refactor `.github/scripts/check-tethers-toolchains.ps1` so live versions are read
from repository authority rather than copied into many regexes.

At minimum it must:

1. Read the exact Rust channel from root `rust-toolchain.toml`.
2. Read `rust-version` and edition from host `Cargo.toml`.
3. Require the exact pinned Rust toolchain to be installed without assuming one
   hard-coded host triple.
4. Require rustfmt and Clippy components for that exact channel.
5. Require `rustc --version` to report exactly Rust 1.97.1.
6. Require Cargo's major/minor to match 1.97, without falsely requiring Cargo's
   patch number to equal the rustc point release.
7. Require rustfmt and Clippy invocations to succeed and report their tool names;
   do not freeze unrelated internal formatter patch versions.
8. Prove Cargo edition 2021 and `rust-version = "1.97"`.
9. Continue validating the existing explicit OCaml switch and locked OCaml,
   Dune, and Yojson versions when an OCaml switch is supplied.
10. Preserve the process-local `RUSTUP_AUTO_INSTALL=0` guard and exact restoration
    on success and failure.
11. Remain read-only.

A small helper for reading one TOML string field is preferred over scattered
version regexes. Do not add a TOML parser dependency or external PowerShell
module.

The checker may add a profile parameter only if it remains backwards compatible
with the existing invocation and test script. Do not redesign the entire
preflight system in M01A.

### 6. Checker tests

Update `.github/scripts/test-check-tethers-toolchains.ps1` to prove:

- the exact root Rust pin is consumed rather than an internal copied constant;
- the installed toolchain/component checks use the derived channel;
- the auto-install guard is restored on success and synthetic failure;
- relative, missing, and malformed OCaml switch paths still fail accurately;
- the real authorised switch still succeeds when supplied;
- repository state remains unchanged;
- no test expects Cargo, rustfmt, or Clippy point versions that are not contractual.

Do not weaken existing negative-path coverage.

### 7. Live documentation

Update only live operational guidance:

- `docs/TASK_PACKET_TEMPLATE.md` must tell future packets to read the exact
  channel from `rust-toolchain.toml`, use plain Cargo, and retain `--locked`.
- `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md` must name Rust 1.97.1, edition
  2021, and `rust-version = 1.97`, while directing agents to repository pins
  rather than remembered chat or floating latest documentation.
- add `docs/TOOLCHAIN_POLICY.md` as the short live policy described below.

Do not rewrite historical worker notes, completed packets, release notes, or
accepted evidence merely because they accurately mention Rust 1.89.0 at the
time they ran.

## Toolchain policy document

Create `docs/TOOLCHAIN_POLICY.md` with these decisions:

- exact Rust point release is pinned in `rust-toolchain.toml`;
- Rust stable is reviewed after meaningful milestones and at least monthly while
  development is active;
- security, soundness, and miscompilation point releases are prioritised;
- toolchain upgrades, dependency upgrades, and edition migrations are separate
  jobs;
- `Cargo.lock` is committed and all ordinary verification uses `--locked`;
- OCaml compiler and package truth comes from the explicit switch contract plus
  `tethers_engine.opam.locked`;
- active operational docs are updated, historical evidence is preserved;
- no automatic background upgrade or floating compiler channel is permitted.

Keep it short enough to remain useful.

## Machine installation authority

Matthew explicitly authorised installation of the exact Rust 1.97.1 toolchain
with rustfmt and Clippy if it is missing.

The only authorised machine-state installation command is equivalent to:

```powershell
rustup toolchain install 1.97.1 --profile minimal --component rustfmt --component clippy
```

Do not update the default global toolchain, uninstall Rust 1.89.0, install other
software, modify PATH, or change user/global Cargo configuration.

## Verification order

After the repository changes, run in this order:

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
pwsh -NoProfile -File .github/scripts/test-check-tethers-toolchains.ps1 -OcamlSwitchPath <EXPLICIT_EXISTING_SWITCH>
pwsh -NoProfile -File .github/scripts/check-tethers-toolchains.ps1 -OcamlSwitchPath <EXPLICIT_EXISTING_SWITCH>
just verify
git diff --check
git status --short
```

Before editing, capture the current full-suite result and warnings under Rust
1.89.0 if that toolchain remains installed. After upgrading, report whether
Rust 1.97.1 adds any new warning or failure. Existing documented environmental
failures must be identified precisely rather than silently accepted.

If no explicit authorised OCaml switch path is available in the worktree
context, stop before claiming complete. Do not guess or search neighbouring
worktrees for one.

## Required evidence

The worker note must include:

- exact installed rustc, Cargo, rustfmt, and Clippy version output;
- the derived toolchain and Cargo metadata values used by the checker;
- proof that `Cargo.lock` is byte-identical;
- proof no production source, dependency, OCaml lock, or edition changed;
- before/after warning and full-suite comparison;
- fail-fast Just evidence;
- checker test counts and full preflight result;
- exact changed files;
- a real verified implementation commit SHA.

## Forbidden changes

M01A must not:

- change any Rust production source or tests;
- update dependencies, features, package version, or `Cargo.lock`;
- change OCaml, Dune, Yojson, opam locks, or Dune language;
- migrate to Rust edition 2024;
- delete `.clinerules`, `.clineignore`, scripts, guides, worker notes, releases,
  architecture documents, or any historical file;
- rename `CURRENT_CLINE_TASK.md` or other historical control filenames;
- change Tethers language, Plug lifecycle, trust, execution, provider, Trail, or
  Anchor behaviour;
- modify CI, release tags, branches other than its own, or Git history;
- install anything except the explicitly authorised Rust 1.97.1 components;
- uninstall Rust 1.89.0.

## M01B handoff

M01A should record observed cleanup candidates in the worker note, but must not
remove them. M01B will independently review:

- inactive `.clinerules` and `.clineignore`;
- stale agent-name wording outside the Rust guide;
- required versus optional developer utilities;
- overlap among environment/toolchain diagnostics;
- obsolete roadmaps and duplicated live documentation;
- one-off scripts and files with no active references;
- the existing Rust warning inventory.

The spring clean uses a broom, not a flamethrower.
