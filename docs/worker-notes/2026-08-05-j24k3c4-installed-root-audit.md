# Worker Note

Task: `J24K3c4 - Global installed-root consistency auditor`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `READY`
Base commit: `WORKTREE`
Implementation checkpoint: `WORKTREE`
Final branch: `opencode/j24k3c4-installed-root-audit`

## Requested outcome

Add one crate-private, read-only global installed-root audit for J24K recovery. Given the optional current validated publication intent, prove that every direct final-form `plug-*` destination is accounted for by one validated installed record or by that one current intent.

The audit must detect and refuse untracked final destinations without adopting, deleting, repairing, or inspecting unrelated non-final entries. Existing installed records remain authoritative through accepted installed-state validation; an intent may authorize its one exact destination even before its record exists.

## Changes made

None yet.

## Decisions and assumptions

- DeepSeek Pro is the selected implementation model for this bounded Rust filesystem and record-reconciliation package.
- The package is read-only and performs no recovery classification, cleanup, publication, intent removal, locking, planning, or executor wiring.
- J24K3c1 owns exact transaction-state observation, J24K3c2 owns exact intent-destination content verification, and J24K3c3 owns evidence-chain freshness. This package owns only the global final-destination accounting invariant.

## Evidence

Not run yet.

## Discoveries

- Existing `InstalledPlugRegistry::load_all()` validates every installed record and its destination but does not enumerate the install root for orphan final directories.
- Final destination identity is generated as `plug-<installed_id>`; the audit must reject a record whose destination identity is not exactly canonical even if its record digest is otherwise internally valid.

## Remaining risks

- The audit must not treat an allowed current intent destination as a completed record or repeat J24K3c2 content verification.
- Staging and unrelated non-final entries must remain outside this package's classification boundary.

## Smallest next action

Implement only the task packet, run the complete focused and regression verification matrix, then return the branch for independent review.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/installation_recovery.rs`
