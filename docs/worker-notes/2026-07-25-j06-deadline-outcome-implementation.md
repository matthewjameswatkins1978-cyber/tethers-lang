# J06 monotonic deadline and truthful outcome implementation

Task: `J06 monotonic deadline and truthful outcome classification`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `Codex`
Status: `COMPLETE`
Base commit: `95976fdac466db61aaa1a88b5a1f0e8574101526`
Implementation checkpoint: `WORKTREE`

## Requested outcome

Implement the authoritative J06 deadline and outcome boundary without J07,
retry, compensation, protocol, planner, or manifest changes.

## Changes made

Added host-owned monotonic deadline classification, typed adapter diagnostics,
redacted durable reason codes, uncertain Result Anchors, and focused tests.
The Red correction makes legacy string errors uncertain by default and passes a
host-computed remaining monotonic duration into every typed provider call.

## Decisions and assumptions

The fresh implementation branch starts from accepted `origin/main` at
`5198f00d6e4905fe7bc2f90290d9962570307d7a`; the packet's frozen historical
base remains the design commit recorded above.

## Control-plane self-repair

The accepted J06 design packet was landed before this implementation at
`5198f00`; the packet's historical design-base field remains `95976fd`.
No semantic packet change was required before implementation.

## Evidence

The 48-case mapping below and the complete required verification sequence are
the objective evidence for this worktree checkpoint.

## Discoveries

The Red review correctly identified that an untyped string error cannot prove
provider-declared failure and that checking a clock only after synchronous
execution cannot make an adapter deadline-aware.  The corrected compatibility
seam maps legacy errors to `NoFinalResponse`; only an override can return
`ExplicitProviderError`.  The typed seam receives the remaining monotonic
duration immediately before invocation.

## Remaining risks

The demo has no concrete process transport executor yet; production adapters
must override `execute_classified` to report their observed transport class.

## Smallest next action

Independent Red review of this uncommitted J06 worktree.

## References

`docs/J06_DEADLINE_OUTCOME_DESIGN.md`; `docs/CURRENT_CLINE_TASK.md`;
`docs/J05_EXACT_ASK_APPROVAL_DESIGN.md`; `docs/DECISIONS.md`; and
`docs/CAPABILITY_BRIDGE.md`.

## Design-case evidence matrix

