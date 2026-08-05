# Worker Note

Task: `J24K3c3 correction - exact trust equality and evidence hygiene`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `READY`
Base commit: `WORKTREE`
Implementation checkpoint: `WORKTREE`

## Requested outcome

Correct three narrow independent-review findings in the otherwise complete J24K3c3 recovery evidence-chain revalidator:

1. enforce literal full `PackageTrustEvidence` equality at every recovery chain boundary rather than comparing only `evidence_digest`;
2. ensure candidate unsafe-path translation and current-suite computation cannot expose a lower-layer or non-recovery-facing error;
3. make the successful read-only regression actually prove entry bytes, modification timestamps, and permissions remain unchanged, while renaming two closed-enum tests so their names describe what they prove.

Do not redesign J24K3c3 or add later recovery work.

## Changes made

None yet.

## Decisions and assumptions

- The accepted J24K3c3 architecture and public types remain unchanged.
- SHA-256 pins remain required, but the frozen packet additionally requires literal trust-object equality.
- `InstallationTrustScope` and `InstallationTargetState` currently expose only their accepted variants; runtime tests cannot construct invalid variants without changing accepted types or using unsafe representation tricks.

## Evidence

Not run yet.

## Discoveries

Independent review found:

- `revalidate_trust`, `InstallationApprovalRecord::require_for_recovery`, and `InstalledPlugRecord::require_for_recovery` compare only `PackageTrustEvidence.evidence_digest`, despite the frozen packet requiring exact object equality;
- `map_candidate_error` copies the candidate layer's message into a new `unsafe_store_path`, contrary to the no-lower-layer-message rule;
- `current_suite_digest()?` can theoretically return a non-recovery-facing error without translation;
- `j24k3c3_success_leaves_stores_quarantine_and_permissions_unchanged` compares logical values and byte hashes but does not compare modification timestamps or permission bits;
- two test names claim invalid enum states fail even though the closed enums permit only the accepted variants and the test bodies simply prove the valid chain passes.

## Remaining risks

The correction must remain read-only and must not touch destination verification, global audit, classification, mutation, publication, intent removal, locking, planner, or executor wiring.

## Smallest next action

Apply only the correction packet, rerun the complete J24K3c3 verification matrix, and return the branch for independent review.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `tethers-0.1/host-rust/src/installation_recovery_evidence.rs`
- `tethers-0.1/host-rust/src/installation_recovery_evidence_tests.rs`
- `tethers-0.1/host-rust/src/installed.rs`
