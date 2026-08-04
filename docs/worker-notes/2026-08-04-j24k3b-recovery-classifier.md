# Worker Note

Task: `J24K3b correction - record validation ordering and final verification`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `e09a16004a9f634e99e39491e2469a6cb5ec337d`
Implementation checkpoint: `d31185fb68e1df5e73853f1807c048285e4c0da2`

## Requested outcome

Apply one bounded correction to the otherwise complete J24K3b classifier: validate every present installed record immediately after validating the intent and before applying any recovery-matrix row. Add direct invalid-record edge coverage, repair the control packet structure, and complete the required full verification.

## Changes made

In `classify_installation_recovery`, added `if let Some(record) = observation.installed_record { record.validate().map_err(|_| recovery_conflict())?; }` immediately after intent validation and before the staging-plus-destination early return or matrix match. Removed the duplicate `record.validate()` from the destination-plus-record match arm.

This ensures every present installed record is validated regardless of observation state, including record-without-destination and staging-plus-destination rows. Lower-layer installed-record validation errors map only to the stable `installation_recovery_conflict` contract.

Added two direct edge tests:
- `j24k3b_invalid_record_without_destination_conflicts`: invalid record (schema_version=0), no staging, no destination → `installation_recovery_conflict`
- `j24k3b_staging_plus_destination_plus_invalid_record_conflicts`: invalid record, both staging and destination → `installation_recovery_conflict`

All fourteen existing tests retained. Total: 16 direct J24K3b tests.

## Decisions and assumptions

- The four successful dispositions and recovery matrix remain unchanged.
- Matching still requires validated exact full-record equality.
- Record validation occurs before the staging-plus-destination early return, complying with the requirement that every present record be validated before matrix classification.

## Evidence

- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`: passed.
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --lib j24k3b --locked`: 16 passed, 991 filtered, 0 failed.
- `cargo nextest run --config-file .config/nextest.toml --manifest-path tethers-0.1/host-rust/Cargo.toml --all-features --locked -E 'test(j24k3b)'`: 16 passed, 1231 skipped, 0 retries.
- J24K3a unit tests: 25 passed, 0 failed.
- J24K2 unit tests: 26 passed, 0 failed.
- J24J integration: 24 passed, 0 failed.
- M3 lifecycle: 13 passed, 0 failed (one pre-existing parallel handle-contention failure in `m3_windows_handle_allow_list_excludes_unrelated_inheritable_handle` passed on required serial rerun).
- `$env:PATH = "$PSHOME;$env:PATH"; just verify`: 1007 lib tests passed, 0 failed; all integration suites passed.
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`: PASS.
- `git diff --check`: passed.
- `Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256`: `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB`.
- Implementation checkpoint: `d31185fb68e1df5e73853f1807c048285e4c0da2`.
- Final remote tip at evidence commit: `4a77729f7cd13f181fe2ecbead58cc723a6a610d`.

## Discoveries

The production function originally checked `staging_present && destination_present` before validating a supplied record and validated a record only in the destination-plus-record match arm. This contradicted the explicit contract that intent validation is followed by validation of any present installed record before the matrix is classified. The correction moves record validation to immediately follow intent validation for all observation states.

The pre-existing `m3_windows_handle_allow_list_excludes_unrelated_inheritable_handle` test fails intermittently under parallel execution (Windows handle contention) and passes reliably on serial rerun. This is unrelated to J24K3b.

## Remaining risks

Later J24K3 packages still own filesystem observation, destination verification, installed-root audit, recovery mutation, and executor wiring. None of those concerns belongs in this correction.

## Smallest next action

OpenCode should apply the one production ordering fix, add direct invalid-record tests for broad conflict rows, run the corrected packet checker, run full `just verify`, and return the branch for independent review.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `tethers-0.1/host-rust/src/installation_recovery.rs`
- `tethers-0.1/host-rust/src/installation_recovery_tests.rs`
