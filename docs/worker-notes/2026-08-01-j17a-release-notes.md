# Worker Note

Task: `J17A2 - draft the Tethers 0.2.0 release notes`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `Luna`
Status: `COMPLETE`
Starting branch: `hy3/j17a-product-version`
Starting commit: `7179087ed82a9d2055f4958d23b1e38ac366ebb1`
Working branch: `luna/j17a-release-notes`
Base commit: `7179087ed82a9d2055f4958d23b1e38ac366ebb1`
Implementation checkpoint: `WORKTREE`

## Requested outcome

Draft the 0.2.0 release candidate notes and identify the candidate in README.

## Changes made

- `README.md`
- `docs/ROAD_TO_0_2.md`
- `docs/CURRENT_GOAL.md`
- `docs/PROJECT_DASHBOARD.md`
- `docs/TASK_QUEUE.md`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-01-j16d-complete-clean-verification.md`
- `docs/worker-notes/2026-08-01-j17a-product-version.md`
- `tethers-0.1/SPEC.md`

Created `docs/releases/v0.2.0.md` with candidate status, release scope,
highlights, trust and safety properties, supported environment, known limits,
deferred scope, and release state. It does not claim sign-off, publication, or
tagging.

Verification numbers included and their source records:

- Rust `797 passed, 0 failed, 0 ignored`: `2026-08-01-j17a-product-version.md`
  and `2026-08-01-j16d-complete-clean-verification.md`.
- MCP transcript suite `15/15`: `2026-08-01-j17a-product-version.md`.
- J14C `9/9 rows, 196 assertions`: `2026-08-01-j17a-product-version.md` and
  `2026-07-31-j14c-real-file-move.md`.
- Consolidated matrix `6/6 suites, 79 accepted cases/rows`:
  `2026-07-31-j15d-full-matrix.md` and `2026-08-01-j16d-complete-clean-verification.md`.
- Runner contract `6/6 rows, 49 assertions`: `2026-08-01-j16d-complete-clean-verification.md`.
- Clean-checkout, restart, and replay proof: `2026-08-01-j16d-complete-clean-verification.md`.
- Product identity `0.2.0`: `2026-08-01-j17a-product-version.md`.

Unsupported claims deliberately excluded: sign-off, release, tag creation,
main publication, installer or packaged distribution, cross-platform support,
remote/network support, automatic discovery, retry or compensation, GUI/HQ,
and Lantern Keeper integration.

README now has a short Release Candidate section and the requested current worker
route. Existing 0.1 wording, `tethers-0.1/` naming, architectural links, MCP
boundary, and Lantern Keeper boundary were preserved.

## Decisions and assumptions

The release note records accepted project totals only and treats J17 as the
independent final release gate. No unsupported platform, packaging, sign-off,
publication, or tag claim was added.

## Evidence

The required phrase and version searches, packet checker, whitespace check, and
changed-path/status checks were run after editing. Exact results are recorded in
the handoff report and final Git state.

## Discoveries

The prior J17A1 packet required replacement with the current J17A2 control-v1
structure so the repository checker could validate the new task.

## Remaining risks

J17 independent sign-off remains pending. This note does not constitute a
release gate or publication decision.

## Smallest next action

Lucy performs the independent J17 release gate; do not publish main or create a
tag from this task.

## References

- `docs/releases/v0.2.0.md`
- `docs/ROAD_TO_0_2.md`
- `docs/worker-notes/2026-08-01-j16d-complete-clean-verification.md`
- `docs/worker-notes/2026-08-01-j17a-product-version.md`
- `docs/worker-notes/2026-07-31-j15d-full-matrix.md`

## Final Evidence

Changed paths are exactly:

- `docs/releases/v0.2.0.md`
- `README.md`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-01-j17a-release-notes.md`

Branch is `7` ahead and `0` behind `origin/main`. The worktree is clean after
the authorised commit. J17 sign-off, main publication, and tag creation remain
deferred.
