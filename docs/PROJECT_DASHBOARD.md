# Tethers Project Dashboard

Updated: 2026-08-09

## Current Milestone

Foundation pre-F10 — final gate consistency repair. F1–F9 are complete through
operator truth reconciliation. This programme adds no product capability and
advances only through separately reviewed evidence packages. F10 remains the
sole Foundation completion gate.

## Verified Checkpoint

Latest accepted implementation: F9-FINAL operator truth reconciliation at
`fc33dba435a87833a6f0f53642326697a246694b` (Foundation branch lineage).
F1–F9 have not yet been merged to `origin/main`; live `main` remains
`40ec42eb2aac108901d428af3cbfe264d3edd6dc`.

## Active Task

- Task: PRE-F10 — Final gate consistency repair
- State: IN_PROGRESS
- Owner: OpenCode
- Risk: Green

## Last Accepted Result

F9-FINAL operator truth reconciliation has completed on the Foundation branch
lineage. The all-target Rust Cargo check is warning-free and compiler warnings
are now denied in the repository `just check` / `just verify` path.

## Matthew Decision Required

None.

## Next Route

Complete pre-F10 gate repair. Lucy reviews and prepares F10 clean-checkout
proof.

## Operating Mode

**Gorilla Coding 🦄**

- Lucy: architecture, task compilation, GitHub review, acceptance, continuation.
- OpenCode: ordinary Green and Amber implementation, checks, report, worker note.
- Codex: Red work, difficult local diagnosis, Git/environment/recovery, and
  machine-required verification.
- Matthew: product authority and the short report-routing bridge.

## Cost And Drift

- One implementation owner per bounded task.
- F8 cleanup removed 15 dead-code items. Warning enforcement is now active.
- Broader Clippy advisory diagnostics remain separate from the denied-warning
  gate.
- No speculative post-Foundation plans are authorised.

## Where Details Live

- Present goal and boundaries: `docs/CURRENT_GOAL.md`
- Active task contract: `docs/CURRENT_CLINE_TASK.md`
- Foundation Pass architecture: `docs/architecture/TETHERS_FOUNDATION_PASS.md`
- Persistence inventory: `docs/foundation-pass/PERSISTENCE_INVENTORY.md`
- Evidence and reviews: `docs/worker-notes/`
