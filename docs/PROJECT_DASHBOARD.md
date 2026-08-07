# Tethers Project Dashboard

Updated: 2026-08-07

## Current Milestone

Foundation Pass — bounded strengthening after the accepted J24K/J24L Plug
installation slice. The programme adds no product capability and advances only
through separately reviewed evidence packages.

## Verified Checkpoint

Latest accepted implementation:
`83eec98a0f33f964623f4cbbf4548a76bbdf5255`
(`docs: mark F2 COMPLETE with correction r2 evidence`), now `origin/main`.

F2 preserved F1’s literal compatibility fixtures and repaired two confirmed
operational issues: live stderr tail visibility during a running child process,
and the M3 Windows handle test’s numeric-handle aliasing false positive.

## Active Task

- Task: F3a — Persistence inventory and vocabulary
- State: READY; packet prepared only, not begun
- Owner: Codex
- Risk: Red (persistence contract evidence)

## Last Accepted Result

F2 is accepted and merged. Its evidence branch
`foundation/f2-operational-correctness` remains retained at the same SHA as
main; it has not been deleted.

## Matthew Decision Required

None.

## Next Route

Codex performs the bounded F3a documentation/evidence pass from
`foundation/f3a-persistence-vocabulary`; Lucy independently reviews it before
any F3b Windows primitive evidence work. No persistence repair or F3b work is
authorised by the F3a packet.

## Operating Mode

**Gorilla Coding 🦄**

- Lucy: architecture, task compilation, GitHub review, acceptance, continuation.
- Cline: ordinary Green and Amber implementation, checks, report, worker note.
- Codex: Red work, difficult local diagnosis, Git/environment/recovery, and
  machine-required verification.
- Matthew: product authority and the short report-routing bridge.
- Copilot: not in the active workflow.

## Cost And Drift

- Use one implementation owner per bounded package.
- F3a documents what the accepted mainline proves; it must not infer Windows
  directory durability or repair persistence behaviour.
- F3b remains the only route for Windows primitive evidence.
- Preserve the F1 compatibility fixtures unchanged.

## Where Details Live

- Present goal and boundaries: `docs/CURRENT_GOAL.md`
- Active task contract: `docs/CURRENT_CLINE_TASK.md`
- Foundation Pass architecture: `docs/architecture/TETHERS_FOUNDATION_PASS.md`
- Persistence inventory: `docs/foundation-pass/PERSISTENCE_INVENTORY.md`
- Evidence and reviews: `docs/worker-notes/`
