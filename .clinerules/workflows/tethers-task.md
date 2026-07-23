# Run the Current Tethers Task

Read `docs/CURRENT_CLINE_TASK.md`, then inspect:

```powershell
git status --short --branch
git rev-parse HEAD
```

Do not edit until the packet and live Git state agree.

If the packet is `READY`:

1. Verify its base commit and expected pre-existing changes.
2. Read `AGENTS.md`, `.clinerules/`, and only the task-relevant documents and
   code named by the packet.
3. Reinspect the implementation before trusting the handover.
4. In Plan mode, return at most eight concrete steps. In Act mode, implement
   only the packet.
5. Run every required check, inspect the complete diff, and report exact
   evidence.
6. Update the packet to `COMPLETE` or `BLOCKED`.
7. Do not commit, push, install, or start another task unless explicitly
   authorised.

If the packet is `PROPOSED`, review it without implementation and ask only for
the approval or unresolved decision that is genuinely required.

If it is `COMPLETE`, `BLOCKED`, or missing, inspect `docs/CURRENT_GOAL.md` and
`docs/TASK_QUEUE.md`, inspect the relevant code, and write the next smallest
coherent packet to `docs/CURRENT_CLINE_TASK.md` with status `PROPOSED`.
Summarize it in plain English and ask Matthew for one approval. Do not implement
it until approved, and do not implement Red work without an approved
architectural decision.

If a handover was pasted with this command, inspect the repository and
normalize it into `docs/CURRENT_CLINE_TASK.md`. Preserve its decisions,
exclusions, tests, and stop conditions.

Never claim an unrun check passed.
