# C3-A3 — Failure-Boundary Crucible

Control contract: `1`

Status: `COMPLETE`

Task colour: `Red`

Owner: `C3-A3 Failure-Boundary Agent`

Route: `C3-A3 fatal-halt correction and failure proof — awaiting Lucy review`

Base commit: `d8b094c5d89f78cb5b610f5367f098f6cc0ef277`

Implementation checkpoint: `071470f6c64bf609d2b55e6dd8839a7131697543`

Worker note: `docs/worker-notes/2026-08-15-c3-a3-failure-boundary-crucible.md`

Updated: 2026-08-15

- C3-A1 and corrected C3-A2 are accepted by Lucy.
- One narrow fatal-halt GroupJoin defect is authorised for correction.
- Lucy remote review identified audit_failure trail contamination when N>1.
- Correction: isolate audit_failure detection to current boundary call only.
- N=2 B/A/C regression proof added.
- No other scheduler redesign is authorised.
- C3-A4 is NOT authorised.

## Objective

Correct the single identified production defect in the fatal-halt path (where GroupJoin was erroneously appended even when queued members remained nonterminal), and prove the C3 bounded launch window failure boundaries through deterministic barrier and replay tests.

## Relevant background and existing behaviour

- C3-A1 introduced the bounded launch window `max_active_together_invocations` and C3-A2 proved the deadline isolation and G1 boundaries.
- Under current implementation, when a fatal trusted-state failure occurs during Stage B/C (e.g. Stage C OutcomeEntry durability failure or G2 failure), `launches_halted` is set to `true`, queued siblings remain `GroupMemberState::Prepared`, active workers drain, and Stage D still appends a `GroupJoinEntry` (with `joined=false`).
- This violates the accepted C3 design: `GroupJoin` exists ONLY after every semantic member has reached its legitimate terminal state. A queued Prepared member after fatal trusted-state halt is NOT terminal.

## Required behaviour

1. Add a narrow fail-closed guard before Stage D GroupJoin publication in `execute_group_concurrent_with_limit` so that if any semantic member remains nonterminal, no `GroupJoinEntry` and no `group_joined` response presentation are appended, and execution fails closed returning a deterministic existing non-success/infrastructure result.
2. Prove that normal provider failure releases launch capacity and queued siblings continue to launch and join (Proof 1).
3. Prove that worker panic caught via `PanicGuard` terminalises as `Uncertain`, releases launch capacity, and queued siblings continue to launch and join (Proof 2).
4. Prove that Stage C OutcomeEntry durability failure halts queued launches without appending a `GroupJoinEntry` (Proof 3).
5. Prove that replay G2 publication failure halts queued launches without appending a `GroupJoinEntry` (Proof 4).
6. Prove that replay G1 publication failure halts before any provider effect and without appending a `GroupJoinEntry` (Proof 5).
7. Prove that all-terminal groups continue to produce standard GroupJoin entries and responses (Proof 6 regression).

## Relevant components

- `tethers-0.1/host-rust/src/host_execution.rs`
- `tethers-0.1/host-rust/src/dispatch.rs`
- `tethers-0.1/host-rust/src/replay_runtime.rs`
- `docs/concurrency/C3_BOUNDED_CONCURRENCY_DESIGN.md`

## Frozen decisions and invariants

- GroupJoin exists ONLY after every semantic member has reached its legitimate terminal state.
- Queued Prepared members after a fatal halt are NOT terminal and must not have terminal states fabricated for them.
- No new terminal taxonomy (no Cancelled, Aborted, QueueFailed, InfrastructureStopped).
- First non-success selected in semantic Runtime Plan order.
- ReplayAdmission remains coordinator-owned (!Send).
- Trail remains coordinator-owned (single writer).
- Normal provider failures and panics do NOT trigger fatal launch halt and MUST produce normal GroupJoin after all members are terminal.

## Acceptance criteria

1. Pre-Stage-D guard prevents `GroupJoinEntry` and `group_joined` presentation when any member remains nonterminal, returning deterministic existing non-success/infrastructure error.
2. Normal provider failure on member A releases slot, member B launches and succeeds, both terminalise, GroupJoin exists with joined=false, and final result is A's Failed.
3. Worker panic on member A is caught, terminalises as Uncertain, releases slot, member B launches and succeeds, GroupJoin exists with joined=false, and final result is A's Uncertain.
4. Stage C OutcomeEntry durability failure on member A halts queued launches (B and C never enter provider), no G2 for A if stopped before G2, no GroupJoin is appended, and fails closed through AuditFailed.
5. Replay G2 failure on member A halts queued launches (B and C never get G1, never enter provider), no GroupJoin is appended, and fails closed.
6. Replay G1 failure on member A halts before provider effect (A never enters provider, B never gets G1, never enters provider), no GroupJoin is appended, and fails closed.
7. All-terminal groups (including C3-A1 and C3-A2 test suites) continue to produce standard GroupJoin entries and responses without regression.

## Required verification

1. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_a3 --test-threads=1`
2. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_a2 --test-threads=1`
3. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_a1 --test-threads=1`
4. `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml -- --check`
5. `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml`
6. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- --test-threads=1`
7. `git diff --check`
8. `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`

## Forbidden changes

- Any production changes outside `tethers-0.1/host-rust/src/host_execution.rs`
- Changing source semantics, Runtime Plan, or SemanticPosition
- Changing normal GroupJoin semantics when all members are terminal
- Adding new terminal states or result taxonomy
- Adding queue timeout or scheduler watchdog
- Adding worker pool or async/Tokio
- Adding host-global or provider-aware scheduling
- Moving G0 or G1 publication boundaries
- Changing ReplayAdmission ownership or Sendness
- Modifying OCaml code, SPEC, CONSTITUTION, or DECISIONS

## Stop conditions

- A required production change outside `host_execution.rs` is needed (STOP and report blocker).
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
- `scripts/assert-worktree.ps1`
- `tethers-0.1/engine-ocaml/bin/tethers_cb3t_tie_audit.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_rank_avalanche.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_v2_canon_label.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_v2_canon_label_test.ml`
