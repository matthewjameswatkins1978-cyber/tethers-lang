# Tethers Agent Development Workflow

## Purpose

Use the least expensive capable agent for each task without weakening Tethers'
correctness, determinism, permission boundaries, or auditability.

Repository documents, code, tests, and Git are the source of truth. Agent
reports are claims to verify, not evidence by themselves.

`docs/PROJECT_CONTROL.md` is the operational contract for ownership, task
states, bounded context, worker notes, verification, and stopping. This file
applies that contract to the current Tethers agent stack.

## Roles

- **Architecture reviewer (Sol/Codex):** semantics, trust boundaries,
  architectural decisions, contradictions, high-risk task design, and bounded
  final review.
- **GitHub Copilot:** primary implementer for repository-aware, multi-file work
  whose required behaviour is already specified.
- **Cline/DeepSeek:** narrow, mechanical, easily verified changes following an
  established pattern.
- **Gemini:** external research, language-mechanism consultation, reduced
  reproductions, and alternative technical approaches.
- **Compiler, tests, fixtures, scripts, and Git:** objective acceptance
  evidence.

These assignments are current routes, not definitions of the task colours.
Change them when measured cost or reliability changes; never lower a task's risk
class to fit a cheaper model.

No agent may approve its own architectural change merely because its tests
pass. One task has one named implementation owner. A second agent may verify,
but may not reimplement the task unless it is formally rejected or reassigned.

## Task classification

### Green

One or two files, an existing pattern, low risk, reversible, and easy to test.
Prefer Cline/DeepSeek. Architecture review is normally unnecessary when the
diff is narrow and every acceptance check passes.

### Amber

Several files or module interactions, moderate implementation judgement, and
explicitly specified behaviour without new semantics. Prefer GitHub Copilot in
an isolated worktree. Use one bounded final architecture review.

### Red

Language or protocol semantics, permissions, capability trust, persistence,
compatibility, concurrency, determinism, or hard-to-reverse architecture.
The architecture reviewer defines the design and invariants first. Copilot may
implement the accepted design; the architecture reviewer signs off the result.

When classification is uncertain, treat the task as the higher-risk colour.

## Standard task handover

Every control-v1 implementation task starts from
`docs/TASK_PACKET_TEMPLATE.md` and should state:

1. Control contract version, state, colour, one owner, and current route.
2. Base branch, base commit, and exact worker-note path.
3. Objective.
4. Relevant background and existing behaviour.
5. Required behaviour.
6. Relevant files and components.
7. Frozen decisions and invariants.
8. Forbidden changes.
9. Acceptance criteria paired with required behaviour.
10. Required verification.
11. Stop conditions.
12. Expected pre-existing changes.

The worker note uses `docs/WORKER_NOTE_TEMPLATE.md` and is part of completion,
not an optional report pasted into chat.

An implementation agent must stop and report when requirements conflict, a
semantic or architectural decision is missing, a safety boundary is unclear,
unrelated failures prevent safe completion, or repeated attempts are not
converging.

## Working sequence

1. Inspect the live repository and Git state.
2. Read the authoritative task-relevant documents.
3. Classify the task.
4. For Amber and Red work, create an isolated worktree when practical.
5. Agree a plan before implementation when the task crosses a trust or module
   boundary.
6. Implement only the accepted scope.
7. Run the formatter, compiler, focused tests, full relevant regression suite,
   integration scripts, and Git whitespace checks.
8. Inspect the complete diff and final Git status.
9. Write the worker note named by the packet, update the task state and short
   dashboard, and report evidence, assumptions, unresolved risks, and the
   smallest next task.
10. Stop when the contract is satisfied. Do not begin cleanup or the next task.
11. Do not commit, push, merge, amend, tag, or open a pull request unless the
    task explicitly authorises it.

## Cline handoff

The normal low-friction Cline entry point is:

```text
/tethers-task.md
```

