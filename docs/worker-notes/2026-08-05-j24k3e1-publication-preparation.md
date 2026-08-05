# Worker Note

Task: `J24K3e1 - Read-only disabled installation publication preparation`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Model: `HY3`
Status: `COMPLETE`
Base commit: `fe4f0e84569e793be3c0e8818799ac36e895da1a`
Implementation checkpoint: `6a82dd529a47f2561234e72a8b7154ede92cabb0`
Verification checkpoint: `WORKTREE`

## Requested outcome

Implement one crate-private, read-only preparation boundary for a future crash-safe `PublishDisabledInstallation` transaction.

The preparation must prove the current ordinary J24J plan is still exactly `PublishDisabledInstallation`, prove private recovery is idle, load and revalidate the exact plan-pinned candidate/trust/launch/conformance/approval chain, precompute one immutable disabled installed record and one matching publication intent, then return them in a sealed prepared value.

No durable state may change in this package.

## Changes made

- Added `tethers-0.1/host-rust/src/installation_publication_preparation.rs`, the
  sealed crate-private read-only preparation boundary and its
  `PreparedInstallationPublication` value.
- Added `tethers-0.1/host-rust/src/installation_publication_preparation_tests.rs`
  with 30 direct `j24k3e1` tests against real stores and filesystem fixtures.
- Modified `tethers-0.1/host-rust/src/installed.rs`: extracted the pure
  `build_disabled_installed_record` constructor now shared with the unchanged
  legacy mutation path, and added the crate-private read-only
  `prepare_disabled_installation_record` seam.
- Modified `tethers-0.1/host-rust/src/lib.rs` with two private module
  registrations only.
- Applied Lucy's authorised mechanical task-packet correction: model and route
  fields set to HY3, and a checker-facing numbered index added under
  `## Required behaviour` without altering the ten frozen `### 1.`–`### 10.`
  subsections.

No durable mutation path, public API, schema, dependency or Cargo.lock change.

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

Production changes:

- `installation_publication_preparation.rs`: new crate-private read-only boundary.
  `prepare_disabled_installation_publication` regenerates a fresh authoritative
  J24J plan and requires exact equality with the supplied before-plan, requires
  `PublishDisabledInstallation` with absent installed pins, requires an idle
  J24K3d1 recovery plan (which runs the global installed-root audit), loads the
  exact plan-pinned candidate/trust/launch/conformance/approval chain from the
  authoritative stores, precomputes one immutable disabled installed record,
  constructs the matching intent through
  `InstallationPublicationIntent::from_precomputed_record`, reruns the complete
  J24K3c3 prepared-intent evidence revalidation, proves recovery is still idle,
  and returns a sealed `PreparedInstallationPublication`.
- `installed.rs`: extracted one pure `build_disabled_installed_record` helper now
  shared by the legacy `install_disabled_with_authority` mutation path (identical
  field derivation, binding order and digest coverage preserved) and a new
  crate-private read-only `prepare_disabled_installation_record` seam that refuses
  duplicate package release / duplicate candidate / contradictory registry state
  and generates one UUID and one `created_unix_ms` without touching the
  filesystem.
- `lib.rs`: two private module registrations only.

Verification (all at implementation checkpoint
`6a82dd529a47f2561234e72a8b7154ede92cabb0`):

- `cargo fmt --all -- --check`: clean.
- Direct `--lib j24k3e1`: 30 passed, 0 failed.
- Focused Nextest `-E 'test(j24k3e1)'`: 30 passed, 0 failed, 0 retries.
- Named regressions all green: j24k3d2 (20), j24k3d1 (28/2 ignored), j24k3c4
  (24), j24k3c3 (44), j24k3c2 (21), j24k3c1 (20), j24k3b (16), j24k3a (25),
  j24k2 (26), j24j_installation_reconciliation (24), m3_lifecycle (13).
- Full serial `just verify` and Cargo.lock hash: recorded at verification.

## Discoveries

- The installed registry's `load_all` requires each record's destination to
  exist on disk; tests that publish a competing installed record must create the
  read-only destination as well as the record file. A record-file-only fixture
  fails `installed_record_invalid` before the duplicate check, which the
  duplicate-release direct test now accounts for.
- Quarantine byte drift is caught during fresh J24J authority regeneration, so
  the plan layer's own `candidate_invalid` classification is preserved rather
  than the recovery `installation_intent_evidence_stale` remap; both are
  fail-closed and mutation-free.

## Remaining risks

- Record construction must not drift from the accepted legacy disabled-record schema.
- The later mutation package must freshly revalidate the prepared transaction immediately before creating durable intent state.
- The later composition package must keep preparation and mutation inside one held installation-lock lifetime.

## Smallest next action

Independent Red review, then the later durable-mutation package that freshly
revalidates the prepared transaction immediately before creating durable intent
state.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `tethers-0.1/host-rust/src/installation_plan.rs`
- `tethers-0.1/host-rust/src/installation_execution.rs`
- `tethers-0.1/host-rust/src/installation_recovery_plan.rs`
- `tethers-0.1/host-rust/src/installation_recovery_evidence.rs`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/installed.rs`
