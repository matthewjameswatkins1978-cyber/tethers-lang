# Frictionless Cline Handoff

## Run the current approved task

Open the Tethers workspace in VS Code, select Cline, and type:

```text
/tethers-task.md
```

That project workflow verifies the live Git state, confirms Cline is the one
named owner, loads only the context named by the packet, and either implements
the approved task or stops on a real contradiction. Completion automatically
includes the evidence-backed worker note named by the packet. The matching
project skill is also available for natural requests such as "run the current
Tethers task."

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

Copilot independently checks the live result and worker note, records the
verdict in the short dashboard, and either prepares one next `PROPOSED` packet
or stops for Codex milestone review. Pasting Cline's report is unnecessary
unless repository evidence is genuinely unavailable.

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

Do not ask Cline to invent the next task. Run `/next-tethers-task` in Copilot.
It verifies the return note and repository evidence, then compiles one bounded
proposal or stops for Codex. This keeps task compilation separate from
implementation ownership.

## Safety

- The repository and Git are always the source of truth.
- `READY` means implementation is authorised, not committing or publishing.
- Cline never commits or pushes unless the task packet explicitly says so.
- Existing unrelated changes must be preserved.
- Tests and Git evidence—not an agent's confidence—decide completion.
- Work, required evidence, and the named worker note are all required before
  `COMPLETE`.
- Cline stops after its task; Copilot compiles the next packet; Codex handles Red
  and milestone gates.
