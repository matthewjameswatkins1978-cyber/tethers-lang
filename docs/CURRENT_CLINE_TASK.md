# Current Implementation Task

Control contract: `1`
Task: `F3d - Remaining bounded persistence stores`
Owner: `OpenCode`
Model: `DeepSeek Pro`
Status: `IN_PROGRESS`
Task colour: `Amber`
Route: `DeepSeek Pro performs the bounded F3d persistence store characterization; Lucy independently reviews before F3e`
Worker note: `docs/worker-notes/2026-08-07-f3d-bounded-persistence-stores.md`
Base branch: `main`
Base commit: `40ec42eb2aac108901d428af3cbfe264d3edd6dc`
Implementation branch: `foundation/f3d-bounded-persistence-stores`
Parent branch: `main`
Parent tip: `40ec42eb2aac108901d428af3cbfe264d3edd6dc`
Preparation checkpoint: `40ec42eb2aac108901d428af3cbfe264d3edd6dc`
Implementation checkpoint: `(TBD after first commit)`
OCaml switch path: `N/A`
Rust toolchain: read exact channel from `rust-toolchain.toml`; use plain Cargo (resolved by root pin); `--locked` mandatory
Toolchain preflight: `pwsh -NoProfile -File scripts/check-dev-tools.ps1`

## Objective

Complete the persistence contract audit for the remaining bounded non-Trail/non-Replay stores.

For each store, determine whether the current implementation already satisfies its intended immutable/current-state/journal contract. Add characterization tests for untested properties. Repair only a directly demonstrated defect.

## Central evidence rule

For every claimed property separate:

1. **Observed implementation**
2. **Directly tested property**
3. **Remaining uncertainty**

And:

**PROVEN means there is a hard assertion that fails if that exact statement is false.**

Do not infer correctness from comments, function names, or nearby tests.

## F3d scope — 9 stores

### Immutable records (StoreRoot-backed)

1. **Candidate Registry** (`candidate.rs`)
2. **Publisher Trust Store** (`installation_trust.rs`)
3. **Developer Approval Store** (`trust.rs`)
4. **Launch Profile Evidence** (`launch_profile.rs`)
5. **Conformance Evidence** (`conformance.rs`)
6. **Installation Approval** (`installed.rs`, lines 46-320)
7. **Installed Plug Registry** — record-store contract only (`installed.rs`, lines 322-1474)
8. **Enablement Records** (`enablement.rs`)

### Remaining bounded journal (own filesystem access)

9. **Local Anchor Admission Store** (`local_anchor.rs`)

### Already closed elsewhere

- Installation Publication Intent: accepted F3c
- Installation Recovery Staging/journal: accepted F3c
- Trail: F3e
- Replay: F3e
- Installation Execution Lock: coordination artifact

## F3d evidence dimensions

For each store, characterize across these dimensions:

| Dimension | What to prove |
|---|---|
| Create conflict | Cannot silently overwrite different record; duplicate behaviour deterministic |
| Canonical identity | One canonical filename/path; record digest or identity validated on read |
| Duplicate behaviour | Exact duplicate create deterministic; duplicate logical identity fails closed |
| Malformed/torn state | Malformed bytes fail closed; .tmp remnants surfaced |
| Close/reopen | Valid record survives ordinary close/reopen |
| Corruption detection | Digest mismatch detected; corrupt record not treated as absence |
| Filename/content agreement | Filename identity disagreement fails closed |
| Chain/history validation | Where applicable: predecessor chain, ordering, restart reconstruction |
| Unsafe-path protection | Root validation, ancestor/reparse checks, escape prevention |
| Power-loss durability | UNVERIFIED (F3b) — never upgrade |
| Directory-entry durability | UNVERIFIED (F3b) — never upgrade |

## Required behaviour

1. F3d-1 — Immutable create contract: For each immutable store, characterize create conflict, canonical identity, duplicate behaviour, malformed/torn state, filename/content agreement.

2. F3d-2 — Restart/readback truth: For each store, characterize close/reopen survival, corruption detection, chain validation.

3. F3d-3 — Unsafe-path protection: For each store, characterize root validation, reparse checks, escape prevention. PROVEN only with direct negative test.

4. F3d-4 — Chain/history stores: For Publisher Trust and Enablement, characterize predecessor chain and restart reconstruction.

5. F3d-5 — Installed Plug Registry record boundary: Characterize record identity, create/conflict, digest validation, filename agreement, corruption classification. No publication sequencing changes.

6. F3d-6 — Local Anchor journal contract: Characterize event/admission identity, duplicate semantics, completion/evaluation, restart reconstruction, malformed handling, path safety.

## Relevant components

All files under `tethers-0.1/host-rust/src/`:
- `candidate.rs`
- `installation_trust.rs`
- `trust.rs`
- `launch_profile.rs`
- `conformance.rs`
- `installed.rs`
- `enablement.rs`
- `local_anchor.rs`
- `m3_store.rs`
- `f3d_bounded_persistence_stores_evidence.rs` (new)

