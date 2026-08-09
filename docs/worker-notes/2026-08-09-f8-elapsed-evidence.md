# Worker Note: F8-ELAPSED-EVIDENCE — Automatic Command Timing

Task: `F8-ELAPSED-EVIDENCE — Automatic Command Timing`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `10c5db45dd29192bb274f03d6b720f922171da38`

Implementation checkpoint: `7173a99d61b838ec5150220a13c3fee88edae15d`

## Requested outcome

Make elapsed time ordinary project evidence by wrapping routine
verification/build/test commands with a timing wrapper that records duration
without changing behaviour and without requiring extra runs.

## Changes made

1. `scripts/invoke-timed.ps1` — new timing wrapper script.
2. `justfile` — wrapped 9 recipes (fmt, check, test-rust, verify, agent-tools,
   test-agent, deps-policy, deps-advisories, deps-unused) with invoke-timed.
3. `.gitignore` — added `.tethers/timings.jsonl` to prevent timing history
   from dirtying Git.
4. `docs/WORKER_NOTE_TEMPLATE.md` — added evidence guidance rule: "When a
   required command emits elapsed timing, record that timing with its result.
   Never rerun a command solely to obtain timing."

## Decisions and assumptions

- Scripts are called directly from justfile recipes (not via nested `pwsh -File`)
  because PowerShell's `--` end-of-parameters token is not correctly forwarded
  through `pwsh.exe -File` to the child script.
- `[System.Diagnostics.Stopwatch]` used for high-resolution timing.
- JSONL record uses `[DateTime]::UtcNow.ToString('yyyy-MM-ddTHH:mm:ssK')` for
  second-precision UTC timestamps.
- Telemetry write failures are caught and emit a warning; they never change
  the wrapped command's exit code.

## Evidence

### Wrapper proof

A. Successful native command (exit 0): exit code preserved, one valid JSONL
   record with exit_code=0.
B. Failing native command (exit 7): exit code 7 preserved, JSONL record with
   exit_code=7.
C. JSONL records successfully parsed with `ConvertFrom-Json`.
D. Wrapped command stdout appears before the TIME summary line.

### Pre-checkpoint verification

- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`:
  PASS (no output = no formatting needed).
- `git diff --check`: PASS (no whitespace issues).

### verify-agent full regression (all 1589 tests)

| Command | Elapsed | Result |
| --- | --- | --- |
| task-packet | 1.0s | PASS |
| cargo-fmt | 1.1s | PASS |
| cargo-check | 0.3s | PASS |
| cargo-test | 43.4s | PASS (all 1589) |
| agent-tools | 3.1s | PASS |
| deps-policy | 0.6s | PASS |
| deps-advisories | 1.3s | PASS |
| nextest | 41.2s | PASS (1589/1589) |

All 1589 tests passed, commands behave exactly as before, exit codes preserved,
normal output visible, timing records written automatically.

### Git evidence

- `git diff --check`: PASS.
- `git status --short --branch`: clean (no dirty files; `.tethers/timings.jsonl`
  properly ignored).

## Publication evidence

Branch pushed: `foundation/f8-elapsed-evidence`

See completion report for full remote HEAD SHA, local==remote confirmation, and
final clean status.

## Discoveries

- `pwsh.exe -File` does not correctly forward `--` as an end-of-parameters
  token to the child script's parameter binder. The invoke-timed wrapper was
  designed with `ValueFromRemainingArguments` and `--` separator, but the
  justfile recipes must call the script directly rather than through a nested
  `pwsh -NoProfile -File` invocation.

## Remaining risks

- `deps-unused` recipe is instrumented with invoke-timed but not included in
  `verify-agent`; timings for that recipe are not captured during the
  required full regression. This is by design per the existing verify-agent
  recipe and does not affect the task scope.
- Timing JSONL file grows unbounded over time. No rotation, summarisation, or
  cleanup is implemented; this is deferred per the packet's scope restriction.

## Smallest next action

None — task complete. The next task should be compiled by Lucy.

## References

- Implementation checkpoint: `7173a99d61b838ec5150220a13c3fee88edae15d`
- Timing records: `.tethers/timings.jsonl` (local, git-ignored)
- Branch: `foundation/f8-elapsed-evidence`
- Base: `foundation/f8-nextest-concurrency` at `10c5db45`
