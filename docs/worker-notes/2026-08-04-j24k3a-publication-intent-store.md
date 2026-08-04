# Worker Note

Task: `J24K3a - Private publication intent record and atomic persistence`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `READY`
Base commit: `WORKTREE`
Implementation checkpoint: `WORKTREE`

## Requested outcome

Add the private crash-recovery publication-intent record and its single-record atomic persistence store. The package must pin one exact precomputed `InstalledPlugRecord`, use that record's `installed_id` as the transaction identity, validate all duplicated identity fields and digests, and safely create, load, and remove only `installation-intent/current.json`.

## Changes made

No production implementation has been made yet. Lucy created the bounded J24K3a branch, task packet, and this worker-note scaffold.

## Decisions and assumptions

- J24K3a contains persistence only.
- The precomputed installed record is supplied to the intent layer; J24K3a does not build, stage, publish, recover, or audit an installation.
- The installed record's `installed_id` is also the publication transaction identity, avoiding a second unrelated UUID.
- The intent store is private to the host crate and accepts only one canonical `current.json` record.

## Evidence

Pending implementation. OpenCode must record exact focused tests, regressions, full verification, Cargo.lock hash, diff check, branch tip, and packet-checker results here.

## Discoveries

None yet.

## Remaining risks

The package is security-sensitive persistence. Malformed, torn, duplicated, unknown, reparse-backed, mismatched, or stale intent state must never be treated as absent or overwritten.

## Smallest next action

Run the control-v1 packet checker, mark the task `IN_PROGRESS`, replace this note's Base commit with the packet's exact base SHA, and implement only J24K3a.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/m3_store.rs`
