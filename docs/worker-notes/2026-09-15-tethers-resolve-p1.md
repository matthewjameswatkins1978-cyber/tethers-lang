# Worker Note

Task: `TETHERS x RESOLVE01 / P1`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

This note contains the original P1 acceptance record followed by the remediation
closeout record below.

Base commit: `7066c9604e7062bf46529b9509dc92287e742379`

Implementation checkpoint: `ffdf368ac785e9b8a57a147200ffb080d9d1555f`

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
- Independent review identified missing freshness checks at the preparation
  boundary. The route now re-resolves the exact pinned capability through the
  current provider-availability snapshot, re-evaluates effective policy, and
  revalidates action arguments against the current manifest schema. The test
  also checks unavailable-provider reconstruction fails closed while the
  preparation route has no provider-process boundary.

## Decisions and assumptions

The existing `approval::digest` is reused for JCS canonicalisation and
SHA-256. Binding evidence commits to existing verified manifest/provider/MCP
binding facts, configured scope binding, and the prepared provider launch
  configuration, while exposing none of those raw values in the Resolve-facing
  projection. P1 does not create or consume policy/approval authority. The
  preparation route rechecks current resolution and effective policy itself;
  when the result is `Ask`, it requires the authoritative `ApprovalStore` to
  contain an exact fresh approval for the current action.

## Evidence

- Accepted starting baseline: `66e8360a7247b60ccc8fb6cb447f796b922afb42`.
- Implementation checkpoint before review fix: `54cad2d49115dac42d2b87cf0225c1f3c2e165b8`.
- Task packet checker passes in `COMPLETE` state after the remediation closeout.
- `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml --locked` passes.
- `resolve_guard::tests`: 5 passed; prepared-runtime remediation tests: 2 passed.
- `just verify` passes from the remediation checkout with a current engine built
  from that checkout: OCaml, Rust/static, warning ratchet, cross-language,
  protocol fixtures, MCP transcripts, and compatibility corpus.
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

## P1 remediation review and closeout

The accepted P1 review identified two authority-boundary defects and one
ineffective test. This remediation is based on `7066c9604e7062bf46529b9509dc92287e742379`
on branch `codex/tethers-resolve-p1-remediation`; the implementation checkpoint
is `ffdf368ac785e9b8a57a147200ffb080d9d1555f`.

The preparation route now derives the current pinned capability and effective
policy decision internally. It accepts no caller-supplied
`ResolvedCapability` or `PermissionDecision`. When current policy is `Ask`, it
requires the existing `ApprovalStore` record to be `Approved` and to exactly
match a fresh `ApprovalProof` derived from the current action. A missing,
pending, stale, or mismatched approval is refused.

Fresh reconstruction now compares the rebuilt proof against the previous proof
before returning. Material drift is returned as typed `EvidenceMismatch`; a
caller cannot accidentally treat unequal fresh evidence as successful resume.

The prior unused `CountingExecutor` test double was removed. The focused
prepared-runtime test now exercises the actual preparation and reconstruction
routes with the configured provider launch script absent, proves matching
reconstruction and typed argument drift refusal, and verifies that no provider
process is required by either route. The production API has no executor or
provider-session input.

Focused remediation evidence so far:

- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check` — PASS.
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --locked resolve_guard::tests` — 5 passed.
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --locked configured_runtime::tests::p1_preparation` — 2 passed.
- `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml --locked` — PASS.
- task packet checker — PASS for the remediation packet while it was in
  `IN_PROGRESS` state.

Final remediation verification:

- Task packet checker — PASS (`COMPLETE` state).
- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check` — PASS.
- `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml --locked` — PASS.
- `just verify` — PASS; source commit `b4d44f8515094dddabfe228bd39c8f5f944ea02b`,
  current engine SHA-256 `b29ee1da37b063914e9b4b3218a3e5ab370af901a1661830b29725d6b8fc03e1`,
  10 PASS, 0 FAIL, 0 skipped, 0 not applicable.
- The full Rust/cross-language suite inside `just verify` passed. A prior
  standalone run before current-engine preparation had 22 dependent failures;
  each stopped at the explicit missing-engine prerequisite and was not treated
  as a product regression.
- `git diff --check` — PASS. No OCaml/Core, syntax, Plan, provider execution,
  replay, outcome, Resolve transport, or Resolve database changes are in scope.

The remaining publication evidence is recorded in the final closeout response
after branch, PR, and main integration checks.

## Independent review

Gemini independently reviewed the redacted remediation contract using the
official `gemini-3.8-flash` model at `high` thinking level; the completed
response condition was verified. It conditionally approved the remediation and
confirmed that internal capability/policy derivation, exact approval matching,
internal reconstruction comparison, and the executor-free preparation API
address the four reported findings.

The review requested source confirmation that proof/result fields remain
private and questioned the older public `evaluate_permission_resolved` helper.
Source inspection confirms `GuardPreparationProof`, `PreparedResolveGuard`, and
their nested evidence fields have no public unchecked constructor or public
fields. The older helper remains for existing non-P1 host paths, but its result
is no longer accepted by either P1 preparation or reconstruction; P1 always
calls `evaluate_effective_policy` internally. This is a bounded adjacent API
debt, not a P1 preparation bypass, and changing it would expand into policy
redesign outside this remediation.
