# Git, Worktrees and Line Endings for Tethers Agents

## 1. Purpose and authority

This guide helps Tethers agents work safely with Git, multiple worktrees, and
Windows/Unix text files. It is a practical field guide, not a complete manual.

The guide **supports** task packets and `docs/PROJECT_CONTROL.md`. It does not
replace explicit task instructions, override a task's authority, or authorise
history rewriting. When a task packet gives a specific Git command or branch
rule, the packet wins.

Read it before:

- creating, renaming, or deleting branches;
- pushing, fetching, or comparing remote refs;
- diagnosing line-ending or encoding problems;
- using rebase, reset, cherry-pick, stash, clean, or reflog;
- cleaning up another worktree.

You do not need the full guide for an ordinary source-code commit on a clean
branch inside a routine task. Use it when the Git terrain is unfamiliar.

## 2. A compact mental model

| Concept | What it is |
|---|---|
| **Repository** | The shared object database (`.git/`). All worktrees share it. |
| **Worktree** | One checked-out working directory with its own branch, HEAD, index, and ignored directories. |
| **Branch** | A named pointer that moves with commits. Local only unless pushed. |
| **HEAD** | Where you are now. Usually a branch name; sometimes a detached commit. |
| **Index / staging area** | What `git add` fills. The next commit's proposed snapshot. |
| **Working tree** | The files you edit. Not yet staged. |
| **Local branch** | A branch in your local repository. May differ from its remote-tracking counterpart. |
| **Remote-tracking ref** (`origin/main`) | A local snapshot of what the remote had the last time you fetched. |
| **Commit range** (`base..HEAD`) | Commits reachable from HEAD but not from base. |

Several worktrees share one Git history but have **separate** checked-out files,
separate branches, and separate ignored directories (`_opam`, `target`, caches).
An ignored directory that exists in one worktree is absent from another by
design, not by error.

## 3. Tethers preflight

Every task should confirm where it is before editing. The following PowerShell
preflight is reusable across Tethers tasks:

```powershell
git rev-parse --show-toplevel         # Repository root — must match the task packet
git branch --show-current             # Branch — must match the task packet
git status --short --branch           # Dirty files and ahead/behind — must be clean unless expected
git fetch origin --prune              # Refresh remote-tracking refs
git rev-parse HEAD                    # Exact commit where you stand
git rev-parse origin/main             # What the remote main currently is
git merge-base HEAD origin/main       # The common ancestor
git rev-list --count origin/main..HEAD  # Commits ahead of origin/main
git rev-list --count HEAD..origin/main  # Commits behind origin/main
git worktree list --porcelain         # All worktrees sharing this repository
```

| Result | What it tells you |
|---|---|
| `show-toplevel` | Which repository you are in. Compare with the task packet. |
| `show-current` | Which branch is checked out. |
| `status --short --branch` | Dirty files and divergence. Clean expected; dirty must be declared. |
| `origin/main` after fetch | Stale local `main` is not evidence about the remote. |
| `merge-base` equals `origin/main` | Your branch sits cleanly on top of main. |
| Ahead > 1, Behind > 0, merge-base mismatch | Evidence of unexpected history. Stop. |

Do not assume every task must compare against `origin/main`. When the task
packet names a different base, use that base instead.

## 4. Worktree ownership

One task has one implementation owner. One worktree belongs to one task at a
time.

Important realities of Tethers' multi-worktree setup:

- **Ignored directories are not shared.** `_opam`, `target`, Rust build output,
  and caches may exist in only one checkout. A missing `_opam` is normal for a
  worktree that is not the OCaml switch owner.
- **An external toolchain path does not move the source being built.** The OCaml
  switch path selects the compiler and libraries. The current worktree remains
  the source tree. Do not accidentally build the switch owner's checkout.
- **Unrelated dirty files belong to their existing owner.** The original
  worktree at `D:\The Next Thing\Tethers Lang` has its own branch, its own
  `docs/TETHERS_LUCY_NOTES.md` modification, and its own task. Never stash,
  restore, reset, commit, or clean that worktree.
- **Do not "tidy up" another worktree** merely because it looks untidy. Its
  dirty state is evidence for its owning task.

## 5. Windows and Unix line endings

### What they are

- **LF** (`\n`, `0x0A`): Unix, Linux, macOS native line ending.
- **CRLF** (`\r\n`, `0x0D 0x0A`): Windows native line ending.

### How Git handles them

Git can store one representation in the object database and present another in
the working tree. The key setting is `core.autocrlf`:

- `true` (current Tethers system default): Git stores LF in the index/object
  database and writes CRLF in the working tree on Windows.
- `input`: Git stores LF and writes LF. Useful for cross-platform repositories.
- `false`: Git stores whatever the file contains without conversion.

