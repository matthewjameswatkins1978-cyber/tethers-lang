# Tethers Project Dashboard

Updated: 2026-08-09

## Current Milestone

Foundation F9 — operator truth reconciliation. F1–F8 are complete through
warning enforcement. This programme adds no product capability and advances
only through separately reviewed evidence packages. F10 remains the sole
Foundation completion gate.

## Verified Checkpoint

Latest accepted implementation: F8 warning enforcement at
`5e616357963e70b86f59c870f6c00b7fbc94cb0a` (`origin/main`).

## Active Task

- Task: F9 — Operator truth reconciliation
- State: IN_PROGRESS
- Owner: OpenCode
- Risk: Green (documentation-only)

## Last Accepted Result

F8 warning enforcement is accepted and merged. The all-target Rust Cargo
check is warning-free and compiler warnings are now denied in the repository
`just check` / `just verify` path.

## Matthew Decision Required

None.

## Next Route

Complete F9 documentation updates. Lucy reviews and prepares F10
clean-checkout proof.

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
