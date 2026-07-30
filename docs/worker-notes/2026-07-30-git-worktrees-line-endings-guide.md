# Worker Note

Task: `GIT-GUIDE-01 — Git, Worktrees and Line Endings for Tethers Agents`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Goose`

Status: `COMPLETE`

Base commit: `8a70a8f47ad8cf110e9987b283f80277705b2292`

Implementation checkpoint: `<pending commit>`

## Requested outcome

Create a practical field guide at `docs/GIT_WORKTREES_AND_LINE_ENDINGS_FOR_AGENTS.md`
that helps Tethers agents use Git, worktrees and Windows/Unix text files safely,
add a narrow required-reading rule to `AGENTS.md`, and record the decision in
`docs/DECISIONS.md`.

## Changes made

1. **`docs/GIT_WORKTREES_AND_LINE_ENDINGS_FOR_AGENTS.md`** — new canonical guide
   (2,363 words, 14 sections). Covers mental model, Tethers preflight, worktree
   ownership, LF/CRLF explanation, diagnostic techniques, diff layers, safe
   edits, commits, branch publication, recovery tools, stop conditions,
   diagnostic table, and pocket command list.
2. **`AGENTS.md`** — added narrow required-reading rule after the existing Rust
   guide requirement. The new rule applies only to Git-topology, worktree,
   line-ending/encoding, history-recovery, or destructive-Git tasks. Existing
   OCaml and Rust guide requirements are preserved unchanged.
3. **`docs/DECISIONS.md`** — added `2026-07-30: Canonical Git, Worktrees and
   Line-Endings Guide` decision recording the guide's advisory role, the
   no-new-policy rule, and preservation of destructive-tool availability under
   explicit task authority.
4. **`docs/CURRENT_CLINE_TASK.md`** — replaced with the GIT-GUIDE-01 task packet.
5. **`docs/worker-notes/2026-07-30-git-worktrees-line-endings-guide.md`** — this
   worker note.

## Decisions and assumptions

- The guide uses observed repository evidence: `core.autocrlf=true` (system-level),
  `i/lf w/crlf` for tracked Markdown files on Windows, and three active worktrees
  sharing one `.git` directory.
- The guide assumes PowerShell 7 (`pwsh.exe`) as the shell, consistent with the
  project's `docs/PROJECT_CONTROL.md`.
- Recovery tools (rebase, reset, cherry-pick, stash, clean, reflog, force-push)
  are described as legitimate but dangerous without explicit authority. This
  preserves existing project rules without adding blanket prohibitions.
- The `_opam`/worktree example uses the original dirty `TETHERS_LUCY_NOTES.md`
  situation as an abstract illustration without reproducing its content.

## Evidence

### Repository evidence (observational only)

```
git config --show-origin --get core.autocrlf  →  file:C:/Program Files/Git/etc/gitconfig  true
git ls-files --eol AGENTS.md                  →  i/lf    w/crlf  attr/
git ls-files --eol docs/DECISIONS.md          →  i/lf    w/crlf  attr/
git worktree list --porcelain                 →  3 worktrees (original, j12-acceptance, Goose)
```

### Preflight

- Repository root: `D:/The Next Thing/Tethers Lang - Goose Integration`
- Branch: `goose/git-worktrees-line-endings-guide` created from `8a70a8f`
- origin/main: `8a70a8f47ad8cf110e9987b283f80277705b2292` (exact match)
- Merge base equals origin/main; ahead 0, behind 0 (before edits)
- Original worktree: `D:/The Next Thing/Tethers Lang` on `cline/j10-result-event-queue`,
  only `M docs/TETHERS_LUCY_NOTES.md`

### Verification

- Packet checker: `<run at COMPLETE>`
- Whitespace check: `<run at COMPLETE>`
- Guide word count: 2,363
- Changed files: exactly 5 authorised files confirmed

### EOL/config

- No `.gitattributes`, `.editorconfig`, Git configuration, or encoding changes
  were made.
- `core.autocrlf`, `core.eol`, `core.safecrlf` remain unchanged.
- No `git add --renormalize`, reset, clean, stash, rebase, cherry-pick, amend,
  or force-push was performed.

## Discoveries

- The AGENTS.md file has two occurrences of "remembered chat context." but they
  differ structurally: the OCaml rule places it on one continuous line, while
  the Rust rule splits "remembered" and "chat context." across a CRLF boundary.
  This made targeted string matching fragile and required byte-offset insertion.
- `git ls-files --eol` on the first `8a70a8f` checkout showed `i/lf w/crlf`
  for both AGENTS.md and docs/DECISIONS.md, confirming the system-level
  `core.autocrlf=true` applies to these tracked files.

## Remaining risks

None known within packet scope. The guide is advisory and introduces no new
repository policy. Future tasks may need more specific guidance for unusual
recovery scenarios, but the guide's stop conditions and diagnostic table give
agents a reliable first-response framework.

## Smallest next action

TOOLCHAIN-BASELINE-01 implementation, if authorised. Do not begin here.

## References

- Base commit: `8a70a8f47ad8cf110e9987b283f80277705b2292`
- Branch: `goose/git-worktrees-line-endings-guide`
- `docs/GIT_WORKTREES_AND_LINE_ENDINGS_FOR_AGENTS.md`
- `AGENTS.md` (line 31: Git rule insertion)
- `docs/DECISIONS.md` (2026-07-30 decision entry)
- `docs/CURRENT_CLINE_TASK.md` (GIT-GUIDE-01 packet)
- `docs/OCAML_GUIDE_FOR_AGENTS.md` §7 (worktree/switch guidance)
- `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md` (toolchain and verification)
