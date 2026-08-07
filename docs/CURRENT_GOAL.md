# Current Goal

Updated: 2026-08-07

## Goal

Continue the Tethers Foundation Pass from the accepted F3a persistence
vocabulary. F3b builds direct Windows primitive evidence for every durability
and safety property identified as unverified by F3a, using isolated
characterization tests and platform investigation.

The accepted main is `145a791ceb3f5e3b8855aeadbac83671d9a2b363`.

## Last accepted increment

F3a — Persistence inventory and vocabulary — is accepted and merged. Its
evidence branch `foundation/f3a-persistence-vocabulary` is retained at the same
accepted SHA.

## Active increment

F3b — Windows persistence primitive evidence — is `IN_PROGRESS` on
`foundation/f3b-windows-persistence-evidence`. It adds characterization tests
and documentation; it does not repair, redesign, or change production
persistence behaviour.

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
- F3a worker note: `docs/worker-notes/2026-08-07-f3a-persistence-vocabulary.md`
- Enduring principles: `docs/CONSTITUTION.md`
- Language semantics: `tethers-0.1/SPEC.md`
