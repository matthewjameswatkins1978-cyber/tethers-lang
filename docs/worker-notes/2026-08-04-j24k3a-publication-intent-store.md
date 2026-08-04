# Worker Note

Task: `J24K3a - Private publication intent record and atomic persistence`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `OpenCode`
Status: `COMPLETE`
Base commit: `bff2d53a7951b8f32bbdfdfa62a67091a7f018cb`
Implementation checkpoint: `5b3d2d67c881e21b1ae037ff6710999bb2e4351f`

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

After Lucy's independent review, applied a correction pass that:

- Maps every lower-layer error to the stable intent contract
  (`installation_intent_invalid`, `installation_intent_conflict`,
  `installation_intent_io`), preserving `unsafe_store_path` for reparse/symlink
  refusal.
- Adds direct proofs for valid mismatched removal, true multiple-entry states,
  structural identity equality invariants, noncanonical UUID spelling, embedded
  record digest coverage, and store-level properties including canonical bytes,
  create idempotency, strict deserialization, relative-root refusal, and
  Windows junction refusal.

## Decisions and assumptions

- J24K3a contains persistence only.
- The precomputed installed record is supplied to the intent layer; J24K3a does not build, stage, publish, recover, or audit an installation.
- The installed record's `installed_id` is also the publication transaction identity, avoiding a second unrelated UUID.
- The intent store is private to the host crate and accepts only one canonical `current.json` record.

## Evidence

Exact evidence:

- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --lib j24k3a --locked`: 25 passed, 966 filtered, 0 failed.
- `cargo nextest run --config-file .config/nextest.toml --manifest-path tethers-0.1/host-rust/Cargo.toml --all-features --locked -E 'test(j24k3a)'`: 25 passed, 1206 skipped, 0 retries.
- J24K2 unit tests: 26 passed.
- J24J reconciliation: 24 passed.
- M3 lifecycle: 13 passed.
- `$env:PATH = "$PSHOME;$env:PATH"; just verify`: 991 lib tests passed, 0 failed; all integration suites passed on the second run after one unrelated Windows file-handle contention failure in `j24c_plug_disable_cli::corrupt_forked_chain_fails_without_mutation`, which passed when rerun serially.
- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`: passed.
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`: passed at correction handoff.
- `git diff --check`: passed.
- Cargo.lock SHA-256: `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB`.
- Final remote tip at evidence commit: to be pinned after the documentation commit.

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
