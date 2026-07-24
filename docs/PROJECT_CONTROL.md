# Tethers Project Control Loop

## Purpose

Keep Matthew in control of product direction without making him carry technical
context between agents. Use the least expensive capable worker while preserving
Tethers' architecture, trust boundaries, determinism, and evidence standards.

This document defines the operating procedure. `docs/AGENT_WORKFLOW.md` defines
agent guidance; the Constitution, specifications, decisions, code, tests, and
Git remain authoritative for product behaviour.

## Control roles

- **Matthew — product owner:** direction, taste, priorities, consequential
  trade-offs, consent, and final product judgement.
- **Task compiler — currently Lucy/Codex:** resolves ambiguity, freezes the
  relevant decisions, selects one bounded task, chooses a risk class, and
  assembles the context packet.
- **Implementation owner — one named worker:** owns the task from start through
  changes, evidence, and worker note.
- **Verifier — independent when required:** checks repository evidence and
  returns `ACCEPTED`, `CORRECTION REQUIRED`, or `MILESTONE SIGNED OFF`.
- **Repository:** holds the durable packet, evidence references, current state,
  and worker notes. Chat transcripts are not the project record.

One person or tool may fill more than one role on Green work, but Red work
requires independent architectural sign-off. No implementation owner signs off
its own Red work.

## Risk is separate from routing

A task colour describes risk and ambiguity. It does not permanently name a
vendor or model.

| Class | Meaning | Current preferred route |
| --- | --- | --- |
| Green | Existing pattern, narrow, reversible, objectively testable | Cline/DeepSeek or Copilot inline |
| Amber | Multi-file or module interaction, specified behaviour, moderate judgement | Copilot in an isolated worktree |
| Red | Semantics, permissions, trust, persistence, compatibility, concurrency, determinism, or architecture | Lucy/Codex designs and signs off; an appropriate worker may implement |

Change the route when measured reliability, availability, or cost changes.
Do not weaken the task classification to fit a cheaper worker.

## One active-task state machine

`docs/CURRENT_CLINE_TASK.md` is the current task contract despite its historical
name. Use these states:

1. `PROPOSED` — compiled but not authorised for implementation.
2. `READY` — approved, with one owner and one worker-note path.
3. `IN_PROGRESS` — that owner is working; no second worker may reimplement it.
4. `BLOCKED` — work stopped with evidence and one smallest unresolved question.
5. `COMPLETE` — the owner claims the work, verification, and worker note exist.
6. `ACCEPTED` — independent checking, when required, has accepted the result.
7. `REJECTED` — evidence proves the implementation does not meet the contract.

Only the task compiler may move `PROPOSED` to `READY`. Only the named
implementation owner may use `IN_PROGRESS`, `BLOCKED`, or `COMPLETE`. The
required verifier moves `COMPLETE` to `ACCEPTED` or `REJECTED`.

A task is not `COMPLETE` merely because code was written or tests passed. It
requires the work, required evidence, and the worker note named by the packet.

## Compiled context packet

Each packet contains only context that can affect the task:

- exact outcome and owner;
- risk class and current route;
- base branch and implementation checkpoint;
- relevant files, interfaces, and authoritative document sections;
- decisions and invariants that are frozen for this task;
- permitted and forbidden scope;
- acceptance criteria, each paired with evidence;
- stopping and escalation conditions;
- expected pre-existing changes;
- exact worker-note path.

Workers read the packet, the files it names, and task-relevant code. They do not
read the entire project archive by default. Reviewers may load wider
architectural context where the risk class requires it.

Issued decisions remain frozen. A worker may implement them, report a concrete
contradiction, or stop with one precise question. It may not silently redesign
the surrounding system.

## Work and failure rules

- One task has one implementation owner.
- Another worker does not independently reimplement it unless the task is
  formally rejected or reassigned.
- After two materially similar failed attempts, stop. Record the command or
  action, exact failure, attempted remedies, and smallest unresolved question.
- If an external effect may have occurred but no trustworthy result exists,
  report `uncertain`; never retry automatically.
- When acceptance checks pass, required evidence is captured, and the worker
  note exists, stop. Do not spend requests on speculative cleanup or repeated
  validation without a new reason.
- A report is a claim. Code, tests, fixtures, Trail data, and Git are evidence.

## Worker-note return journey

Every task packet names one file under `docs/worker-notes/`. The implementation
owner creates it from `docs/WORKER_NOTE_TEMPLATE.md`.

The note records the task, actual changes, in-scope decisions, exact evidence,
discoveries, remaining risks, smallest next action, and references. It is a
concise factual handover, not a transcript.

Accepted notes remain in the repository as durable project memory. The next
task compiler selects only notes relevant to the next decision; it never dumps
the entire history into a worker context.

## Verification and review

Green work may be accepted from objective evidence when the diff is narrow and
the packet permits self-verification. Amber work receives one bounded final
review. Red work requires independent architecture review and milestone
sign-off.

A verifier checks:

1. live branch, base, status, and complete diff;
2. each requirement against its paired acceptance evidence;
3. architectural and safety boundaries;
4. unexpected changes and unsupported assumptions;
5. worker-note accuracy;
6. whether the task should stop.

Verification ends with one verdict, not another implementation pass. If
correction is required, compile the smallest correction as a new task rather
than allowing an open-ended repair loop.

## Matthew-facing dashboard

`docs/PROJECT_DASHBOARD.md` is the short control surface. Keep it factual and
brief:

- current milestone and verified checkpoint;
- active task, owner, state, and risk;
- last accepted result;
- decision required from Matthew, or `None`;
- next route;
- cost/risk drift.

Matthew should not need to read implementation transcripts to know what is
happening.

## Control checks

Before handoff and again before claiming completion, run:

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
```

The checker validates packet structure, task state, ownership, base/dirty-state
consistency, acceptance-to-verification mapping, and the required worker note
for completed work.

## Improvement loop

After real work, record only demonstrated friction: bad routing, repeated
failure, missing context, wasted review, or overload. Change the smallest useful
part of this loop and test it on the next real task.

Do not optimise the process for the number of agents used. Optimise total
compute and Matthew effort per accepted, correct change.
