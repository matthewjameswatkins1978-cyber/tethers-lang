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

- `tethers-0.1/scripts/tethers-stdio-fixture.ps1` — added `run-explicit-error`, `run-invalid-output`, `run-hang-call` modes
- `tethers-0.1/host-rust/src/main.rs` — added `j14b_post_admission_intent_failure_retains_id` test (test-only, zero production change)
- `tethers-0.1/scripts/test-j14b-negative-matrix.ps1` — new 11-row matrix harness

## Decisions and assumptions

- Public CLI envelope exposes `result_anchor` only for `Completed` outcomes. `Failed` and `Uncertain` results carry the anchor internally but the `ExecutionServiceResult` enum lacks the `response` field for non-completed outcomes. M07/M08/M09 verify through exit code, machine code, execution ID, provider call count, and durable Trail evidence.
- `missing-tool` fixture mode does not record `initialize` in the marker. M02 verifies zero tools/call rather than provider startup count.

## Evidence

All tests PASS:

- `cargo fmt --check` — PASS
- `cargo check --locked` — PASS (baseline warnings)
- `cargo check --locked --tests` — PASS (baseline warnings)
- `cargo test --locked j14b_ -- --nocapture` — PASS (1 test)
- `cargo test --locked` — PASS (752 tests: 723 + 29)
- `cargo clippy --locked --all-targets --all-features` — PASS (baseline warnings)
- `cargo build --locked` — PASS
- `cargo build --locked --release` — PASS
- `test-j14b-negative-matrix.ps1` — PASS (11 cases, 131 assertions)
- `test-j14a-complete-scenario.ps1` — PASS (5 cases, 95 assertions)
- `test-j13a-check.ps1` — PASS
- `test-j13b-run.ps1` — PASS (10 cases)
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
- `git diff --check 8a06b..HEAD` — PASS (whitespace clean)
- Matrix row results: M01-M11 all PASS

## Discoveries

- `run_command::map_execution_result` exposes `result_anchor` in public envelope only for `Completed`. `Failed`/`Uncertain` lack the `response` field in their `ExecutionServiceResult` variants. Matrix rows verify failed/uncertain through envelope status, exit code, machine code, provider call count, execution ID, and durable Trail evidence instead.

## Remaining risks

None known within packet scope.

## Smallest next action

J15: Consolidate the 0.2 Failure Matrix — merge J14A positive and J14B negative evidence into one discoverable verification entry point.

## References

- Base commit: `8a06b0883f968f1561153bf8d54bfce3818fbde8`
- Implementation commit: `5b5b40e10113cac86f27d83dad51d01c2c26ea06`
- Branch: `opencode/j14b-negative-public-matrix`
- `docs/DECISIONS.md` — 2026-07-31 J14B boundary decision
