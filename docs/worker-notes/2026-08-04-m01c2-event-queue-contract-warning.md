Task: `M01C2 - Event-queue contract warning cleanup`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `2bbbe3c84d65d5610dd417b00e0a8c711904ecf7`
Implementation checkpoint: TBD

## Requested outcome

Remove the warning in `tethers-0.1/host-rust/src/event_queue.rs` caused by the misleading non-proving `!Send` test, replacing it with a truthful compile-time `Send` assertion, while preserving the real J10 queue contract (FIFO, coordinator-driven serial evaluation, no recursion, no retry, no parallel worker).

## Changes made

- `tethers-0.1/host-rust/src/event_queue.rs` (test module only): replaced the
  `queue_cannot_be_moved_across_threads` test with
  `queue_value_is_send_under_current_representation`. The replacement defines a
  local `assert_send<T: Send>()` helper **and calls**
  `assert_send::<ResultEventQueue>()`, so the nested helper is now used and the
  `dead_code` warning it previously triggered is eliminated. Test comments now
  state explicitly that `Send` only means the value may be moved between
  threads, that Tethers still evaluates queued Result Anchors one at a time via
  the coordinator's `pop_front` drain, and that seriality is an execution
  design, not a fake `!Send` marker. No production queue field, method,
  visibility, ordering, or ownership behaviour was changed. The event-queue
  test count is unchanged (9 tests; one test renamed, none added or removed).

## Decisions and assumptions

- The packet and blueprint authorise changing only the misleading test and
  forbid modifying production queue code, adding `!Send` markers, or adding
  `#[allow]`/`#[expect]`. The repair is therefore limited to the test-caused
  warning.
- OpenCode LSP was intentionally not retried; M01C1 already proved it exposes
  operations but returns no usable Rust workspace results under the frozen
  configuration. Reference discovery for this change was unnecessary because the
  target was a self-contained test in the edited file.
- `ResultEventQueue: Send` holds under the current representation
  (`VecDeque<ResultAnchor>`, and `ResultAnchor` is composed only from `Send`
  types), so the compile-time assertion is truthful and compiles cleanly.

## Warning evidence before and after

Captured with `cargo clippy --manifest-path tethers-0.1/host-rust/Cargo.toml
--all-targets --all-features --locked --message-format=json`.

| Metric | Before | After |
|--------|--------|-------|
| Total emitted warnings (JSON) | 120 | 119 |
| Warnings whose primary span is `src/event_queue.rs` | 2 | 1 |
| New warnings outside target | 0 | 0 |
| New suppression attributes | 0 | 0 |

Warnings whose primary span is `src/event_queue.rs`:

- Before:
  1. `dead_code` — `function assert_send is never used` (line 200). **Caused by
     the false test** (nested helper defined but never called). → **Removed** by
     calling the helper.
  2. `dead_code` — `methods is_empty and len are never used` (lines 42/47,
     emitted in the **lib target**). This is a **pre-existing production-code**
     warning: production code drains the queue via `pop_front` and never calls
     `is_empty`/`len`; those methods are only exercised by the `#[cfg(test)]`
     module, which is not compiled in the lib target. It is **not caused by the
     false test** and is outside M01C2's permitted scope (fixing it would
     require modifying production queue code, which the packet forbids).
- After: only item 2 (the production `is_empty`/`len` `dead_code`) remains; item
  1 is gone.

Because `cargo clippy --all-targets` compiles the same source for the lib and
test targets, the unique `is_empty`/`len` source warning is emitted once per
target that compiles the lib; the count above reflects unique source warnings
grouped by primary span.

## Focused Nextest evidence

Command:

```text
cargo nextest run --config-file .config/nextest.toml \
  --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets \
  --all-features --locked -E 'test(event_queue::)'
```

Result: `9 tests run: 9 passed, 1124 skipped`, 0 failed, 0 retries. The filtered
set lists exactly the 9 event-queue tests including the renamed
`queue_value_is_send_under_current_representation` (the `assert_send`
compile-time assertion executes and passes as part of the test).

## Final Cargo evidence

`just verify` (packet checker, Rustfmt, `cargo check`, `cargo test
--all-targets --all-features --locked`) passes. Cargo test total: 926 passed, 0
failed — unchanged from the accepted M01C1 baseline. (Full Nextest, cargo-deny,
cargo-machete, and `just verify-agent` were intentionally not run: this is a
test-and-comment-only change and those scans cannot add relevant evidence;
Cargo.toml, Cargo.lock, deny policy, and all dependency metadata are forbidden
from changing, and the diff confirms they did not change.)

## Tool usefulness

| Tool | Used | Useful | Notes |
|------|------|--------|-------|
| cargo clippy | Yes | Yes | Authoritative warning evidence; the JSON capture let us separate the test-caused `assert_send` warning from the unrelated production `is_empty`/`len` warning by primary span and target. |
| cargo nextest | Yes | Yes | Clear per-test focused feedback; zero-retry root config; confirmed the renamed test compiles and passes. |
| cargo (plain `test`) | Yes | Yes | Final authority via `just verify`; 926/926 unchanged. |
| OpenCode LSP | No | N/A | Intentionally not retried; proven ineffective in M01C1. |

## Discoveries

- The baseline `event_queue.rs` carries **two** `dead_code` warnings, not one.
  The packet and blueprint anticipated only the test-caused `assert_send`
  warning. The second warning (`is_empty`/`len` never used, lib target) is a
  pre-existing production-code `dead_code` unrelated to the false test.
- Removing the production `is_empty`/`len` warning is not achievable within
  M01C2's permitted files and forbidden-change rules. Permitted options would be
  either deleting the methods (modifying production code — forbidden) or adding
  `#[allow(dead_code)]` (forbidden). It must be addressed by a separate task or
  with explicit authorisation.

## Remaining risks

- The `is_empty`/`len` `dead_code` warning remains in `event_queue.rs`. It is
  benign (the methods are public API intended for future coordinator use and are
  exercised by the unit tests) but does mean the literal acceptance criterion
  "zero warnings in `event_queue.rs`" is not met by a permitted-only change.
  This is a scoping limitation of M01C2, not a regression introduced by the
  change.
- No runtime, dependency, protocol, CLI, language, Plug, Trail, replay,
  admission, or concurrency behaviour changed.

## Smallest next action

Lucy reviews the pushed M01C2 evidence. Recommended follow-up (separate task):
decide whether the production `is_empty`/`len` `dead_code` on
`ResultEventQueue` should be removed (by either using the methods in the
coordinator drain or narrowing their visibility), since M01C2 is forbidden from
touching production queue code.

## References

- Packet: `docs/CURRENT_CLINE_TASK.md`
- Blueprint: `docs/architecture/M01C2_EVENT_QUEUE_CONTRACT_WARNING_CLEANUP.md`
- J10 queue contract: `docs/worker-notes/2026-07-27-j10-result-event-queue.md`
- M01C1 worker note: `docs/worker-notes/2026-08-04-m01c1-engine-session-warning-pilot.md`
- Implementation commit: TBD (recorded after commit)
- Branch: `opencode/m01c2-event-queue-contract-warning`
