# Current Implementation Task

Control contract: `1`

Task: `GIT-GUIDE-01 — Git, Worktrees and Line Endings for Tethers Agents`

Status: `COMPLETE`

Task colour: `Amber`

Owner: `Goose`

Route: `Goose Medium — bounded operational documentation`

Worker note: `docs/worker-notes/2026-07-30-git-worktrees-line-endings-guide.md`

Base branch: `main`

Base commit: `8a70a8f47ad8cf110e9987b283f80277705b2292`

Branch: `goose/git-worktrees-line-endings-guide`

## Objective

Create a small practical field guide at
`docs/GIT_WORKTREES_AND_LINE_ENDINGS_FOR_AGENTS.md` that helps Tethers agents
use Git, worktrees and Windows/Unix text files safely. The guide gives agents
useful judgement and diagnostic techniques without turning Git into rigid
prohibitions.

## Relevant background and existing behaviour

Tethers uses multiple Git worktrees on native Windows. The `_opam`, `target`,
and generated directories are ignored and may exist in only one checkout.
The OCaml and Rust guides already contain worktree and toolchain rules. The
current repository has `core.autocrlf=true` (system-level) and index objects
are stored as LF with working-tree CRLF for at least some tracked files.

AGENTS.md already requires the OCaml and Rust guides before the first edit in
those languages. This task adds a narrow Git rule without weakening the
existing language-guide requirements.

## Required behaviour

1. Create `docs/GIT_WORKTREES_AND_LINE_ENDINGS_FOR_AGENTS.md` — a practical
   field guide of approximately 1,500–2,500 words with the fourteen sections
   specified in the task.
2. Add a narrow AGENTS.md rule: for tasks involving Git topology, branch
   publication, worktrees, line-ending/encoding investigation, history
   recovery, or destructive Git commands, read the new guide before the first
   Git mutation.
3. Append a concise dated decision to `docs/DECISIONS.md` recording the
   canonical guide, its advisory role, the no-new-policy rule, and
   preservation of destructive-tool availability under explicit authority.
4. Replace `docs/CURRENT_CLINE_TASK.md` with this task packet, setting status
   to `COMPLETE` only after all acceptance evidence exists.
5. Write the worker note at
   `docs/worker-notes/2026-07-30-git-worktrees-line-endings-guide.md` with
   exact evidence.

## Relevant components

- `docs/GIT_WORKTREES_AND_LINE_ENDINGS_FOR_AGENTS.md` — new canonical guide
- `AGENTS.md` — narrow required-reading addition
- `docs/DECISIONS.md` — concise decision record
- `docs/CURRENT_CLINE_TASK.md` — task packet
- `docs/worker-notes/2026-07-30-git-worktrees-line-endings-guide.md` — evidence

## Frozen decisions and invariants

- The guide assists rather than replaces explicit task instructions.
- The guide introduces no `.gitattributes`, `.editorconfig`, editor policy,
  EOL policy, or global Git configuration.
- Destructive or history-rewriting operations remain available under explicit
  task or recovery authority.
- Unrelated dirty work must be preserved; never clean up another worktree.
- This task does not implement TOOLCHAIN-BASELINE-01 or begin J13C.
- No production Rust, OCaml, fixtures, scripts, dependencies, or toolchain
  files are changed.
- Line-ending policy is observational only; do not convert or renormalise.

## Acceptance criteria

1. The branch began from exact base `8a70a8f47ad8cf110e9987b283f80277705b2292`.
2. The guide exists at `docs/GIT_WORKTREES_AND_LINE_ENDINGS_FOR_AGENTS.md`, is
   practical, Tethers-specific, and between approximately 1,500 and 2,500 words.
3. The guide explains LF/CRLF, worktrees, diff layers, and clean publication
   accurately.
4. The guide provides diagnostic help without banning legitimate advanced Git
   tools.
5. The guide introduces no new repository or machine Git policy.
6. `AGENTS.md` uses the guide only for relevant Git/worktree tasks.
7. `docs/DECISIONS.md` records the bounded guide decision.
8. The current task packet and worker note contain exact evidence.
9. Packet checker and whitespace checks pass.
10. Only the five authorised files changed:
    - `AGENTS.md`
    - `docs/CURRENT_CLINE_TASK.md`
    - `docs/DECISIONS.md`
    - `docs/GIT_WORKTREES_AND_LINE_ENDINGS_FOR_AGENTS.md`
    - `docs/worker-notes/2026-07-30-git-worktrees-line-endings-guide.md`
11. The branch is pushed for review; main is untouched.
12. The Goose worktree is clean after commit and push.
13. The original worktree (`D:\The Next Thing\Tethers Lang`) and its
    `docs/TETHERS_LUCY_NOTES.md` modification remain untouched.

## Required verification

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File .\.github\scripts\check-tethers-task-packet.ps1

# Word count (after writing guide)
(Get-Content docs/GIT_WORKTREES_AND_LINE_ENDINGS_FOR_AGENTS.md | Out-String).Split(" ", [System.StringSplitOptions]::RemoveEmptyEntries).Count

git diff --check
git diff --stat
git diff
git status --short --branch

# Before committing, inspect each authorised file
git diff -- AGENTS.md
git diff -- docs/DECISIONS.md
git diff -- docs/CURRENT_CLINE_TASK.md
git diff -- docs/GIT_WORKTREES_AND_LINE_ENDINGS_FOR_AGENTS.md
git diff -- docs/worker-notes/2026-07-30-git-worktrees-line-endings-guide.md

# Cached diff checks
git diff --cached --check
git diff --cached --stat
git diff --cached

# After commit
git status --short --branch
git show --stat --oneline HEAD
git diff --check 8a70a8f47ad8cf110e9987b283f80277705b2292..HEAD
```

## Forbidden changes

- No `.gitattributes`, `.editorconfig`, or Git configuration changes.
- No `core.autocrlf`, `core.eol`, `core.safecrlf` modification.
- No `git add --renormalize`, `git reset --hard`, `git clean`, stash of
  another worktree, rebase, cherry-pick, amend, or force-push.
- No production code, test, fixture, or script changes.
- No TOOLCHAIN-BASELINE-01 or J13C work.
- No merge or push to main.
- No branch or worktree deletion.

## Stop conditions

Stop when: origin/main is not the exact base; the Goose worktree is dirty
before branch creation; the branch name already exists with unexpected history;
an existing file suffers an unexplained line-ending or encoding flood; accurate
guidance would require imposing a new Git or EOL policy; the original dirty
worktree changes; two materially similar correction attempts fail.

## Expected pre-existing changes

None. Starting from clean `main` at `8a70a8f47ad8cf110e9987b283f80277705b2292`.
