# C3-A2 — Resource / Deadline / G1 Crucible

Control contract: `1`

Status: `COMPLETE`

Task colour: `Red`

Owner: `C3-A2 Proof Agent`

Route: `C3-A2 deterministic launch-boundary proof — awaiting Lucy review`

Base commit: `c775d2bd335ea295c6567e85def1b8672568fd73`

Implementation checkpoint: `9388f86459145551d843442ff72018427238ca96`

Worker note: `docs/worker-notes/2026-08-15-c3-a2-resource-deadline-g1-proof.md`

Updated: 2026-08-15

**C3-A1 is accepted by Lucy. This task may add proof/test support only.
Production scheduler behaviour is frozen. C3-A3/A4 are NOT authorised.**

## Objective

Prove the C3 bounded launch window resource, deadline isolation, G1 boundary,
and semantic-order launch properties through deterministic barrier/replay tests.

## Relevant background and existing behaviour

- C3-A1 is complete and accepted by Lucy at `c775d2bd335ea295c6567e85def1b8672568fd73`.
- `execute_group_concurrent_with_limit` gates worker launch on `active_count < N` in semantic order.
- Stage A prepares all members serially (G0, Trail intent). Stage B launches when capacity available (G1, provider). Stage C collects results (G2, terminal).
- Existing test harness `C3A1GroupHarness` provides barrier-based provider synchronization.
- Existing `ObservingReplayAuthority` and `ReplayTrace` record G0/G1/G2 events for test observation.
- The barrier fixture creates `active-member-{name}` and `entered-member-{name}` files for physical synchronization.

## Required behaviour

1. Add a test-only `run_group_with_trace` method on `C3A1GroupHarness` that uses `ObservingReplayAuthority` to observe replay events while preserving barrier-based physical synchronization.
2. Prove that a capacity-blocked member has G0 observed but G1 NOT observed and provider effect absent.
3. Prove that queue wait duration exceeding a member's configured timeout does not cause timeout classification; the member succeeds after launch with a fresh deadline.
4. Prove that when capacity becomes available, the earliest semantic-order waiting member launches next.
5. Prove that queued member replay events follow strict G0 → capacity wait → G1 → provider → G2 ordering.

## Relevant components

- `tethers-0.1/host-rust/src/host_execution.rs`
- `docs/concurrency/C3_BOUNDED_CONCURRENCY_DESIGN.md`

## Frozen decisions and invariants

- Stage A remains serial and unchanged in Runtime Plan order
- G0 published in Stage A; G1 published in Stage B immediately before launch
- ReplayAdmission remains coordinator-owned (!Send)
- Trail remains coordinator-owned (single writer)
- Provider timeout does NOT run while waiting for capacity
- No queue timeout, no new terminal taxonomy
- Join occurs only after all members reach terminal state
- First non-success selected in semantic Runtime Plan order

## Acceptance criteria

1. Waiting member has G0 but no G1 and no provider effect while capacity-blocked.
2. Queue wait longer than provider timeout does not consume provider timeout; member succeeds after launch.
3. Next capacity slot launches earliest semantic-order waiting member.
4. Queued member replay events follow strict G0 → (capacity wait) → G1 → provider → G2 ordering.
5. All four proofs use `ObservingReplayAuthority` + `ReplayTrace` for replay event observation and barrier files for physical synchronization.

## Required verification

1. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_a2 --test-threads=1`
2. `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml -- --check`
3. `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml`
4. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- --test-threads=1`
5. `git diff --check`
6. `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`

## Forbidden changes

- Production execution code in host_execution.rs
- External configuration schemas or public CLI/API changes
- Replay ledger / ReplayAdmission ownership or Sendness changes
- Worker pools, thread pools, or async/Tokio runtimes
- New terminal outcome taxonomy or queue deadlines
- Modifying OCaml code, SPEC, CONSTITUTION, or DECISIONS

## Stop conditions

- A required production change outside test code is needed (STOP and report blocker).
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
