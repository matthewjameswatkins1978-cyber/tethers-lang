# Current Implementation Task

Control contract: `1`
Task packet: `F8-D5+D6+D10 — Dead Exact-Approval Translation/Resume Layer`
Owner: `Codex`
Status: `COMPLETE`
Task colour: `Amber`
Route: `Codex removed the classified dead exact-approval translation/resume layer and verified it with the restored local engine output`
Worker note: `docs/worker-notes/2026-08-09-f8-d5-d6-d10-exact-approval-cleanup.md`
Base branch: `foundation/f8-d3-event-admission-probe-cleanup`
Base commit: `17b60df43b6c32ec8040952e4f7b1a99eb16b1d3`
Implementation branch: `foundation/f8-d5-d6-d10-exact-approval-cleanup`
Implementation checkpoint: `5a728d6956a288cafa69a84d0be50f7ffafaa1ea`
OCaml switch path: `N/A`
Rust toolchain: `1.97.1`
Rust change class: `RUST`

## Objective

Remove the abandoned exact-approval translation/resume subsystem D5, D6, and
D10 while preserving all live approval, Trail, replay, policy, and dispatch
contracts at their surviving seams.

## Relevant background and existing behaviour

The packet's established structural chain is `HumanApprovalDecision` exclusively
to `record_human_approval_decision`, then `ApprovalStore::decide`. All callers
of `resume_and_execute_exact_approval` are tests whose setup uses D5/D6.
`resume_and_execute_exact_approval_with_authority` is a separately suppressed,
deliberately injectable exact-approval execution seam and is not part of D10.

The accepted D4 closeout is the predecessor. It left 11 production-library
warnings (D5-D15), with a test baseline of 1592 passed, 0 failed, 2 skipped.

## Required behaviour

1. Classify and remove `HumanApprovalDecision` and
   `record_human_approval_decision` without recreating either wrapper under a
   new name.
2. Classify and remove `resume_and_execute_exact_approval` without removing or
   weakening `resume_and_execute_exact_approval_with_authority`.
3. Migrate affected tests to `ApprovalStore::decide`, `precheck_exact_approval`,
   or `resume_and_execute_exact_approval_with_authority` according to the
   behavioural contract each test proves.
4. Audit old Trail-write-failure and invalidation tests and retain every genuine
   invariant at an appropriate surviving seam.
5. Preserve approval trail entries, request/precheck semantics, exact
   `ApprovalProof`, fresh policy re-evaluation, live execution machinery,
   non-dispatch on denial/cancellation, stale/mismatched rejection, and live
   Trail behaviour.
6. Reduce the intended production-library warning count from 11 to 8 while
   leaving D7-D9 and D11-D15 unresolved.
7. Run focused checks during implementation and one final `just verify-agent`
   umbrella regression after the implementation checkpoint.

## Relevant components

### AUTHORISED PATHS
- `tethers-0.1/host-rust/src/application.rs` — remove D5/D6/D10 and migrate
  inline behavioural tests to surviving seams

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-f8-d5-d6-d10-exact-approval-cleanup.md`

## Frozen decisions and invariants

- D5, D6, and D10 are one abandoned test-facing subsystem only if live reference
  evidence continues to support that classification.
- Preserve `approval_trail_entry`, `request_exact_approval`,
  `precheck_exact_approval`, `ApprovalStore`, exact `ApprovalProof` semantics,
  fresh-policy re-evaluation, `authorise_and_execute_inner`, and live execution
  machinery.
- Preserve denial/cancellation non-dispatch, stale/mismatched approval rejection,
  and live Trail semantics.
- `resume_and_execute_exact_approval_with_authority` is deliberately distinct:
  retain it and independently preserve its injectable test/live-architecture
  role.
- No `#[allow(dead_code)]` added merely to silence warnings. No D7-D9 or
  D11-D15 resolution, suppression, or adjacent refactoring.

## Acceptance criteria

1. `HumanApprovalDecision` and `record_human_approval_decision` are removed and
   post-change Rust searches have zero matches — proven by `rg`.
2. `resume_and_execute_exact_approval` is removed while the `_with_authority`
   seam remains — proven by exact Rust searches and diff inspection.
3. Each migrated test directly targets the retained state, precheck, or
   execution/replay-authority seam that owns its original contract — proven by
   focused test results and test diff inspection.
4. Trail-write-failure and invalidation invariants remain directly tested at a
   surviving seam — proven by named focused tests.
5. Required approval, policy, replay, dispatch, and Trail invariants remain
   unchanged — proven by focused tests and complete source diff inspection.
6. `cargo check` reports 8 intended remaining production-library warnings and
   no D5/D6/D10 warning — proven by captured compiler output.
7. D7-D9 and D11-D15 remain unresolved and no dead-code suppression is added —
   proven by source searches and diff inspection.
8. Formatter output is limited to the authorised Rust path and whitespace checks
   pass — proven by immediate formatter-diff inspection and `git diff --check`.
9. The packet checker and one final `just verify-agent` umbrella regression pass
   against the implementation checkpoint — proven by command output.
10. The closeout contains only packet and worker-note changes after the
    implementation checkpoint, and the completed branch is normally pushed with
    matching remote SHA and clean status — proven by Git range/status evidence.

## Required verification

1. `rg "HumanApprovalDecision" --type rust`,
   `rg "record_human_approval_decision" --type rust`, and
   `rg "fn resume_and_execute_exact_approval\\(" --type rust` after removal.
2. Targeted `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked` after the structural change.
3. Named focused exact-approval tests selected from the affected test module.
4. `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all` before the
   implementation checkpoint, followed immediately by an inspection of its
   diff; stop if it changes an unauthorised Rust path.
5. `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`
   and `git diff --check`.
6. `cargo clippy --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked` because `just verify-agent` does not run Clippy.
7. `just verify-agent` once, after the implementation checkpoint. It covers the
   packet checker, formatter check, full `cargo check`, full `cargo test`, Rust
   toolchain check, dependency licences/bans/sources, dependency advisories, and
   full nextest. It does not cover Clippy or release builds.
8. `git diff --check`, complete range diff inspection, packet checker, remote
   branch SHA equality after normal push, and `git status --short --branch`.

## Formatting and checkpoint sequence

The only authorised Rust path is `tethers-0.1/host-rust/src/application.rs`.
Before the implementation checkpoint run the exact mutating formatter command
listed above and inspect its immediate diff. STOP if rustfmt changes any file
outside that authorised Rust path; do not absorb unrelated formatting debt.

## Completion and publication

After the documentation-only closeout commit, normally push this branch to
`origin`. Report the remote branch, full remote HEAD SHA, local-equals-remote
confirmation, and clean `git status --short --branch`. No force-push, merge,
rebase, direct `main` update, or pull request is authorised.

## Forbidden changes

- No D7-D9 or D11-D15 cleanup or suppression.
- No removal of `resume_and_execute_exact_approval_with_authority`.
- No weakening or removal of Trail-write-failure, invalidation, stale/mismatched
  rejection, cancellation, denial, replay, fresh-policy, or dispatch evidence.
- No OCaml, fixture, build, protocol, dependency, CI, lint-policy, merge,
  amend, tag, force-push, direct `main` update, or pull-request changes.

## Stop conditions

STOP this job if a target has a real production caller, deleting a wrapper
removes the only representation of a live contract, test migration weakens
behavioural evidence, an architectural choice is required, formatter output
leaves the authorised Rust path, verification is untrustworthy, or a second
materially similar implementation attempt fails. Return exact evidence and one
smallest unresolved question. Do not begin Job B without an accepted Job A tip.

## Expected pre-existing changes

None.
