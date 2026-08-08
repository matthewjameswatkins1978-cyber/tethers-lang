# Current Implementation Task

Control contract: `1`
Task: `F3e2a - Replay Claim evidence harvest`
Owner: `DeepSeek Pro HIGH`
Model: `DeepSeek Pro HIGH`
Status: `IN_PROGRESS`
Task colour: `Amber`
Route: `OpenCode implements F3e2a Replay Claim evidence harvest; do not describe all F3e or Replay as complete`
Worker note: `docs/worker-notes/2026-08-08-f3e2a-replay-claim-evidence.md`
Base branch: `main`
Base commit: `dfae673407ecef38a9dcf8376b06ddbad4a97abc`
Implementation branch: `foundation/f3e2a-replay-claim-evidence`
Parent branch: `main`
Parent tip: `dfae673407ecef38a9dcf8376b06ddbad4a97abc`
OCaml switch path: `N/A`
Rust toolchain: read exact channel from `rust-toolchain.toml`; use plain Cargo (resolved by root pin); `--locked` mandatory
Toolchain preflight: `pwsh -NoProfile -File scripts/check-dev-tools.ps1`

## Objective

Audit Replay Claim / identity persistence only. Do not audit Replay Generations.

Answer: what properties of the durable Replay Claim are already directly proved, what remains genuinely unverified, and are there any demonstrated defects?

This is an evidence-harvest task, not a redesign.

## F3e2a scope

Replay Claim only — `LogicalExecutionKey`, `ExecutionId`, `ExecutionBinding`, `Claim` canonical serialization/deserialization (replay.rs), and the Claim read/publication path in `ReplayLedger` (replay_windows.rs). Replay Generations explicitly excluded.

## Relevant components

- `tethers-0.1/host-rust/src/replay.rs` — LogicalExecutionKey, ExecutionId, ExecutionBinding, Claim, builder/tester, canonical JSON
- `tethers-0.1/host-rust/src/replay_windows.rs` — ReplayLedger, ValidatedHostRoot, claim publication, admit_or_recover, inline tests (ledger_01–ledger_30, F3b-3 primitives)
- `docs/foundation-pass/PERSISTENCE_INVENTORY.md` — F3b Replay rows, F3e1 Trail evidence

## Evidence dimensions checked

1. Canonical logical-key identity
2. Fresh immutable Claim creation
3. Execution identity creation
4. Ordinary close/reopen recovery of same Claim identity
5. Existing Claim behaviour (already-published, recovered)
6. Conflicting binding behaviour
7. Malformed/noncanonical Claim handling
8. Claim digest corruption handling
9. Filename/content identity agreement
10. Collision/non-replacement at Claim boundary
11. Unexpected temporary/debris handling
12. Unsafe-path protection at Claim boundary
13. Exact bytes/readback

## Relevant background and existing behaviour

F3b characterized Windows flush/sync primitives for Replay (F3b-3): CreateFileW(CREATE_NEW | FILE_FLAG_WRITE_THROUGH), FlushFileBuffers before and after rename, ReplaceIfExists:false, CREATE_NEW exclusion, and close/reopen byte verification. F3b did not exercise the Claim-level store semantics. F3d explicitly excluded Trail and Replay. F3e1 audited Trail only.

## Required behaviour

1. F3e2a-1 — Harvest existing Replay Claim tests and map every property to PROVEN/DISPROVEN/UNVERIFIED with exact test citations and hard assertions.
2. F3e2a-2 — Identify genuine gaps where no existing hard assertion proves the exact Claim statement.
3. F3e2a-3 — Add ≤3 characterization tests to close identified gaps.
4. F3e2a-4 — Record exact remaining UNVERIFIED properties. Do not upgrade F3b claims.
5. F3e2a-5 — No production code change unless an exact characterization test demonstrates a production defect inside the Claim slice.

## Frozen decisions and invariants

- Accepted F3e1 base: `dfae673407ecef38a9dcf8376b06ddbad4a97abc`
- F3b UNVERIFIED platform properties preserved
- Trail evidence from F3e1 preserved
- Replay Generations NOT in scope for this task
- No production code redesign
- No more than 3 characterization tests

## F3e2a findings

### Existing proof (13 dimensions)

