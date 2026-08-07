# Current Goal

Updated: 2026-08-07

## Goal

Continue the Tethers Foundation Pass from the accepted F2 operational
correctness handover into F3a, the bounded persistence inventory and vocabulary
checkpoint. The pass adds no product capability; it makes existing contracts
explicit and evidence-backed before any persistence repair is considered.

The accepted main is `83eec98a0f33f964623f4cbbf4548a76bbdf5255`.

## Last accepted increment

F2 — Operational correctness defects — is accepted and merged. It repaired
truthful live stderr visibility and the M3 handle allow-list test’s
handle-value-aliasing false positive without changing F1 fixtures, public
contracts, or dependencies. Its retained evidence branch is
`foundation/f2-operational-correctness` at the same accepted SHA; do not delete
that branch yet.

## Active increment

F3a — Persistence inventory and vocabulary — is `READY` on
`foundation/f3a-persistence-vocabulary`. It is documentation/evidence only:
classify every store, record its actual write/recovery/durability/path-safety
evidence, and identify uncertainty honestly. It does not authorise F3b Windows
primitive experiments or any persistence repair.

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
- F1 persistence/debt evidence: `docs/foundation-pass/PERSISTENCE_INVENTORY.md`
  and `docs/foundation-pass/DEBT_LEDGER.md`
- F1 worker note: `docs/worker-notes/2026-08-06-f1-baseline.md`
- F2 worker note: `docs/worker-notes/2026-08-07-f2-operational-correctness.md`
- Enduring principles: `docs/CONSTITUTION.md`
- Language semantics: `tethers-0.1/SPEC.md`
