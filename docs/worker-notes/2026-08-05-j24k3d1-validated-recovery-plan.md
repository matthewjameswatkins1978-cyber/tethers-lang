# Worker Note

Task: `J24K3d1 - Validated read-only installation recovery plan`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `READY`
Base commit: `WORKTREE`
Implementation checkpoint: `WORKTREE`
Verification checkpoint: `WORKTREE`

## Requested outcome

Compose the accepted J24K3 recovery seams into one crate-private, read-only recovery planning boundary.

Given the typed installation request, the authoritative publication-intent store, the installed registry, and the accepted evidence stores, return either:

- no pending recovery; or
- one exact validated recovery disposition whose required read-only proofs have completed.

The planner must load the optional current intent itself. The caller must not be able to suppress an existing transaction by supplying `None`.

The package must perform no mutation, acquire no lock, delete no staging directory, publish no installed record, remove no intent, and wire no executor action.

## Changes made

None yet.

## Decisions and assumptions

- DeepSeek Pro is selected for this bounded Rust composition package.
- The accepted classifier remains pure and unchanged.
- The accepted intent store, observer, destination verifier, evidence revalidator, and installed-root audit remain the authority boundaries; this package composes rather than duplicates them.
- A no-intent result still performs the global installed-root audit with `None`, so an orphan final destination cannot be hidden by the absence of a transaction.
- Cleanup-only dispositions do not require current package evidence because they do not publish or bless durable installed state.
- Publication-ready and completed-publication dispositions require both current evidence and exact destination verification.
- Workers record implementation and verification checkpoints only. Lucy records the reviewed remote tip after review, avoiding self-referential SHA updates.

## Evidence

Not run yet.

## Discoveries

- J24K3a through J24K3c4 provide every read-only primitive needed to decide whether recovery is absent, cleanup-only, publication-ready, or completion-ready.
- Mutation remains safer as a later package if it consumes one already validated recovery plan rather than independently recomposing evidence.

## Remaining risks

- The returned plan must not carry mutable stores, arbitrary paths, callbacks, caller-supplied booleans, or an externally supplied intent.
- A later mutation package must recheck the exact authoritative intent before changing durable state because this package is intentionally read-only.

## Smallest next action

Implement only the task packet, verify it, and return the branch for independent review. Do not merge.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/installation_recovery.rs`
- `tethers-0.1/host-rust/src/installation_recovery_evidence.rs`
- `tethers-0.1/host-rust/src/installed.rs`
