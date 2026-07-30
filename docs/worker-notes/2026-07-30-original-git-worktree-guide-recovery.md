# GIT-GUIDE-RECOVERY-01 Worker Note

Task: `GIT-GUIDE-RECOVERY-01 — recover the original Git/worktree booklet`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `8a70a8f47ad8cf110e9987b283f80277705b2292`

Implementation checkpoint: `WORKTREE`

## Requested outcome

Recover the original Work-mode Git, worktrees, and line-endings booklet rather
than the later Goose duplicate, then add only the minimum repository signposts
needed to make that original booklet canonical.

## Changes made

- Restored docs/GIT_WORKTREES_AND_LINE_ENDINGS_FOR_AGENTS.md from the original
  final Work-mode guide commit.
- Added a narrow AGENTS.md required-reading rule for relevant Git/worktree
  tasks.
- Added a concise DECISIONS.md entry recording the guide's authority and its
  non-policy boundary.
- Replaced the completed prior packet with this completed recovery record.

## Decisions and assumptions

The original booklet is authoritative because Matthew explicitly preferred the
earlier refined version. No wording or organisation was borrowed from the
later Goose duplicate. No path or current-main metadata correction was needed
inside the original booklet.

## Evidence

- original branch: docs/git-worktrees-line-endings-guide
- initial guide commit: b4e961f443e60674766c79215cbd446497995672
- final reviewed guide commit: 3e958ceba22bbeed1937b1fa62fa3054fab1596b
- original Git blob: 2c0eb37e7e75ab761939f08392c9819300461883
- Desktop preservation copy:
  C:\Users\Matmus\OneDrive\Desktop\Git, Worktrees and Line Endings for Tethers Agents.md
- SHA-256 for the original commit text, Desktop copy, and recovered guide:
  9BA2F4B9D05813CD15DEAD46C0AA4C2D749B158A24CA36A06CD9DCE132679FCC
- guide word count: 2193
- task-packet checker: PASS
- complete recovery-range whitespace check: PASS

## Discoveries

The later Goose duplicate branch goose/git-worktrees-line-endings-guide at
e63a90d0587c918a07dc2697db6c0f1dace77872 is a substantial duplicate:
276 additions and 231 deletions against the original. It changes the booklet
into a fourteen-section generic field guide. It was not used as a source.

## Remaining risks

The recovery branch is review-only until independently accepted. The Goose
duplicate remains preserved and untouched for historical reference.

## Smallest next action

Independently review the recovery branch, then publish only this branch if its
five-file scope and original-booklet checksum are confirmed.

## References

- 3e958ceba22bbeed1937b1fa62fa3054fab1596b
- e63a90d0587c918a07dc2697db6c0f1dace77872
- 8a70a8f47ad8cf110e9987b283f80277705b2292
