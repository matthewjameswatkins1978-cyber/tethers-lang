# Current Goal

Updated: 2026-08-10

## Goal

Tethers 0.3 public Plug authoring. P1 (Generic Operational Scope Evidence) is
ACCEPTED. P2 (public pack / inspect / conform authoring surface) is FINAL
ACCEPTED. P3 (PDF Tools Reference Plug Crucible) is FINAL ACCEPTED. P4 (Plug
Author Manual) is FINAL ACCEPTED. P5 (fresh-agent authoring proof) is complete,
awaiting Lucy review. P6 is next after P5 and has NOT started.

P4 acceptance is anchored to accepted head
`1e1f9b8738a48f727187316dd0078b7f9435f1c6`.

## Last accepted increment

P4 — Plug Author Manual — FINAL ACCEPTED at
`1e1f9b8738a48f727187316dd0078b7f9435f1c6`.

The canonical public author manual is `docs/PLUG_AUTHORING.md` and documents the
interfaces and behaviour proven by P1–P3: author source tree, `plug.json`,
capability manifests, the provider contract, Operational Scope Evidence, and
the `plug pack` → `plug inspect` → `plug conform` journey.

## Active increment

P5 — Fresh-Agent Plug Authoring Proof — is complete, awaiting Lucy review. A
fresh DeepSeek V4 Flash / High-thinking author, guided only by
`docs/PLUG_AUTHORING.md`, built the new `tethers.text-stats` Plug under
`reference-plugs/text-stats-proof/` and completed the full public journey.
Experiment log: `docs/p5-fresh-agent-proof.md`.

P6 (adversarial-provider proof) is next after P5 and has NOT started.

## Foundation Pass boundaries

- Preserve external JSON, exit codes, Trail shape, replay digests, and recovery
  behaviour unless a later packet explicitly authorises a migration.
- Compatibility fixtures are literal committed evidence and are not generated
  by the implementation being tested.
- Every packet reports each required command as PASS, FAIL, or NOT RUN; a
  mandatory NOT RUN blocks COMPLETE.
- Final packet verification is serial after the last permitted code/test edit.
- P5 must not smuggle PDF-specific semantics back into generic Plug machinery.

## Authoritative references

- Road to 0.3: `docs/ROAD_TO_0_3.md`
- Current task: `docs/CURRENT_CLINE_TASK.md`
- Foundation Pass plan: `docs/architecture/TETHERS_FOUNDATION_PASS.md`
- F3a persistence inventory: `docs/foundation-pass/PERSISTENCE_INVENTORY.md`
- F1 debt ledger: `docs/foundation-pass/DEBT_LEDGER.md`
- Enduring principles: `docs/CONSTITUTION.md`
- Language semantics: `tethers-0.1/SPEC.md`
