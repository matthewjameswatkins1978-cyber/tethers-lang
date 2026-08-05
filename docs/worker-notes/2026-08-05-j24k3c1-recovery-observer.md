# Worker Note

Task: `J24K3c1 - Read-only exact publication-state observer`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `READY`
Base commit: `WORKTREE`
Implementation checkpoint: `WORKTREE`

## Requested outcome

Add one private read-only observer that accepts a validated publication intent and reports whether that exact transaction's staging directory, final destination, and installed record are present. The observer must preserve path and reparse safety, distinguish absence from invalid or inaccessible state, and perform no verification or mutation.

## Changes made

No production changes yet. This note scaffolds the bounded J24K3c1 implementation package.

## Decisions and assumptions

- J24K3c1 observes only the exact transaction named by one publication intent.
- The pure J24K3b classifier remains unchanged.
- Destination contents, current evidence, global installed-root consistency, cleanup, publication, and executor wiring remain later work.

## Evidence

No implementation evidence yet. OpenCode must run the packet checker before work and record exact focused, regression, full-verification, Cargo.lock, diff, and clean-status evidence here.

## Discoveries

None yet.

## Remaining risks

Filesystem absence must not be inferred from broad `Path::exists` checks because those suppress errors and can follow unsafe path state. Exact entry observation must fail closed on reparse, non-ordinary, malformed, or inaccessible state.

## Smallest next action

OpenCode should read the task packet and accepted storage code, implement only the exact read-only observer, add direct filesystem tests, and return the branch for independent review.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/installation_recovery.rs`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/m3_store.rs`
