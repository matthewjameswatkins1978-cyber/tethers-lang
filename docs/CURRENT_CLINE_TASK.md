# C2-A3a — Final Provider Overlap Correction

Control contract: `1`

Status: `COMPLETE`

Task colour: `Red`

Owner: `Mini 2.5 Pro — independent final verification`

Route: `C2-A3a complete — awaiting Lucy merge review`

Base commit: `58aecd0c789802cdfea57d4560b51fd21d5340ae`

Worker note: `docs/worker-notes/2026-08-14-c2-a3a-provider-overlap-correction.md`

Updated: 2026-08-15

The C2-A3a correction and independent final verification are complete at formal
base `58aecd0c789802cdfea57d4560b51fd21d5340ae`. The implementation is
published on `feature/c2-a3a-provider-overlap` and is awaiting Lucy merge
review; it is NOT yet merged to `main`. No prior packet text constitutes
implementation authority for this task.

## Objective

Codex must surgically correct the existing C2-A3a implementation at
`58aecd0c789802cdfea57d4560b51fd21d5340ae` without rewriting it from scratch.
The coordinator/worker overlap architecture is preserved while correcting:

1. loss of exact terminal member classifications
2. fabricated/dummy semantic objects used for Rust ownership transitions
3. incomplete empirical proof of actual provider-effect overlap

The task ends only when real provider overlap AND exact C1 terminal semantics
are proved.

## Relevant background and existing behaviour

- C1 Together semantics are accepted.
- C2-A1, A2a and A2b are complete.
- C2-A3 design is accepted.
- Current A3a implementation is at `58aecd0c789802cdfea57d4560b51fd21d5340ae`.
- Current A3a architecture already has:
  - serial Stage A preparation
  - `std::thread::scope` workers
  - mpsc result delivery
  - coordinator-owned ReplayAdmission
  - coordinator-owned Trail
  - ephemeral trusted RetainedProviderSession worker path
  - sequential non-Together path preserved
- Current A3a is NOT ACCEPTED.

Recorded review defects:

A. GroupMemberState terminal variants currently reduce non-success state to a
boolean, and final aggregation collapses distinct outcomes into generic Failed.

B. Production code still uses temporary/fabricated objects including:
- tmp DispatchReadyAction
- test_manifest()
- test_resolved_capability()
- NoopReplayAdmissionGuard

C. The required real provider-effect concurrency crucible is not yet present.

## Required behaviour

1. Every Together member must retain one exact terminal result compatible with
   existing C1 ActionStep / step_succeeded / aggregate_step semantics. Denied,
   ApprovalRequired, Unavailable, Uncertain, Unattempted, AuditFailed, replay
   classifications and Completed/Failed must not be flattened.

2. Preparation failures must preserve the same semantic/service result as the
   existing serial execution path. Do not manufacture a second set of
   concurrent-only OutcomeEntry reason codes or empty execution identities.

3. Rust ownership transitions must use structural state movement such as
   Option::take or whole-enum transitions. Production code must not create fake
   DispatchReadyAction, manifest, ResolvedCapability or replay guards to satisfy
   borrowing.

4. Keep mpsc arrival-order result delivery. The coordinator must durably run
   Stage C for a received worker result while other provider workers may still
   be in flight. Trail physical order remains durable append order.

5. Implement deterministic provider-fixture tests proving actual tools/call
   overlap for both same-provider and different-provider Together members,
   including max simultaneous provider effects >= 2.

6. Add a prompt-persistence test proving member B's OutcomeEntry is durable
   while member A is intentionally still blocked in provider execution.

7. Preserve deterministic final non-success selection by semantic Runtime Plan
   member order independent of provider completion/Trail append order.

8. Preserve G0 → durable Trail intent → deadline → G1 → possible provider
   effect ordering for every invoked member.

9. Sequential non-Together Actions must remain on the existing serial retained
   session path.

10. Replay format/identity, Trail schema, approval semantics, Result Anchor
    format, Core/OCaml, Canonical V2 and Rocket must remain unchanged.

11. No force push. The correction must be one or more normal descendant commits
    from base `58aecd0c789802cdfea57d4560b51fd21d5340ae`.

## Relevant components

Likely relevant implementation files:

- `tethers-0.1/host-rust/src/host_execution.rs`
- `tethers-0.1/host-rust/src/application.rs`

Potential cleanup/test-support surface:

- `tethers-0.1/host-rust/src/dispatch.rs`
- `tethers-0.1/host-rust/src/executor.rs`
- `tethers-0.1/host-rust/src/manifest.rs`
- `tethers-0.1/host-rust/src/resolver.rs`
- existing stdio provider fixture/support

This list is not blanket modification authority.

## Frozen decisions and invariants

- all Together members are attempted through fan-out semantics
- sibling failure does not cancel siblings
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
- no C3 resource scheduler

## Acceptance criteria

1. Exact terminal classifications survive group execution and final
   aggregation.

2. Preparation failures match existing serial result semantics.

3. No production dummy/fake semantic objects remain in ownership transitions.

4. mpsc Stage C persists results as they are received while sibling workers may
   still run.

5. Same-provider tools/call overlap test proves >=2 simultaneous effects.

6. Different-provider overlap test proves >=2 simultaneous effects.

7. Prompt durable outcome test proves one outcome is written before blocked
   sibling release.

8. Inverse physical completion orders produce identical deterministic final
   semantic selection.

9. Intent and G1 are proved before provider effect.

10. Sequential control remains serial and compatible.

11. No forbidden semantic/schema/replay/resource-scheduler changes and no
    force push.

## Required verification

Codex must run and report PASS / FAIL / NOT RUN for each of:

- focused C2-A3a concurrency crucible tests
- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml -- --check`
- `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml`
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- --test-threads=1`
- `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features`
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features -- --test-threads=1`
- `git diff --check`
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`

No mandatory NOT RUN can result in COMPLETE.

## Forbidden changes

- Core/OCaml semantic changes
- Canonical V2 changes
- Rocket changes
- replay format or identity changes
- Trail schema changes
- approval redesign
- Result Anchor format changes
- retries
- fail-fast sibling cancellation
- Tokio/async
- global worker pool
- C3 rate/resource scheduling
- nested Together
- force push
- history rewriting

## Stop conditions

Codex must STOP rather than widen scope if:

- exact terminal-result preservation requires changing accepted C1 semantics
- replay objects must become Send/Sync
- Trail requires multiple concurrent writers
- provider overlap requires bypassing trusted Socket discovery
- a replay/Trail schema migration appears necessary
- Core/Canonical/Rocket changes appear necessary
- existing branch ancestry cannot be preserved with a normal push
- required real provider-effect overlap cannot be deterministically tested

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
