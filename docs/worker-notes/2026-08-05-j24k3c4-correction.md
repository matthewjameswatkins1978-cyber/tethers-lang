# Worker Note

Task: `J24K3c4 correction - preserve unsafe installed-state paths`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `READY`
Base commit: `WORKTREE`
Implementation checkpoint: `WORKTREE`

## Requested outcome

Correct one narrow independent-review finding in the otherwise sound J24K3c4 global installed-root consistency auditor.

`InstalledPlugRegistry::audit_installation_recovery_destinations` currently maps every `load_all()` failure to `installation_recovery_conflict`. This incorrectly converts explicit `unsafe_store_path` returned while validating a tracked installed destination or installed-record entry. Preserve explicit unsafe-path refusal, map genuine store access failure to `installation_recovery_io`, and keep malformed or contradictory installed state mapped to `installation_recovery_conflict`.

Do not redesign J24K3c4 or add later recovery work.

## Changes made

None yet.

## Decisions and assumptions

- DeepSeek Pro remains the implementation model because this is one surgical Rust error-mapping and regression-test correction.
- Existing direct final-namespace reparse handling is already correct and must remain unchanged.
- Public `load_all()` behaviour remains unchanged.

## Evidence

Not run yet.

## Discoveries

Independent review found that the existing Windows junction and Unix symlink tests exercise an untracked direct `plug-*` child. They do not exercise the earlier `load_all()` path used when a validated installed record tracks the reparse destination or when an installed-record entry itself is unsafe.

## Remaining risks

The correction must remain read-only and must not add recovery classification, cleanup, publication, intent removal, locking, planner, or executor wiring.

## Smallest next action

Apply only the correction packet, run focused and full verification, and return the branch for independent review.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/installation_recovery_audit_tests.rs`
