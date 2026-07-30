# Git, Worktrees and Line Endings for Tethers Agents

Status: required operating guidance for Git, worktree, publication, recovery,
or line-ending work

Primary audience: Goose, Cline, Codex, and reviewers working on native Windows

This is a practical guide, not a new Git policy. It complements the task packet,
`AGENTS.md`, `docs/PROJECT_CONTROL.md`, and `docs/AGENT_WORKFLOW.md`. Those
documents still decide authority, scope, publication, and stop conditions.

The aim is simple: know which checkout is being changed, preserve work that does
not belong to the current task, and distinguish a real content change from a
line-ending or encoding accident. Use normal Git operations when authorised;
do not freeze at the sight of a warning. Equally, do not "fix" a warning by
rewriting a repository-wide setting during a bounded task.

## 1. The first minute: establish location and authority

Before the first edit, run this from the intended worktree in PowerShell 7:

```powershell
git rev-parse --show-toplevel
git branch --show-current
git status -sb
git rev-parse HEAD
git status --short --branch
git fetch origin --prune
git rev-parse origin/main
git merge-base HEAD origin/main
```

Compare the output with the task packet: exact worktree path, branch, expected
base, owner, and known pre-existing changes. `HEAD` says what this checkout
contains. `origin/main` says what the fetched remote main contains. They are
different facts; never substitute one for the other.

`git branch --show-current` is intentionally blank in a detached-HEAD checkout.
That is not automatically an error: `git status -sb` and `git rev-parse HEAD`
still identify the state. Continue detached only when the task explicitly names
the review commit or detached worktree; do not invent a branch or switch away
merely to make the branch-name check non-empty.

If the task names a branch head, verify it too:

```powershell
git rev-parse origin/<branch>
git rev-list --left-right --count origin/main...HEAD
```

The first count is commits this branch is ahead of `origin/main`; the second is
commits it is behind. A branch can look locally healthy while being stale against
the remote. Fetch before making a publication decision, but do not fetch or pull
into an occupied worktree when the packet says to preserve its exact state.

`git worktree list --porcelain` is the map when several checkouts exist. Treat
each listed path as a real room with its own branch, status, build output, and
possibly unrelated user work. Do not switch the branch in an occupied worktree
merely because it is convenient. A clean dedicated worktree is often the safest
place for a review or authorised fast-forward.

Unix equivalents are the same Git commands. The project automation shell on
Windows is PowerShell 7; do not introduce Bash, WSL, or Unix-only helpers just
to perform these checks.

## 2. Dirty does not mean disposable

`git status --short --branch` is a map, not a cleanup request. A modified or
untracked path may be another task, Matthew's note, generated evidence needed
for diagnosis, or the only copy of unfinished work.

Classify before acting:

- **Expected current-task change:** permitted only when the packet says the
  task is in progress or the change was just made.
- **Unrelated user or another-owner change:** leave it untouched; do not stage,
  stash, restore, reset, clean, or absorb it in the current commit.
- **Unknown change:** inspect it read-only with `git diff -- <path>` or
  `Get-Content`; stop if ownership or safety remains unclear.

Useful narrow commands are:

```powershell
git diff --stat
git diff --name-status
git diff -- <path>
git diff --cached -- <path>
git ls-files --others --exclude-standard
```

Stage by explicit path: `git add -- path1 path2`. Never use `git add -A` in a
mixed worktree. Never use `git reset --hard`, `git clean`, blanket `git restore`,
or a stash as a convenience manoeuvre unless the task explicitly authorises the
exact recovery operation and its target is known.

## 3. Base, range, and working-tree diffs answer different questions

Use the right comparison.

| Question | PowerShell-friendly command |
| --- | --- |
| What is unstaged here? | `git diff` |
| What is staged here? | `git diff --cached` |
| What did this branch add over remote main? | `git diff origin/main...HEAD` |
| What changed between two reviewed commits? | `git diff <base> <head>` |
| Is the branch-owned proposed change clean? | `git diff --check origin/main...HEAD` |
| Are my latest edits whitespace-clean? | `git diff --check` |

