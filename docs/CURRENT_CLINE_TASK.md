# Current Implementation Task

Control contract: `1`

Task: `J15C - prove the consolidated runner contract`

Owner: `OpenCode`

Recommended model: `Hy3 High`

Status: `COMPLETE`

Task colour: `Green`

Route: `OpenCode implementation - Lucy independent review`

Base commit: `12b4ed70a77de59d7e24285637a4877151308cc1`

Branch: `opencode/j15-consolidated-verification`

Worker note: `docs/worker-notes/2026-07-31-j15c-runner-contract.md`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

## Objective

Add a self-contained contract test for the J15 consolidated runner
(`verify-0.2.ps1`) that proves its behaviour without invoking the real J13 or
J14 child suites. Then run exactly the focused `J14B` suite through the real
runner.

## Relevant background and existing behaviour

J15A delivered `verify-0.2.ps1` (list mode, `-Suite` selection, unknown/duplicate
rejection before launch, separate child `pwsh.exe` processes, relative child
paths resolved from `$PSScriptRoot`, prefixed output forwarding as `<ID> | `,
per-suite `START`/`PASS`/`FAIL` lines, honest consolidated tally, exit 0/1/2).
J15B wired the three J14 suites so the canonical default order is J13A, J13B,
J13C, J14A, J14B, J14C.

The runner was not modified in this task: the contract test exposed no defect.

## Required behaviour

1. Add `tethers-0.1/scripts/test-verify-0.2.ps1` that tests the runner itself and
   never invokes the real J13 or J14 child scripts.
2. The test creates one temporary directory containing both a space and a
   non-ASCII character, copies `verify-0.2.ps1` into it, creates six canonical
   stub children, and runs the copied runner from a different caller working
   directory.
3. The test reports exactly six rows R01 through R06: list mode; unknown suite
   exits 2 and launches no child; duplicate suite exits 2 and launches no child;
   selected suites preserve order and prefix output; a failing middle suite does
   not prevent the later suite; a missing child is reported as failure and later
   suites still run.
4. R04 uses three passing stubs and proves supplied order, correct `<ID> |`
   prefixing, exactly one `START`/`PASS` per selected suite, `TOTAL: 3 suites, 3
   passed, 0 failed`, `RESULT: PASS`, exit 0.
5. R05 uses pass/fail/pass and proves all three launch, the final stub launches
   after the middle failure, the actual failing exit code is reported, `TOTAL: 3
   suites, 2 passed, 1 failed`, `RESULT: FAIL`, exit 1. R06 omits one selected
   child and proves one `FAIL`, no PowerShell stack trace, the later stub still
   launches, an honest total, exit 1.
6. After the contract test passes, run exactly `verify-0.2.ps1 -Suite J14B`,
   requiring one `J14B START`, 11 passing rows, one `J14B PASS`, consolidated
   `TOTAL: 1 suites, 1 passed, 0 failed`, `RESULT: PASS`, exit 0.

## Relevant components

- `tethers-0.1/scripts/verify-0.2.ps1` (runner under test; not modified).
- New: `tethers-0.1/scripts/test-verify-0.2.ps1`.
- Real J14B child: `tethers-0.1/scripts/test-j14b-negative-matrix.ps1`.

## Frozen decisions and invariants

- Base commit is exactly `12b4ed70a77de59d7e24285637a4877151308cc1`.
- The runner is not modified; the contract test proved the existing contract.
- The contract test creates its own stub children and never runs the real J13 or
  J14 suites.
- No prerequisite detection, automatic repair, retries, parallelism, JSON output,
  logging files, configuration files, or CI changes are added.
- The full default six-suite run remains deferred to J15D.

## Acceptance criteria

1. The public packet checker passes for control-v1 consistency with this packet.
2. The contract test prints six rows R01-R06, all `PASS`, and a final
   `TOTAL: 6 rows, 6 passed, 0 failed` with a non-zero assertion count.
3. R01 prints the exact six-suite canonical list, exit 0, and no `SUITE` START
   line; R02 and R03 exit 2 and launch no child (proven by marker files).
4. R04 proves order preservation, correct `<ID> | ` prefixing, exactly one
   `START`/`PASS` per selected suite, `TOTAL: 3 suites, 3 passed, 0 failed`,
   `RESULT: PASS`, exit 0.
5. R05 proves pass/fail/pass continues, reports the actual failing exit code,
   `TOTAL: 3 suites, 2 passed, 1 failed`, `RESULT: FAIL`, exit 1; R06 proves a
   missing child yields one `FAIL`, no PowerShell stack trace is exposed, the
   later selected stub still launches, an honest total, exit 1.
6. `verify-0.2.ps1 -Suite J14B` runs 11 rows (all pass), one `J14B PASS`,
   consolidated `TOTAL: 1 suites, 1 passed, 0 failed`, `RESULT: PASS`, exit 0.

## Required verification

Do not run the default six-suite command. Do not run J13A, J13B, J13C, J14A, or
J14C. Run exactly:

1. `pwsh -NoProfile -ExecutionPolicy Bypass -File tethers-0.1/scripts/test-verify-0.2.ps1`
   -> six rows PASS, final `TOTAL: 6 rows, 6 passed, 0 failed`, exit 0.
2. `pwsh -NoProfile -ExecutionPolicy Bypass -File tethers-0.1/scripts/verify-0.2.ps1 -Suite J14B`
   -> 11 rows pass, consolidated `TOTAL: 1 suites, 1 passed, 0 failed`,
   `RESULT: PASS`, exit 0.

## Forbidden changes

Do not modify:

- production Rust or Rust tests;
- OCaml;
- `tethers-0.1/scripts/verify-0.2.ps1` (the runner) unless a contract test
  exposes a real defect;
- the six child scripts `test-j13a-check.ps1`, `test-j13b-run.ps1`,
  `test-j13c-trail.ps1`, `test-j14a-complete-scenario.ps1`,
  `test-j14b-negative-matrix.ps1`, `test-j14c-real-file-move.ps1`;
- any existing manifest, scenario, or fixture;
- Cargo files or `Cargo.lock`;
- public CLI, runtime schema, scope model, Trail schema, replay format, Result
  Anchor schema, language grammar, or protocol version;
- J15D work;
- AGENTS.md or workflow/control documents other than this packet.

## Stop conditions

Return `BLOCKED` when:

- any pre-flight ref or worktree differs;
- any unauthorised path changes;
- the contract test cannot be isolated from the real child suites;
- the runner must be modified to pass (that is a real defect and must be
  reported, not silently patched for style);
- J14B cannot be run because prerequisites are missing;
- two materially similar attempts fail.

## Expected pre-existing changes

At start of this task the branch `opencode/j15-consolidated-verification` was at
`f3f9e4f` (J15A + J15B committed), 3 ahead of and 0 behind `origin/main`, with a
clean worktree. J15D (full default six-suite release run) is deferred.

## Commit and publication boundary

Create exactly one commit:

`test: prove j15 consolidated runner contract`

The packet must already say `COMPLETE` before this commit. Push only:

`opencode/j15-consolidated-verification`

Do not push main. Do not begin J15D.

## Return contract

Return `COMPLETE` or `BLOCKED` and stop.

For `COMPLETE`, report commit SHA, changed paths, R01-R06 result, assertion
count, J14B row result, consolidated J14B summary, packet checker result, branch
ahead/behind, and worktree cleanliness.

Stop after reporting. Do not begin J15D.
