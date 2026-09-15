# Worker Note

Task: `TETHERS x RESOLVE01 / P1`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `IN_PROGRESS`

Base commit: `66e8360a7247b60ccc8fb6cb447f796b922afb42`

Implementation checkpoint: `WORKTREE`

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

## Decisions and assumptions

The existing `approval::digest` is reused for JCS canonicalisation and
SHA-256. Binding evidence commits to existing verified manifest/provider/MCP
binding facts, configured scope binding, and the prepared provider launch
configuration, while exposing none of those raw values in the Resolve-facing
projection. A pre-existing `PermissionDecision::Allow` is required; P1 does
not create policy or approval authority.

## Evidence

- Accepted starting baseline: `66e8360a7247b60ccc8fb6cb447f796b922afb42`.
- Task packet checker passes in `IN_PROGRESS` state.
- `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml` passes.
- `resolve_guard::tests`: 5 passed.
- Prepared-runtime P1 zero-provider/reconstruction test: 1 passed.
- Cargo formatter was run and its immediate diff is limited to the authorised
  Rust files.

## Discoveries

The existing public scope assessor returned only a three-state enum, so the
smallest non-duplicating integration is an internal resolved-scope projection
shared by the assessor and P1. Existing provider capability preparation is
already immutable and provider-free, which permits the P1 preparation route to
remain a pure host computation.

## Remaining risks

Full repository verification, final diff review, independent review, commit,
push, PR, merge, and post-merge canonical-main confirmation remain. The
repository's existing duplicate-target warning remains pre-existing debt.

## Smallest next action

Run the complete Rust and repository verification route, inspect the full diff,
then update this note and the task packet with the committed checkpoint before
publication.

## References

- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P0_SEAM_AUDIT.md`
- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P1_PREPARATION_PROOF.md`
- `docs/CONSTITUTION.md`
- `docs/CAPABILITY_BRIDGE.md`
- `docs/PROJECT_CONTROL.md`
- `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md`
