# F1 Baseline Transcript

Date: 2026-08-06
Repository: matthewjameswatkins1978-cyber/tethers-lang
Worktree: `D:/The Next Thing/Tethers Lang - Goose Integration`

## Git Baseline

| Fact | SHA |
|---|---|
| `origin/main` | `24428139807cac0adeb0b62264547e61ca809d16` |
| Baseline `HEAD` (prep checkpoint) | `158422a54bede77ea59d6d08fe1fcdb5ed21d499` |
| Merge base (`HEAD`..`origin/main`) | `24428139807cac0adeb0b62264547e61ca809d16` |
| Commits ahead of `origin/main` | 3 |
| Commits behind `origin/main` | 0 |
| Branch | `foundation/f1-baseline` |

### Branch commit log (`origin/main..HEAD`)

```
158422a docs: prepare F1 baseline packet
4262cf5 docs: fix Foundation Pass whitespace
4604c50 docs: define Foundation Pass programme
```

## Toolchain

| Tool | Version |
|---|---|
| Rust (active via rust-toolchain.toml) | 1.97.1-x86_64-pc-windows-msvc |
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

## Build Status (warm)

All commands run at `tethers-0.1/host-rust` with `--locked`.

| Command | Result | Notes |
|---|---|---|
| `cargo fmt --all -- --check` | PASS | No formatting violations |
| `cargo check --all-targets --all-features --locked` | PASS | Compiles with warnings only |
| `cargo test --all-targets --all-features --locked` | FAIL (flaky) | 1 flaky test: `m3_windows_handle_allow_list_excludes_unrelated_inheritable_handle` — failed first run, passed second run |
| `cargo clippy --all-targets --all-features --locked -- -W clippy::all` | PASS (137 warnings) | No hard errors; 137 distinct warning occurrences |
| `just verify` | PASS | Combined rustfmt + cargo check + cargo test (all passed warm) |

## Test Summary

| Category | Count |
|---|---|
| Unit tests passed | 1,254 |
| Integration tests passed warm | All (21 suites) |
| Flaky tests | `m3_windows_handle_allow_list_excludes_unrelated_inheritable_handle` (1) |
| Ignored tests | 2 |

## OCaml Engine

OCaml switch path: N/A (no active switch in this worktree).
Engine source: `tethers-0.1/engine-ocaml/bin/` (924 lines of OCaml across 6 `.ml` files).
Engine was not built during this baseline capture (F1 is documentation-only, Rust toolchain was sufficient for the required matrix).

## Environment

- OS: Windows (native)
- Shell: PowerShell 7 (`pwsh.exe`)
- Dev tools: all present per `scripts/check-dev-tools.ps1`
