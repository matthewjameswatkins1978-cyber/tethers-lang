# Worker Note: C3-A2 Resource / Deadline / G1 Crucible

Task: `C3-A2 — Resource / Deadline / G1 Crucible`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `C3-A2 Proof Agent`

Status: `COMPLETE`

Base commit: `c775d2bd335ea295c6567e85def1b8672568fd73`

Implementation checkpoint: `c832ad894f75b3111712534a8223f4dc645829e6`

## Requested outcome

Prove the C3 bounded launch window resource, deadline isolation, G1 boundary,
and semantic-order launch properties through deterministic barrier/replay tests.
C3-A1 is accepted by Lucy. This task may add proof/test support only. Production
scheduler behaviour is frozen. This correction addresses two proof defects in the
original implementation.

## Changes made

- `tethers-0.1/host-rust/src/host_execution.rs` (test code only, no production changes):
  - Added `C3A1GroupHarness::new_with_timeout_overrides()` — applies timeout
    overrides to manifest JSON BEFORE `prepare_runtime()`, ensuring the prepared
    runtime contains the overridden timeout values.
  - Added `C3A1GroupHarness::run_group_with_live_trace()` — accepts an external
    `ReplayTrace` reference, enabling live snapshot while the group thread is running.
  - Refactored `C3A1GroupHarness::new()` to delegate to `new_with_timeout_overrides`
    with empty overrides.
  - Removed dead post-hoc manifest mutation from `run_group_with_trace` (the old
    timeout override wrote to manifest file after prepare_runtime, which was
    ineffective).
  - Corrected `c3_a2_waiting_member_has_g0_without_g1_or_provider_effect` to use
    live `ReplayTrace` snapshot while A is physically active, proving B has G0,
    no G1, and B's durable Trail intent exists during the wait.
  - Corrected `c3_a2_queue_wait_does_not_consume_provider_timeout` to use
    `new_with_timeout_overrides` so the prepared runtime actually contains B's
    500ms timeout. Added explicit assertion that
    `provider_b.capabilities[0].verified_manifest.manifest().timeout_ms == 500`.
    Replaced string search for "Unattempted" with structured outcome assertion.
  - Preserved `c3_a2_next_slot_launches_earliest_semantic_waiter` and
    `c3_a2_queued_member_replay_order_is_g0_wait_g1_g2` unchanged.
- `docs/CURRENT_CLINE_TASK.md`: Updated task packet to C3-A2 corrected specification.

## Decisions and assumptions

- Timeout overrides are now applied at manifest construction time (before
  `prepare_runtime`), following the existing `TerminalHarnessBuilder` pattern.
- `run_group_with_live_trace` accepts an external `ReplayTrace` so the test
  thread can snapshot shared Arc<Mutex<Vec<String>>> state while the group
  thread is still executing. This avoids the unsound approach of trying to
  access `ScopedJoinHandle` fields before join.
- The `timeout_overrides` parameter was removed from `run_group_with_trace`
  since it is no longer needed; the new constructor handles overrides.
- Queue-wait timeout proof uses 500ms member timeout vs 1500ms hold duration.

## Evidence

- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_a2 --test-threads=1`:
  4 passed, 0 failed.
  - `c3_a2_waiting_member_has_g0_without_g1_or_provider_effect` — PASS (live trace
    proves G0(B) present and G1(B) absent while A is active; Trail proves B's
    durable intent during wait)
  - `c3_a2_queue_wait_does_not_consume_provider_timeout` — PASS (prepared runtime
    verified to contain 500ms timeout; B succeeds after 1500ms queue wait)
  - `c3_a2_next_slot_launches_earliest_semantic_waiter` — PASS
  - `c3_a2_queued_member_replay_order_is_g0_wait_g1_g2` — PASS
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_a1 --test-threads=1`:
  3 passed, 0 failed (no regressions).
- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml -- --check` — clean.
- `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml` — clean.
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- --test-threads=1`:
  1519 passed, 0 failed, 5 ignored (full suite, no regressions).
- `git diff --check` — clean (CRLF informational only).
- `.github/scripts/check-tethers-task-packet.ps1` — `PASS control-v1/COMPLETE`.

## Publication evidence

Branch `feature/c3-bounded-window-proof` pushed normally to origin.
Implementation checkpoint `c832ad894f75b3111712534a8223f4dc645829e6`.

## Discoveries

None.

## Remaining risks

None known within packet scope. C3-A3/A4 remain NOT authorised.

## Smallest next action

Submit corrected C3-A2 proofs to Lucy for Red architectural review and acceptance.

## References

- `tethers-0.1/host-rust/src/host_execution.rs`
- `docs/concurrency/C3_BOUNDED_CONCURRENCY_DESIGN.md`
- `docs/worker-notes/2026-08-15-c3-a1-bounded-launch-window.md`
