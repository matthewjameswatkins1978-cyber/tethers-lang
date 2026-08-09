# Current Goal

Updated: 2026-08-09

## Goal

Tethers 0.2.2 has been released. The verified implementation checkpoint is
`c6ea1e1652fa2785a1f06e0ace2fcd5e826ee6ec`, published by advancing `main`
and tagging `v0.2.2`.

Foundation F1–F10 is COMPLETE and ACCEPTED. The accepted F10 final evidence
is at `5108b06f1f694d6523d5f3f342c08ca0f9b9cbc1`.

The next programme is Tethers 0.3 public Plug authoring. The first 0.3
implementation packet will be compiled by Lucy from the published `v0.2.2` tag.

## Last accepted increment

Tethers 0.2.2 published. Foundation F1–F10 merge and tagged `v0.2.2`
complete. Verified implementation checkpoint:
`c6ea1e1652fa2785a1f06e0ace2fcd5e826ee6ec`.

## Active increment

None — awaiting Lucy compilation of the first Tethers 0.3 implementation packet.

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
