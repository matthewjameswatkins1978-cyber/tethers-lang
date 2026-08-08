# Worker Note

Task: `F3e2b - Replay Generations & Recovery evidence harvest`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `DeepSeek Pro HIGH`

Status: `COMPLETE`

Base commit: `477e2b901c0dfec55f4df6f9dca79a66e9294e0a`

Implementation checkpoint: `42f4d289783f15a59cbedf32ab510fc419fa26a5`

## Requested outcome

Audit Replay Generations 0/1/2 and their durable restart reconstruction only. Map every evidence dimension to PROVEN/DISPROVEN/UNVERIFIED with exact test citations and hard assertions. Do not change production code. Do not re-audit Replay Claim identity (accepted F3e2a).

F3e2b-R1 repair: close two Generation evidence gaps — filename/content agreement (#13) and exact bytes close/reopen (#14) — with direct characterization tests exercising the production Generation reader.

## Changes made

- Added 2 characterization tests (no production changes):
  - `f3e2b_generation_exact_bytes_survive_close_and_reopen` (replay_windows.rs) — publishes G0/G1/G2, reads file bytes, drops/reopens ledger, asserts all three generation files have identical bytes after close/reopen.
  - `f3e2b_generation_filename_content_disagreement_fails_closed` (replay_windows.rs) — publishes G0+G1, swaps filenames (G0↔G1 via 3-way rename), asserts ledger open returns `PersistenceUnavailable`.

- Updated `docs/CURRENT_CLINE_TASK.md` to describe F3e2b and F3e2b-R1.
- Updated `docs/foundation-pass/PERSISTENCE_INVENTORY.md` with F3e2b Replay Generations evidence section.
- Created this worker note.

## Decisions and assumptions

- Replay Generation slice only — Replay Claim identity explicitly excluded (accepted F3e2a).
- Dimensions #13 and #14 were marked PROVEN in the initial F3e2b based on reconstruction pipeline, but lacked direct characterization tests. F3e2b-R1 adds those tests.
- F3b UNVERIFIED platform properties preserved (never upgrade).
- F3b UNVERIFIED platform properties preserved (never upgrade).

## Evidence

| # | Property | Status | Test | Exact Hard Assertion |
|---|---|---|---|---|
| 1 | Canonical Generation representation | PROVEN | `generation_three_is_not_representable_or_parseable` (replay.rs:657) negative; reconstruction tests (ledger 21–26) positive | G3 bytes → `assert!(Generation::from_canonical_bytes(&bytes).is_err())`; reconstruction: publish→reopen→state matches expected |
| 2 | Generation 0 publication | PROVEN | `ledger_12_valid_generation_zero_publication` (replay_windows.rs:2287) | `assert_eq!(admission.state(), ReplayState::IntentRecorded)`; `assert_eq!(directory_names(&path), vec!["g0000000000000000.json"])` |
| 3 | G0 → G1 transition | PROVEN | `ledger_13_valid_generation_zero_to_one_transition` (replay_windows.rs:2302) | `assert_eq!(admission.state(), ReplayState::InvocationArmed)` |
| 4 | G2 terminal states (all 3) | PROVEN | `ledger_14_each_valid_generation_two_terminal_state` (replay_windows.rs:2316) | Loop over all 3: `assert_eq!(admission.state(), state)` |
| 5 | Missing-generation rejection | PROVEN | `ledger_15_generation_one_without_zero_is_rejected` (replay_windows.rs:2340), `ledger_16_generation_two_without_one_is_rejected` (replay_windows.rs:2363) | G1-only → open err; G0+G2 skip G1 → open err |
| 6 | Illegal state-at-generation | PROVEN | `ledger_17_illegal_state_transition_is_rejected` (replay_windows.rs:2398); `chain_cannot_skip_armed` (replay.rs:644) | G0 with "succeeded" → open err; G0→G2 direct → validate_chain err |
| 7 | Predecessor-digest linkage | PROVEN | `ledger_18_predecessor_mismatch_is_rejected` (replay_windows.rs:2428) | G1 with tampered predecessor_digest → open err |
| 8 | Generation immutability / non-replacement | PROVEN | `ledger_19_generation_collision_never_replaces_bytes` (replay_windows.rs:2463) | Conflicting file → `publish_armed().is_err()`; `assert_eq!(read(collision), b"different-immutable-bytes")` |
| 9 | Generation upper bound | PROVEN | `generation_three_is_not_representable_or_parseable` (replay.rs:657), `generation_filename(3)` (replay_windows.rs:1213), `ledger_20_generation_three_is_rejected` (replay_windows.rs:2493) | Model/parser/persistence all reject generation≥3 |
| 10 | Restart reconstruction (6 variants) | PROVEN | ledger_21–26 (replay_windows.rs:2516–2614) | 6 state assertions after reopen |
| 11 | Recovered admissions cannot advance history | PROVEN | `recovered_claim_g0_and_g1_admissions_cannot_advance_or_mutate` (replay_windows.rs:2616); `recovered_terminal_admission_cannot_publish_or_mutate` (replay_windows.rs:2653) | All mutations → `Err(PersistenceUnavailable)`; `assert_eq!(tree_snapshot(&root), before)` |
| 12 | Malformed/corrupt chain fail-closed | PROVEN | `ledger_28_malformed_chain_fails_closed`, `ledger_27_orphan_chain_fails_whole_ledger_closed`, ledger_17, ledger_18 | Malformed JSON/orphan/tampered → open err |
| 13 | Filename/content agreement | PROVEN | `f3e2b_generation_filename_content_disagreement_fails_closed` (replay_windows.rs) NEW | Swap filenames → `assert!(ReplayLedger::open(&root).is_err())` |
| 14 | Exact bytes / close-reopen | PROVEN | `f3e2b_generation_exact_bytes_survive_close_and_reopen` (replay_windows.rs) NEW; `ledger_30_restart_never_generates_new_uuid_for_existing_tuple` (replay_windows.rs:2741); `ledger_populated_valid_subtrees_reopen_without_reprovisioning` (replay_windows.rs:2769) | `assert_eq!(g0_after, g0_before)` etc. for all 3 |

## Discoveries

- The initial F3e2b evidence harvest found all 14 dimensions PROVEN through the existing test suite, but dimensions #13 (filename/content agreement) and #14 (exact bytes close/reopen) relied on the reconstruction pipeline rather than dedicated characterization tests.
- F3e2b-R1 adds two direct characterization tests that exercise the production Generation reader.
- All 124 Replay tests pass (122 existing + 2 new).
- No defect found in the Replay Generation slice.

## Remaining risks

- Power-loss durability: UNVERIFIED (F3b) — never upgrade
- Directory-entry durability: UNVERIFIED (F3b) — never upgrade
- Atomic visibility during rename: UNVERIFIED (F3b) — never upgrade
- Parent-directory flush in production: DISPROVEN (F3b)

## Smallest next action

Acceptance review by Lucy. F3e2b is the final evidence harvest for Replay Generations.

## References

- `docs/CURRENT_CLINE_TASK.md` — task packet (F3e2b)
- `docs/foundation-pass/PERSISTENCE_INVENTORY.md` — F3e2b evidence section appended
- `tethers-0.1/host-rust/src/replay.rs` — Generation model and validate_chain
- `tethers-0.1/host-rust/src/replay_windows.rs` — Generation persistence, reconstruction, ledger tests