Working-tree checks do not prove that an earlier commit in the branch is clean.
Merge-base range checks show what the branch introduced even when remote main
has advanced. Before review or publication, inspect that complete branch-owned
change, not merely the last edit or a worker report:

```powershell
git diff --stat origin/main...HEAD
git diff --name-status origin/main...HEAD
git diff --check origin/main...HEAD
git log --oneline origin/main..HEAD
```

For `git diff`, three dots compare `HEAD` with the merge base. Two dots compare
the two tips directly and can make newer `origin/main` work appear as deletions
from the branch. For `git log`, the two-dot form above is correct: it lists
commits reachable from `HEAD` but not from `origin/main`.

For an exact fast-forward, require the merge base to equal `origin/main`, zero
commits behind, and only the reviewed commits ahead. If any condition differs,
stop and report the SHAs rather than guessing whether a rebase or merge is safe.

## 4. LF, CRLF, normalisation, and encoding

LF is line feed (`\n`). CRLF is carriage return plus line feed (`\r\n`). Windows
editors and tools may display either perfectly well, but Git, parsers, scripts,
patches, and exact byte-sensitive tests can still care. Tether *source* accepts
LF, CRLF, and mixed LF/CRLF line endings by explicit parser decision; that does
not make every repository file or protocol framing byte-insensitive.

Git may normalise content between the index and a working tree according to
repository attributes and local configuration. A warning that LF will be
replaced by CRLF is information about that checkout transformation, not proof
that the committed file is wrong. Do not react by changing `.gitattributes`,
`core.autocrlf`, editor settings, or global Git configuration. Those are policy
or environment decisions and require their own authority.

First inspect the actual diff. A line-ending flood commonly has a huge changed
line count, a familiar document reported as nearly all deletion/addition, and no
meaningful textual change. Check:

```powershell
git diff --stat
git diff --numstat
git diff --ignore-space-at-eol -- <path>
git diff --word-diff=porcelain -- <path>
git diff --check
```

`--ignore-space-at-eol` is a diagnostic lens only. It must not be used to hide
an unwanted conversion in acceptance review. If a narrow edit changed the whole
file, stop before committing. Restore or reapply the intended small edit using
the file's existing encoding and line-ending style, then inspect the result
again. Use a focused patch; do not run a whole-document formatter as repair.

When a task explicitly authorises normalising one file against the repository's
existing attributes, `git add --renormalize -- <path>` is the narrow Git-native
operation. Inspect its staged diff immediately. It changes the index and must
not be used speculatively, across the repository, or as a substitute for a
separate `.gitattributes` decision.

Encoding is a separate problem. A UTF-8 BOM, UTF-16 file, or an editor that
silently changes encoding can break a script even when line endings look fine.
When a file becomes unexpectedly unreadable or a PowerShell/parser error points
to its first character, inspect bytes before editing:

```powershell
Format-Hex -Path <path> -Count 16
Get-Content -LiteralPath <path> -Raw
```

`EF BB BF` is a UTF-8 BOM; `FF FE` or `FE FF` signals UTF-16. Do not remove or
add a BOM merely because it is visible. Preserve the existing format unless a
separate task has frozen a compatibility reason to change it.

PowerShell 7 normally uses UTF-8 without a BOM for text output, but scripts
should still state the intended encoding when they rewrite files. Prefer patch
tools for repository edits. When a script genuinely must write text, use an
explicit no-BOM API such as `Set-Content -Encoding utf8NoBOM` or
`[System.IO.File]::WriteAllText()` and then inspect the diff. Do not use raw
redirection or `Out-File` casually for source or documentation rewrites.

## 5. Safe commits, review branches, and publication

Commit only after the required checks and a full diff inspection. A normal
bounded commit has an intentional message and explicit paths:

```powershell
git add -- docs/example.md AGENTS.md
git diff --cached --check
git diff --cached --stat
git commit -m "docs: explain Git worktree recovery"
```

Push a dedicated review branch when authorised; a push does not merge it. That
is the normal publication target unless the task explicitly grants a direct
`main` update. Before any fast-forward publication, fetch and prove the exact
head and main relationship. For a clean accepted branch already descended from
current remote main, an explicitly authorised exact ref push can publish
without moving the local branch:

