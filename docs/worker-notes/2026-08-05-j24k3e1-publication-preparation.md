# Worker Note

Task: `J24K3e1 - Read-only disabled installation publication preparation`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Model: `HY3`
Status: `READY`
Base commit: `fe4f0e84569e793be3c0e8818799ac36e895da1a`
Implementation checkpoint: `WORKTREE`
Verification checkpoint: `WORKTREE`

## Requested outcome

Implement one crate-private, read-only preparation boundary for a future crash-safe `PublishDisabledInstallation` transaction.

The preparation must prove the current ordinary J24J plan is still exactly `PublishDisabledInstallation`, prove private recovery is idle, load and revalidate the exact plan-pinned candidate/trust/launch/conformance/approval chain, precompute one immutable disabled installed record and one matching publication intent, then return them in a sealed prepared value.

No durable state may change in this package.

## Decisions and assumptions

- J24J remains the sole ordinary installation reconciliation authority.
- J24K3d1 remains the sole recovery planning, audit, classification and recovery-proof boundary.
- The preparation accepts the existing `before` plan only as an exact value to compare with a newly generated authoritative J24J plan.
- A pending or malformed recovery transaction blocks preparation.
- The new installed ID and `created_unix_ms` are generated exactly once per prepared transaction and are frozen into the record and intent.
- The prepared value has private fields and no arbitrary constructor.
- The existing public and authority-aware legacy installation methods retain their signatures, mutation order and observable behaviour.
- Intent persistence, staging creation, staging verification, destination rename, exact-record publication, intent removal, lock integration and public executor wiring remain later packages.
- Workers record implementation and verification checkpoints only. Do not commit a final remote tip field.

## Evidence

Not run yet.

## Discoveries

None yet.

## Remaining risks

- Record construction must not drift from the accepted legacy disabled-record schema.
- The later mutation package must freshly revalidate the prepared transaction immediately before creating durable intent state.
- The later composition package must keep preparation and mutation inside one held installation-lock lifetime.

## Smallest next action

Read the task packet and accepted J24J, recovery, evidence and installed-state seams completely before editing.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `tethers-0.1/host-rust/src/installation_plan.rs`
- `tethers-0.1/host-rust/src/installation_execution.rs`
- `tethers-0.1/host-rust/src/installation_recovery_plan.rs`
- `tethers-0.1/host-rust/src/installation_recovery_evidence.rs`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/installed.rs`
