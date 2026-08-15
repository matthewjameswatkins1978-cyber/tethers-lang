# Worker Note: C3-V1 Independent Final Architectural Review

Task: `C3-V1 — Independent Final Architectural Review`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `C3-V1 Proof Gap Correction Agent`

Status: `COMPLETE`

Base commit: `8a09203715cc44f42c011c0c8902ff4f72a246c7`

Implementation checkpoint: `8a09203715cc44f42c011c0c8902ff4f72a246c7`

## Requested outcome

Independent verification that C3 (bounded concurrency) satisfies the accepted design without changing Tethers source semantics, replay truthfulness, deterministic result selection, or coordinator ownership boundaries. Read-only review with test execution and evidence collection.

## Changes made

None. This is a read-only / test-only review. No production code was modified.

## Decisions and assumptions

This is an independent review of frozen C3 implementation code. No design decisions were made. All architectural choices were frozen before this review began. The reviewer treated previous worker reports as untrusted claims and verified all invariants against actual code and test evidence.

## Evidence

### 20-point review matrix

| # | Invariant | Verdict | Evidence |
|---|-----------|---------|----------|
| 1 | Resource bound | PASS | `execute_group_concurrent_with_limit` at `host_execution.rs:1791` bounds exactly one group. No global scheduler. |
| 2 | Configuration | PASS | `runtime_config.rs:32-49` — one optional field, default 2, N>=1 validated, zero rejected, `deny_unknown_fields` preserved. |
| 3 | Stage A | PASS | `host_execution.rs:1808-2063` — serial preparation in Runtime Plan order: scope → policy → resolution → replay admission → G0 → Trail intent. No deadline, no G1, no worker. |
| 4 | Waiting state | PASS | `c3_a2_waiting_member_has_g0_without_g1_or_provider_effect` proves G0=yes, durable intent=yes, G1=no, provider effect=no. |
| 5 | Capacity authority | PASS | `count_active_members` at `host_execution.rs:1712` counts `Launched` and `Transitioning` only. No separate mutable counter. |
| 6 | Admission order | PASS | `member_states.iter().position(Prepared)` at `host_execution.rs:2076` selects earliest semantic-order Prepared member. |
| 7 | Launch boundary | PASS | `host_execution.rs:2121-2162` — deadline start → remaining calculation → final deadline check → G1 → worker launch. Queue wait does not consume timeout. |
| 8 | Worker ownership | PASS | `WorkerInput` at `host_execution.rs:1601` carries only arguments, provider, tool_name, remaining. Workers do not touch Trail, response, replay, approvals, or anchors. |
| 9 | Same-provider concurrency | PASS | `c2_a3a_same_provider_tools_call_overlap_is_real` proves same-provider overlap through independent ephemeral sessions. |
| 10 | Slot release | PASS | `host_execution.rs:2282-2344` — capacity released only after `execute_boundary_invoke_only` completes and state transitions to `Terminal`. |
| 11 | Normal failure | PASS | `c3_a3_normal_provider_failure_releases_slot_and_joins` — failed member terminalises, sibling launches, join evaluates both. |
| 12 | Worker panic | PASS | `c3_a3_worker_panic_terminalises_uncertain_and_releases_slot` — catch_unwind maps to Uncertain, releases slot, no hang. |
| 13 | Fatal trust failure | PASS | `c3_a3_outcome_durability_failure_halts_queued_effects_without_join` and `c3_a3_g1_failure_halts_before_any_later_effect_without_join` — launches_halted prevents new launches. |
| 14 | Incomplete group | PASS | `host_execution.rs:2360-2389` — any nonterminal member prevents GroupJoin, returns AuditFailed. |
| 15 | GroupJoin | PASS | `host_execution.rs:2392-2478` — GroupJoin only after all terminal. All-success test. First non-success by semantic order. |
| 16 | Physical completion inversion | PASS | `c2_a3a_semantic_first_non_success_preserves_exact_step` — different physical orders yield same semantic result. |
| 17 | N equivalence | PASS | `c3_a1_n1_limits_active_invocations_to_at_most_one`, `c3_a1_n2_limits_active_invocations_to_at_most_two_and_reaches_two`, `c3_a1_full_width_preserves_full_overlap` — all N values preserve semantic results. |
| 18 | Replay ownership | PASS | `replay_runtime.rs:60-61` — `FileReplayAuthority` uses `Rc<ReplayLedger>` (not Send). No Arc/Mutex conversion. |
| 19 | Trail | PASS | Trail remains coordinator single-writer (`&mut dyn Trail`). No C3 config metadata schema added. |
| 20 | Channel audit | PASS | `worker_invoke_provider` at `host_execution.rs:1634` — catch_unwind wraps inner, always sends WorkerResult. Channel scoped to group lifetime. |

### Future-proof test matrix

