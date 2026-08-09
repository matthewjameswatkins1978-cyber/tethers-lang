# Current Implementation Task

Control contract: `1`
Task packet: `RELEASE-0.2.2-PREP — Tethers 0.2.2 Release Candidate`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Amber`
Route: `OpenCode prepares complete 0.2.2 release candidate`
Worker note: `docs/worker-notes/2026-08-09-release-0.2.2-prep.md`
Base branch: `foundation/f10-clean-checkout-proof`
Base commit: `5108b06f1f694d6523d5f3f342c08ca0f9b9cbc1`
Implementation branch: `release/v0.2.2-prep`
Implementation checkpoint: `c6ea1e1652fa2785a1f06e0ace2fcd5e826ee6ec`
OCaml switch path: `resolve from existing machine state only`
Rust toolchain: `1.97.1`
Rust change class: `PRODUCT`

## Objective

Prepare the complete Tethers 0.2.2 release candidate: version identity, Cargo
single-source-of-truth, fixture migration, release notes, README front door,
and Foundation completion recording.

## Relevant background and existing behaviour

Foundation F1–F10 has been independently accepted. The previous published
version is v0.2.0. 0.2.1 was never separately published. The CLi has a
hard-coded version string separate from Cargo.toml.

## Required behaviour

1. Complete version-surface inventory: search all `0.2.0`/`v0.2.0` occurrences,
   classify as CURRENT/HISTORICAL/FIXTURE/UNRELATED.
2. Change `Cargo.toml` product version from `0.2.0` to `0.2.2`.
3. Update `Cargo.lock` directly: only local `tethers-reference-host` package
   identity changes `0.2.0 → 0.2.2`; no dependency version, checksum, source,
   or graph change.
4. Replace hard-coded `version = "0.2.0"` in `cli.rs` with
   `env!("CARGO_PKG_VERSION")`; no new dependency, no envelope/exit change.
5. Recapture `docs/foundation-pass/fixtures/cli-output/version.txt` from real
   0.2.2 binary output; envelope/schema/status/exit unchanged.
6. Update `docs/foundation-pass/FIXTURE_MANIFEST.md` with post-Foundation
   migration record; preserve F1 provenance.
7. Update `README.md`: opening introduces Tethers to first-time visitors with
   "Make things happen. Keep the receipts."; current release section describes
   0.2.2 as release candidate.
8. Create `docs/releases/v0.2.2.md` with release candidate status, highlights,
   version history, known limitations; use v0.2.0.md as structural precedent.
9. Update `docs/CURRENT_GOAL.md`: F1-F10 COMPLETE/ACCEPTED, 0.2.2 prep active,
   F10 accepted SHA recorded, main not advanced.
10. Update `docs/PROJECT_DASHBOARD.md`: Foundation COMPLETE/ACCEPTED, 0.2.2
    release candidate prep active, F10 checkpoint recorded.
11. Update `docs/foundation-pass/MODULE_DEPENDENCY_MAP.md`: crate version
    `v0.2.0` → `v0.2.2`.
12. Prove Cargo metadata reports product version exactly `0.2.2`.
13. Prove binary `--version` reports `0.2.2` with envelope/schema/status/exit
    unchanged.
14. Prove captured output matches fixture byte-for-byte, only `0.2.0→0.2.2`
    differs.
15. Run `just verify-agent` once; all Rust tests, formatting, dependencies pass.
16. Run engine tests, MCP transcripts, fixture validator — all pass.

## Relevant components

### AUTHORISED PATHS
- `tethers-0.1/host-rust/Cargo.toml`
- `tethers-0.1/host-rust/Cargo.lock`
- `tethers-0.1/host-rust/src/cli.rs`
- `docs/foundation-pass/fixtures/cli-output/version.txt`
- `docs/foundation-pass/FIXTURE_MANIFEST.md`
- `README.md`
- `docs/releases/v0.2.2.md`
- `docs/CURRENT_GOAL.md`
- `docs/PROJECT_DASHBOARD.md`

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-release-0.2.2-prep.md`

## Frozen decisions and invariants

- Product version: 0.2.2 (patch release).
- Language semantics: 0.1 (unchanged).
- No new product capability.
- No dependency update.
- No broad fixture refresh.
- v0.2.0 history preserved as historical.
- No invented 0.2.1 release.
- No main advance, no tag, no GitHub Release, no publication.

## Acceptance criteria

1. Cargo product version is exactly 0.2.2.
2. CLI reports 0.2.2 from Cargo-owned product metadata.
3. No duplicate hard-coded live CLI product version remains.
4. Cargo.lock changes only the local package identity.
5. Version fixture migration matches real binary output.
6. All other Foundation fixtures unchanged.
7. Public version envelope/schema/status/exit semantics unchanged.
8. Foundation F1–F10 recorded COMPLETE/ACCEPTED.
9. README opens with "Make things happen. Keep the receipts."
10. README and v0.2.2.md agree this is a release candidate.
11. Historical v0.2.0 evidence remains historical.
12. No fictional 0.2.1 release.
13. Focused/version/cross-language checks pass.
14. One complete `just verify-agent` passes.
15. COMPLETE-state task checker passes.
16. Branch is pushed and clean.

## Required verification

1. Version inventory sweep.
2. Cargo metadata reports 0.2.2.
3. Binary `--version` reports 0.2.2 with unchanged envelope.
4. Captured output == fixture byte-for-byte.
5. Directly affected CLI/version tests.
6. Cargo.lock diff: only local package identity.
7. Fixture validator.
8. Final `0.2.0`/`v0.2.0` sweep: all remaining are historical.
9. No duplicate live 0.2.2 constant; CLI derives from Cargo.
10. `just verify-agent` once.
11. Engine tests, MCP transcripts, fixture checks.
12. COMPLETE-state packet checker.

## Forbidden changes

- No new product capability.
- No language-semantic change.
- No dependency update.
- No broad fixture refresh.
- No rewriting v0.2.0 history.
- No invented 0.2.1 release.
- No Clippy cleanup.
- No workflow redesign.
- No unrelated README rewrite.
- No main update, no tag, no GitHub Release, no publication.

## Stop conditions

STOP if version identity, envelope semantics, or fixture-pinning differs from
the packet contract.

## Expected pre-existing changes

None.
