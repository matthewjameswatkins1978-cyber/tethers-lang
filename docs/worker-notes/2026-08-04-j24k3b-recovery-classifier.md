# Worker Note

Task: `J24K3b - Pure publication recovery-state classifier`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `02ff3a9f6475d6ee243ab8fe662a4d3bb74d1b73`
Implementation checkpoint: `2ee9b59048811461b1549f800a28744cc1075424`

## Requested outcome

Add one private, pure, typed classifier for the validated-current-intent portion of the frozen J24K recovery matrix. The classifier receives one validated publication intent plus already-observed staging, destination, and installed-record presence. It returns one typed recovery disposition or fails closed on contradictory state.

## Changes made

Added `src/installation_recovery.rs` with the crate-private `InstallationRecoveryObservation` struct, `InstallationRecoveryDisposition` enum, and `classify_installation_recovery` function. The observation carries named typed facts (intent reference, staging/destination booleans, optional installed record reference) with no paths or mutable capabilities. The disposition enum has exactly four variants: `RemoveIntentOnly`, `RemoveStagingThenIntent`, `RevalidateDestinationThenPublishRecord`, `VerifyCompletedPublicationThenRemoveIntent`.

The classifier validates intent first, then validates any present installed record, then matches against the frozen matrix:
- intent only -> `RemoveIntentOnly`
- intent + staging only -> `RemoveStagingThenIntent`
- intent + destination only -> `RevalidateDestinationThenPublishRecord`
- intent + destination + exact matching record -> `VerifyCompletedPublicationThenRemoveIntent`
- all other combinations -> `installation_recovery_conflict`
- invalid intent -> `installation_intent_invalid`

Staging plus destination always conflicts regardless of record state. Record without destination always conflicts regardless of staging state. Matching requires validated exact full-record equality with the embedded precomputed record. No filesystem access, evidence revalidation, mutation, or I/O is performed.

Registered `mod installation_recovery` and `#[cfg(test)] mod installation_recovery_tests` privately in `lib.rs`.

Added `src/installation_recovery_tests.rs` with 14 direct production-seam tests prefixed `j24k3b` covering all four successful rows, all required contradictory rows, invalid intent before state classification, same-ID-different-fields conflict, non-mutation of supplies, and deterministic repeatability.

## Decisions and assumptions

- J24K3b classifies facts only.
- It performs no filesystem access, evidence revalidation, mutation, cleanup, publication, installed-root audit, planning, locking, or executor wiring.
- Absence of a current intent and untracked-final detection remain for later observation/audit work.
- A present installed record matches only by validated exact equality with the intent's embedded precomputed record.
- Classification order places staging-plus-destination early return and record-present-without-destination in the fallback arm to prevent contradictory rows from being swallowed.

## Evidence

- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`: passed.
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --lib j24k3b --locked`: 14 passed, 991 filtered, 0 failed.
- `cargo nextest run --config-file .config/nextest.toml --manifest-path tethers-0.1/host-rust/Cargo.toml --all-features --locked -E 'test(j24k3b)'`: 14 passed, 1231 skipped, 0 retries.
- J24K3a unit tests: 25 passed, 0 failed.
- J24K2 unit tests: 26 passed, 0 failed.
- J24J integration: 24 passed, 0 failed.
- M3 lifecycle: 13 passed, 0 failed.
- `just test-rust` (with `$PSHOME` prepended): 1005 lib tests passed, 0 failed; all integration suites passed.
- `cargo fmt --check`: passed.
- `git diff --check`: passed.
- `Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256`: `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB`.
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`: structural mismatch (23 required vs 18 acceptance) — pre-existing packet-authoring issue, not caused by implementation. `just verify` fails on this step alone; all test portions pass.
- Production checkpoint SHA: `2ee9b59048811461b1549f800a28744cc1075424`.

## Discoveries

- The `InstallationPublicationIntent` and `InstalledPlugRecord` types are accessible from a crate-private module via the existing public module declarations. No change to accepted modules was required.
- Test helper `valid_record()` mirrors the pattern from `installation_publication_intent_tests.rs`; same-ID-different-fields record uses identical installed_id with a different package_version and a separately recomputed digest to prove full-record equality (not ID-only or digest-field-only).
- The packet checker structural constraint (23 required vs 18 acceptance) is a pre-existing authoring issue. The task was `READY` when taken, and Lucy compiled it. All implementation acceptance criteria are satisfied against the evidence.

## Remaining risks

- The classifier must not accidentally imply that destination verification or evidence revalidation has already succeeded. Its output names the next required recovery path; it does not authorise or perform that path.
- Later J24K3 work must still perform destination byte verification, installed-root audit, and untracked-final detection; those responsibilities remain outside this package.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/installed.rs`
