Task: `J09 durable replay protection`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `055f52186ec2bf6adbd015b5684a57cd2152b8c0`

Starting checkpoint: `0543e253d1e9574aee40435a7d4000ae51ad473a`

Implementation checkpoint: `WORKTREE`

## Requested outcome

Complete J09 from the accepted durable-ledger checkpoint by wiring the
host-owned admission guard into the existing J05/J06 runtime. Preserve the
proven Windows substrate, add only the explicit `--host-data-root` normal-run
authority, and prove the frozen claim, consume, intent, armed, provider,
outcome, terminal-generation, Result Anchor, and replay-block ordering.

## Changes made

J09 implementation is complete in the independently reviewed worktree. The
existing publication/provisioning evidence below remains the accepted
foundation.

- Added strict normal-run parsing for one optional
  `--host-data-root <ABSOLUTE_PATH>` while preserving the separate
  `provision-replay` shape. Duplicate, missing-value, relative, and unknown
  options fail closed; no default or inferred replay root exists.
- Added a narrow lazy `replay_runtime` authority. It opens the already
  provisioned ledger only after fresh dispatch gates, owns the real admission
  guard through Result Anchor success or failure, and exposes only the four
  frozen redacted replay results through the existing `execution_status`
  field.
- Replaced planner-evaluation-derived execution identity at the dispatch seam
  with the host UUID recovered from the held replay admission. Planner
  evaluation ID and Action ID remain unchanged in the logical tuple, binding,
  request/response, provider, and Result Anchor seams.
- Split approved-Ask fresh precheck from one-shot consumption. A fresh claim
  now precedes consumption; recovered or unavailable replay state consumes
  nothing; consumption failure leaves claim-only manual resolution and never
  restores approval.
- Wired the exact runtime order: admission, optional J05 consume, g0, Trail
  intent, monotonic deadline start/check, g1, one provider call, J06
  classification, durable outcome, g2 bound to the canonical durable-outcome
  digest, one existing Result Anchor, then admission release.
- Preserved known J06 classifications when outcome, g2, or Result Anchor
  persistence fails. Outcome failure leaves g1; g2 failure leaves the durable
  outcome; Result Anchor failure leaves g2; none retries.
- Added 42 named counting-fake integration cases plus eight real native-ledger
  runtime cases for claim-only, g0, g1, success, failure, uncertain, binding
  mismatch, and fresh-success restart replay blocking.

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
- Runtime integration proof used new ACL-valid base
  `C:\Users\Matmus\Documents\j09ri-72269328`; every native test created a new
  provisioned child and no retained evidence root was reused as a host-data
  root. `cargo test replay -- --nocapture` passed 109, including all 42
  counting-fake runtime cases and all eight real file-backed runtime paths.
- Full native-enabled `cargo test` passed 442. `cargo fmt`,
  `cargo fmt --check`, and `cargo check` passed; `cargo check` retained only
  the six current non-test dead-code warnings.
- After the bounded Red correction, the first new root under redirected
  OneDrive Documents failed before replay logic with Win32 error 2 and was left
  untouched. A second fresh ACL-valid local root,
  `C:\Users\Matmus\Documents\j09ri-red2-37449f72`, passed the replay-focused
  native suite 109/109. `cargo test j05` passed 4/4; `cargo fmt`,
  `cargo fmt --check`, and `cargo check` passed with the same six warnings.
- `check-fixtures.ps1`, `test-engine.ps1`, `test-mcp-transcripts.ps1`,
  `test-host-denial.ps1`, `test-host-execution-failure.ps1`, and `demo.ps1`
  all passed. `opam exec -- dune build` passed in `engine-ocaml`.
- Final independent Red verification used three separate fresh ACL-valid roots:
  `C:\Users\Matmus\Documents\j09-red-focused-c0d51b35` passed the replay suite
  109/109; `C:\Users\Matmus\Documents\j09-red-full-c0d51b35` passed the full
  Rust suite 442/442; and
  `C:\Users\Matmus\Documents\j09-red-process-c0d51b35` passed the explicitly
  rerun second-process exclusion, termination-release, different-key
  independence, and eight native runtime/restart cases. The same review reran
  all six PowerShell scripts and the OCaml build successfully.
