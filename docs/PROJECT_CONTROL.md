# Tethers Project Control Loop

Status: current operating procedure

## Purpose

Keep Matthew in control of product direction without making him reconstruct the
technical project from memory. Preserve Tethers' architecture, trust boundaries,
determinism, implementation quality, and evidence while using the smallest
practical team.

The current operating mode is **Gorilla Coding 🦄**: Lucy controls architecture
and continuation, OpenCode normally implements, and Codex enters for Red work or
genuine difficulty. It is agile and asymmetric, never sloppy.

The Constitution, specifications, decisions, implementation standard, code,
tests, fixtures, Trails, and Git remain authoritative for product behaviour and
engineering evidence.

## Control Roles

- **Matthew, product owner:** direction, taste, priorities, consequential
  trade-offs, consent, installation, publication, and final product judgement.
- **Lucy, task compiler and controller:** resolves ambiguity, freezes relevant
  decisions, classifies risk, assembles one bounded packet, reviews pushed
  evidence, and chooses accept, correct, or escalate.
- **OpenCode, default implementation owner:** owns ordinary Green and Amber work
  from authorised packet through changes, checks, report, and worker note.
- **Codex, escalation engineer and Red reviewer:** handles Red implementation or
  sign-off, difficult local diagnosis, Git/environment/recovery work, and tasks
  OpenCode cannot complete reliably.
- **Repository:** holds durable packets, decisions, current state, worker notes,
  code, tests, and evidence references.

Cline and Goose are not part of the active route. Historical filenames,
branches, packets, and worker notes may retain their names.

One worker owns each implementation task. Red work requires independent
architectural sign-off; no implementation owner signs off its own Red work.

## Instruction Loading

`AGENTS.md` is the startup authority for coding agents. OpenCode also loads the
core control documents through the project-root `opencode.json` when the active
OpenCode release supports the `instructions` field.

A file named inside `AGENTS.md` is not assumed to have been loaded. Before any
mutation, the implementation owner must complete the startup gate in `AGENTS.md`
and explicitly read any mandatory document missing from the effective context.

## Risk Is Separate From Routing

A colour describes risk, not a permanent model assignment.

| Class | Meaning | Current route |
| --- | --- | --- |
| Green | Existing pattern, narrow, reversible, objectively testable | OpenCode implements; Lucy reviews as needed |
| Amber | Multi-file or module interaction, settled behaviour, moderate judgement | OpenCode implements; Lucy performs one bounded review |
| Red | Semantics, permissions, trust, persistence, compatibility, concurrency, determinism, security, or architecture | Lucy freezes the design; Codex normally implements or performs computer-enabled sign-off |

Do not lower a risk class to fit available compute. Do not escalate ordinary
work merely because it uses advanced language features.

## One Active-Task State Machine

`docs/CURRENT_CLINE_TASK.md` is the single current implementation contract. Its
historical filename does not make Cline responsible for task compilation or
implementation.

1. `PROPOSED` — Lucy has compiled a candidate task; implementation is not yet
   authorised.
2. `READY` — approved, with one owner, route, base, and worker-note path.
3. `IN_PROGRESS` — that owner is working; no second worker may reimplement it.
4. `BLOCKED` — work stopped cleanly with evidence and one smallest unresolved
   question.
5. `COMPLETE` — the owner claims the work, required checks, report, and worker
   note exist.
6. `ACCEPTED` — Lucy or the required verifier has accepted the evidence.
7. `REJECTED` — evidence proves the implementation does not meet the contract.

Lucy normally moves `PROPOSED` to `READY`. Matthew's explicit approval may be
recorded through Lucy. Only the named implementation owner uses `IN_PROGRESS`,
`BLOCKED`, or `COMPLETE`. Lucy or the required independent verifier moves
`COMPLETE` to `ACCEPTED` or `REJECTED`.

A task is not `COMPLETE` merely because code was written or one test passed.
Completion requires:

- the authorised implementation exists
- a committed implementation checkpoint exists
- required verification was run against that checkpoint
- the final worker note records those actual results
- only closeout documentation may follow the checkpoint

