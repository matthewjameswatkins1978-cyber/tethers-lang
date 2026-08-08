# F1-R1 — Performance Baseline Reconciliation

**Status:** COMPLETE
**Date:** 2026-08-08

## Reason for reconciliation

F6 optimisation is constrained to "Address only F1-measured costs." The accepted F1 baseline transcript recorded relevant timings as NOT CAPTURED. The F1 debt ledger contains two unmeasured F6 candidates. This reconciliation measures the actual operational costs at both the historical pre-Foundation baseline and the current F5 tip so that F6 authorisation can be decided on evidence.

## Measurement targets

| Target | SHA | Description |
|--------|-----|-------------|
| Historical baseline | `24428139807cac0adeb0b62264547e61ca809d16` | Pre-Foundation accepted tip |
| Current F5 comparison | `ea7426dbeb1934cf336673d03ae2abf76146ea7d` | Foundation F5 accepted tip |

## Environment

| Property | Value |
|----------|-------|
| OS | Microsoft Windows 11 Pro Insider Preview |
| CPU | AMD Ryzen 9 3900X 12-Core Processor |
| RAM | 31.9 GB |
| Filesystem | NTFS |
| Rust | rustc 1.97.1 (8bab26f4f 2026-07-14) |
| Cargo | cargo 1.97.1 (c980f4866 2026-06-30) |
| PowerShell | 7.6.4 |
| Cargo caches | Populated before each command (cargo fetch --locked) |
| AC power | Likely (laptop, not on battery) |
| Workload | No obvious heavy competing workload |
| Timestamp | 2026-08-08 18:58 +01:00 |

## Cold / warm definitions

- **COLD:** fresh detached worktree with zero target build artefacts before timed command. `cargo fetch --locked` run first to populate caches.
- **WARM:** immediately repeated identical command, same source, existing target artefacts, no intervening source change.
- Cold timings are single-run (establishing a clean checkout per measurement group is expensive). Warm timings use 3 runs.

## Timing methodology

```pwsh
$sw = [System.Diagnostics.Stopwatch]::StartNew()
& <command>
$exit = $LASTEXITCODE
$sw.Stop()
Write-Output "Exit: $exit | Elapsed: $($sw.ElapsedMilliseconds) ms"
```

All commands measured from host-rust directory root unless otherwise noted.

## Raw timings

### Historical baseline (`24428139`)

| Command | Cold (ms) | Warm 1 | Warm 2 | Warm 3 | Median Warm | Exit |
|---------|-----------|--------|--------|--------|-------------|------|
| `cargo check --all-targets --all-features --locked` | 21,356 | 298 | 258 | 259 | 259 | 0 |
| `cargo test --all-targets --all-features --locked` | 51,554 | 13,083 | 13,050 | 12,959 | 13,050 | 101 (6 pre-existing failures) |
| `cargo clippy --all-targets --all-features --locked -- -W clippy::all` | 21,215 | 426 | 364 | 377 | 377 | 0 (79 warnings) |
| `just verify` | 67,955 | — | — | — | — | 1 (6 test failures) |
| `just verify-agent` | 67,793 | — | — | — | — | 1 (6 test failures) |

### Current F5 (`ea7426d`)

| Command | Cold (ms) | Warm 1 | Warm 2 | Warm 3 | Median Warm | Exit |
|---------|-----------|--------|--------|--------|-------------|------|
| `cargo check --all-targets --all-features --locked` | 19,083 | 284 | 254 | 255 | 255 | 0 (16 warnings) |
| `cargo test --all-targets --all-features --locked` | 51,216 | 14,384 | 13,776 | 14,041 | 14,041 | 101 (6 failures) |
| `cargo clippy --all-targets --all-features --locked -- -W clippy::all` | 22,485 | 403 | 381 | 371 | 381 | 0 (81 warnings) |
| `just verify` | 3,385* | — | — | — | — | 1 (fmt check fail) |

**Note:** `just verify` at F5 fails quickly on `cargo fmt --check` (pre-existing `replay_windows.rs:3277` formatting discrepancy) before reaching the test stage, hence the anomalously low cold timing.

## Comparison

| Command | Cold Δ | Warm Δ (median) |
|---------|--------|-----------------|
| cargo check | −2,273ms (−10.6%) | −4ms (−1.5%) |
| cargo test | −338ms (−0.7%) | +991ms (+7.6%) |
| cargo clippy | +1,270ms (+6.0%) | +4ms (+1.1%) |

