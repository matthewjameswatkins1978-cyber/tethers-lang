# Tethers Agent Development Workflow

Status: current operating workflow

## Purpose

Build Tethers with the least friction and compute that still produces correct,
well-evidenced software.

The current operating mode is **Gorilla Coding**: a deliberately small command
chain built for speed, continuity, and scarce computer-enabled model usage. It is
not permission to lower engineering standards.

Repository documents, code, compiler output, tests, fixtures, Trails, and Git are
the evidence. Agent reports are useful handovers, but remain claims until checked
against that evidence.

`docs/PROJECT_CONTROL.md` defines task state and evidence.  
`docs/IMPLEMENTATION_LANGUAGE_STANDARD.md` defines implementation technique.

## Instruction Startup

OpenCode automatically receives the applicable `AGENTS.md`. The project-root
`opencode.json` also names the core control documents for OpenCode releases that
support additional instruction loading.

Neither mechanism removes the startup gate. Before any edit, the active worker
must report the effective instruction files and explicitly read any mandatory
file that is not already loaded. A filename mentioned by another document is not
assumed to be an import.

## Current Team

- **Matthew, product owner:** direction, priorities, consequential trade-offs,
  consent, and final product judgement.
- **Lucy in ordinary chat, architect and controller:** inspects pushed GitHub
  state, resolves ambiguity, makes architecture and semantic decisions, compiles
  the next bounded task, reviews OpenCode's result, accepts or rejects work, and
  decides when Codex is required.
- **OpenCode, primary implementation owner:** performs ordinary Green and Amber
  implementation, runs the required checks, records evidence, writes the worker
  note, and stops.
- **Codex, escalation engineer:** enters for Red implementation or review, a
  difficult local failure, Git or environment work, recovery, or a problem
  OpenCode cannot resolve cleanly.
- **Repository and toolchain:** hold durable state and objective evidence.

Cline, Goose, and Copilot are not part of the current active workflow.
Historical filenames, packets, branches, and notes may retain their names.

## Routing Rule

Default route:

```text
Matthew and Lucy decide direction
    -> Lucy compiles one bounded task
    -> Matthew gives it to OpenCode
    -> OpenCode implements, verifies, and reports
    -> Matthew gives the report to Lucy
    -> Lucy inspects GitHub evidence and decides: accept, correct, or escalate
```

Use Codex when:

- the task is Red;
- OpenCode reports a genuine architectural contradiction;
- OpenCode has made two materially similar failed attempts;
- the problem depends on unpushed local state, terminal behaviour, Git recovery,
  environment configuration, or machine access Lucy does not have;
- pushed evidence is incomplete or cannot establish the truth;
- a milestone or release gate requires independent computer-enabled review.

Lucy has final say on technical routing and acceptance within Matthew's product
authority. Matthew may perform the physical routing between chat, OpenCode, and
Codex.

## Risk Classification

### Green

Narrow, reversible, follows an established pattern, and has objective focused
verification. OpenCode implements. Lucy may accept from the pushed diff, tests,
and worker note.

### Amber

Crosses several files or modules and requires moderate implementation judgement,
but behaviour and invariants are already specified. OpenCode remains the default
implementation owner. Lucy performs one bounded final review.

### Red

Changes language or protocol semantics, permissions, capability trust,
persistence, compatibility, concurrency, determinism, security boundaries, or
hard-to-reverse architecture.

Lucy freezes the design before implementation. Codex normally implements or
performs the computer-enabled final review. A Red implementation owner never
signs off its own architectural work.

When classification is genuinely uncertain, use the higher class.

## Implementation Standard

All production code follows `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md`.

Task size and language sophistication are separate. A bounded task should make a
bounded diff, but may use powerful, idiomatic OCaml or Rust features when they
make the domain safer, clearer, or easier to maintain.

Do not write implementation code as a tutorial for Matthew. Explain unfamiliar
but justified technique in the task packet, implementation report, worker note,
or review. Do not use advanced technique decoratively.

## Task Packet

Every implementation task uses `docs/CURRENT_CLINE_TASK.md`. The filename is a
historical interface and does not name the active owner. The packet identifies:

1. state, risk, one owner, and route;
2. base branch, implementation checkpoint, and expected pre-existing changes;
3. objective and relevant existing behaviour;
4. required behaviour;
5. relevant files and interfaces;
6. frozen decisions and invariants;
7. permitted and forbidden changes;
8. acceptance criteria paired with verification;
9. stop conditions;
10. exact worker-note path.

`PROPOSED` is design-ready but not authorised.
`READY` authorises the named implementation.
`IN_PROGRESS` belongs to that owner.
`COMPLETE` is the named owner's evidence-backed completion claim.
`BLOCKED` is a clean stop with one smallest unresolved question.
`ACCEPTED` and `REJECTED` are Lucy or the required verifier's verdicts.

## Normal Work Sequence

1. Lucy inspects the current GitHub state and compiles one task.
2. Matthew gives the approved task to OpenCode in the correct worktree.
3. OpenCode completes the `AGENTS.md` startup report and reads the mandatory
   control files before editing.
4. OpenCode verifies packet state and live local Git state before editing.
5. OpenCode implements only the authorised scope using the target language
   idiomatically.
6. OpenCode runs development and focused checks, then inspects the complete diff.
7. OpenCode commits the implementation checkpoint.
8. OpenCode runs every required acceptance check against that exact committed
   checkpoint. If verification changes production code, return to step 6 and
   establish a new implementation checkpoint.
9. Only after final verification, OpenCode writes the worker note from actual
   final results, marks the task `COMPLETE` or `BLOCKED`, and commits/pushes
   closeout documentation only.
10. OpenCode returns a concise report to Matthew.
11. Matthew pastes that report to Lucy. This is an accepted handoff in Gorilla
    Coding mode, not a process failure.
12. Lucy checks pushed GitHub evidence where available, reads the relevant worker
    note and diff, then records one verdict or compiles one bounded correction.
13. Matthew routes the next task to OpenCode or Codex as Lucy directs.

OpenCode must not invent, authorise, or begin the next task. Lucy controls
continuation.

Do not write or finalise `COMPLETE` evidence before the implementation checkpoint
exists. A check result is stale if code affecting that check changed afterward.

## Report Contract

The implementation owner's return report should contain only:

- outcome: `COMPLETE` or `BLOCKED`;
- files changed;
- important implementation choices;
- commands and checks actually run, with exact results;
- commands not run;
- unresolved risks or the smallest blocker;
- worker-note path;
- final Git status;
- commit or pushed branch reference when one exists.

The report may be pasted into chat. Durable decisions and evidence still belong
in the packet, worker note, code, tests, dashboard, and Git.

## Failure And Stop Rules

- One task has one implementation owner.
- Do not let OpenCode and Codex edit the same task or checkout simultaneously.
- After two materially similar failed attempts, stop and escalate with exact
  evidence.
- Do not turn a missing semantic, permission, or trust decision into code.
- Do not use repeated audit loops without new evidence.
- Do not begin cleanup or the next task after completion.
- Commit, push, merge, amend, tag, installation, and publication require explicit
  authority in the task or from Matthew.
- Never claim an unrun check passed.

## Cost Posture

- Use ordinary chat Lucy for all repository-visible architecture, planning,
  review, and acceptance work.
- Use OpenCode as the normal coding engine.
- Spend Codex only where machine access, Red risk, recovery, or demonstrated
  OpenCode difficulty justifies it.
- Optimise total compute and Matthew effort per accepted correct change, not the
  number of agents involved.

That is Gorilla Coding: few participants, short supply lines, strong evidence,
and no ceremonial paperwork jungle.
