# TETHERS L5 - Linux Failure Taxonomy and Repair

Task: `TETHERS L5 / Linux Failure Taxonomy and Repair`

Control contract: `1`

Status: `COMPLETE`

Task colour: `Amber`

Owner: `Codex`

Route: `Implement the bounded Linux provider-fixture repair from the accepted L4 branch, classify the three unresolved Linux cases, verify provenance remains fail-closed, document the evidence, commit and push the task branch. Do not merge or repair out-of-scope concurrency and expanded-target failures.`

Base commit: `ad5a728e753e1be77c4a4e1be40c34be3f827578`

Evidence checkpoint: `41db56f8700f59ef919c2a6ca528504920085b75`

Worker note: `docs/worker-notes/2026-09-17-tethers-l5-linux-failure-taxonomy.md`

Suggested branch:

`feature/tethers-l5-linux-failure-taxonomy`

## Objective

Make the remaining Linux test failures tell the truth. Repair the shared
Windows-only provider fixture assumption for native Linux, investigate the
three explicitly named unresolved cases without expanding into unrelated
concurrency or platform cleanup, and leave reproducible evidence for the
remaining taxonomy.

## Relevant background and existing behaviour

L4 established a native Linux OCaml engine build and provenance manifest. The
verified engine is selected only after source-commit and binary-hash checks.
The comparable Linux provider-fixture family contains 18 failures caused by
Windows PowerShell fixture construction. The remaining unresolved cases are
`c2a3a_group_join_after_all_terminals`, `p2a_refuses_wrong_extension`, and
`j13c_missing_trail_file_maps_to_not_found`. L4 also records 41 concurrency
failures and 72 expanded-target platform/provider failures that are not this
task's repair target.

## Required behaviour

1. Provide a native Linux provider fixture path without changing the existing
   Windows PowerShell fixture or production provider semantics.
2. Repair the shared in-scope fixture causes and add focused evidence that the
   affected provider tests exercise the intended protocol and outcomes.
3. Re-run and classify the three named unresolved Linux tests without silently
   repairing or hiding them in this task.
4. Preserve L4 engine provenance and fail-closed verification, and document
   before/after Linux totals and every remaining failure family.
5. Commit and normally push the bounded task branch without merging it.

## Relevant components

- `tethers-0.1/scripts/tethers-stdio-fixture.ps1`
- `tethers-0.1/scripts/tethers-stdio-fixture.sh`
- `tethers-0.1/host-rust/src/stdio_provider.rs`
- `tethers-0.1/host-rust/src/host_execution.rs`
- `scripts/prepare-current-engine.sh`
- `scripts/run-rust-tests.sh`
- `verification/current-engine-provenance.json`
- `docs/worker-notes/2026-09-17-tethers-l5-linux-failure-taxonomy.md`

## Frozen decisions and invariants

- Tethers production semantics, provider execution ordering, replay, outcome
  taxonomy, and the Windows verified-engine path remain unchanged.
- Linux must not fabricate Windows environment variables or invoke PowerShell
  as a substitute for a native fixture.
- Platform boundaries are semantic; no test may be hidden merely because it
  fails on Linux.
- L4 provenance must continue to reject missing, stale, tampered, or absent
  engine evidence and accept only the current verified engine.
- The three named unresolved tests, the 41 concurrency failures, and the 72
  expanded-target failures remain outside the repair scope unless direct
  evidence proves a shared in-scope root cause.
- No WSL, global Git, toolchain, Windows checkout, or environment repair is
  authorised.

## Acceptance criteria

1. A native Linux provider fixture runs the affected protocol tests and the
   existing Windows fixture remains unchanged.
2. The comparable provider-fixture family is structurally classified and its
   valid shared root cause is repaired without weakened assertions.
3. The three named unresolved tests are individually rerun and their results
   are recorded with exact first failures.
4. Raw and verified Linux totals, taxonomy deltas, provenance checks, and
   remaining failures are recorded in the worker note.
5. Formatting, compilation, relevant tests, `bl verify tethers`, and
   `git diff --check` pass, subject only to explicitly recorded unrelated
   tooling limitations.
6. The final branch is clean, committed, pushed normally, and remains
   unmerged.

## Required verification

- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo test --workspace --no-run --locked`
- focused provider and unresolved-test commands
- verified/default and verified/single-thread Linux test runners
- `bl verify tethers`
- L4 provenance valid/stale/tampered/missing checks
- `git diff --check`
- final clean Git status and remote SHA equality

## Forbidden changes

- No repair of the concurrency family unless it is proven to share the
  provider-fixture root cause.
- No general repair of expanded-target platform/provider failures.
- No production redesign, provider architecture rewrite, test disabling,
  assertion weakening, random dependency installation, or fake Windows
  environment.
- No changes to the Windows checkout, WSL configuration, global Git
  configuration, toolchain versions, L4 provenance contract, or main.

## Stop conditions

- Stop if the fix would require changing intended Windows behaviour or
  weakening a semantic assertion.
- Stop if a remaining failure cannot be classified without expanding scope.
- Stop if L4 provenance no longer fails closed or the current engine cannot be
  attributed to the current source.
- After two materially similar failed approaches to the same root cause, stop
  and record the smallest unresolved technical issue.

## Expected pre-existing changes

None.

## Implementation scope

- `tethers-0.1/scripts/tethers-stdio-fixture.sh`
- `tethers-0.1/host-rust/src/stdio_provider.rs`
- `tethers-0.1/host-rust/src/host_execution.rs`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-09-17-tethers-l5-linux-failure-taxonomy.md`
