# Current Implementation Task

Control contract: `1`
Task: `F3e2b - Replay Generations & Recovery evidence harvest`
Owner: `DeepSeek Pro HIGH`
Model: `DeepSeek Pro HIGH`
Status: `COMPLETE`
Task colour: `Amber`
Route: `OpenCode implements F3e2b Replay Generations & Recovery evidence harvest; do not describe all F3e or Replay as complete`
Worker note: `docs/worker-notes/2026-08-08-f3e2b-replay-generations-evidence.md`
Base branch: `main`
Base commit: `477e2b901c0dfec55f4df6f9dca79a66e9294e0a`
Implementation branch: `foundation/f3e2b-replay-generations-evidence`
Implementation checkpoint: `9f4df4676d0a91d3dddd632de40bf6d1f19bfb9d`
Parent branch: `foundation/f3e2a-replay-claim-evidence`
Parent tip: `477e2b901c0dfec55f4df6f9dca79a66e9294e0a`
OCaml switch path: `N/A`
Rust toolchain: read exact channel from `rust-toolchain.toml`; use plain Cargo (resolved by root pin); `--locked` mandatory
Toolchain preflight: `pwsh -NoProfile -File scripts/check-dev-tools.ps1`

## Objective

Audit Replay Generations 0/1/2 and their durable restart reconstruction only. Do not re-audit Replay Claim identity (accepted in F3e2a).

Answer: what properties of the durable Replay Generation chain and restart reconstruction are already directly proved, what remains genuinely unverified, and are there any demonstrated defects?

This is an evidence harvest, not a redesign.

## F3e2b scope

Replay Generations only — `Generation`, `validate_chain`, generation publication, generation readback, chain validation, restart reconstruction, and recovered-admission mutation blocking in `replay.rs` and `replay_windows.rs`. Replay Claim identity explicitly excluded (accepted F3e2a).

## Relevant components

- `tethers-0.1/host-rust/src/replay.rs` — Generation struct, constructors (intent/armed/terminal), canonical serialization/deserialization, validate_chain, inline tests
- `tethers-0.1/host-rust/src/replay_windows.rs` — publish_generation, read_generation_directory, scan_chains, reconstruct, admit_or_recover, ledger_12–ledger_all_bounded tests
- `docs/foundation-pass/PERSISTENCE_INVENTORY.md` — F3b Replay rows, F3e1 Trail evidence, F3e2a Claim evidence

## Relevant background and existing behaviour

F3b characterized Windows flush/sync primitives for Replay (F3b-3): CreateFileW(CREATE_NEW | FILE_FLAG_WRITE_THROUGH), FlushFileBuffers before and after rename, ReplaceIfExists:false, CREATE_NEW exclusion, and close/reopen byte verification. F3e2a audited Replay Claim identity only — all 13 Claim dimensions PROVEN. Replay Generations were explicitly deferred. The Generation model supports numbers 0 (IntentRecorded), 1 (InvocationArmed), and 2 (terminal: Succeeded/Failed/Uncertain). Generations are linked by predecessor-digest chain and validated by `validate_chain`. Publication uses the same `publish_new_canonical_file_with_temporary_stem` primitive as Claims (handle-based rename + post-rename byte verification). Recovery reconstructs ReplayState from persisted Generations via `reconstruct()`.

## Required behaviour

1. F3e2b-1 — Harvest existing Replay Generation tests and map every property to PROVEN/DISPROVEN/UNVERIFIED with exact test citations and hard assertions.
2. F3e2b-2 — Identify genuine gaps where no existing hard assertion proves the exact Generation statement.
3. F3e2b-3 — Add ≤3 characterization tests to close identified gaps (2 added).
4. F3e2b-4 — Record exact remaining UNVERIFIED properties. Do not upgrade F3b claims.
5. F3e2b-5 — No production code change unless an exact characterization test demonstrates a production defect inside the Generation slice.

## Evidence dimensions audited

1. Canonical Generation representation
2. Generation 0 publication
3. Generation 0 → Generation 1 transition
4. Generation 2 terminal states (all 3: Succeeded, Failed, Uncertain)
5. Missing-generation rejection (G1 without G0, G2 without G1)
6. Illegal state-at-generation rejection
7. Predecessor-digest linkage
8. Generation immutability / non-replacement
9. Generation upper bound (model, parser, persistence)
10. Restart reconstruction (6 state variants)
11. Recovered admissions cannot advance history (with filesystem unchanged proof)
12. Malformed/corrupt Generation-chain fail-closed behaviour
13. Filename/content agreement for Generations
14. Exact bytes / ordinary close-reopen

