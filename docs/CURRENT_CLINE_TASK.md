# Current Implementation Task

Control contract: `1`

Task: `J15D - run the complete consolidated 0.2 failure matrix`

Owner: `OpenCode`

Recommended model: `Hy3 High`

Status: `COMPLETE`

Task colour: `Green`

Route: `OpenCode implementation - Lucy independent review`

Base commit: `12b4ed70a77de59d7e24285637a4877151308cc1`

Branch: `opencode/j15-consolidated-verification`

Worker note: `docs/worker-notes/2026-07-31-j15d-full-matrix.md`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

## Objective

Run the complete consolidated J15 release matrix through `verify-0.2.ps1` with
no `-Suite` argument, proving the canonical six-suite order and the full
accepted child matrix. This is a verification-and-evidence job only.

## Relevant background and existing behaviour

J15A delivered the runner and J15B wired all six suites (canonical default order
J13A, J13B, J13C, J14A, J14B, J14C). J15C proved the runner contract with an
isolated stub test and ran the focused J14B suite. The runner is unchanged.

The complete matrix was not run until this task: the full default six-suite run
was explicitly deferred from J15A/J15B/J15C.

## Required behaviour

1. Run `tethers-0.1/scripts/verify-0.2.ps1` exactly once with no `-Suite`
   argument, capturing output to a temp file under `$env:TEMP` and measuring
   wall-clock time.
2. Require canonical execution order J13A, J13B, J13C, J14A, J14B, J14C, exactly
   one `START` and one `PASS` per suite, and no `FAIL` line.
3. Require the exact child results: J13A `25 passed, 0 failed`, J13B
   `10 passed, 0 failed`, J13C `19 cases, 19 passed, 0 failed`, J14A
   `5 cases, 5 passed, 0 failed` and `95 assertions`, J14B
   `11 rows, 11 passed, 0 failed` and `243 assertions`, J14C
   `9 rows, 9 passed, 0 failed` and `196 assertions`.
4. Require the final consolidated `TOTAL: 6 suites, 6 passed, 0 failed`,
   `RESULT: PASS`, and process exit 0.
5. Delete any external temporary capture after extracting evidence; do not create
   any helper or log inside the repository.
6. Only after a passing run, update this packet to COMPLETE and add the J15D
   worker note; nothing else may change.

## Relevant components

- `tethers-0.1/scripts/verify-0.2.ps1` (runner; not modified).
- Six child suites: `test-j13a-check.ps1`, `test-j13b-run.ps1`,
  `test-j13c-trail.ps1`, `test-j14a-complete-scenario.ps1`,
  `test-j14b-negative-matrix.ps1`, `test-j14c-real-file-move.ps1`.

## Frozen decisions and invariants

- Base commit is exactly `12b4ed70a77de59d7e24285637a4877151308cc1`.
- The runner and every child test script are unchanged; this is evidence only.
- The accepted child matrix represents 79 release cases or rows in total
  (25 + 10 + 19 + 5 + 11 + 9).
- No prerequisite detection, repair, retries, parallelism, JSON, logs, config, or
  CI changes are added.
- J15 implementation is complete pending Lucy's independent acceptance.
- Publication to main is not part of this task. J16 has not begun.

## Acceptance criteria

1. The public packet checker passes for control-v1 consistency with this packet.
2. The complete matrix runs once with no `-Suite`, canonical order
   J13A, J13B, J13C, J14A, J14B, J14C, exactly one `START` and one `PASS` per
   suite, and zero `FAIL` lines.
3. Child counts match exactly: J13A 25/0, J13B 10/0, J13C 19 cases/19 passed/0
   failed, J14A 5 cases/5 passed/0 failed/95 assertions, J14B 11 rows/11 passed/0
   failed/243 assertions, J14C 9 rows/9 passed/0 failed/196 assertions (79 total).
4. Final consolidated `TOTAL: 6 suites, 6 passed, 0 failed`, `RESULT: PASS`,
   process exit 0.
5. Wall-clock duration is recorded and the external temp capture is removed.
6. Only the two authorised paths change: `docs/CURRENT_CLINE_TASK.md` and
   `docs/worker-notes/2026-07-31-j15d-full-matrix.md`.

## Required verification

Run exactly once:

`pwsh -NoProfile -ExecutionPolicy Bypass -File tethers-0.1/scripts/verify-0.2.ps1`

-> canonical six-suite order, six `START`/`PASS`, no `FAIL`, child counts as
above, consolidated `TOTAL: 6 suites, 6 passed, 0 failed`, `RESULT: PASS`, exit 0.

## Forbidden changes

Do not modify:

- `tethers-0.1/scripts/verify-0.2.ps1` (the runner);
- the six child scripts;
- production Rust, Rust tests, or OCaml;
- any manifest, scenario, fixture, Cargo file, `Cargo.lock`;
- public CLI, runtime schema, scope model, Trail schema, replay format, Result
  Anchor schema, language grammar, or protocol version;
- AGENTS.md or workflow/control documents other than this packet;
- any path outside the two authorised files.

## Stop conditions

Return `BLOCKED` when:

- any pre-flight ref or worktree differs;
- any unauthorised path changes;
- any suite fails, any expected count differs, output order differs, or the
  runner exits non-zero (report the failing suite, actual summary, and exit code;
  do not repair);
- two materially similar attempts fail.

## Expected pre-existing changes

At start of this task the branch `opencode/j15-consolidated-verification` was at
`56ae7d7` (J15A + J15B + J15C committed), 4 ahead of and 0 behind `origin/main`,
with a clean worktree.

## Commit and publication boundary

Create exactly one commit:

`test: verify complete j15 release matrix`

The packet must already say `COMPLETE` before this commit. Push only:

`opencode/j15-consolidated-verification`

Do not push main. Do not create a second completion commit. Do not begin J16.

## Return contract

Return `COMPLETE` or `BLOCKED` and stop.

For `COMPLETE`, report commit SHA, wall-clock duration, six suite results, total
release cases or rows, J14 assertion counts, consolidated final summary, runner
exit code, packet checker result, changed paths, branch ahead/behind, and
worktree cleanliness.

Stop after reporting. Do not begin J16.
