# Current Implementation Task

Control contract: `1`
Task packet: `F8-VERIFY-PARALLEL — Bounded Verifier Parallelism`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Green`
Route: `OpenCode parallelises just verify-agent independent lanes`
Worker note: `docs/worker-notes/2026-08-09-f8-verify-parallel.md`
Base branch: `foundation/f8-t1-test-warning-cleanup`
Base commit: `5b679b4f799d47ee0e5a76e247678c246baa3057`
Implementation branch: `foundation/f8-verify-parallel`
Implementation checkpoint: `5b679b4f799d47ee0e5a76e247678c246baa3057`
Rust change class: `NON_RUST`

## Objective

Reduce wall-clock time of `just verify-agent` by parallelising only independent
verification lanes, without removing, weakening, or changing any verification.

## Relevant background and existing behaviour

Current `verify-agent` runs all five sub-recipes sequentially:
verify → agent-tools → deps-policy → deps-advisories → test-agent.
`verify` itself runs packet checker, fmt check, cargo check, and cargo test
sequentially. `agent-tools`, `deps-policy`, `deps-advisories`, and `test-agent`
are independent of verify's per-recipe Cargo work. Parallelising the three
independent lanes reduces wall-clock time without changing any verification.

## Required behaviour

1. Baseline: 1 warm-up + 3 timed `just verify-agent` runs, record median.
2. Implement target topology in justfile.
3. Candidate: 1 warm-up + 3 timed `just verify-agent` runs, record median.
4. Keep change only if candidate median >= 10% faster than baseline AND all
   verification passes.
5. If improvement < 10%, revert justfile and close as measured NO-OP.

## Target topology

```
[private] verify-deps: deps-policy deps-advisories
[private] verify-agent-preflight: verify || agent-tools || verify-deps (parallel)
verify-agent: verify-agent-preflight && test-agent
```

`verify` runs sequentially (packet checker, fmt check, cargo check, cargo test).
In parallel: `agent-tools` and `verify-deps`.
After all parallel lanes complete: `test-agent` (cargo nextest).

## Frozen decisions and invariants

- Do not run cargo test and cargo nextest concurrently.
- Do not change any Rust source, test, script, dependency policy, CI, or tool version.
- Do not remove, weaken, or change any verification.
- `test-agent` MUST still run after the parallel preflight join.
- Only `justfile` may be changed (plus task packet + worker note for closeout).

## Acceptance criteria

1. All existing verification still runs
2. All verification passes
3. `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check` passes
4. `just --fmt --check` passes
5. `just verify` passes
6. `just verify-agent` passes
7. Candidate median >= 10% faster than baseline median
8. `git diff --check` passes
9. Packet checker passes
10. Diff touches only justfile + task packet + worker note

## Required verification

- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`
- `just --fmt --check`
- `just verify`
- `just verify-agent`
- `git diff --check`
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`

## Relevant components

### AUTHORISED PATH
- `justfile`

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-f8-verify-parallel.md`

## Forbidden changes

- No Rust source changes
- No test changes
- No script changes
- No dependency policy changes
- No warning inventory changes
- No F8 warning cleanup
- No CI changes
- No tool version changes

## Stop conditions

STOP if `just --fmt --check` changes any file outside justfile.
STOP if a verification fails.
STOP if candidate improvement < 10% (revert, close as NO-OP).
STOP if new flaky/interleaved failure appears.
STOP if two materially similar implementation attempts fail.

## Expected pre-existing changes

None.
