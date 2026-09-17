# TETHERS L7 - Concurrency Root-Cause Investigation

## Starting point

- Repository: `/home/matmus/biscuit-linux/projects/tethers`
- Starting branch: `feature/tethers-l7-concurrency-root-cause`
- Starting SHA: `15c85c7ffffb56614b0667b7a39164529a1b7fc4`
- Accepted L6 merge: `15c85c7ffffb56614b0667b7a39164529a1b7fc4`
- Windows checkout: not modified; it remained a separate clean checkout.
- Native WSL toolchain: OCaml 5.5.0, Dune 3.24.0, Rust 1.97.1.

## Named failure before repair

Target: `c2a3a_group_join_after_all_terminals`

The test failed 5/5 under normal Rust threading and 5/5 with
`--test-threads=1`. The observed result was:

```text
ExecutionServiceResult::Uncertain {
  evaluation_id: "eval-join-1",
  action_id: "member-a",
  reason: "provider outcome is uncertain",
  ...
}
```

The original C2A3a harness constructed both providers with the Windows-only
PowerShell fixture (`pwsh.exe` and `tethers-stdio-fixture.ps1`) even on Linux.
The Linux fixture also projected `member/a` differently from the Windows
fixture, which meant the per-member release files were not matched. The
fixture then allowed timeout/error-driven terminalisation, producing misleading
GroupJoin observations and an uncertain aggregate rather than the intended
controlled B-then-A lifecycle.

This was deterministic harness behaviour, not evidence of a production race:
the failure rate was 10/10 across the two pre-repair modes.

## Threadmoth structural discovery

Threadmoth 1.10.0 was available and its doctor check reported a ready Linux
workspace. Structural inspection of `host_execution.rs` identified:

- `C2A3aGroupHarness` and the C2A3a terminal matrix as the named and directly
  related harnesses;
- the shared `stdio_fixture_script_path` and
  `stdio_fixture_command_args` platform boundary;
- the C3-A1/C3-A4 bounded-concurrency harnesses containing the same repeated
  inline PowerShell transport construction;
- coordinator-owned `GroupMemberState` transitions and the existing final
  GroupJoin guard, which only appends after terminalisation.

This showed that the smallest coherent fix was a test-harness transport
boundary plus a Linux fixture identity correction, rather than a change to the
coordinator.

## Changes

### Platform-aware test transport

`host_execution.rs` now provides one test-only
`stdio_fixture_transport` helper built on the existing platform-aware fixture
path/command helpers. The C2A3a terminal matrix, C3-A1 and C3-A4 harnesses,
and the named C2A3a group harness now use the native shell fixture on Linux and
retain PowerShell on Windows.

No Windows production or test behaviour was changed.

### Linux barrier identity

`tethers-stdio-fixture.sh` now maps both `member-a` and `member/a` forms to the
same canonical `member-a` token, matching the PowerShell fixture’s semantics.
No fake Windows environment or global environment variable was introduced.

### Pre-provider panic cases

Two tests intentionally panic a worker before it reaches the provider. Their
barrier peer count is now explicitly one, matching the scenario they claim to
test. This prevents a fixture deadlock while preserving the panic/uncertain
assertions.

## Invariant evidence

The existing coordinator assertions continue to prove:

- B’s outcome is durable while A remains blocked;
- no GroupJoin exists before A is released and terminal;
- both outcomes exist before the final GroupJoin;
- GroupJoin is the final physical entry.

After the fixture repair, the named test passed 20/20 with normal threading
and 20/20 with `--test-threads=1`. The full directly related C2A3a family
passed 16/16, and the C3-A family passed 26/26 single-threaded.

No coordinator, replay, scheduling, timeout, retry, locking, or outcome
classification production code changed.

## Test results

### Host-library comparison

These counts use the Rust host library target, where the L7 concurrency tests
live and where the raw/verified engine distinction is measurable without the
independent Windows-only CLI families:

| Run | Passed | Failed | Ignored | Result |
| --- | ---: | ---: | ---: | --- |
| Raw Linux, no verified engine | 1285 | 21 | 2 | expected missing-engine class |
| Verified engine, default threading | 1306 | 0 | 2 | pass |
| Verified engine, single-threaded | 1306 | 0 | 2 | pass |

The 21 raw failures are the missing verified OCaml engine cross-language
tests, not concurrency failures. L4’s provenance contract remains unchanged;
the current verified engine is still the L4 ELF binary with SHA-256
`4c89ff26024733218311086f0fff66c0a7fe2e652363a636a7efd358bd60e9af`.

