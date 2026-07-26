Task: `J09 durable replay protection`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `IN_PROGRESS`

Base commit: `055f52186ec2bf6adbd015b5684a57cd2152b8c0`

Starting checkpoint: `0543e253d1e9574aee40435a7d4000ae51ad473a`

## Requested outcome

Continue J09 from the proven native publication and provisioning checkpoint.
Add only the below-dispatch durable-ledger subsystem: cross-process logical-key
locking, immutable canonical redacted claims, immutable generation chains,
strict restart reconstruction and orphan detection, and deterministic
persistence fault seams. Do not begin dispatch integration.

## Changes made

Current durable-ledger checkpoint work is `IN_PROGRESS`. The existing
publication/provisioning evidence below is preserved as the starting
foundation; new implementation and numbered-case evidence will be added
progressively.

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
- Added a safe owning logical-key guard over share-denying `OPEN_ALWAYS`
  `CreateFileW` plus `LockFileEx(LOCKFILE_EXCLUSIVE_LOCK)` at byte offset zero
  for the one-byte range `low=1, high=0`. Handle drop or process termination
  releases the exclusion.
- Added the admitted `ReplayLedger`, strict whole-ledger scanning, immutable
  canonical redacted claim publication/recovery, exact binding comparison,
  immutable g0/g1/g2 publication, full predecessor/digest/identity validation,
  restart state reconstruction, keyed-temporary rejection, and whole-ledger
  orphan detection.
- Added bounded deterministic fault seams for lock open/acquisition, claim
  read/publication/collision reopen, chain validation, each generation
  publication, generation reopen, digest verification, restart scan, and
  orphan detection.
- Added 30 individually named ledger tests plus native populated-ledger and
  fault-matrix proofs. No dispatch, provider, Trail, Result Anchor, J05,
  planner, OCaml, protocol, manifest, retry, or compensation code changed.

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

Claim records contain exactly `record_kind`, `ledger_format_version`,
`logical_key_digest`, host-created canonical lower-case UUID-v4
`execution_id`, `execution_id_digest`, the complete redacted `binding`,
`binding_digest`, and `claim_digest`. Digests use
`sha256:<64 lower-case hex>` inside canonical RFC 8785/JCS records; filesystem
components use only the 64 lower-case hex. A valid existing claim recovers its
original UUID and binding. Malformed, non-canonical, digest-invalid,
filename-mismatched, or binding-mismatched claims fail closed. Publication
failure after a temporary has been created never returns usable admission.

Generation records contain ledger and record version, both identity digests,
generation number, state, predecessor, redacted `state_data`, and enclosing
`record_digest`. g0 and g1 carry an empty object; g2 carries only
`durable_outcome_digest`. The implementation accepts only
`intent_recorded -> invocation_armed -> succeeded|failed|uncertain`, requires
claim/g0/g1 predecessor links, and has no generation three. Before any new or
exact-existing generation is accepted, the complete current chain is reopened
and validated.

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
- Fresh native durable-ledger proof root
  `C:\Users\Matmus\Documents\j09p-76534f48` passed `cargo test replay
  -- --nocapture`: 55 passed, including explicit child-process same-key
  exclusion/release, process-termination release, different-key independence,
  all 30 named ledger cases, populated-ledger restart/reprovisioning, and every
  bounded persistence seam. Child state changes use explicit
  ready/release/result files; time is only a bounded liveness assertion.
- Final fresh native root
  `C:\Users\Matmus\Documents\j09p-f93e0655` passed the corrected focused suite
  with `59 passed`; full `cargo test` passed `392`; `cargo fmt`,
  `cargo fmt --check`, `cargo check`, packet checker
  `control-v1/IN_PROGRESS`, and `git diff --check` passed. `cargo check`
  reported only the nine pre-existing dead-code warnings.
- Independent Red review caught that recovered g0/g1 admissions could call the
  next publication method. `publish_armed` and `publish_terminal` now require
  the original fresh admission, as `publish_intent` already did. Focused
  restart tests prove recovered claim-only, g0, g1, and terminal admissions
  cannot publish any next state and leave the complete tree unchanged.
