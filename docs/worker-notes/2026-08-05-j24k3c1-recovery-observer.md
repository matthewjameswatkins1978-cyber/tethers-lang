# Worker Note

Task: `J24K3c1 correction - preserve unsafe root-chain refusal`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `77438622e431e68bb3c57c1b89fd23abbdf68e34`
Implementation checkpoint: `0a83c036c3a446e63e1587cd220f8988e08683f4`

## Requested outcome

Apply one bounded correction to the otherwise accepted J24K3c1 observer. Preserve an explicit `unsafe_store_path` returned while rechecking either accepted registry root chain instead of collapsing it into `installation_recovery_io`.

## Changes made

### Production correction (installed.rs)
- Added `map_recovery_path_error(error: M3Error) -> M3Error` that returns the original error unchanged when `error.code == "unsafe_store_path"` and maps all other errors to `recovery_io()`.
- Changed both root-chain checks in `observe_installation_recovery` to use `map_recovery_path_error` instead of the broad `|_| recovery_io()` erasure.

### Direct tests (installation_recovery_observation_tests.rs)
- Added `j24k3c1_windows_junction_install_root_verify_chain_is_refused`: opens a registry, replaces the already-opened install root directory with a Windows junction, and verifies `observe_installation_recovery` returns `unsafe_store_path`.
- Added `j24k3c1_windows_junction_record_root_verify_chain_is_refused`: opens a registry, replaces the already-opened record root directory with a Windows junction, and verifies `observe_installation_recovery` returns `unsafe_store_path`.
- Retained all sixteen existing `j24k3c1` tests unchanged. Total: 18 tests.

## Decisions and assumptions

- The snapshot, exact-path observer, strict record decoding, classifier bridge, and all existing state semantics remain unchanged.
- Only explicit `unsafe_store_path` from install-root or record-root chain verification must pass through unchanged.
- Other root-chain observation failures remain the stable `installation_recovery_io` error.
- No broader path API, mutation, destination verification, audit, or executor integration belongs in this correction.

## Evidence

- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`: passed.
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --lib j24k3c1 --locked`: 18 passed, 1007 filtered, 0 failed.
- `cargo nextest run --config-file .config/nextest.toml --manifest-path tethers-0.1/host-rust/Cargo.toml --all-features --locked -E 'test(j24k3c1)'`: 18 passed, 1247 skipped, 0 retries.
- J24K3b unit tests: 16 passed, 0 failed.
- J24K3a unit tests: 25 passed, 0 failed.
- J24K2 unit tests: 26 passed, 0 failed.
- J24J integration: 24 passed, 0 failed.
- M3 lifecycle: 13 passed, 0 failed (no handle-contention failure).
- `$env:PATH = "$PSHOME;$env:PATH"; just verify`: 1025 lib tests passed, 0 failed; all integration suites passed; packet checker PASS.
- `git diff --check`: passed.
- `Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256`: `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB`.
- Implementation checkpoint: `0a83c036c3a446e63e1587cd220f8988e08683f4`.
- OpenCode final remote tip reviewed by Lucy: `5936c6e28d9c00a866e1d9de28316fbcc0cb8c5c`.
- The subsequent reviewer documentation commit changes only this worker note and does not alter production code or test evidence.

## Discoveries

The previous worker note stated that unsafe-path errors from root-chain rechecking were preserved, but the production code preserved them only for exact child `reject_reparse` calls. The correction now applies the same narrow preservation rule to both accepted root chains.

The junction fixtures require removing the empty opened root directory before `mklink /J` can recreate that exact path as a junction. `verify_chain` detects the replacement because it checks every existing ancestor for reparse state.

## Remaining risks

Later J24K3 work still owns destination-content verification, evidence revalidation, global installed-root audit, recovery mutation, and executor wiring. None of those concerns belongs in J24K3c1.

## Smallest next action

Lucy should complete independent review and fast-forward the accepted branch to `main` if the branch remains a clean descendant of accepted main.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/installation_recovery_observation_tests.rs`
- `tethers-0.1/host-rust/src/m3_store.rs`
