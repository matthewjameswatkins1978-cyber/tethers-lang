# Project Workflow

**INACTIVE / HISTORICAL INTEGRATION.** Cline is not part of the current
active Tethers route. This file does not authorise repository mutation.
Current authority is `AGENTS.md`, `docs/PROJECT_CONTROL.md`,
`docs/AGENT_WORKFLOW.md`, and the current packet. Reactivation of Cline
requires an explicitly authorised future task.

The remaining content is preserved as historical integration detail.

---

## Authority

Follow, in order:

1. `AGENTS.md`
2. `docs/PROJECT_CONTROL.md`
3. `docs/AGENT_WORKFLOW.md`
4. `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md`
5. `docs/CURRENT_CLINE_TASK.md`
6. task-named specifications, decisions, code, tests, and worker notes

The packet controls scope. The implementation standard controls how production
code is expressed. A narrow task does not require primitive code.

## Gorilla Coding Roles 🦄

- Matthew owns product direction, consequential trade-offs, installation,
  publication, and irreversible actions.
- Lucy in ordinary chat controls architecture, task compilation, continuation,
  GitHub-visible review, and acceptance.
- Historically Cline was the default implementation owner for ordinary Green and Amber work.
- Codex entered for Red work, difficult local failure, Git/environment/recovery
  work, or machine-required diagnosis.

Copilot is not part of the current route. Do not tell Matthew to open Copilot or
run `/next-tethers-task`.

## Coordination Rules

- One task has one implementation owner.
- Cline and Codex must not edit the same task or checkout simultaneously.
- No worker approves its own Red architectural change.
- Commit, push, merge, amend, tag, installation, or publication requires explicit
  authority.
- After two materially similar failed attempts, stop and return exact evidence.
- Tests, fixtures, compiler output, Trails, and Git establish the result; model
  confidence does not.
- Cline never invents or begins the next task.

## Task Construction

Lucy converts:

```text
Matthew's intention
    -> architectural contract
    -> bounded implementation task
    -> observable proof
```

Each packet identifies:

1. one outcome and one owner;
2. risk and route;
3. base branch, base checkpoint, and expected pre-existing changes;
4. required behaviour and frozen invariants;
5. permitted and forbidden scope;
6. acceptance criteria paired with verification;
7. stop and escalation conditions;
8. exact worker-note path.

Give Cline the right context, not the maximum context. Point to relevant entry
files and allow normal code navigation. Do not combine implementation,
independent audit, Git administration, and a retrospective essay into one task.

The ten-minute step limit is a runaway brake, not a target. Stop at a coherent,
recoverable point when the limit or a real blocker is reached.

## Implementation Language

Use OCaml, Rust, PowerShell, JSON, and future implementation languages according
to `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md`.

- Write idiomatic production code for senior engineers and capable AI.
- Use suitable types, modules, ownership, pattern matching, traits, interfaces,
  and abstractions to enforce the domain.
- Do not simplify code merely so Matthew can read it.
- Do not use advanced machinery without a concrete correctness, safety,
  modularity, or maintenance benefit.
- Explain unfamiliar but justified technique in the report and worker note.
- Keep product semantics out of PowerShell orchestration.
- Preserve deterministic and fail-closed boundaries.

## Normal Work Sequence

1. Verify packet state, owner, route, base, and live Git state.
2. Run the packet consistency checker.
3. Read task-bounded authoritative context.
4. Implement only the authorised scope.
5. Run focused checks during development and proportionate final verification.
6. Inspect the complete diff and final Git status.
7. Write the exact required worker note.
8. Mark the packet `COMPLETE` or `BLOCKED` honestly.
9. Return the concise report defined by `docs/CLINE_HANDOFF.md`.
10. Stop. Do not begin cleanup or the next task.

A documentation-only correction does not normally require rebuilding the whole
project. A trust-boundary or cross-language change requires evidence at the
actual boundary.

## Handoff

Use `/tethers-task.md` for an approved Cline task. The task contract is
`docs/CURRENT_CLINE_TASK.md`.

- `PROPOSED` is read-only.
- `READY` authorises only its bounded work.
- `IN_PROGRESS` belongs to the named owner.
- `COMPLETE` and `BLOCKED` require the named worker note and report.
- `ACCEPTED` and `REJECTED` are Lucy or the required verifier's outcomes.

After completion, tell Matthew to paste the concise report to Lucy. Do not route
to Copilot and do not compile another task.

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
