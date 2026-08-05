# Worker Note

Task: `J24K3d1 - Validated read-only installation recovery plan`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `e2cffcb93fdd457cadf2091b8657e7e6a4e8a5a2`
Implementation checkpoint: `351a2782b59d1b08c5529bd18caf8a7fa29cde6b`
Verification checkpoint: `b76691c5b97bd1b3a82de824535365fec4676c20`

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

Full `just verify` (RUST_TEST_THREADS=1): 1141 lib passed, 239 integration passed (m3_lifecycle intermittent excluded).

## Discoveries

- The sealed-plan pattern (private fields, no public constructor) works cleanly with `Option<(Intent, Disposition)>` internally while exposing `is_idle()`, `intent()`, `disposition()` accessors
- Evidence-store staleness tests require fresh empty stores rather than directory deletion, since missing directories produce `recovery_io` on Windows instead of `evidence_stale`
- Copying files from the actual quarantine extraction directory is necessary for destination verification, since evidence digests match the extracted files

## Remaining risks

- The returned plan carries the loaded intent; a later mutation package must recheck the authoritative intent before changing durable state
- m3_lifecycle intermittent handle-contention failure is known and documented

## Smallest next action

Push the final documentation commit. Return the verified branch to Matthew for Lucy's independent review. Do not merge.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/installation_recovery.rs`
- `tethers-0.1/host-rust/src/installation_recovery_evidence.rs`
- `tethers-0.1/host-rust/src/installed.rs`
