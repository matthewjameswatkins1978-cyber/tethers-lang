# Worker Note: C3-D2 Adversarial Bounded Concurrency Design Review

Task: `C3-D2 — Adversarial Bounded Concurrency Design Review`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `C3-D2 Independent Design Reviewer`

Status: `COMPLETE`

Base commit: `8268fb9e0284b7e9f2268279b4dea5d37f01a9bc`

Implementation checkpoint: `440e9e2e40e0ea9c8db70fe39346224b4705465c`

## Requested outcome

Independently attack the proposed C3 bounded-concurrency design against the
actual merged runtime, correct documentation where the code or frozen A3
semantics prove the current design text wrong or internally inconsistent, and
deliver a rigorous adversarial design review without redesigning C3 or changing
production code.

## Changes made

- `docs/concurrency/C3_BOUNDED_CONCURRENCY_DESIGN.md`:
  - Corrected Section 2 (G0/G1 recovery semantics) to distinguish effect certainty
    (G0 present + G1 absent proves invocation boundary was never armed, so no provider
    effect occurred) from replay authority (existing replay recovery policy remains
    authoritative, mapping recovered `IntentRecorded` strictly to
    `ReplayDispatchResult::RequiresManualResolution`).
  - Unified Section 4 and Section 7 (Slot release / Active-count release boundary)
    to establish a single unambiguous terminal boundary: capacity is released only
    after the complete coordinator Stage C boundary (`execute_boundary_invoke_only`
    including durable `OutcomeEntry` write, G2 `publish_terminal`, presentation/response
    updates, and Result Anchor writing) and `GroupMemberState` transition to
    `Terminal`.
  - Clarified Section 8C (Worker channel failure) to match actual runtime behavior
    where workers catch internal panics in `worker_invoke_inner` via `catch_unwind`,
    and unexpected channel disconnection halts coordinator reception and fails the join
    closed with `AuditFailed`.
  - Updated Section 14 (Proof Matrix Item 4) to explicitly state recovered
    `IntentRecorded` maps to `RequiresManualResolution`.
- `docs/CURRENT_CLINE_TASK.md`:
  - Updated packet to C3-D2 hostile design review specification.

## Decisions and assumptions

- Effect certainty is distinct from replay authority: while `G0=yes, G1=no` is
  unambiguous pre-invocation evidence that no provider effect occurred, existing
  replay recovery policy in `replay_runtime.rs` maps `IntentRecorded` to
  `RequiresManualResolution`. The design strictly respects this existing policy.
- The Stage C completion boundary is singular: `execute_boundary_invoke_only` encapsulates
  outcome classification, Trail append, G2 terminal publication, presentation updates,
  and Result Anchor writing. Capacity is released only after all of these finish and
  `GroupMemberState` transitions to `Terminal`.
- Channel failure is fail-closed `AuditFailed`: missing worker results do not produce
  fabricated outcomes and fail closed at join evaluation.
- State-derived capacity from `GroupMemberState` is confirmed sound without a second
  independent mutable counter.

## Evidence

- Mandatory startup gate completed and dev tools verified via `pwsh -NoProfile -File scripts/check-dev-tools.ps1`.
- `git diff --check` passed with 0 errors.
- `.github/scripts/check-tethers-task-packet.ps1` verified for packet consistency.
- Code audit of `tethers-0.1/host-rust/src/replay_runtime.rs`, `tethers-0.1/host-rust/src/host_execution.rs`, and `tethers-0.1/host-rust/src/application.rs` confirmed runtime alignment.

## Publication evidence

Branch pushed: `feature/c3-bounded-window-design`.

Full remote HEAD SHA: `440e9e2e40e0ea9c8db70fe39346224b4705465c` (implementation) / closeout commit.

Local `HEAD == remote HEAD` confirmed.

`git status --short --branch` confirmed clean.

## Discoveries

None.

## Remaining risks

None known within packet scope. C3-A1 remains NOT authorised.

## Smallest next action

Submit C3-D2 adversarial review to Lucy for architecture acceptance.

## References

- `docs/concurrency/C3_BOUNDED_CONCURRENCY_DESIGN.md`
- `docs/concurrency/C2_A3_PHYSICAL_CONCURRENCY_DESIGN.md`
- `tethers-0.1/host-rust/src/replay_runtime.rs`
- `tethers-0.1/host-rust/src/host_execution.rs`
- `tethers-0.1/host-rust/src/application.rs`
