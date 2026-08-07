# Worker Note

Task: `F2 - Operational correctness defects`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `f295daa288f4d3dc48181888d6655df798675033`

Implementation checkpoint: `130a760c3c8c7e8ad3599f5b2f333ea02b91e119`

## Checkpoints

| Checkpoint | SHA | Purpose |
|---|---|---|
| F2 packet | `f5245f3` | Task definition and documentation |
| F2a regression | `2b7e7f7` | Failing live-stderr regression test |
| F2a repair | `c8cc0fe` | Stderr capture fix + 6 sub-tests |
| F2b repair r1 | `54216d7` | WaitForSingleObject approach (superseded) |
| F2 corrections | `0b02c43` | Object-identity test, exact-bound test, cleanup result |
| F2 correction r2 | `130a760` | Truthful wait, reaped logic, cleanup assertions, 5 helper tests |

## Requested outcome

Repaired two F1-confirmed operational defects: truthful live stderr capture (F2a) and nondeterministic M3 handle allow-list test (F2b). Added structured cleanup accounting, truthful wait classification, and deterministic unit tests for Windows wait-result interpretation. All public contracts preserved.

## Changes made

### F2a: Live stderr and child cleanup (`child_process.rs`)

**Defect:** Stderr reader thread (lines 410-431) accumulated bytes in local `buf` and only copied to `Arc<Mutex<Vec<u8>>>` after loop exit. `stderr_tail()` returned empty while child alive.

**Repair:** `guard.clone_from(&buf)` after each successful `reader.read()` under the Mutex lock, providing incremental visibility. Raw bytes stored internally; lossy UTF-8 only at `stderr_tail()`.

**Six sub-tests:**
1. `f2a_regression_live_stderr_not_visible_before_exit` — confirms failure on accepted main
2. `f2a_live_stderr_visible_before_exit` — marker visible while child alive
3. `f2a_bounded_stderr_tail` — 100-byte limit, known byte pattern, exact-equality assertion
4. `f2a_timeout_remains_timeout_with_stderr_available` — timeout correctly classified
5. `f2a_exit_distinguishable_from_timeout_and_disconnect` — `ProcessExited(7)` matches actual exit code
6. `f2a_windows_cleanup_reaps_child_and_joins_threads` — inspects `ChildCleanup` struct

### F2b: M3 handle allow-list (3 files)

**Root cause (r2):** Handle-value aliasing (Option B). `GetHandleInformation` validates handle existence, not object identity. Windows handle reuse causes ~35% false-positive rate.

**Fix (r2 — object-identity):**
1. Parent creates event in **unsignalled** state (`bInitialState = 0`)
2. Child attempts `SetEvent(raw)` on the raw handle value
3. Conformance runs normally (no error from the handle check)
4. Parent calls `WaitForSingleObject(canary, 0)` — must be `WAIT_TIMEOUT`
5. This proves the child could not signal the parent's event

If `SetEvent` succeeds on a colliding handle of a different type (file, key), the parent's event remains unsignalled — no false failure. The parent-side check is authoritative.

**Conformance change:** The handle-allow-list check in `conformance.rs:484-501` no longer returns an error. It records the fact but does not fail conformance.

### Cleanup accounting (`child_process.rs`)

**New struct:** `ChildCleanup` with fields: `graceful_exited`, `job_terminated`, `child_killed`, `child_waited`, `reaped`, `stdout_thread_joined`, `stderr_thread_joined`. Each step records its actual outcome.

**API change:** `shutdown(self) -> ChildCleanup` returns truthful accounting. `shutdown_inner(&mut self) -> ChildCleanup` builds the result. Drop calls `shutdown_inner()` and discards the result (best-effort).

**Production callers** compile unchanged (Rust silently discards non-`()` return types).

## Decisions and assumptions

- `max_line_bytes` remains dead code → F8.
- No new dependencies, no widened visibility, no public contract changes.
- `clone_from` per-read bounded to `stderr_tail_bytes` (64 KiB default) — negligible overhead.
- F2b: `SetEvent` on a non-event or wrong-type handle returns 0 (failure) — no side effect.
- `just verify` and `just verify-agent` both run the full test suite; `verify-agent` additionally runs `cargo-deny`, `cargo-machete`, and `cargo-nextest`. All passed independently.

