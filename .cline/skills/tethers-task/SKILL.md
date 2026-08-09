---
name: tethers-task
description: Run or review the current bounded Tethers implementation task. Use when Matthew invokes /tethers-task.md, pastes an approved Lucy handover, or wants Cline to continue from docs/CURRENT_CLINE_TASK.md. Do not use this skill to invent or compile the next task.
---

**INACTIVE / HISTORICAL INTEGRATION.** Cline is not part of the current
active Tethers route. This skill file does not authorise repository mutation.
Current authority is `AGENTS.md`, `docs/PROJECT_CONTROL.md`,
`docs/AGENT_WORKFLOW.md`, and the current packet. Reactivation of Cline
requires an explicitly authorised future task.

The remaining content is preserved as historical integration detail.

---

# Tethers Task

This skill is the low-friction Cline entry point for Gorilla Coding 🦄.

## First Action

Read:

- `AGENTS.md`
- `docs/PROJECT_CONTROL.md`
- `docs/AGENT_WORKFLOW.md`
- `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md`
- `docs/CURRENT_CLINE_TASK.md`

Then inspect:

```powershell
git status --short --branch
git rev-parse HEAD
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
```

Do not edit anything until the packet and live Git state agree.

## Task Packet States

### `READY`

1. Run the packet checker and stop if it fails.
2. Confirm Cline is the named owner or route.
3. Confirm live dirty paths match `Expected pre-existing changes`.
4. Read only the task-named authoritative documents, code, tests, and worker
   notes.
5. Reinspect the relevant implementation before trusting the packet or pasted
   handover.
6. In Plan mode, produce at most eight concrete steps. In Act mode, implement
   only the packet.
7. Use the target language idiomatically under the implementation standard.
8. Run every required check in the stated order.
9. Inspect the complete diff and final Git status.
10. Write the exact worker note named by the packet.
11. Mark the packet `COMPLETE` only when every acceptance criterion and evidence
    requirement is met. Otherwise mark it `BLOCKED`.
12. Run the packet checker again.
13. Return the concise report defined by `docs/CLINE_HANDOFF.md`.
14. Stop. Do not commit, push, merge, amend, tag, install software, clean up
    beyond scope, or start the next task unless explicitly authorised.

### `PROPOSED`

Review without implementation. Do not authorise it yourself. State any concrete
contradiction or missing approval and stop. Lucy controls task approval and
continuation.

### `IN_PROGRESS`

Continue only when Cline is the named owner and this is the same task session.
Otherwise stop to avoid two owners.

### `COMPLETE`, `ACCEPTED`, or `REJECTED`

Do not implement or compile anything else. Tell Matthew to paste the current
report to Lucy.

### `BLOCKED`

Do not guess or invent a replacement. Return the exact evidence and smallest
unresolved question for Lucy. After two materially similar failed attempts,
Codex is the normal escalation.

### Missing Packet

Stop and tell Matthew to obtain the next bounded task from Lucy. Cline must not
infer the next job from `CURRENT_GOAL.md`, `TASK_QUEUE.md`, or the roadmap.

## Pasted Approved Handovers

When Matthew provides an explicitly approved Lucy handover:

1. reinspect the live repository;
2. preserve decisions, exclusions, tests, and stop conditions;
3. normalise it into `docs/CURRENT_CLINE_TASK.md` only when that write is
   authorised;
4. do not convert unresolved Red design into implementation;
5. implement only after the resulting packet is `READY`.

## Reporting

Return only:

- `COMPLETE` or `BLOCKED`;
- files changed;
- important implementation choices;
- checks run and exact results;
- checks not run;
- unresolved risks or smallest blocker;
- worker-note path;
- final Git status;
- pushed commit or branch reference when available.

Never report an unrun check as passed. Copilot is not part of the current route.
