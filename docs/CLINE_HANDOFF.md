# Gorilla Coding Cline Handoff 🦄

## Run The Current Approved Task

Open the Tethers workspace in VS Code, select Cline, and type:

```text
/tethers-task.md
```

The workflow reads `docs/CURRENT_CLINE_TASK.md`, verifies the live Git state,
loads only task-relevant context, and implements only when the packet is `READY`
and Cline is the named owner.

If Cline is in Plan mode, let it return its short plan, switch to Act mode, and
say:

```text
Implement the approved current task.
```

## After Cline Finishes

Cline must stop with either `COMPLETE` or `BLOCKED`, write the named worker note,
and return a concise report containing:

- files changed;
- important implementation choices;
- checks run and exact results;
- checks not run;
- unresolved risks or smallest blocker;
- worker-note path;
- final Git status;
- pushed commit or branch reference when available.

Paste that report to Lucy in ordinary chat.

Lucy then inspects the pushed GitHub evidence and decides one of:

1. `ACCEPTED`;
2. one bounded correction for Cline;
3. escalation to Codex because the task is Red or requires local machine access,
   Git/environment work, recovery, or difficult diagnosis.

Pasting the report is an accepted Gorilla Coding handoff. The report is not the
source of truth; the packet, worker note, code, tests, and Git remain the durable
evidence.

## Hand Cline A New Task

Lucy compiles or updates `docs/CURRENT_CLINE_TASK.md`. Once it is approved and
marked `READY`, run:

```text
/tethers-task.md
```

A task may also be pasted after the command when necessary:

```text
/tethers-task.md

<paste the approved Lucy task here>
```

Cline reinspects the live repository and normalises only an explicitly approved
handover. It must not turn an unresolved Red decision into code.

## When No Task Is Ready

Return to Lucy. Cline must not invent, compile, authorise, or begin the next task.
Lucy inspects GitHub, reviews the previous result, and provides the next bounded
task or routes the work to Codex.

## Safety

- The repository and Git are the source of evidence.
- `READY` authorises only the named bounded implementation.
- Cline does not commit or push unless the task explicitly authorises it.
- Existing unrelated changes must be preserved.
- Tests, compiler output, fixtures, Trails, and Git decide completion.
- Work, evidence, report, and worker note are required before `COMPLETE`.
- Cline stops after its task.
- Lucy controls continuation and acceptance.
- Codex handles Red work, difficult local failures, Git/environment recovery, and
  machine-required diagnosis.
