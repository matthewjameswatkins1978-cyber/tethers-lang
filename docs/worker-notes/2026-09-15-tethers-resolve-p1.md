# Worker Note

Task: `TETHERS x RESOLVE01 / P1`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `66e8360a7247b60ccc8fb6cb447f796b922afb42`

Implementation checkpoint: `5235b04d0da08aa0b90f0a0519869e17025f3496`

## Requested outcome

Implement the Rust-host preparation-proof and opaque ScopeKey substrate frozen
by P0 without adding Resolve transport, admission, execution, replay, Core, or
syntax semantics.

## Changes made

- Added `resolve_guard.rs` and exported it from the Rust host library.
- Added controlled `GuardPreparationProof`, typed proof digest,
  `ResolveGuardRequired`, fresh reconstruction, exact comparison, and bounded
  error/mismatch types.
- Added runtime-owned resolved scope projection reusing the existing scope
  assessment extraction and boundary checks.
- Added deterministic versioned PathPrefix/Unrestricted projections and
  sorted/deduplicated opaque ScopeKeys.
- Added the P1 architecture note and replaced the current task packet with the
  scoped P1 packet.
- Added focused unit tests and a prepared-runtime zero-provider-invocation
  test.
- Independent review identified a missing preparation-boundary input-schema
  check and evidence wording that could be read as stronger than the counting
  test proved. The route now revalidates action arguments against the resolved
  manifest schema; the test documents compile-time executor separation and
  checks the counter remains zero for valid, invalid, matching, and
  reconstruction paths.

## Decisions and assumptions

The existing `approval::digest` is reused for JCS canonicalisation and
SHA-256. Binding evidence commits to existing verified manifest/provider/MCP
binding facts, configured scope binding, and the prepared provider launch
configuration, while exposing none of those raw values in the Resolve-facing
projection. A pre-existing `PermissionDecision::Allow` is required; P1 does
not create policy or approval authority.

## Evidence

- Accepted starting baseline: `66e8360a7247b60ccc8fb6cb447f796b922afb42`.
- Implementation checkpoint before review fix: `54cad2d49115dac42d2b87cf0225c1f3c2e165b8`.
- Task packet checker passes in `COMPLETE` state at `5235b04`.
- `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml` passes.
- `resolve_guard::tests`: 5 passed.
- Prepared-runtime P1 zero-provider/reconstruction test: 1 passed.
- `just verify` passes: engine provenance, OCaml, Rust/static, warning ratchet,
  cross-language, protocol fixtures, MCP transcripts, and compatibility corpus.
- Cargo formatter was run and its immediate diff was limited to the authorised
  Rust files; `git diff --check` passes.
- The explicit all-target Clippy probe remains non-green because of broad
  pre-existing repository lint debt; no P1-file diagnostic was reported and
  the repository-authoritative `just check` and warning ratchet pass.

## Discoveries

The existing public scope assessor returned only a three-state enum, so the
smallest non-duplicating integration is an internal resolved-scope projection
shared by the assessor and P1. Existing provider capability preparation is
already immutable and provider-free, which permits the P1 preparation route to
remain a pure host computation.

## Remaining risks

The repository's existing duplicate-target warning remains pre-existing debt.

## Smallest next action

Publish the accepted checkpoint through the normal branch/PR route, inspect the
actual PR, merge when green, and confirm the resulting main SHA.

## References

- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P0_SEAM_AUDIT.md`
- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P1_PREPARATION_PROOF.md`
- `docs/CONSTITUTION.md`
- `docs/CAPABILITY_BRIDGE.md`
- `docs/PROJECT_CONTROL.md`
- `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md`
