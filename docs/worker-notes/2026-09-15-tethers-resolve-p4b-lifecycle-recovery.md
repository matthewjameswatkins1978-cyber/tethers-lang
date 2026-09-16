# Tethers x Resolve01 P4b worker note

Task: `TETHERS x RESOLVE01 / P4b - End-to-End Guarded Lifecycle & Recovery`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `63aa8e21bfa359ad444d8b32b3007b398fa9002e`

Implementation checkpoint: `00361fd89a381ec48500ff0003e998073e18a347`

Branch: `codex/tethers-resolve-p4b-lifecycle-recovery`

Worktree: `D:\The Next Thing\Tethers Lang - Resolve P4b Lifecycle Recovery`

## Requested outcome

Provide direct evidence that the accepted Tethers/Resolve guarded lifecycle
recovers fail-closed, limits provider effects to one per execution identity,
does not retry admission or providers automatically, and redelivers only a
durable exact outcome.

## Changes made

- Added the test-only P4b lifecycle composition module under the existing
  application test module.
- Added deterministic test-only Trail fault injection for durable intent,
  admission and outcome evidence failures.
- Added the P4b architecture note and replaced the active task packet with an
  explicit P4b implementation scope.

## Decisions and assumptions

The accepted replay, admission, provider, Trail and outcome-journal authorities
already provide the required semantics, so P4b composes them rather than
introducing another recovery state machine. The P4b crash inventory uses safe
deterministic test seams; it does not add unsafe process-kill hooks.

## Evidence

Focused P4b verification passes 6 tests, including the corrected durable-intent
fault case, guarded happy path, replay restart refusal, provider
failed/uncertain outcomes, exact delivery recovery after journal reopen, and
the twelve-entry crash-boundary inventory. Relevant P1 guard tests pass 8,
P2 application tests pass 4, P4a identity continuity passes 1, native replay
recovery passes 8, and P4 conformance passes 2. Cargo formatting, locked
check/build/debug-release build, full `just verify`, warning ratchet and
`git diff --check` pass. `just verify` reports PASS for the current engine,
OCaml, Rust/cross-language, protocol fixture, MCP transcript and compatibility
corpus suites.

The generated machine report is `verification/tethers-verification.json` with
schema `tethers.verify/1`, source commit
`00361fd89a381ec48500ff0003e998073e18a347`, current-engine provenance and no
first meaningful failure. The report was generated before closeout-doc edits,
so its dirty flag is expected for that run and it is not release evidence for
the final documentation commit.

Independent second opinion: Gemini `gemini-3.8-flash`, high thinking level,
completed. It found no high-severity defect and gave a conditional go. Its two
medium evidence limitations are recorded here: the matrix uses safe in-process
fault seams rather than OS-level kill/power-loss testing, and the post-provider
pre-outcome-persistence crash window is not a new live two-process proof. Those
are bounded follow-up evidence requirements, not claims of production crash
coverage.

## Discoveries

The accepted host path already orders durable intent, guard evidence, replay G1,
provider execution, durable outcome and delivery. A prior focused test had a
real weakness because its injected executor was not connected to preparation;
the P4b helper now passes the counting executor through the production guarded
boundary. The synthetic admission proof must use the fixture action identity
`action-1` or P2 correctly returns indeterminate. `RecordingTrail` and the P4b
module are both behind `cfg(test)`, confirmed by successful non-test debug and
release builds.

## Remaining risks

The live two-process Tethers/Resolve restart smoke was not run because no
Resolve service or Firestore emulator was listening in the available
environment; no process or port prerequisite was present. Existing strict
Clippy findings and the duplicate-target warning are outside P4b; the warning
ratchet confirms no new warning was introduced.

## Additional evidence

- No-intent and stale replay recovery use the existing replay authority and
  produce no admission/provider call.
- Admission rejection, indeterminacy and adapter error each stop before the
  provider; no automatic admission retry exists.
- Delivery failure followed by journal reopen sends the exact durable outcome,
  then `Delivered` remains terminal without another adapter/provider call.
- P4b makes no direct Resolve call; accepted P3 transport and Resolve S3
  contract remain the live cross-system boundary.

## Smallest next action

Run the full relevant Rust and repository verification, then record the exact
implementation checkpoint and final packet evidence before publication.

## References

- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P0_SEAM_AUDIT.md`
- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P1_PREPARATION_PROOF.md`
- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P2_GUARD_ADMISSION.md`
- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P3_LIVE_TRANSPORT.md`
- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P4_OUTCOME_DELIVERY.md`
- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P4B_LIFECYCLE_RECOVERY.md`
- `C:\dev\resolve-ai\docs\TETHERS_GUARD_PROTOCOL.md`
