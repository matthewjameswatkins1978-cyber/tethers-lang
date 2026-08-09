# Current Implementation Task

Control contract: `1`
Task packet: `F8-NEXTEST-CONCURRENCY-R3 — Single-Test Exclusion`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Green`
Route: `OpenCode probes Nextest concurrency for test suite wall-clock`
Worker note: `docs/worker-notes/2026-08-09-f8-nextest-concurrency.md`
Base branch: `foundation/f8-verify-parallel`
Base commit: `154997e82a391cc8d7f23da985fb55311d35a465`
Implementation branch: `foundation/f8-nextest-concurrency`
Implementation checkpoint: `cc5258224706a47172f85426bd1f1c46c9ec0377`
Rust change class: `NON_RUST`

## Objective

Enable Nextest `num-cpus` parallelism for the 1589-test suite with serial groups
for J24K2 integration tests and single-test thread-exclusive scheduling for one
timing-sensitive child_process test.

## Relevant background and existing behaviour

R1: Serial baseline ran 1589 tests in 192.4s.
R2: `num-cpus` + J24K2 serial group — 1588 passed, 1 failed
(`f2a_exit_distinguishable_from_timeout_and_disconnect` — process-exit race).
All other tests, including all J24K2, passed under parallelism.

## Required behaviour

1. Apply `num-cpus` parallelism with J24K2 serial group (proven in R2).
2. Add `threads-required = "num-test-threads"` override for the single
   failing child_process test.
3. Run full suite once with `--no-fail-fast`.
4. Keep if all 1589 pass.

## Target config

```
[profile.default]
retries = 0
fail-fast = true
test-threads = "num-cpus"

[test-groups]
j24k2-serial = { max-threads = 1 }

[[profile.default.overrides]]
filter = 'binary(j24k2_locked_single_step_executor)'
test-group = 'j24k2-serial'

[[profile.default.overrides]]
filter = 'test(=child_process::tests::f2a_exit_distinguishable_from_timeout_and_disconnect)'
threads-required = "num-test-threads"
```

## Frozen decisions and invariants

- Do not change any Rust source, test, script, dependency policy, CI, or tool version.
- Do not remove, weaken, or change any verification.
- J24K2 integration binary MUST remain serialized.
- Only `.config/nextest.toml` may be changed (plus task packet + worker note for closeout).

## Acceptance criteria

1. All 1589 tests pass under candidate config
2. J24K2 remains serialized (confirmed via `show-config`)
3. `threads-required` override matches exactly one test
4. Full suite `--no-fail-fast` run completes without failure

## Required verification

- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`
- `cargo nextest show-config test-groups` (confirm J24K2 group)
- `cargo nextest list -E '...'` (confirm single-test override match)
- `git diff --check`
- Packet checker

## Relevant components

### AUTHORISED PATH
- `.config/nextest.toml`

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-f8-nextest-concurrency.md`

## Forbidden changes

- No Rust source changes
- No test changes
- No justfile changes
- No PowerShell script changes
- No CI changes
- No dependency policy changes
- No warning inventory changes
- No tool version changes

## Stop conditions

STOP if a verification fails.
STOP if any test failure or concurrency-related instability appears.
STOP if two materially similar implementation attempts fail.

## Expected pre-existing changes

None.
