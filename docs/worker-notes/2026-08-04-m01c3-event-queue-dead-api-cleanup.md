Task: `M01C3 - Event-queue dead API cleanup`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `170063ea24b3ba4ba5529749ae6fc615e7c58de6`
Implementation checkpoint: `a145714f47ee04e729e6dfbb2419521aa95e7bbb`
Branch: `opencode/m01c3-event-queue-dead-api` (created from `origin/main` at `170063ea`; implementation commit `a145714`)

## Requested outcome

Delete the unused public `ResultEventQueue::is_empty` and `ResultEventQueue::len`
methods from `tethers-0.1/host-rust/src/event_queue.rs` and rewrite every caller
(colocated queue tests and the `application.rs` `#[cfg(test)]` callers) through
`pop_front()`, finishing with zero warnings whose primary span is
`src/event_queue.rs`.

## Changes made

- `tethers-0.1/host-rust/src/event_queue.rs`:
  - Deleted `pub fn is_empty(&self)` and `pub fn len(&self)` and their docs.
  - Rewrote colocated queue tests (9 tests preserved) so emptiness/exhaustion is
    asserted only through `pop_front().is_none()`:
    - `empty_queue_is_empty_and_pops_nothing`: asserts `pop_front()` is `None`.
    - `enqueue_then_pop_front_returns_fifo_order`: dropped the redundant
      `len()==3`; FIFO order still asserted via the three `pop_front().unwrap()`
      calls then `pop_front().is_none()`.
    - `enqueue_during_draining_appends_to_tail`,
      `child_never_jumps_ahead_of_existing_sibling`,
      `failed_pop_does_not_reinsert`: `is_empty()` replaced by
      `pop_front().is_none()`.
  - Preserved `new`, `enqueue`, `pop_front`, `Default`, the private `pending`
    field, and the `ResultEventQueue: Send` compile-time assertion (test 7).
- `tethers-0.1/host-rust/src/application.rs` (test module only, no production
  code): rewrote the `queue.is_empty()` / `queue.len()` assertions to drain via
  `pop_front` instead of inspecting length:
  - `j10_queued_processing_error_stops_later_events` (was 7150): pop the single
    remaining anchor, assert its `event_id == "never/result"`, then `is_none()`.
  - `j10_failed_item_is_not_retried` (7163): `pop_front().is_none()`.
  - `j10_coordinator_stops_on_error_without_reinsert_or_retry` (7236): pop the
    remaining anchor, assert `event_id == "c/result"`, then `is_none()`.
  - `shared_queue` J11 anchor-chain test (7407/7413 and 7448/7451/7457): folded
    the `len()==1` checks into the existing `pop_front().expect(...)` of A and B,
    asserting `pop_front().is_none()` after each so exactly one anchor is present
    and the queue is empty before the next dispatch.
  - `j11_duplicate_sibling_not_evaluated_twice` (7605): `pop_front().is_some()` +
    `pop_front().is_none()` (= exactly one remaining).
  - `j11_causal_depth_rejection` (7834), `j11_rejection_produces_no_anchor` (8025),
    `j11_packet4_dup_recorded` (8485): pop the remaining anchor, assert its
    `event_id`, then `is_none()`.
  - `j11_completed_follow_ups_preserved_before_rejection` (7963) and the sibling
    follow-up test (8160): `pop_front().is_none()`.

## Decisions and assumptions

- Lucy widened the permitted-files scope (option 1) to include the `application.rs`
  `#[cfg(test)]` callers; the packet was updated accordingly before editing.
- Did not retry OpenCode LSP (proven ineffective in M01C1; carried forward by the
  packet and blueprint).
- Did not add any public `is_empty`/`len` replacement, `#[allow]`, `#[expect]`,
  dummy caller, or `#[cfg(test)]` shim. Emptiness is observed only by draining.
- Rewrites preserve every original behavioural assertion: FIFO order, exactly-one
  anchor between dispatches, the specific remaining `event_id`s, and exhaustion.

## Evidence

Reference proof (bounded `rg`, pre-edit): `is_empty`/`len` had no non-test
(production) caller; production drains only via `pop_front`. The only callers were
the colocated queue tests and `application.rs` `#[cfg(test)]` callers — both now
rewritten.

Verification (all run on branch `opencode/m01c3-event-queue-dead-api`):

- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`:
  exit 0 (clean).
- Focused Nextest
  `cargo nextest run ... -E 'test(event_queue::)'`: `9 passed, 0 failed, 0
  retries` — all nine event-queue tests run and pass; none removed.
- Clippy (`--all-targets --all-features --locked`, JSON counted):
  - before (M01C2 baseline): **119 total warnings**, **1 unique source warning**
    in `src/event_queue.rs` (dead_code on `is_empty`/`len`).
  - after: **118 total warnings**, **0 warnings** whose primary span is
    `src/event_queue.rs`.
  - total decreased by exactly 1 (the removed dead_code warning); no new or
    changed warning outside the target.
- `just verify` (packet checker + fmt + `cargo check` + full `cargo test
  --all-targets --locked`): exit 0 — full suite green, 0 failed, no test missing.
- `Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256`:
  `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB`
  (unchanged from M01C2 baseline).
- `git diff --check`: exit 0. Diff is localized (application.rs 31/27,
  event_queue.rs 35/44); no line-ending flood. Git's `LF will be replaced by CRLF`
  note is the normal checkout-normalization informational message, not a content
  change.

## Discoveries

- M01C3's original premise ("test conveniences only, used solely by the colocated
  queue tests") was incomplete: `application.rs` `#[cfg(test)]` also depended on
  `is_empty`/`len`. The clean fix was Lucy's option 1 (widen scope to the
  `application.rs` test callers), now applied.
- `drain_result_event_queue` stops draining when an admission rejection occurs,
  leaving remaining anchors in the queue; the rewritten `len()` checks correctly
  fold into the existing `pop_front().expect(...)` of the known remaining anchor.

## Remaining risks

- None for this task. The queue's public surface is now `new`, `enqueue`,
  `pop_front`, `Default` only; coordinator and production runtime behaviour are
  unchanged.

## Smallest next action

None. Task complete; ready for Lucy's independent review.

## References

- Packet: `docs/CURRENT_CLINE_TASK.md`
- Blueprint: `docs/architecture/M01C3_EVENT_QUEUE_DEAD_API_CLEANUP.md`
- M01C2 worker note: `docs/worker-notes/2026-08-04-m01c2-event-queue-contract-warning.md`
- Relevant source: `tethers-0.1/host-rust/src/event_queue.rs`,
  `tethers-0.1/host-rust/src/application.rs`
- Implementation commit: `a145714f47ee04e729e6dfbb2419521aa95e7bbb`
- Branch: `opencode/m01c3-event-queue-dead-api`
