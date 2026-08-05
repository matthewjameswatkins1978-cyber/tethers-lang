# Worker Note

Task: `J24K3d2 - Exact installation recovery executor`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Model: `Luna`
Status: `IN_PROGRESS`
Base commit: `ea4076085ed246a95eb2c0edab462b8c69d461fc`
Implementation checkpoint: `WORKTREE`
Verification checkpoint: `WORKTREE`

## Requested outcome

Implement one crate-private recovery executor that consumes only a sealed J24K3d1 plan, rechecks the authoritative current state immediately before mutation, performs the exact accepted recovery sequence, and proves recovery returns to idle.

The package must complete only recovery. It must not create new publication intents, build a new staging directory, rename staging into a final destination, acquire the installation lock, run J24J, execute an ordinary installation action, or wire the public executor.

## Changes made

None yet.

## Decisions and assumptions

- J24K3d1 remains the sole planner and classifier composition boundary.
- The executor accepts only `ValidatedInstallationRecoveryPlan`; callers cannot supply an intent, disposition, booleans, paths, or repair policy.
- A fresh J24K3d1 plan must exactly match the supplied sealed plan immediately before the first mutation.
- Staging cleanup and exact installed-record publication are narrow host-owned registry operations.
- `InstallationPublicationIntentStore::remove_if_matches` remains the only intent-removal seam.
- Failed staging cleanup or failed record publication retains the authoritative intent.
- After staging cleanup, recovery must replan to `RemoveIntentOnly` before removing the intent.
- After exact record publication, recovery must replan to `VerifyCompletedPublicationThenRemoveIntent` before removing the intent.
- The final postcondition is a fresh idle recovery plan after the global installed-root audit.
- Lock integration remains a later package. This crate-private seam is not wired into any public entry point in J24K3d2.
- Workers record implementation and verification checkpoints only. Do not commit a final remote tip field.

## Evidence

Not run yet.

## Discoveries

None yet.

## Remaining risks

- The later lock-integration package must ensure planning and recovery execution occur inside one held installation lock lifetime.
- The later publication package must create the durable intent and staging/final destination transaction that this executor recovers.

## Smallest next action

Implement only the packet, verify the complete recovery matrix, push the branch, and return it for Lucy’s independent review.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `tethers-0.1/host-rust/src/installation_recovery_plan.rs`
- `tethers-0.1/host-rust/src/installation_recovery.rs`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/installed.rs`
