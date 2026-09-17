# TETHERS L6 - Linux Path Semantics Repair

Task: `TETHERS L6 / Linux Path Semantics Repair`

Control contract: `1`

Status: `COMPLETE`

Task colour: `Green`

Owner: `Codex`

Route: `Repair the two bounded Linux path-semantics test fixtures from the accepted L5 branch, verify the concurrency case remains untouched, document the evidence, commit and push the task branch. Do not merge.`

Base commit: `6b19b4c9af9fb047c847a82ef93f63f7de4381a5`

Evidence checkpoint: `a0ed61bf9b86752ecb51378ed0cc3ae1aace7bb5`

Worker note: `docs/worker-notes/2026-09-17-tethers-l6-linux-path-semantics.md`

Suggested branch:

`feature/tethers-l6-linux-path-semantics`

## Objective

Make `p2a_refuses_wrong_extension` and
`j13c_missing_trail_file_maps_to_not_found` express their intended semantic
scenarios on Linux using native temporary paths. Preserve production path
validation, Windows behaviour, the verified engine, provider fixtures, and the
out-of-scope concurrency failure.

## Relevant background and existing behaviour

L5 reduced provider fixture failures from 18 to 0 and left three unresolved
Linux cases. The group-join case is a concurrency failure and remains outside
L6. The two path tests use Windows drive literals; on Linux those are relative
paths, so the tests exercise the wrong validation branches. Existing
production code already handles absolute missing paths and wrong extensions
correctly.

## Required behaviour

1. Reproduce and repair the two path-semantics fixtures with native,
   cross-platform temporary paths without weakening assertions.
2. Keep the `c2a3a_group_join_after_all_terminals` concurrency failure and the
   remaining concurrency/expanded-target families outside this task.
3. Preserve L5 provider-fixture results, L4 engine provenance, Windows path
   semantics, and production behaviour.
4. Run targeted, containing-module, raw, verified/default, and
   verified/single-thread checks and document exact before/after evidence.
5. Commit and normally push the bounded task branch without merging it.

## Relevant components

- `tethers-0.1/host-rust/src/plug_pack.rs`
- `tethers-0.1/host-rust/src/trail_command.rs`
- `scripts/run-rust-tests.sh`
- `verification/current-engine-provenance.json`
- `docs/worker-notes/2026-09-17-tethers-l6-linux-path-semantics.md`

## Frozen decisions and invariants

- The repair is test-fixture-only unless independent evidence proves a
  production defect.
- Native temporary paths must be absolute and unique on every supported host.
- No Windows-specific tests are weakened, disabled, or cfg-gated.
- The L4 verified engine and fail-closed provenance contract remain unchanged.
- Provider fixtures remain at zero failures.
- `c2a3a_group_join_after_all_terminals` remains a concurrency issue and is
  not repaired here.
- No WSL, global Git, toolchain, dependency, or environment changes are
  authorised.

## Acceptance criteria

1. Both named path tests pass on Linux with their original semantic assertions
   intact.
2. Their containing test modules pass and the full raw/verified measurements
   show exactly the expected two-failure reduction.
3. Provider tests remain passing and the concurrency failure remains present
   and classified as out of scope.
4. Formatting, compilation, provenance, `bl verify tethers`, `bl doctor
   tethers`, and `git diff --check` are recorded honestly.
5. The worker note records the root causes, Threadmoth discovery, taxonomy,
   unchanged production/Windows semantics, and remaining concurrency case.
6. The final branch is clean, committed, pushed normally, and unmerged.

## Required verification

- targeted `p2a_refuses_wrong_extension`
- targeted `j13c_missing_trail_file_maps_to_not_found`
- containing plug-pack and trail-command modules
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

- No repair of `c2a3a_group_join_after_all_terminals` or the 41 concurrency
  failures.
- No repair of the 72 expanded-target platform/provider failures.
- No production path redesign, assertion weakening, ignored tests, cfg-out,
  dependency additions, or fake platform paths.
- No changes to Windows files, WSL configuration, global Git configuration,
  toolchains, L4 provenance scripts, or main.

## Stop conditions

- Stop if either test requires a production semantic change rather than a
  fixture correction.
- Stop if a path fixture cannot express the same intended scenario on Linux
  and Windows without platform fiction.
- Stop if provenance becomes stale or fail-open.
- After two materially similar failed approaches, stop and record the smallest
  unresolved issue.

## Expected pre-existing changes

None.

## Implementation scope

- `tethers-0.1/host-rust/src/plug_pack.rs`
- `tethers-0.1/host-rust/src/trail_command.rs`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-09-17-tethers-l6-linux-path-semantics.md`
