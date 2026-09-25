# TETHERS 0.8.1 — RECOVERY + THREE-PLATFORM TRUST CLOSEOUT

Control contract: `1`

Status: `IN_PROGRESS`

Task colour: `Amber`

Owner: `Tethers 0.8.1 Maintenance Agent`

Route: `OpenCode implementation on canonical Windows checkout; Linux x86-64 & macOS (ARM64 + Intel) proof via GitHub Actions CI; gated tag/crates.io final steps`

Worker note: `docs/worker-notes/2026-09-25-tethers-0.8.1-maintenance.md`

Base branch: `main`

Base commit: `c5bd371ff348879313186406696457f7024f4446`

The Base commit is `v0.8.0` (Merge pull request #48). All 0.8.1 work builds forward from it.

Supersession note: this packet supersedes the stale `IN_PROGRESS` Codex 0.8.0-readiness
packet previously in this file (base `7e29110`). That 0.8.0 objective is factually
complete on `main` (`v0.8.0` tag resolves to the Base commit above); its packet state
was never flipped to `COMPLETE`/`ACCEPTED`. No Codex work is overwritten: the prior
packet text remains in Git history. Full acceptance contract is the supplied
`TETHERS 0.8.1 — RECOVERY + THREE-PLATFORM TRUST CLOSEOUT` packet; this file is
its condensed control record, not a second competing specification.

Rust toolchain: `1.97.1`; plain Cargo resolved by root pin; `--locked` mandatory

Toolchain preflight: `required`

## Objective

Finish Tethers 0.8.1 as a genuinely reusable and trusted runtime across three target
operating systems (Windows x86-64, Linux x86-64, macOS ARM64 official; plus macOS
Intel x86-64 compatibility lane). Recover stopped-agent work from `D:\tethers-h2-gate`,
incorporate hands-on audit addendum repairs (Items 1-11), provide native packaging and
CI proof across platforms, and preserve bit-for-bit `tethers.authority/1` guarantees
without architecture redesign or feature bloat.

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
4. macOS ARM64 (official) and macOS x86-64 (Intel compatibility lane) runtimes built,
   packaged version-driven, clean-package smoked (permissions, `otool -L` audit,
   architecture validation, provider lifecycle, and external consumer authority smoke).
5. Hands-on audit addendum items 1-11 implemented and regression-tested (Git log SHA,
   describe reconciliation, smoke.py packaging, Git CLI aliases, Replay trail collision,
   Replay diagnostics, runtime-config ambiguity, provision-replay machine output).
6. Windows x86-64 regression proof preserved; packager generalised off hard-coded 0.8.0.
7. `cargo package --locked` and `cargo publish --dry-run --locked` pass; package
   contents inspected; version bumped to 0.8.1 at release-candidate stage only.
8. Actual crates.io publication and macOS Developer ID signing/notarization treated
   as gated final steps requiring external credentials; otherwise reported truthfully
   as BLOCKED without fabricating trust.
9. Release provenance (source SHA/tree hash, tool versions, artifact hashes, manifests)
   for every artifact; `VERSION = 0.8.1`, release notes, README/support-matrix updates
   from evidence only.
10. R2 authority behaviour bit-for-bit preserved; no `tethers.authority/2`.

## Relevant components

- `tethers-0.1/host-rust/` (Cargo.toml, src, tests incl. `r2_authority_gate.rs`, `j13a_cli.rs`, `child_process.rs`)
- `tethers-0.1/engine-ocaml/`
- `scripts/run-rust-tests.ps1`, `scripts/run-rust-tests.sh`
- `scripts/package-tethers-0.8-release.ps1`, `scripts/package-linux-release.sh`, `scripts/package-macos-release.sh`
- `scripts/test-linux-package.sh`, `scripts/test-linux-package-full.sh`
- `scripts/test-macos-package.sh`, `scripts/test-macos-package-full.sh`
- `scripts/prepare-current-engine.sh`, `scripts/verify-tethers.py`
- `scripts/check-dev-tools.ps1`
- `.github/workflows/tethers-verification.yml`, `.github/workflows/tethers-linux-package.yml`, `.github/workflows/tethers-macos-package.yml`
- `examples/external-consumer/`, `examples/rust-crate-consumer/`
- `docs/INTEGRATING_TETHERS.md`, `docs/SUPPORT_MATRIX.md`, `VERSION`, `justfile`

## Frozen decisions and invariants

- A Plan is not permission; approval is not execution; COMMIT records durable intent;
  the external Host alone performs effects; outcomes stay truthful; forged/malformed
  authority input fails closed.
- Do not enable release `debug-assertions` globally; do not merge PR #45 wholesale;
  do not resurrect 0.7.1 assumptions; do not weaken tests for cross-platform green;
  do not claim crates.io publication on dry-run alone; do not claim macOS Developer
  ID notarization when credentials are not configured (report BLOCKED); no proof
  laundering across SHAs.
- Target platforms: Windows x86-64, Linux x86-64, macOS ARM64 (official) + macOS Intel x86-64 (compat).

## Acceptance criteria

1. `cargo test --release` (genuine profile, `--all-targets --all-features --locked`) compiles and passes.
2. Ignored-test disposition table complete; `ignored-only` and `include-ignored` runs reported.
3. Linux x64 runtime packaged from the release tag, clean-package authority smoke green.
4. macOS ARM64 runtime and macOS Intel runtime packaged, clean-package authority smoke green.
5. Windows x64 runtime packaged from the release tag, installed/downloaded smoke green.
6. Crate 0.8.1 packaged, dry-run green, contents reviewed; registry consumer proved iff published.
7. R2 gate suite plus external hostile proof green on shipped artifacts.
8. Hands-on audit items 1-11 verified with regression test coverage.
9. Provenance/manifest/hash record complete for every artifact; `VERSION`, release
   notes, README, and support-matrix state updated from evidence only.
10. R2 authority behaviour bit-for-bit preserved with no `tethers.authority/2`;
    the shipped diff is limited to the authorised maintenance surface.

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
- No OCaml→Rust rewrite; no half-tested crate publication.
- No PR #45 wholesale merge; no 0.7.1 architecture revival.
- No global release debug-assertions; no unexplained ignored tests; no weakened tests.
- No force-push, rebase, or direct `main` update; normal branch push only.
- No cleaning, resetting, or absorbing the legacy Goose checkout
  (`D:\The Next Thing\Tethers Lang - Goose Integration`).

## Stop conditions

- Conflicting server-name/authority derivation between check path and run path.
- A required platform gate cannot be satisfied (report; do not declare official).
- Missing crates.io credentials/authority at the final step (PUBLICATION BLOCKED, rest stays green).
- Discovered production defect; unavailable required verification; contradictory frozen architecture.
- After two materially similar failed attempts on one underlying problem: stop with
  exact evidence and the smallest unresolved question.

## Expected pre-existing changes

None. Canonical checkout was clean at the Base commit before branching.
