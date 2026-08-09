# Gorilla Coding Worker Handoff 🦄

Historical filename: `docs/CLINE_HANDOFF.md`. Cline is not part of the current
active route. This document is the current worker-neutral handoff guide.

## Current Operating Route

```text
Lucy compiles tasks, architecture, and reviews
    -> OpenCode implements ordinary Green and Amber work
    -> Codex enters for Red work, local/Git/environment recovery, or
       machine-required diagnosis
    -> Matthew routes concise worker reports back to Lucy
```

`docs/CURRENT_CLINE_TASK.md` is a historical filename; the named packet
owner is authoritative.

## After The Worker Finishes

The worker must stop with either `COMPLETE` or `BLOCKED`, write the named
worker note, and return a concise report containing:

- `COMPLETE` or `BLOCKED`;
- files changed;
- important implementation choices;
- checks and tests actually run, with exact results;
- checks and tests not run;
- unresolved risks or the smallest blocker;
- worker-note path;
- final Git status;
- implementation checkpoint and final branch reference;
- remote branch;
- full remote HEAD SHA;
- local HEAD equals remote HEAD confirmation.

Paste that report to Lucy in ordinary chat.

Lucy then inspects the pushed GitHub evidence and decides one of:

1. `ACCEPTED`;
2. one bounded correction;
3. escalation to Codex.

Pasting the report is an accepted Gorilla Coding handoff. The report is not
the source of truth; the packet, worker note, code, tests, and Git remain the
durable evidence.

## When No Task Is Ready

Return to Lucy. The worker must not invent, compile, authorise, or begin the
next task. Lucy inspects GitHub, reviews the previous result, and provides the
next bounded task or routes the work to Codex.

## Safety

- The repository and Git are the source of evidence.
- `READY` authorises only the named bounded implementation.
- Every `COMPLETE` branch is pushed normally to `origin`. The worker resolves
  the full remote HEAD SHA and confirms it equals local HEAD with clean Git
  status.
- No force-push, direct `main` update, merge, or other publication is implied.
- Existing unrelated changes must be preserved.
- Tests, compiler output, fixtures, Trails, and Git decide completion.
- Work, evidence, report, and worker note are required before `COMPLETE`.
- The worker stops after its task.
- Lucy controls continuation and acceptance.
- Codex handles Red work, difficult local failures, Git/environment recovery,
  and machine-required diagnosis.
