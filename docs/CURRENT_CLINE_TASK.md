# Current Implementation Task

Control contract: `1`

Task: `TOOLCHAIN-BASELINE-01 — enforce repository toolchain baseline`

Status: `COMPLETE`

Task colour: `Amber`

Owner: `Goose`

Route: `Goose Medium — bounded toolchain enforcement`

Worker note: `docs/worker-notes/2026-07-30-toolchain-baseline-01.md`

Base branch: `main`

Base commit: `bb08cc0d09a74db147e3ce6845d4e414e883aad2`

Branch: `goose/toolchain-baseline-01`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

## Objective

Enforce one reproducible repository-level toolchain baseline: Rust 1.89.0
with MSRV, rustfmt, clippy, and locked Cargo; OCaml 5.5.0 with a tightened
compatibility range, committed opam lock recording Dune 3.24.0 and Yojson
2.2.2; and one non-mutating PowerShell preflight that verifies the baseline
without installing software.

## Relevant background and existing behaviour

TOOLCHAIN-BASELINE-01 was approved on 30 July 2026 but remained unenforced.
The OCaml guide and DECISIONS.md recorded the intent. The readiness audit
(TOOLCHAIN-BASELINE-01-R0) confirmed the machine and explicit directory
switch are ready. The existing package constraints are broader than the
approved baseline: Cargo lacks rust-version, the OCaml opam file accepts
compilers back to 5.1.0, no rust-toolchain.toml or opam lock exists.

## Required behaviour

1. Create `rust-toolchain.toml` selecting Rust 1.89.0, minimal profile,
   rustfmt and clippy components.
2. Add `rust-version = "1.89"` to `Cargo.toml` while preserving edition 2021.
3. Tighten `tethers_engine.opam` OCaml range to `>= 5.5.0 & < 5.6.0`.
4. Generate `tethers_engine.opam.locked` through the explicit authorised
   switch, recording OCaml 5.5.0, Dune 3.24.0, Yojson 2.2.2.
5. Create `.github/scripts/check-tethers-toolchains.ps1` — non-mutating
   preflight requiring explicit OcamlSwitchPath, disabling rustup auto-install
   process-locally, restoring the prior value, and verifying versions.
6. Create `.github/scripts/test-check-tethers-toolchains.ps1` — focused
   PowerShell tests covering missing/relative/wrong switch, no _opam,
   no .opam-switch, authorised switch success, RUSTUP_AUTO_INSTALL
   preservation, failure output, no fallback search, no repository changes.
7. Update `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md` and
   `docs/OCAML_GUIDE_FOR_AGENTS.md` to state TOOLCHAIN-BASELINE-01 is
   now enforced.
8. Update `docs/TASK_PACKET_TEMPLATE.md` for toolchain-preflight and
   explicit-switch declarations. Inspect `docs/PROJECT_CONTROL.md` for
   any necessary toolchain-related wording; no change was required.
9. Add the enforcement decision to `docs/DECISIONS.md`.
10. Verify unchanged Cargo.lock; pass all Rust, OCaml and repository checks.

## Relevant components

- `rust-toolchain.toml` — new root toolchain selector
- `tethers-0.1/host-rust/Cargo.toml` — edition and MSRV
- `tethers-0.1/engine-ocaml/tethers_engine.opam` — compiler range
- `tethers-0.1/engine-ocaml/tethers_engine.opam.locked` — new exact lock
- `.github/scripts/check-tethers-toolchains.ps1` — new non-mutating preflight
- `.github/scripts/test-check-tethers-toolchains.ps1` — new focused tests
- `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md` — enforced-baseline update
- `docs/OCAML_GUIDE_FOR_AGENTS.md` — enforced-baseline update
- `docs/TASK_PACKET_TEMPLATE.md` — toolchain declarations
- `docs/PROJECT_CONTROL.md` — narrow project-control wording
- `docs/DECISIONS.md` — enforcement decision record
- `docs/CURRENT_CLINE_TASK.md` — this task packet
- `docs/worker-notes/2026-07-30-toolchain-baseline-01.md` — evidence

## Frozen decisions and invariants

- Rust 1.89.0 is the sole development toolchain; MSRV is 1.89; edition 2021.
- OCaml 5.5.0 is the sole development compiler; compatibility is 5.5.x only.
- Dune 3.24.0 and Yojson 2.2.2 are locked; Dune language remains 3.10.
- Cargo.lock is unchanged and authoritative; --locked is mandatory.
- The preflight is non-mutating; no installation, upgrade, or repair.
- The explicit absolute OcamlSwitchPath is mandatory; no worktree search.
- RUSTUP_AUTO_INSTALL is disabled process-locally and restored.
- No bare cargo, rustc, rustfmt or clippy invocation in the preflight.
- Toolchain upgrades require a separate decision.

## Acceptance criteria

