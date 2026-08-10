# Tethers Project Dashboard

Updated: 2026-08-10

## Current Milestone

Tethers 0.3 public Plug authoring. P1 and P2 are FINAL ACCEPTED. P3 — PDF Tools
Reference Plug Crucible — is next.

## Verified Checkpoint

P2 final independently reviewed evidence head:
`84f1002904dd54929fa8002d1634c42c85112f54`.

Canonical P2C implementation checkpoint:
`4c32b96446e7ae3e20d2994056d0fd435dcc32f3`.

## Active Task

- Task: none issued yet
- State: ready for P3 discovery / packet compilation
- Owner: Lucy for architecture and packet compilation
- Risk: Green until P3 implementation begins

## Last Accepted Result

P2 — Public pack / inspect / conform authoring surface — FINAL ACCEPTED.

The real public author journey is proven end to end:

```text
plug pack
→ plug inspect
→ explicit conform execution approval
→ plug conform
```

Final gate evidence:

- full Cargo test: 1714 passed, 0 failed, 2 ignored
- Nextest: 1714 passed, 2 skipped
- `just verify-agent`: PASS
- P2C production changes: 0
- P2C dependency changes: 0

## Matthew Decision Required

None.

## Next Route

P3 discovery, then a bounded implementation packet to move PDF Tools across the
public boundary into `reference-plugs/pdf-tools/` without PDF-specific knowledge
in generic host machinery.

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
- No speculative P3 implementation before the packet is frozen.

## Where Details Live

- Present goal and boundaries: `docs/CURRENT_GOAL.md`
- Current completed task contract: `docs/CURRENT_CLINE_TASK.md`
- Roadmap: `docs/ROAD_TO_0_3.md`
- Foundation Pass architecture: `docs/architecture/TETHERS_FOUNDATION_PASS.md`
- Persistence inventory: `docs/foundation-pass/PERSISTENCE_INVENTORY.md`
- Evidence and reviews: `docs/worker-notes/`
