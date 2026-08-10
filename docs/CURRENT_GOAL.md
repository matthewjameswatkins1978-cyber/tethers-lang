# Current Goal

Updated: 2026-08-10

## Goal

Tethers 0.3 public Plug authoring. P1 (Generic Operational Scope Evidence) is
ACCEPTED. P2 (public pack / inspect / conform authoring surface) is FINAL
ACCEPTED. P3 (PDF Tools Reference Plug Crucible) is FINAL ACCEPTED. P4 (Plug
Author Manual) is FINAL ACCEPTED. P5 (fresh-agent authoring proof) is FINAL
ACCEPTED. P6 (The Evil Bunny Test) is complete, awaiting Lucy review. P7 / 0.4
is next and has NOT started.

P5 acceptance is anchored to accepted head
`ffbe25e1c36123301182383c97265a6174b5dd98`.

## Last accepted increment

P5 — Fresh-Agent Plug Authoring Proof — FINAL ACCEPTED at
`ffbe25e1c36123301182383c97265a6174b5dd98`.

A fresh DeepSeek V4 Flash / High-thinking author, guided only by
`docs/PLUG_AUTHORING.md`, built the new `tethers.text-stats` Plug under
`reference-plugs/text-stats-proof/` and completed the full public journey.
Experiment log: `docs/p5-fresh-agent-proof.md`.

## Active increment

P6 — The Evil Bunny Test — is complete, awaiting Lucy review. The adversarial
protocol fixture under `reference-plugs/evil-bunny-proof/` proved that hostile
providers cannot turn bad evidence into conformance success; three genuine
generic conformance gaps (discovery `outputSchema`, JSON-RPC response
correlation, shutdown-refusal cleanup accounting) were corrected in
`tethers-0.1/host-rust/src/conformance.rs` with regression evidence at the real
discovery/conformance seam. Experiment log: `docs/p6-evil-bunny-proof.md`.

P7 / 0.4 is next after P6 and has NOT started.

## Foundation Pass boundaries

- Preserve external JSON, exit codes, Trail shape, replay digests, and recovery
  behaviour unless a later packet explicitly authorises a migration.
- Compatibility fixtures are literal committed evidence and are not generated
  by the implementation being tested.
- Every packet reports each required command as PASS, FAIL, or NOT RUN; a
  mandatory NOT RUN blocks COMPLETE.
- Final packet verification is serial after the last permitted code/test edit.
- P6 must not smuggle adversarial-provider-specific knowledge into generic host
  code; every production correction is generic and regression-backed.

## Authoritative references

- Road to 0.3: `docs/ROAD_TO_0_3.md`
- Current task: `docs/CURRENT_CLINE_TASK.md`
- Foundation Pass plan: `docs/architecture/TETHERS_FOUNDATION_PASS.md`
- F3a persistence inventory: `docs/foundation-pass/PERSISTENCE_INVENTORY.md`
- F1 debt ledger: `docs/foundation-pass/DEBT_LEDGER.md`
- Enduring principles: `docs/CONSTITUTION.md`
- Language semantics: `tethers-0.1/SPEC.md`