Existing test files:
- `candidate.rs` inline tests
- `installation_trust.rs` (no inline tests; tested via `current_trust_tests.rs`)
- `trust.rs` inline tests
- `launch_profile.rs` (no inline tests)
- `conformance.rs` (no inline tests)
- `current_trust_tests.rs`
- `enablement.rs` inline tests
- `local_anchor.rs` inline tests

Documentation:
- `docs/architecture/TETHERS_FOUNDATION_PASS.md`
- `docs/foundation-pass/PERSISTENCE_INVENTORY.md`
- `docs/foundation-pass/DEBT_LEDGER.md`

## Relevant background and existing behaviour

F3a established the persistence inventory (4 classes: Immutable Atomic Record, Replaceable Current-State Record, Append-Only Causal Log, Multi-Step Intent/Recovery Journal). F3b characterized Windows primitive behaviour (sync_all + rename survival, parent-directory flush feasibility, power-loss UNVERIFIED). F3c audited Installation Publication Intent and Recovery.

The 9 remaining stores vary in maturity of existing test coverage:
- Candidate Registry has inline tests for torn .tmp, filename mismatch, duplicate identity, unsafe path
- Publisher Trust Store has chain validation, torn state, restart tests in trust.rs
- Developer Approval Store has basic approve/find test in trust.rs
- Launch Profile Evidence has comprehensive inline tests in j24h_installation_evidence_access.rs
- Conformance Evidence has tests in m3_lifecycle.rs and j24j_installation_reconciliation.rs
- Installation Approval is exercised through integration tests (j24k2, m3_lifecycle)
- Installed Plug Registry record contract is exercised through m3_lifecycle.rs and CLI tests
- Enablement Records have inline enable/disable/availability test
- Local Anchor Admission Store has comprehensive inline tests for duplicate, conflict, restart, corruption

Seven stores are StoreRoot-backed and share the StoreRoot contract for torn state, filename/id agreement, close/reopen, reparse protection, and digest validation. Candidate Registry and Local Anchor have custom filesystem access.

All files under `tethers-0.1/host-rust/src/`:
- `candidate.rs`
- `installation_trust.rs`
- `trust.rs`
- `launch_profile.rs`
- `conformance.rs`
- `installed.rs`
- `enablement.rs`
- `local_anchor.rs`
- `m3_store.rs`
- `f3d_bounded_persistence_stores_evidence.rs` (new)

Existing test files:
- `candidate.rs` inline tests
- `installation_trust.rs` (no inline tests; tested via `current_trust_tests.rs`)
- `trust.rs` inline tests
- `launch_profile.rs` (no inline tests)
- `conformance.rs` (no inline tests)
- `current_trust_tests.rs`
- `installation_recovery_*.rs` test files
- `enablement.rs` inline tests
- `local_anchor.rs` inline tests

Documentation:
- `docs/architecture/TETHERS_FOUNDATION_PASS.md`
- `docs/foundation-pass/PERSISTENCE_INVENTORY.md`
- `docs/foundation-pass/DEBT_LEDGER.md`

## Frozen decisions and invariants

- Accepted main: `40ec42eb2aac108901d428af3cbfe264d3edd6dc` (F3c merged)
- F3d is audit and characterization. Repair only directly demonstrated defects.
- No global persistence redesign. No StoreRoot rewrite.
- F3b UNVERIFIED platform properties preserved.
- No F3c architecture reopened.
- Trail, Replay untouched.

## Acceptance criteria

1. F3d characterization tests covering all 9 stores, with evidence matrix.
2. PERSISTENCE_INVENTORY.md updated with F3d evidence.
3. DEBT_LEDGER.md updated only for demonstrated defects.
4. F1 fixtures byte-identical.
5. Complete branch diff: tests + documentation only (unless a repair required).
6. F3d worker note records exact evidence and findings.

## Forbidden changes

- Universal persistence abstraction
- StoreRoot redesign
- Parent-directory flushing
- Trail/Replay modification
- F3c publication/recovery architecture changes
- CLI, JSON, exit codes, protocol, fixture changes
- Weakening existing negative tests
- Beginning F3e

## Stop conditions

STOP if:
- `origin/main` differs from `40ec42eb2aac108901d428af3cbfe264d3edd6dc`
- A required property cannot be characterized
- A repair would require redesign outside F3d
- A required check fails
- Two materially similar attempts fail

## Expected pre-existing changes

None

## Required verification

```powershell
git fetch origin --prune
git rev-parse origin/main
git rev-parse HEAD
git status --short --branch

cargo fmt --all -- --check
cargo check --all-targets --all-features --locked
cargo test --all-targets --all-features --locked
cargo clippy --all-targets --all-features --locked -- -W clippy::all

just verify
just verify-agent

pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1

git diff --exit-code origin/main...HEAD -- docs/foundation-pass/fixtures
git diff --check origin/main...HEAD
git diff --name-only origin/main...HEAD
git status --short --branch
```
