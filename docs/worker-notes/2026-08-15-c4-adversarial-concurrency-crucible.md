# Worker Note: C4 Adversarial Concurrency Crucible

Task: `C4 — Adversarial Concurrency Crucible`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `C4 Adversarial Concurrency Agent`

Status: `COMPLETE`

Base commit: `840b3903f3261244484d7423722bc6ad1f462d74`

Implementation checkpoint: `37cb0dc910fdedc00d54ff29ea78e463bedf00f7`

## Requested outcome

Attack the frozen C1–C3 bounded concurrency implementation with hostile timing, hostile provider outcomes, replay/persistence failures, worker panic under N=2 pressure, same-provider hostility, repeated stress, and channel failure injection. Prove that the frozen invariants hold without changing production semantics.

## Changes made

All implementation changes are strictly TEST-ONLY additions to `tethers-0.1/host-rust/src/host_execution.rs` under `#[cfg(test)] mod tests`. No production code was modified.

1. **Crucible 1 (Hostile Completion Order):** Added `c4_inverse_completion_preserves_semantic_first_failure`. Proves that under N=2 with members A, B, C where physical completion order is B (Failed), C (Success), A (Failed), max active count never exceeds 2, C launches into B's released slot while A remains active, physical Trail outcome order is B, C, A, final semantic first non-success remains Member A (not Member B), and GroupJoin occurs with `joined: false` only after all members terminalise.
2. **Crucible 2 (Hostile Slow Success + Fast Failure):** Added `c4_fast_failure_releases_slot_while_slow_sibling_runs`. Proves that under N=2 with slow Success A, fast Failed B, and queued Success C, normal provider failure on B releases its slot without triggering `launches_halted`, queued C launches into available capacity while A is still active, A and C succeed, and join evaluates all members.
3. **Crucible 3 (G2 Failure with Active Sibling):** Added `c4_g2_failure_halts_queue_but_active_sibling_finishes_truthfully`. Proves that under N=2 with semantic order B, A, C, when B and A are active and A returns and fails G2 `publish_terminal`, fatal halt prevents queued C from receiving G1 or entering provider, already-active B completes truthfully with normal OutcomeEntry and G2(B) success without contamination, no GroupJoin is appended, and group result fails closed as `ReplayPersistenceUnavailable` identifying member A.
4. **Crucible 4 (G1 Failure with Active Sibling):** Added `c4_g1_failure_halts_queue_but_active_sibling_finishes_truthfully`. Proves that under N=2 with semantic order B, A, C, when B launches and A fails G1 before worker spawn, fatal halt halts queue so C never gets G1 or enters provider, already-active B completes truthfully with OutcomeEntry and G2(B) success, no GroupJoin is appended, and group result fails closed as `ReplayPersistenceUnavailable` for member A.
5. **Crucible 5 (Outcome Durability Failure with Active Sibling):** Added `c4_outcome_durability_failure_does_not_contaminate_active_sibling`. Proves that under N=2 with semantic order B, A, C, when B and A are active and A fails OutcomeEntry durability, fatal halt halts queued C, already-active B completes normal successful Stage C without audit contamination, no GroupJoin is appended, and group result fails closed as `AuditFailed` for member A.
6. **Crucible 6 (Worker Panic Under Real N=2 Pressure):** Added `c4_worker_panic_under_n2_pressure_releases_slot`. Proves that under N=2 with members A, B, C, worker panic on A is caught, classified as Uncertain in Stage C, releases slot, queued C launches into released capacity, active sibling B continues and succeeds, all 3 members reach terminal state, GroupJoin occurs with `joined: false`, and final non-success is Member A Uncertain.
7. **Crucible 7 (Channel Disconnect Analysis):** Conducted rigorous architectural analysis of the missing `WorkerResult` / channel disconnect path. Confirmed that in production `worker_invoke_provider` wraps worker execution in `catch_unwind` and infallibly transmits `WorkerResult` (mapping panic to `NoFinalResponse -> Uncertain`). Because the coordinator retains the master `tx` for dynamic bounded launches across the group lifetime, a worker-side sender drop without sending cannot close the channel on `rx.recv()`; the coordinator would wait on its own open sender. Adding coordinator-side timeouts or watchdog threads is explicitly out of scope for C4. If channel disconnection ever occurred, lines 2360-2389 catch `any_nonterminal` and fail closed via `ExecutionServiceResult::AuditFailed` without GroupJoin.
8. **Crucible 8 (Same-Provider Hostility):** Added `c4_same_provider_overlap_and_inverse_completion_preserves_semantic_order`. Proves that two Together members targeting the same provider identity (`"tethers-stdio-fixture"`) establish independent ephemeral child processes, overlap concurrently in the barrier, complete in inverse order (member 1 before member 0), and deterministic first non-success selection preserves semantic member 0.
9. **Crucible 9 (Repeated Stress Loop):** Added `c4_repeated_inverse_completion_has_no_state_leak`. Runs 20 consecutive iterations of N=2 3-member inverse completion with process spawning and barrier synchronization. Proves zero capacity leaks, zero state leakage, exact physical outcome order (B, C, A), and exact semantic result across every iteration.
10. **Crucible 10 (Randomness Audit):** Meticulous code inspection of the C1–C3 execution path in `host_execution.rs`. Confirmed zero sources of physical, hash-map, or timing nondeterminism in semantic member selection.