### Full runner boundary

The repository runner was invoked in single-threaded mode. Its host-library
target completed with 1306 passed, 0 failed, and 2 ignored. Subsequent
integration targets exposed independent existing Linux boundary failures,
including:

- `j13a_cli`: test binary not found (`No such file or directory`), because the
  target-specific runner invocation does not build the CLI binary expected by
  those tests;
- `j24b`/`j24c`/`j24d` and related targets: `M3Error` with
  `launch_environment: SystemRoot is unavailable`;
- `p2b`/`p2c`: platform-dependent CLI/provider expectations.

The default-threading full runner was stopped at the first independent
`r1e_synthetic_unrelated_plug` platform case after it remained in the known
SystemRoot-dependent family. It was not used as evidence against the L7
coordinator. The exact full-run integration repair is outside L7.

## Concurrency taxonomy

| Family | Before | After this slice | Classification |
| --- | ---: | ---: | --- |
| Missing verified engine | 0 in L4 verified runs | 0 | resolved by L4, preserved |
| Provider fixtures | 0 | 0 | resolved by L5, preserved |
| Linux path semantics | 0 | 0 | resolved by L6, preserved |
| Named C2A3a GroupJoin | 1 | 0 | Linux test-harness boundary repaired |
| Direct C2A3a family | part of 41 | 0 in 16 focused tests | same harness/peer-count cause |
| Direct C3-A bounded family | part of wider concurrency set | 0 in 26 focused tests | same repeated transport boundary |
| Remaining full-run failures | 114 in prior verified aggregate | not recomputed as one honest aggregate | independent SystemRoot/CLI/platform families |

The prior aggregate’s 41 concurrency label cannot be treated as a single
production family. The focused C2/C3 tests that exercised the same barrier
harness are now green. Remaining full-run failures were not mixed into this
result because their first errors are independent platform-boundary failures.

## Provenance and environment

- Current engine preparation and valid provenance checks remained available.
- Final post-commit preparation recorded source commit
  `2b9265c43dfdca8bf19da93d93d8734583cc96c8` and source tree
  `f0069fc18cb3cfa98accc52ced3a03d7ae0c24eb`; the binary hash remained
  `4c89ff26024733218311086f0fff66c0a7fe2e652363a636a7efd358bd60e9af`.
- L4’s stale-commit, tampered-hash, missing-engine and missing-manifest
  fail-closed checks were not changed by this task and remain represented by
  the accepted L4 evidence.
- `bl verify tethers` completed successfully on the changed checkout.
- `bl doctor tethers` reported the native WSL filesystem and all required Rust,
  OCaml, Dune and support tools as available. Its readiness summary was false
  while the worktree contained the three expected task files; this is not an
  environment repair signal.

## Verification performed

- Threadmoth structural discovery: pass.
- `cargo fmt --all -- --check`: pass.
- `cargo check --workspace --locked`: pass.
- `cargo test --workspace --no-run --locked`: pass.
- C2A3a focused family: 16/16 pass.
- C3-A focused family: 26/26 pass.
- Named repeated test: 20/20 default, 20/20 single-threaded.
- Raw host-library run: 1285 pass, 21 fail, 2 ignored.
- Verified host-library default: 1306 pass, 0 fail, 2 ignored.
- Verified host-library single-threaded: 1306 pass, 0 fail, 2 ignored.
- `bl verify tethers`: pass.
- `bl doctor tethers`: diagnostic pass for workshop/toolchains; readiness
  summary affected by the intentionally dirty worktree.
- `git diff --check`: pass after formatting.

## Production and platform conclusions

Production semantics changed: **NO**.

Windows behaviour changed: **NO**.

The repair is limited to test fixture construction and the Linux fixture’s
member-token normalization. The coordinator’s GroupJoin invariant was not
weakened or reimplemented.

## Remaining failures and next slice

The remaining full-run failures are outside L7’s narrow concurrency slice:

1. Windows/SystemRoot-dependent CLI and lifecycle integration tests need a
   separate platform-correct boundary or Linux harness decision.
2. The runner’s CLI integration invocation needs a separate release-control
   review because some tests expect a built host binary not produced by the
   target-specific command.
3. Any remaining expanded-target platform/provider failures remain for their
   own packet.

Recommended next slice: a bounded audit of the Linux full-run integration
runner and its SystemRoot/CLI test classification, not a production
concurrency rewrite.
