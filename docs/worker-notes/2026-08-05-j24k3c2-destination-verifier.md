# Worker Note

Task: `J24K3c2 - Exact recovery destination verifier`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `e8b80c13728cf45911880b42734cc4f19fe6d73e`
Implementation checkpoint: `89fd8a1880fe3a6938923c920f4ab711ad61b7d3`

## Requested outcome

Add one crate-private, read-only verifier that proves the exact final destination named by a validated publication intent matches the intent's precomputed installed record: exact file set, lengths, hashes, read-only permissions, and path/reparse safety. Also ensure already-opened registry roots still exist as ordinary safe directories before J24K3c1 observation or J24K3c2 verification proceeds.

## Changes made

### Production code (tethers-0.1/host-rust/src/installed.rs)
- Added `require_existing_recovery_root(path)` shared guard: calls `verify_chain`, preserves `unsafe_store_path`, requires the root to exist as an ordinary directory via `symlink_metadata`, rejects reparse points, and maps every other failure to `installation_recovery_io`.
- Added `require_existing_recovery_destination(path)` for the exact destination: same chain/reparse safety, but a missing destination returns `installation_recovery_conflict` and a non-directory destination returns `installation_recovery_conflict`.
- Replaced the two broad `verify_chain(...).map_err(|_| recovery_io())` calls in `observe_installation_recovery` with `require_existing_recovery_root` for both install and record roots.
- Added `pub(crate) fn verify_installation_recovery_destination(&self, intent: &InstallationPublicationIntent) -> Result<()>` that validates the intent, validates the install root, derives only `install_root / intent.destination_relative_path`, builds the expected file set from the precomputed record (`plug_json`, `payloads`, `signature_files`), recursively enumerates the exact destination, compares the collected set, and verifies each expected file's length, SHA-256 digest, and read-only state.
- Added `recovery_expected_files`, `recovery_expected_path`, and `collect_recovery_files` helpers to enforce the exact file universe, reject duplicate/unsafe/ambiguous expected paths, and collect only ordinary files while refusing reparse points.

### Test code
- Added `tethers-0.1/host-rust/src/installation_recovery_destination_tests.rs` with 21 `j24k3c2` tests.
- Added two `j24k3c1` regression tests in `installation_recovery_observation_tests.rs` proving missing install or record root returns `installation_recovery_io`.
- Registered the new test module in `lib.rs`.

## Decisions and assumptions

- Kimi K2.7Code is the selected implementation model for this bounded repository-reading and Rust verification package.
- This package verifies destination bytes and filesystem shape only.
- Current exact-candidate trust, launch profile, conformance, approval-chain freshness, global installed-root audit, recovery mutation, and executor wiring remain later packages.
- Existing public installation behaviour remains unchanged.

## Evidence

- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`: passed.
- `cargo nextest run --config-file .config/nextest.toml --manifest-path tethers-0.1/host-rust/Cargo.toml --all-features --locked -E 'test(j24k3c2)'`: 21 passed, 1247 skipped, 0 retries.
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --lib j24k3c2 --locked`: 21 passed, 1027 filtered, 0 failed.
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --lib j24k3c1 --locked`: 20 passed, 1028 filtered, 0 failed.
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --lib j24k3b --locked`: 16 passed, 1032 filtered, 0 failed.
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --lib j24k3a --locked`: 25 passed, 1023 filtered, 0 failed.
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --lib j24k2 --locked`: 26 passed, 1022 filtered, 0 failed.
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --test j24j_installation_reconciliation --locked`: 24 passed, 0 failed.
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --test m3_lifecycle --locked`: 13 passed, 0 failed.
- `$env:PATH = "$PSHOME;$env:PATH"; just verify`: lib tests 1048 passed, 0 failed; all integration suites passed; no handle-contention failure.
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`: PASS.
- `git diff --check`: passed.
- `Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256`: `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB`.
- Implementation checkpoint: `89fd8a1880fe3a6938923c920f4ab711ad61b7d3`.
- Evidence commit recorded by the worker: `6860a501546f165ba9e9bbd65109bae63c5faef2`.
- OpenCode final handoff tip: `de5e8f5d66fe012fac20bfc21c77f096bced33bf`.

## Discoveries

- `verify_chain` intentionally allows `NotFound` because it is also used before directory creation; for already-opened registry roots the new `require_existing_recovery_root` closes the gap by requiring `symlink_metadata` to return an ordinary directory.
- Windows junction fixture tests must remove the empty directory before `mklink /J` can create the junction at the same path.
- Duplicate expected paths in the precomputed record are rejected by `recovery_expected_files` before any destination enumeration, so the destination filesystem state cannot mask the conflict.
- `set_permissions(readonly=false)` on Windows reliably clears the read-only flag for the writable-file negative test; the verifier's `metadata.permissions().readonly()` check detects it as expected.

## Remaining risks

- Global installed-root consistency audit, recovery classification, staging cleanup, intent removal, record publication, lock integration, and executor wiring remain later packages.
- Current-authority and evidence-freshness revalidation are intentionally outside this package.
- No public API or dependency changes were made; this module is crate-private and read-only.

## Independent review

Lucy independently inspected the accepted-main ancestry, changed-file boundary, production root guards, exact destination verifier, expected-path normalization, destination-only traversal, immutable evidence checks, and direct tests at OpenCode handoff tip `de5e8f5d66fe012fac20bfc21c77f096bced33bf`.

No production or test correction was required. The only reviewer change was this documentation clarification distinguishing the worker's evidence commit from the actual OpenCode handoff tip.

The reported Rust and repository verification was not rerun personally by Lucy.

## Smallest next action

Implement the next bounded J24K3 package for current-authority and pinned evidence-chain revalidation. Global installed-root audit, mutation, publication, and executor wiring remain later work.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/installation_recovery.rs`
- `tethers-0.1/host-rust/src/m3_store.rs`