| Case | Evidence |
| --- | --- |
| 1 | `tests::j06_deadline_before_invocation_is_unattempted_without_provider_outcome_or_anchor` proves durable intent exists before the deadline check. |
| 2 | `tests::j06_elapsed_before_authorisation_does_not_consume_execution_deadline` proves elapsed injected planning time before intent leaves the provider a full `10s` remaining duration. |
| 3 | Same test covers approval waiting before intent using the same full remaining duration. |
| 4 | `outcome::ProductionMonotonicClock` uses `std::time::Instant`; `outcome::tests::deterministic_clock_advances_only_when_directed` exercises its deadline abstraction. |
| 5 | `outcome::tests::deterministic_clock_advances_only_when_directed` and `tests::j06_elapsed_before_authorisation_does_not_consume_execution_deadline`. |
| 6 | `tests::j06_deadline_before_invocation_is_unattempted_without_provider_outcome_or_anchor` proves the final remaining-duration check precedes presentation. |
| 7 | Same test (`calls == 0`, no `action_started`). |
| 8 | Same test (`outcome_entries.is_empty()`). |
| 9 | Same test (`result_anchor` absent). |
| 10 | `tests::authorise_and_execute_writes_succeeded_outcome`. |
| 11 | `tests::authorise_and_execute_writes_failed_outcome` uses `FailingExecutor`'s explicit typed provider-error override. |
| 12 | `tests::missing_required_output_field_fails_validation`. |
| 13 | `tests::j06_response_observed_at_deadline_is_uncertain_even_when_provider_succeeds`. |
| 14 | `tests::j06_post_invocation_transport_ambiguities_are_uncertain_and_redacted` table case `ProcessLost`. |
| 15 | Same table case `ResponseMalformed`. |
| 16 | Same table case `ResponseTruncated`. |
| 17 | Same table case `ProtocolInterrupted`. |
| 18 | Same table case `NoFinalResponse`, now returned through the deadline-aware typed executor contract. |
| 19 | `tests::j06_response_observed_at_deadline_is_uncertain_even_when_provider_succeeds`. |
| 20 | `tests::j06_deadline_before_invocation_is_unattempted_without_provider_outcome_or_anchor` proves zero remaining duration creates neither invocation nor `action_started`; identity-mismatch regressions prove the other pre-boundary case. |
| 21 | `tests::no_result_anchor_on_unavailable`. |
| 22 | `tests::j06_deadline_before_invocation_is_unattempted_without_provider_outcome_or_anchor`. |
| 23 | `tests::no_result_anchor_on_deny`, `no_result_anchor_on_ask`, `no_result_anchor_on_unavailable`, and J06 pre-invocation test. |
| 24 | `tests::authorise_and_execute_writes_succeeded_outcome`. |
| 25 | `tests::authorise_and_execute_writes_failed_outcome`. |
| 26 | `tests::j06_post_invocation_transport_ambiguities_are_uncertain_and_redacted`. |
| 27 | `tests::outcome_write_failure_after_success_preserves_status_and_audits`. |
| 28 | `tests::outcome_write_failure_after_failure_preserves_status_and_audits`. |
| 29 | `tests::j06_outcome_audit_failure_keeps_uncertainty_but_withholds_anchor`. |
| 30 | `tests::outcome_write_audit_failure_withholds_result_anchor`, `outcome_write_audit_failure_after_executor_error_withholds_failed_anchor`, and J06 uncertainty audit test. |
| 31 | The three case-30 tests each prove one provider call and no second call. |
| 32 | `tests::j06_response_observed_at_deadline_is_uncertain_even_when_provider_succeeds`. |
| 33 | `tests::j06_outcome_audit_failure_keeps_uncertainty_but_withholds_anchor`. |
| 34 | `outcome::tests::redaction_is_stable_bounded_and_contains_no_private_diagnostic`. |
| 35 | Same redaction test and J06 transport table's stable codes. |
| 36 | `tests::j06_outcome_audit_failure_keeps_uncertainty_but_withholds_anchor` proves injected private text is absent from response/durable data. |
| 37 | `tests::j05_production_seam_consumes_exact_approved_fixture_before_intent`. |
| 38 | `approval::tests::consume_is_one_shot_and_requires_full_proof` plus `tests::authorise_and_execute_writes_failed_outcome`. |
| 39 | `approval::tests::consume_is_one_shot_and_requires_full_proof` plus J06 transport ambiguity table. |
| 40 | `tests::j05_authorisation_trail_write_failures_leave_no_usable_approval` and J06 audit-failure tests. |
| 41 | `tests::j06_deadline_before_invocation_is_unattempted_without_provider_outcome_or_anchor`: no retry path is entered after durable intent. |
| 42 | `dispatch::tests::file_trail_writes_durable_intent_and_outcome`. |
| 43 | `tests::j06_post_invocation_transport_ambiguities_are_uncertain_and_redacted` records durable `uncertain`; no reconstruction/retry code exists. |
| 44 | J06 focused tests assert a single call; no host retry loop exists. |
| 45 | J06 focused tests and production dispatch contain no compensation call. |
| 46 | `tests::no_result_anchor_on_deny`, `no_result_anchor_on_ask`, `no_result_anchor_on_unavailable`, `no_result_anchor_on_provider_identity_mismatch`, and `no_result_anchor_on_intent_write_failure`. |
| 47 | `policy::tests::effective_policy_denies_scope_not_established_before_local_allow`. |
| 48 | Required verification commands recorded in the final section when complete. |

## Implementation notes

- `outcome.rs` owns the monotonic clock, deterministic test clock, typed
  provider diagnostics, and pure redaction boundary.
- Deadline starts only after `prepare_and_record` returns a valid readiness
  token.  The invocation boundary is immediately before `execute_classified`.
- Attempted outcomes are durable before standard Result Anchors.  Audit-write
  failure retains the response classification but withholds the Anchor.
- No retry, compensation, replay, protocol, planner, manifest, J07, or safety
  branch changes were added.

## Verification

- `cargo fmt --check` — PASS.
- `cargo test` — PASS, 333 tests passed, 0 failed, including the Red
  regressions for legacy-string uncertainty and remaining-duration delivery.
- `pwsh -NoProfile -File scripts/check-fixtures.ps1` — PASS, 46 JSON and 30
  JSONL fixtures valid.
- `pwsh -NoProfile -File scripts/test-engine.ps1` — PASS, all listed engine
  fixture cases plus deterministic repeat.
- `pwsh -NoProfile -File scripts/test-mcp-transcripts.ps1` — PASS, 15 cases.
- `pwsh -NoProfile -File scripts/test-host-denial.ps1` — PASS.
- `pwsh -NoProfile -File scripts/test-host-execution-failure.ps1` — PASS.
- `pwsh -NoProfile -File scripts/demo.ps1` — PASS; unassessed structured
  scope remains denied before execution.
- `opam exec -- dune build` — PASS.
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` —
  PASS (`control-v1/COMPLETE`, base `95976fd`, HEAD `5198f00`).
- `git diff --check` — PASS.
- `git status --short --branch`, `git diff --stat`, and complete-diff
  inspection — completed; the six files listed in the final handoff are the
  bounded J06 worktree change.
