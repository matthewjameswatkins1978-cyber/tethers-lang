# Current Goal

Updated: 2026-08-10

## Goal

Tethers 0.3 public Plug authoring. P1 (Generic Operational Scope Evidence) is
ACCEPTED. P2 is ACTIVE: P2A (public deterministic plug pack) is implemented
and awaiting Lucy review.

## Last accepted increment

P1 Generic Operational Scope Evidence accepted at
`270a5913a93d64256113cca3450619c484b7ddc7`.

## Active increment

P2A — Public deterministic plug pack. Implementation branch:
`feature/0.3-p2a-public-plug-pack`. Awaiting Lucy independent review.

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
