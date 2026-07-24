# Worker Note Template

Copy this file to the exact path named by the current task packet under
`docs/worker-notes/`. Remove instructional text before handoff.

Task: `<short task name>`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `<one worker or agent>`

Status: `COMPLETE` or `BLOCKED`

Base commit: `<40-character implementation checkpoint>`

Implementation checkpoint: `<commit SHA, or WORKTREE when intentionally uncommitted>`

## Requested outcome

State what the packet required in two or three sentences.

## Changes made

List exact files and interfaces changed. Say `None` when blocked before edits.

## Decisions and assumptions

Record only judgement made within the permitted scope. Do not reopen frozen
architecture here.

## Evidence

List exact commands, tests, results, relevant output locations, and diff/status
checks. Distinguish run evidence from tests merely read or inferred.

## Discoveries

Record unexpected behaviour or project facts that could affect later work. Say
`None` when there were none.

## Remaining risks

Record unresolved, uncertain, fragile, or deliberately deferred matters. Say
`None known within packet scope` only when justified by the evidence.

## Smallest next action

Give one bounded next action. Do not start it.

## References

List exact files, commits, branches, issues, Trail files, fixtures, screenshots,
or other evidence locations.
