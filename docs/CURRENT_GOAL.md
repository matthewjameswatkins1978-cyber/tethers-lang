# Current Goal

Updated: 2026-08-07

## Goal

Complete the Tethers Foundation Pass through separately reviewed evidence
packages. F3a established the persistence vocabulary, F3b established direct
Windows primitive evidence, F3c covered installation intent/publication, and
F3d has completed the remaining bounded-store characterization without a
production repair.

The accepted main is `40ec42eb2aac108901d428af3cbfe264d3edd6dc` (F3c).

## Last accepted increment

F3c — Installation intent and publication contract — is accepted and merged.
Its evidence branch `foundation/f3c-installation-intent-publication` is retained
at the same accepted SHA.

## Active increment

F3d — Remaining bounded persistence stores — is `COMPLETE` on
`foundation/f3d-bounded-persistence-stores`, pending Lucy's independent Amber
review. It adds and cites characterization evidence only; no production
persistence behaviour was repaired or redesigned.

## Foundation Pass boundaries

- No language-semantic, Plug-capability, or new-CLI work.
- Preserve external JSON, exit codes, Trail shape, replay digests, and recovery
  behaviour unless a later package explicitly authorises a migration.
- Compatibility fixtures are literal committed evidence and are not generated
  by the implementation being tested.
- Every package reports each required command as PASS, FAIL, or NOT RUN; a
  mandatory NOT RUN blocks COMPLETE.
- Final package verification is serial after the last permitted edit.

## Authoritative references

- Foundation Pass plan: `docs/architecture/TETHERS_FOUNDATION_PASS.md`
- Current task: `docs/CURRENT_CLINE_TASK.md`
- F3a persistence inventory: `docs/foundation-pass/PERSISTENCE_INVENTORY.md`
- F1 debt ledger: `docs/foundation-pass/DEBT_LEDGER.md`
- F3d worker note: `docs/worker-notes/2026-08-07-f3d-bounded-persistence-stores.md`
- Enduring principles: `docs/CONSTITUTION.md`
- Language semantics: `tethers-0.1/SPEC.md`
