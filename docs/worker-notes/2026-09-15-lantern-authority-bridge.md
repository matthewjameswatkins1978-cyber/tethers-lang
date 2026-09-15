# Worker Note

Task: `LANTERN KEEPER x TETHERS / Milestone 3 authority bridge`

Owner: `Codex`

Status: `READY_FOR_REVIEW`

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

## Verification

- Required startup/tool diagnostics passed.
- `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml` passed.
- Rust formatting check passed after formatting.
- Focused `lantern_authority` provider test passed (1 passed).
- The full verifier passed all 10 suites: task packet checker, Rust
  formatting, OCaml engine/provenance, OCaml tests, Rust static checks,
  warning ratchet, Rust and cross-language tests, protocol fixture sanity,
  MCP transcript suite, and compatibility corpus.
- The report was run against the exact pinned base source commit in the dirty
  implementation worktree; its release-eligibility field was therefore false.
  The committed branch subsequently passed repository verification in CI.

## Constraints

No OCaml/Core changes, no second scope parser, no Resolve P1 authority, no
OpenShell, no secret in source/config/receipt/export, and no mutation of the
unrelated dirty Tethers checkout at `D:\The Next Thing\Tethers Lang`.
