# Worker Note

Task: `J24K3a - Private publication intent record and atomic persistence`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `bff2d53a7951b8f32bbdfdfa62a67091a7f018cb`
Implementation checkpoint: `c41673ae8551b71a15194c92466bafeacbd78ca5`

## Requested outcome

Add the private crash-recovery publication-intent record and its single-record atomic persistence store. The package must pin one exact precomputed `InstalledPlugRecord`, use that record's `installed_id` as the transaction identity, validate all duplicated identity fields and digests, and safely create, load, and remove only `installation-intent/current.json`.

## Changes made

Added the private `InstallationPublicationIntent` record and
`InstallationPublicationIntentStore`, registered privately in `lib.rs`, with
direct crate-private behavioral tests. The constructor preserves the supplied
installed record and timestamp, uses its installed ID as the transaction ID,
and covers every intent field except the cleared intent digest with canonical
SHA-256 bytes. The store reuses `StoreRoot` for safe roots and atomic create,
accepts only empty or one ordinary `current.json`, and removes only an exact
validated match.

## Decisions and assumptions

- J24K3a contains persistence only.
- The precomputed installed record is supplied to the intent layer; J24K3a does not build, stage, publish, recover, or audit an installation.
- The installed record's `installed_id` is also the publication transaction identity, avoiding a second unrelated UUID.
- The intent store is private to the host crate and accepts only one canonical `current.json` record.

## Evidence

Exact evidence:

- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --lib j24k3a --locked`: 8 passed, 966 filtered, 0 failed.
- `cargo nextest run --config-file .config/nextest.toml --manifest-path tethers-0.1/host-rust/Cargo.toml --all-features --locked -E 'test(j24k3a)'`: 8 passed, 1206 skipped, 0 retries.
- J24K2 unit tests: 26 passed.
- J24J reconciliation: 24 passed.
- M3 lifecycle: 13 passed; the required suite was rerun serially after one concurrent-run handle-test failure and then passed.
- `$env:PATH = "$PSHOME;$env:PATH"; just verify`: 974 passed, 0 failed.
- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`: passed.
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`: passed before implementation and at final handoff.
- `git diff --check`: passed.
- Cargo.lock SHA-256: `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB`.
- Final remote tip: recorded after the documentation commit.

## Discoveries

- `StoreRoot::create_json("current", ...)` provides the required canonical,
  exclusive temporary, synced, same-directory atomic publication without
  changing the accepted persistence primitive.
- The focused Nextest expression discovered the new tests when their names
  carried the required `j24k3a` prefix.

## Remaining risks

The package is security-sensitive persistence. Malformed, torn, duplicated,
unknown, reparse-backed, mismatched, or stale intent state must never be
treated as absent or overwritten. Later J24K3 recovery still must validate the
destination and complete installed-record chain; that work is intentionally
outside this package.

## Smallest next action

Lucy should independently review the pushed five-file diff against the packet
and frozen architecture before acceptance.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/m3_store.rs`
