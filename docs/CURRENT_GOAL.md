# Current Goal

Updated: 2026-08-09

## Goal

Complete the Tethers Foundation Pass through separately reviewed evidence
packages. F1–F8 are complete through warning enforcement. F9 is the current
documentation/operator-truth phase. F10 remains the sole Foundation
completion gate.

No new product capability is being added.

## Last accepted increment

F8 warning enforcement is accepted at
`5e616357963e70b86f59c870f6c00b7fbc94cb0a` on `origin/main`. The all-target
Rust Cargo check is warning-free and compiler warnings are denied in the
repository `just check` / `just verify` path. Broader Clippy advisory policy
is not globally denied.

## Active increment

F9 — Operator truth reconciliation — updates the current/operator-facing
documents to reflect the completed Foundation work through F8.

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
