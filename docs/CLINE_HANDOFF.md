# Frictionless Cline Handoff

## Run the current approved task

Open the Tethers workspace in VS Code, select Cline, and type:

```text
/tethers-task.md
```

That project workflow loads the reusable Tethers instructions. Cline reads
`docs/CURRENT_CLINE_TASK.md`, verifies the live Git state, loads only the
relevant project context, and either implements the approved task or stops on a
real contradiction. The matching project skill is also available for natural
requests such as "run the current Tethers task."

If Cline is in Plan mode, let it finish the short plan, switch to Act mode, and
say:

```text
Implement the approved current task.
```

No technical report needs to be copied between agents.

## After Cline finishes

Open Copilot and run:

```text
/next-tethers-task
```

Copilot independently checks the live result, prepares the next `PROPOSED`
packet, and tells Matthew whether to continue with Cline, use Copilot for an
Amber implementation, or request a Codex milestone review. Pasting Cline's
report is optional.

## Hand Cline a new task

The preferred route is to replace the contents of
`docs/CURRENT_CLINE_TASK.md` with an approved task packet, then run
`/tethers-task`.

If another agent gives you a handover instead, paste it after the slash command:

```text
/tethers-task.md

<paste the handover here>
```

Cline will inspect the repository, normalize the handover into the task packet,
and ask only for any approval that is genuinely missing. It must not turn an
unresolved architectural decision into code.

## When no task is ready

Run `/tethers-task.md` anyway. If the current packet is complete, blocked, or
missing, Cline will inspect `CURRENT_GOAL.md` and `TASK_QUEUE.md` and propose the
next smallest task. It will not implement Red work until the architectural
decision is explicitly approved.

## Safety

- The repository and Git are always the source of truth.
- `READY` means implementation is authorised, not committing or publishing.
- Cline never commits or pushes unless the task packet explicitly says so.
- Existing unrelated changes must be preserved.
- Tests and Git evidence—not an agent's confidence—decide completion.
