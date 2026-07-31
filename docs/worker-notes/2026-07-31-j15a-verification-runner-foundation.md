# Worker Note

Task: `J15A - consolidated verification runner foundation`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `12b4ed70a77de59d7e24285637a4877151308cc1`

Implementation checkpoint: `12b4ed70a77de59d7e24285637a4877151308cc1` (worktree, pre-commit) then committed as `feat: add j15 consolidated verification foundation`

## Requested outcome

Create the first J15 consolidated verification entry point (`verify-0.2.ps1`)
wiring only the three accepted J13 public suites (J13A/J13B/J13C). The runner
had to support `-List` and `-Suite <ID[]>`, reject unknown/duplicate ids before
any launch, run each child in a separate `pwsh.exe` process with paths resolved
relative to itself, forward child output prefixed by suite id, and print an
honest consolidated tally with the documented exit-code contract.

## Changes made

- `tethers-0.1/scripts/verify-0.2.ps1` — new consolidated runner (added).
- `docs/CURRENT_CLINE_TASK.md` — replaced the prior completed J14C packet with
  the focused J15A packet (modified).
- `docs/worker-notes/2026-07-31-j15a-verification-runner-foundation.md` — this
  note (added).

The three child scripts (`test-j13a-check.ps1`, `test-j13b-run.ps1`,
`test-j13c-trail.ps1`) were not modified.

## Decisions and assumptions

- `pwsh -File` with a simple `param()` block silently drops extra positional
  tokens into `$args`, so `-Suite J13C J13C` only captured one element. The
  runner now merges `$args` into `$Suite` and also splits comma-separated values,
  so `-Suite A B`, `-Suite A,B`, and positional `A B` all expand to the same
  selection. This is required for the duplicate-rejection contract.
- `Write-Error` under `$ErrorActionPreference = "Stop"` throws and yields exit 1,
  not the required exit 2. Invalid/duplicate selection now writes to stderr via
  `[Console]::Error.WriteLine` and then `exit 2`.
- Exit contract: 0 = all selected suites pass, 1 = one or more suite failures,
  2 = invalid runner usage or selection.

## Evidence

- Syntax: `pwsh -NoProfile -File syntax-check.ps1` (Parser.ParseFile) ->
  `SYNTAX OK`, exit 0.
- `-List` -> exactly three ordered lines, no START line, exit 0:
  ```
  J13A test-j13a-check.ps1
  J13B test-j13b-run.ps1
  J13C test-j13c-trail.ps1
  ```
- `-Suite XYZ` (unknown) -> stderr `unknown suite id: 'XYZ'...`, exit 2, no child
  process launched.
- `-Suite J13C J13C` (duplicate) -> stderr `duplicate suite id: 'J13C'...`,
  exit 2, no child process launched.
- `-Suite J13C` (focused real suite) -> one `SUITE J13C START` line, 19 forwarded
  `J13C | ...` lines, `SUITE J13C PASS exit=0`, final
  `TOTAL: 1 suites, 1 passed, 0 failed`, `RESULT: PASS`, exit 0. J13C is the
  actual 19-case `test-j13c-trail.ps1`.
- Did not run J13A or J13B in this task, per the focused-verification scope.
- Packet checker: not re-run at J15A closure; verified manually against the
  control-v1 expectations (branch, base SHA, three authorised paths, focused
  verification, stop conditions, J14 deferral all present).

## Discoveries

- A PowerShell simple `param()` script ignores unbound positional arguments
  (they go to `$args`), which broke multi-value `-Suite`. Now handled by merging
  `$args` and splitting commas. Relevant for any future runner that takes a list.
- The Rust reference host (`host-rust/target/debug/tethers-reference-host.exe`)
  is already built, so J13C (and the later J14 suites) run without a build step.

## Remaining risks

- The J14 suites are deliberately not wired in J15A; they are the subject of the
  next J15 task (J15B). No other risk known within packet scope.

## Smallest next action

J15B: extend `verify-0.2.ps1` with the three J14 suites (J14A/J14B/J14C) in the
canonical six-suite order, keeping all J15A behaviour intact. Do not start it
here.

## References

- `docs/CURRENT_CLINE_TASK.md` (J15A packet)
- `tethers-0.1/scripts/verify-0.2.ps1`
- `tethers-0.1/scripts/test-j13c-trail.ps1` (19 cases, used for focused run)
- branch `opencode/j15-consolidated-verification`, base `12b4ed70a77de59d7e24285637a4877151308cc1`