- The first native attempt used the redirected OneDrive Documents location and
  could not create test children. A second overly long diagnostic base exposed
  ordinary Win32 `MAX_PATH` fail-closed behaviour before claim temporary
  creation. The final proof used the previously accepted ACL-valid native root
  family at the explicit short path above. No path validation or publication
  rule was weakened.

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

## Progressive J09 verification matrix

Status is scoped to this checkpoint. `Done` means direct current or preserved
checkpoint evidence; `Partial` means the below-dispatch invariant is proved but
dispatch ordering remains; `Deferred` identifies the next dispatch batch.

| Case | Status | Evidence |
| --- | --- | --- |
| 1 | Done | `native_provisioning_is_exact_idempotent_and_non_repairing`; normal lookup is `ReplayLedger::open` only. |
| 2 | Done | `claim_round_trip_is_exact_canonical_and_redacted`; digest-only path constructors; no raw-data field exists. |
| 3 | Done | `ledger_05_fresh_claim_creates_one_host_execution_identity`; `substituted_execution_identity_is_rejected`. |
| 4 | Done | `ledger_06_restart_recovers_same_execution_identity`. |
| 5 | Deferred | Terminal reconstruction is proved; zero provider calls and duplicate-Anchor suppression require dispatch integration. |
| 6 | Done | `ledger_30_restart_never_generates_new_uuid_for_existing_tuple`. |
| 7 | Done | `ledger_07_sibling_actions_have_distinct_keys_claims_and_identities`. |
| 8 | Done | `ledger_06_restart_recovers_same_execution_identity` for the selected Action. |
| 9 | Done | `ledger_07_sibling_actions_have_distinct_keys_claims_and_identities`. |
| 10 | Done | `different_evaluations_are_distinct`. |
| 11 | Partial | `ledger_09_binding_mismatch_fails_closed`; approval/intent/provider zero-count ordering is deferred. |
| 12 | Partial | `ledger_05_fresh_claim_creates_one_host_execution_identity` and `ledger_21_claim_only_reconstructs_blocked_incomplete`; J05 ordering is deferred. |
| 13 | Partial | `ledger_12_valid_generation_zero_publication`; Trail/provider ordering is deferred. |
| 14 | Partial | `ledger_13_valid_generation_zero_to_one_transition`; provider-boundary ordering is deferred. |
| 15 | Partial | `ledger_14_each_valid_generation_two_terminal_state`; outcome/Anchor ordering is deferred. |
| 16 | Done | Cases 12-20 and pure `validate_chain` prove only 0 -> 1 -> 2 and the terminal vocabulary. |
| 17 | Done | Cases 21-26 reopen and validate the complete contiguous chain. |
| 18 | Done | Cases 10 and 15-20 plus 28-29 cover gaps, state, predecessor, checksum, malformed data, versions, and extensions. |
| 19 | Partial | g0 restart block is proved by case 22; Trail/provider zero counts are deferred. |
| 20 | Partial | g0 restart block is proved by case 22; provider zero counts are deferred. |
| 21 | Partial | g1 restart block is proved by case 23; provider zero-count integration is deferred. |
| 22 | Deferred | Durable J06 outcome ordering is dispatch integration. |
| 23 | Partial | Terminal restart block is proved by cases 24-26; duplicate-Anchor suppression is deferred. |
| 24 | Done | `ledger_29_unexpected_ledger_entry_fails_closed` uses an exact keyed claim temporary; no cleanup occurs. |
| 25 | Done | `ledger_27_orphan_chain_fails_whole_ledger_closed`. |
| 26 | Partial | `ledger_all_bounded_persistence_seams_fail_closed` plus preserved native publication/ACL failures; before-dispatch integration is deferred. |
| 27 | Partial | Real process exclusion is case 1 and sequential recovery is cases 6/8; a competing-admission provider proof is deferred. |
| 28 | Partial | Exclusion/recovery is proved; provider zero-count integration is deferred. |
| 29 | Done | `ledger_19_generation_collision_never_replaces_bytes` accepts exact fully reconstructed bytes and preserves different bytes. |
| 30 | Partial | Cases 1-4 prove native lock exclusion/failure; J05/provider ordering is deferred. |
| 31 | Deferred | J05 approval-consumption counts require dispatch integration. |
| 32 | Deferred | Fresh approved-Ask ordering requires dispatch integration. |
| 33 | Deferred | J05 consumption failure handling requires dispatch integration. |
| 34 | Deferred | Trail-intent, armed, deadline, and provider ordering require dispatch integration. |
| 35 | Deferred | J06 outcome/final/Anchor ordering requires dispatch integration. |
| 36 | Deferred | Outcome/final-generation failure and Anchor suppression require dispatch integration. |
| 37 | Partial | Cases 24-25 plus `recovered_terminal_admission_cannot_publish_or_mutate` prove terminal immutability; provider zero-count integration is deferred. |
| 38 | Partial | Cases 21-26 plus both recovered-admission non-mutation tests prove incomplete/uncertain manual-only state; no retry/compensation code exists; host result wiring is deferred. |
| 39 | Deferred | J12/J13 host-admission integration is the next batch. |
| 40 | Done for current scope | Focused native replay 59/59, full Rust 392/392, packet, formatting, compiler, whitespace, and complete-diff checks passed. OCaml/protocol code was unchanged. |
| 41 | Done | Preserved `native_local_fixed_ntfs_volume_is_accepted` and handle-bound volume validation. |
| 42 | Done | Preserved component-by-component reparse-safe handle admission and path rejection. |
| 43 | Done | `ledger_01_real_second_process_exclusion_and_release`. |
| 44 | Done | Preserved `native_publication_survives_reopen_and_never_replaces` and competing-publisher proof. |
| 45 | Done | Preserved native write/flush/rename/reopen/verification proof and test diagnostic seam. |
| 46 | Done | All J09 Win32 calls remain contained and safety-commented in `replay_windows.rs`. |
| 47 | Done | Cargo dependency diff is empty; publication still uses only the accepted handle rename. |
| 48 | Deferred | Missing host-data-root dispatch branch is not wired in this checkpoint. |
| 49 | Deferred | Approved-Ask host-data-root ordering is not wired in this checkpoint. |
| 50 | Done below dispatch | Replay APIs accept only the explicit root path and never inspect `TRAIL_PATH`. |
| 51 | Done | Preserved relative/missing-root rejection and no root creation. |
| 52 | Done below dispatch | `ReplayLedger::open` never provisions; dispatch call placement is deferred. |
| 53 | Done | Preserved exact provisioning test. |
| 54 | Done | Preserved non-mutating `AlreadyProvisioned`; populated valid ledger proof added. |
| 55 | Done | Preserved partial/unknown/version non-repair tests; keyed temporary now also fails closed. |
| 56 | Done | Preserved handle-bound owner equality validation. |
| 57 | Done | Preserved present/non-null DACL fail-closed branch. |
| 58 | Done | Preserved ACE walker and unrelated-write rejection. |
| 59 | Done | `unrelated_read_only_authority_is_safe`. |
| 60 | Done | Preserved current-user/System/Administrators trusted-writer set and live native root. |
| 61 | Done | Preserved independent retained handle-chain substitution test. |
| 62 | Done | `ledger_populated_valid_subtrees_reopen_without_reprovisioning`. |
| 63 | Partial | Existing unmatched/denied/pending-Ask regressions pass in the full Rust suite; explicit replay-open counts await dispatch integration. |

## Remaining risks

The publication, provisioning, cross-process lock, immutable claim/generation,
restart reconstruction, orphan scan, and bounded persistence-fault substrate
are now proven below dispatch. J09 as a whole remains `IN_PROGRESS`: J05/J06
ordering, Trail intent/outcome ordering, provider-boundary retention, duplicate
Result Anchor suppression, deadline seams, and reference-host dispatch
integration remain deliberately absent. This checkpoint must not be described
as J09 completion.

## Smallest next action

After independent review accepts this checkpoint, compile the next bounded J09
packet for dispatch ordering and integration. Do not begin that work from this
checkpoint.

## References

- `docs/J09_DURABLE_REPLAY_DESIGN.md`, native Windows substrate and publication
  sections
- `tethers-0.1/host-rust/src/replay_windows.rs`
- `tethers-0.1/host-rust/src/main.rs`
- `4a467256ba191522e7e350455112041f790e60d9`
