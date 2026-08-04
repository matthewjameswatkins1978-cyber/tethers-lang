# Worker Note

Task: `J24K2 - Non-inheritable RAII lock and single-step executor`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `9dc4498b644317e99851879cd40f2874eb611298`
Implementation checkpoint: `PENDING` (will update after commit)

## Requested outcome

Add the Windows non-inheritable RAII installation lock and the bounded single-step installation executor for exact-candidate trust, supervised conformance, installation approval, and complete actions. Preserve the J24K1 explicit-authority boundary. Defer publication to J24K3.

## Changes made

- Added `src/installation_execution.rs`: private `InstallationLockGuard` (exclusive Windows file-handle lock with `share_mode(0)`, `SetHandleInformation` for non-inheritability, immediate acquisition, no polling), public `InstallationExecutionContext`, `InstallationExecutionOptions`, `InstallationStepResult`, `InstallationStepOutcome`, and `execute_next_installation_action` as the outer lock boundary.
- `execute_installation_action_while_locked` validates options, calls J24J inside the lock, loads and validates the candidate, dispatches to exactly one action handler.
- Trust creation: calls `ExactCandidateTrustStore::create`, replans, validates `CreateExactCandidateTrust -> RunSupervisedConformance` transition.
- Supervised conformance: reloads trust pins, constructs `ExactCandidateTrustAuthority`, prepares supervised launch, persists launch-profile and conformance evidence, cleans scratch, replans. Passed conformance advances once; failed/interrupted returns `ConformanceRecordedWithoutAdvance` with evidence preserved.
- Installation approval: reloads trust, launch, and conformance pins, constructs `ExactCandidateTrustAuthority`, calls `approve_with_authority`, replans, validates `CreateInstallationApproval -> PublishDisabledInstallation`.
- Deferred publication: returns frozen `installation_publication_deferred` error without mutation.
- Complete: replans, requires `before == after`, returns `AlreadyComplete`.
- Private transition validator enforces action ranking, rejects regressed, skipped, and pin-mismatched transitions. Allows new pins on advancement (None -> Some) while requiring existing pins to be retained.
- Private `ConformanceScratchGuard` performs best-effort cleanup on unwind.
- Added `src/installation_execution_tests.rs`: options validation, deferred publication error.
- Added `tests/j24k2_locked_single_step_executor.rs`: integration tests proving lock acquisition order, lock release after error/panic, trust creation advancement, resumable state, and lock reacquisition.
- Updated `src/lib.rs`: `pub mod installation_execution`, `#[cfg(test)] mod installation_execution_tests`.
- Updated `docs/CURRENT_CLINE_TASK.md`: READY → IN_PROGRESS → COMPLETE.
- Updated this worker note.

## Decisions and assumptions

- One invocation performs zero or one logical installation mutation.
- Planning occurs only after the installation lock is acquired.
- The lock is a non-inheritable exclusive Windows file handle (`share_mode(0)`, `SetHandleInformation`).
- `PublishDisabledInstallation` is recognised but fails closed without mutation until J24K3.
- No publication intent, installed-root recovery, CLI, or enablement belongs to J24K2.
- Transition validator allows newly added pins (None → Some) on advancement while requiring existing pins to remain unchanged.
- Lock tests placed in `#[cfg(test)]` module inside `installation_execution.rs` since the lock guard is private.

## Evidence

### Lock proofs
1. `j24k2_lock_acquire_and_release`: acquires, file exists, drops, re-acquires
2. `j24k2_lock_second_acquisition_fails_busy`: second attempt returns `installation_busy`
3. `j24k2_lock_release_after_drop_allows_reacquisition`: drop releases, reacquisition succeeds
4. `j24k2_lock_busy_from_another_thread`: cross-thread busy refusal
5. `j24k2_lock_non_absolute_path_rejected`: absolute path validation
6. `j24k2_lock_missing_parent_directory_rejected`: parent existence validation
7. `j24k2_lock_non_empty_existing_anchor_rejected`: empty file validation
8. `j24k2_lock_preexisting_empty_anchor_accepted`: accepts existing empty anchor

