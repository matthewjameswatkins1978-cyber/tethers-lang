# Tethers Project Dashboard

Updated: 2026-08-07

## Current Milestone

Foundation Pass — bounded strengthening after the accepted J24K/J24L Plug
installation slice. The programme adds no product capability and advances only
through separately reviewed evidence packages.

## Verified Checkpoint

Latest accepted implementation:
`145a791ceb3f5e3b8855aeadbac83671d9a2b363`
(F3a merged to main), now `origin/main`.

F3a classified 14 filesystem-backed persistence stores, identified 1
coordination artifact, and routed all Windows primitive evidence questions
to F3b.

## Active Task

- Task: F3b — Windows persistence primitive evidence
- State: IN_PROGRESS
- Owner: OpenCode
- Risk: Red (persistence contract evidence)

## Last Accepted Result

F3a is accepted and merged. Its evidence branch
`foundation/f3a-persistence-vocabulary` remains retained at the same SHA as
main; it has not been deleted.

## Matthew Decision Required

None.

## Next Route

OpenCode performs the bounded F3b Windows primitive evidence pass from
`foundation/f3b-windows-persistence-evidence`; Lucy independently reviews
before F3c. No persistence repair or F3c installation intent/publication
work is authorised by the F3b packet.

## Operating Mode

**Gorilla Coding 🦄**

- Lucy: architecture, task compilation, GitHub review, acceptance, continuation.
- OpenCode: ordinary Green and Amber implementation, checks, report, worker note.
- Codex: Red work, difficult local diagnosis, Git/environment/recovery, and
  machine-required verification.
- Matthew: product authority and the short report-routing bridge.

## Cost And Drift

- Use one implementation owner per bounded package.
- F3b establishes direct Windows evidence; it must not infer guarantees or
  repair persistence behaviour.
- F3c remains the only route for installation intent/publication repair.
- Preserve the F1 compatibility fixtures unchanged.
- Separate every property: file-data durability, atomic visibility, directory
  entry durability, interruption behaviour, reparse-point defence.

## Where Details Live

- Present goal and boundaries: `docs/CURRENT_GOAL.md`
- Active task contract: `docs/CURRENT_CLINE_TASK.md`
- Foundation Pass architecture: `docs/architecture/TETHERS_FOUNDATION_PASS.md`
- Persistence inventory: `docs/foundation-pass/PERSISTENCE_INVENTORY.md`
- Evidence and reviews: `docs/worker-notes/`
