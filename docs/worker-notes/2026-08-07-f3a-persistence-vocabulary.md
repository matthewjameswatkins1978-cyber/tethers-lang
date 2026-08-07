# Worker Note

Task: `F3a - Persistence inventory and vocabulary`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `READY`

Base commit: `83eec98a0f33f964623f4cbbf4548a76bbdf5255`

Implementation checkpoint: `WORKTREE`

## Requested outcome

Prepare an evidence-only inventory of every filesystem-backed persistence
store in accepted main, using the Foundation Pass four-class vocabulary. No
implementation, test, fixture, dependency, or F3b work is authorised.

## Changes made

Preparation stub only. The implementation owner must replace this section with
the actual documentation changes before marking the task complete.

## Decisions and assumptions

The accepted F2 mainline is the sole implementation baseline. Directory-entry
durability remains `UNVERIFIED (F3b)` unless direct accepted-main evidence
proves a narrower claim.

## Evidence

Packet preparation was based on the accepted F2 mainline and the named F1/F2
Foundation Pass evidence. No F3a inventory work or implementation test has run.

## Discoveries

The existing inventory already distinguishes atomic records, current-state
records, causal logs, and journals, but F3a must revalidate its rows from live
accepted-main source and direct tests rather than preserve historical wording.

## Remaining risks

Current directory-durability statements are intentionally unverified; F3b is
the dedicated Windows primitive-evidence package. Trail path safety and
line-level corruption recovery remain evidence topics, not authorised repairs.

## Smallest next action

The named owner should perform F3a only: inspect the named source/tests,
reconcile the documentation inventory, run the required documentation checks,
and stop before F3b.

## References

- Accepted main/base: `83eec98a0f33f964623f4cbbf4548a76bbdf5255`
- Foundation plan: `docs/architecture/TETHERS_FOUNDATION_PASS.md`
- Inventory: `docs/foundation-pass/PERSISTENCE_INVENTORY.md`
- F1/F2 notes: `docs/worker-notes/2026-08-06-f1-baseline.md` and `docs/worker-notes/2026-08-07-f2-operational-correctness.md`
