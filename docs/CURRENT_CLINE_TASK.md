# TETHERS L7 - Concurrency Root-Cause Investigation

Task: `TETHERS L7 / Concurrency Root-Cause Investigation`

Control contract: `1`

Status: `IN_PROGRESS`

Task colour: `Amber`

Owner: `Codex`

Route: `Investigate the named group-join failure from the fresh merged L6 baseline, repair only the demonstrated shared Linux test-harness cause, verify the coordinator invariant, classify the wider concurrency population, document evidence, commit and push the task branch. Do not merge.`

Base commit: `15c85c7ffffb56614b0667b7a39164529a1b7fc4`

Worker note: `docs/worker-notes/2026-09-17-tethers-l7-concurrency-root-cause.md`

Suggested branch:

`feature/tethers-l7-concurrency-root-cause`

## Objective

Determine why `c2a3a_group_join_after_all_terminals` previously observed an
uncertain result and premature-looking GroupJoin state on Linux. Establish
whether the defect is in the provider fixture/test harness or production
coordinator, and repair only the smallest demonstrated shared cause.

## Relevant background and existing behaviour

L6 merged at `15c85c7ffffb56614b0667b7a39164529a1b7fc4`. The verified Linux
engine, provider fixtures, and Linux path-semantics families are already
resolved. The prior verified suites reported 41 concurrency failures and 72
expanded-target platform/provider failures. The named test failed repeatedly
before L7 changes.

Initial L7 reproduction found the C2A3a harness hard-coded to PowerShell and
the Linux fixture used a different member-token projection from the Windows
fixture. The Linux fixture must preserve the same barrier semantics without
using Windows executables.

## Required behaviour

1. Reproduce the named test under default and single-threaded execution and
   record repeatability.
2. Use Threadmoth for structural discovery where cross-file or repeated
   concurrency-harness structure is involved.
3. Ensure GroupJoin is emitted only after every group member is terminal.
4. Repair a test/fixture boundary if that is the proven cause; change
   production coordination only if an independent reproduction proves it.
5. Classify the wider concurrency population without pretending this slice
   resolves unrelated failures.
6. Commit and normally push the bounded task branch without merging it.

## Relevant components

- `tethers-0.1/host-rust/src/host_execution.rs`
- `tethers-0.1/scripts/tethers-stdio-fixture.sh`
- `tethers-0.1/scripts/tethers-stdio-fixture.ps1`
- `scripts/run-rust-tests.sh`
- `verification/current-engine-provenance.json`
- `docs/worker-notes/2026-09-17-tethers-l7-concurrency-root-cause.md`

## Frozen decisions and invariants

- GroupJoin is valid only after all group members reach terminal state.
- Provider fixtures must be semantically equivalent across Windows and Linux.
- No Windows executable may be used as the Linux fixture route.
- No production concurrency semantics, replay semantics, scheduling policy,
  timeout, or locking behaviour may be changed without proof of a production
  defect.
- Do not add sleeps, globally increase timeouts, disable tests, weaken
  assertions, or serialize unrelated production.
- L4 provenance remains fail-closed and unchanged.
- Windows behaviour remains unchanged.

## Acceptance criteria

1. The named failure is reproduced before repair and its root cause is
   evidenced.
2. The named test passes repeatedly under default and single-threaded runs.
3. Directly related concurrency harnesses use the correct platform fixture and
   do not poison later tests through avoidable fixture deadlocks.
4. The coordinator invariant is tested by existing assertions and no
   production semantic change is made without independent evidence.
5. The wider concurrency failures are mapped into honest families.
6. Formatting, compilation, verified engine, `bl verify tethers`,
   `bl doctor tethers`, and `git diff --check` are recorded honestly.
7. The worker note records exact before/after evidence and remaining failures.
8. The final branch is clean, committed, pushed normally, and unmerged.

## Required verification

- repeated named test under default and `--test-threads=1`
- containing C2A3a concurrency tests
- raw Linux suite
- verified/default and verified/single-thread suites
- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo test --workspace --no-run --locked`
- `bl verify tethers`
- `bl doctor tethers`
- valid, stale, tampered, missing-engine, and missing-manifest provenance
  checks
- `git diff --check`
- final clean status and remote SHA equality

## Forbidden changes

- No broad repair of the 41 concurrency failures.
- No repair of the 72 expanded-target platform/provider failures.
- No production scheduling, locking, replay, retry, timeout, or outcome
  redesign.
- No sleeps or retries added to conceal a race.
- No ignored tests, disabled tests, weakened assertions, fake Windows
  environments, dependency additions, or global configuration changes.
- No changes to the Windows checkout, WSL configuration, toolchains, L4
  provenance contract, or main.

## Stop conditions

- Stop and report if the named test still fails after the shared fixture
  boundary is semantically correct and the remaining evidence points to a
  production race requiring architectural judgement.
- Stop if a production change would be needed to repair a test-only platform
  assumption.
- Stop if provenance becomes stale or fail-open.
- After two materially similar failed approaches, stop and record the smallest
  unresolved issue.

## Expected pre-existing changes

None.

## Implementation scope

- `tethers-0.1/host-rust/src/host_execution.rs`
- `tethers-0.1/scripts/tethers-stdio-fixture.sh`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-09-17-tethers-l7-concurrency-root-cause.md`
