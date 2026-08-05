# Worker Note

Task: `J24K3c3 correction - exact trust equality and evidence hygiene`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `7b148070a04a3af30ffe0165c35ea56e499b36a5`
Implementation checkpoint: `099149d84d5e62c1f65268e05d534c6c832d3a83`

## Requested outcome

Correct three narrow independent-review findings in the otherwise complete J24K3c3 recovery evidence-chain revalidator:

1. enforce literal full `PackageTrustEvidence` equality at every recovery chain boundary rather than comparing only `evidence_digest`;
2. ensure candidate unsafe-path translation and current-suite computation cannot expose a lower-layer or non-recovery-facing error;
3. make the successful read-only regression actually prove entry bytes, modification timestamps, and permissions remain unchanged, while renaming two closed-enum tests so their names describe what they prove.

Do not redesign J24K3c3 or add later recovery work.

## Changes made

- `tethers-0.1/host-rust/src/installation_recovery_evidence.rs`:
  - `revalidate_trust`: changed from comparing only `evidence_digest` to full `PackageTrustEvidence` equality against `intent.installed_record.trust_evidence`.
  - `map_candidate_error`: replaced `error.message` copy with a fixed recovery-owned message `"candidate location is unsafe"` for the `unsafe_destination` branch.
  - `current_suite_digest()` call in `revalidate_installation_recovery_evidence` now uses `map_err(|_| evidence_stale())` to prevent lower-layer error leakage.

- `tethers-0.1/host-rust/src/installed.rs`:
  - `InstallationApprovalRecord::require_for_recovery`: changed `self.trust_evidence.evidence_digest != trust.evidence_digest` to full `self.trust_evidence != *trust`.
  - `InstalledPlugRecord::require_for_recovery`: changed `self.trust_evidence.evidence_digest != trust.evidence_digest` to full `self.trust_evidence != *trust`.

- `tethers-0.1/host-rust/src/installation_recovery_evidence_tests.rs`:
  - Added `SnapshotEntry` enum (Directory / File with hash, modified timestamp, read-only flag).
  - Changed `tree_snapshot` to capture filesystem metadata alongside SHA-256 hashes.
  - Expanded `j24k3c3_success_leaves_stores_quarantine_and_permissions_unchanged` to snapshot all six evidence roots (candidates, quarantine, exact trust, launch profiles, conformance, approvals) before and after revalidation, comparing exact entry sets, bytes/hashes, modification timestamps, and read-only permissions.
  - Renamed `j24k3c3_non_exact_trust_scope_fails_stale` to `j24k3c3_exact_candidate_trust_scope_passes_validation`.
  - Renamed `j24k3c3_non_disabled_target_fails_stale` to `j24k3c3_disabled_target_state_passes_validation`.

- `docs/CURRENT_CLINE_TASK.md`: added missing required sections (`Relevant background and existing behaviour`, `Relevant components`, `Frozen decisions and invariants`); updated status to `COMPLETE`.

Only the five permitted files were changed; no dependency, Cargo.lock, public API, or unrelated module edits.

## Decisions and assumptions

- The accepted J24K3c3 architecture and public types remain unchanged.
- SHA-256 pins remain required, but the frozen packet additionally requires literal trust-object equality.
- `InstallationTrustScope` and `InstallationTargetState` currently expose only their accepted variants; runtime tests cannot construct invalid variants without changing accepted types or using unsafe representation tricks.

## Evidence

- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check` passed.
- `cargo nextest run --config-file .config/nextest.toml --manifest-path tethers-0.1/host-rust/Cargo.toml --all-features --locked -E 'test(j24k3c3)'` passed: 44/44, zero retries.
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --lib j24k3c3 --locked` passed: 44/44.
- Regression suites passed: J24K3c2 (21/21), J24K3c1 (20/20), J24K3b (16/16), J24K3a (25/25), J24K2 (26/26), J24I (30/30), J24H (19/19), J24J (24/24), M3 lifecycle (13/13).
- Full `$env:PATH = "$PSHOME;$env:PATH"; just verify` passed: 1092 unit tests (zero failures) plus all integration tests.
- `Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256` returned `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB`.
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` passed with status `COMPLETE`.
- `git diff --check` reported only expected LF-to-CRLF conversion warnings; no trailing-whitespace or whitespace errors.
- No `Cargo.lock` change; only permitted files were modified.

## Discoveries

- The three trust-boundary digest-only comparisons were the only places `PackageTrustEvidence` was not already compared by full equality; all other evidence comparisons already used full value equality.
- The `m3_lifecycle` Windows handle-contention intermittent failure did not occur during this correction run; all 13 lifecycle tests passed on first attempt under parallel execution.
- The `SnapshotEntry` enum with `modified_unix_ms` and `readonly` fields was straightforward to add and the existing `tree_snapshot` pattern required minimal restructuring.
- The existing `j24k3c3_approval_trust_drift_fails_stale` and `j24k3c3_reconstructed_trust_must_equal_intent_trust_evidence` tests continue to prove the full-equality correction because they tamper with fields inside `PackageTrustEvidence`, making the entire object unequal.

## Remaining risks

- The `m3_lifecycle` intermittent handle-contention failure may still occasionally fail parallel runs; it is unrelated to this change and serial rerun is the accepted mitigation.
- Lower-layer error messages are mapped to the three stable recovery-facing codes, but individual helpers must continue to use only those mapped error families for any new paths.
- The closed enum limitation (`InstallationTrustScope` and `InstallationTargetState` with single accepted variants) means tests can only prove the valid-chain path passes; invalid-variant construction would require unsafe representation tricks that are forbidden.

## Smallest next action

Hand off to Lucy for independent review and routine safe merge. No further implementation work is required.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `tethers-0.1/host-rust/src/installation_recovery_evidence.rs`
- `tethers-0.1/host-rust/src/installation_recovery_evidence_tests.rs`
- `tethers-0.1/host-rust/src/installed.rs`
