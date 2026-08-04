# M01C3 Event Queue Dead API Cleanup

Status: frozen implementation blueprint

## Purpose

Remove the remaining `dead_code` warning in `tethers-0.1/host-rust/src/event_queue.rs` by deleting the unused `ResultEventQueue::is_empty` and `ResultEventQueue::len` methods and rewriting their test-only assertions through the queue's real operational surface.

This is a narrow internal API cleanup. It does not alter FIFO ordering, coordinator ownership, admission, follow-up evaluation, retry behaviour, or concurrency design.

## Accepted baseline

M01C2 is accepted on `main` at `21671b06365f28923d7375005d9b14d9559b71a4`.

```text
Rust             1.97.1
Cargo tests      926 passing
Nextest tests    1133 passing
Nextest retries  0
Clippy emitted   119 warnings
Cargo.lock       D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB
```

M01C2 removed the false test-caused warning and left one unique source warning in `event_queue.rs`: `ResultEventQueue::is_empty` and `ResultEventQueue::len` are unused in the production library target.

OpenCode LSP must not be retried. M01C1 already proved it ineffective for this workspace. Exact Clippy diagnostics and one bounded `rg` reference check are sufficient.

## Current design

`ResultEventQueue` wraps a private `VecDeque<ResultAnchor>` and exposes:

- `new`;
- `enqueue`;
- `pop_front`;
- `is_empty`;
- `len`.

Production code creates, fills, and drains the queue through `new`, `enqueue`, and `pop_front`. The coordinator uses:

```rust
while let Some(anchor) = queue.pop_front() {
    // admit and evaluate one anchor
}
```

The `is_empty` and `len` methods are used only by colocated tests. They do not participate in production behaviour and create warning debt in the library target.

## Required repair

1. Capture the Rust 1.97.1 Clippy baseline before editing.
2. Confirm with one exact text-reference pass that `ResultEventQueue::is_empty` and `ResultEventQueue::len` have no non-test callers in `tethers-0.1/host-rust/src`.
3. Delete the two unused methods and their documentation comments.
4. Update only the existing colocated queue tests that call those methods:
   - replace empty-state assertions with `pop_front().is_none()` only where the queue is expected to be exhausted;
   - preserve all FIFO and no-retry assertions;
   - do not expose or inspect the private `pending` field.
5. Keep the event-queue test count exactly nine.
6. Preserve the M01C2 compile-time assertion that `ResultEventQueue: Send` under its current representation.
7. Remove every warning whose primary span is `src/event_queue.rs`.

## Forbidden repair shapes

- Do not keep the methods behind `#[cfg(test)]` merely to hide the production warning.
- Do not add `#[allow(...)]`, `#[expect(...)]`, underscore renaming, dummy calls, `black_box`, unreachable code, or source-text theatre.
- Do not alter `ResultEventQueue` fields, `VecDeque`, `enqueue`, `pop_front`, `Default`, visibility of the type, or queue ownership.
- Do not change the coordinator drain to call `is_empty` or `len` simply to manufacture a production use.
- Do not add threads, async code, channels, locks, retry loops, worker pools, or concurrency markers.
- Do not modify dependencies, Cargo.lock, tool configuration, deny policy, Nextest policy, or Just recipes.

## Permitted files

Only:

- `tethers-0.1/host-rust/src/event_queue.rs`;
- `docs/CURRENT_CLINE_TASK.md` for state and checkpoint;
- `docs/worker-notes/2026-08-04-m01c3-event-queue-dead-api-cleanup.md`.

## Behavioural invariants

- FIFO remains `push_back` / `pop_front`.
- Generated children remain appended to the tail.
- The queue remains process-local and in-memory.
- The coordinator remains the only production drainer.
- One event remains processed at a time.
- No recursion, retry, thread, async runtime, parallel worker, persistence, or scheduling policy is added.
- Result Anchor identity, correlation, causation, generation, admission, replay, dispatch, and serialization remain unchanged.
- No public CLI, language, Plug, Trail, capability, or protocol behaviour changes.

## Tool use

### Reference check

Use one bounded exact search before editing. Record every source occurrence of:

```text
.is_empty()
.len()
ResultEventQueue
```

Distinguish the queue methods from unrelated collections. If either method has a real non-test caller, stop before editing because the frozen premise is false.

Do not retry LSP.

### Clippy

Clippy is the warning authority. Capture machine-readable output before and after and compare by lint code, file, target, and primary span.

Expected before state from M01C2:

- total emitted warnings: 119;
- one unique `event_queue.rs` warning covering unused `is_empty` and `len` methods.

Actual diagnostics are authoritative.

### Nextest

Use Nextest only for the focused edit loop. Run the nine `event_queue` tests with the committed root configuration and zero retries.

### Cargo

Run `just verify` once after focused checks. Ordinary Cargo remains final authority. Expected Cargo floor is 926 passed, zero failed. No test may disappear.

### Dependency tools

Do not run full Nextest, cargo-deny, cargo-machete, or `just verify-agent`. This task cannot change dependency or tool metadata. Confirm forbidden paths are absent from the diff instead.

## Warning accounting

Record before and after:

- total emitted warning messages;
- warning counts by lint code;
- unique warnings whose primary span is `src/event_queue.rs`;
- warnings outside the target file.

Acceptance requires:

- zero warnings whose primary span is `src/event_queue.rs`;
- total emitted warnings lower than the captured baseline;
- no new or changed warning outside the target;
- no suppression attribute or fake production use.

Because Cargo may emit the same source diagnostic for more than one target, report both emitted-message totals and unique source warnings.

## Verification floor

Run only evidence-bearing checks:

1. task packet checker;
2. clean-worktree and branch checks;
3. one bounded exact reference search;
4. Clippy before and after, including JSON capture;
5. Rustfmt;
6. focused Nextest event-queue tests with zero retries;
7. `just verify` once;
8. Cargo.lock hash;
9. diff and status checks.

## Stop conditions

Stop only for a genuine scope contradiction:

- either supposedly dead method has a non-test source caller;
- the accepted warning is no longer present before editing;
- removing it requires changing production behaviour outside `event_queue.rs`;
- an evidence-bearing verification exposes a real failure that cannot be corrected within the permitted files.

Do not stop for ineffective LSP, skipped dependency scans, ordinary warning duplication across targets, or a harmless line-number drift.

## Completion evidence

The worker note must record:

- exact reference-search result;
- exact baseline warning and target duplication, if any;
- deleted methods;
- exact test assertions rewritten;
- focused Nextest command and 9/9 result;
- final Cargo total;
- before/after warning table;
- unchanged Cargo.lock hash;
- exact changed files;
- confirmation that coordinator and runtime behaviour are untouched;
- smallest next action.