```powershell
git push origin HEAD:refs/heads/review/<task-id>
# Only with explicit direct-main authority:
git push origin <accepted-sha>:refs/heads/main
git fetch origin
git rev-parse origin/main
```

Use `git merge --ff-only` only in the specific clean worktree authorised for
the merge. It refuses a non-fast-forward and is therefore useful evidence, not
a workaround. Do not make a merge commit, squash, force-push, delete branches,
or "sync" a dirty worktree merely because a review branch is ready.

Rebase, cherry-pick, and reset are legitimate Git tools, not forbidden magic.
They are appropriate only when a task explicitly grants that operation, names
the source and target, and says how unrelated work is protected. In particular,
do not rebase a reviewed or published branch to make a graph prettier, do not
cherry-pick a broad historical commit without a complete diff review, and never
use reset to erase ambiguous local work. Stop for a recovery packet when the
history or ownership is unclear.

## 6. Worktrees and ignored toolchains

Worktrees share repository object storage but not their ordinary working files,
untracked files, or ignored build directories. An ignored directory such as
`_opam`, Rust `target`, or an OCaml `_build` can legitimately exist beside one
checkout and not another.

Do not infer a toolchain from the current directory or search neighbouring
worktrees. For OCaml, use the explicit absolute `OcamlSwitchPath` supplied by
the task and run Dune in the current worktree's source directory. The switch
provides tools; the current worktree provides source. Do not copy, junction,
move, recreate, or silently select a switch to make a check green.

Similarly, a fresh review worktree may need its own local build output before
tests can find a binary. That is not evidence that the source branch is broken.
Build the current worktree with the authorised existing toolchain, then rerun
the test. Stop if doing so would alter a shared toolchain, install software, or
make it unclear which source tree is being built.

## 7. Stop and recovery conditions

Stop cleanly and report exact output plus the smallest question when:

- the worktree, branch, `HEAD`, base, or expected dirty state differs;
- an edit produces whole-file line-ending or encoding churn;
- the required toolchain is missing, corrupt, or resolves to another source
  worktree;
- a range contains unreviewed commits or a branch is unexpectedly behind;
- a proposed recovery would discard, overwrite, or hide unknown work;
- an operation needs a rebase, cherry-pick, reset, force push, configuration
  change, installation, or policy change not explicitly authorised.

Report the worktree path, branch, `HEAD`, `origin/main`, status, affected paths,
commands run, exact error, and what was deliberately *not* changed. A clean
stop is useful evidence, not a failure of initiative.

## 8. Fast first checks

| Symptom | Likely cause | First safe check |
| --- | --- | --- |
| Git says the worktree is dirty | Current task, user work, build output, or another owner | `git status --short --branch` then narrow diffs; do not clean |
| `git branch --show-current` returns nothing | Checkout is at a detached `HEAD` | Run `git status -sb` and `git rev-parse --short HEAD` |
| Branch looks current but publication refuses | Local branch is stale against remote main | `git fetch origin --prune`; compare `HEAD`, `origin/main`, and merge base |
| Main comparison makes new main work look deleted | Used a two-tip diff instead of a merge-base diff | Use `git diff origin/main...HEAD` |
| Every line changed after a small edit | LF/CRLF or encoding conversion | `git diff --stat`, `--numstat`, and `--word-diff=porcelain` |
| `git diff --check` reports many lines | Trailing whitespace or a conversion flood | Inspect the named lines and complete range before editing |
| OCaml check has no active switch | `_opam` belongs to another worktree | Verify the explicit `OcamlSwitchPath`; do not create one |
| Test cannot find an engine binary in a fresh worktree | That worktree lacks local build output | Build current source with the authorised explicit toolchain |
| A commit is visible locally but not reviewable | Branch was not pushed | `git rev-parse HEAD`; `git ls-remote origin refs/heads/<branch>` |
| A tool appears missing despite PATH entries | Rustup/opam proxy or versioned toolchain selection | Inspect the tool manager's active/explicit toolchain, not only PATH |

Keep the routine small: establish location, protect unrelated work, inspect the
right diff, run the authorised checks, and publish only the exact reviewed
history. That is enough guardrail to prevent expensive mistakes without turning
ordinary Git work into a ceremony.
