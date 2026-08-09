Task: `F8-NEXTEST-CONCURRENCY-R3 — Single-Test Exclusion`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `154997e82a391cc8d7f23da985fb55311d35a465`

Implementation checkpoint: `cc5258224706a47172f85426bd1f1c46c9ec0377`

## Requested outcome

Enable Nextest `num-cpus` parallelism for the 1589-test suite, with J24K2
integration tests serialized and one timing-sensitive test given exclusive
thread access.

## Changes made

`.config/nextest.toml`: configured `test-threads = "num-cpus"`, added
`j24k2-serial` test group (`max-threads = 1`) for the `j24k2_locked_single_step_executor`
integration binary, and added `threads-required = "num-test-threads"` override
for `child_process::tests::f2a_exit_distinguishable_from_timeout_and_disconnect`.

No Rust source, tests, scripts, dependency policy, CI, or tool versions were changed.

## Evidence

### R1 — Baseline (serial, test-threads=1)
- Wall-clock: 192.4s
- Tests: 1589 passed, 2 skipped

### R2 — Failure map (num-cpus, J24K2 serial group, --no-fail-fast)
- Wall-clock: 35.1s
- Tests: 1588 passed, 1 failed, 2 skipped
- Failed: `child_process::tests::f2a_exit_distinguishable_from_timeout_and_disconnect`
  (process-exit/protocol-line race)

### R3 — Candidate (num-cpus, J24K2 serial group, single-test threads-required)
- Wall-clock: **39.5s** (79.5% faster than baseline)
- Tests: **1589 passed**, 2 skipped, 0 failed

### Verification (against implementation checkpoint)
- `cargo nextest list -E 'test(=child_process::tests::f2a_exit_distinguishable_from_timeout_and_disconnect)'`: matches exactly **1 test**
- `cargo nextest show-config test-groups`: J24K2 serial group confirmed (9 tests, max-threads=1)
- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`: PASS
- `cargo nextest run --no-fail-fast`: PASS (1589/1589)
- `git diff --check`: PASS
- Packet checker: PASS

## Decisions and assumptions

The single failing test is a genuine timing sensitivity: under concurrent CPU
load, the child process's exit notification and the protocol-line read timeout
race. `threads-required = "num-test-threads"` gives the test exclusive access
to the global Nextest thread pool, preventing interference from parallel tests
while allowing all other tests to run at full parallelism.

## Discoveries

Only 1 of 1589 tests (`f2a_exit_distinguishable_from_timeout_and_disconnect`)
is sensitive to Nextest-level parallelism. All J24K2, M3 lifecycle, conformance,
and other process-heavy tests pass correctly under `num-cpus` as long as the
J24K2 integration binary is isolated. The `threads-required` override is the
narrowest possible fix — a serial group would have been unnecessary for a
single test.

## Remaining risks

None within packet scope. The test suite passes consistently under the candidate
config. The `threads-required` override may add marginal wall-clock overhead if
the test pool is large, but the measured 39.5s is well within acceptable range.

## Smallest next action

Lucy may accept this as the new default Nextest configuration.

## References

- Baseline commit: `154997e82a391cc8d7f23da985fb55311d35a465`
- Implementation checkpoint: `cc5258224706a47172f85426bd1f1c46c9ec0377`
- Branch: `foundation/f8-nextest-concurrency`
- R1 baseline: 1589 passed, 2 skipped, 192.4s
- R2 failure map: 1588 passed, 1 failed (f2a_exit), 35.1s
- R3 candidate: **1589 passed**, 0 failed, **39.5s** (79.5% improvement)
