# Worker Note

Task: `F3c - Installation intent and publication contract`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `71f79f7c80b2a09921ee59ac4b1acfa3926bf834`

Implementation checkpoint: `fdd216139702e38f48874f2e975db98f2abf924a`

## Requested outcome

Audit the specialised installation intent/publication/recovery contract and,
only where directly justified, align it. Add characterization tests proving
every F3c property with hard assertions. The audit found the existing contract
correct across all six F3c property clusters. No production repairs required.

## Changes made

1. **F3c task packet** (`docs/CURRENT_CLINE_TASK.md`): written per the
   user's specification, covering all six F3c properties with required
   behaviour, acceptance criteria, stop conditions, and verification matrix.

2. **F3c characterization tests** (`src/f3c_installation_intent_publication_evidence.rs`):
   43 tests covering F3c-1 through F3c-6. Every PROVEN claim has a hard
   assertion that fails if the exact statement is false.

3. **Module declaration** (`src/lib.rs`): added `#[cfg(test)] mod
   f3c_installation_intent_publication_evidence;`.

4. **PERSISTENCE_INVENTORY.md**: added F3c evidence section with detailed
   findings tables for all six property clusters.

5. **No production code changed**: zero changes to `installation_publication_intent.rs`,
   `installed.rs`, `installation_execution.rs`, `installation_plan.rs`,
   `installation_publication_mutation.rs`, `installation_publication_preparation.rs`,
   `installation_recovery.rs`, `installation_recovery_execution.rs`, or
   `installation_recovery_plan.rs`.

## F3c findings

### F3c-1 — Publication intent identity: PROVEN

All six sub-properties verified with hard assertions:
- One canonical identity: deterministic digest from `covered_bytes()`
- Stored bytes bind exact operation: tampering any field invalidates digest
- Conflicting intent refused: `create()` returns conflict; original bytes preserved
- Duplicate retry deterministic: second `create()` with same intent returns conflict
- Singleton `current.json` enforced: zero entries → `None`, wrong name → invalid
- Malformed state fails closed: invalid JSON, unknown fields, `.tmp` remnants

### F3c-2 — Exact-match removal: PROVEN

All required negative properties verified:
- Wrong digest cannot remove → `installation_intent_conflict`
- Wrong identity cannot remove → `installation_intent_conflict`
- Stale intent cannot remove → `installation_intent_conflict`
- Invalid expected does not mutate store → `installation_intent_invalid`
- Missing distinguished from mismatched → `Ok(false)` vs `Err(conflict)`

### F3c-3 — Publication ordering: PROVEN

The 7-step sequence in `execute_prepared_disabled_installation_publication`:
1. Intent created → 2. Staging built → 3. Staging renamed → 4. Record published
→ 5. Intent removed → 6-7. Post-condition verified

Power-loss durability: UNVERIFIED (F3b). Concurrent rename atomicity: UNVERIFIED (F3b).

### F3c-4 — Recovery state matrix: PROVEN

8 combinations of (staging, destination, record) exhaustively tested:
- 4 valid states → correct dispositions
- 4 invalid states → `installation_recovery_conflict`
- Invalid intent → `installation_intent_invalid` (before classification)
- Classification is deterministic and idempotent

### F3c-5 — Recovery must not destroy evidence: PROVEN

All negative properties verified:
- Mismatched destination never deleted
- Mismatched record never overwritten
- Unrelated staging never removed
- Wrong intent never cleared
- Corruption evidence preserved on disk
- No silent normalization of ambiguous states

### F3c-6 — Canonical bytes / digest truth: PROVEN

All properties verified:
- Digest over canonical representation (covered_bytes + sha256)
- Read-back identity re-validated by `load()` → `validate()`
- Filename identity disagreement fails closed
- Recovery decisions use validated persisted state
- Written bytes are exact canonical intent
- All content fields digest-covered

## Evidence

### Focused F3c tests

| Command | Result |
|---|---|
| `cargo test --lib -- f3c` | PASS (43/43) |

### Full test suite

`cargo test --all-targets --all-features --locked`: 1316 tests PASS, 0 FAIL.

### Verification matrix

| Command | Result |
|---|---|
| `git fetch origin --prune` | PASS |
| `git rev-parse origin/main` | PASS (`71f79f7`) |
| `cargo fmt --all -- --check` | PASS |
| `cargo check --all-targets --all-features --locked` | PASS |
| `cargo test --all-targets --all-features --locked` | PASS (1316/1316) |
| `cargo clippy --all-targets --all-features --locked -- -W clippy::all` | PASS |
| `just verify` | PASS |
| `just verify-agent` | PASS (1574/1574) |
| `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` | PASS |
| `git diff --exit-code origin/main...HEAD -- docs/foundation-pass/fixtures` | PASS |
| `git diff --check origin/main...HEAD` | PASS |

## Decisions and assumptions

- F3b power-loss durability and concurrent rename atomicity labels preserved (UNVERIFIED).
- No parent-directory flushing added; production does not perform it.
- Existing 19 intent tests and 16 recovery tests preserved unchanged.
- No production code modified. Zero repair required.
- All PROVEN labels correspond to hard assertions in characterization tests.

## Unresolved

- Power-loss durability: UNVERIFIED for all states in the publication pipeline.
- Concurrent atomic visibility during rename: UNVERIFIED.
- Parent-directory flush: feasible per F3b but production does not perform it.

## Smallest next action

Push the branch and route to Lucy for independent Amber review.
Do not begin F3d.
