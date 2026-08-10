# Tethers Project Dashboard

Updated: 2026-08-10

## Current Milestone

Tethers 0.3 public Plug authoring. P1 and P2 are FINAL ACCEPTED. P3 — PDF Tools
Reference Plug Crucible — is FINAL ACCEPTED at
`e23030ad5e9820373133b25222680194af967c39`. P4 — Plug Author Manual — is the
active/current increment.

## Verified Checkpoint

P3 final implementation correction checkpoint:
`fcf22bff911393869d8dd560efeee1442a50b119`.

## Active Task

- Task: P4 — Plug Author Manual
- State: implementation in progress
- Owner: OpenCode (implementation); Lucy (independent GitHub review and acceptance)
- Risk: Green; documentation-only, no production code changes

## Last Accepted Result

P3 — PDF Tools Reference Plug Crucible — FINAL ACCEPTED at
`e23030ad5e9820373133b25222680194af967c39`.

The real reference Plug journey is proven end to end:

```text
standalone provider
→ public pack / inspect / conform
→ real installed generic execution
```

Final gate evidence:

- `just verify-agent`: PASS
- Nextest: 1670 passed, 4 skipped
- standalone provider checks and both ignored P3 crucible tests: PASS
- host PDF production references and provider host dependency searches: empty

## Matthew Decision Required

None.

## Next Route

P4 — Plug Author Manual implementation by OpenCode, then Lucy GitHub review and
acceptance. P5 (fresh-agent authoring proof) remains next after P4 and has NOT
started.

## Operating Mode

**Gorilla Coding 🦄**

- Lucy: architecture, task compilation, GitHub review, acceptance, continuation.
- OpenCode: ordinary Green and Amber implementation, checks, report, worker note.
- Codex: Red work, difficult local diagnosis, Git/environment/recovery, and
  machine-required verification.
- Matthew: product authority and the short report-routing bridge.

## Cost And Drift

- One implementation owner per bounded task.
- Broad discovery before packet compilation.
- Cheap checks early. Expensive proof once.
- P5 implementation awaits a separately authorised packet.

## Where Details Live

- Present goal and boundaries: `docs/CURRENT_GOAL.md`
- Current completed task contract: `docs/CURRENT_CLINE_TASK.md`
- Roadmap: `docs/ROAD_TO_0_3.md`
- Foundation Pass architecture: `docs/architecture/TETHERS_FOUNDATION_PASS.md`
- Persistence inventory: `docs/foundation-pass/PERSISTENCE_INVENTORY.md`
- Evidence and reviews: `docs/worker-notes/`
