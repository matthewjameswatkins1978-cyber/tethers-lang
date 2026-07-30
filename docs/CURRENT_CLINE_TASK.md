# Current Implementation Task

Control contract: `1`

Task: `GIT-GUIDE-RECOVERY-01 — recover the original Git/worktree booklet`

Status: `COMPLETE`

Task colour: `Amber`

Owner: `Codex`

Route: `Codex — evidence-led documentation recovery`

Worker note: `docs/worker-notes/2026-07-30-original-git-worktree-guide-recovery.md`

Base branch: `main`

Base commit: `8a70a8f47ad8cf110e9987b283f80277705b2292`

Branch: `docs/original-git-worktree-guide-recovery`

## Objective

Recover the original approved Git, worktrees, and line-endings booklet exactly,
place it at the canonical repository path, and add only the narrow authority
scaffolding needed for agents to find and use it.

## Relevant background and existing behaviour

The original booklet survives as commit
3e958ceba22bbeed1937b1fa62fa3054fab1596b on the local branch
docs/git-worktrees-line-endings-guide and as a byte-identical Desktop copy.
The later Goose booklet on goose/git-worktrees-line-endings-guide was a
duplicate and is deliberately not authority for this recovery.

## Required behaviour

1. Recover the original booklet verbatim at
   docs/GIT_WORKTREES_AND_LINE_ENDINGS_FOR_AGENTS.md.
2. Add one narrow AGENTS.md required-reading rule for relevant Git and
   worktree operations.
3. Record the canonical-guide decision without creating Git, editor, encoding,
   or line-ending policy.
4. Record exact recovery provenance, comparison, checks, and the rejected
   duplicate in the worker note.
5. Keep the recovery branch documentation-only and preserve main, the Goose
   duplicate, and the original dirty worktree.

## Relevant components

- AGENTS.md
- docs/DECISIONS.md
- docs/GIT_WORKTREES_AND_LINE_ENDINGS_FOR_AGENTS.md
- docs/worker-notes/2026-07-30-original-git-worktree-guide-recovery.md

## Frozen decisions and invariants

- The original booklet text is authoritative; do not borrow Goose duplicate
  prose into it.
- The only permitted booklet corrections are strictly necessary path or
  current-main metadata corrections; none are required for this recovery.
- Do not modify .gitattributes, .editorconfig, Git configuration, production
  code, toolchains, or main.
- Do not delete, force-update, merge, or publish the Goose duplicate branch.
- Preserve D:\The Next Thing\Tethers Lang on
  cline/j10-result-event-queue with only its existing
  docs/TETHERS_LUCY_NOTES.md modification.

## Acceptance criteria

1. The recovered booklet is byte-identical to the original commit source and
   Desktop copy.
2. The guide remains original in wording and organisation, with no Goose prose.
3. AGENTS.md points to the guide only for relevant Git/worktree work.
4. DECISIONS.md records the bounded canonical-guide decision.
5. The worker note records source provenance, checksum, comparison, and exact
   checks.
6. Packet checker, word count, complete diff review, and whitespace checks
   pass.
7. Only AGENTS.md, this packet, docs/DECISIONS.md, the guide, and its worker
   note change.
8. main, the Goose duplicate branch, and the original dirty worktree remain
   untouched.

## Required verification

    pwsh -NoProfile -ExecutionPolicy Bypass -File .github/scripts/check-tethers-task-packet.ps1
    (Get-Content -LiteralPath docs/GIT_WORKTREES_AND_LINE_ENDINGS_FOR_AGENTS.md -Raw | Measure-Object -Word).Words
    git diff --check 8a70a8f47ad8cf110e9987b283f80277705b2292..HEAD
    git diff --stat 8a70a8f47ad8cf110e9987b283f80277705b2292..HEAD
    git diff 8a70a8f47ad8cf110e9987b283f80277705b2292..HEAD
    git status --short --branch

## Forbidden changes

- No .gitattributes, .editorconfig, Git configuration, production, toolchain,
  or main changes.
- No Goose-duplicate branch deletion, modification, merge, or publication.
- No original-worktree mutation.

## Stop conditions

Stop if the original booklet cannot be recovered byte-for-byte, the expected
base differs, any unauthorised path changes, or preservation of the original
dirty worktree cannot be demonstrated.

## Expected pre-existing changes

None in this recovery worktree.