The index (`i/`) and working-tree (`w/`) columns from `git ls-files --eol` tell
you the actual representations:

```
i/lf    w/crlf  attr/          AGENTS.md
```

This means: the committed blob is LF, and your working-tree copy is CRLF. Git
converts on checkout. This is normal and intentional.

### Why this matters

A two-line edit can appear as a **whole-file rewrite** when:

- An editor changes every line ending (LF→CRLF or CRLF→LF).
- A tool strips trailing whitespace across the file.
- An encoding conversion adds or removes a UTF-8 BOM.
- Git's `text` attribute or a `.gitattributes` change triggers renormalisation.

**Line-ending policy is distinct from whitespace policy.** A file with correct
line endings can still have trailing whitespace. `git diff --check` catches
trailing whitespace and-conflict markers; it does not report line-ending
differences.

### Do not blindly convert

Before changing a file, inspect its current convention with `git ls-files --eol`.
Preserve the existing encoding and line-ending convention. Do not impose a
personal preference. Do not run `git add --renormalize` without explicit
authorisation.

## 6. Diagnosing line-ending and encoding floods

When every line appears changed, use these commands before editing further:

```powershell
git ls-files --eol -- <path>                      # Index vs working-tree EOL
git diff -- <path>                                 # Full diff
git diff --ignore-space-at-eol -- <path>           # Mask trailing whitespace
git diff --word-diff=porcelain -- <path>           # Word-by-word; reveals real changes
git diff --numstat -- <path>                       # Added/deleted line counts
git diff --check                                    # Trailing whitespace and conflict markers
```

Check for these specific causes:

| Symptom to check | How to detect |
|---|---|
| Whole-file EOL rewrite | Every line shows `+`/`-` in `git diff`. `--word-diff=porcelain` shows no content change. |
| UTF-8 BOM added or removed | First line shows `~` at position 0 in word-diff. `Format-Hex <path> \| Select -First 4` in PowerShell. |
| Encoding conversion (e.g., Latin-1→UTF-8) | Non-ASCII characters change. `--word-diff=porcelain` shows isolated character changes. |
| Trailing whitespace stripped | `git diff --check` shows many lines; `git diff --ignore-space-at-eol` is clean. |
| Tabs→spaces or spaces→tabs | Whitespace-only changes. `git diff -w` is clean. |
| Final newline added or removed | `\ No newline at end of file` in diff. |
| EOL change inside fixtures or source strings | Only those lines change. Distinguish from whole-file conversion. |

Use `Format-Hex` in PowerShell for focused encoding inspection. Do not dump or
rewrite whole files casually.

## 7. Working-tree, staged, and range diffs

These four `git diff` variants answer different questions:

```powershell
git diff                     # Unstaged edits — what you changed but haven't staged
git diff --cached            # Staged edits — what will go into the next commit
git diff <base>..HEAD        # Range diff — everything committed on this branch since base
git diff --check             # Whitespace check on unstaged edits
git diff --check <base>..HEAD  # Whitespace check on the whole branch
```

**Important lesson from Tethers practice:** A clean working tree can coexist
with a failing range check. The defect is already committed on the branch.
Run `git diff --check <base>..HEAD` before publication, not only `git diff
--check`.

## 8. Safe small edits

When changing an existing file:

1. **Preserve the existing encoding and EOL convention.** Inspect with
   `git ls-files --eol` before editing.
2. **Make the narrowest edit.** Change only what the task requires.
3. **Inspect the diff early.** Run `git diff` after each logical change.
4. **Run `git diff --check` early**, not only at publication time.
5. **Stop when a tiny edit becomes a whole-file rewrite.** Something changed
   the encoding or line endings. Diagnose it before continuing.
6. **Prefer targeted edits over whole-file replacement APIs.** A surgical
   `replace` preserves surrounding bytes. A wholesale `write_file` may
   introduce invisible encoding changes.

These are strong recommendations for normal task work. A recovery task may
legitimately need bulk operations.

## 9. Commits and evidence

Before committing:

1. **Stage only authorised files.** Use `git add -- <path>` for each file.
2. **Inspect `git diff --cached`.** Know exactly what you are committing.
3. **Use a task-specific commit message.** The first line should identify
   the task.
4. **Record the exact commit SHA** with `git rev-parse HEAD`.
5. **Distinguish local commit from pushed branch.** A local commit exists
   only in your repository until pushed.
6. **Report unrun checks honestly.** "NOT RUN" is evidence. Silence is not.

After committing, verify with `git status --short --branch` and
`git show --stat --oneline HEAD`.

## 10. Safe branch publication

The normal clean fast-forward pattern for Tethers:

1. **Fetch** to get current remote state.
2. **Verify the exact base** and its relationship to your branch (merge-base,
   ahead/behind counts).