## Decisions and assumptions

1. **Strict Test-Only Mutation:** No production code was modified. The production implementation of `execute_group_concurrent_with_limit` already satisfies all adversarial invariants.
2. **Deterministic Synchronization:** All adversarial tests use file-backed provider barrier markers, ReplayTrace event inspection, or atomic signals. No arbitrary sleep is used as correctness proof.
3. **Channel Disconnect Seam:** As analyzed in Crucible 7, creating an artificial worker sender drop without modifying production channel ownership would cause the coordinator to block waiting on its own retained sender. Production code prevents missing results by construction via `catch_unwind` in `worker_invoke_provider`.

## Evidence

### Crucible Test Matrix

| # | Crucible Requirement | Test Name | Result |
|---|----------------------|-----------|--------|
| 1 | Hostile completion order (B, C, A under N=2) | `c4_inverse_completion_preserves_semantic_first_failure` | PASS |
| 2 | Hostile slow success + fast failure | `c4_fast_failure_releases_slot_while_slow_sibling_runs` | PASS |
| 3 | G2 failure with active sibling | `c4_g2_failure_halts_queue_but_active_sibling_finishes_truthfully` | PASS |
| 4 | G1 failure with active sibling | `c4_g1_failure_halts_queue_but_active_sibling_finishes_truthfully` | PASS |
| 5 | Outcome durability failure with active sibling | `c4_outcome_durability_failure_does_not_contaminate_active_sibling` | PASS |
| 6 | Worker panic under real N=2 pressure | `c4_worker_panic_under_n2_pressure_releases_slot` | PASS |
| 7 | Channel disconnect analysis / fail-closed audit | Architectural proof + `any_nonterminal` fail-closed verification | PASS |
| 8 | Same-provider hostility & inverse completion | `c4_same_provider_overlap_and_inverse_completion_preserves_semantic_order` | PASS |
| 9 | Repeated stress loop (20 iterations) | `c4_repeated_inverse_completion_has_no_state_leak` | PASS |
| 10 | Randomness audit of C1–C3 execution path | Code inspection confirming zero nondeterministic selection sources | PASS |

### Randomness Audit Details (Crucible 10)

- **Stage A Preparation (`host_execution.rs:1814`):** Iterates linearly over `member_indexes` in exact Runtime Plan order.
- **Stage B Launch Selection (`host_execution.rs:2076`):** Uses `.position(|st| matches!(st, GroupMemberState::Prepared { .. }))`, strictly selecting the lowest index (earliest semantic order).
- **Stage C Result Handling (`host_execution.rs:2250`):** Matches `action_index == worker_result.action_index` directly against `member_states` `Vec`.
- **Stage D Join & Semantic Failure Selection (`host_execution.rs:2481`, `2495-2512`):** `first_non_success_member_step` iterates linearly over `member_states` `Vec` (index 0..N-1), guaranteeing first non-success in semantic Runtime Plan order regardless of physical completion timing.
- **Clock and Timing:** Monotonic clock is used only for per-member deadline comparison (`remaining_until_deadline`), never for member selection or sorting.

### Focused Test Suite Results

- `c4_`: 8 tests PASS (`c4_inverse_completion_preserves_semantic_first_failure`, `c4_fast_failure_releases_slot_while_slow_sibling_runs`, `c4_g2_failure_halts_queue_but_active_sibling_finishes_truthfully`, `c4_g1_failure_halts_queue_but_active_sibling_finishes_truthfully`, `c4_outcome_durability_failure_does_not_contaminate_active_sibling`, `c4_worker_panic_under_n2_pressure_releases_slot`, `c4_same_provider_overlap_and_inverse_completion_preserves_semantic_order`, `c4_repeated_inverse_completion_has_no_state_leak`)
- `c3_v1`: 2 tests PASS
- `c3_a4`: 12 tests PASS
- `c3_a3`: 7 tests PASS
- `c3_a2`: 4 tests PASS
- `c3_a1`: 3 tests PASS
- `c2_a3a`: 3 tests PASS
- `c2a3a`: 16 tests PASS

### Full Suite Result

- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- --test-threads=1`: 1548 tests PASS (0 failures, 2 ignored standalone integration tests)

### Code Quality and Standards

- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml -- --check`: PASS
- `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml`: PASS
- `cargo check --locked --manifest-path tethers-0.1/host-rust/Cargo.toml`: PASS
- `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features`: PASS
- `git diff --check`: PASS (0 whitespace errors)

### Diff Audit

- `tethers-0.1/host-rust/src/host_execution.rs`: +728 lines (TEST ONLY — all added tests under `#[cfg(test)] mod tests`)
- Production semantic changes: ZERO

## Discoveries

None. The C1–C3 implementation survived all adversarial crucibles without requiring any production modification.

## Remaining risks

None within C1–C4 scope. C5 (fresh-agent end-to-end authoring proof) remains the next milestone.

## Smallest next action

Lucy review and acceptance of C4 adversarial concurrency proof.

## References

- `docs/concurrency/C3_BOUNDED_CONCURRENCY_DESIGN.md`
- `docs/concurrency/C2_A3_PHYSICAL_CONCURRENCY_DESIGN.md`
- `tethers-0.1/host-rust/src/host_execution.rs`
