# F1 Baseline Transcript

Date: 2026-08-06
Repository: matthewjameswatkins1978-cyber/tethers-lang
Worktree: `foundation/f1-baseline` at `D:/The Next Thing/Tethers Lang - Goose Integration`

## Environment

- OS: Windows (native), NTFS file system
- Shell: PowerShell 7 (`pwsh.exe`)
- Dev tools: all present per `scripts/check-dev-tools.ps1`
- Worktree: existing clean worktree (NOT a fresh clone). The worktree was prepared from `origin/main` at `24428139807cac0adeb0b62264547e61ca809d16` with the Foundation Pass programme commits layered on top. The F1 preparation checkpoint is `158422a54bede77ea59d6d08fe1fcdb5ed21d499`.
- Rust build artefact: warm build from `tethers-0.1/host-rust/`. Binary was `tethers-reference-host.exe` in `target/debug/`.
- OCaml engine: compiled artefacts exist in `engine-ocaml/_build/install/default/bin/` from a prior build. No active opam switch in this worktree.

## Git Baseline

| Fact | SHA |
|---|---|
| `origin/main` | `24428139807cac0adeb0b62264547e61ca809d16` |
| Prep checkpoint (initial F1 branch state) | `158422a54bede77ea59d6d08fe1fcdb5ed21d499` |
| Merge base (`HEAD`..`origin/main`) | `24428139807cac0adeb0b62264547e61ca809d16` |
| Commits ahead of `origin/main` | 3 (programme preamble) |
| Commits behind `origin/main` | 0 |
| Branch | `foundation/f1-baseline` |

### Branch commit log (`origin/main..HEAD` at prep checkpoint)

```
158422a docs: prepare F1 baseline packet
4262cf5 docs: fix Foundation Pass whitespace
4604c50 docs: define Foundation Pass programme
```

## Toolchain

| Tool | Version |
|---|---|
| Rust (active via `rust-toolchain.toml`) | 1.97.1-x86_64-pc-windows-msvc |
| rustup default host | x86_64-pc-windows-msvc |
| Cargo | 1.97.1 (c980f4866 2026-06-30) |
| PowerShell | 7.6.4 |
| Git | 2.54.0.windows.1 |
| just | 1.57.0 |
| ripgrep | 15.2.0 |
| fd | 10.4.2 |
| jq | 1.8.2 |
| yq | 4.53.3 |
| gh | 2.97.0 |

## Build Status

All commands run at `tethers-0.1/host-rust` with `--locked`.

Preconditions: warm `target/` directory from prior build. OCaml engine binaries pre-built. No cold-from-scratch timing captured.

| Command | Result | Timing | Notes |
|---|---|---|---|
| `cargo fmt --all -- --check` | PASS | NOT CAPTURED (warm build; timing not recorded) | No formatting violations |
| `cargo check --all-targets --all-features --locked` | PASS | NOT CAPTURED | Compiles with warnings only |
| `cargo test --all-targets --all-features --locked` | FAIL (cold), PASS (warm) | Cold (first run): NOT CAPTURED. Warm (second+): NOT CAPTURED | 1 flaky test: `m3_windows_handle_allow_list_excludes_unrelated_inheritable_handle` — failed first run (cold test binary), passed on second and subsequent runs |
| `cargo clippy --all-targets --all-features --locked -- -W clippy::all` | PASS (137 warnings) | NOT CAPTURED | No hard errors; 137 distinct warning occurrences |

Cold/warm distinction: A "cold" run means the test binary freshly built and executed without prior process warm-up. A "warm" run means the binary was already compiled (from the prior `cargo check`) and the OS disk cache was populated from the previous test run. F1 did not measure cold-from-power-on or cold-from-clean-build timings.

## Test Summary

| Category | Count |
|---|---|
| Unit tests passed | 1,254 |
| Integration tests passed (warm) | All (21 suites: J13, J23, J24, M3, M4, M5) |
| Flaky tests | 1: `m3_windows_handle_allow_list_excludes_unrelated_inheritable_handle` |
| Ignored tests | 2 |

## OCaml Engine

OCaml switch path: N/A (no active switch in this worktree).
Engine source: `tethers-0.1/engine-ocaml/bin/` (924 lines of OCaml across 6 `.ml` files).
Engine binaries: pre-built artefacts exist at `tethers-0.1/engine-ocaml/_build/install/default/bin/` (tethers_engine, tethers_mcp_server) from a prior build.
Engine was not rebuilt during F1. OCaml-level warnings were not captured.
