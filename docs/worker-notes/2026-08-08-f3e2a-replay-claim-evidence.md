# Worker Note

Task: `F3e2a - Replay Claim evidence harvest`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `DeepSeek Pro HIGH`

Status: `COMPLETE`

Base commit: `dfae673407ecef38a9dcf8376b06ddbad4a97abc`

Implementation checkpoint: `5248af24cca2eab6306d77c870ed63bc08ef3592`

## Requested outcome

Audit Replay Claim / identity persistence only. Map every evidence dimension to
PROVEN/DISPROVEN/UNVERIFIED with exact test citations and hard assertions. Close
remaining gaps with characterization tests. Do not change production code.

## Changes made

- Added 1 characterization test (no production changes):
  `f3e2a_claim_filename_content_disagreement_fails_closed` (replay_windows.rs) —
  publishes a valid claim, renames the file to a different logical-key hex digest,
  then reopens; hard-asserts `PersistenceUnavailable`. Filename/content identity
  agreement was UNVERIFIED; now PROVEN.

- Updated `docs/CURRENT_CLINE_TASK.md` to describe F3e2a.
- Updated `docs/foundation-pass/PERSISTENCE_INVENTORY.md` with F3e2a Replay Claim
  evidence section.
- Created this worker note.

## Decisions and assumptions

- Replay Claim slice only — Replay Generations (0/1/2) explicitly deferred.
- No production code redesign. Test only.
- F3b UNVERIFIED platform properties preserved (never upgrade).
- Primitive-level F3b-3 evidence (flush, CREATE_NEW, rename) is not upgraded
  to Claim-store evidence.

## Evidence

| # | Property | Status | Test |
|---|---|---|---|
| 1 | Canonical logical-key identity | PROVEN | `sibling_actions_are_distinct`, `different_evaluations_are_distinct` (replay.rs) |
| 2 | Fresh immutable Claim creation | PROVEN | `claim_round_trip_is_exact_canonical_and_redacted` (replay.rs) |
| 3 | Execution identity creation | PROVEN | `ledger_05_fresh_claim_creates_one_host_execution_identity` (replay_windows.rs) |
| 4 | Close/reopen recovery of same Claim identity | PROVEN | `ledger_06_restart_recovers_same_execution_identity` (replay_windows.rs) |
| 5 | Existing Claim behaviour (collision) | PROVEN | `ledger_08_exact_claim_collision_recovers_only_valid_winner` (replay_windows.rs) |
| 6 | Conflicting binding behaviour | PROVEN | `ledger_09_binding_mismatch_fails_closed` (replay_windows.rs) |
| 7 | Malformed/noncanonical Claim handling | PROVEN | `non_canonical_or_unknown_claim_is_rejected` (replay.rs) |
| 8 | Claim digest corruption handling | PROVEN | `ledger_10_malformed_or_digest_invalid_claim_fails_closed` (replay_windows.rs) |
| 9 | Filename/content identity agreement | PROVEN | `f3e2a_claim_filename_content_disagreement_fails_closed` (replay_windows.rs) NEW |
| 10 | Collision/non-replacement at Claim boundary | PROVEN | `native_publication_survives_reopen_and_never_replaces` (replay_windows.rs) |
| 11 | Unexpected temporary/debris handling | PROVEN | `ledger_29_unexpected_ledger_entry_fails_closed` (replay_windows.rs) |
| 12 | Unsafe-path protection at Claim boundary | PROVEN | `relative_root_is_rejected_before_win32`, `unc_roots_are_rejected_before_win32`, `traversal_ads_and_separator_final_filenames_are_rejected`, `validated_child_retains_complete_independent_handle_chain` (replay_windows.rs) |
| 13 | Exact bytes/readback | PROVEN | `claim_round_trip_is_exact_canonical_and_redacted` (replay.rs), `ledger_30_restart_never_generates_new_uuid_for_existing_tuple` (replay_windows.rs) |

## Discoveries

- Filename/content identity agreement was the only UNVERIFIED dimension in the
  Replay Claim slice; now PROVEN with a direct store-level test.
- `scan_claims()` rejects non-`.claim.json` filenames and non-64-hex digests,
  preventing both debris injection and filename-based path traversal.
- No defect found. Replay Generations untouched.

## Remaining risks

- Power-loss durability: UNVERIFIED (F3b) — never upgrade
- Directory-entry durability: UNVERIFIED (F3b) — never upgrade
- Atomic visibility during rename: UNVERIFIED (F3b) — never upgrade
- Parent-directory flush in production: DISPROVEN (F3b)

## Smallest next action

Lucy reviews F3e2a evidence. F3e2b (Replay Generations evidence harvest) is the
next bounded subtask.

## References

- `docs/CURRENT_CLINE_TASK.md` — F3e2a task packet
- `docs/foundation-pass/PERSISTENCE_INVENTORY.md` — F3e2a Replay Claim evidence section
- `tethers-0.1/host-rust/src/replay.rs` — Claim identity module
- `tethers-0.1/host-rust/src/replay_windows.rs` — ReplayLedger, claim persistence, tests
