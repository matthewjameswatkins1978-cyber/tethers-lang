# Tethers Project Dashboard

Updated: 2026-08-10

## Current Milestone

Tethers 0.3 public Plug authoring. P1, P2, P3, and P4 are FINAL ACCEPTED. P5 —
Fresh-Agent Plug Authoring Proof — is FINAL ACCEPTED at
`ffbe25e1c36123301182383c97265a6174b5dd98`. P6 — The Evil Bunny Test — is
complete, awaiting Lucy review.

## Verified Checkpoint

P5 final implementation checkpoint:
`ffbe25e1c36123301182383c97265a6174b5dd98`.

## Active Task

- Task: P6 — The Evil Bunny Test (adversarial provider proof)
- State: complete, awaiting Lucy independent review
- Owner: OpenCode (implementation); Lucy (independent GitHub review and acceptance)
- Risk: Amber; bounded generic conformance corrections plus fixture/evidence

## Last Accepted Result

P5 — Fresh-Agent Plug Authoring Proof — FINAL ACCEPTED at
`ffbe25e1c36123301182383c97265a6174b5dd98`.

A fresh author using `docs/PLUG_AUTHORING.md` as its only guide built the
`tethers.text-stats` Plug and completed the full public journey, surfacing one
narrow manual gap (advertise both `inputSchema` and `outputSchema` in
`tools/list`), fixed in the manual during P5.

## Matthew Decision Required

None.

## Next Route

Lucy review of P6: the Evil Bunny Chronicles (`docs/p6-evil-bunny-proof.md`),
the committed fixture and per-case evidence
(`reference-plugs/evil-bunny-proof/`), the three generic conformance
corrections in `tethers-0.1/host-rust/src/conformance.rs`, and the p6
regression tests (`tests/p6_evil_bunny.rs`). P7 / 0.4 remains next and has NOT
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
- P5 surfaced one narrow manual gap (advertise both `inputSchema` and
  `outputSchema` in `tools/list`), fixed in the manual during P5.
- P6 proved the conformance suite itself still accepted an advertised-only
  `inputSchema` / mismatched `outputSchema`, an uncorrelated JSON-RPC response
  id, and a shutdown-refusing provider; those three generic gaps were corrected
  in `conformance.rs` with before/after evidence and regression tests.

## Where Details Live

- Present goal and boundaries: `docs/CURRENT_GOAL.md`
- Current completed task contract: `docs/CURRENT_CLINE_TASK.md`
- Roadmap: `docs/ROAD_TO_0_3.md`
- Foundation Pass architecture: `docs/architecture/TETHERS_FOUNDATION_PASS.md`
- Persistence inventory: `docs/foundation-pass/PERSISTENCE_INVENTORY.md`
- Evidence and reviews: `docs/worker-notes/`
