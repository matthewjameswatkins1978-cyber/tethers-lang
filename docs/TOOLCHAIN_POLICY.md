# Tethers Toolchain Policy

Status: current operating policy

## Purpose

Define how the Tethers repository owns its build-toolchain truth and how
compiler, dependency, and edition changes are planned and separated.

## Exact pins

The active Rust toolchain is an exact point release pinned in
`rust-toolchain.toml`, not the floating `stable` channel. The repository uses a
minimal profile with `rustfmt` and `clippy` components declared. Commands run
plain `cargo`, `rustc`, `rustfmt`, or `clippy` from the repository root so
rustup resolves the pinned toolchain automatically.

The declared `rust-version` in host `Cargo.toml` tracks the compiler's
major/minor. Cargo's point version is not required to equal rustc's point-release
number.

OCaml compiler and package truth comes from the explicit opam switch contract
plus the committed `tethers_engine.opam.locked`.

## Review cadence

Rust stable is reviewed after meaningful milestones and at least monthly while
development is active. Security, soundness, and miscompilation point releases
are prioritised and may be applied independently of scheduled review.

## Job separation

Toolchain compiler upgrades, dependency updates, edition migrations, warning
cleanup, and repository-pruning are separate tasks. They are never combined
into one change. Each stands on its own evidence.

## Locked builds

`Cargo.lock` is committed and all ordinary verification uses `--locked`.
Lockfile updates are a deliberate action paired with review, never an automatic
side effect of a toolchain or unrelated code change.

## Historical preservation

Active operational documents are updated when versioned truth changes.
Historical worker notes, completed packets, release notes, and architecture
documents are preserved as written even when they reference an earlier compiler
version.

## Forbidden

No automatic background upgrade or floating compiler channel is permitted. No
global default toolchain, PATH, or Cargo configuration change is made during a
repository toolchain update without explicit separate authorisation.
