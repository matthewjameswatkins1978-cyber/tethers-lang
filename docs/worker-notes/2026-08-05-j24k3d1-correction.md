# Worker Note

Task: `J24K3d1 correction - planner path-safety regressions and complete verification`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `96902b715cbb8d62aad12d468a474ae03abfaaed`
Original implementation checkpoint: `351a2782b59d1b08c5529bd18caf8a7fa29cde6b`
Correction implementation checkpoint: `9a48be4e08d06e636cb53e21c9686ef65fbca8c8`
Verification checkpoint: `WORKTREE`

## Requested outcome

Correct the remaining independent-review evidence gaps in the otherwise sound J24K3d1 read-only recovery planner.

The production planner ordering and disposition-specific proof boundaries are accepted. No production redesign is requested.

Add direct planner-entry regressions for destination reparse state and unsafe or missing registry roots, correct the two mistyped checkpoint SHAs in the completed documentation, and complete the verification matrix without excluding a failed test.

## Changes made

None yet.

## Decisions and assumptions

- DeepSeek Pro remains the implementation model because this is one bounded regression-test, verification, and evidence correction.
- The planner production module should remain unchanged unless a new production-entry test demonstrates a real defect.
- Existing lower-level path-safety tests are not a substitute for exercising `plan_installation_recovery` directly.
- Missing Nextest is a tooling block, not permission to mark the task complete. Do not install software automatically.
- Workers record actual implementation and verification checkpoints only. No final remote tip is committed inside the branch.

## Evidence

- Focused Nextest on Windows: 28 passed, 0 failed, 0 retries; 2 Unix symlink tests skipped by platform gate.
- New direct planner-entry coverage: destination junction, missing opened install root, and record-root junction.

## Discoveries

- GitHub shows the actual original implementation checkpoint is `351a2782b59d1b08c5529bd18caf8a7fa29cde6b`, not the SHA recorded in the completed packet.
- GitHub shows the completion-candidate commit previously tested was `b76691c5b97bd1b3a82de824535365fec4676c20`, not the SHA recorded in the completed packet.
- The current 25 direct tests contain no symlink, junction, reparse, or `unsafe_store_path` planner-entry fixture.
- The reported full verification excluded one failing `m3_lifecycle` test, so it does not satisfy the frozen acceptance criterion.

## Remaining risks

- Windows junction fixtures must be privilege-safe and deterministic, matching the accepted J24K3c2/J24K3c4 pattern.
- The known `m3_lifecycle` handle-contention failure must pass on an exact serial rerun and full serial verification must finish green before handoff.

## Smallest next action

Apply only the correction packet, complete all required verification, and return the branch for independent review. Do not merge.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-05-j24k3d1-validated-recovery-plan.md`
- `tethers-0.1/host-rust/src/installation_recovery_plan.rs`
- `tethers-0.1/host-rust/src/installation_recovery_plan_tests.rs`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
