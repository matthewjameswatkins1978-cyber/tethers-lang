# Worker Note

Task: `J14B - negative public integration matrix`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `8a06b0883f968f1561153bf8d54bfce3818fbde8`

Implementation checkpoint: `5b5b40e10113cac86f27d83dad51d01c2c26ea06`

## Requested outcome

Prove 11 J14 negative matrix rows through public check/run commands, one focused Rust test, and the existing debug probe boundary.

## Changes made

- `tethers-0.1/scripts/tethers-stdio-fixture.ps1` — added `run-explicit-error`, `run-invalid-output`, `run-hang-call` modes; J14B-R added `missing-tool` to instrumented initialize/tools/list recording so M02 can prove provider startup evidence and exact counts
- `tethers-0.1/host-rust/src/main.rs` — added `j14b_post_admission_intent_failure_retains_id` test and `j14b_failed_and_uncertain_result_anchor_kinds` test (both test-only, zero production change)
- `tethers-0.1/scripts/test-j14b-negative-matrix.ps1` — new 11-row matrix harness with explicit M01-M11 row identities, internal Rust proof invocation, strengthened pre-admission assertions, exact provider method counts, execution-specific Trail assertions, hard-bounded M09, exact M11 Trail records, and post-run integrity checks

## J14B-R correction

Lucy requested a stronger J14B negative matrix with explicit row identities and additional evidence. The original harness defects corrected in J14B-R were:

- Rows were numbered implicitly, making it possible to drop or duplicate a row silently.
- M06 relied on a separately-run Rust command rather than being consumed and asserted by the matrix entry point.
- M07/M08/M09 did not prove the standard Result Anchor kind (`capability.failed`/`capability.uncertain`) because the public envelope does not expose it for non-completed outcomes.
- M01/M05 did not prove complete workspace-tree non-mutation before provider launch.
- M02 did not assert provider startup counts because `missing-tool` did not record `initialize`/`tools/list`.
- M03/M04/M07/M08/M09/M10 did not assert exact provider method counts.
- M07/M08/M09 did not filter the Trail by execution ID and assert exactly one terminal outcome.
- M09 used an unbounded synchronous call.
- M11 reported the debug probe and release-binary rejection as separate rows.

Repaired matrix output:

```text
TEST: M01 malformed manifest
  PASS
TEST: M02 unavailable provider
  PASS
TEST: M03 Ask
  PASS
TEST: M04 Deny
  PASS
TEST: M05 stale pinned digest
  PASS
TEST: M06 post-admission durable intent failure
  PASS
TEST: M07 executor failure
  PASS
TEST: M08 invalid provider output
  PASS
TEST: M09 uncertain timeout
  PASS
TEST: M10 duplicate replay
  PASS
TEST: M11 causal depth beyond eight
  PASS

============================================
TOTAL: 11 rows, 11 passed, 0 failed
ASSERTIONS: 243
============================================
```

Focused Rust tests included in the matrix entry point:

- `j14b_post_admission_intent_failure_retains_id`
- `j14b_failed_and_uncertain_result_anchor_kinds`

Both are inside `#[cfg(test)]` in `main.rs`. The matrix harness runs `cargo test --locked j14b_ -- --nocapture`, requires the output to name both tests, requires zero failures, and asserts the test count is exactly two.

The repair commit SHA is reported externally in the implementation report; it is not inserted here.

## Decisions and assumptions

- Public CLI envelope exposes `result_anchor` only for `Completed` outcomes. `Failed` and `Uncertain` results carry the anchor internally but the `ExecutionServiceResult` enum lacks the `response` field for non-completed outcomes. M07/M08/M09 prove the standard Result Anchor kind through the focused `j14b_failed_and_uncertain_result_anchor_kinds` Rust seam, while the public command proves envelope status, exit code, machine code, execution ID, provider call count, and durable Trail evidence.
- `missing-tool` fixture mode now records `initialize` and `tools/list` exactly like the other instrumented run modes, without changing its protocol response or missing-tool semantics.
- J14B completes its implementation claim when all eleven rows pass. J14 becomes complete only after Lucy independently accepts and the candidate is published.

## Evidence

All required checks PASS:

- `cargo fmt --check` — PASS
- `cargo check --locked` — PASS (baseline warnings)
- `cargo check --locked --tests` — PASS (baseline warnings)
- `cargo test --locked j14b_ -- --nocapture` — PASS (2 tests)
- `cargo test --locked` — PASS (753 tests: 724 + 29)
- `cargo clippy --locked --all-targets --all-features` — PASS (baseline warnings)
- `cargo build --locked` — PASS
- `cargo build --locked --release` — PASS
- `test-j14b-negative-matrix.ps1` — PASS (11 rows, 243 assertions)
- `test-j14a-complete-scenario.ps1` — PASS (5 cases, 95 assertions)
- `test-j13a-check.ps1` — PASS (25 cases)
- `test-j13b-run.ps1` — PASS (10 cases; flaky in current environment, passed on retry)
- `test-j13c-trail.ps1` — PASS (19 cases)
- `test-host-denial.ps1` — PASS
- `test-host-execution-failure.ps1` — PASS
- `test-host-result-follow-up.ps1` — PASS
- `test-host-event-admission.ps1` — PASS
- `test-host-event-admission-trail.ps1` — PASS
- `check-fixtures.ps1` — PASS
- `test-mcp-transcripts.ps1` — PASS (15 cases)
- `test-engine.ps1` — PASS
- `demo.ps1` — PASS
- `check-tethers-toolchains.ps1` — PASS
- Cargo.lock SHA-256: `d323870ea02f09391a5d0d9aa0e9a701cf686a5ac005b840ee7218e70edb5602` confirmed
- `git diff --check 8a06b0883f968f1561153bf8d54bfce3818fbde8..HEAD` — PASS (whitespace clean)
- Branch range contains exactly the six authorised paths

## Discoveries

- `run_command::map_execution_result` exposes `result_anchor` in public envelope only for `Completed`. `Failed`/`Uncertain` lack the `response` field in their `ExecutionServiceResult` variants. The public command proves the envelope, execution ID, provider call count, and durable Trail outcome; the anchor kind is proved by the focused Rust seam.
- `test-j13b-run.ps1` test 10 (Ctrl+C interruption) is flaky in the current native Windows environment, returning `unavailable` instead of `interrupted` on some runs. It passes on retry. This is pre-existing and unrelated to the J14B-R changes.

## Remaining risks

- The public `run` envelope still does not expose the standard Result Anchor kind for `Failed`/`Uncertain` outcomes. The matrix relies on the test-only Rust seam for that evidence; future 0.2 work may decide whether to expose it publicly.
- `test-j13b-run.ps1` interruption case is flaky; if the flake rate increases it should be investigated separately without weakening the public interruption contract.

## Smallest next action

J15: Consolidate the 0.2 Failure Matrix — merge J14A positive and J14B negative evidence into one discoverable verification entry point.

## References

- Base commit: `8a06b0883f968f1561153bf8d54bfce3818fbde8`
- Implementation commit: `5b5b40e10113cac86f27d83dad51d01c2c26ea06`
- Repair commit: reported externally in the implementation report
- Branch: `opencode/j14b-negative-public-matrix`
- `docs/DECISIONS.md` — 2026-07-31 J14B boundary decision