The project workflow at `.clinerules/workflows/tethers-task.md` reads the
approved contract from `docs/CURRENT_CLINE_TASK.md`, verifies it against the
live Git state, and loads only task-relevant context. The matching project skill
supports natural-language activation. See `docs/CLINE_HANDOFF.md` for the short
Matthew-facing workflow.

Other agents should express implementation plans using the same task-packet
fields. Red work remains `PROPOSED` until its architectural decision is
explicitly approved.

### Task-packet consistency gate

The packet's `Base commit` identifies the implementation checkpoint inspected
when the next task was designed. Before implementation, later commits containing only
`docs/CURRENT_CLINE_TASK.md`, `docs/COPILOT_TRIAL.md`, and
`docs/PROJECT_DASHBOARD.md` may sit above that checkpoint. Requiring the packet to contain its own commit SHA is impossible
and must not be attempted.

Before writing a packet, record the implementation checkpoint and exact dirty
paths. `Expected pre-existing changes` is that snapshot, not a copied list from
an older task. Use exact file paths and write `None` for a clean snapshot.

Every required failure or mismatch branch must map to an acceptance criterion
and focused check. A representative negative test is not sufficient evidence
for several separately required branches.

Both packet producers and implementation agents run:

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
```

For `PROPOSED` and `READY`, the checker permits only planning-control
commits above the base and compares live non-planning dirty paths with the
packet's expected pre-work list. For later states it allows implementation
changes, validates the control contract, and requires the named worker note for
`BLOCKED`, `COMPLETE`, `ACCEPTED`, and `REJECTED`.

## Low-Codex continuation loop

After Cline finishes, Matthew normally runs this Copilot workspace prompt:

```text
/next-tethers-task
```

Copilot independently checks the completed increment and the live repository,
then writes one `PROPOSED` packet for the smallest correction or next task.
Matthew should not need to paste a technical report unless the live evidence is
incomplete.

The prompt ends with one route:

- continue with Cline for a proposed Green task;
- use Copilot for a reviewed Amber implementation;
- request a Codex milestone review when a milestone gate is reached.

Codex is not required after every Green increment. It is required for Red work,
before publishing a meaningful milestone, after three accepted or corrected
increments since the cadence baseline, when agents disagree, or when tests and
Git cannot establish the truth.

When any milestone gate is reached, Copilot must stop and tell Matthew:

```text
Please ask Codex to sign off the current Tethers milestone before continuing.
```

It must also provide the standard copy-ready Codex request and must not prepare
or authorise another implementation until Codex records `SIGNED OFF` or
`NOT SIGNED OFF`.

## Copilot trial

Evaluate Copilot across representative work:

1. repository comprehension with no code changes;
2. a bounded two-or-three-file feature;
3. a contained implementation with one real language-level difficulty;
4. a reproduced bug plus regression test;
5. an autonomous worktree task with a complete evidence report.

For each accepted task record:

- task colour and model;
- first-pass correctness;
- files and lines changed;
- verification completed;
- number of correction loops;
- architecture-review time required;
- included or additional AI usage;
- assumptions or scope escapes;
- accepted, corrected, or rejected outcome.

The useful metric is total agent usage plus architecture-review effort per
accepted change, not how impressive one response sounds.

Review routing after approximately ten real tasks. Keep or change the workflow
based on evidence.

Record trial evidence in `docs/COPILOT_TRIAL.md`.

## Cost and safety posture

- Prefer Auto or a lower-cost capable model for Green and ordinary Amber work.
- Reserve frontier reasoning models for Red work and failed or ambiguous
  implementations.
- Use worktree isolation for background changes.
- Keep terminal and consequential tool approval at the default prompt boundary;
  do not broadly auto-approve shell commands.
- Do not enable additional paid usage merely to avoid a task stopping. Change
  budgets only by Matthew's explicit decision.
- Do not let subagents multiply work or usage unless the task benefits from
  independently verifiable parallel work.
- After two materially similar failures, stop and return exact evidence plus one
  smallest unresolved question.
- Do not use Matthew as the transport for technical handovers. Put the packet,
  note, dashboard state, and evidence references in the repository.
