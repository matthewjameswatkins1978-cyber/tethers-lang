# Worker Note

Task: `J24K3c4 - Global installed-root consistency auditor`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `612b43eaa92d2142975d2dd4878561a1f60e4313`
Implementation checkpoint: `31c741b663e08ffd631004de7ca0d3556f5cedfe`
Final branch: `opencode/j24k3c4-installed-root-audit`

## Requested outcome

Add one crate-private, read-only global installed-root audit for J24K recovery. Given the optional current validated publication intent, prove that every direct final-form `plug-*` destination is accounted for by one validated installed record or by that one current intent.

The audit must detect and refuse untracked final destinations without adopting, deleting, repairing, or inspecting unrelated non-final entries. Existing installed records remain authoritative through accepted installed-state validation; an intent may authorize its one exact destination even before its record exists.

## Changes made

- `tethers-0.1/host-rust/src/installed.rs`: Added `destination_untracked()` error function and `InstalledPlugRegistry::audit_installation_recovery_destinations()` crate-private method. The audit validates an optional intent, requires both roots safe and present, loads and validates all installed records via `load_all()`, enforces `installed_id` canonical UUID and `installation_relative_path == plug-<installed_id>` for every record, detects duplicate destination claims, cross-checks optional intent against existing records (matching record at intent destination must equal `intent.installed_record` exactly), enumerates direct `plug-*` children, and returns stable recovery-facing errors. No public API, no mutation, no dependency changes.
- `tethers-0.1/host-rust/src/installation_recovery_audit_tests.rs`: 23 focused J24K3c4 production-entry-point tests.
- `tethers-0.1/host-rust/src/lib.rs`: Registered `installation_recovery_audit_tests` module.

## Evidence

- Focused Nextest: 23 passed, 0 failed, 0 skipped, 0 retries
- `cargo test --lib j24k3c4`: 23 passed
- J24K3c3 regression: 44 passed
- J24K3c2 regression: 21 passed
- J24K3c1 regression: 20 passed
- J24K3b regression: 16 passed
- J24K3a regression: 25 passed
- J24K2 regression: 26 passed
- J24J regression: 24 passed
- M3 lifecycle regression: 13 passed
- Full `just verify`: 1115 passed, 0 failed
- `cargo fmt --check`: pass
- Cargo.lock SHA-256: `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB`
- Task packet checker: pass
- `git diff --check`: clean (LF/CRLF warnings only, pre-existing)
- `git status --short`: only permitted files changed

## Discoveries

- `load_all()` already validates record digests and destination files; the audit adds canonical UUID identity, `plug-<installed_id>` path identity, duplicate destination detection, and global `plug-*` enumeration.
- `load_all()` enforces unique `(package_id, package_version)` per release; tests with multiple records required distinct package identities to avoid false `load_all()` failures.
- The existing `require_existing_recovery_root` helper reuses `verify_chain` + `reject_reparse` + `is_dir` guards, providing consistent root safety revalidation.

## Remaining risks

- The audit does not catch a `plug-*` non-directory entry that is not accounted for (returns `installation_destination_untracked` rather than `installation_recovery_conflict`) only when it is unaccounted; an accounted but non-directory entry correctly returns `installation_recovery_conflict`.
- Non-UTF-8 filenames in the install root are classified as `installation_recovery_conflict` per the task contract.
- J24K3b recovery classification, J24K3c1-c3 evidence revalidation, and the executor remain separate.

## Smallest next action

Return branch for independent review by Lucy. Do not merge.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/installation_recovery.rs`
