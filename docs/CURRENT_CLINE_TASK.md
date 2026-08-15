# C3-D1 — Bounded Concurrency Design

Control contract: `1`

Status: `IN_PROGRESS`

Task colour: `Red`

Owner: `C3-D1 Architecture Agent`

Route: `C3-D1 design freeze — no implementation authorised`

Base commit: `f189361e80bdb43c13989200e48513cdb68bd004`

Worker note: `docs/worker-notes/2026-08-15-c3-d1-bounded-concurrency-design.md`

Updated: 2026-08-15

**This task is design only. No Rust implementation is authorised. C3-A1 cannot
begin until Lucy accepts this design. C2-A3a semantics are frozen inputs, not
redesign candidates.**

## Objective

Freeze the exact C3 bounded-concurrency architecture before any implementation.
C2-A3a is complete and merged. C3-A must bound physical Together provider
invocation without changing Together source semantics, Runtime Plan semantic
identity, member SemanticPosition, replay G0/G1/G2 meaning, Trail truthfulness,
terminal taxonomy, deterministic final non-success selection, GroupJoin
semantics, or sequential Action behaviour. The design must remain deliberately
smaller than a general scheduler.

## Relevant background and existing behaviour

- C2-A3a physical concurrency design is accepted at `docs/concurrency/C2_A3_PHYSICAL_CONCURRENCY_DESIGN.md`.
- C2-A3a implementation is complete and merged to `main` at `f189361e80bdb43c13989200e48513cdb68bd004`.
- A3a architecture: serial Stage A, `std::thread::scope` workers, mpsc result delivery, coordinator-owned ReplayAdmission, coordinator-owned Trail, ephemeral trusted RetainedProviderSession worker path, sequential non-Together path preserved.
- A3a currently launches all eligible Together workers simultaneously with no bound on concurrent provider invocations.
- C3 introduces bounded launch windows to limit active provider invocations within a group execution.

## Required behaviour

1. C3-A must bound exactly one resource: active Together provider invocations within one group execution, parameterised by `max_active_together_invocations = N` where `N >= 1`.

2. Stage A must remain serial and A3-compatible: scope, policy, capability resolution, provider availability, replay admission, G0 publish_intent, durable Trail intent — all in Runtime Plan order for every member.

3. G0 without G1 must be valid crash/recovery evidence: G0 records coordinator intent to attempt, G1 absence means provider was never touched, the combination is unambiguous.

4. An internal `PREPARED_WAITING` runtime condition must be defined for members that completed Stage A but are waiting for capacity — this is scheduling state only, not a source-language state or Trail terminal.

5. Capacity must be derived from `GroupMemberState` (active_count = members past G1 that have not completed terminalisation), not from a second independent mutable counter.

6. Admission order must be earliest semantic-order `PREPARED_WAITING` member when capacity is available, with no provider-aware skipping.

7. The launch boundary must be: capacity eligibility → deadline start → remaining timeout → final deadline check → G1 → worker launch. The provider timeout must NOT run while `PREPARED_WAITING`.

8. Slot release must occur only after successful coordinator Stage C terminalisation (durable OutcomeEntry + G2 + member terminal transition), not merely on in-memory WorkerResult arrival.

9. Failure boundaries must be explicitly distinguished: normal provider failure (capacity reusable), worker panic (Uncertain, capacity reusable), worker channel failure (fail-closed), Stage C durability failure (halt further launches), replay G2 failure (halt further launches).

10. Group semantics must be preserved: failure does not cancel siblings (except fatal trusted-state failure), GroupJoin only after all members terminal, final non-success by semantic member order.

11. Same-provider overlap must be preserved through independent ephemeral child process instances.

12. Semantic equivalence across N=1, N=2, N>=group-size must hold for source meaning, Plan identity, SemanticPosition, terminal classification, join result, and final non-success selection.

## Relevant components

Design output: `docs/concurrency/C3_BOUNDED_CONCURRENCY_DESIGN.md`

Context files (read-only):

- `docs/concurrency/C2_A3_PHYSICAL_CONCURRENCY_DESIGN.md`
- `docs/ROAD_TO_0_4.md`
- `tethers-0.1/host-rust/src/host_execution.rs`

This list is not modification authority. This task produces no code changes.

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
- G0 without G1 is unambiguous pre-invocation evidence
- provider timeout does not run during capacity wait
- Stage C / G2 failure halts further launches

## Acceptance criteria

1. The design document `docs/concurrency/C3_BOUNDED_CONCURRENCY_DESIGN.md` exists and addresses all 14 required design questions from the task mandate.

2. The design explicitly bounds one resource (active Together provider invocations per group execution) and explicitly excludes host-wide scheduling, global semaphores, worker pools, adaptive concurrency, priorities, fairness, CPU/RAM accounting, rate limiting, and API quotas.

3. Stage A is proved A3-compatible: serial, same ordering, G0/G1 separation preserved.

4. `PREPARED_WAITING` is defined as runtime-only scheduling state, not a new source-language or Trail concept.

5. Capacity is derived from `GroupMemberState`, with exactly which states count as active enumerated.

6. Admission order is semantic-order, with no provider-aware skipping.

7. Launch boundary invariant is stated: G0 → durable intent → capacity wait → deadline start → G1 → worker launch.

8. Slot release point is stated: after Stage C terminalisation, not on WorkerResult arrival.

9. All five failure boundaries (normal, panic, channel, Stage C, G2) are explicitly distinguished with fail-closed treatment.

10. Group semantics are preserved: no sibling cancellation, GroupJoin after all terminal, final non-success by semantic order.

11. Required implementation decomposition (C3-A1 through C3-V1) is provided with objectives, permitted scope, and proof targets.

12. Required future proof matrix (12 items) is provided.

## Required verification

1. `git diff —check` — no whitespace errors
2. `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` — packet consistency PASS

No Rust build/test suite is required because this task must not change code.

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
