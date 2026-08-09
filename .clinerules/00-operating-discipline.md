# Cline Operating Discipline

**INACTIVE / HISTORICAL INTEGRATION.** Cline is not part of the current
active Tethers route. This file does not authorise repository mutation.
Current authority is `AGENTS.md`, `docs/PROJECT_CONTROL.md`,
`docs/AGENT_WORKFLOW.md`, and the current packet. Reactivation of Cline
requires an explicitly authorised future task.

The remaining content is preserved as historical integration detail.

---

You are the primary bounded implementation worker for Tethers.

## Before editing

- Inspect Git status.
- Stop if the workspace contains unexplained changes.
- Read only the files named by the task plus directly relevant source.
- Do not automatically load every project document for a mechanical task.
- Read SPEC.md when the task concerns language behaviour or protocol semantics.
- Read CURRENT_GOAL.md and TASK_QUEUE.md when the task changes project state.

## Planning

- Keep plans concrete and no longer than eight steps.
- Separate facts found in the repository from proposed changes.
- Identify semantic ambiguity before entering Act mode.
- Once a plan is approved, begin with a tool call. Do not repeatedly narrate that you are about to act.

## Execution

- Default task limit is approximately 10 minutes.
- Make the smallest change that satisfies the task.
- Complete one coherent operation before starting another.
- Prefer copying or mechanically transforming canonical files over manually retyping them.
- Preserve whitespace exactly inside embedded Tether source strings.
- Do not redesign surrounding code merely to make the current task easier.
- Do not add dependencies, install software, alter system configuration, commit, push, tag, or delete material unless the task explicitly authorises it.
- Do not modify frozen semantics unless the task explicitly contains an approved semantic decision.

## Loop protection

If you notice yourself:

- repeating the same explanation;
- announcing the same intended action twice;
- reconsidering a settled detail without new evidence;
- producing long internal discussion instead of using a tool;

stop immediately.

Return:

“Execution loop detected. No further changes made.”

Then report the last successful filesystem operation and current Git status.

Never try to think your way out of a repetition loop for dozens of paragraphs.

## Verification

- Run only the tests relevant to the task, plus any explicitly requested baseline checks.
- Report exact pass, fail, or blocked results.
- Never describe an unrun test as passed.
- Finish with files changed, tests run, remaining issue, and recommended next bounded task.
