# Worker Note

Task: `J15C - prove the consolidated runner contract`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `12b4ed70a77de59d7e24285637a4877151308cc1`

Implementation checkpoint: `f3f9e4f642943599b3e601e4c063d809c9408eb5`

## Requested outcome

Add a self-contained contract test that proves the behaviour of the J15
consolidated runner (`verify-0.2.ps1`) without invoking the real J13 or J14
child suites, then run exactly the focused `J14B` suite through the real runner.

## Changes made

- `tethers-0.1/scripts/test-verify-0.2.ps1` — new contract test (see below).
- `docs/CURRENT_CLINE_TASK.md` — replaced the J15B packet with the focused J15C
  packet, status `COMPLETE`.
- `docs/worker-notes/2026-07-31-j15c-runner-contract.md` — this note.

The runner `verify-0.2.ps1` was not modified: the contract test exposed no real
defect. No other file changed.

## Decisions and assumptions

- The test copies the real runner into a temp directory whose name contains a
  space and a non-ASCII character, then runs it from a different caller working
  directory (`$env:TEMP`). The runner resolves children through `$PSScriptRoot`,
  so the copied runner uses the six stub children placed in that same temp
  directory and never touches the real J13/J14 scripts.
- Stub children write a marker file when they actually run. This gives direct
  evidence for "launches no child" (R02/R03) and "later stub still launches"
  (R06), satisfying the packet's marker-file proof requirement.
- Each stub forwards non-empty lines; the runner prefixes every forwarded line
  with `<ID> | `. R04 asserts the exact prefixed lines and the absence of any
  bare child line.
- For R05 the failing stub exits 3, proving the runner reports the actual child
  exit code (`SUITE <ID> FAIL exit=3`) rather than a generic failure.
- For R06 the selected child file is deleted before the run; the runner reports
  `SUITE J14A FAIL exit=-1`, never throws, and the later selected stub still
  runs. The test asserts no PowerShell stack-trace pattern appears in output.
- The test is fail-fast per row and uses `try/finally` cleanup; it throws if the
  temp root survives cleanup.

## Evidence

Contract test (49 assertions, exit 0):

- R01 list mode: prints exactly the six canonical lines
  `J13A test-j13a-check.ps1`, `J13B test-j13b-run.ps1`, `J13C
  test-j13c-trail.ps1`, `J14A test-j14a-complete-scenario.ps1`, `J14B
  test-j14b-negative-matrix.ps1`, `J14C test-j14c-real-file-move.ps1`; exit 0;
  no `SUITE` START line; zero child markers.
- R02 unknown suite (`-Suite ZZTOP`): exit 2; reports `unknown suite id`; no
  `SUITE` START line; zero child markers.
- R03 duplicate suite (`-Suite J13A J13A`): exit 2; reports `duplicate suite id`;
  no `SUITE` START line; zero child markers.
- R04 selected pass order (`-Suite J13A J14B J14C`): exit 0; `START` lines in
  supplied order; exactly one `START`/`PASS` per selected suite; every child line
  prefixed (`J13A | alpha ...`, `J14B | beta one`, `J14C | gamma ...`); no bare
  child line; final `TOTAL: 3 suites, 3 passed, 0 failed`, `RESULT: PASS`; three
  child markers.
- R05 pass/fail/pass (`-Suite J13A J13B J14C`): exit 1; all three `START` in
  order; `SUITE J13B FAIL exit=3` (actual failing exit code); the final stub
  `J14C START` occurs after the middle failure; final `TOTAL: 3 suites, 2 passed,
  1 failed`, `RESULT: FAIL`; three child markers.
- R06 missing child (`-Suite J14A J14C`, J14A file omitted): exit 1; `SUITE J14A
  FAIL exit=-1`; no `SUITE J14A PASS`; no PowerShell stack-trace pattern in
  output; `SUITE J14C START` and `SUITE J14C PASS exit=0` prove the later stub
  still launches; final `TOTAL: 2 suites, 1 passed, 1 failed`, `RESULT: FAIL`;
  exactly one child marker (the later stub).
- Final summary: `TOTAL: 6 rows, 6 passed, 0 failed`, `ASSERTIONS: 49`.

Focused real J14B run (`verify-0.2.ps1 -Suite J14B`, exit 0):

- One `J14B START test-j14b-negative-matrix.ps1`.
- 11 rows all pass (M01-M11), `J14B TOTAL: 11 rows, 11 passed, 0 failed`,
  `ASSERTIONS: 243`.
- One `J14B PASS exit=0`.
- Consolidated: `TOTAL: 1 suites, 1 passed, 0 failed`, `RESULT: PASS`, exit 0.

Not run in this task: the default six-suite command, and J13A/J13B/J13C/J14A/J14C.

Packet checker: `.github/scripts/check-tethers-task-packet.ps1` ->
`PASS task packet consistency (control-v1/COMPLETE): base 12b4ed7, HEAD ...`,
exit 0.

`git diff --check` (unstaged/committed) -> exit 0 (only informational LF/CRLF
normalization notices; no whitespace errors).

## Discoveries

- The contract test isolated the runner fully: it ran the copied runner against
  stub children only, so the real J13/J14 suites were never invoked.
- J14B self-reports 11 rows / 243 assertions and, for M11, rebuilds the debug
  host as part of the row; the runner forwarded that output correctly prefixed.

## Remaining risks

- The full default six-suite consolidated run is still deferred to J15D.
- No other risk known within packet scope.

## Smallest next action

J15D: perform the full default six-suite release run (J13A, J13B, J13C, J14A,
J14B, J14C) through `verify-0.2.ps1` with no `-Suite` argument. Do not start it
here.

## References

- `docs/CURRENT_CLINE_TASK.md` (J15C packet)
- `tethers-0.1/scripts/verify-0.2.ps1` (runner under test, unmodified)
- `tethers-0.1/scripts/test-verify-0.2.ps1` (new contract test)
- `tethers-0.1/scripts/test-j14b-negative-matrix.ps1` (real J14B child, 11 rows)
- branch `opencode/j15-consolidated-verification`, base `12b4ed70a77de59d7e24285637a4877151308cc1`, J15C start `f3f9e4f`
