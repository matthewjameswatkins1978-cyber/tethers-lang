# Tethers L3 — Platform-Correct Test Boundary

## Authority

- Starting SHA: `738da0f123145d9be9876fa22e9c892c67200d80`
- Branch: `feature/tethers-l3-platform-test-boundary`
- Parent: `feature/tethers-l2-linux-failure-taxonomy`
- Worktree: native WSL checkout at `/home/matmus/biscuit-linux/projects/tethers`
- Starting worktree: clean
- Windows checkout: not opened or modified

## Changes

This slice changes test compilation and test fixtures only. No production
semantics changed.

### Windows-only test boundaries

- `src/lib.rs`: the complete installation/current-trust/recovery/publication
  test families are now compiled only on Windows. They exercise the existing
  Windows installation launch path, which requires `SystemRoot` and Windows
  installation-lock semantics.
- `src/installation_execution.rs`: the exclusive-lock `lock_tests` family is
  Windows-only. It proves Windows handle sharing and inheritance behaviour;
  the portable transition tests remain available on Linux.
- `src/child_process.rs`: PowerShell fixture launch, Job Object cleanup,
  Windows descendant/process-boundary, and live Windows stderr tests are
  Windows-only. Generic interrupt, nonexistent-command, and portable child
  error tests remain shared.
- `src/host_execution.rs`: the retained PowerShell provider test is
  Windows-only. The test proves Windows provider launch/retention behaviour.
- Windows-only support imports and test helpers were cfg-gated as well, so
  Linux does not compile unused Windows fixture machinery.

These boundaries are semantic, not failure-based: each gated family asserts
PowerShell, `SystemRoot`, Windows executable/path, Job Object, handle, or
Windows exclusive-lock behaviour. No fake `SystemRoot`, PowerShell, `.exe`,
or Windows path was introduced on Linux.

### Cross-platform harness repairs

- `application.rs`, `plug_command.rs`, and `plug_install_command.rs` now use
  platform-native absolute test fixture paths. The tests still prove the
  same absolute-vs-relative CLI and package-path contracts on both platforms.
- `runtime_config.rs` uses platform-native absolute strings for the generic
  absolute-source/manifest rejection tests.
- `validation.rs` uses a unique native temporary absolute path for the
  nonexistent-directory refusal test, preserving the validation contract
  without a Windows drive-letter fixture.

These changes do not alter production path semantics; they repair test input
construction so portable semantic tests are actually portable.

## Gating evidence

The meaningful boundaries introduced are `#[cfg(windows)]` on the Windows
installation test modules, Windows installation lock tests, PowerShell and
Windows process-boundary tests, and their test-only helpers. The boundary is
at the existing semantic test-family/module level, not individual assertions.

`SystemRoot` enters through the reviewed Windows PowerShell launch environment
and is not a universal Rust-runtime requirement. Linux therefore does not
receive a fabricated environment variable.

## Production changes

No production semantics changed. The diff contains test-module cfg boundaries,
test-only helper cfg boundaries, and test fixture construction changes only.
No OCaml/Core, Tether syntax, provider execution, replay, scheduling,
outcome, or public runtime behaviour was changed.

## Test delta

L2 baseline:

```text
1258 passed
245 failed
4 ignored
```

L3 final native Linux workspace run:

```text
1225 passed
81 failed
2 ignored
```

The reduction is the removal of the 164 inapplicable Windows-boundary
failures. The remaining failures map exactly to the excluded L2 categories:

| L2 category | Before | Remaining | Change |
|---|---:|---:|---:|
| Windows launch environment | 144 | 0 | -144 |
| Windows path semantics | 16 | 0 | -16 |
| Windows process boundary | 4 | 0 | -4 |
| Concurrency cascades | 41 | 41 | 0 |
| Missing verified OCaml engine | 21 | 21 | 0 |
| Provider fixtures | 16 | 16 | 0 |
| Unresolved Linux behaviour | 3 | 3 | 0 |

The complete raw run is preserved at:
`docs/worker-notes/evidence/tethers-l3-linux-tests.log`.

## Three unresolved tests

These were investigated only and were not changed. Because workspace-wide
exact filtering does not match module-qualified unit-test names, the
qualified library invocations were used.

- `cargo test --lib host_execution::tests::c2a3a_group_join_after_all_terminals -- --exact --nocapture`
  — failed: `execute_group_concurrent returned: Uncertain`, then the test
  asserted that the result was `Completed`.
- `cargo test --lib plug_pack::tests::p2a_refuses_wrong_extension -- --exact --nocapture`
  — failed: `err.message.contains(".tetherplug")`.
- `cargo test --lib trail_command::tests::j13c_missing_trail_file_maps_to_not_found -- --exact --nocapture`
  — failed: actual error code `invalid_data`, expected `not_found`.

These remain assigned to the later unresolved-Linux-behaviour slice.

## Verification evidence

- `cargo fmt --all -- --check`: pass
- `cargo check --workspace`: pass
- `cargo test --workspace --no-run`: pass
- `cargo test --workspace`: completed with `1225 passed; 81 failed; 2 ignored`
- `git diff --check`: pass
- `bl doctor tethers`: pass in the prepared login-shell WSL environment
- `bl verify tethers`: must be recorded at closeout after the final commit

The remaining compiler warnings are existing duplicate-target, unused/dead
code, and platform-specific test-support debt; no warning was promoted to a
semantic failure and no global lint suppression was added.

## Scope and stop conditions

No WSL configuration, global Git configuration, authentication, PATH, shell
startup, toolchain version, or Windows checkout was changed. L2 engine,
provider, concurrency, and unresolved behaviour groups were not repaired in
this packet. The branch is ready for final repository verification and normal
publication, but must not be merged by this task.