| Design requirement | Test(s) | Verdict | Comment |
|--------------------|---------|---------|---------|
| N=1, group=5 | `c3_v1_n1_group_of_five_proves_bound_and_full_terminalisation` | PASS | Max active exactly 1, all 5 members terminal, join evaluates all five |
| N=2, group=5 | `c3_v1_n2_group_of_five_proves_bound_reached_and_full_terminalisation` | PASS | Max active never exceeds 2, observed max reaches 2, all 5 terminal |
| N>=group size | `c3_a1_full_width_preserves_full_overlap` | PASS | Preserves A3a full overlap |
| Waiting member state | `c3_a2_waiting_member_has_g0_without_g1_or_provider_effect` | PASS | G0 yes, G1 no, provider untouched |
| Queue wait > timeout | `c3_a2_queue_wait_does_not_consume_provider_timeout` | PASS | Timeout starts fresh at launch |
| Earliest semantic order | `c3_a2_next_slot_launches_earliest_semantic_waiter` | PASS | Not completion order |
| Normal provider failure | `c3_a3_normal_provider_failure_releases_slot_and_joins` | PASS | Slot freed, sibling launches |
| Worker panic | `c3_a3_worker_panic_terminalises_uncertain_and_releases_slot` | PASS | Uncertain, no hang |
| Physical completion inversion | `c2_a3a_semantic_first_non_success_preserves_exact_step` | PASS | Semantic order preserved |
| GroupJoin timing | `c3_v1_n1_group_of_five_proves_bound_and_full_terminalisation`, `c3_v1_n2_group_of_five_proves_bound_reached_and_full_terminalisation`, `c3_a3_all_terminal_preserves_group_join` | PASS | Live no-join-while-active/waiting assertions at multiple refill points; join appears only after all terminal |
| Stage C durability failure | `c3_a3_outcome_durability_failure_halts_queued_effects_without_join` | PASS | launches_halted, no join |
| Replay G2 failure | `c3_a3_g2_failure_halts_queued_effects_without_join` | PASS | launches_halted, no join |

### Cumulative production diff audit

Production Rust files changed for C3 (from `f189361e80bdb43c13989200e48513cdb68bd004` to `e3df16e44cbbe295a950faa918b10f19772b9892`):

| File | Lines changed | Classification |
|------|---------------|----------------|
| `configured_runtime.rs` | +58 | REQUIRED BY DESIGN — max_active_together_invocations in PreparedRuntime |
| `dispatch.rs` | +15 | TEST SUPPORT ONLY — outcome_error_signal in RecordingTrail |
| `host_execution.rs` | +2668/-155 | REQUIRED BY DESIGN — bounded launch window, GroupMemberState, WorkerInput/WorkerResult |
| `runtime_config.rs` | +112 | REQUIRED BY DESIGN — config field with validation |

No unexplained production drift. No SUSPECT/UNRELATED changes.

### Focused test results

| Suite | Tests | Result | Time |
|-------|-------|--------|------|
| c3_v1 | 2 | PASS | 9.78s |
| c3_a1 | 3 | PASS | 8.10s |
| c3_a2 | 4 | PASS | 11.75s |
| c3_a3 | 7 | PASS | 14.62s |
| c3_a4 | 12 | PASS | 6.31s |

### Full suite result

`cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- --test-threads=1` — PASS (1540 tests, 0 failures)

### Additional verification

- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml -- --check` — PASS (no formatting changes)
- `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml` — PASS (warnings only, no errors)
- `git diff --check` — PASS (no whitespace issues)

## Discoveries

Documentation numbering inconsistency: the accepted design text says "All 14 future-proof matrix items" in one sentence, but Section 14 currently enumerates 12 numbered requirements. This is a documentation inconsistency only — all 12 enumerated required matrix items have genuine evidence.

## Remaining risks

Deferred C4 adversarial cases:
- Hostile channel-disconnect construction (adversarial transport seam) — explicitly deferred to C4 per design
- Host-wide concurrency across evaluations
- Per-provider concurrency quotas
- Provider rate limits

## Smallest next action

Lucy acceptance of C3-V1 review. No implementation action required.

## References

- `docs/concurrency/C3_BOUNDED_CONCURRENCY_DESIGN.md` — accepted design
- `docs/concurrency/C2_A3_PHYSICAL_CONCURRENCY_DESIGN.md` — frozen A3a foundation
- `tethers-0.1/host-rust/src/host_execution.rs` — main implementation
- `tethers-0.1/host-rust/src/runtime_config.rs` — configuration
- `tethers-0.1/host-rust/src/configured_runtime.rs` — PreparedRuntime
- `tethers-0.1/host-rust/src/dispatch.rs` — Trail and SemanticPosition
- `tethers-0.1/host-rust/src/replay_runtime.rs` — ReplayAdmission
