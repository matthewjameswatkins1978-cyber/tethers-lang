# Worker Note

Task: `J24K2 - Non-inheritable RAII lock and single-step executor`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `READY`
Base commit: `4077aaaad7f59690eea48987e36c88d60ac244fa`
Implementation checkpoint: `PENDING`

## Requested outcome

Add the Windows host installation lock and the bounded single-step J24K executor for exact trust, supervised conformance, installation approval, and complete actions. Preserve the J24K1 explicit-authority boundary. Disabled installation publication remains deferred to J24K3.

## Changes made

Pending implementation.

## Decisions and assumptions

- One invocation performs zero or one logical installation mutation.
- Planning occurs only after the installation lock is acquired.
- The lock is a non-inheritable exclusive Windows file handle, not lock-file existence.
- `PublishDisabledInstallation` is recognised but must fail closed without mutation until J24K3.
- No publication intent, installed-root recovery, CLI, or enablement belongs to J24K2.

## Evidence

Pending implementation and verification.

## Discoveries

Pending implementation.

## Remaining risks

Pending implementation review.

## Smallest next action

Implement the bounded J24K2 packet, update this note with exact code and verification evidence, commit, and push to the same branch.

## References

- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `docs/architecture/J24J_READ_ONLY_INSTALLATION_RECONCILIATION_PLANNER.md`
- `docs/worker-notes/2026-08-04-j24k1-current-trust-authority.md`
