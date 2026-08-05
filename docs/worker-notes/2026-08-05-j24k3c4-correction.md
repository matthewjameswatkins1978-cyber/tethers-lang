# Worker Note

Task: `J24K3c4 correction - preserve unsafe installed-state paths`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `READY`
Base commit: `WORKTREE`
Implementation checkpoint: `WORKTREE`

## Requested outcome

Correct one narrow production finding and one documentary finding in the otherwise sound J24K3c4 global installed-root consistency auditor.

`InstalledPlugRegistry::audit_installation_recovery_destinations` currently maps every `load_all()` failure to `installation_recovery_conflict`. This incorrectly converts explicit `unsafe_store_path` returned while validating a tracked installed destination or installed-record entry. Preserve explicit unsafe-path refusal, map genuine store access failure to `installation_recovery_io`, and keep malformed or contradictory installed state mapped to `installation_recovery_conflict`.

The completed original J24K3c4 worker note also retains duplicated stale scaffold sections saying `Not run yet` after its real passing evidence. Remove those duplicate stale sections so the audit record is internally consistent.

Do not redesign J24K3c4 or add later recovery work.

## Changes made

None yet.

## Decisions and assumptions

- DeepSeek Pro remains the implementation model because this is one surgical Rust error-mapping and regression-test correction.
- Existing direct final-namespace reparse handling is already correct and must remain unchanged.
- Public `load_all()` behaviour remains unchanged.
- The original worker-note cleanup is documentary only and must not rewrite its truthful completed evidence.

## Evidence

Not run yet.

## Discoveries

- The existing Windows junction and Unix symlink tests exercise an untracked direct `plug-*` child. They do not exercise the earlier `load_all()` path used when a validated installed record tracks the reparse destination or when an installed-record entry itself is unsafe.
- `docs/worker-notes/2026-08-05-j24k3c4-installed-root-audit.md` contains a second stale `Evidence`, `Discoveries`, and `Remaining risks` block left from its scaffold, including the contradictory text `Not run yet`.

## Remaining risks

The correction must remain read-only and must not add recovery classification, cleanup, publication, intent removal, locking, planner, or executor wiring.

## Smallest next action

Apply only the correction packet, run focused and full verification, clean the duplicate stale worker-note tail, and return the branch for independent review.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-05-j24k3c4-installed-root-audit.md`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/installation_recovery_audit_tests.rs`
