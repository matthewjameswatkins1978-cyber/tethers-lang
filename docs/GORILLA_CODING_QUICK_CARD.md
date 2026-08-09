# Gorilla Coding Quick Card 🦄

Keep this beside the computer. The normal team is **Lucy + OpenCode**, with **Codex** used for Red work, machine problems, Git recovery, or when OpenCode cannot proceed cleanly.

## 1. Before Starting Anything

Your local VS Code folder does **not** automatically receive changes made on GitHub. Pull the latest repository state before starting a new job, but never pull blindly over local work.

Safest agent instruction:

```text
Update the local Tethers checkout from GitHub safely.

First inspect the current branch and Git status. Preserve every local change.
If the checkout is clean and on main, fetch origin and run git pull --ff-only.
If it is dirty, diverged, or not on main, do not stash, reset, switch branches,
or pull. Return the exact Git state and stop.
```

You may give that to OpenCode for ordinary work or Codex when Git is already confused.

## 2. Start The Next Job

Tell Lucy:

```text
Inspect Tethers and prepare the next job.
```

Lucy will inspect GitHub, decide the risk, compile one bounded task, and tell you whether it goes to OpenCode or Codex.

For an OpenCode task:

1. Pull the latest GitHub state safely.
2. Open the Tethers workspace in VS Code.
3. Open OpenCode and give it the task.

Do not give OpenCode several jobs at once. One task, one owner, one finish.

## 3. While The Job Is Running

Let OpenCode solve ordinary compiler, type, ownership, formatting, and test problems inside the agreed task.

Stop and return to Lucy when:

- OpenCode says a design, permission, trust, or compatibility decision is missing;
- the same kind of failure happens twice;
- OpenCode starts widening the task or redesigning neighbouring systems;
- Git, the environment, or the local machine becomes the actual problem;
- the task is Red or Lucy has asked for Codex.

Do not let OpenCode and Codex edit the same checkout or task at the same time.

## 4. Stop Properly

A proper stop is either `COMPLETE` or `BLOCKED`.

The worker should:

- stop after the authorised task;
- run the required checks;
- inspect the full diff and Git status;
- write the named worker note;
- mark the task `COMPLETE` or `BLOCKED`;
- not invent or begin the next task.

Do not ask it to "keep going while it is there." That is how one bounded job grows antlers.

## 5. What To Give Lucy

Paste the worker's final report to Lucy. It should contain:

- `COMPLETE` or `BLOCKED`;
- files changed;
- important implementation choices;
- commands and tests actually run, with exact results;
- anything not run;
- errors, remaining risks, or the smallest blocker;
- worker-note path;
- final Git status;
- pushed commit or branch reference, when available.

When something goes wrong, include the exact error text and what the worker already tried. Screenshots are useful when the terminal or UI itself is the problem.

Lucy will inspect GitHub and reply with exactly one route:

1. accepted;
2. one bounded correction for OpenCode;
3. escalate to Codex.

## The Rule To Remember

```text
Lucy chooses and reviews.
OpenCode normally builds.
Codex enters for Red work or real trouble.
Matthew routes the message, not the architecture.
```

Few participants. Short supply lines. Strong evidence. No ceremonial paperwork jungle.
