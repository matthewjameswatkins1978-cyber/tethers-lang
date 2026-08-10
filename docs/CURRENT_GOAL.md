# Current Goal

Updated: 2026-08-10

## Goal

Tethers 0.3 public Plug authoring. P1 (Generic Operational Scope Evidence) is
ACCEPTED. P2 (public pack / inspect / conform authoring surface) is FINAL
ACCEPTED. P3 (PDF Tools Reference Plug Crucible) is next.

P2 acceptance is anchored to final independently reviewed evidence head
`84f1002904dd54929fa8002d1634c42c85112f54`, with canonical P2C implementation
checkpoint `4c32b96446e7ae3e20d2994056d0fd435dcc32f3`.

## Last accepted increment

P2 — Public pack / inspect / conform authoring surface.

Proven public workflow:

```text
plug pack
→ plug inspect
→ plug conform
```

P2 final gate evidence:

- full Cargo test: 1714 passed, 0 failed, 2 ignored
- Nextest: 1714 passed, 2 skipped
- `just verify-agent`: PASS
- no production or dependency drift in P2C

## Active increment

No P3 implementation packet has been issued yet.

Next: P3 — PDF Tools Reference Plug Crucible. The goal is to move PDF Tools
across the public Plug boundary into `reference-plugs/pdf-tools/` and prove that
a real existing capability can operate without PDF-specific knowledge in the
generic host.

## Foundation Pass boundaries

- Preserve external JSON, exit codes, Trail shape, replay digests, and recovery
  behaviour unless a later packet explicitly authorises a migration.
- Compatibility fixtures are literal committed evidence and are not generated
  by the implementation being tested.
- Every packet reports each required command as PASS, FAIL, or NOT RUN; a
  mandatory NOT RUN blocks COMPLETE.
- Final packet verification is serial after the last permitted code/test edit.
- P3 must not smuggle PDF-specific semantics back into generic Plug machinery.

## Authoritative references

- Road to 0.3: `docs/ROAD_TO_0_3.md`
- Current task: `docs/CURRENT_CLINE_TASK.md`
- Foundation Pass plan: `docs/architecture/TETHERS_FOUNDATION_PASS.md`
- F3a persistence inventory: `docs/foundation-pass/PERSISTENCE_INVENTORY.md`
- F1 debt ledger: `docs/foundation-pass/DEBT_LEDGER.md`
- Enduring principles: `docs/CONSTITUTION.md`
- Language semantics: `tethers-0.1/SPEC.md`