1. Branch started from exact base `bb08cc0d09a74db147e3ce6845d4e414e883aad2`.
2. Effective Goose reasoning confirmed MEDIUM before mutation.
3. `rust-toolchain.toml` selects exact Rust 1.89.0, minimal, rustfmt, clippy.
4. Cargo declares edition 2021 and rust-version 1.89.
5. Cargo.lock unchanged (SHA256: `d323870ea...`).
6. OCaml range is `>= 5.5.0 & < 5.6.0`.
7. opam lock records OCaml 5.5.0, Dune 3.24.0, Yojson 2.2.2.
8. Lock has no local path, pin or unexplained drift.
9. Preflight is genuinely non-mutating; RUSTUP_AUTO_INSTALL guarded.
10. No bare Rust proxy invocation; no worktree search or global-switch fallback.
11. Focused preflight tests pass all cases.
12. Rust fmt, check, tests, clippy, builds pass with 1.89.0 and --locked.
13. OCaml dune build passes with the explicit switch.
14. Fixture, engine, MCP and demo scripts pass.
15. Packet checker and whitespace checks pass.
16. Only authorised files changed.
17. Review branch pushed; main untouched.
18. Original worktree and TETHERS_LUCY_NOTES.md untouched.

## Forbidden changes

No production runtime, Tethers language/protocol, permission, replay, Trail,
persistence, or dispatch change. No Dune language, Yojson, Rust crate,
Cargo.lock, edition, or dependency change. No ocamlformat, reformatting,
software installation, or global configuration change. No .gitattributes,
.editorconfig, or Git configuration change. No merge or push to main. No
amend, squash, rebase, or force-push. No branch or worktree deletion.

## Stop conditions

Return BLOCKED when: origin/main mismatch, dirty worktree, branch exists,
reasoning not MEDIUM, Rust components missing, switch missing or wrong,
lock contains unexpected versions/paths/pins, Cargo.lock changes, preflight
cannot remain non-mutating, original worktree changes, two similar failures.

## Expected pre-existing changes

None. Starting from clean `main` at `bb08cc0d09a74db147e3ce6845d4e414e883aad2`.

## Required verification

```powershell
# Rust (all proxied through rustup run 1.89.0, RUSTUP_AUTO_INSTALL=0)
rustup run 1.89.0 cargo fmt --manifest-path .\tethers-0.1\host-rust\Cargo.toml --check
rustup run 1.89.0 cargo check --manifest-path .\tethers-0.1\host-rust\Cargo.toml --locked
rustup run 1.89.0 cargo check --manifest-path .\tethers-0.1\host-rust\Cargo.toml --locked --tests
rustup run 1.89.0 cargo test --manifest-path .\tethers-0.1\host-rust\Cargo.toml --locked
rustup run 1.89.0 cargo clippy --manifest-path .\tethers-0.1\host-rust\Cargo.toml --locked --all-targets --all-features
rustup run 1.89.0 cargo build --manifest-path .\tethers-0.1\host-rust\Cargo.toml --locked
rustup run 1.89.0 cargo build --manifest-path .\tethers-0.1\host-rust\Cargo.toml --locked --release

# OCaml (explicit switch)
$OcamlSwitchPath = "D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml"
pwsh -NoProfile -File .\.github\scripts\test-check-tethers-toolchains.ps1 -OcamlSwitchPath $OcamlSwitchPath
pwsh -NoProfile -File .\.github\scripts\check-tethers-toolchains.ps1 -OcamlSwitchPath $OcamlSwitchPath
Push-Location .\tethers-0.1\engine-ocaml
opam exec --switch="$OcamlSwitchPath" -- dune build
Pop-Location

# Repository
pwsh -NoProfile -File .\.github\scripts\check-tethers-task-packet.ps1
pwsh -NoProfile -File .\tethers-0.1\scripts\check-fixtures.ps1
$env:OPAMSWITCH = "D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml"
pwsh -NoProfile -File .\tethers-0.1\scripts\test-engine.ps1
Remove-Item Env:OPAMSWITCH -ErrorAction SilentlyContinue
pwsh -NoProfile -File .\tethers-0.1\scripts\test-mcp-transcripts.ps1
pwsh -NoProfile -File .\tethers-0.1\scripts\demo.ps1

# Diff and status
git diff --check
git diff --check bb08cc0d09a74db147e3ce6845d4e414e883aad2..HEAD
git diff --stat bb08cc0d09a74db147e3ce6845d4e414e883aad2..HEAD
git diff --name-status bb08cc0d09a74db147e3ce6845d4e414e883aad2..HEAD
git status --short --branch
```

## Discoveries

1. Ordinary Clippy exits zero with 24 pre-existing warnings across production
   and test code (dead_code, unused_imports, unused_variables, clippy
   complexity). These are not caused by the toolchain baseline changes and
   do not block TOOLCHAIN-BASELINE-01 acceptance.
2. `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md` independently establishes
   `cargo clippy ... -- -D warnings` which pre-dates this task. Not edited;
   a future task should reconcile.
3. `demo.ps1` and `test-engine.ps1` require an explicit OPAMSWITCH when no
   global switch is set. Both pass with process-local OPAMSWITCH.

## Frozen exclusions

Do not change production runtime logic, the Tethers language or protocol,
permissions, replay, Trail, persistence, dispatch, Dune language, Yojson,
Rust crates, Cargo.lock, Rust edition, Windows binary-mode stdio, Git
configuration, .gitattributes, or .editorconfig. Do not adopt ocamlformat,
reformat code, or install/upgrade software.
