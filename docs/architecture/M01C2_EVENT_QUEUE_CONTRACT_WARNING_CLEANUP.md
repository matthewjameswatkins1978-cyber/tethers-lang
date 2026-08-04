# M01C2 Event Queue Contract Warning Cleanup

Status: frozen implementation blueprint

## Purpose

Remove the remaining warning in `tethers-0.1/host-rust/src/event_queue.rs` by correcting a misleading test, while preserving the real J10 queue contract: FIFO, one-at-a-time coordinator-driven evaluation, no recursion, no retry, and no parallel worker.

This is a test-and-comment correction. It is not a runtime queue redesign and does not make the queue artificially `!Send`.

## Accepted baseline

M01C1 is accepted on `main` at `2fbebfc14c8e2c55990f2bdfc8e85830da339b65`.

```text
Rust             1.97.1
Cargo tests      926 passing
Nextest tests    1133 passing
Nextest retries  0
Cargo.lock       D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB
```

OpenCode LSP has already been tried honestly and found ineffective for this workspace. M01C2 must not spend time retrying it. Use exact Clippy diagnostics and `rg` where reference confirmation is useful.

## Problem

`event_queue.rs` contains a test named `queue_cannot_be_moved_across_threads`. The test defines an `assert_send<T: Send>()` helper but deliberately does not call it. Its comments claim `ResultEventQueue` is `!Send`, although the type contains no non-`Send` marker and J10 established serial execution through coordinator structure rather than through a type-level `!Send` guarantee.

The unused nested helper is expected to produce a warning. The test therefore both creates warning debt and documents a guarantee that is not actually enforced.

## Required repair

1. Capture the accepted Rust 1.97.1 Clippy baseline before editing.
2. Confirm every warning whose primary span is `src/event_queue.rs`.
3. Replace the misleading `queue_cannot_be_moved_across_threads` test with a truthful compile-time assertion that `ResultEventQueue: Send` under the current representation.
4. Name and document the replacement so it is explicit that:
   - `Send` only means the value may be moved between threads;
   - Tethers still performs result-event evaluation serially;
   - seriality is enforced by the coordinator and queue-drain design, not by a fake `!Send` claim.
5. Keep the number of event-queue tests unchanged.
6. Remove every warning whose primary span is `src/event_queue.rs` if the captured warning is caused by this false test.
7. Do not alter production queue fields, methods, visibility, ordering, or ownership behaviour.

## Forbidden repair shapes

- Do not add `PhantomData`, `Rc`, `Cell`, raw pointers, or another marker merely to force `!Send`.
- Do not add `#[allow(...)]`, `#[expect(...)]`, underscore renaming, dummy use, `black_box`, unreachable code, sleeps, retries, or source-text theatre.
- Do not add threads, async code, channels, locks, parallel workers, or concurrency infrastructure.
- Do not change `ResultAnchor`, the coordinator, admission logic, causal depth, replay, dispatch, or follow-up evaluation behaviour.
- Do not change dependencies, Cargo.lock, tool configuration, deny policy, Nextest policy, or Just recipes.

## Tool use

### Clippy

Clippy is the warning authority. Capture machine-readable output before and after and compare warnings by lint code, file, and primary span.

### Nextest

Use Nextest only for the focused edit loop. List and run the `event_queue` tests with the committed root configuration and zero retries.

### Cargo

Ordinary Cargo through `just verify` remains final authority. The full Cargo total must remain 926 unless an unavoidable harness accounting difference is explained and no test disappears.

### Dependency tools

Cargo-deny and cargo-machete are not required in this task because Cargo.toml, Cargo.lock, deny policy, and all dependency metadata are forbidden from changing. Confirm those paths are absent from the diff instead of running irrelevant dependency scans.

### LSP

Do not retry OpenCode LSP. M01C1 already proved it exposes operations but returns no useful Rust workspace results under the frozen configuration.

## Permitted files

Only:

- `tethers-0.1/host-rust/src/event_queue.rs`;
- `docs/CURRENT_CLINE_TASK.md` for state and checkpoint;
- `docs/worker-notes/2026-08-04-m01c2-event-queue-contract-warning.md`.

## Behavioural invariants

- FIFO remains `push_back` / `pop_front`.
- The queue remains process-local and in-memory.
- One event is processed at a time by the coordinator.
- No recursion, retry, thread, async runtime, parallel worker, persistence, or scheduling policy is added.
- Result Anchor identity, causation, correlation, generation, admission, and serialization are unchanged.
- No public CLI, language, Plug, Trail, capability, or protocol behaviour changes.

## Warning accounting

Record before and after:

- total emitted warning messages;
- warnings grouped by lint code;
- warnings whose primary span is `src/event_queue.rs`;
- warnings outside the target file.

Acceptance requires:

- zero warnings whose primary span is `src/event_queue.rs`;
- no new warning outside the target;
- no suppression attribute;
- total warnings lower than the baseline if the expected target warning is emitted.

Because Cargo may compile the same source for multiple targets, distinguish unique source warnings from repeated emitted messages.

## Verification floor

Run only the evidence-bearing checks:

1. task packet checker;
2. Rust agent tool checker;
3. Rustfmt;
4. focused Nextest event-queue tests with zero retries;
5. Clippy before and after;
6. `just verify` once as the full Cargo authority;
7. Cargo.lock hash;
8. diff and status checks.

Do not run full Nextest, cargo-deny, cargo-machete, or `just verify-agent` for this test-only task unless an unexpected relevant file changes. If that occurs, stop rather than broadening scope.

## Completion evidence

The worker note must record:

- exact baseline warning;
- why the old test was false or non-proving;
- the replacement test and compile-time assertion;
- focused Nextest command and result;
- final Cargo total;
- before/after warning table;
- unchanged Cargo.lock hash;
- exact changed files;
- confirmation that no runtime source or dependency file changed;
- smallest next action.
