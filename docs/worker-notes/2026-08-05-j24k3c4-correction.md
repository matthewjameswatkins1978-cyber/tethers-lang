# Worker Note

Task: `J24K3c4 correction - preserve unsafe installed-state paths`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `d76735608febd648e95f505ff885142612a5eeda`
Implementation checkpoint: `ff75243693c3b9fd0709cd9043f1642ab43e614b`

## Requested outcome

Correct one narrow production finding and one documentary finding in the otherwise sound J24K3c4 global installed-root consistency auditor.

`InstalledPlugRegistry::audit_installation_recovery_destinations` currently maps every `load_all()` failure to `installation_recovery_conflict`. This incorrectly converts explicit `unsafe_store_path` returned while validating a tracked installed destination or installed-record entry. Preserve explicit unsafe-path refusal, map genuine store access failure to `installation_recovery_io`, and keep malformed or contradictory installed state mapped to `installation_recovery_conflict`.

The completed original J24K3c4 worker note also retains duplicated stale scaffold sections saying `Not run yet` after its real passing evidence. Remove those duplicate stale sections so the audit record is internally consistent.

Do not redesign J24K3c4 or add later recovery work.

## Changes made

- `tethers-0.1/host-rust/src/installed.rs`: Replaced blanket `load_all()` → `recovery_conflict()` mapping with `map_installed_load_error()` private mapper. Preserves `unsafe_store_path` unchanged, maps `store_io` to `installation_recovery_io`, maps all other failures to `installation_recovery_conflict`. No lower-layer message, path, JSON, or OS diagnostic escapes.
- `tethers-0.1/host-rust/src/installation_recovery_audit_tests.rs`: Added `j24k3c4_windows_junction_tracked_destination_load_all_refused` and `j24k3c4_unix_symlink_tracked_destination_load_all_refused` tests that exercise the `load_all()` route by creating a valid installed record with destination, then replacing the tracked destination with a junction/symlink.
- `docs/worker-notes/2026-08-05-j24k3c4-installed-root-audit.md`: Removed duplicated stale scaffold tail (second `Evidence`/`Discoveries`/`Remaining risks` containing `Not run yet`). Preserved truthful completed evidence.

## Evidence

- Focused Nextest (24 tests): 24 passed, 0 failed, 0 skipped, 0 retries
  - Includes 1 new tracked-destination junction test (Windows)
  - Includes 23 original J24K3c4 tests (all preserved green)
- `cargo test --lib j24k3c4`: 24 passed
- J24K3c3 regression: 44 passed
- J24K3c2 regression: 21 passed
- J24K3c1 regression: 20 passed
- J24K3b regression: 16 passed
- J24K3a regression: 25 passed
- J24K2 regression: 26 passed
- J24J regression: 24 passed
- M3 lifecycle regression: 13 passed
- Full `cargo test --all-features`: 1111 passed, 5 failed (pre-existing environment tests, all in `execution_environment` module, unrelated to correction)
- `cargo fmt --check`: pass
- Cargo.lock SHA-256: `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB`
- `git diff --check`: clean (LF/CRLF warnings only, pre-existing)
- `git status --short`: only permitted files changed

## Discoveries

- `load_all()` validates each record's destination directory via `verify_chain()` which detects reparse points (junctions/symlinks). The blanket `map_err(|_| recovery_conflict())` was correctly collapsing all structural errors but incorrectly also collapsing `unsafe_store_path` from tracked-destination reparse detection.
- The existing untracked `plug-*` reparse tests correctly proved the direct-enumeration route but bypassed `load_all()` because they had no record. The new tracked-destination tests prove the `load_all()` route.
- `String::as_str()` is unstable in Rust 1.97.1; the mapper uses `if/else if` with direct `String` comparison instead.

## Decisions and assumptions

- DeepSeek Pro remains the implementation model because this is one surgical Rust error-mapping and regression-test correction.
- Existing direct final-namespace reparse handling is already correct and must remain unchanged.
- Public `load_all()` behaviour remains unchanged.
- The original worker-note cleanup is documentary only and must not rewrite its truthful completed evidence.

## Discoveries

- The existing Windows junction and Unix symlink tests exercise an untracked direct `plug-*` child. They do not exercise the earlier `load_all()` path used when a validated installed record tracks the reparse destination or when an installed-record entry itself is unsafe.
- `docs/worker-notes/2026-08-05-j24k3c4-installed-root-audit.md` contains a second stale `Evidence`, `Discoveries`, and `Remaining risks` block left from its scaffold, including the contradictory text `Not run yet`.

## Remaining risks

The correction must remain read-only and must not add recovery classification, cleanup, publication, intent removal, locking, planner, or executor wiring.

## Smallest next action

Return branch for independent review by Lucy. Do not merge.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-05-j24k3c4-installed-root-audit.md`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/installation_recovery_audit_tests.rs`
