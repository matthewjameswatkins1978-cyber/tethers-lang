# C3-V1 — Proof Gap Correction

Control contract: `1`

Status: `COMPLETE`

Task colour: `Red`

Owner: `C3-V1 Proof Gap Correction Agent`

Route: `Proof correction — awaiting Lucy acceptance`

Base commit: `8a09203715cc44f42c011c0c8902ff4f72a246c7`

Implementation checkpoint: `8a09203715cc44f42c011c0c8902ff4f72a246c7`

Worker note: `docs/worker-notes/2026-08-15-c3-v1-independent-review.md`

Updated: 2026-08-15

## Objective

Fix proof gaps in the C3-V1 independent review. The review incorrectly marked two frozen future-proof requirements PASS when the exact group-of-five requirements were not actually tested. Also correct same-provider evidence mapping.

## Relevant background and existing behaviour

- Independent review found no implementation defect in C3.
- Lucy found inaccurate proof attribution in V1 review.
- Exact group-of-five frozen requirements were not actually tested.
- Same-provider concurrency was incorrectly attributed to `c3_a1_full_width_preserves_full_overlap`.
- This packet fills proof gap only. C4 is NOT authorised.

## Required behaviour

1. Add deterministic test `c3_v1_n1_group_of_five_proves_bound_and_full_terminalisation` with exactly 5 members (a, b, c, d, e) and N=1.
2. Add deterministic test `c3_v1_n2_group_of_five_proves_bound_reached_and_full_terminalisation` with exactly 5 members (a, b, c, d, e) and N=2.
3. Both tests must include live GroupJoin absence assertions while members are active/waiting.
4. Correct same-provider evidence to reference `c2_a3a_same_provider_tools_call_overlap_is_real`.
5. Update review worker note with corrected evidence matrix.

## Relevant components

- `tethers-0.1/host-rust/src/host_execution.rs` (TEST additions only)

## Frozen decisions and invariants

- No production semantic changes.
- No scheduler redesign.
- C4 is NOT authorised.
- Any required production behavior change is a BLOCKER.

## Acceptance criteria

1. `c3_v1_n1_group_of_five_proves_bound_and_full_terminalisation` passes with exactly 5 members.
2. `c3_v1_n2_group_of_five_proves_bound_reached_and_full_terminalisation` passes with exactly 5 members.
3. Both tests assert GroupJoin absence during execution at multiple refill points.
4. Both tests assert GroupJoin presence after all terminal.
5. Same-provider evidence correctly references `c2_a3a_same_provider_tools_call_overlap_is_real`.
6. All existing C3 tests remain green.
7. Full suite passes.
8. No production semantic changes in diff.

## Required verification

1. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_v1 --test-threads=1`
2. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_a1 --test-threads=1`
3. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_a2 --test-threads=1`
4. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_a3 --test-threads=1`
5. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_a4 --test-threads=1`
6. `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml -- --check`
7. `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml`
8. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- --test-threads=1`
9. `git diff --check`
10. `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`

## Forbidden changes

- No production Rust modification (test/#[cfg(test)] additions only)
- No scheduler redesign
- No new production counters
- C4

## Stop conditions

- If satisfying these proofs requires production behavior changes.
- Any discovered semantic defect.

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

## Requested outcome

1. Add two group-of-five tests proving frozen design requirements §14.1 and §14.2.
2. Include live GroupJoin absence assertions.
3. Correct same-provider evidence mapping.
4. Update review worker note.
5. Push correction branch.

## Primary question

Can the frozen group-of-five requirements be proven without production changes?

**Answer: YES.** Two new tests with exactly 5 members each prove N=1 and N=2 bounds with live GroupJoin absence assertions.

## Verdict

**C3-V1 PROOF MATRIX COMPLETE — REVIEW BRANCH PUBLISHED**

## Evidence

- c3_v1: 2 tests PASS (N=1 group-of-five, N=2 group-of-five)
- c3_a1: 3 tests PASS
- c3_a2: 4 tests PASS
- c3_a3: 7 tests PASS
- c3_a4: 12 tests PASS
- Full suite: 1542 tests PASS
- cargo fmt: PASS
- cargo check: PASS
- git diff --check: PASS
- Production diff: ZERO semantic changes (test additions only)
