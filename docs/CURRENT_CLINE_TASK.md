# Current Implementation Task

Control contract: `1`

Task: `J15B - add the three accepted J14 suites to the consolidated verification runner`

Owner: `OpenCode`

Recommended model: `Hy3 High`

Status: `IN_PROGRESS`

Task colour: `Green`

Route: `OpenCode implementation - Lucy independent review`

Base commit: `12b4ed70a77de59d7e24285637a4877151308cc1`

Branch: `opencode/j15-consolidated-verification`

Worker note: `docs/worker-notes/2026-07-31-j15b-add-j14-suites.md`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

## Objective

Extend the J15 consolidated verification runner (`verify-0.2.ps1`) created in
J15A so it also wires the three accepted J14 public suites. The J15A behaviour
is preserved exactly; this task only adds the J14 entries and keeps the canonical
six-suite order.

## Relevant background and existing behaviour

J15A delivered `tethers-0.1/scripts/verify-0.2.ps1` connecting the three J13
suites (J13A/J13B/J13C) with `-List` and `-Suite <ID[]>`, unknown/duplicate
rejection before launch, separate `pwsh.exe` child processes, relative child
paths, prefixed output forwarding, per-suite START/PASS/FAIL lines, and an honest
consolidated tally. The three J14 suites were deferred in J15A.

The three accepted J14 child scripts already exist:

- `tethers-0.1/scripts/test-j14a-complete-scenario.ps1`
- `tethers-0.1/scripts/test-j14b-negative-matrix.ps1`
- `tethers-0.1/scripts/test-j14c-real-file-move.ps1`

## Required behaviour

1. Extend `verify-0.2.ps1` with mappings J14A -> `test-j14a-complete-scenario.ps1`,
   J14B -> `test-j14b-negative-matrix.ps1`, J14C -> `test-j14c-real-file-move.ps1`.
2. The complete canonical default order becomes J13A, J13B, J13C, J14A, J14B, J14C.
3. `-List` prints exactly one line per suite in that order and exits 0 with no
   `SUITE` START line.
4. Default execution (no `-Suite`) runs all six suites in canonical order.
5. Preserve every accepted J15A behaviour: unknown ids rejected before launch
   (exit 2), duplicate ids rejected before launch (exit 2), invalid usage exits
   2, separate child process, child paths resolved relative to the runner, child
   output forwarded without parsing, suite START/PASS/FAIL lines, later suites
   continue after a failed suite, final totals remain honest, exit 0 for all pass,
   1 for one or more suite failures, 2 for invalid runner use.

## Relevant components

Reuse the existing runner and the six accepted child scripts. Do not modify the
child scripts.

## Frozen decisions and invariants

- Base commit is exactly `12b4ed70a77de59d7e24285637a4877151308cc1`.
- The runner does not build Rust or OCaml and does not modify any child script.
- The runner does not parse, rewrite, suppress, or invent child case results; it
  only forwards and tallies.
- No prerequisite detection, automatic repair, retries, parallelism, JSON output,
  logging files, configuration files, or CI changes are added.
- J14B is mapped but is deliberately not executed through the runner in this task.

## Acceptance criteria

1. The public packet checker passes for control-v1 consistency with this packet.
2. `-List` exits 0 with exactly six ordered lines and no `SUITE` START line.
3. An unknown suite id exits 2 with no child process launched.
4. A duplicate suite id (`J14A` twice) exits 2 with no child process launched.
5. Running `-Suite J14A J14C` launches exactly those two suites in the supplied
   order, prints one START and one PASS line for each, forwards each suite's
   output with the correct prefix, reports `TOTAL: 2 suites, 2 passed, 0 failed`,
   `RESULT: PASS`, and exits 0.
6. A PowerShell syntax parse of `verify-0.2.ps1` succeeds and only the three
   authorised paths change.
7. This packet accurately describes the task, the three authorised paths, the
   branch and base SHA, the six-suite canonical order, focused verification, stop
   conditions, and defers J14B execution and J15C.

## Required verification

Do not run the full default six-suite command. Run only:

1. PowerShell syntax parse of `verify-0.2.ps1`.
2. List mode: exit 0, exactly six lines, exact canonical order, no START lines.
3. Invalid suite: an unknown id, exit 2, no child suite launched.
4. Duplicate suite: `J14A` twice, exit 2, no child suite launched.
5. Multi-suite real execution: `-Suite J14A J14C`; require both suites to run in
   supplied order, one START and one PASS line each, correct output prefixes,
   final `TOTAL: 2 suites, 2 passed, 0 failed`, `RESULT: PASS`, exit 0.

Do not run J13A, J13B, J13C, or J14B in this task.

## Forbidden changes

Do not modify:

- production Rust or Rust tests;
- OCaml;
- the six child scripts `test-j13a-check.ps1`, `test-j13b-run.ps1`,
  `test-j13c-trail.ps1`, `test-j14a-complete-scenario.ps1`,
  `test-j14b-negative-matrix.ps1`, `test-j14c-real-file-move.ps1`;
- any existing manifest, scenario, or fixture;
- Cargo files or `Cargo.lock`;
- public CLI, runtime schema, scope model, Trail schema, replay format, Result
  Anchor schema, language grammar, or protocol version;
- J15C work;
- AGENTS.md or workflow/control documents other than this packet.

## Stop conditions

Return `BLOCKED` when:

- any pre-flight ref or worktree differs;
- any unauthorised path changes;
- the runner cannot forward child output or tally results as specified;
- J14A or J14C cannot be run because prerequisites are missing;
- production Rust, Rust tests, OCaml, schema, grammar, or existing fixtures need
  modification;
- two materially similar attempts fail.

## Expected pre-existing changes

J15A is already committed on this branch (`feat: add j15 consolidated verification
foundation`). The worktree must be clean before this task's mutation, descended
from `12b4ed70a77de59d7e24285637a4877151308cc1` and 0 behind `origin/main`.

## Commit and publication boundary

Create one implementation commit:

`feat: add j14 suites to consolidated verification`

Push only:

`opencode/j15-consolidated-verification`

Do not push main. Do not delete branches or worktrees. Do not begin J15C.

## Return contract

Return `COMPLETE` or `BLOCKED` and stop.

For `COMPLETE`, report branch, commit SHA, changed paths, exact six-line `-List`
output, invalid selection exit, duplicate selection exit, J14A result, J14C
result, combined final summary, packet checker result, branch ahead/behind, and
worktree cleanliness.

Stop after reporting. Do not begin J15C.