## Frozen decisions and invariants

- Accepted F3e2a-R1 base: `477e2b901c0dfec55f4df6f9dca79a66e9294e0a`
- F3b UNVERIFIED platform properties preserved
- Replay Claim evidence from F3e2a preserved
- No production code redesign
- No more than 3 characterization tests (0 added — see findings)

## F3e2b findings

### Evidence table (14 dimensions)

| # | Property | Status | Exact test | Exact hard assertion |
|---|---|---|---|---|
| 1 | Canonical Generation representation | PROVEN | `generation_three_is_not_representable_or_parseable` (replay.rs:657) negative; reconstruction tests (ledger 21–26) positive | G3 bytes → `assert!(Generation::from_canonical_bytes(&bytes).is_err())`; reconstruction: publish G0→G1→G2, reopen, state matches expected — proves publish→reopen→parse round-trip |
| 2 | Generation 0 publication | PROVEN | `ledger_12_valid_generation_zero_publication` (replay_windows.rs:2287) | `assert_eq!(admission.state(), ReplayState::IntentRecorded)`; `assert_eq!(directory_names(&path), vec!["g0000000000000000.json"])` — exact durable filename |
| 3 | G0 → G1 transition | PROVEN | `ledger_13_valid_generation_zero_to_one_transition` (replay_windows.rs:2302) | `admission.publish_intent().unwrap(); admission.publish_armed().unwrap(); assert_eq!(admission.state(), ReplayState::InvocationArmed)` |
| 4 | G2 terminal states (all 3) | PROVEN | `ledger_14_each_valid_generation_two_terminal_state` (replay_windows.rs:2316) | Loop over `[Succeeded, Failed, Uncertain]`: publish terminal → `assert_eq!(admission.state(), state)` for each |
| 5a | G1 without G0 rejected | PROVEN | `ledger_15_generation_one_without_zero_is_rejected` (replay_windows.rs:2340) | Manually write G1 file only → `assert!(ReplayLedger::open(&root).is_err())` |
| 5b | G2 without G1 rejected | PROVEN | `ledger_16_generation_two_without_one_is_rejected` (replay_windows.rs:2363) | Write G0 + G2, skip G1 → `assert!(ReplayLedger::open(&root).is_err())` |
| 6 | Illegal state-at-generation | PROVEN | `ledger_17_illegal_state_transition_is_rejected` (replay_windows.rs:2398); `chain_cannot_skip_armed` (replay.rs:644) | G0 tampered to "succeeded" → open err; G0→G2 direct (skip armed) → `assert!(validate_chain(&claim, &[invalid]).is_err())` |
| 7 | Predecessor-digest linkage | PROVEN | `ledger_18_predecessor_mismatch_is_rejected` (replay_windows.rs:2428) | G1 with tampered predecessor_digest → `assert!(ReplayLedger::open(&root).is_err())` |
| 8 | Generation immutability / non-replacement | PROVEN | `ledger_19_generation_collision_never_replaces_bytes` (replay_windows.rs:2463) | Pre-existing conflicting G1 file → `publish_armed().is_err()`; `assert_eq!(std::fs::read(collision).unwrap(), b"different-immutable-bytes")` — original bytes unchanged |
| 9a | G3 model/parser rejection | PROVEN | `generation_three_is_not_representable_or_parseable` (replay.rs:657) | `assert!(Generation::from_canonical_bytes(&bytes).is_err())` |
| 9b | G3 filename rejection | PROVEN | `generation_filename(3)` call (replay_windows.rs:1213) | `assert!(generation_filename(3).is_err())` |
| 9c | G3 persistence rejection | PROVEN | `ledger_20_generation_three_is_rejected` (replay_windows.rs:2493) | G0+G1+G2 terminal published → second terminal → `assert!(admission.publish_terminal(...).is_err())` |
| 10a | Claim-only reconstruction | PROVEN | `ledger_21_claim_only_reconstructs_blocked_incomplete` (replay_windows.rs:2516) | Reopen → `assert_eq!(recovered.state(), ReplayState::ClaimedNoState)` |
| 10b | G0 reconstruction | PROVEN | `ledger_22_generation_zero_reconstructs_blocked_incomplete` (replay_windows.rs:2536) | `assert_eq!(recovered.state(), ReplayState::IntentRecorded)` |
| 10c | G1 reconstruction | PROVEN | `ledger_23_armed_reconstructs_blocked_possible_invocation` (replay_windows.rs:2555) | `assert_eq!(...state(), ReplayState::InvocationArmed)` |
| 10d | G2 Succeeded reconstruction | PROVEN | `ledger_24_succeeded_reconstructs_permanently_blocked` (replay_windows.rs:2577) | `assert_eq!(recovered.state(), ReplayState::Succeeded)` |
| 10e | G2 Failed reconstruction | PROVEN | `ledger_25_failed_reconstructs_permanently_blocked` (replay_windows.rs:2582) | `assert_eq!(recovered.state(), ReplayState::Failed)` |
| 10f | G2 Uncertain reconstruction | PROVEN | `ledger_26_uncertain_reconstructs_manual_resolution` (replay_windows.rs:2587) | `assert_eq!(recovered.state(), ReplayState::Uncertain)` |
| 11a | Recovered Claim/G0/G1 cannot advance | PROVEN | `recovered_claim_g0_and_g1_admissions_cannot_advance_or_mutate` (replay_windows.rs:2616) | Loop claim/G0/G1: mutation attempt → `assert!(matches!(result, Err(ReplayError::PersistenceUnavailable)))`; `assert_eq!(tree_snapshot(&root), before)` — filesystem unchanged |
| 11b | Recovered terminal cannot mutate | PROVEN | `recovered_terminal_admission_cannot_publish_or_mutate` (replay_windows.rs:2653) | `assert!(recovered.publish_intent().is_err()); assert!(recovered.publish_armed().is_err()); assert!(recovered.publish_terminal(...).is_err())`; `assert_eq!(tree_snapshot(&root), before)` |
| 12a | Malformed record fails closed | PROVEN | `ledger_28_malformed_chain_fails_closed` (replay_windows.rs:2704) | `g0000000000000000.json` = `b"{"` → `assert!(ReplayLedger::open(&root).is_err())` |
| 12b | Orphan chain fails closed | PROVEN | `ledger_27_orphan_chain_fails_whole_ledger_closed` (replay_windows.rs:2684) | Empty execution dir (no claim) → `assert!(matches!(ReplayLedger::open(&root), Err(ReplayError::PersistenceUnavailable)))` |
| 12c | Digest/state corruption (via items 6–8 above) | PROVEN | ledger_17, ledger_18 | Tampered state or predecessor → open err |
| 13 | Filename/content agreement for Generations | PROVEN | `f3e2b_generation_filename_content_disagreement_fails_closed` (replay_windows.rs) NEW | Publish G0+G1, swap filenames (G0↔G1), reopen → `assert!(ReplayLedger::open(&root).is_err())` |
| 14 | Exact bytes / close-reopen | PROVEN | `f3e2b_generation_exact_bytes_survive_close_and_reopen` (replay_windows.rs) NEW; `ledger_30_restart_never_generates_new_uuid_for_existing_tuple` (replay_windows.rs:2741) claim bytes; `ledger_populated_valid_subtrees_reopen_without_reprovisioning` (replay_windows.rs:2769) full chain | G0/G1/G2 bytes read before and after close/reopen, `assert_eq!(g0_after, g0_before)` etc. for all 3; `assert_eq!(claim_bytes, claim_before)` after 2 restarts; full chain reopens with correct state and tree unchanged |

