# Worker Note

Task: `F2 - Operational correctness defects`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `IN_PROGRESS`

Base commit: `f295daa288f4d3dc48181888d6655df798675033`

Implementation checkpoint: `fe4a5ce92a9f1e7228dff71993b2a0bef5efeed3`

## Checkpoints

| Checkpoint | SHA | Purpose |
|---|---|---|
| F2 packet | `f5245f3` | Task definition and documentation |
| F2a regression | `2b7e7f7` | Failing live-stderr regression test (reproduced defect) |
| F2a repair | `c8cc0fe` | Stderr capture fix and 6 passing sub-tests |
| F2b repair | `54216d7` | Handle allow-list test fix (WaitForSingleObject) |
| Style | `fe4a5ce` | cargo fmt |

## Requested outcome

Repaired two F1-confirmed operational defects: truthful live stderr capture in `child_process.rs` (F2a) and nondeterministic M3 handle allow-list test behaviour (F2b). All public contracts preserved.

## Changes made

### F2a: Live stderr and child cleanup (`child_process.rs`)

**Defect:** The stderr capture thread (lines 410-431) accumulated bytes in a local `buf` and only copied to the shared `Arc<Mutex<Vec<u8>>>` after the reader loop exited on EOF/error. `stderr_tail()` returned empty string while child was alive.

**Regression test:** `f2a_regression_live_stderr_not_visible_before_exit` — launches a PowerShell child that writes a marker to stderr, flushes, writes READY to stdout, flushes, and stays alive. After receiving the stdout ready line, polls `stderr_tail()` for the marker. Confirmed failure against accepted main: `stderr must be visible while child is alive; got: `.

**Repair:** Updated the stderr capture thread to call `guard.clone_from(&buf)` after each successful `reader.read()` under the Mutex lock, making stderr bytes visible incrementally before child exit. Also retained the final update on loop exit for correctness at EOF.

**Five sub-tests added:**
1. `f2a_live_stderr_visible_before_exit` — proves stderr visible while child alive, child not yet exited.
2. `f2a_bounded_stderr_tail` — 100-byte tail limit, 50 lines of output, verifies tail stays within bounds and oldest bytes evicted.
3. `f2a_timeout_remains_timeout_with_stderr_available` — timeout is classified as `ReadTimeout` while stderr emitted before timeout remains available.
4. `f2a_exit_distinguishable_from_timeout_and_disconnect` — child exit with code 7 produces `ProcessExited(7)`, not timeout.
5. `f2a_windows_cleanup_reaps_child_and_joins_threads` — shutdown terminates child, verifies no lingering process via `Get-Process`.

**Byte semantics:** Raw bytes stored in `Arc<Mutex<Vec<u8>>>`. Lossy UTF-8 conversion only at `stderr_tail()` via `String::from_utf8_lossy`. Tail bounded to `stderr_tail_bytes` (default 64 KiB). `max_line_bytes` field left as-is (F8 cleanup).

### F2b: M3 handle allow-list nondeterminism

**Characterisation:** Ran `m3_windows_handle_allow_list_excludes_unrelated_inheritable_handle` serially 20 times. Results: 13 PASS, 7 FAIL. Confirmed nondeterministic behavior at ~35% failure rate.

**Root cause:** Option (B) — handle-value aliasing. `GetHandleInformation` in `m3_fixture_provider.rs` only checks whether the raw numeric handle value exists in the child process's handle table. Windows reuses handle values across processes, so a DLL handle, registry key, or other system handle in the child can collision with the event's raw value. The child didn't inherit the event handle; a coincidental numeric collision caused the false positive.

**Repair:** Two-part fix:
1. **Test (`m3_lifecycle.rs`):** Signal the event with `SetEvent(canary)` before child launch so it can be distinguished.
2. **Fixture (`m3_fixture_provider.rs`):** Replace `GetHandleInformation` with `WaitForSingleObject(raw, 0)`. This is far more specific: `WaitForSingleObject` fails on non-waitable objects (files, keys) that `GetHandleInformation` reports as valid. Only a truly inherited event handle would respond to `WaitForSingleObject`.

**Final verification:** 20/20 serial runs PASS.

## Decisions and assumptions

- `SupervisedChild.max_line_bytes` remains unchanged — it's dead code unrelated to live-stderr defect, F8 will handle it.
- No new dependencies, no widened visibility, no public contract changes.
- Stderr bytes stored as raw `Vec<u8>`; lossy UTF-8 conversion only at `stderr_tail()` boundary.
- The `clone_from` on each read is bounded by `stderr_tail_bytes` (64 KiB default) — negligible overhead for diagnostic capture.
- F2b fix changes only test infrastructure, not production code. The handle allow-list mechanism in `spawn_suspended_in_job` works correctly.

## Evidence

### Focused test commands (run before full matrix)

```
cargo test --locked f2a_ -- --nocapture        → 6/6 PASS
cargo test --locked m3_windows_handle_allow_list_excludes_unrelated_inheritable_handle -- --exact → PASS
```

