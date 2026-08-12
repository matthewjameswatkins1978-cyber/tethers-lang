# Worker Note

- **Task Packet:** TETHERS R1 — Retained Replay Authority
- **Owner:** OpenCode
- **Status:** `COMPLETE`
- **Base Commit:** `83a1f3aa74b2d0534cdb11fcd9ad7c848dc5cc0d`
- **Implementation checkpoint:** `40751ffffed7dbfdcd22e2a67b4733c5a5c4f72c`
- **Branch:** `perf/r1-retained-replay-authority`

## Requested outcome

Retain the existing `FileReplayAuthority` across the `run_selected` evaluation loop
instead of creating a new one per Action. This eliminates the per-Action
`ReplayLedger::open` / `validate_whole_ledger` cost that PF1 measured as the
growing stage in retained evaluation runs.

## Changes made

- `tethers-0.1/host-rust/src/host_execution.rs` — 4 functions changed:
  - `run_with_tether_indexes`: creates one `FileReplayAuthority` before the evaluation loop, passes `&mut` through
  - `evaluate_one`: new `replay_authority: &mut dyn ReplayAuthority` parameter, forwarded to `dispatch_matched_plan`
  - `dispatch_matched_plan`: new `replay_authority: &mut dyn ReplayAuthority` parameter, forwarded to `execute_one_action`
  - `execute_one_action`: new `replay_authority: &mut dyn ReplayAuthority` parameter; removed per-Action `FileReplayAuthority::new(self.host_data_root)`; passes authority directly to `execute_shared_boundary`
  - `bench_evaluate_one`: new `replay_authority: &mut dyn ReplayAuthority` parameter, forwarded to `evaluate_one`
  - Two test call sites in `c1c1_present_non_array_plan_groups_fails_closed_before_dispatch` updated to pass `FileReplayAuthority::new(None)`
- `tethers-0.1/host-rust/src/bin/bench_retained.rs` — creates `FileReplayAuthority` before warmup loop, passes to both warmup and measured `bench_evaluate_one` calls
- `tethers-0.1/host-rust/src/bin/bench_prod.rs` — creates `FileReplayAuthority` before warmup loop, passes to both warmup and measured `bench_evaluate_one` calls

## Decisions and assumptions

- The `application.rs` `authorise_and_execute_with_writer` path (MCP follow-up) was not modified — it is a separate execution path outside the `run_selected` evaluation loop.
- Test call sites pass `FileReplayAuthority::new(None)` since those tests fail before replay admission is reached.
- `&mut dyn ReplayAuthority` used for the parameter type to match the existing `execute_shared_boundary` signature.
- `#[allow(clippy::too_many_arguments)]` already present on `execute_one_action` accommodates the additional parameter.

## Evidence

### Formatting
```
cargo fmt --all -- --check — PASS (no output)
```

### Compilation
```
cargo check --all-targets --all-features — PASS (Finished in 0.30s)
```

### Clippy
```
cargo clippy --all-targets --all-features — PASS
(all warnings pre-existing, none from R1 changes)
```

### Tests
```
cargo test --all-targets --all-features — PASS
1451 passed; 0 failed; 2 ignored
```

### Diff
```
git diff --check — PASS (no output)
3 files changed, 22 insertions(+), 2 deletions(-)
```

### Structural proof
The `FileReplayAuthority` is now created exactly once in `run_with_tether_indexes` (line ~587) and threaded through the entire evaluation loop. The `ledger: Option<ReplayLedger>` field inside `FileReplayAuthority` is lazily initialized on first `admit()` and reused for all subsequent admissions within the same retained run. `ReplayLedger::open` / `validate_whole_ledger` now occurs once per retained run, not once per Action.

## Publication evidence

Not yet pushed. Push pending Matthew/Lucy direction.

## Discoveries

- `FileReplayAuthority` was already designed to retain `ReplayLedger` between admissions via its `ledger: Option<ReplayLedger>` field. Production merely destroyed it after each Action by creating a new instance. R1 makes production use the existing abstraction as intended.
- The `application.rs` `authorise_and_execute_with_writer` path creates its own `FileReplayAuthority` — this is correct for its separate MCP follow-up flow.

## Remaining risks

- PF1 benchmark has not been rerun on this branch. The structural change is verified; the performance proof requires the PF1 retained P10 benchmark to confirm `replay_admit` growth disappears.
- C2 future note: the `&mut ReplayAuthority` boundary is fine for C1 serial execution. When physical Together concurrency arrives, the mutable authority boundary will need revisiting.

## Smallest next action

Rerun PF1 retained P10 benchmark against this branch to prove `replay_admit` no longer grows with retained history.

## References

- Branch: `perf/r1-retained-replay-authority`
- Implementation SHA: `40751ffffed7dbfdcd22e2a67b4733c5a5c4f72c`
- Base: `83a1f3aa74b2d0534cdb11fcd9ad7c848dc5cc0d` (perf/b0-original-baseline)
- `FileReplayAuthority` struct: `tethers-0.1/host-rust/src/replay_runtime.rs:60-76`
- `ReplayAuthority` trait: `tethers-0.1/host-rust/src/replay_runtime.rs:52-58`
- `execute_shared_boundary`: `tethers-0.1/host-rust/src/application.rs:2006-2043`
