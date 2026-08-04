# Current Implementation Task

Control contract: `1`
Task: `M01C3 - Event-queue dead API cleanup`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Green`
Route: `OpenCode using HY3 for a narrow Rust internal-API cleanup; Lucy performs independent review`
Base branch: `main`
Base commit: `170063ea24b3ba4ba5529749ae6fc615e7c58de6`
Implementation branch: `opencode/m01c3-event-queue-dead-api`
Worker note: `docs/worker-notes/2026-08-04-m01c3-event-queue-dead-api-cleanup.md`
Implementation blueprint: `docs/architecture/M01C3_EVENT_QUEUE_DEAD_API_CLEANUP.md`
Rust toolchain: exact `1.97.1`; plain Cargo; `--locked` mandatory
Agent tools: Clippy JSON and cargo-nextest 0.9.140; do not retry ineffective OpenCode LSP
OCaml switch path: `N/A`
Implementation checkpoint: `a145714f47ee04e729e6dfbb2419521aa95e7bbb`

## Objective

Remove the remaining `dead_code` warning in `tethers-0.1/host-rust/src/event_queue.rs` by deleting the unused `ResultEventQueue::is_empty` and `ResultEventQueue::len` methods and rewriting their test-only assertions through `pop_front`.

Read `docs/architecture/M01C3_EVENT_QUEUE_DEAD_API_CLEANUP.md` completely before editing. It is authoritative.

## Relevant background and existing behaviour

M01C2 is accepted on `main` at:

`21671b06365f28923d7375005d9b14d9559b71a4`

Accepted baseline:

```text
Rust             1.97.1
Cargo tests      926 passing
Nextest tests    1133 passing
Nextest retries  0
Clippy emitted   119 warnings
Cargo.lock       D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB
```

M01C2 established that `ResultEventQueue` is truthfully `Send` under its current representation and that serial processing is enforced by coordinator structure, not a fake type marker.

The remaining warning covers two methods that production does not call:

- `ResultEventQueue::is_empty`;
- `ResultEventQueue::len`.

Production creates, enqueues, and drains through `new`, `enqueue`, and `pop_front`. The two warned methods are test conveniences only.

## Required behaviour

1. Keep FIFO ordering exactly unchanged.
2. Keep coordinator-driven one-at-a-time draining exactly unchanged.
3. Delete the two dead methods rather than manufacturing production callers or hiding them behind test configuration.
4. Rewrite affected queue tests through `pop_front().is_none()` only when the queue is expected to be exhausted.
5. Keep exactly nine event-queue tests.
6. Preserve the compile-time `ResultEventQueue: Send` assertion added by M01C2.
7. Finish with zero warnings whose primary span is `src/event_queue.rs`.

## Relevant components

- `tethers-0.1/host-rust/src/event_queue.rs` — queue type, dead methods, and colocated tests.
- `tethers-0.1/host-rust/src/application.rs` — production coordinator drain using `while let Some(anchor) = queue.pop_front()`; inspection only.
- `.config/nextest.toml` — accepted zero-retry focused test configuration; inspection only.
- `justfile` — final Cargo verification route; inspection only.

## Frozen decisions and invariants

- `ResultEventQueue` continues to wrap private `VecDeque<ResultAnchor>` storage.
- `enqueue` remains `push_back`; `pop_front` remains `pop_front`.
- The queue remains process-local and in-memory.
- No recursion, retry, worker thread, async runtime, channel, lock, scheduling policy, or persistence is added.
- Result Anchor identity, correlation, causation, generation, admission, replay, dispatch, and serialization remain unchanged.
- No CLI, language, Plug, Trail, capability, protocol, dependency, lockfile, or tool-policy change is permitted.

## Startup procedure

1. Require a clean worktree.
2. Fetch `origin`.
3. Verify M01C2 is an ancestor of `origin/main`:

   ```powershell
   git merge-base --is-ancestor 21671b06365f28923d7375005d9b14d9559b71a4 origin/main
   ```

4. Read the packet and blueprint directly from current `origin/main`.
5. Confirm the implementation branch does not already exist locally or remotely.
6. Create `opencode/m01c3-event-queue-dead-api` from current `origin/main`.
7. Update this packet's Base commit to the exact branch base before the implementation commit. Record the same base in the worker note.
8. Read completely before editing:
   - `AGENTS.md`;
   - this packet;
   - the M01C3 blueprint;
   - `tethers-0.1/host-rust/src/event_queue.rs`;
   - the production drain in `tethers-0.1/host-rust/src/application.rs`;
   - the M01C2 worker note;
   - `.config/nextest.toml`;
   - `justfile`.
9. Record the Cargo.lock SHA-256.
10. Run the task-packet checker.

## Reference proof

Do not retry OpenCode LSP.

Run one bounded exact text-reference pass before editing. Confirm that `is_empty` and `len` have no non-test `ResultEventQueue` callers in `tethers-0.1/host-rust/src`. Distinguish unrelated collection methods from the queue methods.

Useful searches include:

```powershell
rg -n "ResultEventQueue|queue\.is_empty\(\)|queue\.len\(\)" tethers-0.1/host-rust/src
```

Record the exact result in the worker note. Do not broaden into repository archaeology.

## Baseline warning capture

Before editing, run ordinary Clippy once and capture machine-readable output once:

```powershell
cargo clippy --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked

