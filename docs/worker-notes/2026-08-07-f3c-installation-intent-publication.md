# Worker Note

Task: `F3c - Installation intent and publication contract`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE` (correction pass)

Base commit: `71f79f7c80b2a09921ee59ac4b1acfa3926bf834`

Implementation checkpoint: `(final — read from Git)`

## Correction pass (Lucy review)

Three evidence-strength corrections applied. No production code changed.

### Correction 1: F3c-3 — Publication ordering evidence

**Problem:** The original F3c tests only proved intent lifecycle (create → read → remove).
The claim that the full 7-step production sequence was proven within the F3c module was inaccurate.

**Fix:**
- Renamed `f3c3_publication_sequence_is_deterministic_and_mapped` → `f3c3_intent_lifecycle_is_deterministic` (accurately describes what it proves)
- Added `f3c3_intent_creation_is_the_first_publication_step` — hard-asserts no staging, no destination, no side effects after intent creation only
- Documentation now cites the existing mutation/execution tests that hard-assert the complete publication sequence:
  - `j24k3e2_valid_prepared_publication_completes_exactly_once` — calls `execute_prepared_disabled_installation_publication`; asserts final state (destination is_dir, record is_file, intent removed, staging gone)
  - `j24k3f_test_only_post_intent_failure_is_recoverable_and_publishes_once` — post-intent hook boundary: intent loaded, staging/destination/records NOT created

### Correction 2: F3c-5 — Recovery preserves evidence

**Problem:** Several F3c tests only proved classifier disposition (enum return value) but claimed filesystem non-mutation properties (e.g., "destination never deleted", "record never overwritten"). A classifier result alone is not direct proof of filesystem non-mutation.

**Fix:**
- Renamed tests to explicitly label classification scope:
  - `f3c5_mismatched_destination_is_never_deleted_by_recovery` → `f3c5_classifier_mismatched_destination_returns_revalidate_not_delete`
  - `f3c5_mismatched_record_never_overwritten` → `f3c5_classifier_mismatched_record_returns_conflict_not_overwrite`
  - `f3c5_unrelated_staging_is_not_removed` → `f3c5_classifier_staging_plus_destination_returns_conflict`
  - `f3c5_recovery_never_silently_normalises_ambiguous_state_into_success` → `f3c5_all_four_classified_invalid_states_return_error`
- Each renamed test cites the existing j24k3d2_* execution test that provides hard filesystem snapshot evidence:
  - `j24k3d2_recovery_never_adopts_or_deletes_final_destination` — tree_snapshot before/after byte-identical
  - `j24k3d2_completed_publication_removes_only_intent` — destination + record byte-identical
  - `j24k3d2_staging_recovery_removes_exact_staging_then_intent` — sibling staging survives
  - `j24k3d2_unrelated_stores_remain_unchanged` — 6 unrelated stores byte-identical
  - `j24k3d2_idle_plan_performs_no_mutation` — tree_snapshot before/after identical
- Two F3c tests retained as direct executor-level proof (they already assert filesystem bytes):
  - `f3c5_wrong_intent_is_never_cleared` — byte snapshot before == after on `remove_if_matches` conflict
  - `f3c5_corruption_tamper_evidence_preserved` — tampered file remains on disk with tampered content
- Broad executor proof across all 4 invalid states: UNVERIFIED (no single test exercises the executor for every combination)

### Correction 3: F3c-6 — Canonical bytes / digest truth

**Problem:** `f3c6_digest_computed_over_canonical_representation` only proved "same input → same digest" and self-consistency via `validate()`. It did not independently construct the exact covered representation and compare against the intent's digest.

**Fix:**
- Strengthened the test to independently construct the covered representation:
  ```rust
  let mut covered = intent.clone();
  covered.intent_digest.clear();
  let canonical_covered_bytes = canonical(&covered).unwrap();
  let expected_digest = sha256(&canonical_covered_bytes);
  assert_eq!(intent.intent_digest, expected_digest);
  ```
- Retained the existing deterministic/cross-instance assertions.

## Evidence summary (corrected)

### F3c-1 — Publication intent identity: PROVEN (6 tests)
All six sub-properties hard-asserted in F3c characterization tests.

### F3c-2 — Exact-match removal: PROVEN (7 tests)
All negative properties hard-asserted in F3c characterization tests (byte snapshots).

### F3c-3 — Publication ordering: PROVEN
- Intent lifecycle: `f3c3_intent_lifecycle_is_deterministic`, `f3c3_intent_only_persists_and_does_not_imply_publication`
- Intent-first step: `f3c3_intent_creation_is_the_first_publication_step`
- Full production sequence: `j24k3e2_valid_prepared_publication_completes_exactly_once`
- Post-intent boundary: `j24k3f_test_only_post_intent_failure_is_recoverable_and_publishes_once`

### F3c-4 — Recovery state matrix: PROVEN (16 tests)
All valid/invalid states; classification is deterministic, idempotent, preserves inputs.

### F3c-5 — Recovery must not destroy evidence: PROVEN
- Classification: all 4 invalid states return error (F3c)
- Executor filesystem non-mutation: 5 j24k3d2_* tests (tree_snapshot before/after)
- Intent-store non-mutation: 2 F3c tests (byte snapshots)
- Broad executor proof across all 4 invalid states: UNVERIFIED

### F3c-6 — Canonical bytes / digest truth: PROVEN (7 tests)
Independent construction of covered bytes; read-back identity; filename enforcement; validation gate.

## Unresolved

- Power-loss durability: UNVERIFIED for all 7 publication steps (F3b)
- Concurrent rename atomicity: UNVERIFIED (F3b)
- Parent-directory flush: DISPROVEN (production does not perform it; F3b)
- Broad executor proof across all 4 invalid recovery states: UNVERIFIED

## Focused test results

```
cargo test --lib -- f3c    → 44/44 PASS
cargo test --lib -- j24k3d2 → 20/20 PASS
cargo test --lib -- j24k3e2 → 26/26 PASS
cargo test --lib -- j24k3f  → 10/10 PASS
```

## Full verification matrix

| Command | Result |
|---|---|
| `git fetch origin --prune` | PASS |
| `git rev-parse origin/main` | PASS (`71f79f7`) |
| `cargo fmt --all -- --check` | PASS |
| `cargo check --all-targets --all-features --locked` | PASS |
| `cargo test --all-targets --all-features --locked` | PASS (1317, 0 failures) |
| `cargo clippy --all-targets --all-features --locked -- -W clippy::all` | PASS |
| `just verify` | PASS |
| `just verify-agent` | PASS (1575, 0 failures) |
| `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` | PASS |
| `git diff --exit-code origin/main...HEAD -- docs/foundation-pass/fixtures` | PASS |
| `git diff --check origin/main...HEAD` | PASS |

## Smallest next action

Push and route to Lucy for independent Amber re-review. Do not begin F3d.