## Evidence

### Focused tests

```
cargo test --locked f2a_ -- --nocapture        → 11/11 PASS (6 stderr/cleanup + 5 interpret_process_wait)
cargo test --locked m3_windows_handle_allow_list_excludes_unrelated_inheritable_handle -- --exact → PASS
```

### R2 corrections (`130a760`)

| Correction | What changed |
|---|---|
| Truthful wait | `ManagedChild::wait()` inspects `WaitForSingleObject` return (`WAIT_OBJECT_0`) and rejects `STILL_ACTIVE` exit code |
| Helper + tests | `interpret_process_wait(wait_result, exit_code)` factored for deterministic unit test coverage of all four failure modes |
| Reaped logic | `self.reaped = cleanup.child_waited` — kill is not reaping |
| Cleanup test | New assertions: `job_terminated`, `child_waited`, `reaped`, `stdout_thread_joined`, `stderr_thread_joined`, process no longer exists |

### Final verification matrix

| # | Command | Result |
|---|---|---|
| 1 | `git fetch origin --prune` → `rev-parse origin/main` | PASS (`f295daa...`) |
| 2 | `git rev-parse HEAD` | PASS (`130a760...`) |
| 3 | `git status --short --branch` | PASS (clean) |
| 4 | `cargo fmt --all -- --check` | PASS |
| 5 | `cargo check --all-targets --all-features --locked` | PASS |
| 6 | `cargo test --all-targets --all-features --locked` | PASS (1515 tests) |
| 7 | `cargo clippy --all-targets --all-features --locked -- -W clippy::all` | PASS (pre-existing only) |
| 8 | `just verify` | PASS |
| 9 | `just verify-agent` | PASS (1515 tests, cargo-deny, cargo-nextest) |
| 10 | `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` | PASS |
| 11 | `git diff --exit-code origin/main...HEAD -- docs/foundation-pass/fixtures` | PASS (byte-identical) |
| 12 | `git diff --check origin/main...HEAD` | PASS (clean) |
| 13 | `git diff --name-only origin/main...HEAD` | PASS (7 files) |
| 14 | `git status --short --branch` | PASS (clean) |

## Branch diff

7 files changed:
- `docs/CURRENT_CLINE_TASK.md`, `docs/CURRENT_GOAL.md`, `docs/worker-notes/2026-08-07-f2-operational-correctness.md`
- `tethers-0.1/host-rust/src/child_process.rs` — stderr repair, ChildCleanup, 6 tests
- `tethers-0.1/host-rust/src/bin/m3_fixture_provider.rs` — SetEvent approach
- `tethers-0.1/host-rust/src/conformance.rs` — relaxed handle check
- `tethers-0.1/host-rust/tests/m3_lifecycle.rs` — parent-side WaitForSingleObject

Zero fixture changes. Zero public contract changes. No new dependencies.

## Discoveries

1. M3 handle test: `GetHandleInformation` checks handle existence, not object identity. `WaitForSingleObject` narrows but still accepts any waitable object. Only parent-side `WaitForSingleObject` on the actual event proves exclusion.
2. Rust silently discards non-`()` return values — `shutdown()` return type change did not require caller updates.
3. `WAIT_TIMEOUT` and `WAIT_OBJECT_0` not exported by windows-sys 0.61 — defined as local constants.

## Remaining risks

- `max_line_bytes` dead code → F8.
- M3 test passes 20/20 (deterministic) — cold-from-boot not tested.
- `stdout_thread` and `stderr_thread` field inspection relies on `#[cfg(test)]` private access.

## Smallest next action

Push corrected branch `foundation/f2-operational-correctness`. Await Lucy's independent review. Do not merge or begin F3.

## References

- Base: `f295daa288f4d3dc48181888d6655df798675033` (`origin/main`)
- F1 worker note: `docs/worker-notes/2026-08-06-f1-baseline.md`
- F1 debt ledger: `docs/foundation-pass/DEBT_LEDGER.md`
- Task packet: `docs/CURRENT_CLINE_TASK.md`
