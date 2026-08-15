# C3-D2 — Adversarial Bounded Concurrency Design Review

Control contract: `1`

Status: `COMPLETE`

Task colour: `Red`

Owner: `C3-D2 Independent Design Reviewer`

Route: `C3-D2 hostile review — no implementation authorised`

Base commit: `8268fb9e0284b7e9f2268279b4dea5d37f01a9bc`

Worker note: `docs/worker-notes/2026-08-15-c3-d2-adversarial-design-review.md`

Updated: 2026-08-15

**This task is review and narrow design correction only. No Rust implementation is
authorised. C3-A1 remains NOT authorised. C2-A3a semantics are frozen inputs,
not redesign candidates.**

## Objective

Independently attack the proposed C3 bounded-concurrency design against the
actual merged runtime, correct documentation where the code or frozen A3
semantics prove the current design text wrong or internally inconsistent, and
deliver a rigorous adversarial design review without redesigning C3 or changing
production code.

## Relevant background and existing behaviour

- C2-A3a physical concurrency design is accepted at `docs/concurrency/C2_A3_PHYSICAL_CONCURRENCY_DESIGN.md`.
- C2-A3a implementation is merged to `main` at `f189361e80bdb43c13989200e48513cdb68bd004`.
- C3-D1 produced candidate design `docs/concurrency/C3_BOUNDED_CONCURRENCY_DESIGN.md` at `8268fb9e0284b7e9f2268279b4dea5d37f01a9bc`.
- Replay recovery runtime (`replay_runtime.rs`) maps `ReplayState::IntentRecorded` (`G0=yes, G1=no`) to `ReplayDispatchResult::RequiresManualResolution`.
- Stage C coordinator execution (`host_execution.rs`, `application.rs`) executes `execute_boundary_invoke_only` (durable `OutcomeEntry`, G2 `publish_terminal`, presentation/response updates, Result Anchor) before transitioning `GroupMemberState` to `Terminal`.
- Worker execution wraps `worker_invoke_inner` with `catch_unwind` and sends `WorkerResult` via mpsc; unexpected channel closure causes coordinator receive to break and un-terminalised members to fail closed with `AuditFailed` at join.

## Required behaviour

1. Known Issue 1 (G0 recovery claim): Correct the design text to distinguish effect certainty (G0 present + G1 absent proves invocation boundary was not armed, no external effect occurred) from replay authority (existing replay recovery policy remains authoritative, mapping recovered `IntentRecorded` to `RequiresManualResolution`).

2. Known Issue 2 (Slot release boundary): Unify the slot release definition so that capacity is released only after the complete Stage C boundary (`execute_boundary_invoke_only` including durable `OutcomeEntry`, G2 `publish_terminal`, presentation/response updates, and Result Anchor writing) and `GroupMemberState` transition to `Terminal`.

3. Adversarial Check 3 (Channel failure): Clarify channel failure handling against actual `worker_invoke_provider` and `execute_group_concurrent` behaviour, confirming fail-closed `AuditFailed` behaviour when channel disconnects.

4. Adversarial Check 4 (Failure stop rule): Verify and confirm the rule that trusted Stage C / persistence failure stops further launches while allowing in-flight scoped threads to finish without fabricating ordinary provider results.

5. Adversarial Check 5 (Capacity definition): Verify that state-derived capacity from `GroupMemberState` is sound, unambiguous, and free from double-counting or counter drift.

6. Adversarial Check 6 (No scope creep): Verify that the design strictly excludes host-global scheduling, provider-specific quotas, worker pools, fairness, priority, adaptive concurrency, queue timeouts, new taxonomy, JIT G0, and provider-aware queue skipping.

## Relevant components

Design document: `docs/concurrency/C3_BOUNDED_CONCURRENCY_DESIGN.md`

Context files (read-only):

- `docs/concurrency/C2_A3_PHYSICAL_CONCURRENCY_DESIGN.md`
- `tethers-0.1/host-rust/src/replay_runtime.rs`
- `tethers-0.1/host-rust/src/host_execution.rs`
- `tethers-0.1/host-rust/src/application.rs`

## Frozen decisions and invariants

- all Together members are attempted through fan-out semantics
- sibling failure does not cancel siblings (except fatal trusted-state failure)
- join waits for all semantic members terminal
- join succeeds iff all members are successful under existing C1 rules
- ReplayBlockedCompletedSuccess counts as success
- first non-success is selected by semantic Runtime Plan member order
- SemanticPosition derives from flat Runtime Plan indexes
- Trail physical order is durable append order
- ReplayAdmission remains coordinator-owned and need not become Send
- workers own provider invocation material only
- no group-wide replay identity
- sequential Actions remain physically serial
- no Tokio/async
- no host-wide scheduler
- G0 without G1 is unambiguous pre-invocation evidence of no provider effect
- recovered IntentRecorded requires manual resolution under existing replay policy
- provider timeout does not run during capacity wait
- Stage C / G2 failure halts further launches
- C3-A1 remains NOT authorised

## Acceptance criteria

1. The design document `docs/concurrency/C3_BOUNDED_CONCURRENCY_DESIGN.md` accurately describes G0/G1 effect certainty and strictly records that recovered `IntentRecorded` maps to `RequiresManualResolution`.

2. The slot release boundary is singular and unambiguous across all sections, naming the full Stage C boundary including Result Anchor writing and response updates.

3. Channel failure is accurately documented as fail-closed `AuditFailed` matching the actual runtime implementation.

4. Failure stop rules for Stage C durability and G2 failures are confirmed compatible with existing taxonomy and boundaries.

5. State-derived capacity from `GroupMemberState` is confirmed sound without second mutable counters.

6. All C3 scope creep items remain explicitly excluded from C3-A requirements.

## Required verification

1. `git diff --check` — no whitespace errors
2. `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` — packet consistency PASS

## Forbidden changes

- Rust source code
- PowerShell fixtures
- scheduler implementation code
- config fields
- tests
- worker pool introduction
- A3 replay semantics redesign
- G0 relocation
- queue deadlines
- new result taxonomy
- provider-aware scheduling
- host-global concurrency

## Stop conditions

If the existing implementation contradicts one of the frozen assumptions in a way that materially changes C3 semantics, STOP. Do not solve it in code. Report: `BLOCKED — <one exact architectural contradiction>`.

## Expected pre-existing changes

- `WORKTREE.md`
- `docs/CANONICAL_FORMAT_V2_SPEC_DRAFT.md`
- `docs/performance/CORE_PHASE_A_IMPLEMENTATION_PACKET.md`
- `docs/performance/R1_PERFORMANCE_PROOF.md`
- `docs/performance/core-phase-a/RESULT.md`
- `docs/performance/core-phase-a/after-stage-profile.txt`
- `docs/performance/core-phase-a/before-stage-profile.txt`
- `docs/performance/r1/retained-p10-after.csv`
- `docs/performance/r1/retained-p10-after.json`
- `docs/worker-notes/2026-08-12-c-core-cheap-structural-fixes.md`
- `docs/worker-notes/2026-08-14-c2a1-together-semantic-bridge.md`
- `scripts/assert-worktree.ps1`
- `tethers-0.1/engine-ocaml/bin/tethers_cb3t_tie_audit.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_rank_avalanche.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_v2_canon_label.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_v2_canon_label_test.ml`
