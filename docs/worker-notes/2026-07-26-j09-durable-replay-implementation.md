Task: `J09 durable replay protection`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `IN_PROGRESS`

Base commit: `055f52186ec2bf6adbd015b5684a57cd2152b8c0`

Starting checkpoint: `4a467256ba191522e7e350455112041f790e60d9`

## Requested outcome

Continue J09 from the existing root-admission checkpoint. Add the frozen native
Windows no-replace immutable-publication primitive and the explicit
`provision-replay` hierarchy operation, without beginning locks, claims, or
dispatch wiring.

## Changes made

- Created local checkpoint `4a467256ba191522e7e350455112041f790e60d9` for
  the completed replay identity and Windows root-admission foundation.
- Added the `provision-replay <ABSOLUTE_HOST_DATA_ROOT>` command surface.
- Added validated leaf-name rejection, create-new temporary-file creation,
  partial-write handling, two file flushes, handle-based no-replace rename,
  close/reopen byte verification, and exact replay/v1 hierarchy validation in
  `src/replay_windows.rs`.
- Corrected publication to the documented Win32 form that succeeds on this
  host: `FileRenameInfo`, `ReplaceIfExists = FALSE`, `RootDirectory = NULL`,
  and the absolute final path derived only from the admitted directory plus a
  validated simple leaf.
- Retained the complete admitted ancestor-handle chain in every
  `ValidatedDirectory`, so the absolute target cannot be redirected after
  admission. Added an operating-system test proving an ancestor rename is
  denied until the child authority is dropped.
- Added native success, collision, competing-publisher, close/reopen,
  exact-byte, idempotent provisioning, partial-state, unknown-file, and
  unknown-version proofs. Provisioning now rejects a non-empty unprovisioned
  host root before creating anything.
- No lock, claim, generation, or dispatch changes were made.

## Decisions and assumptions

Lucy authorised a test-only native diagnostic seam for the blocked publication
primitive. It may expose only a stage identifier and numeric Win32 error code
to focused tests. It must never enter durable replay data, Trails, Result
Anchors, normal output, or production error text.

All direct Win32 calls remain in `src/replay_windows.rs`. Publication returns
only `ReplayError::PersistenceUnavailable` on every uncertain result and does
not delete the temporary file. The implementation does not use Trail storage
as a root and normal execution does not invoke provisioning.

The source temporary handle is opened with
`GENERIC_READ | GENERIC_WRITE | DELETE`, sharing `0`, `CREATE_NEW`,
`FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OPEN_REPARSE_POINT |
FILE_FLAG_WRITE_THROUGH`. Rename uses `FileRenameInfo` with replacement false.
The full zeroed aligned allocation is at least
`sizeof(FILE_RENAME_INFO) + filename bytes`; `FileNameLength` excludes all
trailing zero padding and `dwBufferSize` is the complete allocation length.

## Evidence

- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` passed
  before the checkpoint commit: `control-v1/IN_PROGRESS`, base `055f521`, HEAD
  `07df6e3`.
- Before checkpoint: `cargo fmt --check`, `cargo check`, `cargo test replay`
  (11 passed), and `git diff --check` all passed.
- After the publication work: `cargo fmt`, `cargo check`, and `cargo test
  replay` passed with 13 focused tests.
- Native test roots under `C:\Users\Matmus\Documents` inherited owner
  `MATMUS69-SCUMPU\Matmus`, full control for that owner, LocalSystem and
  Administrators, and read/execute only for `CodexSandboxUsers`; this passes
  the frozen owner/DACL rule.
- On two fresh valid roots, provisioning created only `replay/v1/locks`,
  `claims`, and `chains`, then returned `PersistenceUnavailable` before
  `FORMAT.json` appeared. Each retained exactly one unique `FORMAT.json.*.tmp`
  file. A repeat correctly treated that partial hierarchy as unavailable and
  performed no repair.
- The authorised test-only seam reproduced the failure on two further fresh,
  ACL-valid roots: `Rename`, Win32 error `87`. In each root, `FORMAT.json` was
  absent, exactly one temporary file remained, and that temporary file held the
  complete expected format bytes.
- The diagnostic seam is `#[cfg(test)]` only and stores only
  `NativePublishStage` plus a numeric Win32 error code. It is neither compiled
  into normal publication nor written to replay data, Trails, Result Anchors,
  normal output, or production errors.
- Windows SDK `10.0.22621.0` and `windows-sys 0.61.2` agree exactly:
  `sizeof(FILE_RENAME_INFO) = 24`, union offset `0` and size `4`,
  `RootDirectory` offset `8`, `FileNameLength` offset `16`, and `FileName`
  offset `20`.
