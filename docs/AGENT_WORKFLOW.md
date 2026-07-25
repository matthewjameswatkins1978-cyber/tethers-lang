# Tethers Agent Development Workflow

## Purpose

Use the least expensive capable route for each task without weakening Tethers'
correctness, determinism, permission boundaries, implementation quality, or
auditability.

Repository documents, code, compiler output, tests, fixtures, Trails, and Git are
the source of evidence. Agent reports are claims to verify.

`docs/PROJECT_CONTROL.md` defines ownership, task states, bounded context,
worker notes, review, and stopping. `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md`
defines how production implementation languages are used. This document applies
those contracts to the available agent routes.

## Roles, Not Brands

Treat roles as responsibilities rather than permanent model assignments:

- **Product owner, Matthew:** direction, priorities, taste, consequential
  trade-offs, consent, and final product judgement.
- **Task compiler and architecture reviewer, usually Lucy/Codex:** resolves
  ambiguity, freezes decisions, classifies risk, prepares the bounded packet,
  and performs independent review where required.
- **Implementation owner:** one named worker owns the authorised change,
  verification, and worker note.
- **Verifier:** independently checks evidence and returns `ACCEPTED`,
  `REJECTED`, or a bounded correction verdict.
- **Repository and toolchain:** durable task state and objective evidence.

Ordinary chat Lucy may inspect pushed GitHub state and perform architecture,
review, task compilation, and acceptance checking when local machine access is
not required. Codex is reserved for Red gates, difficult local diagnosis, Git or
environment work, and tasks that genuinely require direct computer access.

Current preferred implementation routes are:

- Green: Cline/DeepSeek or another reliable low-cost worker.
- Amber: Copilot or another repository-aware worker, normally isolated.
- Red: design and sign-off by Lucy/Codex; implementation by the most suitable
  authorised worker.

These routes may change with measured cost, reliability, and availability. Never
lower a task's risk class to fit a cheaper worker.

## Task Classification

### Green

Narrow, reversible, follows an established pattern, and has objective focused
verification. Self-verification may be sufficient when the packet permits it.

### Amber

Crosses several files or modules and requires moderate implementation judgement,
but the behaviour and invariants are already specified. Requires one bounded
final review.

### Red

Changes language or protocol semantics, permissions, capability trust,
persistence, compatibility, concurrency, determinism, security boundaries, or
hard-to-reverse architecture. Design must be frozen before implementation and
independently signed off afterward.

When classification is genuinely uncertain, use the higher class.

## Implementation Language

All production code follows `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md`.

Task scope and code sophistication are separate questions. A small task should
produce a small diff, but may use powerful, idiomatic language features when
those features make the domain safer, clearer, or easier to maintain.

Do not write OCaml, Rust, PowerShell, or future implementation code as a tutorial
for Matthew. Explain technical choices in the packet, worker note, or review.
Do not use advanced technique decoratively. Use the least complicated technique
that accurately expresses and protects the design.

## Standard Task Packet

Every control-v1 implementation task starts from
`docs/TASK_PACKET_TEMPLATE.md` and identifies:

1. control contract, state, colour, one owner, and route;
2. base branch, implementation checkpoint, and worker-note path;
3. objective and relevant existing behaviour;
4. required behaviour;
5. relevant files and interfaces;
6. frozen decisions and invariants;
7. permitted and forbidden changes;
8. acceptance criteria paired with verification;
9. stop conditions;
10. exact expected pre-existing changes.

The worker note uses `docs/WORKER_NOTE_TEMPLATE.md` and is part of completion,
not an optional chat report.

A worker must stop when requirements conflict, an architectural or safety
decision is missing, unrelated failures prevent trustworthy verification, or two
materially similar attempts fail to converge.

## Working Sequence

1. Inspect the live repository, packet, and Git state.
2. Confirm task class, owner, route, base, and expected pre-existing changes.
3. Run the packet consistency checker.
4. Read only task-relevant authoritative documents and code.
5. Agree a design first when the task crosses a Red boundary.
6. Implement only the accepted scope using the target language idiomatically.
7. Run formatter, compiler, focused tests, relevant regression tests,
   integration scripts, and whitespace checks required by the packet.
8. Inspect the complete diff and final Git status.
9. Write the exact worker note, update the task state honestly, and update short
   current-state documents only with established facts.
10. State the smallest useful next action and stop.

Do not commit, push, merge, amend, tag, publish, or open a pull request unless
the task explicitly authorises it.

## Handoff And Continuation

The normal Cline entry point is `/tethers-task.md`. It reads the approved
contract from `docs/CURRENT_CLINE_TASK.md`, checks live Git state, and loads only
task-relevant context.

The local `/next-tethers-task` Copilot prompt is optional. Use it when direct
checkout inspection materially helps. Ordinary chat Lucy can instead inspect
pushed GitHub evidence and compile or review the next task without consuming
Codex computer credits.

A completed task leads to one review verdict and, when appropriate, one bounded
`PROPOSED` next packet. Do not prepare the next task while the implementation
owner is still working. Do not create repeated audit loops without new evidence.

A milestone gate, Red decision, agent disagreement, ambiguous evidence, or
untrustworthy local state must stop ordinary continuation and route to the
appropriate independent reviewer.

## Task-Packet Consistency

Before authoring a packet, capture the implementation checkpoint and exact dirty
paths. `Expected pre-existing changes` is that live snapshot, not a copied list
from an older task.

Each required negative branch must have a matching acceptance criterion and
focused check. A representative failure case is not evidence for several
separate fail-closed requirements.

Run before handoff and before claiming completion:

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
```

The checker validates control structure, ownership, base and dirty-state
consistency, acceptance-to-verification mapping, and required worker-note state.

## Cost And Safety Posture

- Optimise total compute and Matthew effort per accepted correct change.
- Prefer ordinary chat Lucy for repository-visible thought and review.
- Reserve computer-enabled frontier work for tasks that need the machine or Red
  judgement.
- Do not multiply agents unless independent parallel evidence has real value.
- Do not enable additional paid usage merely to avoid a clean stop.
- Preserve terminal and consequential-action approval boundaries.
- After two materially similar failures, stop with exact evidence and one
  smallest unresolved question.
- Put durable handovers in the repository. Do not use Matthew as the network
  cable between agents.
