Task: `F8-NEXTEST-CONCURRENCY — Quick Probe`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `154997e82a391cc8d7f23da985fb55311d35a465`

Implementation checkpoint: `154997e82a391cc8d7f23da985fb55311d35a465`

## Requested outcome

Probe whether allowing normal Nextest parallelism materially reduces the 1589-test
suite runtime while preserving serialization only for J24K2 integration tests.

## Changes made

`.config/nextest.toml` was trialled with `num-cpus` parallelism and a `j24k2-serial`
test group (`max-threads = 1`) isolating only the `j24k2_locked_single_step_executor`
binary. Candidate failed with a concurrency-sensitive test in `child_process`
(`f2a_exit_distinguishable_from_timeout_and_disconnect`). Config was immediately
reverted to `test-threads = 1`. `.config/nextest.toml` is byte-identical to base.

No Rust source, tests, scripts, dependency policy, CI, or tool versions were changed.
The probe is a measured NO-OP.

## Evidence

**Baseline (serial, test-threads=1):**
- Wall-clock: 192.4s
- Tests: 1589 passed, 2 skipped

**Candidate (num-cpus parallelism, J24K2 serial group only):**
- Wall-clock: 7.2s (did not complete)
- Tests: 316 passed, 1 failed, 2 skipped, 1272 not run (fail-fast)
- Failed test: `child_process::tests::f2a_exit_distinguishable_from_timeout_and_disconnect`
- J24K2 serial group: confirmed via `cargo nextest show-config test-groups` — 9 tests in `j24k2-serial` (max-threads=1)

**Verification (retained base state):**
- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`: PASS
- `git diff --check`: PASS
- Packet checker: PASS

## Decisions and assumptions

The candidate failed immediately on a non-J24K2 test involving process exit,
timeout, and disconnect detection — a known concurrency-sensitive test. Per
the packet's stop condition, the config was reverted immediately. The test suite
contains process-level concurrency dependencies beyond J24K2 that prevent
trivial Nextest parallelism. Further investigation would require identifying and
grouping all concurrency-sensitive tests.

## Discoveries

The candidate failed on a non-J24K2 concurrency-sensitive test
(`f2a_exit_distinguishable_from_timeout_and_disconnect` in `child_process`).
The test suite contains process-level concurrency dependencies beyond J24K2 that
prevent trivial Nextest parallelism. The J24K2 serial group was correctly
configured and isolated; the failure was elsewhere.

## Remaining risks

None within packet scope. `.config/nextest.toml` is unchanged from base.

## Smallest next action

Lucy may decide whether to explore finer-grained test-group isolation in a future
packet, or accept the current serial configuration.

## References

- Baseline commit: `154997e82a391cc8d7f23da985fb55311d35a465`
- Branch: `foundation/f8-nextest-concurrency`
- Baseline run: 1589 passed, 2 skipped, 192.4s
- Candidate: failed at test 294/1589 (f2a_exit_distinguishable_from_timeout_and_disconnect)
