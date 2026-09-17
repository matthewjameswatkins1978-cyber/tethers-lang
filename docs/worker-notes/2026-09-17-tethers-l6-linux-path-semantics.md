# TETHERS L6 - Linux Path Semantics Repair

Task: `TETHERS L6 / Linux Path Semantics Repair`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `6b19b4c9af9fb047c847a82ef93f63f7de4381a5`

Implementation checkpoint: `a0ed61bf9b86752ecb51378ed0cc3ae1aace7bb5`

## Starting point

L6 started from the accepted L5 commit
`6b19b4c9af9fb047c847a82ef93f63f7de4381a5` on
`feature/tethers-l6-linux-path-semantics`. L5's relevant baseline was:

- raw Linux: `1241 passed; 65 failed; 2 ignored`;
- verified/default: `1405 passed; 116 failed; 2 ignored`;
- verified/single-thread: `1405 passed; 116 failed; 2 ignored`;
- missing verified engine: `0`;
- provider fixtures: `0`;
- unresolved Linux behaviour: `3`;
- concurrency cascades: `41`;
- expanded-target platform/provider: `72`.

## Failures

### `p2a_refuses_wrong_extension`

Original failure: the test passed `C:\\src` and `C:\\out.zip`. On Linux,
`Path` does not interpret those Windows drive literals as absolute paths, so
`pack()` returned the earlier `--source must be absolute` validation error and
the `.tetherplug` assertion was never testing the wrong-extension condition.

Intended scenario: an absolute source path and an absolute output path whose
extension is not `.tetherplug` must produce `invalid_cli_usage` with the
extension guidance.

Repair: construct unique absolute `source` and `output` paths below
`std::env::temp_dir()` using the existing `uuid` dependency. The paths need
not be created because the extension validation occurs before filesystem
access. The original assertions remain unchanged.

### `j13c_missing_trail_file_maps_to_not_found`

Original failure: the test passed `C:\\does-not-exist-j13c-test.jsonl`. On
Linux this is a relative path, so `run_trail()` correctly returned
`invalid_data` / `TRAIL_NOT_ABSOLUTE` before checking file absence.

Intended scenario: a valid absolute path to a missing regular trail file must
produce `not_found`, exit code `9`, and `TRAIL_NOT_FOUND`.

Repair: construct a unique path below `std::env::temp_dir()`, assert that it
does not exist, and pass it by reference. The production validation and all
semantic assertions remain unchanged.

## Structural discovery

Threadmoth was used for structural discovery. It confirmed both target files
are Rust syntax-understood files and exposed the relevant source/test regions.
Repository inspection found no shared path helper between these two modules;
the existing native temporary-path convention was already used by adjacent
tests in both modules. The repair therefore remains two local fixture changes,
with no new abstraction or repository-wide path cleanup.

## Test evidence

Targeted tests after repair:

- `p2a_refuses_wrong_extension`: `1 passed; 0 failed`.
- `j13c_missing_trail_file_maps_to_not_found`: `1 passed; 0 failed`.

Containing modules:

- `plug_pack::tests`: `18 passed; 0 failed`.
- `trail_command::tests`: `30 passed; 0 failed`.

Full measurements:

| Suite | Before | After |
| --- | --- | --- |
| Raw Linux | `1241 passed; 65 failed; 2 ignored` | `1243 passed; 63 failed; 2 ignored` |
| Verified/default | `1405 passed; 116 failed; 2 ignored` | `1407 passed; 114 failed; 2 ignored` |
| Verified/single-thread | `1405 passed; 116 failed; 2 ignored` | `1407 passed; 114 failed; 2 ignored` |

The verified runners exit non-zero because the already-classified remaining
concurrency and expanded-target failures are still present. The two path
failures disappeared without changing those families.

The provider fixture suite remained `21 passed; 0 failed`. The explicitly
rerun `c2a3a_group_join_after_all_terminals` still returned `Uncertain` and
failed its existing `ExecutionServiceResult::Completed` assertion; it was not
modified.

## Taxonomy

- Missing verified engine: `0 -> 0`.
- Provider fixtures: `0 -> 0`.
- Unresolved Linux behaviour: `3 -> 1`.
- Concurrency cascades: `41 -> 41`.
- Expanded-target platform/provider: `72 -> 72`.

The remaining unresolved Linux behaviour is exclusively
`c2a3a_group_join_after_all_terminals`, classified as concurrency.

## Verification

- `cargo fmt --all -- --check`: pass.
- `cargo check --workspace --locked`: pass.
- `cargo test --workspace --no-run --locked`: pass.
- `bl verify tethers`: pass.
- `git diff --check`: pass before the implementation checkpoint.
- Valid provenance verification: pass.
- Stale source commit, tampered binary hash, missing engine, and missing
  provenance manifest: all rejected before tests.
- Engine: `tethers-0.1/engine-ocaml/_build/default/bin/tethers_mcp_main.exe`.
- Engine source commit at checkpoint:
  `a0ed61bf9b86752ecb51378ed0cc3ae1aace7bb5`.
- Engine source tree: `7a8f760e79b5f788360e560ac6cd7c0d46349efe`.
- Engine SHA-256:
  `4c89ff26024733218311086f0fff66c0a7fe2e652363a636a7efd358bd60e9af`.
- OCaml `5.5.0`; Dune `3.24.0`.

`bl doctor tethers` was run while the two source edits were uncommitted and
reported `dirty_files: 2`, so readiness was false for that intermediate state.
It must be rerun after the final documentation commit and clean-tree check.
The native WSL image has no `pwsh`, so the PowerShell task-packet checker was
not run; this is recorded as an environment limitation rather than solved by
invoking Windows tooling.

## Production semantics

Production semantics changed: `NO`.

Only test fixtures changed. `pack()` and `run_trail()` production path
handling and assertions were not weakened.

## Windows behaviour

Windows behaviour changed: `NO`.

The existing Windows path literals and Windows-specific behaviour were not
altered. The tests now construct equivalent absolute scenarios on all hosts.

## Remaining Linux failure

`c2a3a_group_join_after_all_terminals` remains the sole unresolved Linux case
in this taxonomy. It returns `Uncertain` for `member-a` and is reserved for a
separate concurrency packet. L6 did not modify scheduling, locking, retry,
timing, or concurrency semantics.

## References

- L5 worker note: `docs/worker-notes/2026-09-17-tethers-l5-linux-failure-taxonomy.md`
- L4 runner: `scripts/run-rust-tests.sh`
- L4 provenance preparation: `scripts/prepare-current-engine.sh`
