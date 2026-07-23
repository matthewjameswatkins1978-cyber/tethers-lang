---
name: tethers-task
description: Run, review, or prepare the current bounded Tethers implementation task. Use when Matthew invokes /tethers-task.md, asks for the next Tethers job, pastes an approved handover, or wants Cline to continue from docs/CURRENT_CLINE_TASK.md.
---

# Tethers Task

This skill is the low-friction entry point for bounded Tethers work.

## First action

Read `docs/CURRENT_CLINE_TASK.md`.

Then inspect:

```powershell
git status --short --branch
git rev-parse HEAD
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
```

Do not edit anything until the task packet and live Git state agree.

## Task packet states

### `READY`

1. Run `.github/scripts/check-tethers-task-packet.ps1` and stop if it fails.
   `Base commit` may equal `HEAD`, or it may be an ancestor followed only by
   committed planning changes to `CURRENT_CLINE_TASK.md` and
   `COPILOT_TRIAL.md`. This avoids the impossible requirement for a committed
   packet to contain its own commit SHA.
2. Confirm the checker's live dirty-path result matches `Expected pre-existing
   changes`; planning-control documents being authored are excluded by design.
3. Read `AGENTS.md`, `.clinerules/`, and only the authoritative documents and
   code named by the packet.
4. Reinspect the relevant implementation before trusting the packet.
5. If Cline is in Plan mode, produce a plan of at most eight concrete steps and
   ask Matthew to toggle to Act mode. Do not repeat the task text.
6. In Act mode, implement only the packet.
7. Run every required check in the stated order.
8. Inspect the complete diff and final Git status.
9. Change the packet status to `COMPLETE` only if every acceptance criterion is
   met. Otherwise change it to `BLOCKED` and record the exact blocker.
10. Do not commit, push, merge, amend, tag, install software, or start the next
    task unless the packet explicitly authorises it.

### `PROPOSED`

Review the proposal against the repository. Do not implement it. Return one
short approval sentence and only the genuine unresolved decisions. If Matthew
approves it, update the packet to `READY`; implementation begins only after
that explicit approval.

### `COMPLETE`, `BLOCKED`, or missing

Do not silently invent a new task. Read `docs/CURRENT_GOAL.md` and
`docs/TASK_QUEUE.md`, inspect the relevant code, and write the next smallest
coherent task to `docs/CURRENT_CLINE_TASK.md` using status `PROPOSED`. Summarize
it in plain English and ask Matthew for one approval. Treat uncertain work as
the higher risk colour. Red work requires an explicit architectural decision
before its status can become `READY`.

## Pasted handovers

If Matthew invokes this skill with a pasted handover:

1. Reinspect the live repository.
2. Normalize the handover into the structure used by
   `docs/CURRENT_CLINE_TASK.md`.
3. Preserve explicit decisions, exclusions, tests, and stop conditions.
4. Do not convert an unresolved Red decision into implementation work.
5. Ask only for the single approval genuinely required, then save the approved
   packet as `READY`.

## Reporting

Keep the final report short:

- outcome;
- files changed;
- checks and exact results;
- unresolved issue, if any;
- final Git status;
- smallest next task.

Never report an unrun check as passed.