| # | Property | Status | Exact test | Exact assertion |
|---|---|---|---|---|
| 1 | Canonical logical-key identity | PROVEN | `sibling_actions_are_distinct` (replay.rs:589), `different_evaluations_are_distinct` (replay.rs:600) | `assert_ne!(key1.as_digest(), key2.as_digest())` |
| 2 | Fresh immutable Claim creation | PROVEN | `claim_round_trip_is_exact_canonical_and_redacted` (replay.rs:613) | `assert_eq!(recovered, claim)` and no `raw_argument` in output bytes |
| 3 | Execution identity creation | PROVEN | `ledger_05_fresh_claim_creates_one_host_execution_identity` (replay_windows.rs:2145) | `assert!(admission.is_fresh()); assert!(ExecutionId::parse(admission.execution_id().to_owned()).is_ok())` |
| 4 | Close/reopen recovery of same Claim identity | PROVEN | `ledger_06_restart_recovers_same_execution_identity` (replay_windows.rs:2162) | `assert!(!recovered.is_fresh()); assert_eq!(recovered.execution_id(), first)` |
| 5 | Existing Claim behaviour (collision) | PROVEN | `ledger_08_exact_claim_collision_recovers_only_valid_winner` (replay_windows.rs:2206) | `assert!(!recovered.is_fresh()); assert_eq!(recovered.execution_id(), winner)` |
| 6 | Conflicting binding behaviour | PROVEN | `ledger_09_binding_mismatch_fails_closed` (replay_windows.rs:2226) | `assert!(matches!(result, Err(ReplayError::BindingMismatch)))` |
| 7 | Malformed/noncanonical Claim handling | PROVEN | `non_canonical_or_unknown_claim_is_rejected` (replay.rs:635) | Both spaced and unknown-field JSON: `assert!(Claim::from_canonical_bytes(...).is_err())` |
| 8 | Claim digest corruption handling | PROVEN | `ledger_10_malformed_or_digest_invalid_claim_fails_closed` (replay_windows.rs:2246) | Forged `claim_digest` → `assert!(matches!(ReplayLedger::open(&root), Err(ReplayError::PersistenceUnavailable)))` |
| 9 | Filename/content identity agreement | PROVEN | `f3e2a_claim_filename_content_disagreement_fails_closed` (replay_windows.rs) NEW | Claim file renamed to different-logical-key hex → `assert!(matches!(ReplayLedger::open(&root), Err(ReplayError::PersistenceUnavailable)))` |
| 10 | Collision/non-replacement at Claim boundary | PROVEN | `native_publication_survives_reopen_and_never_replaces` (replay_windows.rs:1868) | Second publish → `Err(PersistenceUnavailable)`, original bytes preserved, `.tmp` debris retained |
| 11 | Unexpected temporary/debris handling | PROVEN | `ledger_29_unexpected_ledger_entry_fails_closed` (replay_windows.rs:2722) | `.tmp` debris in claims dir → `assert!(matches!(ReplayLedger::open(&root), Err(ReplayError::PersistenceUnavailable)))` |
| 12 | Unsafe-path protection at Claim boundary | PROVEN | `relative_root_is_rejected_before_win32` (replay_windows.rs:1777), `unc_roots_are_rejected_before_win32` (replay_windows.rs:1780), `traversal_ads_and_separator_final_filenames_are_rejected` (replay_windows.rs:1771), `validated_child_retains_complete_independent_handle_chain` (replay_windows.rs:1833) | Relative/UNC/reparse/devices rejected; handle chain prevents TOCTOU substitution |
| 13 | Exact bytes/readback | PROVEN | `claim_round_trip_is_exact_canonical_and_redacted` (replay.rs:613), `ledger_30_restart_never_generates_new_uuid_for_existing_tuple` (replay_windows.rs:2742) | `assert_eq!(recovered, claim)`; `assert_eq!(claim_bytes, claim_before)` after 2 restarts |

One characterization test added: `f3e2a_claim_filename_content_disagreement_fails_closed` — publishes a valid claim, renames the file to a different logical-key hex digest, then reopens; hard-asserts `PersistenceUnavailable`. Filename/content identity agreement was UNVERIFIED; now PROVEN.

### Remaining UNVERIFIED

- Power-loss durability: UNVERIFIED (F3b) — never upgrade
- Directory-entry durability: UNVERIFIED (F3b) — never upgrade
- Atomic visibility during rename: UNVERIFIED (F3b) — never upgrade
- Parent-directory flush in production: DISPROVEN (F3b)

No defect found. Replay Generations untouched.

## Forbidden changes

- Touch Replay Generations (0/1/2)
- Redesign Replay
- Alter public contracts
- Add more than 3 characterization tests
- Upgrade F3b UNVERIFIED claims
- Call a property PROVEN based on implementation inspection alone

## Stop conditions

STOP if:
- A required property cannot be characterized
- A repair would require redesign outside F3e2a
- A required check fails
- Two materially similar attempts fail

## Expected pre-existing changes

None

## Acceptance criteria

1. Replay Claim evidence map across 13 dimensions.
2. Exact remaining UNVERIFIED properties recorded.
3. PERSISTENCE_INVENTORY.md updated with F3e2a Replay Claim evidence.
4. F3e2a worker note records exact evidence and findings.
5. No production code changed. Replay Generations untouched.

## Required verification

```powershell
cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check
cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --lib -- replay
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
git diff --check
```
