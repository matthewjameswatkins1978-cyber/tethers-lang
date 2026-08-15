# C3-A1 — Minimal Bounded Launch Window

Control contract: `1`

Status: `COMPLETE`

Task colour: `Red`

Owner: `C3-A1 Implementation Agent`

Route: `C3-A1 bounded launch implementation — awaiting Lucy review`

Base commit: `d01e41ec00c89b548e6641e9c7661c7ffd2ccfe8`

Worker note: `docs/worker-notes/2026-08-15-c3-a1-bounded-launch-window.md`

Updated: 2026-08-15

**C3-D1/D2 design is accepted by Lucy. C3-A1 may implement ONLY the minimal bounded
launch mechanism. C3-A2/A3/A4 are NOT authorised. No configuration schema change
is authorised. No C3 redesign is authorised. Production Rust changes are restricted
to host_execution.rs.**

## Objective

Implement the minimal bounded launch window for Together group execution as
frozen by accepted design C3-D1/D2, bounding active provider invocations to an
internal/test-injectable parameter `max_active_together_invocations: usize` (N >= 1)
without altering Together source semantics, Runtime Plan semantic identity,
member SemanticPosition, replay G0/G1/G2 meaning, Trail truthfulness, terminal
taxonomy, deterministic final non-success selection, or sequential Action behaviour.

## Relevant background and existing behaviour

- C2-A3a physical concurrency is complete and merged to `main` at `f189361e80bdb43c13989200e48513cdb68bd004`.
- C3-D1/D2 architecture design `docs/concurrency/C3_BOUNDED_CONCURRENCY_DESIGN.md` is accepted at `d01e41ec00c89b548e6641e9c7661c7ffd2ccfe8`.
- In A3a, `execute_group_concurrent` prepares all members in Stage A and immediately spawns all eligible prepared workers into scoped threads with unbounded physical concurrency.
- In C3-A1, Stage A still prepares all members in semantic order, but worker launches are gated by `active_count < N` in semantic order.
- Capacity is strictly derived from `GroupMemberState` (active = crossed G1, not yet completed Stage C terminalisation).

## Required behaviour

1. Parameterised Bounded Execution: Provide an internal/test-injectable group-local execution parameter `max_active_together_invocations: usize` (N >= 1), while preserving existing `execute_group_concurrent` callers by defaulting to group width (A3a-compatible full overlap).

2. State-Derived Capacity: Derive active invocation capacity directly from `member_states` (counting members that have crossed G1 / launch boundary and have not yet transitioned to `Terminal` after complete Stage C processing) with no independent mutable counter as authority.

3. Semantic-Order Admission: When capacity exists (`active_count < N`), admit and launch the earliest semantic-order `Prepared` waiting member without provider-aware skipping, randomisation, or priority.

4. Launch-Boundary Invariants: For each selected member at launch time, establish per-member monotonic deadline start (`clock.now()`), calculate remaining timeout, perform pre-invocation deadline check, publish G1 (`publish_armed`), and spawn worker. Provider timeout does NOT run while waiting for capacity.

5. Singular Stage C Release Point: Release capacity (decrement active count) only after complete coordinator Stage C processing (`execute_boundary_invoke_only` including durable `OutcomeEntry`, G2 `publish_terminal`, presentation/response updates, and Result Anchor writing) and transition of `GroupMemberState` to `Terminal`.

6. Fail-Closed Launch Halt: On trusted Stage C durability or G2 failure, halt any further launches immediately, allow in-flight scoped threads to drain, and fail closed with existing taxonomy (`AuditFailed` / `ReplayPersistenceUnavailable`).

7. Focused Concurrency Verification: Prove with real provider/barrier tests that N=1 limits active provider invocations to at most 1, N=2 limits active invocations to at most 2 while reaching 2, and N>=group_size preserves full physical overlap.

## Relevant components

- `tethers-0.1/host-rust/src/host_execution.rs`
- `docs/concurrency/C3_BOUNDED_CONCURRENCY_DESIGN.md`
- `docs/concurrency/C2_A3_PHYSICAL_CONCURRENCY_DESIGN.md`

## Frozen decisions and invariants

- Stage A remains serial and unchanged in Runtime Plan order
- G0 published in Stage A; G1 published in Stage B immediately before launch
- ReplayAdmission remains coordinator-owned (!Send)
- Trail remains coordinator-owned (single writer)
- provider timeout does not run while waiting for capacity
- no queue timeout, no new terminal taxonomy
- same-provider overlap via ephemeral child processes preserved
- join occurs only after all members reach terminal state
- first non-success selected in semantic Runtime Plan order
- sequential Actions remain serial
- no Tokio / async, no worker pool, no host-wide scheduler

## Acceptance criteria

1. An internal helper `execute_group_concurrent_with_limit` accepts `max_active_together_invocations: usize` (N >= 1), and `execute_group_concurrent` preserves A3a-compatible full-width behavior.

2. Capacity calculation derives active count exclusively from `member_states` lifecycle transitions.

3. Prepared waiting members are launched strictly in earliest semantic Runtime Plan order when slots become available.

4. Monotonic deadline starts and G1 publication occur per-member at physical launch, keeping queue wait isolated from provider timeout.

5. Capacity is released only upon full completion of `execute_boundary_invoke_only` and transition to `GroupMemberState::Terminal`.

6. Coordinator halts new launches if Stage C persistence or G2 publication fails, failing closed through existing error types.

7. Deterministic barrier/counter tests prove N=1 never exceeds 1 active provider invocation, N=2 reaches 2 without exceeding 2, and full-width preserves overlap.

## Required verification

1. `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml -- --check`
2. `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml`
3. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- --test-threads=1`
4. `git diff --check`
5. `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`

## Forbidden changes

- Rust production files other than `tethers-0.1/host-rust/src/host_execution.rs`
- External configuration schemas or public CLI/API changes
- Replay ledger / ReplayAdmission ownership or Sendness changes
- Worker pools, thread pools, or async/Tokio runtimes
- Host-global schedulers or cross-group semaphores
- Provider-specific priority, fairness, or queue-skipping logic
- New terminal outcome taxonomy or queue deadlines
- Modifying OCaml code, SPEC, CONSTITUTION, or DECISIONS

## Stop conditions

- A required production change outside `host_execution.rs` is needed (STOP and report blocker).
- ReplayAdmission requires `Send` / `Arc` / `Mutex` (STOP).
- A frozen design invariant cannot be satisfied without redesign (STOP).
- Repeated failure rule: 2 materially similar failed attempts on the same underlying problem.

## Expected pre-existing changes

- `WORKTREE.md`
- `docs/CANONICAL_FORMAT_V2_SPEC_DRAFT.md`
- `docs/performance/CORE_PHASE_A_IMPLEMENTATION_PACKET.md`
- `docs/performance/R1_PERFORMANCE_PROOF.md`
- `docs/performance/core-phase-a/`
- `docs/performance/r1/`
- `docs/worker-notes/2026-08-12-c-core-cheap-structural-fixes.md`
- `docs/worker-notes/2026-08-14-c2a1-together-semantic-bridge.md`
- `docs/worker-notes/2026-08-15-c3-d2-adversarial-design-review.md`
- `scripts/assert-worktree.ps1`
- `tethers-0.1/engine-ocaml/bin/tethers_cb3t_tie_audit.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_rank_avalanche.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_v2_canon_label.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_v2_canon_label_test.ml`
