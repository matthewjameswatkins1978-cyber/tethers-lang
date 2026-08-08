# Worker Note — F1-R1 Missing Performance Baseline Reconciliation

Task: `F1-R1 — Missing Performance Baseline Reconciliation`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `ea7426dbeb1934cf336673d03ae2abf76146ea7d`
Implementation checkpoint: `5cc9f3b6a1d303be369cd93715f7a425c2c5bb73`

## Requested outcome

Measure cold/warm timings at the historical pre-Foundation baseline (`24428139`) and current F5 tip (`ea7426d`) for cargo check, cargo test, cargo clippy, just verify, and just verify-agent. Analyse P1 (application.rs compile-time) and P2 (result_large_err) hypotheses. Decide F6 authorisation. No production changes.

## Changes made

### Permanent (1)
- `docs/foundation-pass/PERFORMANCE_BASELINE_R1.md` — complete performance evidence document with raw timings, P1/P2 analysis, F6 authorisation table

### Closeout (2)
- `docs/CURRENT_CLINE_TASK.md` — updated to reflect F1-R1 task packet
- `docs/worker-notes/2026-08-08-f1-r1-performance-baseline.md` — this note

### Temporary (removed)
- Detached worktree at historical SHA `24428139807cac0adeb0b62264547e61ca809d16`
- Detached worktree at current F5 SHA `ea7426dbeb1934cf336673d03ae2abf76146ea7d`

## Decisions and assumptions

- All cold measurements used fresh detached worktrees with zero target artefacts. `cargo fetch --locked` ran first to avoid network timing contamination.
- Single cold run plus 3 warm runs per command group. This balances measurement cost against noise detection.
- `cargo test --all-targets --all-features --locked` was used for the measurement baseline. The `--all-features` flag surfaces 6 pre-existing test failures at both SHAs (installation_recovery_destination_tests, engine_stdio tests); these are pre-Foundation, not F5-introduced.
- `just verify` at F5 fails on `cargo fmt --check` before reaching tests due to the known pre-existing `replay_windows.rs:3277` formatting discrepancy. This is documented as a known baseline condition.
- Per-file compile-time attribution is not possible without unstable compiler features (`cargo -Z timings`), which would require a dependency change. The task forbids such changes.
- Cargo global cache was already populated; `cargo fetch` confirmed no additional downloads were needed.
- Machine was on AC power with no obvious heavy competing workload.

## Evidence

### Timing summary (cold / median warm, in milliseconds)

| Command | Historical | F5 |
|---------|-----------|-----|
| cargo check | 21,356 / 259 | 19,083 / 255 |
| cargo test | 51,554 / 13,050 | 51,216 / 14,041 |
| cargo clippy | 21,215 / 377 | 22,485 / 381 |
| just verify | 62,140 / 16,718 | 4,552 / 3,242 |
| just verify-agent | 59,832 / 16,855 | 3,203 / 3,367 |

All commands have one cold run and three warm runs. F5 wrapper commands short-circuit on `cargo fmt --check` (pre-existing `replay_windows.rs:3277` discrepancy) before reaching the test stage.

### P1 (application.rs compile-time)

- application.rs: 8,260 (hist) vs 8,264 (F5) lines — effectively unchanged
- Cold check: 21.4s (hist) / 19.1s (F5) — ~2.3s lower in this observation; no cause is attributed
- Warm check: essentially identical (~255ms)
- **Classification: UNATTRIBUTED COST — NOT AUTHORISED FOR F6**

### P2 (result_large_err)

- `run_command::RunResult` and `check_command::CheckResult` trigger Clippy `result_large_err` at both SHAs
- `run_command.rs` exists at both SHAs (1,050 lines)
- These are CLI command entry paths (once per invocation), not engine hot loop
- **Classification: UNMEASURED HYPOTHESIS — NOT AUTHORISED FOR F6**

### F6 authorisation

| Candidate | F6 authorised? |
|-----------|----------------|
| P1 — application.rs | **NO** |
| P2 — result_large_err | **NO** |

## Discoveries

- The Foundation Pass did not materially change the cold test timing (~51s at both SHAs).
- F5 cold cargo check was ~2.3s lower. These measurements do not attribute this difference to any specific change, warning count, file, or compiler behaviour.
- The F5 `just` wrapper commands short-circuit on `cargo fmt --check` before reaching the test stage.
- Neither F1 hypothesis has measured evidence supporting F6 optimisation.

## Remaining risks

None. This is an evidence-only task. The measurement method is reproducible but not laboratory-grade; variance is within OS scheduling noise.

## Smallest next action

Lucy reviews F1-R1 evidence. F6 requires measured cost to be authorised for production optimisation; currently none exists.

## References

- Historical baseline SHA: `24428139807cac0adeb0b62264547e61ca809d16`
- Current F5 SHA: `ea7426dbeb1934cf336673d03ae2abf76146ea7d`
- F5 accepted branch: `foundation/f5-ocaml-boundaries`
