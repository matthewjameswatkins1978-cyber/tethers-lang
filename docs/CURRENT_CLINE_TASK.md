# Tethers 0.8-C — Linux Native Parity

Task: `Tethers 0.8-C / R3`

Control contract: `1`

Status: `BLOCKED`

Task colour: `Red`

Owner: `Codex`

Route: `Implement and verify native GNU/Linux x86-64 parity from the fetched 0.8-B baseline, preserving Windows behaviour and stopping short of any unsupported release claim.`

Base commit: `ddc5d0fccfc0a38ff0af3013c5e9ff6ba357e0b2`

Worker note: `docs/worker-notes/2026-09-14-tethers-0-8-c.md`

Suggested branch:

`codex/tethers-0.8-linux-native-parity`

## Objective

Make native GNU/Linux x86-64 a reproducible, reviewable Tethers target across source, build, Core, host, provider supervision, filesystem safety, verification, packaging, clean installation, and CI. Keep support claims provisional until all acceptance gates have evidence.

## Relevant background and existing behaviour

The fetched 0.8-B baseline is `ddc5d0fccfc0a38ff0af3013c5e9ff6ba357e0b2`. The repository previously compiled the Rust host on non-Windows systems but returned Windows-only persistence errors, used Windows-only engine paths and fixtures, and had no native Linux package or CI acceptance route. WSL evidence is supplemental and cannot establish native Linux support.

## Required behaviour

1. Build the Rust host and OCaml Core natively for `x86_64-unknown-linux-gnu`, with explicit glibc/runtime evidence and no source-level semantic divergence.
2. Provide Linux replay persistence with the existing claim/generation model, canonical bytes, durable publication, exclusive logical-key locking, symlink rejection, and fail-closed recovery.
3. Supervise Linux provider descendants with process-group ownership, parent-death handling, shell-free argv execution, bounded pipes, and truthful cleanup evidence.
4. Port host/provider installation locks, launch environments, engine discovery, verification scripts, fixtures, and machine reports without weakening Windows behaviour.
5. Add a native Linux x86-64 package containing host, engine, provenance, checksums, required documentation, and a clean-install smoke test outside the source tree.
6. Add a GitHub-hosted Ubuntu verification lane covering format, check, Core build/tests, Rust/cross-language verification, compatibility, package creation, and clean-package smoke.
7. Record exact evidence, risks, and any incomplete gates; do not mark Linux official or release-eligible without all required evidence.

## Relevant components

- `tethers-0.1/host-rust/src/`
- `scripts/`
- `tethers-0.1/scripts/`
- `.github/scripts/`
- `.github/workflows/`
- `justfile`
- `docs/LINUX_PORTABILITY_AUDIT.md`
- `docs/VERIFICATION.md`
- `docs/SUPPORT_MATRIX.md`
- `docs/ROAD_TO_1_0.md`
- `docs/SECURITY.md`
- `docs/worker-notes/`

## Frozen decisions and invariants

- No new Tethers syntax, Core semantic change, Plan meaning change, Trail meaning change, Plug trust-boundary relaxation, MCP modernization, or Agent Surface feature.
- Windows remains a supported target and its existing native safety and tests must not be weakened.
- Linux baseline is native Ubuntu-class GNU/Linux x86-64 with glibc; WSL is supplemental evidence only.
- A Plan remains a proposal, not permission; Core remains application-agnostic; providers remain host-supervised and host-authorised.
- Semantic corpus output must be byte/meaning equivalent across platforms apart from explicitly platform-labelled diagnostics and paths.
- No automatic retry, shell interpolation, ambient provider environment, symlink traversal, overwrite publication, or unsupported isolation claim.

## Acceptance criteria

1. Native Linux Rust and OCaml builds succeed from this source tree, with exact toolchain and runtime evidence.
2. Linux replay lifecycle, restart/recovery, lock exclusion, malformed-state rejection, symlink rejection, and durable publication tests pass.
3. Linux child process and descendant shutdown tests demonstrate group cleanup and reaping; the audit states the exact guarantee and limitation.
4. Linux verification scripts and `just verify` produce a truthful machine report with no path or extension assumptions that fail natively.
5. A Linux x86-64 tarball and checksum are built from source and pass clean extraction smoke outside the checkout.
6. The Ubuntu CI job is reviewable and runs the required native checks without WSL, containers, or product-only claims.
7. Windows formatting/check/tests and repository safety checks remain passing, or any environment-only prerequisite failure is reported precisely.
8. The worker note and this packet contain exact SHAs, commands, results, unresolved risks, and a truthful final status.

## Required verification

Run the repository tool diagnostic and packet checker; scoped Rust formatting; Rust check/tests; Linux native check/tests; OCaml build/tests where the native switch is available; MCP transcripts; compatibility corpus; warning policy; `just verify`; package checksum and clean-install smoke; `git diff --check`; complete diff/status review; and final remote branch identity checks for a complete task. Do not substitute WSL for the native CI gate.

## Forbidden changes

- No new language syntax, Core semantics, protocol redesign, MCP HTTP/OAuth work, mobile/macOS/ARM support, final release automation, SBOM/attestation, package-manager integration, container product, or unrelated cleanup.
- No force push, history rewrite, direct main update, merge, tag, or release claim without the packet’s completed evidence and required review.
- No weakening, deletion, or platform-skipping of Windows safety tests.

## Stop conditions

Stop and report before claiming completion for any semantic mismatch, changed OS-independent digest, unproven descendant supervision, filesystem escape, symlink race that cannot be closed, unverifiable provenance, package dependent on the checkout, failed required native gate, weakened Windows behaviour, new syntax, scope expansion, or missing native CI evidence. After two materially similar failed attempts at the same underlying repair, stop that approach and report the exact first error plus the smallest unresolved question.

## Expected pre-existing changes

None.

## Implementation scope

- `docs/CURRENT_CLINE_TASK.md`
- `docs/LINUX_PORTABILITY_AUDIT.md`
- `tethers-0.1/host-rust/src/replay_linux.rs`
- `tethers-0.1/host-rust/src/replay_store.rs`
- `tethers-0.1/host-rust/src/replay_runtime.rs`
- `tethers-0.1/host-rust/src/child_process.rs`
- `tethers-0.1/host-rust/src/execution_environment.rs`
- `tethers-0.1/host-rust/src/installation_execution.rs`
- `tethers-0.1/host-rust/src/launch_profile.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `tethers-0.1/host-rust/src/application.rs`
- `tethers-0.1/host-rust/src/host_execution.rs`
- `tethers-0.1/host-rust/src/bin/bench_cold.rs`
- `tethers-0.1/host-rust/src/bin/bench_prod.rs`
- `tethers-0.1/host-rust/src/bin/bench_retained.rs`
- `tethers-0.1/host-rust/Cargo.toml`
- `tethers-0.1/host-rust/Cargo.lock`
- `scripts/prepare-current-engine.ps1`
- `scripts/run-rust-tests.ps1`
- `scripts/verify-tethers.ps1`
- `scripts/check-compatibility-corpus.ps1`
- `scripts/package-linux.ps1`
- `scripts/test-linux-package.ps1`
- `tethers-0.1/scripts/check-fixtures.ps1`
- `tethers-0.1/scripts/test-mcp-transcripts.ps1`
- `README.md`
- `QUICKSTART.md`
- `docs/ROAD_TO_1_0.md`
- `docs/SECURITY.md`
- `docs/VERIFICATION.md`
- `justfile`
- `.github/scripts/check-tethers-toolchains.ps1`
- `.github/workflows/tethers-verification.yml`
