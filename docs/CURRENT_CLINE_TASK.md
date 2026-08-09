# Current Implementation Task

Control contract: `1`
Task packet: `F8-NEXTEST-CONCURRENCY — Quick Probe`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Green`
Route: `OpenCode probes Nextest concurrency for test suite wall-clock`
Worker note: `docs/worker-notes/2026-08-09-f8-nextest-concurrency.md`
Base branch: `foundation/f8-verify-parallel`
Base commit: `154997e82a391cc8d7f23da985fb55311d35a465`
Implementation branch: `foundation/f8-nextest-concurrency`
Implementation checkpoint: `154997e82a391cc8d7f23da985fb55311d35a465`
Rust change class: `NON_RUST`

## Objective

Determine whether allowing normal Nextest parallelism materially reduces the
1589-test suite runtime while preserving serialization only where genuinely required.

## Relevant background and existing behaviour

Current `test-threads = 1` in `.config/nextest.toml` runs all 1589 tests serially.
J24K2 interruption-sensitive integration tests share process-global state and
must remain serialized.

## Required behaviour

1. Run baseline serial suite once, record wall-clock.
2. Change nextest config: normal tests use `num-cpus` parallelism; J24K2
   integration binary assigned to `j24k2-serial` test group with `max-threads = 1`.
3. Run candidate once.
4. Keep if >= 20% faster; revert on any test failure or < 20% improvement.

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
```

## Frozen decisions and invariants

- Do not change any Rust source, test, script, dependency policy, CI, or tool version.
- Do not remove, weaken, or change any verification.
- J24K2 integration binary MUST remain serialized.
- Only `.config/nextest.toml` may be changed (plus task packet + worker note for closeout).

## Acceptance criteria

1. All expected tests pass in candidate
2. J24K2 remains serialized (confirmed via `show-config`)
3. Ordinary tests gain parallel execution
4. Candidate >= 20% faster than baseline
5. Revert immediately on any test failure

## Required verification

- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`
- `cargo nextest show-config test-groups` (confirm J24K2 group)
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
STOP if any test failure or concurrency-related instability appears (revert immediately).
STOP if two materially similar implementation attempts fail.

## Expected pre-existing changes

None.
