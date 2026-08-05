# Worker Note

Task: `J24K3d1 correction - planner path-safety regressions and complete verification`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `96902b715cbb8d62aad12d468a474ae03abfaaed`
Original implementation checkpoint: `351a2782b59d1b08c5529bd18caf8a7fa29cde6b`
Implementation checkpoint: `208ef6f0cbf29c5933cc72a8c93ca87973a3f733`
Correction implementation checkpoint: `208ef6f0cbf29c5933cc72a8c93ca87973a3f733`
Verification checkpoint: `aa8720b5758296ea550cb119354b6073908664a6`

## Requested outcome

Correct the remaining independent-review evidence gaps in the otherwise sound J24K3d1 read-only recovery planner.

The production planner ordering and disposition-specific proof boundaries are accepted. No production redesign is requested.

Add direct planner-entry regressions for destination reparse state and unsafe or missing registry roots, correct the two mistyped checkpoint SHAs in the completed documentation, and complete the verification matrix without excluding a failed test.

## Changes made

- Added a private `m3_lifecycle.rs` test helper that retries only Windows teardown `PermissionDenied` or `DirectoryNotEmpty` errors for a bounded two-second deadline, treats `NotFound` as success, and panics with the complete path and final error otherwise.
- Replaced only the final teardown call in `m3_malformed_and_interrupted_conformance_fail_without_retry_or_install`.

## Decisions and assumptions

- DeepSeek Pro remains the implementation model because this is one bounded regression-test, verification, and evidence correction.
- The planner production module should remain unchanged unless a new production-entry test demonstrates a real defect.
- Existing lower-level path-safety tests are not a substitute for exercising `plan_installation_recovery` directly.
- Missing Nextest is a tooling block, not permission to mark the task complete. Do not install software automatically.
- Workers record actual implementation and verification checkpoints only. No final remote tip is committed inside the branch.
- Lucy authorised this one test-only Windows teardown correction after the exact serial rerun also failed. No production, assertion, child-shutdown, or conformance behavior is changed.

## Evidence

- Focused Nextest on Windows: 28 passed, 0 failed, 0 retries; 2 Unix symlink tests skipped by platform gate.
- New direct planner-entry coverage: destination junction, missing opened install root, and record-root junction.
- Named regression suites passed, including the first full serial verification through `j24c_plug_disable_cli` after its exact failing test passed on serial rerun.
- Full serial `just verify` failed on `m3_malformed_and_interrupted_conformance_fail_without_retry_or_install` at `tests/m3_lifecycle.rs:1009` with `Os { code: 5, kind: PermissionDenied, message: "Access is denied." }`.
- The exact permitted serial rerun of that test failed with the same error.
- The bounded teardown correction is intentionally not best-effort: unexpected errors fail immediately and persistent contention fails after two seconds with the final error.
- After the correction, the exact failing test passed serially: 1 passed, 0 failed, 12 filtered out.
- Ten exact serial repetitions of the failing test passed: 10/10, each 1 passed and 0 failed.
- Focused Nextest passed with 28 tests passed, 0 failed, 0 retries, and 2 Unix symlink tests skipped by the Windows platform gate.
- All named J24K3d1/J24K3a-J24K3c4/J24K3b/J24K2/J24J and `m3_lifecycle` regression commands passed.
- Full serial `just verify` passed with zero failures.
- Cargo.lock SHA-256 remained `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB`.

## Discoveries

- GitHub shows the actual original implementation checkpoint is `351a2782b59d1b08c5529bd18caf8a7fa29cde6b`, not the SHA recorded in the completed packet.
- GitHub shows the completion-candidate commit previously tested was `b76691c5b97bd1b3a82de824535365fec4676c20`, not the SHA recorded in the completed packet.
- The current 25 direct tests contain no symlink, junction, reparse, or `unsafe_store_path` planner-entry fixture.
- The reported full verification excluded one failing `m3_lifecycle` test, so it does not satisfy the frozen acceptance criterion.

## Remaining risks

- Windows junction fixtures must be privilege-safe and deterministic, matching the accepted J24K3c2/J24K3c4 pattern.
- The known `m3_lifecycle` handle-contention failure must pass on an exact serial rerun and full serial verification must finish green before handoff.

## Smallest next action

Run the ten exact serial repetitions, then the complete packet verification matrix. If bounded cleanup still fails, record the complete path, final OS error, and remaining `m3_fixture_provider` processes, then stop BLOCKED without broadening the correction.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-05-j24k3d1-validated-recovery-plan.md`
- `tethers-0.1/host-rust/src/installation_recovery_plan.rs`
- `tethers-0.1/host-rust/src/installation_recovery_plan_tests.rs`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
