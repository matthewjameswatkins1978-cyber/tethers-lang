# Current Goal

Updated: 2026-08-10

## Goal

Tethers 0.3 public Plug authoring. P1 (Generic Operational Scope Evidence) is
ACCEPTED. P2 is AWAITING FINAL ACCEPTANCE: P2A (public deterministic plug pack)
is ACCEPTED at `3d7fd7e580d274de0a422fb78c5741a6bd1405f1`. P2B (public
supervised plug conform) is ACCEPTED at
`061a57d4bd48e59cae2d496b889834df7fe54418`. P2C (end-to-end public author proof
+ final P2 verification gate) is COMPLETE. P2 is now awaiting Lucy final
independent acceptance before proceeding to P3.

## Last accepted increment

P1 Generic Operational Scope Evidence accepted at
`270a5913a93d64256113cca3450619c484b7ddc7`.

## Active increment

P2 — Public pack / inspect / conform authoring surface. All three sub-phases
(P2A, P2B, P2C) are now implemented with all verification gates passing. P2
awaits Lucy final independent acceptance. P3 (PDF Tools Reference Plug Crucible)
is next.

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
- Enduring principles: `docs/CONSTITUTION.md`
- Language semantics: `tethers-0.1/SPEC.md`