- The temporary SDK C probe was compiled with MSVC 19.44 on Windows 11 Pro
  build `22631`. Its source and executable stayed outside the repository under
  `C:\Users\Matmus\AppData\Local\Temp\tethers-j09-native-probe-7b4f3ac8`.
  Exact results were:
  - null root plus simple leaf, `FileRenameInfo`: `Rename` / `17`;
  - existing share-denying authority handle, `FileRenameInfo`: `Rename` / `87`;
  - distinct `FILE_TRAVERSE | FILE_READ_ATTRIBUTES` directory handle with
    `FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE`,
    `FileRenameInfo`: `Rename` / `87`;
  - the same distinct handle, `FileRenameInfoEx`, `Flags = 0`:
    `Rename` / `87`;
  - null root plus full absolute path, `FileRenameInfo`: success / `0`.
  The zero-initialised SDK buffer already contained a trailing UTF-16 zero
  outside `FileNameLength`, so the compatibility-NUL hypothesis was also
  covered.
- A fresh Rust native run with the full absolute path succeeded through both
  flushes, close, reopen, and exact-byte verification. `FORMAT.json` existed,
  its temporary name was gone, and its exact bytes were
  `{"replay_format_version":1}`.
- `cargo test replay -- --nocapture` on fresh ACL-valid root
  `C:\Users\Matmus\Documents\tethers-j09-proof3-c15150cd21ab48168c8028f221956049`
  passed `19`; collision failed at `Rename` with Win32 `183`, left the complete
  losing temporary, and preserved the accepted final bytes. Two concurrent
  publishers produced exactly one accepted final, one fail-closed loser, and
  one retained loser temporary.
- The native tests also proved exact first provisioning, exact-byte reopen,
  non-mutating `AlreadyProvisioned`, partial-state rejection without repair,
  unknown-file rejection before mutation, and unknown-version rejection
  without repair.
- CLI proof on fresh root
  `C:\Users\Matmus\Documents\tethers-j09-cli-0100d0b8fc0d48468433d87dc1598abd`
  returned `Provisioned`, then `AlreadyProvisioned`. The exact tree, format
  last-write timestamp, and SHA-256
  `3810FE9BC501D99CF9EEEE56968D07CB9FF814388743E15989F3D0E666095EA3`
  were unchanged.
- Final verification passed: packet checker
  `control-v1/IN_PROGRESS`; `cargo fmt`; `cargo fmt --check`; `cargo check`;
  fresh-root `cargo test replay -- --nocapture` (`19 passed`, collision
  `Rename` / `183`); full `cargo test` (`352 passed`); and
  `git diff --check`.

## Discoveries

The root cause is not Rust layout, buffer alignment, buffer size, a missing
terminator, directory access, or directory sharing. The native SDK probe
reproduced every Rust result. On this Win32 wrapper/build, non-null
`RootDirectory` is rejected with error `87`; a null root plus a relative leaf
is resolved against the process current directory and therefore failed
cross-volume with error `17`. The documented Win32 null-root absolute-path form
is the only tested ordinary no-replace form that succeeded.

The absolute path does not become free-standing authority. It is derived
internally from a fully admitted absolute directory and a strict simple leaf.
The already-open source handle identifies the exact temporary file; sharing
zero prevents another open from renaming or deleting it; every admitted
ancestor and final directory handle remains live and share-denies deletion;
the focused substitution test proves that an ancestor cannot be renamed while
that authority exists; `ReplaceIfExists` remains false; and success still
requires the second flush, close, reopen, and exact-byte comparison.

## Remaining risks

The publication and explicit provisioning substrate is now proven, but J09 as
a whole remains in progress. The frozen fault-injection matrix beyond the
native publication branches, cross-process `LockFileEx` admission, claims,
generation chains, restart reconstruction, and dispatch ordering/wiring are
not implemented. This checkpoint must not be described as J09 completion.

## Smallest next action

After independent review accepts this checkpoint, compile the next bounded J09
packet for cross-process exclusion and immutable replay records. Do not begin
locks, claims, generations, or dispatch from this work item.

## References

- `docs/J09_DURABLE_REPLAY_DESIGN.md`, native Windows substrate and publication
  sections
- `tethers-0.1/host-rust/src/replay_windows.rs`
- `tethers-0.1/host-rust/src/main.rs`
- `4a467256ba191522e7e350455112041f790e60d9`
