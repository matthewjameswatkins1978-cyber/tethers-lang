# Worker Note

Task: `J04 effective policy resolution`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Copilot (isolated worktree)`

Status: `COMPLETE`

Base commit: `643c6ed40e3e8a167afd53eca2c98597c0aa8f24`

Implementation checkpoint: `WORKTREE`

## Requested outcome

Implement a deterministic effective-policy resolver in `policy.rs` returning
exactly one of `allow`, `ask`, `deny`, or `unavailable` for a proposed Action,
following the complete fail-closed precedence frozen by J03/J03a/J03b in
`docs/DECISIONS.md`, excluding the Ask-approval-resume step reserved for J05.

## Changes made

- `tethers-0.1/host-rust/src/policy.rs`: added `ScopeAssessment`,
  `ProposedAction`, `PolicyReason`, `PolicyEvaluation`, and
  `evaluate_effective_policy()`. The existing `PermissionDecision`,
  `evaluate_permission`, and `evaluate_permission_resolved` are unchanged;
  `PolicyEvaluation` carries the decision and a distinct reason as a new,
  additive structure so no existing caller (including `dispatch.rs`) required
  any change. Added 19 focused tests covering every required-behaviour branch.
- `tethers-0.1/host-rust/src/validation.rs`: extracted the existing schema
  checker into a new public `validate_against_schema()`; `validate_output()`
  is now a thin wrapper over it, unchanged for existing callers. Used by
  `evaluate_effective_policy()` to validate Action arguments against the
  manifest's `input_schema`.
- `tethers-0.1/host-rust/src/main.rs`: added `extract_proposed_action()`,
  which reads `evaluation_id`, `plan.id`, the single Action's `action_id`,
  `capability`, bridge pins, and `arguments` from the already-produced engine
  response. Replaced the `evaluate_permission_resolved` call site with
  `evaluate_effective_policy`, passing `ScopeAssessment::WithinScope` as an
  explicit placeholder (no real binding-specific assessor exists yet; see
  Remaining risks). `authorise_and_execute`, `verify_action_bridge_pins`, and
  `dispatch.rs` are unchanged.

## Decisions and assumptions

- Every capability resolved through this host is bridge-backed (the only
  `BindingKind` is `Mcp`), so all three bridge pins are required for every
  proposed Action; a missing pin is treated as a malformed Action identity
  and denied before resolution.
- The Action's pinned `bridge_provider_identity` is passed as
  `resolve_capability`'s `expected_provider`, so a provider-identity/binding
  mismatch surfaces as the resolver's existing `ProviderIdentityMismatch` ->
  `Unavailable`, reusing existing resolution logic rather than adding a new
  comparison.
- `PermissionDecision` was kept completely unchanged; the distinct reason is
  returned alongside it in `PolicyEvaluation` rather than embedded in the
  enum, so `dispatch.rs`'s `prepare_and_record` match arms required no edits.
- `main.rs`'s demo call site supplies `ScopeAssessment::WithinScope` as an
  explicit, commented placeholder for its own trusted demo capability. This
  is not a generic argument-scope rule; a later binding/adapter task must
  supply a real assessor before any non-demo structured-scope capability can
  be pre-authorised.

## Evidence

- `cargo fmt --check`: passed (no output).
- `cargo test` (from `tethers-0.1/host-rust`): passed, `316 passed; 0 failed`.
- `scripts/check-fixtures.ps1`: passed, `46 JSON files, 30 JSONL files`.
- `scripts/test-engine.ps1`: passed all fixture cases plus deterministic
  repeat.
- `scripts/test-mcp-transcripts.ps1`: passed, `15 cases`.
- `scripts/test-host-denial.ps1`: passed — `deny` posture still produces
  `execution_status: denied` end to end through the new policy path.
- `scripts/test-host-execution-failure.ps1`: passed — `allow` posture with a
  failing executor still produces one durable `action_started`/`action_failed`
  and zero `action_completed`.
- `scripts/demo.ps1`: passed — full round trip still reaches
  `action_completed` with `allow` posture through the new policy path.
- `opam exec -- dune build` (OCaml engine, untouched): passed.
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`: passed
  after this note and the `COMPLETE` status update.
- `git diff --check`: passed, whitespace clean.

## Discoveries

- `main.rs` previously called `evaluate_permission_resolved`, which cannot
  itself return `Unavailable` (it never calls the resolver); the demo's early
  `resolver::resolve_capability(...).map_err(...)?` call was the only path
  that could observe an unresolved capability, and it aborted the process
  rather than producing a Trail-visible `Unavailable` decision. The new
  `evaluate_effective_policy` resolves internally and can return
  `Unavailable` through the ordinary decision path; the early resolution in
  `main()` is unchanged and still used for the dispatch/executor setup.
- The frozen decision's phrase "capability name/version that does not exactly
  match the resolved capability" (Required behaviour 1) has no separate
  code path here: `evaluate_effective_policy` resolves using the Action's own
  `capability_name`/pinned version, so a mismatch against the trusted store's
  admitted identity surfaces as `NoAdmittedManifest` -> `Unavailable`
  (Required behaviour 4), not as a distinct Deny branch. This is noted for
  the next reviewer rather than silently assumed.

## Remaining risks

- Manifest `permission_scope` (`path_prefix`/`repository`/`calendar`)
  argument-level enforcement is still not implemented anywhere; J03b's
  host-owned `ScopeAssessment` is a policy-side boundary only. A later
  binding/adapter task must supply a real assessor before a structured-scope
  capability besides the trusted demo can be safely pre-authorised.
  `main.rs`'s `WithinScope` placeholder is explicit and commented, not hidden.
  Structured-scope Actions must not be treated as authorised in a real
  provider flow until that assessor exists.
- The Ask-approval-resume step (J03/J03a precedence step 4) remains
  unimplemented by design; every current Ask outcome is plain `Ask`. J05 must
  add the one-shot approval-consumption path without reordering this
  precedence.
- This task was implemented directly in the primary checkout rather than a
  separate physical `git worktree`, since only one implementation owner was
  active in this session; no parallel agent was concurrently working.

## Smallest next action

Route this evidence for one bounded Codex review (capability-trust/permission
semantics), then compile J05's frozen one-shot Ask-resume contract before any
further implementation.

## References

- `docs/DECISIONS.md` — J03, J03a, and J03b sections
- `docs/CAPABILITY_BRIDGE.md`
- `tethers-0.1/host-rust/src/policy.rs`
- `tethers-0.1/host-rust/src/validation.rs`
- `tethers-0.1/host-rust/src/main.rs`
