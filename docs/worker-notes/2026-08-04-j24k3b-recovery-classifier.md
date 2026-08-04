# Worker Note

Task: `J24K3b - Pure publication recovery-state classifier`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `READY`
Base commit: `WORKTREE`
Implementation checkpoint: `WORKTREE`

## Requested outcome

Add one private, pure, typed classifier for the validated-current-intent portion of the frozen J24K recovery matrix. The classifier receives one validated publication intent plus already-observed staging, destination, and installed-record presence. It returns one typed recovery disposition or fails closed on contradictory state.

## Changes made

Not started.

## Decisions and assumptions

- J24K3b classifies facts only.
- It performs no filesystem access, evidence revalidation, mutation, cleanup, publication, installed-root audit, planning, locking, or executor wiring.
- Absence of a current intent and untracked-final detection remain for later observation/audit work.
- A present installed record matches only by validated exact equality with the intent's embedded precomputed record.

## Evidence

Not run.

## Discoveries

None yet.

## Remaining risks

The classifier must not accidentally imply that destination verification or evidence revalidation has already succeeded. Its output names the next required recovery path; it does not authorise or perform that path.

## Smallest next action

Run the task-packet checker, read the frozen recovery matrix and accepted J24K3a seam, then implement the exact typed matrix without broadening scope.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/installed.rs`
