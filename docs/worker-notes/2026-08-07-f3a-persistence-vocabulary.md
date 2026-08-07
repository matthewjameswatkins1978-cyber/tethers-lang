# Worker Note

Task: `F3a - Persistence inventory and vocabulary`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `83eec98a0f33f964623f4cbbf4548a76bbdf5255`

Preparation checkpoint: `3e4845e2908b3e69c5cdc30bf59f28642149642e`

## Requested outcome

Evidence-backed documentation-only inventory of every filesystem-backed
persistence store in the accepted F2 mainline. Every store classified using the
frozen four-class vocabulary. Every durability statement rooted in accepted-main
source or direct tests; unsupported claims marked `UNVERIFIED (F3b)`.

## Changes made

### PERSISTENCE_INVENTORY.md — complete F3a correction pass

**Header and baseline:**
- Baseline updated from F1 (`24428139`) to accepted F2 (`83eec98a`)
- Added evidence-claim preamble

**Write-primitive clarifications:**
- Candidate Registry: `write_new()` writes `.{id}.tmp` with `sync_all()` on tmp
  BEFORE rename (not after). Source: `candidate.rs:497-504`
- Installation Publication Intent: corrected from "overwritten" to
  remove-then-recreate pattern (`remove_if_matches()` + `create()`)
- Installed Plug Registry: staging directory uses `.staging-{id}` prefix, not
  `.{id}.tmp` suffix. Source: `installed.rs:761-796`
- Installation Recovery Staging: same `.staging-{id}` convention; listed
  separated recovery functions alongside `install_disabled_with_authority()`

**Test citation corrections (source-verified negative findings):**
- Launch Profile Evidence: no inline `#[cfg(test)]` module in `launch_profile.rs`.
  Replaced "Inline `mod tests`" with actual integration test files
- Conformance Evidence: no inline `#[cfg(test)]` module in `conformance.rs`.
  Replaced with actual integration test files
- Installation Approval: no inline tests; replaced with actual exercise files
- Installed Plug Registry: replaced method name `audit_installation_recovery_destinations`
  with concrete test file names (`installation_recovery_destination_tests.rs`,
  `installation_recovery_audit_tests.rs`, etc.)
- Local Anchor Admission Store: added two more verified tests
- Trail: added second file-trail test name

**Line-number references added:**
- Every store row now includes one or more `file:line` references for the write
  primitive, recovery reader, and test locations

**In-Memory Appendix:**
- Renamed from "Non-Durable Appendix" to "In-Memory Appendix"
- Clarified process-local state is not durable persistence

**F3b Route Map:**
- Expanded from 7 to 10 stores (added Candidate Registry, Local Anchor Admission
  Store, and Installation Execution Lock)
- Added lock-file row noting OS handle release, not disk durability

**New section: Changes Made in F3a:**
- Records the specific evidence base of each correction

### DEBT_LEDGER.md

No changes required. The A1 entry (directory durability not explicitly tested)
correctly reflects the current `UNVERIFIED (F3b)` state. No ledger statement
was found to be inaccurate as source evidence.

### CURRENT_GOAL.md

Updated to reflect F3a `IN_PROGRESS` state and distinguish from F3b.

### CURRENT_CLINE_TASK.md

Updated metadata: Owner `OpenCode`, Model `DeepSeek Pro`, Status `COMPLETE`,
Route updated, Preparation checkpoint set to concrete SHA.

## Classification summary

| Store | Class | Dir-Entry Durability |
|---|---|---|
| Candidate Registry | Immutable Atomic Record | UNVERIFIED (F3b) |
| Publisher Trust Store | Immutable Atomic Record | UNVERIFIED (F3b) |
| Developer Approval Store | Immutable Atomic Record | UNVERIFIED (F3b) |
| Launch Profile Evidence | Immutable Atomic Record | UNVERIFIED (F3b) |
| Conformance Evidence | Immutable Atomic Record | UNVERIFIED (F3b) |
| Installation Approval | Immutable Atomic Record | UNVERIFIED (F3b) |
| Installed Plug Registry | Immutable Atomic Record | UNVERIFIED (F3b) |
| Enablement Records | Immutable Atomic Record | UNVERIFIED (F3b) |
| Replay Claim (identity) | Immutable Atomic Record | UNVERIFIED (F3b) |
| Installation Publication Intent | Replaceable Current-State Record | UNVERIFIED (F3b) |
| Trail (FileTrail) | Append-Only Causal Log | UNVERIFIED (F3b) |
| Replay Generations (0-2) | Multi-Step Intent/Recovery Journal | UNVERIFIED (F3b) |
| Installation Recovery Staging | Multi-Step Intent/Recovery Journal | UNVERIFIED (F3b) |
| Installation Execution Lock | Multi-Step Intent/Recovery Journal | N/A |
| Local Anchor Admission Store | Multi-Step Intent/Recovery Journal | UNVERIFIED (F3b) |

Total: 15 classified stores across 4 classes. 13 have directory-entry durability
routed to F3b. 1 (execution lock) has no data durability requirement. 1 (Trail)
has no atomic rename and no path safety.

## Residual F3b questions

1. Does `sync_all()` on a file followed by `fs::rename` on NTFS guarantee
   directory metadata durability on the primary target? (All StoreRoot-backed
   stores + Candidate Registry)
2. Does post-rename `FlushFileBuffers` on the renamed file handle flush the
   parent directory entry? (Replay Ledger)
3. Does `FILE_FLAG_WRITE_THROUGH` on temporary-file `CreateFileW` guarantee file
   data durability on the exact NTFS volume class used in production?
   (Replay Ledger)
4. Is line-level JSONL append with `sync_data()` after each line adequate for
   Trail recovery after a crash (partial last line)?
5. Should the Local Anchor Admission Store apply `verify_chain()` on its root
   directory before `create_dir_all()`?
6. What happens to the file data after `fs::rename` when power is lost before
   the rename reaches stable directory metadata?

## Verification matrix

All commands run serially after final documentation edit on the F3a branch at
`foundation/f3a-persistence-vocabulary`.

| Command | Result |
|---|---|
| `git fetch origin --prune` | PASS |
| `git rev-parse origin/main` | PASS (`83eec98`) |
| `git rev-parse HEAD` | PASS |
| `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` | PASS |
| `git diff --exit-code origin/main...HEAD -- docs/foundation-pass/fixtures` | PASS (no fixture changes) |
| `git diff --check origin/main...HEAD` | PASS (no whitespace issues) |
| `git diff --name-only origin/main...HEAD` | PASS (docs only) |
| `git diff --name-only origin/main...HEAD -- ':!docs/**'` | PASS (empty — no production changes) |
| `git status --short --branch` | PASS (clean) |
| `cargo fmt --all -- --check` | NOT RUN (no production changes) |
| `cargo check --all-targets --all-features --locked` | NOT RUN (no production changes) |
| `cargo test --all-targets --all-features --locked` | NOT RUN (no production changes) |
| `cargo clippy --all-targets --all-features --locked` | NOT RUN (no production changes) |
| `just verify` | NOT RUN (no production changes) |
| `just verify-agent` | NOT RUN (no production changes) |

All Rust/OCaml/integration test suites are NOT RUN because F3a changes
documentation only. This is authorised by the packet's Required Verification
section.

## Forbidden changes

None performed. No Rust, OCaml, test, fixture, dependency, Cargo.lock, protocol,
or CLI changes. No persistence repairs, write-primitive changes, directory
flushes, or migration. F1 fixtures remain byte-identical.
