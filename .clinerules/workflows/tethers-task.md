# Run The Current Tethers Task

Read:

- `AGENTS.md`
- `docs/PROJECT_CONTROL.md`
- `docs/AGENT_WORKFLOW.md`
- `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/PROJECT_DASHBOARD.md`

Then inspect:

```powershell
git status --short --branch
git rev-parse HEAD
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
```

Do not edit until the packet and live Git state agree.

## READY

When the packet is `READY`:

1. Require the packet checker to pass. Stop rather than guessing when its base,
   planning-only descendants, or expected dirty paths disagree.
2. Confirm Cline is the single named `Owner` or named by the `Route`. If Codex or
   another worker owns it, stop.
3. Set the packet to `IN_PROGRESS`.
4. Read only the task-relevant authoritative documents, prior worker notes,
   code, and tests named by the packet.
5. Reinspect the implementation before trusting any handover claim.
6. In Plan mode, return at most eight concrete steps. In Act mode, implement only
   the packet.
7. Use the implementation language idiomatically under
   `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md`.
8. Run every required check and inspect the complete diff and Git status.
9. Create the worker note at the exact `Worker note` path using
   `docs/WORKER_NOTE_TEMPLATE.md`.
10. Set the packet to `COMPLETE` only when the work, evidence, and worker note
    exist. Otherwise set it to `BLOCKED`.
11. Run the packet checker again.
12. Return the concise report defined by `docs/CLINE_HANDOFF.md`.
13. Stop. Do not commit, push, install, clean up beyond scope, compile the next
    task, or continue validating without a new reason unless explicitly
    authorised.

After two materially similar failed attempts, stop with the exact action,
failure, attempted remedies, and one smallest unresolved question. Codex, not a
third Cline attempt, is the normal escalation.

## Other States

- `PROPOSED`: review without implementation. Do not change it to `READY` unless
  the packet explicitly records Matthew/Lucy approval.
- `IN_PROGRESS`: continue only when Cline is the named owner and this is the same
  task session. Otherwise stop to avoid a second implementation owner.
- `COMPLETE`, `ACCEPTED`, or `REJECTED`: do not implement or compile another
  task. Tell Matthew to paste the current report to Lucy.
- `BLOCKED`: do not guess or start a replacement. Return the worker-note evidence
  and smallest unresolved question to Lucy through Matthew.
- missing packet: stop and ask Matthew to obtain the next task from Lucy.

## Pasted Handovers

A pasted handover is usable only when it is explicitly an approved Lucy task.
Reinspect the live repository, preserve its decisions and exclusions, and
normalise it into the packet only when authorised. Do not convert an unresolved
Red decision into implementation work.

Never claim an unrun check passed. Copilot is not part of the current route.
