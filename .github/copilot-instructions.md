# Tethers Copilot Instructions

Treat these as always-on authority:

1. `AGENTS.md`
2. `docs/PROJECT_CONTROL.md`
3. `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md`
4. `docs/AGENT_WORKFLOW.md`
5. `docs/CURRENT_CLINE_TASK.md`
6. task-relevant specifications, decisions, code, tests, and worker notes

Read the active packet and dashboard, then load only context that can affect the
task. Do not read the complete project archive by default.

## Engineering Rules

- Classify work Green, Amber, or Red before implementation.
- Use Git, compiler output, tests, fixtures, and Trail evidence rather than
  trusting handover claims.
- Preserve deterministic planning, fail-closed host policy, and permission
  boundaries.
- Keep Tethers Core application-agnostic.
- Make the smallest coherent change in scope, not the least sophisticated code.
- Use each implementation language idiomatically and to the depth justified by
  the present problem.
- Do not simplify production code merely so Matthew can read it; explain the
  design in the task packet, worker note, or review.
- Do not invent missing semantics, APIs, tests, or completed commands.
- Stop when an architectural, compatibility, permission, or safety decision is
  required.
- Inspect the complete diff and final Git status.
- Never commit, push, merge, amend, tag, publish, or open a pull request unless
  the current task explicitly authorises it.

Use PowerShell 7 for Windows automation. Do not substitute WSL, Docker, Bash,
Windows PowerShell 5.1, or `jq` for the verified native workflow merely for
convenience.

## Task Completion

A final implementation report must state:

- outcome and packet state;
- files changed;
- design choices and invariants preserved;
- commands and tests actually run, with exact results;
- commands or checks not run;
- assumptions and unresolved risks;
- smallest useful next action;
- final Git status.

Create the evidence-backed worker note at the exact path named by a control-v1
packet. A task is not `COMPLETE` without the required note and evidence.

## Continuation And Routing

Ordinary chat Lucy may inspect pushed GitHub state, review implementation,
compile the next task, and make architectural decisions without consuming Codex
computer credits. Use that route whenever repository-visible evidence is enough.

The workspace prompt `/next-tethers-task` is an optional local continuation tool,
not a mandatory stage. Use it when direct checkout inspection materially helps.
It must remain read-only with respect to implementation and may update only the
planning and factual control files authorised by its workflow.

Do not prepare a next task while an implementation owner is still working. A
completed task leads to one bounded review verdict and, when appropriate, one
`PROPOSED` next packet.

Current preferred routing:

- Green: Cline/DeepSeek or another reliable low-cost worker.
- Amber: Copilot or another repository-aware worker, normally isolated.
- Red: Lucy/Codex freezes the design and independently signs off the result.
- Direct machine, Git recovery, environment, or difficult local diagnosis:
  Codex or another explicitly authorised computer-capable worker.

Route based on measured reliability, cost, and task needs. Do not force every
accepted increment through Codex, and do not lower task risk to avoid using it.

## Task-Packet Consistency

Every new packet starts from `docs/TASK_PACKET_TEMPLATE.md` and includes:

- control contract;
- state, colour, owner, and route;
- base branch and implementation checkpoint;
- exact worker-note path;
- `Frozen decisions and invariants`;
- required behaviour paired with acceptance criteria;
- verification and stop conditions;
- exact expected pre-existing changes.

Before authoring a packet, capture the live implementation checkpoint and exact
dirty paths. Do not copy the previous task's list. A planning-only commit may sit
above the implementation base only where the packet checker explicitly permits
it; never attempt to place a file's own future commit SHA inside that file.

Every named failure or mismatch branch requires matching focused verification.
One representative negative test does not prove several independent fail-closed
requirements.

Run before handoff and after the required worker note exists:

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
```
