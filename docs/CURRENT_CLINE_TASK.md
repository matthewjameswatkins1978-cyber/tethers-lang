# Current Implementation Task

Control contract: `1`
Task packet: `F8-ELAPSED-EVIDENCE — Automatic Command Timing`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Green`
Route: `OpenCode implements elapsed timing instrumentation`
Worker note: `docs/worker-notes/2026-08-09-f8-elapsed-evidence.md`
Base branch: `foundation/f8-nextest-concurrency`
Base commit: `10c5db45dd29192bb274f03d6b720f922171da38`
Implementation branch: `foundation/f8-elapsed-evidence`
Implementation checkpoint: `7173a99d61b838ec5150220a13c3fee88edae15d`
Rust change class: `NON_RUST`

## Objective

Make elapsed time ordinary project evidence. Whenever routine
verification/build/test commands run, record how long they took without
requiring extra runs and without changing their behaviour.

## Relevant background and existing behaviour

Routine verification commands (verify, test, fmt, check, etc.) already run
through `just`. No elapsed timing is captured today. There is no instrumentation
wrapper.

## Required behaviour

1. Create `scripts/invoke-timed.ps1` timing wrapper.
2. Update `justfile` to wrap routine commands.
3. Add `.tethers/timings.jsonl` to `.gitignore`.
4. Update `docs/WORKER_NOTE_TEMPLATE.md` Evidence guidance.

## Frozen decisions and invariants

- Do not change Rust or OCaml.
- Do not change tests.
- Do not change Nextest configuration.
- Do not change verification meaning.
- Do not change command arguments.
- Do not add a database, service, telemetry framework, benchmark, or dashboard.
- Do not modify the task-packet checker.

## Acceptance criteria

1. Commands behave exactly as before.
2. Native exit codes are preserved.
3. Normal stdout/stderr remains visible.
4. Timing appears automatically.
5. Timing JSONL is valid.
6. Timing history does not dirty Git.
7. Telemetry failure cannot fail successful work.
8. `just verify-agent` passes once.
9. Worker note records observed timings.
10. No extra benchmark/test runs created merely for timing.

## Required verification

- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`
- `git diff --check`
- Packet checker
- `just verify-agent` (full regression)

## Relevant components

### AUTHORISED PATHS
- `scripts/invoke-timed.ps1`
- `justfile`
- `.gitignore`
- `docs/WORKER_NOTE_TEMPLATE.md`

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-f8-elapsed-evidence.md`

## Forbidden changes

- No Rust source changes
- No OCaml source changes
- No test changes
- No Nextest configuration changes
- No CI changes
- No dependency policy changes
- No tool version changes
- No task-packet checker changes

## Stop conditions

STOP if a verification fails.
STOP if two materially similar implementation attempts fail.

## Expected pre-existing changes

None.
