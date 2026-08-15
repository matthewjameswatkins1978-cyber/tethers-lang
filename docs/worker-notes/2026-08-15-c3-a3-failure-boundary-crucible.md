# Worker Note — C3-A3 Failure-Boundary Crucible

Task: `C3-A3 — Failure-Boundary Crucible`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `C3-A3 Failure-Boundary Agent`
Status: `COMPLETE`
Base commit: `d8b094c5d89f78cb5b610f5367f098f6cc0ef277`
Implementation checkpoint: `6df1c55416ee7cb24ee236db1ec427250f9e8eab`

## Requested outcome

1. Correct the fatal-halt GroupJoin defect in `tethers-0.1/host-rust/src/host_execution.rs` so that when any member remains nonterminal after Stage B/C, no `GroupJoinEntry` and no `group_joined` response presentation are appended, failing closed through deterministic existing non-success/infrastructure taxonomy.
2. Prove that normal provider failure releases launch capacity and queued siblings continue to launch and join (Proof 1).
3. Prove that worker panic caught via `PanicGuard` terminalises as `Uncertain`, releases launch capacity, and queued siblings continue to launch and join (Proof 2).
4. Prove that Stage C OutcomeEntry durability failure halts queued launches without appending a `GroupJoinEntry` (Proof 3).
5. Prove that replay G2 publication failure halts queued launches without appending a `GroupJoinEntry` (Proof 4).
6. Prove that replay G1 publication failure halts before any provider effect and without appending a `GroupJoinEntry` (Proof 5).
7. Prove all-terminal groups continue to produce standard GroupJoin entries and responses (Proof 6 regression).
8. Perform channel-failure audit on worker/coordinator contract.

## Changes made

- In `tethers-0.1/host-rust/src/host_execution.rs`:
  - Added Stage C audit failure detection on `execute_boundary_invoke_only` response trail, setting `result.outcome = SharedExecutionOutcome::AuditFailed` and `launches_halted = true` on audit failure or `ReplayPersistenceUnavailable`.
  - Added narrow fail-closed pre-Stage-D guard before GroupJoin publication in `execute_group_concurrent_with_limit`: if `any_nonterminal` is true (one or more semantic members remain in `Prepared`, `Launched`, or `Transitioning`), GroupJoin publication and `group_joined` presentation are skipped, returning the first semantic terminal non-success via `first_non_success_member_step` or failing closed with `AuditFailed`.
  - Extended test-only `ObservingReplayAuthority` and `ObservingAdmission` with `with_fail_points` to support deterministic G1 armed and G2 terminal failure injection for selected action IDs.
  - Added test helper methods on `C3A1GroupHarness`: `set_member_outcome`, `run_group_with_trail_and_authority`, `run_group_with_authority`.
  - Implemented 6 deterministic tests in `host_execution.rs`:
    - `c3_a3_normal_provider_failure_releases_slot_and_joins`
    - `c3_a3_worker_panic_terminalises_uncertain_and_releases_slot`
    - `c3_a3_outcome_durability_failure_halts_queued_effects_without_join`
    - `c3_a3_g2_failure_halts_queued_effects_without_join`
    - `c3_a3_g1_failure_halts_before_any_later_effect_without_join`
    - `c3_a3_all_terminal_preserves_group_join`

## Decisions and assumptions

- **GroupJoin Invariant**: GroupJoin exists only when every semantic group member has reached its legitimate terminal state (`Terminal` or `PreparationTerminal`). Queued `Prepared` members following a fatal trusted-state halt are not terminal, and terminal states are never fabricated for them.
- **Fail-closed Result Aggregation**: When execution halts with nonterminal members, the first legitimate terminal non-success in semantic Runtime Plan order is aggregated. If no terminal non-success exists, `ExecutionServiceResult::AuditFailed` is returned.
- **Channel Failure Audit**: The current worker contract executes within `std::thread::scope` where `worker_invoke_provider` wraps execution in `catch_unwind` and always attempts `tx.send(WorkerResult)`. A missing `WorkerResult` without whole-process death is not constructible through the normal worker path without deliberately hanging a thread or modifying channel transport; adversarial channel failure testing is deferred to C4.

## Evidence

- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_a3 --test-threads=1`: PASS (6/6 tests passed)
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_a2 --test-threads=1`: PASS (4/4 tests passed)
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_a1 --test-threads=1`: PASS (3/3 tests passed)
- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml -- --check`: PASS (clean formatting)
- `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml`: PASS (compiles clean)
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- --test-threads=1`: PASS (all 1525 lib tests + all 23 binary/integration suites passed with 0 failures)
- `git diff --check`: PASS (clean diff, no whitespace errors)

## Discoveries

- The test-only `RecordingTrail` is `!Send` because it retains an internal `Rc<RefCell<Vec<&'static str>>>` events log. Constructing it within the scoped thread closure avoids crossing thread boundaries while allowing full inspection of `outcome_entries` and `group_join_entries`.

## Remaining risks

- None within C3-A3 scope. External configuration and CLI exposure remain for C3-A4.

## Smallest next action

- Await Lucy review and acceptance of C3-A3 on published branch `feature/c3-failure-boundary`.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/concurrency/C3_BOUNDED_CONCURRENCY_DESIGN.md`
- `tethers-0.1/host-rust/src/host_execution.rs`
