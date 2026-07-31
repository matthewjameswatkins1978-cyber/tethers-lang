# Current Implementation Task

Control contract: `1`

Task: `J15A - consolidated verification runner foundation`

Owner: `OpenCode`

Recommended model: `Hy3 High`

Status: `COMPLETE`

Task colour: `Green`

Route: `OpenCode implementation - Lucy independent review`

Base commit: `12b4ed70a77de59d7e24285637a4877151308cc1`

Branch: `opencode/j15-consolidated-verification`

Worker note: `docs/worker-notes/2026-07-31-j15a-verification-runner-foundation.md`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

## Objective

Create the first real J15 consolidated verification entry point and connect only
the three accepted J13 public suites (`check`, `run`, `trail`). This task lays the
runner foundation; the J14 suites are deliberately deferred to a later J15 task.

## Relevant background and existing behaviour

J13 delivered the public `check`, `run`, and `trail` commands and three accepted
standalone PowerShell suites:

- `tethers-0.1/scripts/test-j13a-check.ps1`
- `tethers-0.1/scripts/test-j13b-run.ps4` (actually `test-j13b-run.ps1`)
- `tethers-0.1/scripts/test-j13c-trail.ps1`

Each suite already fails fast and prints its own PASS/FAIL summary. There is
currently no single discoverable runner that wires them together. J15 is the
later consolidated release verification entry point; J15A establishes the runner
and connects the J13 suites only.

## Required behaviour

1. Add `tethers-0.1/scripts/verify-0.2.ps1` with parameters `-Suite <ID[]>` and
   `-List`, mapping J13A -> `test-j13a-check.ps1`, J13B -> `test-j13b-run.ps1`,
   and J13C -> `test-j13c-trail.ps1`, with default order J13A, J13B, J13C.
2. `-List` prints exactly one line per suite in default order
   (`J13A test-j13a-check.ps1`, `J13B test-j13b-run.ps1`, `J13C test-j13c-trail.ps1`),
   launches no suite, and exits 0.
3. `-Suite` rejects unknown and duplicate suite ids before launching any suite
   and exits 2 with a clear error for invalid selection.
4. Each selected child script runs in a separate `pwsh.exe` process using
   `-NoProfile -ExecutionPolicy Bypass -File <script>`, resolved relative to
   `verify-0.2.ps1`, never the caller's current directory.
5. Forward every non-empty child output line prefixed `<ID> | <child line>` and
   print `SUITE <ID> START <filename>` before and
   `SUITE <ID> PASS exit=0` or `SUITE <ID> FAIL exit=<code>` after each suite;
   continue to later suites when one fails; treat a missing child script as a
   suite failure and continue.
6. Print the final consolidated block
   (`J15 CONSOLIDATED VERIFICATION`, `TOTAL: <n> suites, <p> passed, <f> failed`,
   `RESULT: PASS|FAIL`) and exit 0 only when every selected suite passes, 1 when
   one or more fail, and 2 only for invalid runner usage or selection.
7. Replace the completed J14C packet with this focused J15A packet.

## Relevant components

Reuse patterns from the three accepted J13 suites:

- `tethers-0.1/scripts/test-j13a-check.ps1`
- `tethers-0.1/scripts/test-j13b-run.ps1`
- `tethers-0.1/scripts/test-j13c-trail.ps1`

## Frozen decisions and invariants

- Base commit is exactly `12b4ed70a77de59d7e24285637a4877151308cc1`.
- The runner does not build Rust or OCaml and does not modify the three child
  scripts.
- The runner does not parse, rewrite, suppress, or invent child case results; it
  only forwards and tallies.
- J14 suites are not connected in this task; only J13A, J13B, J13C are wired.
- No prerequisite detection, automatic repair, retries, parallelism, JSON output,
  logging files, configuration files, or CI changes are added.

## Acceptance criteria

1. The public packet checker passes for control-v1 consistency with this packet.
2. `-List` exits 0 with exactly the three ordered lines and no `SUITE` START line.
3. An unknown suite id exits 2 with no child process launched.
4. A duplicate suite id exits 2 with no child process launched.
5. Running only J13C launches exactly that suite, forwards its output, and reports
   `TOTAL: 1 suites, 1 passed, 0 failed` and `RESULT: PASS` with exit 0.
6. A PowerShell syntax parse of `verify-0.2.ps1` succeeds and only the three
   authorised paths change.
7. This packet accurately describes the task, the three authorised paths, the
   branch and base SHA, focused verification, stop conditions, and defers the J14
   suites.

## Required verification

Do not run the full release suite. Run only:

1. PowerShell syntax parse of `verify-0.2.ps1`.
2. List mode: exit 0, exact three lines, exact order, no suite START lines.
3. Invalid suite: an unknown id, exit 2, no child suite launched.
4. Duplicate suite: `J13C` twice, exit 2, no child suite launched.
5. Focused real suite: run only J13C; require J13C's actual 19-case script to
   pass, one J13C START line, one J13C PASS line, final `TOTAL: 1 suites, 1
   passed, 0 failed`, and `RESULT: PASS`.

Do not run J13A or J13B in this task.

## Forbidden changes

Do not modify:

- production Rust or Rust tests;
- OCaml;
- the three child scripts `test-j13a-check.ps1`, `test-j13b-run.ps1`,
  `test-j13c-trail.ps1`;
- any existing manifest, scenario, or fixture;
- Cargo files or `Cargo.lock`;
- public CLI, runtime schema, scope model, Trail schema, replay format, Result
  Anchor schema, language grammar, or protocol version;
- J14 suites or J15B+ work;
- AGENTS.md or workflow/control documents other than this packet.

## Stop conditions

Return `BLOCKED` when:

- any pre-flight ref or worktree differs;
- any unauthorised path changes;
- the runner cannot forward child output or tally results as specified;
- J13C cannot be run because its prerequisites are missing;
- production Rust, Rust tests, OCaml, schema, grammar, or existing fixtures need
  modification;
- two materially similar attempts fail.

## Expected pre-existing changes

None.

The worktree must be completely clean before mutation, descended from
`12b4ed70a77de59d7e24285637a4877151308cc1` with zero commits ahead or behind
`origin/main`.

## Commit and publication boundary

Create one implementation commit:

`feat: add j15 consolidated verification foundation`

Push only:

`opencode/j15-consolidated-verification`

Do not push main. Do not delete branches or worktrees. Do not begin J15B.

## Return contract

Return `COMPLETE` or `BLOCKED` and stop.

For `COMPLETE`, report branch, commit SHA, changed paths, exact `-List` output,
invalid selection exit, duplicate selection exit, J13C result, packet checker
result, branch ahead/behind, and worktree cleanliness.

Stop after reporting. Do not begin J15B.
