# Project Workflow

## Authority

Follow, in order:

1. `AGENTS.md`
2. `docs/PROJECT_CONTROL.md`
3. `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md`
4. `docs/CURRENT_CLINE_TASK.md`
5. task-named specifications, decisions, code, tests, and worker notes

The task packet controls scope. The implementation language standard controls
how production code is expressed. A narrow task does not require primitive code.

## Roles And Routing

Roles are responsibilities, not permanent model assignments.

- Matthew owns product direction, consequential trade-offs, installation,
  publication, and irreversible actions.
- Lucy in ordinary chat or Codex may compile tasks, make semantic decisions,
  inspect repository evidence, and perform independent review.
- One named implementation owner performs the bounded change, verification, and
  worker note.
- A separate verifier is required where the task risk demands it.

Current economical routing:

- Green: Cline/DeepSeek or another reliable low-cost worker.
- Amber: a repository-aware worker such as Copilot, normally isolated.
- Red: Lucy/Codex freezes the design and signs off; the most suitable worker may
  implement.
- Direct machine, Git recovery, environment, or difficult local diagnosis:
  Codex or another explicitly authorised computer-capable worker.

Change routing when measured reliability, cost, or availability changes. Never
lower risk classification to fit a cheaper worker.

## Coordination Rules

- One task has one implementation owner.
- Two workers must not edit the same task or checkout simultaneously.
- No worker approves its own Red architectural change.
- Commit, push, merge, amend, tag, or publication requires explicit packet
  authority.
- A worker gets one focused correction after a concrete finding. Do not restart
  the original task or enter an open-ended repair loop.
- Tests, fixtures, compiler output, Trail evidence, and Git establish the result;
  model confidence does not.

## Task Construction

Lucy or another authorised task compiler converts:

```text
Matthew's intention
    -> architectural contract
    -> bounded implementation task
    -> observable proof
```

Each packet must identify:

1. one outcome and one owner;
2. risk and current route;
3. base branch, base checkpoint, and expected pre-existing changes;
4. required behaviour and frozen invariants;
5. permitted and forbidden scope;
6. acceptance criteria paired with verification;
7. stop conditions;
8. exact worker-note path.

Give the worker the right context, not the maximum context. Point to relevant
entry files and allow normal code navigation. Do not combine implementation,
independent audit, Git administration, and a retrospective essay into one task.

The ten-minute implementation-step limit is a runaway brake, not a target. Stop
at a coherent recoverable point when the limit or a real blocker is reached.

## Implementation Language

Use OCaml, Rust, PowerShell, JSON, and future implementation languages according
to `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md`.

In particular:

- write idiomatic production code for senior engineers and capable AI;
- use types, modules, ownership, pattern matching, traits, interfaces, and other
  suitable language features to enforce the domain;
- do not simplify code merely so Matthew can read it;
- do not use advanced machinery without a concrete correctness, safety,
  modularity, or maintenance benefit;
- explain unfamiliar but justified technique in the worker note;
- keep product semantics out of PowerShell orchestration;
- preserve deterministic and fail-closed boundaries.

## Normal Work Sequence

1. Verify packet state, owner, route, base, and live Git state.
2. Run the packet consistency checker.
3. Read task-bounded authoritative context.
4. Implement only the authorised scope.
5. Run focused checks during development and proportionate final verification.
6. Inspect the complete diff and final Git status.
7. Write the exact required worker note.
8. Mark the packet `COMPLETE` or `BLOCKED` honestly.
9. Stop. Do not begin cleanup or the next task.

A documentation-only correction does not normally require rebuilding the entire
project. A trust-boundary or cross-language change requires evidence at the
actual boundary.

## Handoff

Use `/tethers-task.md` for an approved ordinary Cline handoff. Its workflow lives
at `.clinerules/workflows/tethers-task.md`; the task contract remains
`docs/CURRENT_CLINE_TASK.md`.

- `PROPOSED` is read-only.
- `READY` authorises only its bounded work.
- `IN_PROGRESS` belongs to the named owner.
- `COMPLETE` and `BLOCKED` require the named worker note.
- `ACCEPTED` and `REJECTED` are verifier outcomes.

A technical handover belongs in the repository, not in Matthew's memory or a
chain of pasted chat summaries.

## Environment

- Primary platform: Windows.
- OCaml: native project-local opam switch.
- Rust host: `tethers-0.1/host-rust/`.
- Required automation shell: PowerShell 7 (`pwsh.exe`).
- Do not substitute Windows PowerShell 5.1, WSL, Docker, Bash, or `jq` for the
  verified workflow merely for convenience.
- Do not move `tethers-0.1/engine-ocaml/`; its local switch is path-bound.

Important paths:

- Specification: `tethers-0.1/SPEC.md`
- Implementation standard: `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md`
- OCaml guide: `docs/OCAML_GUIDE_FOR_AGENTS.md`
- Decisions: `docs/DECISIONS.md`
- Project state: `docs/CURRENT_GOAL.md`, `docs/PROJECT_DASHBOARD.md`
- Task contract: `docs/CURRENT_CLINE_TASK.md`
- Evidence: `docs/worker-notes/`
