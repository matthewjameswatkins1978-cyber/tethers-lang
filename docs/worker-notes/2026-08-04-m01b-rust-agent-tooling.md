# Worker Note

- **Task Packet:** M01B - Rust agent tooling foundation
- **Owner:** OpenCode
- **Status:** COMPLETE
- **Base Commit:** 57e709f7c3fd0a85fdf52d5f027bbd4bdf9af5bf
- **Final Commit:** bb859ae753eaf2433b213800d69185b93e5ff21d
- **Branch:** opencode/m01b-rust-agent-tooling

## Files Modified
- `rust-toolchain.toml` — added `rust-analyzer` component
- `opencode.json` — added `lsp: true` and `permission: { lsp: "allow" }`
- `justfile` — added `agent-tools`, `test-agent`, `deps-policy`, `deps-advisories`, `deps-unused`, `verify-agent`
- `AGENTS.md` — added "Agent toolset" section documenting tool roles and authority
- `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md` — added "Agent tools" section
- `docs/TOOLCHAIN_POLICY.md` — added "Agent tooling" section
- `docs/CURRENT_CLINE_TASK.md` — status transitions

## Files Added
- `tools/rust-agent-tools.json` — frozen tool declarations (schema 1)
- `scripts/install-rust-agent-tools.ps1` — exact, idempotent installer
- `scripts/check-rust-agent-tools.ps1` — read-only non-mutating checker
- `scripts/test-check-rust-agent-tools.ps1` — focused checker tests (6/6 pass)
- `scripts/start-opencode-lsp.ps1` — opt-in LSP launcher with env restoration
- `.config/nextest.toml` — root nextest config (also copied to workspace)
- `tethers-0.1/host-rust/.config/nextest.toml` — workspace-local copy for discovery
- `deny.toml` — concise cargo-deny policy

## Behavioural Result
All four agent tools installed at exact frozen versions. Second install is a no-op.
Checker and focused tests pass. Just recipes integrated. Guidance documents updated.

## Installed Versions
- `rust-analyzer` 1.97.1 (8bab26f4 2026-07-14) — Rust 1.97.1 toolchain component
- `cargo-nextest` 0.9.140 (a9fef2964 2026-07-05)
- `cargo-deny` 0.19.7
- `cargo-machete` 0.9.2
- `opencode` not on PATH — needs wrapper script

## Invariants Preserved
- Cargo.toml, Cargo.lock, dependencies, features, edition, rust-version unchanged
- Production source and tests unchanged
- OCaml files, opam locks, Dune files unchanged
- Rust channel 1.97.1 unchanged
- Cargo.lock SHA-256: `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB` (unchanged)

## Evidence

### Installation & Idempotency
- First install: rust-analyzer (already installed), cargo-nextest 0.9.140 (installed), cargo-deny 0.19.7 (installed), cargo-machete 0.9.2 (installed) — PASS
- Second install: all tools "already installed", no installation performed — PASS (idempotent)

### Checker (read-only, non-mutating)
- `scripts/check-rust-agent-tools.ps1` — 14/14 PASS
- `scripts/test-check-rust-agent-tools.ps1` — 6/6 PASS
  - Missing config rejection: PASS
  - Malformed JSON rejection: PASS
  - Wrong schema rejection: PASS
  - Impossible configured version rejection: PASS
  - Real accepted configuration: PASS
  - Repository non-mutation: PASS

### Nextest
- Config accepted: required version 0.9.140, evaluation ok
- Fail-fast runs ~411/1133 tests (stops at first pwsh.exe failure)
- No-fail-fast: 1128 passed, 5 failed (same 5 pwsh.exe env failures as cargo test)
- Zero retries enforced

### Cargo test (ordinary)
- 921 passed, 5 failed (pwsh.exe not found — pre-existing environmental)
- 30 pre-existing warnings (identical to 1.89.0 baseline)

### Cargo-deny
- bans: ok
- sources: ok
- advisories: ok (no advisories)
- licenses: FAILED on root crate `tethers-reference-host` (no `license` field in Cargo.toml; Cargo.toml changes forbidden by M01B packet; pre-existing)

### Cargo-machete
- No unused dependencies found

### Benchmark (Native Windows, warm, 3 runs each)

| Runner | Run 1 | Run 2 | Run 3 | Median |
|--------|-------|-------|-------|--------|
| `cargo test` | 7.51s | 7.48s | 7.51s | **7.51s** |
| `cargo nextest` (no-fail-fast) | 14.29s | 14.26s | 14.13s | **14.26s** |

Nextest is approximately 2x slower than ordinary Cargo test on this native Windows machine.

### Just Recipes
- `just --list` — PASS (all 16 recipes visible)
- `just agent-tools` — PASS (14/14)
- `just fmt` — PASS
- `just check` — PASS
- `just test-rust` — PASS (921 passed, 5 pwsh failures)
- `just deps-advisories` — PASS (advisories ok)
- `just deps-unused` — PASS (no unused deps found)
- `just deps-policy` — FAILS on license check (root crate, pre-existing)
- `just verify` — FAILS on packet checker (Lucy planning commits, pre-existing)
- `just test-agent` — FAILS on fail-fast + pwsh.exe (environmental)
- `just verify-agent` — FAILS (dependencies on above failures)

### Pre-existing conditions (not M01B regressions)
- Packet checker: expects M01B to be `READY` but detects Lucy's planning commits on `origin/main` after base commit `57e709f` — same pattern as M01A
- 5 `pwsh.exe not found` test failures — environmental, pre-existing
- 30 pre-existing compiler warnings — 15 in lib, 15 in tests
- Root crate `tethers-reference-host` has no `license` field — pre-existing

### OpenCode/LSP
- `opencode.json` — LSP enabled, permission `allow`
- `opencode` binary not on PATH — wrapper script provided, LSP environment variables configured
- `opencode debug config` — cannot be verified because `opencode` is not on PATH
- Repository configuration is ready; a currently running process need not hot-reload

## Discoveries
- cargo-deny `deny.toml` config format differs from the documentation of some versions: `[licenses]` `copyleft` and `default` keys are removed in 0.19.7; `unlicensed` was also removed. Exception matching for workspace crates without a `license` field does not work as expected.
- Nextest config must be discoverable from the Cargo workspace root directory. A `.config/nextest.toml` at the repo root is not found by `cargo nextest` when the manifest is at `tethers-0.1/host-rust/Cargo.toml`. Both locations are provided.
- Nextest with `fail-fast = true` stops at the first test failure (~3.4s), making it unsuitable for running the complete test graph when environmental failures exist. Without fail-fast, it runs all tests (~14.3s) but is ~2x slower than ordinary `cargo test` (~7.5s).

## Remaining Risks
- OpenCode's LSP integration cannot be verified without `opencode` on PATH. Repository configuration is ready and the wrapper script correctly sets `OPENCODE_EXPERIMENTAL_LSP_TOOL=true` and `OPENCODE_DISABLE_LSP_DOWNLOAD=true` with environment restoration. Acceptance must wait until the wrapper is used for a real OpenCode process.

## M01C candidates
- Root crate `license` field addition (unblocks `just deps-policy`)
- `syn` duplicate version (2.0.119 and 3.0.1) — dependency harmonisation
- 30 pre-existing compiler warnings
- Packet checker: planning-commit detection logic
- Stale/unused documentation files (per M01A worker note)

## Recommended Next Action
Lucy reviews pushed M01B evidence and either accepts or compiles a bounded correction.