- The packet checker passed as `control-v1/COMPLETE`; final whitespace, status,
  stat, and complete-diff checks passed in the reviewed worktree.

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

## J09 verification matrix

| Case | Mapping |
| --- | --- |
| 1 | Done: `native_provisioning_is_exact_idempotent_and_non_repairing`; normal lookup remains `ReplayLedger::open`. |
| 2 | Done: `claim_round_trip_is_exact_canonical_and_redacted` and digest-only durable paths. |
| 3 | Done: `ledger_05_fresh_claim_creates_one_host_execution_identity` and `substituted_execution_identity_is_rejected`. |
| 4 | Done: `ledger_06_restart_recovers_same_execution_identity`. |
| 5 | Done: `j09_replay_runtime_native_fresh_success_restart_makes_zero_second_call`. |
| 6 | Done: `ledger_30_restart_never_generates_new_uuid_for_existing_tuple`. |
| 7 | Done: `ledger_07_sibling_actions_have_distinct_keys_claims_and_identities`. |
| 8 | Done: `ledger_06_restart_recovers_same_execution_identity`. |
| 9 | Done: `ledger_07_sibling_actions_have_distinct_keys_claims_and_identities`. |
| 10 | Done: `different_evaluations_are_distinct`. |
| 11 | Done: native binding-mismatch proof plus runtime case 16 prove zero approval consumption, Trail intent, and provider work. |
| 12 | Done: `j09_runtime_20_approved_ask_consumes_between_claim_and_g0` and native claim-only recovery. |
| 13 | Done: `j09_runtime_17_success_has_the_exact_observable_order` proves g0 before Trail intent. |
| 14 | Done: `j09_runtime_17_success_has_the_exact_observable_order` proves g1 before provider. |
| 15 | Done: runtime cases 17-19 prove durable outcome, matching g2 digest, then Anchor. |
| 16 | Done: ledger cases 12-20 and `validate_chain` accept only 0 -> 1 -> 2. |
| 17 | Done: ledger cases 21-26 select state only after full contiguous validation. |
| 18 | Done: ledger cases 10, 15-20, and 28-29 cover every malformed-chain class. |
| 19 | Done: `j09_runtime_23_trail_intent_failure_leaves_g0_and_zero_calls`. |
| 20 | Done: `j09_replay_runtime_native_g0_is_manual_without_provider`. |
| 21 | Done: `j09_replay_runtime_native_g1_is_manual_without_provider`. |
| 22 | Done: runtime cases 26-29 prove outcome-before-g2 and durable-outcome-without-g2 closure. |
| 23 | Done: `j09_runtime_30_anchor_failure_leaves_g2_without_retry` proves an Anchor failure after g2 creates no substitute or retry; `j09_replay_runtime_native_success_is_blocked_without_provider`, `j09_replay_runtime_native_failure_is_blocked_without_provider`, and `j09_replay_runtime_native_uncertain_is_manual_without_provider` prove recovered terminal states expose zero Anchors. |
| 24 | Done: `ledger_29_unexpected_ledger_entry_fails_closed` preserves keyed temporary evidence. |
| 25 | Done: `ledger_27_orphan_chain_fails_whole_ledger_closed`. |
| 26 | Done: `ledger_all_bounded_persistence_seams_fail_closed` plus runtime cases 16, 22, 25, and 29. |
| 27 | Done: `ledger_01_real_second_process_exclusion_and_release` and native fresh-success restart proof. |
| 28 | Done: `ledger_01_real_second_process_exclusion_and_release` proves cross-process same-key exclusion/release, composed with `j09_replay_runtime_native_fresh_success_restart_makes_zero_second_call` proving fresh success then restart makes zero second provider calls. |
| 29 | Done: `ledger_19_generation_collision_never_replaces_bytes`. |
| 30 | Done: ledger lock cases 1-4 plus runtime admission-failure case 16. |
| 31 | Done: recovered-state cases 10-15 table-drive all six states with a counting approval and consume zero; cases 20 and 39 prove fresh and unavailable gates. |
| 32 | Done: `j09_runtime_20_approved_ask_consumes_between_claim_and_g0`. |
| 33 | Done: `j09_runtime_21_approval_consumption_failure_leaves_claim_only` and J05 audit-failure regression. |
| 34 | Done: runtime cases 17, 23, 24, and 25 prove intent/deadline/armed/provider ordering. |
| 35 | Done: runtime cases 17-19 prove success/failure/uncertain outcome -> g2 -> one Anchor. |
| 36 | Done: runtime cases 26-30 prove outcome/g2/Anchor failure closure without retry. |
| 37 | Done: native success/failure recovered cases and `recovered_terminal_admission_cannot_publish_or_mutate`. |
| 38 | Done: native claim/g0/g1/uncertain cases are manual-only; no retry, compensation, restoration, or executor exists. |
| 39 | Done: `j09_runtime_33_binding_uses_exact_planner_ids_and_host_uuid_stays_local`. |
| 40 | Done: native replay 109/109, full Rust 442/442, all PowerShell regressions, OCaml build, packet, format, compiler, and diff checks. |
| 41 | Done: `native_local_fixed_ntfs_volume_is_accepted`. |
| 42 | Done: preserved component-by-component handle admission and reparse rejection tests. |
| 43 | Done: `ledger_01_real_second_process_exclusion_and_release`. |
| 44 | Done: `native_publication_survives_reopen_and_never_replaces` and competing publishers. |
| 45 | Done: preserved write/flush/rename/reopen/final-verification fault evidence. |
| 46 | Done: every unsafe Win32 call remains contained and documented in `replay_windows.rs`. |
| 47 | Done: dependency diff is empty from the accepted checkpoint; only authorised Windows dependencies remain. |
| 48 | Done: `j09_runtime_09_allow_without_root_is_persistence_unavailable`. |
| 49 | Done: `j09_runtime_39_approved_ask_missing_root_consumes_zero_approvals`. |
| 50 | Done: `j09_runtime_38_trail_and_replay_roots_remain_explicitly_distinct`. |
| 51 | Done: parser case 4 and native `relative_root_is_rejected_before_win32`; normal open never creates a missing root. |
| 52 | Done: runtime cases 6-8 and 36-37 prove non-dispatch paths never admit; `ReplayLedger::open` never provisions. |
| 53 | Done: `native_provisioning_is_exact_idempotent_and_non_repairing` proves the exact hierarchy. |
| 54 | Done: the same native provisioning test proves non-mutating `AlreadyProvisioned`. |
| 55 | Done: native partial, unknown-file, unknown-version, and keyed-temporary cases never repair. |
| 56 | Done: preserved handle-bound owner equality validation. |
| 57 | Done: preserved present, non-null DACL validation. |
| 58 | Done: `generic_write_is_rejected` and `unrelated_write_authority_is_rejected`. |
| 59 | Done: `unrelated_read_only_authority_is_safe`. |
| 60 | Done: `trusted_writer_is_accepted` and live current-token ACL proof. |
| 61 | Done: `validated_child_retains_complete_independent_handle_chain`. |
| 62 | Done: `ledger_populated_valid_subtrees_reopen_without_reprovisioning` and native runtime admission. |
| 63 | Done: runtime cases 6-8, 36, and 37 prove denied, Ask, unavailable, unmatched, and fresh-pending paths open no replay authority. |

## Remaining risks

No known J09 implementation risk remains inside the frozen scope. The
independent Red review and the complete verification suite passed. J10 queueing,
J11 deduplication, retry, compensation, and recovery execution remain
explicitly outside J09.

## Smallest next action

No further implementation action is authorised. This note accompanies the
task-authorised commit and branch push; stop afterward without beginning
J10/J11.

## References

- `docs/J09_DURABLE_REPLAY_DESIGN.md`, native Windows substrate and publication
  sections
- `tethers-0.1/host-rust/src/replay_windows.rs`
- `tethers-0.1/host-rust/src/main.rs`
- `4a467256ba191522e7e350455112041f790e60d9`
