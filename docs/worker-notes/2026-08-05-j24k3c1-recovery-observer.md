# Worker Note

Task: `J24K3c1 correction - preserve unsafe root-chain refusal`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `READY`
Base commit: `WORKTREE`
Implementation checkpoint: `WORKTREE`

## Requested outcome

Apply one bounded correction to the otherwise accepted J24K3c1 observer. Preserve an explicit `unsafe_store_path` returned while rechecking either accepted registry root chain instead of collapsing it into `installation_recovery_io`.

## Changes made

No production correction has been applied yet. The reviewed observer at `4bf11118369d5dc1d7ae50d4b1b86be969b96db9` correctly handles exact child entries but currently maps every `verify_chain` root error to `installation_recovery_io`.

## Decisions and assumptions

- The snapshot, exact-path observer, strict record decoding, classifier bridge, and all existing state semantics remain unchanged.
- Only explicit `unsafe_store_path` from install-root or record-root chain verification must pass through unchanged.
- Other root-chain observation failures remain the stable `installation_recovery_io` error.
- No broader path API, mutation, destination verification, audit, or executor integration belongs in this correction.

## Evidence

Independent review found the mismatch in the two root `verify_chain(...).map_err(|_| recovery_io())` calls. Existing exact-child `reject_reparse` handling already preserves `unsafe_store_path` correctly.

## Discoveries

The previous worker note stated that unsafe-path errors from root-chain rechecking were preserved, but the production code preserves them only for exact child `reject_reparse` calls.

## Remaining risks

The correction must be demonstrated through the production observer after a previously opened install root or record root is replaced by an accepted platform symlink, junction, or reparse fixture.

## Smallest next action

Preserve explicit root-chain `unsafe_store_path`, add direct install-root and record-root regression coverage, and rerun the complete J24K3c1 verification packet.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/installation_recovery_observation_tests.rs`
- `tethers-0.1/host-rust/src/m3_store.rs`
