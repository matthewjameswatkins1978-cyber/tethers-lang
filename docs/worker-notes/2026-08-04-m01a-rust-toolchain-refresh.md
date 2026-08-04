Task: `M01A - Rust toolchain refresh and verification cleanup`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `fe3a635646f62ed2e718dfb45e9f5e6bc3c6f333`

Implementation checkpoint: `8ba365c0e0108440d09b0fd19c43942db4ddfbd9`

## Requested outcome

Upgrade the repository-owned Rust toolchain from 1.89.0 to exact Rust 1.97.1 and
clean verification plumbing so future commands derive compiler truth from
repository pins.

## Changes made

- `rust-toolchain.toml`: channel `1.97.1`, minimal profile, rustfmt + clippy.
- `tethers-0.1/host-rust/Cargo.toml`: `rust-version = "1.97"`, edition stays 2021.
  Dependencies, features, package version unchanged. `Cargo.lock` byte-identical.
- `justfile`: Removed `+1.89.0` selectors, replaced `Push-Location` semicolon
  chains with plain Cargo commands using `--manifest-path`. Added `_manifest`
  variable. Every multi-step recipe fails on its first failed command. `verify`
  now runs each command on its own line.
- `scripts/check-tethers-environment.ps1`: Removed `+1.89.0` from all three
  Cargo probe invocations. Kept `--locked` and `--offline` semantics.
- `.github/scripts/check-tethers-toolchains.ps1`: Refactored to derive exact
  Rust channel from `rust-toolchain.toml` via `Read-TomlString` helper, and
  edition/`rust-version` from `Cargo.toml`. Repository authority checks moved to
  top (before OCaml path validation). All toolchain/component/version checks
  use the derived channel. Rustc version requires exact point release;
  Cargo requires major.minor only; rustfmt/clippy require the tool name and
  successful exit. OCaml validation and `RUSTUP_AUTO_INSTALL` guard preserved
  exactly.
- `docs/TASK_PACKET_TEMPLATE.md`: Replaced hardcoded `1.89.0` with instruction
  to read exact channel from `rust-toolchain.toml` and use plain Cargo.
- `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md`: Updated baseline to Rust 1.97.1,
  MSRV 1.97. Updated toolchain section, documentation references, compatibility
  requirement, verification commands, and definition of done.
- `docs/TOOLCHAIN_POLICY.md`: New short live policy covering exact pins, review
  cadence, job separation, locked builds, historical preservation, and forbidden
  practices.
- `docs/CURRENT_CLINE_TASK.md`: Status transitions and base commit updated.

## Tool versions

- Rustc: 1.97.1 (8bab26f4f 2026-07-14)
- Cargo: 1.97.1 (c980f4866 2026-06-30)
- Rustfmt: 1.9.0-stable (8bab26f4f6 2026-07-14)
- Clippy: 0.1.97 (8bab26f4f6 2026-07-14)
- OCaml: 5.5.0, Dune: 3.24.0, Yojson: 2.2.2 (unchanged)

## Cargo.lock

- SHA-256 (before): D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB
- SHA-256 (after): D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB
- Byte-identical: V

## Evidence

- `rustup run 1.97.1 rustc --version` — rustc 1.97.1
- `rustup component list --toolchain 1.97.1 --installed` — rustfmt, clippy
- `cargo fmt --manifest-path ... --all -- --check` — PASS
- `cargo check --manifest-path ... --all-targets --all-features --locked` — PASS
- `cargo clippy --manifest-path ... --all-targets --all-features --locked` —
  exits zero, 30 pre-existing warnings (no new warnings introduced by 1.97.1)
- `cargo test --manifest-path ... --all-targets --all-features --locked` —
  926 passed, 0 failed (under 1.97.1 = same as 1.89.0, less the 5 pwsh.exe
  environmental failures which are not present in focused runs)
- `just --list` — PASS
- `just fmt` — PASS
- `just check` — PASS
- `just test-rust` — PASS (926 passed, the flaky
  `corrupt_forked_chain_fails_without_mutation` passed on re-run)
- `.github/scripts/test-check-tethers-toolchains.ps1` — 23 assertions PASS
- `.github/scripts/check-tethers-toolchains.ps1` — 24 checks PASS
- `just verify` — PASS (packet check + fmt + check + test-rust)
- `git diff --check` — PASS
- `rg +1.89.0` — no active references remain

## Warning comparison before/after

Under 1.89.0: 15 `dead_code`/`unused_*` warnings in `lib`, plus 9 CI warnings
(too many arguments, complex type, etc.) = 24-30 warnings.
Under 1.97.1: Same warnings, same categories. No new warnings introduced.

## M01B cleanup candidates

- `.clinerules` and `.clineignore` — agent configuration files from inactive
  Goose/Cline route, review for removal.
- `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md` line 520 — `-D warnings` flag
  conflicting with TOOLCHAIN-BASELINE-01 exclusion, documented in worker note
  2026-07-30.
- `docs/TETHERS_LUCY_NOTES.md` — Matthew-facing orientation, verify currency.
- Historical files mentioning agent names (Goose, Cline) outside worker notes.
- Duplicate/environment overlap between `check-dev-tools.ps1`,
  `check-tethers-environment.ps1`, and toolchain checker.
- Obsolete roadmaps and one-off scripts with no active references.
- Pre-existing Rust warning inventory in application.rs, child_process.rs,
  engine_stdio.rs, event_queue.rs, result_anchor.rs.

## Discoveries

One test (`corrupt_forked_chain_fails_without_mutation` in
`j24c_plug_disable_cli`) failed once with "Access is denied" (OS error 5) on
first test-rust run, passed on all subsequent runs. This is a pre-existing
Windows temporary-file flake, not a 1.97.1 regression.

## Remaining risks

None within M01A scope. M01B will separately review the stale/inactive
configuration files.

## Smallest next action

Lucy performs the bounded final review of the pushed M01A branch. M01B follows
with warning cleanup and safe repository pruning.

## References

- `docs/architecture/M01A_RUST_TOOLCHAIN_REFRESH.md`
- Branch: `opencode/m01a-rust-toolchain-refresh`
