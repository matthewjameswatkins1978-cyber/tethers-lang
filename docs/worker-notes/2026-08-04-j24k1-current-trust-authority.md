# Worker Note

Task: `J24K1 - Explicit current-trust authority foundation`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `db84c71dc92381921cdc05c62029a1899c13d7f2`
Implementation checkpoint: `5559efb432d46637a8da9149e57a1c6604a5c0fa`

## Requested outcome

Introduce the crate-private current-trust authority foundation required by the future locked J24K executor, preserving all accepted publisher and developer trust behaviour while allowing exact-candidate authority to be threaded explicitly through conformance, approval, and installed-publication internals.

## Changes made

- Added crate-private `CurrentTrustAuthority`, `PublisherDeveloperTrustAuthority`, and
  `ExactCandidateTrustAuthority` in `src/current_trust.rs`.
- Added explicit authority-aware launch, conformance, approval, installation-entry, and
  final-installation-revalidation seams. Existing wrappers preserve accepted signatures
  and construct the legacy adapter locally.
- Added crate-private focused tests for matching, stale, absent, wrong-mode, and corrupt
  exact-candidate authority evidence.
- Added crate-test-only `RecordingAuthority` and `FailOnNthAuthority` adapter types and
  six behavioural propagation tests proving that every authority-aware downstream seam
  invokes and propagates the supplied authority.
- Declared the production module privately and the focused test module only under
  `#[cfg(test)]`.

## Decisions and assumptions

- No lock, executor, publication intent, CLI, or multi-step driver is part of J24K1.
- Every authority-aware internal seam requires an explicit authority argument.
- Existing public APIs retain their signatures and use the legacy publisher/developer authority adapter.
- Exact-candidate authority remains crate-private and has no fallback path.
- Exact authority validates candidate and supplied evidence before loading the current
  exact store record, then compares all frozen pins and reconstructed evidence.
- The authority is passed as `&dyn CurrentTrustAuthority`; no optional or implicit
  authority state was introduced.

## Evidence

- J24K1 focused tests: `9 passed, 0 failed` in ordinary Cargo (`9 passed, 931 filtered`)
  and Nextest (`9 run, 931 skipped`, zero retries). 3 original exact-authority tests
  plus 6 new behavioural propagation tests.
- Behavioural tests added:
  1. `j24k1_prepared_launch_revalidate_current_trust_with_uses_supplied_authority`
  2. `j24k1_prepared_launch_launch_for_candidate_with_refuses_before_launch`
  3. `j24k1_run_host_conformance_with_authority_uses_supplied_authority`
  4. `j24k1_approve_with_authority_uses_and_propagates_supplied_authority`
  5. `j24k1_install_disabled_with_authority_uses_supplied_authority_at_entry`
  6. `j24k1_install_disabled_invokes_authority_again_after_staging_before_publication`
     — proves call count 2, sentinel propagation, no published record, staging cleanup.
- Representative regressions: J24I `30 passed`; M3 lifecycle `13 passed`; J23C2
  `8 passed`; J23C3 `1 passed`.
- Full `just verify`: `940 passed, 0 failed`.
- Cargo.lock SHA-256:
  `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB` (unchanged
  throughout both implementation and correction rounds).
- `cargo fmt --all -- --check`: passed. `git diff --check`: passed.
- Task-packet checker final: PASS.

## Discoveries

- The requested J24K1 branch existed on `origin` but was not present in stale local
  refs; it was fetched and checked out without creating a second task branch.
- The packet checker required the full implementation SHA before it could validate
  the completed packet. No checker or frozen architecture file was changed.
- The original implementation's three exact-authority tests proved only the adapter
  itself, not downstream propagation. The correction added `RecordingAuthority` and
  `FailOnNthAuthority` crate-test-only types and six behavioural propagation tests.
- The `FailOnNthAuthority` final-installation-revalidation test required constructing a
  fully valid pipeline (candidate in quarantine, trust, launch, conformance, and
  approval records) to reach the post-staging authority call. Programmatic construction
  using the J24J pattern was sufficient; no actual process launch was required.

## Remaining risks

- Independent Lucy review was required because the packet is Red security-sensitive
  trust refactoring. That review accepted the production routing and the corrected
  behavioural propagation evidence.
- The `RecordingAuthority` and `FailOnNthAuthority` types are crate-test-only and have
  no production footprint.

## Smallest next action

Proceed to the next bounded J24K package after this branch is accepted and merged.

## Final Git Evidence
- Implementation commit: `f82af8b595889a65b2003d425cb0ab18d4f20a7b`.
- Correction commit: `5559efb432d46637a8da9149e57a1c6604a5c0fa` (behavioural
  propagation tests).
- Documentation commits: `273ccaf53090f7e1bb3ed65bc9a8fc392c7cbc6f`,
  `3c04e610cead482e67023ea6b695b833f4319c31`,
  `37bfa96052b8f7afa9725175d849355e23eeb56b`,
  `ac02837654de22d9a31ae4065ce0c071bde14fed`,
  `0d6f2e7fe5364c6c742ad2d18845e0564b86409a`,
  `45048d81bdb1ce9475be13dafaa4dd00f2e27024`,
  `4ede589884dc653c6e859e4908e39b2526161c92`.
- OpenCode correction handoff tip before Lucy's documentation-only acceptance normalisation:
  `8f293321155e90980de09a0d646871ca40f24ebf`.
- Final branch remains `opencode/j24k1-current-trust-authority`.

## References

- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `docs/architecture/J24I_EXACT_CANDIDATE_INSTALLATION_TRUST.md`
- `docs/architecture/J24J_READ_ONLY_INSTALLATION_RECONCILIATION_PLANNER.md`
