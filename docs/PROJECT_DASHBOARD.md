# Tethers Project Dashboard

Updated: 2026-08-10

## Current Milestone

Tethers 0.3 public Plug authoring. P1, P2, and P3 are FINAL ACCEPTED. P4 — Plug
Author Manual — is FINAL ACCEPTED at
`1e1f9b8738a48f727187316dd0078b7f9435f1c6`. P5 — Fresh-Agent Plug Authoring
Proof — is complete, awaiting Lucy review.

## Verified Checkpoint

P4 final implementation checkpoint:
`1e1f9b8738a48f727187316dd0078b7f9435f1c6`.

## Active Task

- Task: P5 — Fresh-Agent Plug Authoring Proof
- State: complete, awaiting Lucy independent review
- Owner: OpenCode (implementation); Lucy (independent GitHub review and acceptance)
- Risk: Green; evidence-heavy, reference-provider and documentation changes only

## Last Accepted Result

P4 — Plug Author Manual — FINAL ACCEPTED at
`1e1f9b8738a48f727187316dd0078b7f9435f1c6`.

The canonical public author manual is `docs/PLUG_AUTHORING.md` and documents the
interfaces and behaviour proven by P1–P3: author source tree, `plug.json`,
capability manifests, the provider contract, Operational Scope Evidence, and
the `plug pack` → `plug inspect` → `plug conform` journey.

## Matthew Decision Required

None.

## Next Route

P5 evidence review by Lucy: fresh-author experiment log
(`docs/p5-fresh-agent-proof.md`), the new `reference-plugs/text-stats-proof/`
Plug, and the narrow `docs/PLUG_AUTHORING.md` correction. P6 (adversarial-provider
proof) remains next after P5 and has NOT started.

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
- P5 surfaced one narrow manual gap (advertise both `inputSchema` and
  `outputSchema` in `tools/list`), fixed in the manual during P5.

## Where Details Live

- Present goal and boundaries: `docs/CURRENT_GOAL.md`
- Current completed task contract: `docs/CURRENT_CLINE_TASK.md`
- Roadmap: `docs/ROAD_TO_0_3.md`
- Foundation Pass architecture: `docs/architecture/TETHERS_FOUNDATION_PASS.md`
- Persistence inventory: `docs/foundation-pass/PERSISTENCE_INVENTORY.md`
- Evidence and reviews: `docs/worker-notes/`