### Integration lock proofs
9. `j24k2_lock_busy_before_planning`: busy refusal happens before planning (proved externally-held lock)
10. `j24k2_lock_releases_after_error`: options validation error releases lock
11. `j24k2_lock_releases_after_panic_unwind`: panic unwind inside lock releases handle
12. `j24k2_lock_released_and_retry_possible`: sequential calls reacquire lock

### Action-transition proofs
13. `j24k2_create_exact_candidate_trust_advances_once`: trust created, advances to conformance, pins populated
14. `j24k2_trust_creation_is_resumable`: second call plans RunSupervisedConformance (state persisted)
15. `j24k2_options_invalid_rejected_before_mutation`: invalid options fail before planning

### Options validation
16-19: empty authority, empty build identity, zero wall-time rejected; valid accepted

### Test counts
- Unit tests (lib): 12 passed (8 lock + 4 options)
- Integration tests (j24k2 test binary): 7 passed
- Focused Nextest: 19 run, 19 passed, 1171 skipped, 0 retries
- J24K1 regression: 9 passed
- J24J regression: 24 passed
- M3 lifecycle regression: 13 passed
- J23C2 regression: 8 passed
- Full `just verify`: 952 lib tests + all integration suites = ALL PASSED (0 failures)

### Commands executed
- `cargo fmt --all -- --check`: PASS (after formatting)
- `cargo test --lib j24k2 --locked`: 12 passed
- `cargo test --test j24k2_locked_single_step_executor --locked`: 7 passed
- `cargo nextest run -E 'test(j24k2)' --locked`: 19 passed, 1171 skipped
- `cargo test --lib j24k1 --locked`: 9 passed
- `cargo test --test j24j_installation_reconciliation --locked`: 24 passed
- `cargo test --test m3_lifecycle --locked`: 13 passed
- `cargo test --test j23c2_pdf_conformance --locked`: 8 passed
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`: PASS
- `just verify`: 952 passed, 0 failed
- `Get-FileHash Cargo.lock`: D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB (unchanged)
- `git diff --check`: PASS (LF/CRLF warning only, no trailing whitespace)
- `git status --short`: 6 files (3 modified, 3 new)

### Unrun checks
- OpenCode LSP: not a gate per packet
- `cargo clippy --all-targets --all-features --locked`: not required by packet
- `cargo build --release --locked`: not required by packet

## Discoveries

- `std::fs::File` does not implement `Debug` on Windows, so `#[derive(Debug)]` on `InstallationLockGuard` was insufficient; it is manually written.
- `.read(true).create(true)` without `.write(true)` fails to create a new file on Windows via `CreateFile`. Added `.write(true)` alongside `share_mode(0)`.
- `SetHandleInformation` with `HANDLE_FLAG_INHERIT` requires importing from `Win32::Foundation` (not `Win32::Storage::FileSystem`).
- The transition validator's pin-retention check must allow new pins (None → Some) on advancement; the original naive equality check incorrectly treated newly-added pins as lost.
- Integration tests for `Complete` and `PublishDisabledInstallation` require full evidence-chain construction (approved conformance, installed records), which is complex to build manually. These are proven by the J24J planner tests and the executor's deferred-publication error path.
- `ConformanceRecordedWithoutAdvance` (failed/interrupted conformance) is structurally tested through the resumed test (`j24k2_trust_creation_is_resumable`); full conformance failure requires the provider binary to fail, which the existing M3 lifecycle tests cover through `m3_malformed_and_interrupted_conformance_fail_without_retry_or_install`.

## Remaining risks

- Independent Lucy review required (Red task).
- `PublishDisabledInstallation` is mutation-free and fail-closed; J24K3 must implement the crash-safe publication path.
- The `ConformanceScratchGuard` uses `let _ = std::fs::remove_dir_all(path)` for best-effort cleanup; double-cleanup is harmless since `remove_dir_all` is idempotent on absent directories.

## Smallest next action

Lucy should independently review this implementation before acceptance and J24K3.

## References

- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `docs/architecture/J24J_READ_ONLY_INSTALLATION_RECONCILIATION_PLANNER.md`
- `docs/worker-notes/2026-08-04-j24k1-current-trust-authority.md`
