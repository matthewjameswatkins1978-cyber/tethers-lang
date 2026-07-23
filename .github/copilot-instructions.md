# Tethers Copilot instructions

Treat `AGENTS.md` as the always-on project authority. Read every document it
requires before changing code, then read the task-relevant implementation and
tests.

Also follow `docs/AGENT_WORKFLOW.md`:

- classify the task Green, Amber, or Red before implementation;
- use Git, code, and tests as truth rather than relying on handover claims;
- preserve deterministic planning and the host permission boundary;
- keep Tethers application-agnostic;
- make the smallest coherent change;
- do not invent missing semantics;
- stop when an architectural or safety decision is required;
- run and report exact verification;
- inspect the complete diff and final Git status;
- never commit, push, merge, amend, tag, or open a pull request unless the
  current task explicitly authorises it.

Use PowerShell 7 for Windows automation. Do not substitute WSL, Docker, Bash,
or `jq` for the native verified workflows.

Final reports must list the outcome, files changed, design choices, commands
and tests run, exact results, assumptions, unresolved risks, and the smallest
next task.

## Low-Codex continuation loop

Matthew is deliberately conserving Codex usage. For ordinary completed Cline
work, use the workspace prompt `/next-tethers-task` instead of asking him to
carry a technical handover between agents.

That prompt must independently inspect the repository and Git before trusting
Cline's report. It may update only `docs/CURRENT_CLINE_TASK.md` and the factual
evidence log in `docs/COPILOT_TRIAL.md`; it must not repair or implement code.

Do not prepare a next task while Cline is still working. A completed task should
lead to one bounded `PROPOSED` packet and one plain routing verdict.

Route to Codex for Red work, before publishing a meaningful milestone, after
three accepted or corrected increments since the recorded cadence baseline,
when agents disagree, or when tests and Git cannot establish the truth.
Otherwise route Green work back to Cline and suitable reviewed Amber work to
Copilot.

When a Codex milestone review is due, stop the continuation loop. Explicitly
tell Matthew to ask Codex for milestone sign-off, include the standard
copy-ready sign-off request from `/next-tethers-task`, and do not authorise or
start another implementation until Codex records its verdict.

## Task-packet consistency

Before authoring a next-task packet, capture:

- the current implementation checkpoint from `git rev-parse HEAD`;
- the exact pre-existing dirty paths from `git status --short`.

Use that implementation checkpoint as `Base commit`. If the packet and trial
log are later committed as a planning-only commit, do not replace the base with
the packet commit: a committed file cannot contain its own commit SHA. The base
may be behind `HEAD` only when every intervening path is
`docs/CURRENT_CLINE_TASK.md` or `docs/COPILOT_TRIAL.md`.

`Expected pre-existing changes` must reproduce the captured dirty paths
exactly, using file paths rather than directory shorthand. Write `None` when
the snapshot was clean. Do not copy a stale list from the previous packet.

Every failure or mismatch named under Required behaviour must have a matching
acceptance criterion and focused verification. Do not weaken several required
branches into “at least one representative branch.”

Run this after writing a packet and again before handing it off:

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
```
