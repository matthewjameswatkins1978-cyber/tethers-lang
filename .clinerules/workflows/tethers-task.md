# Run the Current Tethers Task

Read:

- `AGENTS.md`
- `docs/PROJECT_CONTROL.md`
- `docs/AGENT_WORKFLOW.md`
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
2. Confirm Cline is the single named `Owner` or named by the `Route`. If another
   worker owns it, stop.
3. Set the packet to `IN_PROGRESS`.
4. Read only the task-relevant authoritative documents, prior worker notes,
   code, and tests named by the packet.
5. Reinspect the implementation before trusting any handover claim.
6. In Plan mode, return at most eight concrete steps. In Act mode, implement
   only the packet.
7. Run every required check and inspect the complete diff and Git status.
8. Create the worker note at the exact `Worker note` path using
   `docs/WORKER_NOTE_TEMPLATE.md`.
9. Set the packet to `COMPLETE` only when the work, evidence, and worker note
   exist. Otherwise set it to `BLOCKED`.
10. Run the packet checker again.
11. Stop. Do not commit, push, install, clean up beyond scope, compile the next
    task, or continue validating without a new reason unless explicitly
    authorised.

After two materially similar failed attempts, stop with the exact action,
failure, attempted remedies, and one smallest unresolved question.

## Other states

- `PROPOSED`: review without implementation and ask only for the approval or
  unresolved decision genuinely required. Do not change it to `READY`.
- `IN_PROGRESS`: continue only when Cline is the named owner and this is the
  same task session. Otherwise stop to avoid a second implementation owner.
- `COMPLETE`, `ACCEPTED`, or `REJECTED`: do not implement or compile another
  task. Tell Matthew to run `/next-tethers-task` in Copilot, or request Codex
  when the dashboard says a milestone review is due.
- `BLOCKED`: do not guess or start a replacement. Return the worker-note
  evidence and smallest unresolved question to the task compiler.

If a handover was pasted with this command, treat it as a claim. Do not replace
the repository packet or its frozen decisions unless the task compiler
explicitly authorises that update.

Never claim an unrun check passed. Never use Matthew as the technical message
bus when the evidence belongs in the packet, worker note, dashboard, or Git.
