# Current Implementation Task

Control contract: `1`

Task: `J14A-R evidence closeout`

Owner: `OpenCode`

Status: `COMPLETE`

Task colour: `Green`

Route: `OpenCode documentation closeout — Lucy independent review`

Base commit: `0c64b48d860ce2178858c4c5d8a0af38708bc7cc`

Original J14A implementation commit: `e86471ed8d160d47ba2ca70a6acbfabaf552f6ac`

J14A-R repair implementation commit: `e4b773050f0ceb7e7bef8b236ec199692b827754`

OpenCode instruction-bootstrap checkpoint: `74234dc7bede34ee4ff01adc5110e705c78ad7d3`

Branch: `opencode/j14a-closeout-bootstrap`

Worker note: `docs/worker-notes/2026-07-30-j14a-complete-local-scenario.md`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

## Objective

Complete the remaining J14A-R evidence closeout after the accepted engineering
repair and the user-authorised OpenCode instruction bootstrap.

Remove two stale `Implementation checkpoint: WORKTREE` placeholders from the
J14A worker note and replace them with the exact original and repair
implementation commits. Make no code, test, scenario, configuration, control,
or product-semantics change.

## Relevant background and existing behaviour

The J14A implementation and J14A-R engineering repair are already complete and
pushed. Independent review accepted the typed execution-evidence design and the
repaired committed-scenario harness. Only the worker-note checkpoint labels
remain inaccurate.

The branch also contains a user-authorised OpenCode instruction bootstrap:

- `opencode.json` loads the core control documents where supported;
- `AGENTS.md` requires a visible startup report and explicit reading fallback;
- `docs/PROJECT_CONTROL.md` and `docs/AGENT_WORKFLOW.md` name OpenCode as the
  active ordinary implementation owner.

Those bootstrap files are accepted pre-existing changes and are not part of this
closeout task.

## Required behaviour

1. Complete the mandatory startup report in `AGENTS.md` before editing.
2. Confirm the exact branch, `HEAD`, remote branch, `origin/main`, worktree root,
   and clean status.
3. Set this packet to `IN_PROGRESS` before changing the worker note.
4. Remove both occurrences of `Implementation checkpoint: WORKTREE` from the
   worker note.
5. Record exactly once:

   - `Original J14A implementation commit: e86471ed8d160d47ba2ca70a6acbfabaf552f6ac`
   - `J14A-R repair implementation commit: e4b773050f0ceb7e7bef8b236ec199692b827754`

6. Keep the statement that the final pushed evidence-closeout SHA belongs in the
   external completion report rather than inside the note.
7. Set this packet to `COMPLETE` only after every required check passes.
8. Commit and push only the two authorised documentation files.

## Relevant components

- `docs/CURRENT_CLINE_TASK.md` — this task packet and status.
- `docs/worker-notes/2026-07-30-j14a-complete-local-scenario.md` — the only
  evidence content to correct.

## Expected pre-existing changes

The branch is intentionally ahead of `origin/main` with:

- the complete J14A implementation;
- the J14A-R engineering repair;
- the OpenCode instruction bootstrap in `opencode.json`, `AGENTS.md`,
  `docs/PROJECT_CONTROL.md`, and `docs/AGENT_WORKFLOW.md`.

Preserve every pre-existing path and byte outside the two authorised files.

## Frozen decisions and invariants

- J14A-R production code and tests are accepted and must not change.
- The trusted execution ID remains typed evidence, never planner-response JSON.
- The committed J14A scenario remains unchanged.
- Historical filenames such as `CURRENT_CLINE_TASK.md` remain stable interfaces.
- OpenCode is the current ordinary Green and Amber implementation owner.
- The final closeout commit must not be inserted into its own worker note.

## Acceptance criteria

1. The `AGENTS.md` startup report is returned before mutation.
2. Both stale `WORKTREE` checkpoint placeholders are absent.
3. The original and repair implementation SHAs appear exactly once each.
4. No future, invented, or self-referential final SHA appears in the note.
5. Only this packet and the J14A worker note change during the task.
6. The task-packet checker passes with status `COMPLETE`.
7. `git diff --check` passes with no output.
8. The full branch remains based on `origin/main` with zero commits behind.
9. The final OpenCode worktree is completely clean, including untracked files.
10. The original worktree remains untouched on
    `cline/j10-result-event-queue` with only
    `M docs/TETHERS_LUCY_NOTES.md`.

## Forbidden changes

Do not modify code, tests, scenarios, `opencode.json`, `AGENTS.md`, control docs,
DECISIONS.md, manifests, fixtures, toolchains, Cargo files, OCaml, or Git history.

Do not amend, rebase, squash, reset, force-push, merge, publish `main`, delete a
branch, or begin J14B.

## Stop conditions

Return `BLOCKED` when:

- the startup report is incomplete;
- the worktree root or branch differs;
- local and remote branch history differ unexpectedly;
- `origin/main` differs from
  `0c64b48d860ce2178858c4c5d8a0af38708bc7cc`;
- the worktree is dirty before work;
- any path other than the two authorised files would need to change;
- the task-packet checker fails for a reason outside the authorised files;
- two materially similar attempts fail.

## Required verification

Run:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass `
  -File .\.github\scripts\check-tethers-task-packet.ps1

git diff --check
git diff --stat
git diff
git status --porcelain=v1 --untracked-files=all
```

Before staging, require only the two authorised files to be modified.

Stage only:

```powershell
git add -- `
  docs/CURRENT_CLINE_TASK.md `
  docs/worker-notes/2026-07-30-j14a-complete-local-scenario.md
```

Inspect the complete staged diff, then commit:

```powershell
git commit -m "docs: finalise j14a repair evidence"
```

Push only:

```powershell
git push -u origin opencode/j14a-closeout-bootstrap
```

After pushing, require local and remote branch SHAs to match and final status to
be completely empty.
