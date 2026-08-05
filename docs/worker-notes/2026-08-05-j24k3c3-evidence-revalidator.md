# Worker Note

Task: `J24K3c3 - Exact recovery evidence-chain revalidator`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `READY`
Base commit: `WORKTREE`
Implementation checkpoint: `WORKTREE`

## Requested outcome

Add one crate-private, read-only recovery evidence-chain revalidator. Given the current typed installation request, one validated publication intent, and the existing candidate, exact-trust, launch-profile, conformance, and approval stores, prove that the complete precomputed installed record is still justified by current host-owned evidence.

The package must revalidate the exact request and candidate, current exact-candidate authority, reconstructed package-trust evidence, pinned launch profile, passed conformance against the current suite, complete installation approval chain, and the precomputed installed record.

## Changes made

None yet.

## Decisions and assumptions

- Kimi K2.7Code is the selected implementation model for this second measured repository-reading package.
- This package is read-only and performs no destination verification, global installed-root audit, recovery classification, cleanup, publication, intent removal, lock integration, or executor wiring.
- Existing stores and validation seams remain authoritative; no parallel trust model is permitted.

## Evidence

Not run yet.

## Discoveries

None yet.

## Remaining risks

The recovery revalidator must not accept a structurally valid but incorrectly repinned intent record. It must compare the full chain rather than checking only IDs and top-level digests, and it must map lower-layer failures to stable recovery errors without leaking paths, package-controlled text, or OS diagnostics.

## Smallest next action

Read the task packet and all named authority/store implementations, implement only the exact read-only evidence-chain revalidator and its direct tests, run the complete verification packet, and return the branch for independent review.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `tethers-0.1/host-rust/src/installation_recovery_evidence.rs`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/installation_request.rs`
- `tethers-0.1/host-rust/src/candidate.rs`
- `tethers-0.1/host-rust/src/current_trust.rs`
- `tethers-0.1/host-rust/src/installation_trust.rs`
- `tethers-0.1/host-rust/src/launch_profile.rs`
- `tethers-0.1/host-rust/src/conformance.rs`
- `tethers-0.1/host-rust/src/installed.rs`
