# Worker Note

Task: `J24K3e2 - Exact durable disabled installation publication mutation`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Model: `HY3`
Status: `IN_PROGRESS`
Base commit: `45f78e47a09638d4070bf4479e4f1dcbe39c8cb1`
Implementation checkpoint: `WORKTREE`
Verification checkpoint: `WORKTREE`

## Requested outcome

Implement one crate-private mutation boundary that consumes the sealed J24K3e1 prepared publication, freshly revalidates it immediately before durable mutation, and performs the exact crash-safe disabled-installation publication transaction.

The transaction must persist the exact intent, build and verify exact staging, rename to the exact destination, publish the exact precomputed record unchanged, prove completed publication through fresh recovery planning, remove only the completed intent, and return to idle recovery.

## Changes made

None yet.

## Decisions and assumptions

- J24K3e1 remains the sole transaction-identity and immutable-content preparation authority.
- J24K3 recovery planning and execution remain the sole recovery, audit, cleanup and completion authority.
- This package does not acquire the installation lock or wire the public executor.
- The prepared UUID, destination, timestamp, record digest and intent digest are immutable and must never be regenerated.
- Intent creation is the first durable mutation and must follow one fresh complete revalidation.

## Evidence

Not run yet.

## Discoveries

None yet.

## Remaining risks

- The mutation sequence must not create any durable prefix outside the accepted recovery table.
- Exact record publication may need one minimum crate-private seam without changing legacy installation behaviour.
- Later composition must keep J24K3e1 preparation and J24K3e2 mutation inside one held installation-lock lifetime.

## Smallest next action

Run the task-packet checker, change both statuses to `IN_PROGRESS`, inspect the accepted J24K3e1 and recovery seams, then implement only the bounded exact publication mutation and its direct tests.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `tethers-0.1/host-rust/src/installation_publication_preparation.rs`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/installation_recovery_plan.rs`
- `tethers-0.1/host-rust/src/installation_recovery_execution.rs`
- `tethers-0.1/host-rust/src/installation_recovery_evidence.rs`
- `tethers-0.1/host-rust/src/installation_destination_verification.rs`
- `tethers-0.1/host-rust/src/installed.rs`