`BLOCKED` may reference `WORKTREE` when intentionally uncommitted.

`COMPLETE` may not.

## Compiled Context Packet

Each packet contains only context capable of changing the task:

- exact outcome, owner, route, and risk;
- base branch, implementation checkpoint, and expected pre-existing changes;
- relevant files, interfaces, and authoritative document sections;
- frozen decisions and invariants;
- permitted and forbidden scope;
- acceptance criteria paired with evidence;
- stop and escalation conditions;
- exact worker-note path.

Workers read the startup documents, the packet, the files it names, and
task-relevant code. They do not load the whole project archive by default. Lucy
and Red reviewers may inspect wider architectural context when necessary.

Issued decisions remain frozen. A worker may implement them, demonstrate a
contradiction, or stop with one precise question. It may not silently redesign
the surrounding system.

## Work And Failure Rules

- One task has one implementation owner.
- OpenCode and Codex must not edit the same task or checkout simultaneously.
- Another worker may continue only after formal reassignment or rejection.
- After two materially similar failed attempts, stop. Record the exact command or
  action, failure, attempted remedies, and smallest unresolved question.
- If an external effect may have occurred without a trustworthy result, report
  `uncertain`; never retry automatically.
- When acceptance checks pass and evidence is captured, stop. Do not spend
  requests on speculative cleanup or repeated validation without new reason.
- A report is a claim. Code, tests, fixtures, compiler output, Trails, and Git are
  evidence.

## Return Journey

Every packet names one worker note under `docs/worker-notes/`. The implementation
owner creates it from `docs/WORKER_NOTE_TEMPLATE.md`.

The note records actual changes, in-scope decisions, exact evidence, discoveries,
remaining risks, and references. It is concise project memory, not a transcript.

OpenCode also returns a short report to Matthew. Matthew may paste that report
into ordinary chat Lucy. During Gorilla Coding mode this is an accepted and
useful transport step.

The pasted report does not replace durable evidence. Lucy verifies pushed GitHub
state where available and uses the worker note, diff, tests, packet, and Git
references to decide the result. When critical evidence exists only locally,
Lucy routes the check to Codex or asks for the smallest exact local evidence.

## Verification And Review

Green work may be accepted from objective evidence when the diff is narrow and
the packet permits it.

Amber work receives one bounded final Lucy review. OpenCode remains the normal
implementation owner unless Lucy routes a specific task to Codex.

Red work requires:

1. a frozen design before implementation;
2. an explicitly chosen implementation owner;
3. independent architecture review and sign-off afterward.

A verifier checks:

1. branch, base, status, and complete diff;
2. every requirement against its paired evidence;
3. architectural, semantic, and safety boundaries;
4. unexpected changes and unsupported assumptions;
5. worker-note and report accuracy;
6. whether work should stop.

Verification ends with one verdict. If correction is required, Lucy compiles the
smallest correction as a new task rather than allowing an open-ended repair loop.

## Matthew-Facing Dashboard

`docs/PROJECT_DASHBOARD.md` remains the short human-facing control surface. It is
not implementation authority and is not automatically loaded into OpenCode.
Keep it factual when a reporting task updates it:

- current milestone and accepted checkpoint;
- active task, owner, state, and risk;
- last accepted result;
- decision required from Matthew, or `None`;
- next route: OpenCode, Codex, or stop;
- cost or risk drift.

Matthew should not need implementation transcripts to understand the project,
but he may paste OpenCode's concise report to Lucy to keep the current loop
moving.

## Control Check

Before handoff and before claiming completion, run:

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
```

The checker validates packet structure, task state, ownership, base and dirty
state, acceptance-to-verification mapping, required worker-note state,
implementation-checkpoint ancestry, and post-checkpoint closeout scope.

## Improvement Rule

After real work, record only demonstrated friction: bad routing, repeated
failure, missing context, wasted review, overload, or unnecessary handoff.
Change the smallest useful part and test it on the next real task.

Optimise total compute and Matthew effort per accepted correct change. Short
supply lines are the point.
