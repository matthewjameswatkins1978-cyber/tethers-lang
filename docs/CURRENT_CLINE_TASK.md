# C4 — Adversarial Concurrency Crucible

Control contract: `1`

Status: `COMPLETE`

Task colour: `Red`

Owner: `C4 Adversarial Concurrency Agent`

Route: `C4 adversarial crucible complete — awaiting Lucy acceptance`

Base commit: `840b3903f3261244484d7423722bc6ad1f462d74`

Implementation checkpoint: `37cb0dc910fdedc00d54ff29ea78e463bedf00f7`

Worker note: `docs/worker-notes/2026-08-15-c4-adversarial-concurrency-crucible.md`

Updated: 2026-08-15

## Objective

Attack the frozen C1–C3 concurrency implementation with hostile timing, hostile provider outcomes, replay/persistence failures, worker panic under N=2 pressure, same-provider hostility, repeated stress, and channel failure injection. Prove the invariants hold without changing production semantics.

## Relevant background and existing behaviour

- C1–C3 concurrency implementation and proof matrix are accepted.
- Bounded concurrency coordinator is in `tethers-0.1/host-rust/src/host_execution.rs`.
- Replay authority is in `tethers-0.1/host-rust/src/replay_runtime.rs`.
- Trail and dispatch types are in `tethers-0.1/host-rust/src/dispatch.rs`.
- C4 adds TEST / #[cfg(test)] fault-injection support only.
- No production semantic change is authorised.
- Any production defect is a BLOCKER and must be reported, not repaired.
- C5 is NOT authorised.

## Required behaviour

1. Add Crucible 1: Hostile completion order test where physical completion is B, C, A under N=2, proving max active <= 2, slot reuse, and first non-success remains A in semantic order.
2. Add Crucible 2: Hostile slow success + fast failure test proving provider failure does not halt launches and join evaluates all members.
3. Add Crucible 3: G2 failure with active sibling proving fatal halt stops queued member C while already-active B completes truthfully and no GroupJoin occurs.
4. Add Crucible 4: G1 failure with active sibling proving pre-effect failure on A halts queued C while already-active B completes truthfully and no GroupJoin occurs.
5. Add Crucible 5: Outcome durability failure with active sibling proving Trail failure on A halts queued C while already-active B completes truthfully without audit contamination.
6. Add Crucible 6: Worker panic under real N=2 pressure proving panic on A maps to Uncertain, releases slot, queued C launches, sibling B continues, and GroupJoin evaluates all members.
7. Add Crucible 7: Channel disconnect analysis and safe test-only drop seam to verify fail-closed AuditFailed behaviour without fake terminal or coordinator hang.
8. Add Crucible 8: Same-provider hostility proof confirming overlapping ephemeral sessions preserve semantic order under inverse completion.
9. Add Crucible 9: Deterministic repeated stress loop (20 iterations) verifying no state or capacity leaks under inverse completion.
10. Add Crucible 10: Randomness audit of the C1–C3 execution path confirming zero nondeterministic selection sources.

## Relevant components

- `tethers-0.1/host-rust/src/host_execution.rs` (TEST / #[cfg(test)] additions only)
- `tethers-0.1/host-rust/src/dispatch.rs` (TEST / #[cfg(test)] additions if needed)
- `tethers-0.1/host-rust/src/replay_runtime.rs` (TEST / #[cfg(test)] additions if needed)

## Frozen decisions and invariants

- No production semantic changes authorised.
- No scheduler redesign.
- No worker pool, async/Tokio, global scheduler, rate limiting, or cancellation.
- Host-wide concurrency across evaluations remains out of scope.
- First non-success selected strictly by semantic Runtime Plan order.
- Replay and Trail ownership remain strictly coordinator-side.
- Any required production behavior change is a BLOCKER.
- C5 is NOT authorised.

## Acceptance criteria

1. Crucible 1 test passes and proves semantic first failure selection under inverse completion.
2. Crucible 2 test passes and proves normal failure releases slot without halting queue.
3. Crucible 3 test passes and proves G2 failure halts queue while active sibling finishes truthfully.
4. Crucible 4 test passes and proves G1 failure halts queue while active sibling finishes truthfully.
5. Crucible 5 test passes and proves outcome durability failure halts queue while active sibling finishes truthfully.
6. Crucible 6 test passes and proves worker panic under N=2 releases slot and evaluates all members.
7. Crucible 7 proves channel disconnect fails closed or records exact reason deferred if not constructible with test-only seams.
8. Crucible 8 proves same-provider overlap preserves semantic non-success selection.
9. Crucible 9 passes 20 deterministic stress iterations with no capacity or state leaks.
10. Crucible 10 randomness audit confirms zero nondeterministic selection sources.
11. All retained C1–C3 tests remain green.
12. Full test suite passes.
13. Zero production semantic changes in diff.

## Required verification

1. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c4_ --test-threads=1`
2. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_v1 --test-threads=1`
3. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_a4 --test-threads=1`
4. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_a3 --test-threads=1`
5. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_a2 --test-threads=1`
6. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_a1 --test-threads=1`
7. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c2_a3a --test-threads=1`
8. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c2a3a --test-threads=1`
9. `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml -- --check`
10. `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml`
11. `cargo check --locked --manifest-path tethers-0.1/host-rust/Cargo.toml`
12. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- --test-threads=1`
13. `git diff --check`
14. `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`

## Forbidden changes

- No production semantic Rust changes
- No scheduler redesign
- No new production counters or global state
- No async/Tokio or worker pool
- C5

## Stop conditions

- If any frozen invariant is violated by production code.
- If satisfying any test requires a production semantic repair.
- Discovered production defect.

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
