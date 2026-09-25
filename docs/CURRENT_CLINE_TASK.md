# TETHERS 0.8.1 — MAINTENANCE, LINUX & DISTRIBUTION CLOSEOUT

Control contract: `1`

Status: `IN_PROGRESS`

Task colour: `Amber`

Owner: `Tethers 0.8.1 Maintenance Agent`

Route: `OpenCode implementation on canonical Windows checkout; Linux proof via Ubuntu CI; gated tag/crates.io final steps`

Worker note: `docs/worker-notes/2026-09-25-tethers-0.8.1-maintenance.md`

Base branch: `main`

Base commit: `c5bd371ff348879313186406696457f7024f4446`

The Base commit is `v0.8.0` (Merge pull request #48). All 0.8.1 work builds forward from it.

Supersession note: this packet supersedes the stale `IN_PROGRESS` Codex 0.8.0-readiness
packet previously in this file (base `7e29110`). That 0.8.0 objective is factually
complete on `main` (`v0.8.0` tag resolves to the Base commit above); its packet state
was never flipped to `COMPLETE`/`ACCEPTED`. No Codex work is overwritten: the prior
packet text remains in Git history. Full acceptance contract is the supplied
`TETHERS 0.8.1 — MAINTENANCE, LINUX & DISTRIBUTION CLOSEOUT` packet; this file is
its condensed control record, not a second competing specification.

Rust toolchain: `1.97.1`; plain Cargo resolved by root pin; `--locked` mandatory

Toolchain preflight: `required`

## Objective

Close the named 0.8.0 engineering/distribution gaps as boring maintenance: repair
`cargo test --release` without faking the release profile; account for every ignored
Rust test; port PR #45 Linux-packaging proof onto current `tethers.authority/1`
semantics without merging it; preserve Windows x86-64 proof and crate quality;
ship 0.8.1 with exact provenance. No architecture redesign, no feature release,
no weakening of `tethers.authority/1`.

## Relevant background and existing behaviour

- `cargo test --release` fails to compile test-harness code gated on debug assertions.
- 9 Rust tests are ignored without a current disposition record.
- PR #45 (`codex/tethers-l2-linux-package-proof`, old head `0132dbba`) proved Linux
  packaging against pre-0.8 (0.7.1-era) assumptions; it is non-mergeable and donor-only.
- Windows x64 runtime flow: `scripts/package-tethers-0.8-release.ps1` (hard-codes 0.8.0).
- Crate `tethers-reference-host 0.8.0`; crates.io publication deferred in 0.8.0.
- External authority boundary `tethers.authority/1` via `authority_v1`; R2 gate suite
  `tethers-0.1/host-rust/tests/r2_authority_gate.rs` is the hostile-regression anchor.

## Required behaviour

1. Release-profile test harness compiles and passes genuinely (`cfg(test)`-scoped
   helpers, not global `debug-assertions = true`); `scripts/run-rust-tests.ps1 -Release` green.
2. Every ignored test classified A–E (run / kept-manual / platform-gated / deleted /
   deferred-gap) with a final disposition table; zero unexplained ignored tests.
3. Linux x86-64 full runtime (Rust Host + matching OCaml engine) built natively,
   packaged version-driven from `VERSION`, clean-package smoked (version/help, engine
   availability, path independence, permissions, `ldd` audit, provider lifecycle,
   recovery/replay, external-consumer authority smoke incl. hostile cases).
4. Windows x86-64 regression proof preserved; packager generalised off hard-coded 0.8.0.
5. `cargo package --locked` and `cargo publish --dry-run --locked` pass; package
   contents inspected; version bumped to 0.8.1 at release-candidate stage only.
6. Actual crates.io publication only as the final gated step with credentials and
   release authority; otherwise report PUBLICATION BLOCKED with the exact command.
7. Release provenance (source SHA/tree hash, tool versions, artifact hashes, manifests)
   for every artifact; `VERSION = 0.8.1`, release notes, README/support-matrix updates
   from evidence only.
8. R2 authority behaviour bit-for-bit preserved; no `tethers.authority/2`.

## Relevant components

- `tethers-0.1/host-rust/` (Cargo.toml, src, tests incl. `r2_authority_gate.rs`)
- `tethers-0.1/engine-ocaml/`
- `scripts/run-rust-tests.ps1`, `scripts/run-rust-tests.sh`
- `scripts/package-tethers-0.8-release.ps1`, new `scripts/package-linux-release.sh`
- `scripts/prepare-current-engine.sh`, `scripts/verify-tethers.py`
- `scripts/check-dev-tools.ps1` (repair REQUIRED/OPTIONAL distinction)
- `.github/workflows/tethers-verification.yml`, Linux package lane
- `examples/external-consumer/`, `examples/rust-crate-consumer/`
- `docs/INTEGRATING_TETHERS.md`, `docs/SUPPORT_MATRIX.md`, `VERSION`, `justfile`

## Frozen decisions and invariants

- A Plan is not permission; approval is not execution; COMMIT records durable intent;
  the external Host alone performs effects; outcomes stay truthful; forged/malformed
  authority input fails closed.
- Do not enable release `debug-assertions` globally; do not merge PR #45 wholesale;
  do not resurrect 0.7.1 assumptions; do not weaken tests for cross-platform green;
  do not claim Linux official without all support-matrix gates; do not claim crates.io
  publication on dry-run alone; no proof laundering across SHAs.
- Linux x86-64 only. No macOS/ARM/mobile/remote-hosted scope creep.

## Acceptance criteria

1. `cargo test --release` (genuine profile, `--all-targets --all-features --locked`) compiles and passes.
2. Ignored-test disposition table complete; `ignored-only` and `include-ignored` runs reported.
3. Linux x64 runtime packaged from the release tag, clean-package authority smoke green.
4. Windows x64 runtime packaged from the release tag, installed/downloaded smoke green.
5. Crate 0.8.1 packaged, dry-run green, contents reviewed; registry consumer proved iff published.
6. R2 gate suite plus external hostile proof green on shipped artifacts.
7. Provenance/manifest/hash record complete for every artifact.

## Required verification

1. `pwsh -NoProfile -File scripts/run-rust-tests.ps1 -Release`
2. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked --release -- --test-threads=1`
3. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --locked -- --ignored`
4. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --locked -- --include-ignored`
5. `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`
6. `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked`
7. `cargo package --manifest-path tethers-0.1/host-rust/Cargo.toml --locked`
8. `cargo publish --manifest-path tethers-0.1/host-rust/Cargo.toml --dry-run --locked`
9. R2 suite `r2_authority_gate.rs` + `examples/external-consumer/smoke.py` against packaged runtimes
10. `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`
11. `git diff --check`; Linux package/lifecycle/dependency proofs; Ubuntu CI lane green

## Formatting and checkpoint sequence

Rust-changing: run the packet's Cargo formatter command before the implementation
checkpoint; stop if rustfmt touches files outside authorised Rust paths.
Non-Rust/evidence-only: `cargo fmt --all -- --check` only, no Rust source mutation.

## Completion and publication

Freeze RC, rerun all gates against that exact SHA, merge after green evidence, tag
final `main` as `v0.8.1`, build artifacts from the tag, verify hashes/provenance,
smoke downloaded artifacts, upload GitHub release, publish crate only with
credentials and release authority. Mark `COMPLETE` only with exact-tag evidence and
a `control-v1/COMPLETE` checker pass.

## Forbidden changes

- No Core/language/protocol/authority/replay/provider/execution semantic changes;
  no `tethers.authority/2`; no new policy engine.
- No macOS/ARM ports; no OCaml→Rust rewrite; no half-tested crate publication.
- No PR #45 wholesale merge; no 0.7.1 architecture revival.
- No global release debug-assertions; no unexplained ignored tests; no weakened tests.
- No force-push, rebase, or direct `main` update; normal branch push only.
- No cleaning, resetting, or absorbing the legacy Goose checkout
  (`D:\The Next Thing\Tethers Lang - Goose Integration`).

## Stop conditions

- Conflicting server-name/authority derivation between check path and run path.
- A required Linux gate cannot be satisfied (report; do not declare Linux official).
- Missing crates.io credentials/authority at the final step (PUBLICATION BLOCKED, rest stays green).
- Discovered production defect; unavailable required verification; contradictory frozen architecture.
- After two materially similar failed attempts on one underlying problem: stop with
  exact evidence and the smallest unresolved question.

## Expected pre-existing changes

None. Canonical checkout was clean at the Base commit before branching.
