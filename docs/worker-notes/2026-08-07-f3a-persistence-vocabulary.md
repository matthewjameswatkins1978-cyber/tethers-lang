# Worker Note

Task: `F3a - Persistence inventory and vocabulary`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Model: `DeepSeek Pro`

Status: `COMPLETE`

Base commit: `83eec98a0f33f964623f4cbbf4548a76bbdf5255`

Implementation checkpoint: `3e4845e2908b3e69c5cdc30bf59f28642149642e`

## Requested outcome

Evidence-backed documentation-only inventory of every filesystem-backed
persistence store in the accepted F2 mainline. Every store classified using the
frozen four-class vocabulary. Every durability statement rooted in accepted-main
source or direct tests; unsupported claims marked `UNVERIFIED (F3b)`.

## Changes made (r2 correction pass)

### Correction 1: Atomic Visibility column rewritten

Every Atomic Visibility cell now records only the observed accepted-main
primitive followed by "atomic visibility guarantee UNVERIFIED (F3b)".
The presence of `rename`/`SetFileInformationByHandle`/`fs::rename` in
accepted-main source is not by itself evidence of the exact Windows
atomic-visibility or crash guarantee.

Before (example): `Atomic rename (NTFS)` or `Atomic rename`
After (all stores): `write-then-rename; atomic visibility guarantee UNVERIFIED (F3b)`
Replay variants: `handle-based rename + post-rename byte verification; atomic visibility guarantee UNVERIFIED (F3b)`

No store in the inventory claims a proven atomic-visibility guarantee.

### Correction 2: F3b Route Map rewritten as questions/evidence gaps

Before: prescriptive statements about what NTFS "requires", what
"does not guarantee", and what "behaviour varies by volume type".

After: each row records (a) the observed accepted-main primitive and
(b) the outstanding question F3b must answer. F3a does not answer F3b.

Key Differences section similarly revised to avoid prescriptive claims.

### Correction 3: Installation Execution Lock reclassified

`InstallationLockGuard` creates/opens an empty filesystem anchor solely to
hold an exclusive OS handle (`share_mode(0)`). The holder never writes bytes.
Its persistent file contents do not encode intent, recovery state, or
causal history.

Removed from the four-class persistence-store inventory (previously in
Multi-Step Intent/Recovery Journal). Added to a new appendix section:
"Filesystem Coordination Artifacts — Not Persistence Stores".

### Totals recalculated

After correction 3: 14 classified persistence stores (9 immutable atomic
records + 1 replaceable current-state record + 1 append-only causal log +
3 multi-step journals), plus 1 coordination artifact and 6 in-memory stores.

### Changes Made in F3a section updated

Added correction pass (r2) sub-section documenting the three corrections.

## Decisions and assumptions

- The presence of `fs::rename`/`SetFileInformationByHandle`/`sync_all()`/`FlushFileBuffers`
  in accepted-main source is observed behaviour, not proof of the exact Windows
  atomic-visibility or durability guarantee.
- Category errors (Installation Execution Lock as a persistence store) must be
  corrected rather than qualified.
- `m3_store.rs` (StoreRoot) remains classified as shared infrastructure, not an
  independent store.
- `UNVERIFIED (F3b)` is the honest classification, not `BROKEN`.

## Evidence

- All 14 classified persistence stores: source verified against accepted main
  at `83eec98a0f33f964623f4cbbf4548a76bbdf5255`.
- Test citations verified: `rg` searches confirmed all named test functions
  exist in the referenced source files.
- Negative findings: no inline `#[cfg(test)]` module in `launch_profile.rs`,
  `conformance.rs`, or `installed.rs` (for Installation Approval).
- Staging naming validated: `installed.rs:764` uses `.staging-{installed_id}`.
- Lock classification verified: `InstallationLockGuard` holds no durable state,
  writes no bytes, encodes no intent.

## Discoveries

- The Launch Profile Evidence and Conformance Evidence stores (both described
  as having "Inline `mod tests`" in the F1 inventory) contain zero inline tests.
- The Installed Plug Registry test citation in F1 referenced a method name
  (`audit_installation_recovery_destinations`) rather than test functions.
- The Installation Execution Lock creates an empty anchor file solely for
  exclusive handle semantics — a filesystem coordination artifact rather than
  a persistence store.

## Remaining risks

- All 14 persistence stores rely on `fs::rename` or `SetFileInformationByHandle`
  for atomic visibility; the exact Windows guarantee after interruption or power
  loss is unproven (F3b).
- Trail (FileTrail) has no integrity footer, per-line digest, or path safety
  verification.
- Local Anchor Admission Store has no `verify_chain()` on its root directory.
- The Replay Ledger post-rename `FlushFileBuffers` targets the renamed file
  handle, not the parent directory.

## Smallest next action

Run F3b Windows primitive experiments to establish the exact atomic-visibility
and directory-entry durability guarantees for each observed primitive:
`sync_all()` + `fs::rename`, `FlushFileBuffers` + handle rename +
post-rename reopen/re-read, and `FILE_FLAG_WRITE_THROUGH` on the exact NTFS
volume class used in production.

## References

- Accepted main: `83eec98a0f33f964623f4cbbf4548a76bbdf5255`
- Task packet: `docs/CURRENT_CLINE_TASK.md`
- Foundation Pass plan: `docs/architecture/TETHERS_FOUNDATION_PASS.md`
- F1 worker note: `docs/worker-notes/2026-08-06-f1-baseline.md`
- F2 worker note: `docs/worker-notes/2026-08-07-f2-operational-correctness.md`

## Verification matrix

All commands run serially after the final documentation edit.

| Command | Result |
|---|---|
| `git fetch origin --prune` | PASS |
| `git rev-parse origin/main` | PASS (`83eec98`) |
| `git rev-parse HEAD` | PASS |
| `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` | PASS |
| `git diff --exit-code origin/main...HEAD -- docs/foundation-pass/fixtures` | PASS |
| `git diff --check origin/main...HEAD` | PASS |
| `git diff --name-only origin/main...HEAD` | PASS (docs only) |
| `git diff --name-only origin/main...HEAD -- ':!docs/**'` | PASS (empty) |
| `git status --short --branch` | PASS (clean) |
| `cargo fmt --all -- --check` | NOT RUN (no production changes) |
| `cargo check --all-targets --all-features --locked` | NOT RUN (no production changes) |
| `cargo test --all-targets --all-features --locked` | NOT RUN (no production changes) |
| `cargo clippy --all-targets --all-features --locked` | NOT RUN (no production changes) |
| `just verify` | NOT RUN (no production changes) |
| `just verify-agent` | NOT RUN (no production changes) |

All Rust/OCaml/integration test suites are NOT RUN because F3a changes
documentation only. Authorised by the packet's Required Verification section.

## Residual F3b questions

1. Does `fs::rename` after `sync_all()` on the temporary file guarantee
   directory-entry persistence after interruption or power loss? Is an
   explicit parent-directory flush required? (All StoreRoot-backed stores,
   Candidate Registry, Local Anchor Admission Store)
2. Does post-rename `FlushFileBuffers` on the renamed file handle ensure
   the parent directory entry is durable? (Replay Ledger)
3. Does `FILE_FLAG_WRITE_THROUGH` on temporary-file `CreateFileW` provide
   the expected file-data durability on the exact NTFS volume class?
   (Replay Ledger)
4. Can a partially written final JSONL line be recovered after interruption?
   (Trail)
5. Can a reparse point on the Local Anchor Admission Store root directory
   subvert admission records?
6. After interruption or power loss, does the `fs::rename` guarantee atomic
   visibility (no partial file visible under the final name)?
