# Tethers L1 - Linux Parity Recovery

Status: BLOCKED at the authoritative repository-verification boundary.

## Starting point

- Native WSL workshop: `/home/matmus/biscuit-linux`
- Task worktree: `/home/matmus/biscuit-linux/workspaces/tethers-l1`
- Branch: `codex/tethers-linux-parity-finish`
- Starting current accepted `origin/main`: `15c85c7ffffb56614b0667b7a39164529a1b7fc4`
- Historical PR #25 preserved at: `ebed49020ed5686b299d31d7563802b7ecb1e478`
- L0 workshop baseline: Ubuntu 26.04.1 LTS, WSL2 kernel `6.18.33.2-microsoft-standard-WSL2`, Rust `1.97.1`, OCaml `5.5.0`, Dune `3.24.0`, Yojson `2.2.2`.

## PR #25 delta classification

| Area | Classification | Evidence / decision |
| --- | --- | --- |
| Linux replay persistence and symlink refusal | STILL REQUIRED | Current main had the platform gap; carried as `replay_linux.rs` and `replay_store.rs`. |
| Unix process supervision | STILL REQUIRED | Carried into `child_process.rs`; Linux process-group cleanup tests pass. |
| Unix installation locking | STILL REQUIRED | Carried into `installation_execution.rs`; implementation is present, while the historical lock unit module remains Windows-gated. |
| Linux provider fixture boundary | STILL REQUIRED / TEST-ONLY | PowerShell fixture assumptions were not portable; replaced the Linux fixture with a native protocol-equivalent implementation. |
| Current Resolve/P-stage host files | CONFLICTS / SUPERSEDED | Not blindly merged; current main remains authoritative. |
| PR packaging scripts and PowerShell CI | OBSOLETE FOR L1 | Deferred to the later Linux package/support proof. |
| PR documentation and task-packet history | DOCS-ONLY / NOT CARRIED | Historical PR remains preserved as evidence. |

Threadmoth was used for structural inspection of the current host process seam
and the PR-only replay module. It confirmed the replay module was absent from
current main and helped bound the selective carry-forward. Behavioural claims
were verified by compilation and tests, not by structural matches alone.

## Changes

The implementation was carried forward in bounded commits:

- `f759818` - port Linux replay and host boundaries;
- `6c8f55a` - port Linux provider fixture boundaries;
- `1d5f0f3` - enable Linux installation path coverage;
- `8734e74` - carry forward applicable Linux fixture portability;
- `3466f7e` - complete the native Linux stdio fixture and guarded fixture
  transport.

The final fixture repair adds `tethers-stdio-fixture.py` behind the existing
POSIX shell entry point. It mirrors the tested Windows fixture modes for
initialisation, paginated discovery, opaque cursor loops, catalogue
notifications/drift, provider calls, barriers, hangs, stderr and marker
behaviour. Windows continues to use the existing `.ps1` fixture.

Production semantics changed: NO.

Windows behaviour changed: NO. No Windows checkout or Windows fixture was
modified.

## Native engine and provenance

The explicit workshop switch is `bl-tethers-5.5.0`. The native Core build
passed using:

`opam exec --switch=bl-tethers-5.5.0 -- dune build --root tethers-0.1/engine-ocaml`

The verified engine is:

`tethers-0.1/engine-ocaml/_build/default/bin/tethers_mcp_main.exe`

It is a Linux ELF executable despite the existing `.exe` suffix. The current
manifest binds it to the task checkpoint and records the SHA-256:

`4c89ff26024733218311086f0fff66c0a7fe2e652363a636a7efd358bd60e9af`

The runner also proved fail-closed rejection of:

- stale source commit: `Current engine provenance does not match this source checkout; Rust tests were not attempted.`
- tampered binary hash: `Current engine binary hash does not match provenance; Rust tests were not attempted.`
- missing engine: `Current engine binary is missing or not executable; Rust tests were not attempted.`

Valid provenance was accepted and exported through
`TETHERS_VERIFIED_ENGINE` and `TETHERS_ENGINE_PROVENANCE`.

## Verification evidence

Focused native checks:

- socket provider protocol: `8 passed, 0 failed`;
- stdio provider: `21 passed, 0 failed`;
- check-command provider tests: `14 passed, 0 failed`;
- guarded C2-A3 tests: `16 passed, 0 failed`;
- C3-A tests: `26 passed, 0 failed`;
- replay Linux lifecycle/symlink tests: `2 passed, 0 failed`;
- Unix process group/nonexistent-command tests: `4 passed, 0 failed`;
- full Rust host/integration runner, single-thread: `1,519 passed, 0 failed, 2 ignored`;
- full Rust host/integration runner, default threading: `1,519 passed, 0 failed, 2 ignored`;
- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`: PASS;
- `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml --locked`: PASS;
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --no-run --locked`: PASS;
- `git diff --check`: PASS.

The accepted warning debt remains visible: duplicate target, unused imports/
variable, unnecessary mutability, dead Windows helper on Unix. No warnings
were globally suppressed.

## Verification boundary and blocker

`just verify` was attempted from the task worktree. It does not execute a
native Linux verifier. The recipe invokes `pwsh`, which crosses into the
Windows PowerShell installation through WSL interop and fails before the
repository verifier runs:

`SecurityError: File \\wsl.localhost\\Ubuntu\\home\\matmus\\biscuit-linux\\workspaces\\tethers-l1\\scripts\\verify-tethers.ps1 cannot be loaded. The file .../scripts/verify-tethers.ps1 is not digitally signed.`

This is an external verification-tooling boundary, not evidence of a product
portability failure. The workshop `bl verify tethers` command passes its
lightweight check, but it targets the canonical workshop checkout at revision
`786aa0ec...`, not this task worktree, and therefore is not claimed as the
authoritative L1 verification result.

The native Rust runner is complete, but L1 is not marked complete until the
repository-owned authoritative verification route can run natively or the
project explicitly accepts this external-tooling limitation.

## Remaining work

- Provide or authorise a native Linux equivalent of the full repository
  verifier, including its packet/documentation/compatibility gates, or record
  an explicit owner decision that those PowerShell-only gates are external to
  L1.
- Re-run the resulting authoritative verification and final diff/status checks.
- Push the blocked branch for review; do not begin L2 until the gate is closed.