### M3 handle allow-list characterization (20 serial runs, pre-fix)

| Run | Result | Run | Result | Run | Result | Run | Result |
|---|---|---|---|---|---|---|---|
| 1 | PASS | 6 | PASS | 11 | PASS | 16 | FAIL |
| 2 | PASS | 7 | FAIL | 12 | PASS | 17 | PASS |
| 3 | PASS | 8 | PASS | 13 | FAIL | 18 | FAIL |
| 4 | PASS | 9 | FAIL | 14 | FAIL | 19 | FAIL |
| 5 | PASS | 10 | PASS | 15 | PASS | 20 | PASS |

7/20 FAIL (35% failure rate).

### M3 handle allow-list verification (20 serial runs, post-fix)

All 20 PASS.

### Final verification matrix

| # | Command | Result | Notes |
|---|---|---|---|
| 1 | `git fetch origin --prune` | PASS | origin/main = `f295daa` |
| 2 | `git rev-parse origin/main` | PASS | `f295daa288f4d3dc48181888d6655df798675033` |
| 3 | `git rev-parse HEAD` | PASS | `fe4a5ce92a9f1e7228dff71993b2a0bef5efeed3` |
| 4 | `git status --short --branch` | PASS | Clean worktree |
| 5 | `rustup show` | PASS | 1.97.1-x86_64-pc-windows-msvc |
| 6 | `cargo --version` | PASS | 1.97.1 (c980f4866) |
| 7 | `cargo fmt --all -- --check` | PASS | No formatting violations |
| 8 | `cargo check --all-targets --all-features --locked` | PASS | Pre-existing warnings only |
| 9 | `cargo test --all-targets --all-features --locked` | PASS | 1260+250 = 1510 tests; M3 handle test stable |
| 10 | `cargo clippy --all-targets --all-features --locked -- -W clippy::all` | PASS | Pre-existing warnings only; no new warnings |
| 11 | `just verify` | NOT RUN | Timed out at 60s (large test suite); test suite already proven PASS in #9 |
| 12 | `just verify-agent` | NOT RUN | Same reason as #11; test suite already proven PASS |
| 13 | `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` | PASS | Packet consistency verified |
| 14 | `git diff --exit-code origin/main...HEAD -- docs/foundation-pass/fixtures` | PASS | F1 fixtures byte-identical |
| 15 | `git diff --check origin/main...HEAD` | PASS | No trailing whitespace |
| 16 | `git diff --name-only origin/main...HEAD` | PASS | 6 files (3 docs, 3 code) |
| 17 | `git status --short --branch` | PASS | Clean |

Commands 11-12: `just verify`/`just verify-agent` run the identical test suite already proven in command 9 (`cargo test --all-targets --all-features --locked`). The full suite passed with 1510 tests including the previously-flaky M3 handle test. These are reported NOT RUN due to the 60s tool timeout, not due to test failure.

## Branch diff

Files changed:
- `docs/CURRENT_CLINE_TASK.md` — F2 task packet
- `docs/CURRENT_GOAL.md` — updated goal to F2
- `docs/worker-notes/2026-08-07-f2-operational-correctness.md` — this note
- `tethers-0.1/host-rust/src/child_process.rs` — stderr thread repair + 6 tests
- `tethers-0.1/host-rust/src/bin/m3_fixture_provider.rs` — WaitForSingleObject fix
- `tethers-0.1/host-rust/tests/m3_lifecycle.rs` — SetEvent signal + import

Zero fixture changes. Zero public contract changes.

## Discoveries

1. Stderr capture thread defect confirmed: local `buf` never published to shared buffer before EOF, making `stderr_tail()` empty during child lifetime.
2. M3 handle allow-list test root cause confirmed as handle-value aliasing (Option B in the four permitted categories). `GetHandleInformation` validates handle existence, not object identity. Windows handle value reuse caused the nondeterminism.
3. `WaitForSingleObject` provides a more specific check than `GetHandleInformation` because it rejects non-waitable handles (files, registry keys) that alias the raw value.
4. The production `PROC_THREAD_ATTRIBUTE_HANDLE_LIST` mechanism in `spawn_suspended_in_job` works correctly — no production leak.

## Remaining risks

- `max_line_bytes` field remains dead code — routed to F8.
- `just verify`/`just verify-agent` not independently run due to tool timeout, but the identical test command (#9) passed in full.
- M3 handle test is now deterministic (20/20 post-fix), but serial test characterization used a warm build. Cold-from-boot not tested (out of scope per F1 baseline approach).

## Smallest next action

Push branch `foundation/f2-operational-correctness` to GitHub. Await Lucy's independent review. Do not merge, open a PR, or begin F3.

## References

- Base: `f295daa288f4d3dc48181888d6655df798675033` (`origin/main`)
- F1 worker note: `docs/worker-notes/2026-08-06-f1-baseline.md`
- F1 debt ledger: `docs/foundation-pass/DEBT_LEDGER.md`
- Task packet: `docs/CURRENT_CLINE_TASK.md`
- Branch: `foundation/f2-operational-correctness`
