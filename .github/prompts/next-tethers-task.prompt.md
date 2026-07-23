---
name: next-tethers-task
description: Verify Cline's completed Tethers increment and prepare the next bounded task
argument-hint: Optional Cline report or note
agent: agent
---

Prepare the next Tethers task with minimal involvement from Matthew or Codex.

First read:

- [project instructions](../copilot-instructions.md)
- [agent workflow](../../docs/AGENT_WORKFLOW.md)
- [current Cline task](../../docs/CURRENT_CLINE_TASK.md)
- [current goal](../../docs/CURRENT_GOAL.md)
- [task queue](../../docs/TASK_QUEUE.md)
- [Copilot trial and milestone counter](../../docs/COPILOT_TRIAL.md)

Then independently inspect the live repository, complete diff, Git status, and
the implementation and tests relevant to the current task. Treat any pasted
Cline report as a claim to verify, not as the source of truth.

Before editing planning documents, save the implementation checkpoint from
`git rev-parse HEAD` and the exact pre-existing dirty paths from
`git status --short`. These values control the new packet; never copy them from
the previous packet.

If Cline appears to be working, the current packet remains `READY`, or the
implementation report is incomplete, do not edit anything. Say exactly what
evidence is still needed and stop.

When the increment is complete:

1. Check scope, behaviour, trust boundaries, failure handling, documentation,
   verification evidence, and unrelated working-tree changes.
2. If there is a concrete defect, prepare only the smallest correction.
3. Otherwise select the next smallest coherent increment from the current goal
   and task queue.
4. Classify it Green, Amber, or Red. When uncertain, choose the higher risk.
5. Write the correction or next task to `docs/CURRENT_CLINE_TASK.md` with status
   `PROPOSED`, using the existing field order and including acceptance criteria,
   verification, exclusions, pre-existing changes, and stop conditions.
   Use the saved implementation checkpoint as `Base commit`. If a later
   planning-only commit contains this packet, leave the base pointing to the
   implementation checkpoint; do not attempt to put a commit's own SHA inside
   itself.
6. Add one factual row to `docs/COPILOT_TRIAL.md` only when the completed task
   can be classified as accepted, corrected, or rejected. Do not estimate
   unavailable usage.
7. Check that every failure or mismatch named under Required behaviour has a
   corresponding acceptance criterion and focused verification. “At least one”
   is insufficient when several branches are required.
8. Run
   `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`.
   Correct every failure before handoff.
9. Do not implement the proposed task. Do not commit or push.

Require a Codex milestone review immediately when the completed or proposed
work involves language or protocol semantics, permissions, capability trust,
durability, persistence, compatibility, concurrency, deterministic behaviour,
or a disputed architectural choice. Also require it before publishing a
meaningful milestone, after three accepted or corrected increments since the
cadence baseline, when Cline and Copilot disagree, or when verification cannot
establish the truth.

When a milestone review is due:

1. Do not prepare or authorise another implementation task.
2. Leave `docs/CURRENT_CLINE_TASK.md` as `COMPLETE` or set it to `BLOCKED`;
   never mark the next task `READY`.
3. Tell Matthew plainly: `Please ask Codex to sign off the current Tethers
   milestone before continuing.`
4. Give him this exact copy-ready message:

   `Sign off the current Tethers milestone. Independently inspect the live
   repository, complete diff, verification evidence, authoritative docs and Git
   state. Return SIGNED OFF or NOT SIGNED OFF, record the milestone baseline if
   signed off, and do not push or begin the next task.`

End with exactly one routing line:

- `ROUTE: CONTINUE WITH CLINE` when Matthew may approve the proposed Green task;
- `ROUTE: COPILOT IMPLEMENTATION` for a reviewed Amber task better suited to
  Copilot in an isolated worktree; or
- `ROUTE: CODEX MILESTONE REVIEW DUE` when any milestone condition applies.

Keep the explanation short and tell Matthew only what was achieved, whether
anything is genuinely wrong, and the next necessary action.
