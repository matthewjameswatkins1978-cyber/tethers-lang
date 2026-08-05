# Worker Note

Task: `J24K3e2 - Exact durable disabled installation publication mutation`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Model: `HY3`
Status: `COMPLETE`
Base commit: `45f78e47a09638d4070bf4479e4f1dcbe39c8cb1`
Implementation checkpoint: `043eab0f2c0a45d47aae14adc1777a06882095ca`
Verification checkpoint: `a20e286d9907b7cb54a379abe406d83f709f8c80`

## Requested outcome

Implement one crate-private mutation boundary that consumes the sealed J24K3e1 prepared publication, freshly revalidates it immediately before durable mutation, and performs the exact crash-safe disabled-installation publication transaction.

The transaction persists the exact intent, builds and verifies exact staging, renames to the exact destination, publishes the exact precomputed record unchanged, proves completed publication through fresh recovery planning, removes only the completed intent, and returns to idle recovery.

## Changes made

- `tethers-0.1/host-rust/src/installation_publication_mutation.rs` (new): `execute_prepared_disabled_installation_publication(request, context, prepared)` crate-private seam. Consumes the sealed prepared value, freshly revalidates all evidence, requires idle recovery and a clean global installed-root audit, refuses duplicate release/candidate, persists the exact intent atomically, builds and verifies exact staging, renames to the exact destination, publishes the exact precomputed record, removes only the completed intent through accepted recovery, and proves idle recovery plus exact destination/record.
- `tethers-0.1/host-rust/src/installation_publication_mutation_tests.rs` (new): 26 direct tests named `j24k3e2_*` covering the complete acceptance matrix.
- `tethers-0.1/host-rust/src/installed.rs`: added three crate-private publication seams `build_installation_recovery_staging`, `verify_installation_recovery_staging`, and `rename_installation_recovery_staging`. Removed the unused private helper `install_root_path()` before the implementation commit.
- `tethers-0.1/host-rust/src/lib.rs`: registered the new mutation module and its `#[cfg(test)]` test module.
- `docs/CURRENT_CLINE_TASK.md` and this worker note: status transitions and checkpoint records.

No `InstallationLockGuard`, `InstallationExecutionContext`, `installation_publication_deferred`, public executor, or CLI wiring was changed. No other production file was modified.

## Decisions and assumptions

- J24K3e1 remains the sole transaction-identity and immutable-content preparation authority; the prepared UUID, destination, timestamp, record digest and intent digest are never regenerated.
- J24K3 recovery planning and execution remain the sole recovery, audit, cleanup and completion authority; intent removal goes through `execute_validated_installation_recovery` on a fresh completed-publication plan.
- Intent creation is the first durable mutation and follows one fresh complete revalidation (evidence revalidator, idle recovery plan, global installed-root audit, duplicate-release and duplicate-candidate checks, exact intent and record validation).
- Staging is built only from the candidate file set justified by the prepared record and is verified through the same evidence used to verify the final destination before rename.
- The installed record is published through the existing `publish_installation_recovery_record` seam; it is never routed through legacy `install_disabled`.
- This package does not acquire the installation lock or wire the public executor; later composition must hold one lock across preparation and mutation.

## Evidence

Direct tests at implementation checkpoint `043eab0f2c0a45d47aae14adc1777a06882095ca`:

- `cargo test -p tethers-reference-host j24k3e2 --no-fail-fast` — PASS, 26 passed, 0 failed.

Named regressions (each `cargo test -p tethers-reference-host <filter> --no-fail-fast` — PASS, exit 0):

- `j24k3e1` — 30 passed
- `j24k3d2` — 20 passed
- `j24k3d1` — 28 passed, 2 ignored
- `j24k3c4` — 24 passed
- `j24k3c3` — 44 passed
- `j24k3c2` — 21 passed
- `j24k3c1` — 20 passed
- `j24k3b` — 16 passed
- `j24k3a` — 25 passed
- `j24k2` — 26 passed
- `j24j` — 0 matched by name filter (tests live in `tests/j24j_installation_reconciliation.rs` under descriptive names); fully exercised by full serial verification below.

Full serial verification with `RUST_TEST_THREADS=1`:

- `just verify` — PASS. Task-packet checker, `cargo fmt --check`, `cargo check --all-targets --all-features --locked`, then full `cargo test --all-targets --all-features --locked`. 1220 lib tests passed, 2 ignored; all 25 test-result lines report 0 failures.

Final gates:

- `cargo fmt --all -- --check` — PASS.
- `git diff --check` — PASS.
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` — PASS (control-v1/IN_PROGRESS at checkpoint).
- `cargo check -p tethers-reference-host` — PASS (exit 0).
- `Cargo.lock` — unchanged. SHA-256 `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB` before and after.

Preserved authoritative error classifications (no broad publication error introduced; earlier lower-layer classifications retained):

- `installation_intent_evidence_stale` — stale evidence before mutation creates no intent.
- `installation_recovery_conflict` — non-idle recovery, mismatched record, record-without-destination.
- `installation_destination_untracked` — existing/untracked final destination.
- `installation_recovery_io` — existing staging, staging build failure.
- `unsafe_store_path` — reparse/junction staging path.
- `installation_intent_invalid` — tampered embedded record digest.
- `installed_conflict` — duplicate release or duplicate candidate.

## Discoveries

- The `j24j` tests are named descriptively inside `tests/j24j_installation_reconciliation.rs`, so the `cargo test -p tethers-reference-host j24j --no-fail-fast` filter matches zero tests by name; the tests are still run in full serial verification. This is a naming observation only and was not changed.
- `installed.rs` already carried the accepted recovery snapshot, destination verification, staging removal and record publication seams from earlier J24K3 work; the new build/verify/rename staging seams compose with them rather than duplicating authority.

## Remaining risks

- Staging rename uses `fs::rename` within one install root; crash windows between rename and record publication are the accepted recovery table prefixes and were exercised in the direct tests. No new state outside the accepted recovery table was introduced.
- The record is byte-semantically equal to the prepared record on this run; any future schema or digest change must flow through preparation, not mutation.
- Later composition must keep J24K3e1 preparation and J24K3e2 mutation inside one held installation-lock lifetime; that composition is a separate task.

## Smallest next action

Lucy reviews the branch diff and worker-note evidence, then runs the routine safe merge to `main`; the next bounded task is the installation-lock composition that holds one lock across J24K3e1 preparation and J24K3e2 mutation.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `tethers-0.1/host-rust/src/installation_publication_mutation.rs`
- `tethers-0.1/host-rust/src/installation_publication_mutation_tests.rs`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `tethers-0.1/host-rust/src/installation_publication_preparation.rs`
- `tethers-0.1/host-rust/src/installation_recovery_plan.rs`
- `tethers-0.1/host-rust/src/installation_recovery_execution.rs`
- `tethers-0.1/host-rust/src/installation_recovery_evidence.rs`
