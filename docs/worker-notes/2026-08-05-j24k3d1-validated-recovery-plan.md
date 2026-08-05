# Worker Note

Task: `J24K3d1 - Validated read-only installation recovery plan`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `e2cffcb93fdd457cadf2091b8657e7e6a4e8a5a2`
Implementation checkpoint: `351a27867078f4f37bca80bd2f481e790cdfb5cf`
Verification checkpoint: `WORKTREE`

## Requested outcome

Add one crate-private, read-only recovery-planning boundary that composes the accepted J24K3a through J24K3c4 primitives.

## Changes made

- `tethers-0.1/host-rust/src/installation_recovery_plan.rs` (new): crate-private `plan_installation_recovery`, `InstallationRecoveryPlanningContext`, sealed `ValidatedInstallationRecoveryPlan` with private fields and accessors
- `tethers-0.1/host-rust/src/installation_recovery_plan_tests.rs` (new): 25 direct production-entry-point tests exercising all four recovery dispositions, idle route, evidence staleness, destination drift, conflict states, and read-only proof
- `tethers-0.1/host-rust/src/lib.rs`: registered `installation_recovery_plan` and `installation_recovery_plan_tests` modules
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`: added narrow `pub(crate) fn root_path()` accessor

## Decisions and assumptions

- Planner always loads the authoritative current intent itself from `InstallationPublicationIntentStore`
- Global installed-root audit runs for both intent-present and intent-absent state
- Cleanup-only dispositions (RemoveIntentOnly, RemoveStagingThenIntent) do not require current package evidence
- Publication-ready dispositions (RevalidateDestinationThenPublishRecord, VerifyCompletedPublicationThenRemoveIntent) require both evidence revalidation and exact destination verification
- Sealed plan invariant: idle (no intent, no disposition) or pending (intent + disposition); mixed states unrepresentable outside module
- No mutation, lock acquisition, J24J planning, or executor wiring performed

## Evidence

All 25 j24k3d1 tests pass:
- idle route: 2 tests
- no-intent audit failures: 2 tests
- intent-only: 1 test
- staging-only: 2 tests
- destination-only with evidence: 1 test
- completed publication: 1 test
- conflict states: 2 tests
- untracked destinations: 1 test
- evidence staleness: 6 tests (request, candidate, trust, launch, conformance, approval)
- installed-record staleness: 1 test
- destination drift (file-set, digest, size, permission): 4 tests
- completed publication still requires evidence: 1 test
- read-only proof: 1 test

Regression: J24K3c4 (24), J24K3c3 (44), J24K3c2 (21), J24K3c1 (20), J24K3b (16), J24K3a (25), J24K2 (26), J24J (24) all pass. M3 lifecycle (12 of 13 pass, 1 known intermittent Windows handle-contention failure).

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
