# Worker Note

Task: `J24K3b correction - record validation ordering and final verification`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `READY`
Base commit: `WORKTREE`
Implementation checkpoint: `WORKTREE`

## Requested outcome

Apply one bounded correction to the otherwise complete J24K3b classifier: validate every present installed record immediately after validating the intent and before applying any recovery-matrix row. Add direct invalid-record edge coverage, repair the control packet structure, and complete the required full verification.

## Changes made

No correction implementation has started. The accepted review found that the classifier validates a record only in the destination-plus-record row, despite the packet and handoff stating that every present record is validated before matrix classification. The earlier handoff also ran `just test-rust` rather than the required full `just verify`, and the packet checker exposed a packet-authoring error.

## Decisions and assumptions

- Keep the existing four dispositions and recovery matrix unchanged.
- Preserve `installation_intent_invalid` for invalid intent and `installation_recovery_conflict` for invalid, unequal, or contradictory installed-record state.
- Validation remains pure and performs no I/O or mutation.
- The control-packet numbering defect belongs to the coordinator packet, not to the implementation design.

## Evidence

The reviewed branch tip before this correction scaffold was `ef13e1d5a83ea8adea59aafc1557c3e70f69ba6f`. At that tip, 14 direct J24K3b tests passed, focused Nextest passed with zero retries, J24K3a/J24K2/J24J/M3 regressions passed, and Cargo.lock remained unchanged. The packet checker failed because nested numbered lists produced 23 required items against 18 acceptance criteria. Full `just verify` therefore did not pass.

## Discoveries

The production function checks `staging_present && destination_present` before validating a supplied record and validates a record only in the destination-plus-record match arm. This contradicts the explicit contract that intent validation is followed by validation of any present installed record before the matrix is classified.

## Remaining risks

Later J24K3 packages still own filesystem observation, destination verification, installed-root audit, recovery mutation, and executor wiring. None of those concerns belongs in this correction.

## Smallest next action

OpenCode should apply the one production ordering fix, add direct invalid-record tests for broad conflict rows, run the corrected packet checker, run full `just verify`, and return the branch for independent review.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `tethers-0.1/host-rust/src/installation_recovery.rs`
- `tethers-0.1/host-rust/src/installation_recovery_tests.rs`
