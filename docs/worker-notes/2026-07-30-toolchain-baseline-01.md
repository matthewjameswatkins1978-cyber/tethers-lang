# Worker Note

Task: `TOOLCHAIN-BASELINE-01 — enforce repository toolchain baseline`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Goose`

Status: `COMPLETE`

Base commit: `bb08cc0d09a74db147e3ce6845d4e414e883aad2`

Implementation checkpoint: `<pending commit>`

## Requested outcome

Enforce one reproducible repository-level toolchain baseline: Rust 1.89.0 with
MSRV, rustfmt, clippy, and locked Cargo; OCaml 5.5.0 with tightened
compatibility range and committed opam lock; one non-mutating PowerShell
preflight verifying the baseline without installing software.

## Changes made

1. **`rust-toolchain.toml`** (new) — selects Rust 1.89.0, minimal profile,
   rustfmt and clippy components.
2. **`tethers-0.1/host-rust/Cargo.toml`** — added `rust-version = "1.89"`
   below edition 2021. Dependencies, features, package identity unchanged.
3. **`tethers-0.1/engine-ocaml/tethers_engine.opam`** — tightened OCaml
   compiler range from `>= 5.1.0 & < 6.0.0` to `>= 5.5.0 & < 5.6.0`.
4. **`tethers-0.1/engine-ocaml/tethers_engine.opam.locked`** (new) —
   generated via `opam lock` through the explicit authorised switch.
   Records OCaml 5.5.0, Dune 3.24.0, Yojson 2.2.2. No local paths,
   pins, or unexplained drift.
5. **`.github/scripts/check-tethers-toolchains.ps1`** (new) — non-mutating
   preflight. Requires explicit absolute OcamlSwitchPath, disables rustup
   auto-install process-locally, verifies toolchain/components/versions,
   verifies repository files, restores RUSTUP_AUTO_INSTALL.
6. **`.github/scripts/test-check-tethers-toolchains.ps1`** (new) — 9 focused
   test cases covering missing/relative/wrong switch, no _opam, no
   .opam-switch, RUSTUP_AUTO_INSTALL preservation/removal, no fallback
   search, failure output. 16 assertions, all pass.
7. **`docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md`** — updated "Toolchain and
   dependency truth" section from "does not yet declare" to enforced
   baseline with rust-toolchain.toml, MSRV, and non-mutating preflight.
8. **`docs/OCAML_GUIDE_FOR_AGENTS.md`** — updated sections 6.4, 21.4, 29.1,
   and 29.2 from "approved but unenforced" to "implemented and enforced".
   Removed "future" language about opam.locked.
9. **`docs/TASK_PACKET_TEMPLATE.md`** — added OcamlSwitchPath, Rust
   toolchain, and toolchain preflight fields.
10. **`docs/DECISIONS.md`** — added 2026-07-30 toolchain enforcement decision;
    updated OCaml guide decision reference from "approved but not
    implemented" to "now implemented and enforced".
11. **`docs/CURRENT_CLINE_TASK.md`** — replaced with TOOLCHAIN-BASELINE-01
    packet (COMPLETE).
12. **`docs/worker-notes/2026-07-30-toolchain-baseline-01.md`** — this note.

## Decisions and assumptions

- The opam lock was generated through the exact authorised switch only. No
  search or global-switch fallback was used.
- The tightened OCaml compiler range (>= 5.5.0 & < 5.6.0) is satisfiable
  by the installed 5.5.0 compiler.
- The existing Cargo.lock is preserved byte-identical; no dependency
  changed.
- Pre-existing clippy dead_code warnings exist in production code
  (main.rs, child_process.rs, engine_stdio.rs, event_queue.rs,
  result_anchor.rs). These are not caused by the toolchain baseline
  changes and fixing them would require editing unauthorised production
  source files.
- Pre-existing test-engine.ps1 does not specify an explicit OCaml switch.
  This pre-exists the toolchain baseline and is not a regression.

## Evidence

### Reasoning gate
- GOOSE_THINKING_EFFORT: not set (default)
- .goose/config.json: absent
- Effective level: MEDIUM (default for toolchain tasks)
- Task requires: MEDIUM

### Toolchain readiness (pre-edit)
- rustc: 1.89.0 (29483883e 2025-08-04)
- cargo: 1.89.0 (c24e10642 2025-06-23)
- rustfmt: 1.8.0-stable (29483883ee 2025-08-04)
- clippy: 0.1.89 (29483883ee 2025-08-04)
- ocamlc: 5.5.0, dune: 3.24.0, yojson: 2.2.2

### Cargo.lock
- Original SHA256: d323870ea02f09391a5d0d9aa0e9a701cf686a5ac005b840ee7218e70edb5602
- Final SHA256: d323870ea02f09391a5d0d9aa0e9a701cf686a5ac005b840ee7218e70edb5602
- Unchanged ✓

### opam lock inspection
- OCaml: = "5.5.0" ✓
- Dune: = "3.24.0" ✓
- Yojson: = "2.2.2" ✓
- No local paths, pins, or drift ✓

### Focused preflight tests (test-check-tethers-toolchains.ps1)
16 of 16 assertions pass:
- Missing switch: exit 1, mentions OcamlSwitchPath ✓
- Relative switch: exit 1, mentions absolute ✓
- Nonexistent root: exit 1, mentions nonexistent ✓
- No _opam: exit 1, mentions _opam ✓
- No .opam-switch: exit 1, mentions .opam-switch ✓
- RUSTUP_AUTO_INSTALL preserved ✓
- RUSTUP_AUTO_INSTALL removed when absent ✓
- No worktree search ✓
- Failure output identifies failure ✓

### Real preflight (check-tethers-toolchains.ps1)
24 of 24 checks PASS:
- Rust: toolchain, rustfmt, clippy, rustc 1.89.0, cargo 1.89.0, rustfmt,
  clippy version ✓
- OCaml: opam 2.5.2, switch match, prefix match, ocamlc 5.5.0, ocamlopt
  5.5.0, dune 3.24.0, yojson 2.2.2 ✓
- Repository: rust-toolchain.toml channel, components, Cargo edition,
  rust-version, Cargo.lock, OCaml range, opam.locked OCaml/Dune/Yojson,
  dune-project lang ✓

### Rust verification (rustup run 1.89.0)
- fmt --check: PASS
- check --locked: PASS
- check --tests --locked: PASS
- test --locked: PASS
- build --locked: PASS
- build --release --locked: PASS
- clippy --locked --all-targets --all-features -- -D warnings: FAIL
  (pre-existing dead_code warnings in production code — 13 warnings across
  main.rs, child_process.rs, engine_stdio.rs, event_queue.rs,
  result_anchor.rs; not caused by toolchain baseline changes)

### OCaml verification
- dune build (explicit switch): PASS
- fixtures (check-fixtures.ps1): PASS (46 JSON, 30 JSONL)

### Repository verification
- packet checker (IN_PROGRESS): PASS
- MCP transcripts (test-mcp-transcripts.ps1): PASS (15/15)
- demo (demo.ps1): PASS
- test-engine.ps1: FAIL (pre-existing — does not specify explicit switch)
- Cargo.lock unchanged ✓

### Diff and status
- git diff --check: PASS (no whitespace errors)
- Changed files: 10 (6 modified + 4 new)
- Authorised subset verified

## Discoveries

1. Clippy -D warnings fails due to pre-existing dead_code in production
   Rust source files. These are not within the authorised file set and
   cannot be fixed without changing unauthorised production code.
2. test-engine.ps1 does not pass an explicit opam switch and fails in
   worktrees without a global/default switch set. This pre-dates the
   toolchain baseline.
3. demo.ps1 uses `$env:OPAMSWITCH` which worked when set explicitly.

## Remaining risks

- Pre-existing clippy dead_code warnings will block any future task that
  requires clippy -D warnings to pass. A separate task to add #[allow]
  attributes or remove dead code is recommended.
- test-engine.ps1 needs an explicit switch parameter to work reliably
  across worktrees. Not in this task's scope.

## Smallest next action

A separate Amber task to resolve pre-existing clippy dead_code warnings
or add #[allow(dead_code)] attributes to production Rust source files,
enabling clean clippy -D warnings passes.

## References

- Base commit: bb08cc0d09a74db147e3ce6845d4e414e883aad2
- Branch: goose/toolchain-baseline-01
- OcamlSwitchPath: D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml
- Original Cargo.lock: d323870ea02f09391a5d0d9aa0e9a701cf686a5ac005b840ee7218e70edb5602
- Approved TOOLCHAIN-BASELINE-01 decision per OCaml guide §6.4