cargo clippy --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked --message-format=json 1> $env:TEMP\m01c3-clippy-before.jsonl 2> $env:TEMP\m01c3-clippy-before.stderr.txt
```

Actual diagnostics are authoritative. Expected from M01C2:

- 119 emitted warning messages in total;
- one unique source warning in `src/event_queue.rs` covering unused `is_empty` and `len` methods.

Record emitted totals, unique source warnings, lint codes, targets, and every warning outside the target for comparison.

## Required implementation

Change only `tethers-0.1/host-rust/src/event_queue.rs` production/test content as follows:

1. Delete the `is_empty` method and its documentation comment.
2. Delete the `len` method and its documentation comment.
3. In existing tests, replace assertions that use those methods with assertions through the real queue operation:
   - an untouched new queue must return `None` from `pop_front`;
   - after expected items are drained, one additional `pop_front` must return `None`.
4. Preserve every existing ordering assertion.
5. Preserve the existing nine test functions, permitting only names or assertion bodies needed for this cleanup.
6. Do not expose or inspect the private `pending` field.

## Permitted files

Only:

- `tethers-0.1/host-rust/src/event_queue.rs`;
- `tethers-0.1/host-rust/src/application.rs` — test-module-only `#[cfg(test)]` callers of `is_empty` / `len` (lines 7150, 7163, 7236, 7407, 7448, 7457, 7605, 7834, 7963, 8025, 8160, 8485), rewritten to `pop_front().is_none()` / `pop_front()` assertions; no production code change;
- `docs/CURRENT_CLINE_TASK.md` for base, state, and checkpoint;
- `docs/worker-notes/2026-08-04-m01c3-event-queue-dead-api-cleanup.md`.

Stop before changing any other path or any production (non-test) code in `application.rs`.

## Forbidden changes

- No `#[allow(...)]`, `#[expect(...)]`, underscore concealment, dummy call, `black_box`, unreachable use, or source-text test.
- No `#[cfg(test)]` versions of the deleted methods.
- No coordinator change merely to call the dead methods.
- No queue field, storage type, visibility, `enqueue`, `pop_front`, or `Default` change.
- No dependency, Cargo.lock, policy, configuration, tool, OCaml, protocol, CLI, replay, admission, dispatch, concurrency, or scheduling change.

## Stop conditions

Stop only if:

- either warned method has a genuine non-test source caller;
- the baseline warning is already absent;
- the repair requires an out-of-scope behavioural change;
- an evidence-bearing verification reveals a real defect that cannot be corrected within the three permitted files.

Do not stop for ineffective LSP, warning duplication across targets, line-number drift, or intentionally skipped dependency scans.

## Expected pre-existing changes

None.

## Edit recovery

If an exact replacement fails:

1. reread the current file;
2. make a smaller patch against current content;
3. do not repeat the same failed edit;
4. after two materially different failed edits, use a precise local rewrite of the affected block rather than abandoning the task.

## Focused feedback loop

After the coherent edit:

1. run Rustfmt;
2. run the focused event-queue Nextest filter once;
3. run Clippy once and inspect target warnings;
4. correct any in-scope defect on the same branch.

Expected focused result: 9 passed, 0 failed, 0 retries.

## Final warning accounting

Capture final Clippy JSON once. Require:

- zero warnings whose primary span is `src/event_queue.rs`;
- total emitted warnings lower than before;
- no new or changed warning outside the target;
- no suppression attribute or fake production use.

Report both emitted-message totals and unique source-warning totals.

## Required verification

Run only:

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1

cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check

cargo nextest run --config-file .config/nextest.toml --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked -E 'test(event_queue::)'

cargo clippy --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked

just verify

Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256
git diff --check
git status --short
```

Do not run full Nextest, cargo-deny, cargo-machete, `just verify-agent`, OCaml tests, or unrelated scripts. Confirm dependency and tool-policy paths are absent from the diff instead.

Expected floors:

- focused Nextest: 9 passed, 0 failed, 0 retries;
- full Cargo through `just verify`: at least 926 passed, 0 failed;
- no event-queue test disappears;
- Cargo.lock hash unchanged.

## Acceptance criteria

1. Exact reference proof confirms both deleted methods lacked non-test callers.
2. `is_empty` and `len` are removed, not hidden or artificially used.
3. Tests assert empty/exhausted state through `pop_front` without inspecting storage.
4. Nine event-queue tests pass with zero retries.
5. Full Cargo passes with zero failures and no missing test.
6. `event_queue.rs` emits zero warning.
7. Total warnings decrease and no outside warning changes.
8. Cargo.lock is unchanged.
9. No runtime queue, coordinator, dependency, protocol, CLI, concurrency, admission, replay, or dispatch behaviour changes.

## Completion contract

After every acceptance condition passes:

1. Create the worker note with these control-v1 sections:
   - Requested outcome
   - Changes made
   - Decisions and assumptions
   - Evidence
   - Discoveries
   - Remaining risks
   - Smallest next action
   - References
2. Record the exact implementation checkpoint after the source/test commit.
3. Set this packet to `COMPLETE` and task colour `Green`.
4. Commit documentation normally and push the implementation branch normally.
5. Return only:
   - outcome;
   - branch and remote tip;
   - implementation checkpoint;
   - exact changed files;
   - reference proof;
   - warning before/after table;
   - focused Nextest result;
   - final Cargo result;
   - Cargo.lock hash;
   - confirmation that forbidden scans were intentionally skipped.