3. **Run range checks**: `git diff --check <base>..HEAD`.
4. **Push the branch for review**: `git push -u origin <branch>`.
5. **Publish an exact accepted SHA to main only when explicitly authorised**:
   `git push origin <sha>:refs/heads/main`.
6. **Verify `origin/main` afterward** with `git fetch origin; git rev-parse
   origin/main`.

**Stale local `main` is not evidence** about public `origin/main`. Always fetch
before comparing or publishing. Your local `main` branch may be months behind
the remote and is irrelevant for publication decisions.

## 11. Recovery tools without handcuffs

These are legitimate Git tools. Context and authority determine whether they are
safe:

| Tool | What it does | Dangerous when |
|---|---|---|
| `rebase` | Replays commits onto a new base | Accepted commits are rewritten; another agent's branch changes under them |
| `cherry-pick` | Copies one commit to another branch | Used to merge tasks without review |
| `reset` | Moves HEAD and optionally the index/working tree | Destroys uncommitted work or rewrites shared history |
| `restore` | Discards working-tree or index changes | Destroys dirty work belonging to a task |
| `stash` | Temporarily shelves dirty changes | Shelves another agent's work or is forgotten |
| `clean` | Removes untracked files | Deletes generated artifacts, config files, or diagnostic evidence |
| `reflog` | Browses the local reference history | Execution — reading is always safe; `reset --hard` from reflog dates is not |
| `force push` | Overwrites a remote branch | Destroys accepted commits or another agent's reviewed branch |

None of these tools is banned categorically. They become dangerous when they
can rewrite accepted commits, hide dirty work, move another agent's changes,
or destroy untracked files. An explicit recovery task may authorise them after
exact evidence is captured. A routine implementation task should not use them
unless the packet explicitly permits it.

## 12. Stop conditions

Stop and report evidence when:

- **Wrong worktree.** `git rev-parse --show-toplevel` does not match the
  task packet.
- **Unexpected dirty files.** `git status --short` shows files the task packet
  did not declare.
- **Base mismatch or divergence.** `merge-base` is not the expected base, or
  ahead/behind counts are wrong.
- **Unexplained whole-file rewrite.** A small edit produces a huge diff.
  Diagnose before continuing.
- **Unexpected encoding change.** `git ls-files --eol` shows a different
  convention after the edit.
- **Branch already exists** with different history. Do not overwrite another
  agent's branch.
- **Push requires force** when only fast-forward was authorised.
- **Task needs Git policy or global configuration changes.** Those are
  separate repository decisions.
- **Two materially similar repair attempts fail.** Record exact evidence and
  the smallest unresolved question.

## 13. Diagnostic table

| Symptom | Likely cause | First check |
|---|---|---|
| Every line appears changed | Line-ending or encoding conversion | `git ls-files --eol -- <path>`; `git diff --word-diff=porcelain` |
| Working tree clean but range check fails | Defect already committed on branch | `git diff --check <base>..HEAD` |
| `_opam` or `target` missing | Not the worktree that owns those ignored directories | `git worktree list --porcelain` |
| Local `main` looks old | Not fetched; local `main` is not `origin/main` | `git fetch origin; git rev-parse origin/main` |
| Branch unexpectedly ahead or behind | Wrong base, missing fetch, or another agent pushed | `git fetch origin --prune; git merge-base HEAD origin/main` |
| Push rejected | Branch history diverged or force required | `git fetch origin; git rev-list --count HEAD..origin/<branch>` |
| Tiny edit produces huge diff | Editor or tool changed line endings, encoding, or whitespace | `git diff --word-diff=porcelain`; `git diff -w` |
| Whitespace check fails | Trailing whitespace or conflict markers | `git diff --check`; `git diff --check <base>..HEAD` |
| File behaves differently on Windows and Unix | CRLF vs LF; `core.autocrlf` conversion | `git ls-files --eol -- <path>`; `git config --get core.autocrlf` |
| Untracked files would be removed by cleanup | Generated artifacts, diagnostic files, or evidence in working tree | `git status --short`; `git clean -n` (dry-run only) |

## 14. Pocket command list

```powershell
# Where am I?
git rev-parse --show-toplevel
git branch --show-current
git status --short --branch
git worktree list --porcelain

# What changed?
git diff                      # unstaged
git diff --cached             # staged
git diff <base>..HEAD         # branch range
git diff --check              # whitespace
git diff --check <base>..HEAD
git diff --word-diff=porcelain -- <path>

# Line endings and encoding
git ls-files --eol -- <path>
git config --get core.autocrlf

# Remote state
git fetch origin --prune
git rev-parse origin/main
git merge-base HEAD origin/main
git rev-list --count origin/main..HEAD
git rev-list --count HEAD..origin/main

# Publishing
git push -u origin <branch>
git push origin <sha>:refs/heads/main
git fetch origin; git rev-parse origin/main
```
