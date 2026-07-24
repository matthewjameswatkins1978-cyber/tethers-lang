# Worker Note

Task: `J03a one-shot approval correction`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Lucy/Codex`

Status: `COMPLETE`

Base commit: `643c6ed40e3e8a167afd53eca2c98597c0aa8f24`

Implementation checkpoint: `WORKTREE`

## Requested outcome

Correct the J03 design blocker without changing the 0.2 release boundary or
implementing runtime behaviour.

## Changes made

- Defined the approved one-shot Ask record as the single, exact confirmation
  that converts an otherwise current Ask into one Allow after fresh checks.
- Aligned capability-bridge confirmation denial/cancellation Trail wording
  with the architecture rule that only attempted provider calls emit standard
  result Anchors.
- Documented the Copilot-to-Cline-to-Codex handoff sequence in the dashboard.

## Decisions and assumptions

- Fresh validation, binding, scope and Deny checks precede approval
  consumption; approval is not standing permission.
- An unattempted confirmation denial is an authorisation record, not an
  execution failure.

## Evidence

- The preceding J03 review identified the unreachable mandatory-confirmation
  resume branch and bridge terminology conflict.
- This correction is documentation-only; no runtime code or fixtures changed.
- Packet checker, whitespace check, full documentation diff and final Git
  status are required before handoff.

## Discoveries

The existing road already routes J04 to Copilot and the first planned Green
increment, J08, to Cline/DeepSeek. The needed conveyor change is an explicit
handoff gate, not a reassignment of the Amber policy work.

## Remaining risks

The approved design is not runtime proof. J04 must implement only the frozen
effective-policy resolver from a fresh packet, with its own focused tests and
worker note.

## Smallest next action

Have Copilot run `/next-tethers-task` to compile J04 from this signed baseline.

## Review verdict

`SIGNED OFF` — Codex controller review on 2026-07-24. The complete live diff,
worker note, packet, authoritative policy/Trail documents, Git state, packet
checker, and whitespace check were inspected. No runtime code changed.

## References

- `docs/DECISIONS.md` J03 Four-Outcome Host Policy Contract
- `docs/CAPABILITY_BRIDGE.md` confirmation and execution-Trail sections
- `docs/ROAD_TO_0_2.md` J03, J04 and J08
