# Worker Note

Task: `J24K3c1 - Read-only exact publication-state observer`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `5fb9efa0f64b88217d677ad36bf1b0595d7d39d7`
Implementation checkpoint: `9e0ca9ac5c599adc9e5e4630198972a8946790ce`

## Requested outcome

Add one private read-only observer that accepts a validated publication intent and reports whether that exact transaction's staging directory, final destination, and installed record are present. The observer must preserve path and reparse safety, distinguish absence from invalid or inaccessible state, and perform no verification or mutation.

## Changes made

### installation_recovery.rs
- Added `InstallationRecoverySnapshot` — a crate-private owned struct with `staging_present: bool`, `destination_present: bool`, and `installed_record: Option<InstalledPlugRecord>`. Contains no paths, handles, references, or capabilities.
- Added `as_observation(&self, intent: &InstallationPublicationIntent) -> InstallationRecoveryObservation` — a zero-copy bridge that borrows the snapshot's record and intent for the J24K3b classifier.

### installed.rs
- Added `pub(crate) fn observe_installation_recovery(&self, intent: &InstallationPublicationIntent) -> Result<InstallationRecoverySnapshot>` on `InstalledPlugRegistry`.
- Added `intent_invalid()`, `recovery_conflict()`, and `recovery_io()` error constructors.
- Added `observe_directory(path: &Path) -> Result<bool>` — observes one exact path using `fs::symlink_metadata`, rejects reparse points, requires ordinary directory type, maps `NotFound` to `false`, maps non-ordinary entries to `recovery_conflict`, and maps metadata/chain failures to `recovery_io`.
- Added `observe_record(path: &Path) -> Result<Option<InstalledPlugRecord>>` — observes one exact path using `fs::symlink_metadata`, rejects reparse points, requires ordinary file type, reads and decodes with `strict_json`, maps `NotFound` to `None`, maps non-file or malformed entries to `recovery_conflict`, and maps read failures to `recovery_io`.
- Preserves `unsafe_store_path` from `reject_reparse` for symlink/junction/reparse refusals.

### installation_recovery_observation_tests.rs
- 16 direct filesystem tests prefixed `j24k3c1` using `InstalledPlugRegistry::open_existing` with real temp directories.
- Covers: empty state, staging only, destination only, record only, all three facts, snapshot-to-observation bridge, malformed/duplicate-key/unknown-field record JSON, record-as-directory, staging-as-file, destination-as-file, Windows junction/symlink refusal, unrelated entries untouched, read-only no-mutation, invalid intent rejection.

### lib.rs
- Registered `mod installation_recovery_observation_tests` under `#[cfg(test)]`.

## Decisions and assumptions

- J24K3c1 observes only the exact transaction named by one publication intent.
- The pure J24K3b classifier remains unchanged.
- Destination contents, current evidence, global installed-root consistency, cleanup, publication, and executor wiring remain later work.

## Evidence

- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`: passed.
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --lib j24k3c1 --locked`: 16 passed, 1007 filtered, 0 failed.
- `cargo nextest run --config-file .config/nextest.toml --manifest-path tethers-0.1/host-rust/Cargo.toml --all-features --locked -E 'test(j24k3c1)'`: 16 passed, 1247 skipped, 0 retries.
- J24K3b unit tests: 16 passed, 0 failed.
- J24K3a unit tests: 25 passed, 0 failed.
- J24K2 unit tests: 26 passed, 0 failed.
- J24J integration: 24 passed, 0 failed.
- M3 lifecycle: 13 passed, 0 failed (no handle-contention failure on this run).
- `$env:PATH = "$PSHOME;$env:PATH"; just verify`: 1023 lib tests passed, 0 failed; all integration suites passed; packet checker PASS.
- `git diff --check`: passed.
- `Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256`: `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB`.
- Implementation checkpoint: `9e0ca9ac5c599adc9e5e4630198972a8946790ce`.
- Final remote tip at evidence commit: `41c7286d2e8ac355a08eb46cfd077f74c71940c2`.

## Discoveries

- The observer requires `verify_chain` on both install and record roots before constructing exact child paths, ensuring the accepted root chain is rechecked at observation time.
- `reject_reparse` from `m3_store` is reused for all reparse/symlink checks; the observer maps its `unsafe_store_path` codes through unchanged and maps `store_io` to `recovery_io`.
- The existing `strict_json` primitive (which uses `parse_value_no_dupes`) handles duplicate-key and unknown-field rejection at the deserialization boundary — no new JSON validation was needed.
- `symlink_metadata` is used for all entry type checks to avoid following symlinks or junctions; `is_dir()` and `is_file()` checks follow the metadata (not the target) because symlink_metadata was used.
- The test helper uses `InstalledPlugRegistry::open_existing` on fresh temp directories created with `fs::create_dir_all` to avoid the `StoreRoot::open` auto-creation path.

## Remaining risks

Filesystem absence must not be inferred from broad `Path::exists` checks because those suppress errors and can follow unsafe path state. Exact entry observation must fail closed on reparse, non-ordinary, malformed, or inaccessible state.

## Smallest next action

OpenCode should read the task packet and accepted storage code, implement only the exact read-only observer, add direct filesystem tests, and return the branch for independent review.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/installation_recovery.rs`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/m3_store.rs`
