# Current Goal

Updated: 2026-08-10

## Goal

Tethers 0.3 public Plug authoring. P1 (Generic Operational Scope Evidence) is
ACCEPTED. P2 (public pack / inspect / conform authoring surface) is FINAL
ACCEPTED. P3 (PDF Tools Reference Plug Crucible) is FINAL ACCEPTED. P4 (Plug
Author Manual) is the active/current increment. P5 is next after P4 and has NOT
started.

P3 acceptance is anchored to accepted head
`e23030ad5e9820373133b25222680194af967c39`, with final P3 implementation
correction checkpoint `fcf22bff911393869d8dd560efeee1442a50b119`.

## Last accepted increment

P3 — PDF Tools Reference Plug Crucible — FINAL ACCEPTED.

The real reference Plug journey is proven end to end:

```text
standalone provider
→ public pack / inspect / conform
→ real installed generic execution
```

P3 final gate evidence:

- `just verify-agent`: PASS
- Nextest: 1670 passed, 4 skipped
- standalone provider checks and both ignored P3 crucible tests: PASS

## Active increment

P4 — Plug Author Manual. Write the complete public author manual using only
interfaces and behaviour proven by P1–P3. The canonical manual is
`docs/PLUG_AUTHORING.md`.

P5 (fresh-agent authoring proof) remains next after P4 and has NOT started.

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
