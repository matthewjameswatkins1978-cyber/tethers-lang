# Worker Note: C3-A2 Resource / Deadline / G1 Crucible

Task: `C3-A2 — Resource / Deadline / G1 Crucible`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `C3-A2 Proof Agent`

Status: `COMPLETE`

Base commit: `c775d2bd335ea295c6567e85def1b8672568fd73`

Implementation checkpoint: `9388f86459145551d843442ff72018427238ca96`

## Requested outcome

Prove the C3 bounded launch window resource, deadline isolation, G1 boundary,
and semantic-order launch properties through deterministic barrier/replay tests.
C3-A1 is accepted by Lucy. This task may add proof/test support only. Production
scheduler behaviour is frozen.

## Changes made

- `tethers-0.1/host-rust/src/host_execution.rs` (test code only, no production changes):
  - Added `C3A1GroupHarness::trail_content()` for raw Trail content access.
  - Added `C3A1GroupHarness::run_group_with_trace()` — executes group with
    `ObservingReplayAuthority` + `ReplayTrace` for replay event observation,
    plus optional per-member timeout overrides via manifest mutation on disk.
  - Added four focused C3-A2 proof tests (see Evidence).
- `docs/CURRENT_CLINE_TASK.md`: Updated task packet to C3-A2 specification.

## Decisions and assumptions

- Reused existing `ObservingReplayAuthority` and `ReplayTrace` for replay event
  observation; no new test machinery required.
- Per-member timeout overrides are achieved by mutating the manifest JSON on disk
  before runtime preparation. This is test-only; production manifest loading is unchanged.
- Queue-wait timeout proof uses 500ms member timeout vs 1500ms hold duration
  to ensure reliable proof on Windows without brittle sub-100ms timing.
- `run_group_with_trace` calls `execute_group_concurrent_with_limit` directly
  (not wrapped in `std::thread::scope`) because `sessions` HashMap contains
  non-Send `RetainedProviderSession`.

## Evidence

- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_a2 --test-threads=1`:
  4 passed, 0 failed.
  - `c3_a2_waiting_member_has_g0_without_g1_or_provider_effect` — PASS
  - `c3_a2_queue_wait_does_not_consume_provider_timeout` — PASS
  - `c3_a2_next_slot_launches_earliest_semantic_waiter` — PASS
  - `c3_a2_queued_member_replay_order_is_g0_wait_g1_g2` — PASS
- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml -- --check` — clean.
- `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml` — clean.
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- --test-threads=1`:
  1519 passed, 0 failed, 5 ignored (full suite, no regressions).
- `git diff --check` — clean (CRLF informational only).
- `.github/scripts/check-tethers-task-packet.ps1` — `PASS control-v1/COMPLETE`.

## Publication evidence

Branch `feature/c3-bounded-window-proof` pushed normally to origin.
Implementation checkpoint `9388f86459145551d843442ff72018427238ca96`.

## Discoveries

None.

## Remaining risks

None known within packet scope. C3-A3/A4 remain NOT authorised.

## Smallest next action

Submit C3-A2 proofs to Lucy for Red architectural review and acceptance.

## References

- `tethers-0.1/host-rust/src/host_execution.rs`
- `docs/concurrency/C3_BOUNDED_CONCURRENCY_DESIGN.md`
- `docs/worker-notes/2026-08-15-c3-a1-bounded-launch-window.md`