### Remaining UNVERIFIED

- Power-loss durability: UNVERIFIED (F3b) — never upgrade
- Directory-entry durability: UNVERIFIED (F3b) — never upgrade
- Atomic visibility during rename: UNVERIFIED (F3b) — never upgrade
- Parent-directory flush in production: DISPROVEN (F3b)

No defect found. Two characterization tests added — `f3e2b_generation_filename_content_disagreement_fails_closed` (closes filename/content agreement gap) and `f3e2b_generation_exact_bytes_survive_close_and_reopen` (closes exact bytes close/reopen gap). All 14 dimensions PROVEN.

## Forbidden changes

- No Replay redesign
- No Claim re-audit (accepted F3e2a)
- No application dispatch changes
- No public contract changes
- No more than 3 characterization tests (2 added)
- No upgrading F3b atomic/power-loss claims
- No calling implementation inspection as proof
- No calling all Replay complete

## Stop conditions

STOP if:
- A required property cannot be characterized
- A repair would require redesign outside F3e2b
- A required check fails
- Two materially similar attempts fail

## Expected pre-existing changes

None

## Acceptance criteria

1. Replay Generation evidence map across 14 dimensions — all PROVEN.
2. Exact remaining UNVERIFIED properties recorded (same F3b set).
3. PERSISTENCE_INVENTORY.md updated with F3e2b Replay Generations evidence.
4. F3e2b worker note records exact evidence and findings.
5. No production code changed.

## Required verification

```powershell
cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check
cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --locked --lib -- replay
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
git diff --check
```