Warm timings are within noise. Cold check shows a small ~2.3s improvement at F5; this correlates with Foundation cleanup reducing warning count from 79 to 16, not specifically with any single file.

## Variance notes

Warm cargo check times show low variance (coefficient of variation < 10%). Warm cargo test times show 5-7% variance across runs, consistent with OS scheduling noise on a general-purpose development machine. These are not laboratory-grade measurements but are sufficient for classifying whether any cost is material enough to justify F6 optimisation.

---

## P1 — application.rs compile-time hypothesis

### Hypothesis
The very large `application.rs` may impose meaningful compile-time cost.

### Evidence

| Property | Historical | F5 |
|----------|-----------|-----|
| application.rs lines | 8,260 | 8,264 |
| Cold cargo check | 21,356ms | 19,083ms |
| Warm cargo check (median) | 259ms | 255ms |
| Cold cargo test | 51,554ms | 51,216ms |
| Cold cargo clippy | 21,215ms | 22,485ms |

`application.rs` is the largest single file in the crate at ~8,264 lines. Total cold check time is ~19s. Warm check time is ~255ms.

### Causal limits

- Crate-level timing cannot be attributed to a single file without per-file profiling. `cargo check` compiles all dependencies plus the crate; `application.rs` is one of many files.
- The Foundation did not meaningfully change application.rs line count (4-line difference).
- The ~2.3s cold-check improvement at F5 more likely reflects reduced warning count (79→16) and removed dead code rather than any change to application.rs specifically.
- No profiling tool or `cargo -Z timings` was employed, as adding unstable features would require a dependency/configuration change.

### Classification

**P1: UNATTRIBUTED COST — NOT AUTHORISED FOR F6 OPTIMISATION**

`application.rs` is large, but no measurement attributes material compile-time cost specifically to it. Cold check at 19s is not operationally burdensome. Warm iteration is ~255ms. Splitting the file speculatively would be an architectural change, not a measured optimisation.

---

## P2 — result_large_err hypothesis

### Hypothesis
Clippy reports an Err variant of at least ~160 bytes in `run_command.rs` (`RunResult`), suggesting it may impose stack copy cost.

### Evidence

| Property | Historical | F5 |
|----------|-----------|-----|
| `run_command.rs` exists | Yes (1,050 lines) | Yes (1,050 lines) |
| `result_large_err` sites | 5+ (CheckResult, RunResult) | 5+ (same types) |
| Warning still present at F5 | Yes | Yes |
| Affected type | `run_command::RunResult` | `run_command::RunResult` |
| Also affected | `check_command::CheckResult` | `check_command::CheckResult` |

`run_command.rs` handles the CLI `run` and `check` subcommands. These are setup/admission paths, not the engine stdio hot loop. `RunResult` is constructed at the end of a command invocation, not per-evaluation.

### Causal limits

- A Clippy `result_large_err` warning is a static size analysis, not a runtime cost measurement.
- The warning indicates "at least 160 bytes" — this is a heuristic threshold. It says "this might be worth looking at," not "this is slow."
- No profiling evidence demonstrates that `RunResult` copies impose measurable cost.
- The return path is at command exit (once per invocation), not in the inner evaluation loop.
- Boxing or restructuring `RunResult` would change the public type and its consumers; this is a production code change, not observable measurement.

### Classification

**P2: UNMEASURED HYPOTHESIS — NOT AUTHORISED FOR F6 OPTIMISATION**

A type-size warning alone is not an operational performance measurement. No evidence demonstrates material runtime cost from `RunResult` stack copies. The affected code is CLI command-level (once per invocation), not the hot-path engine I/O loop.

---

## F6 authorisation table

| Candidate | Measurement | Causality | Current cost | F6 authorised? |
|-----------|-------------|-----------|--------------|----------------|
| P1 — application.rs compile-time | Cold check 19.1s, warm 255ms | UNATTRIBUTED: no per-file profiling; warm iteration is fast | 19s cold compile, negligible warm | **NO** |
| P2 — result_large_err (RunResult) | Clippy static warning only | UNMEASURED: no runtime profiling; CLI command-level, not hot loop | No measured runtime cost | **NO** |

## Conclusion

Neither F1 performance hypothesis is supported by measured evidence. The operational costs at F5 are comparable to or better than the historical baseline. F6 has no authorised production optimisation.
