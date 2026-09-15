# Worker Note

Task: `LANTERN KEEPER x TETHERS / Milestone 3 authority bridge`

Owner: `Codex`

Status: `IN_PROGRESS`

Base commit: `7066c9604e7062bf46529b9509dc92287e742379`

## Scope

Connect the existing Rust-host Tethers decision path to Lantern's persistent
authority ledger through a host-owned `AuthorityProvider` seam. Tethers policy,
scope validation, exact approval, replay, Trail, and provider dispatch remain
Tethers-owned. Lantern supplies only an independent persistent grant decision
and server-sealed audit receipts.

## Implemented

- Added host-owned authority configuration with explicit principal and
  authority-required capability identities.
- Added `lantern.authority.check/1` and `lantern.receipt.intent/1` contracts,
  canonical scope projection from the existing Tethers scope assessor, and a
  bounded standard-library HTTP provider.
- Added pre-dispatch Lantern decision receipts and post-Trail outcome receipt
  sync, with no retry after execution.
- Added grouped-action pre-dispatch enforcement and cross-repository fixtures.

## Verification so far

- Required startup/tool diagnostics passed.
- `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml` passed.
- Rust formatting check passed after formatting.
- Focused and full verification remain pending before publication.

## Constraints

No OCaml/Core changes, no second scope parser, no Resolve P1 authority, no
OpenShell, no secret in source/config/receipt/export, and no mutation of the
unrelated dirty Tethers checkout at `D:\The Next Thing\Tethers Lang`.
