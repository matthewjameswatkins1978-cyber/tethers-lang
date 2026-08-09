# Current Goal

Updated: 2026-08-09

## Goal

Prepare the complete Tethers 0.2.2 release candidate for Lucy review and
publication.

Foundation F1–F10 is COMPLETE and ACCEPTED. The accepted F10 final evidence
is at `5108b06f1f694d6523d5f3f342c08ca0f9b9cbc1`.

0.2.2 is a patch release-hardening release. It adds no new product capability.
Language semantics remain 0.1.

`main` has NOT yet been advanced. `v0.2.2` has NOT yet been created.
Publication requires Lucy review after the candidate completes.

## Last accepted increment

Foundation F10 clean-checkout proof independently accepted at
`5108b06f1f694d6523d5f3f342c08ca0f9b9cbc1` (Foundation branch lineage).
F1–F10 have not yet been merged to `origin/main`; live `main` remains
`40ec42eb2aac108901d428af3cbfe264d3edd6dc`.

## Active increment

Tethers 0.2.2 release preparation — version identity, Cargo single-source,
fixture migration, release notes, README front door, and Foundation recording.

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
