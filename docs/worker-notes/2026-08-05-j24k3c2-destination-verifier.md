# Worker Note

Task: `J24K3c2 - Exact recovery destination verifier`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `READY`
Base commit: `WORKTREE`
Implementation checkpoint: `WORKTREE`

## Requested outcome

Add one crate-private, read-only verifier that proves the exact final destination named by a validated publication intent matches the intent's precomputed installed record: exact file set, lengths, hashes, read-only permissions, and path/reparse safety. Also ensure already-opened registry roots still exist as ordinary safe directories before J24K3c1 observation or J24K3c2 verification proceeds.

## Changes made

None yet.

## Decisions and assumptions

- Kimi K2.7Code is the selected implementation model for this bounded repository-reading and Rust verification package.
- This package verifies destination bytes and filesystem shape only.
- Current exact-candidate trust, launch profile, conformance, approval-chain freshness, global installed-root audit, recovery mutation, and executor wiring remain later packages.
- Existing public installation behaviour remains unchanged.

## Evidence

Not run yet.

## Discoveries

None yet.

## Remaining risks

Filesystem verification must fail closed without leaking paths, OS errors, or package-controlled text. It must not mutate the destination or infer absence from `Path::exists()`.

## Smallest next action

Read the task packet and accepted storage code, implement the exact destination verifier and existing-root guard, add direct tests, run the complete verification packet, and return the branch for independent review.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/installation_recovery.rs`
- `tethers-0.1/host-rust/src/m3_store.rs`
