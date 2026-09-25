# Tethers 0.8.1

Status: maintenance, Linux, and distribution closeout (release candidate)

Tethers 0.8.1 takes the released 0.8.0 baseline and closes the remaining named
engineering and distribution gaps. It is not an architecture redesign and not a
feature release. Authority behaviour is unchanged: `tethers.authority/1`
remains the frozen external contract, Core still plans, the Gate still checks
authority and records durable intent, and the external Host still owns physical
effects and truthful outcome reporting.

## What changed since 0.8.0

- **Release-profile tests genuinely pass.** Test-harness helpers that vanished
  under `cfg(debug_assertions)` are now test-scoped (`cfg(any(test,
  debug_assertions))`); production release semantics are untouched and no
  global `debug-assertions` override was introduced. The CLI integration helper
  resolves the host binary for the active Cargo profile instead of hard-coding
  `target/debug`.
- **Ignored tests accounted for.** Nine ignored tests, zero unexplained: two
  Unix-only symlink regressions (platform-gated, run on Linux), two live-service
  Resolve tests (require the accepted Resolve service/Firestore emulator), and
  five provider-dependent tests (p3/p6) with deterministic justfile invocations,
  proven passing against locally built providers.
- **Linux x86-64 full runtime.** Native build, version-driven packaging, and
  clean-package proof ported from PR #45 donor work onto the 0.8 authority
  architecture (PR #45 itself was never merged). Official status follows the
  support-matrix gates and CI evidence, not compilation alone.
- **Windows x86-64 proof preserved.** Same suites, same gates, plus a generalised
  release packager driven by `VERSION` and the exact release tag.
- **Crate quality preserved.** `tethers-reference-host 0.8.1`, MIT OR Apache-2.0,
  locked packaging and publish dry-run green, contents reviewed.
- **Environment checker honesty.** `scripts/check-dev-tools.ps1` now
  distinguishes REQUIRED (`gh`, `git`, `pwsh`) from OPTIONAL convenience tools
  instead of failing the build contract on missing workshop preferences.

## Integration routes

- **Rust source/crate:** `tethers-reference-host` 0.8.1 is available as a
  locked Cargo package under MIT OR Apache-2.0. Crates.io publication happens
  only as the final gated step; until then use the documented `authority_v1`
  module pinned to the `v0.8.1` source tag.
- **Runtime/binary:** Windows x64 and (once accepted) Linux x64 bundles each
  contain the `tethers` host and matching OCaml Core engine. A consumer starts
  the host process and uses the versioned JSON protocol. The OCaml engine
  remains executable-only.

See [integration guide](INTEGRATING_TETHERS.md) and [crate README](../tethers-0.1/host-rust/README.md).

## Compatibility and platform

No 0.1 syntax, `tethers.authority/1` shape, permission, replay, Trail, or
provider semantic changes. The R2 authority gate suite passes unchanged on the
shipped artifacts, including forged-authority, malformed-frame, unsupported
protocol, replay-confusion, and no-effect-by-the-Gate resistance.

Windows x86-64 remains an official full-runtime target. Linux x86-64 becomes
official only when every support-matrix gate has passing evidence; see
[SUPPORT_MATRIX.md](SUPPORT_MATRIX.md) for the current verdict.

## Provenance

The GitHub release assets include a provenance manifest (`tethers.release/1`),
SHA-256 files, and a verification report. They identify the exact tag commit,
source tree, platform, compiler versions, runtime hashes, installation path,
and proof results. The report is produced only after the release artifacts have
been built from the tag and tested. The crate record carries its package
version, source commit, `.crate` hash, and registry identity when published.
