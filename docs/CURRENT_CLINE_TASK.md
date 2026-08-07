# Current Implementation Task

Control contract: `1`
Task: `F3c - Installation intent and publication contract`
Owner: `OpenCode`
Model: `DeepSeek Pro`
Status: `IN_PROGRESS`
Task colour: `Amber`
Route: `DeepSeek Pro performs the bounded installation intent/publication audit and repair; Lucy independently reviews before F3d`
Worker note: `docs/worker-notes/2026-08-07-f3c-installation-intent-publication.md`
Base branch: `main`
Base commit: `71f79f7c80b2a09921ee59ac4b1acfa3926bf834`
Implementation branch: `foundation/f3c-installation-intent-publication`
Parent branch: `main`
Parent tip: `71f79f7c80b2a09921ee59ac4b1acfa3926bf834`
Preparation checkpoint: `71f79f7c80b2a09921ee59ac4b1acfa3926bf834`
Implementation checkpoint: `50a34f0dda50dfb13e178cb7410c13bcd765d345`
OCaml switch path: `N/A`
Rust toolchain: read exact channel from `rust-toolchain.toml`; use plain Cargo (resolved by root pin); `--locked` mandatory
Toolchain preflight: `pwsh -NoProfile -File scripts/check-dev-tools.ps1`

## Objective

Audit and, only where directly justified, align the specialised installation
intent/publication contract. F3c is NOT a universal persistence redesign.

The target is the existing installation sequence and its recovery semantics:

```text
intent -> staged filesystem state -> destination publication
       -> durable installation record -> exact-match intent removal/recovery
```

Preserve the specialised design where it is already correct.

## Central evidence rule

For every claimed property separate:

1. **Observed implementation**
2. **Directly tested property**
3. **Remaining uncertainty**

And:

**PROVEN means there is a hard assertion that fails if that exact statement is false.**

Do not infer correctness from comments, function names, API names, nearby tests
or a green integration suite.

## Relevant background and existing behaviour

The installation publication system at `71f79f7c80b2a09921ee59ac4b1acfa3926bf834`
has a specialised, well-tested contract:

- **InstallationPublicationIntent** (`installation_publication_intent.rs`):
  singleton `current.json` in a StoreRoot-backed directory; digest covers all
  content fields via `covered_bytes()`; validation enforces UUID canonicality,
  schema pin, cross-field consistency, and digest integrity.

- **Exact-match removal** (`remove_if_matches`): validates expected intent,
  loads current, compares with `!=`, returns `Ok(true)` on match,
  `Ok(false)` on absent, `Err(conflict())` on mismatch.

- **Publication mutation** (`installation_publication_mutation.rs`):
  writes intent -> builds staging -> renames staging to destination
  -> publishes record -> removes exact-matching intent via recovery.

- **Recovery classification** (`installation_recovery.rs:35-69`):
  maps 4 valid states to dispositions; all 4 invalid states return conflict.

- **19 intent tests** in `installation_publication_intent_tests.rs`
- **16 recovery tests** in `installation_recovery_tests.rs`
- Comprehensive mutation tests in `installation_publication_mutation_tests.rs`

### F3b constraints carried forward

- flush operation accepted: PROVEN
- bytes survive close/reopen: PROVEN
- pre-rename absence and post-rename complete bytes: PROVEN
- concurrent atomic visibility during rename: UNVERIFIED
- file data surviving sudden power loss: UNVERIFIED
- directory-entry survival after power loss: UNVERIFIED

F3c must not upgrade any UNVERIFIED property to PROVEN.

## Required behaviour

1. **F3c-1 — Publication intent identity**: Establish directly: the intent has one canonical identity; stored bytes/digest bind the exact intended installation operation; conflicting intent cannot silently replace an existing different intent; exact duplicate/retry behaviour is deterministic; the singleton `current.json` contract is correctly enforced; malformed or duplicate intent state fails closed.
2. **F3c-2 — Exact-match removal**: Directly prove that intent removal occurs only when the persisted intent exactly matches the expected operation/identity. Required negative properties: wrong digest cannot remove; wrong installation identity cannot remove; stale intent cannot remove a newer/different intent; malformed state cannot be converted into absence; missing intent is distinguished from mismatched intent.
3. **F3c-3 — Publication ordering**: Map and test the exact normal publication sequence. Identify the points at which: intent exists; staging exists; final destination exists; durable installed-record exists; intent has been removed. Prove the permitted ordering directly.
4. **F3c-4 — Recovery state matrix**: Audit `classify_installation_recovery` against every materially distinct observable combination of intent/staging/destination/record presence and matching/mismatching identity. For every state, prove one explicit outcome.
5. **F3c-5 — Recovery must not destroy evidence**: Directly prove negative properties: mismatching destination is never deleted; mismatching installed record is never overwritten; unrelated staging is never removed; wrong intent is never cleared; corruption/tamper evidence is preserved; recovery does not silently normalise an ambiguous state into success.
6. **F3c-6 — Canonical bytes / digest truth**: Directly prove: digest is computed over the intended canonical representation; persisted/read-back identity is checked; filename/record identity disagreement fails closed; recovery decisions use validated persisted state.

## Relevant components

- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/installation_publication_intent_tests.rs`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/installation_execution.rs`
- `tethers-0.1/host-rust/src/installation_plan.rs`
- `tethers-0.1/host-rust/src/installation_publication_preparation.rs`
- `tethers-0.1/host-rust/src/installation_publication_mutation.rs`
- `tethers-0.1/host-rust/src/installation_publication_mutation_tests.rs`
- `tethers-0.1/host-rust/src/installation_recovery.rs`
- `tethers-0.1/host-rust/src/installation_recovery_execution.rs`
- `tethers-0.1/host-rust/src/installation_recovery_plan.rs`
- `tethers-0.1/host-rust/src/installation_recovery_tests.rs`
- `tethers-0.1/host-rust/src/installation_recovery_evidence.rs`
- `docs/architecture/TETHERS_FOUNDATION_PASS.md`
- `docs/foundation-pass/PERSISTENCE_INVENTORY.md`
- `docs/foundation-pass/DEBT_LEDGER.md`

## Frozen decisions and invariants

- Accepted main is `71f79f7c80b2a09921ee59ac4b1acfa3926bf834` (F3b merged).
- F3c is audit and evidence. Do not redesign the intent format, publication
  flow, recovery, or any other persistence store.
- Every PROVEN claim must map to a hard assertion.
- Document UNVERIFIED properties from F3b without upgrading them.
- One implementation owner per task. Do not begin F3d or F3e.

## Expected pre-existing changes

None. Branch is created from accepted F3b main. Only the files listed in this packet, the F3c characterization tests, and documentation updates should change.

## Acceptance criteria

1. F3c-1 through F3c-6 each have direct characterization tests proving the
   named properties. Every property uses PROVEN, DISPROVEN, or UNVERIFIED.
2. PERSISTENCE_INVENTORY.md updated with F3c evidence tags.
3. DEBT_LEDGER.md updated only for directly demonstrated defects/clarifications.
4. F1 fixtures are byte-identical.
5. Complete branch diff: characterization tests + documentation only.
6. F3c worker note records exact evidence and findings.

## Forbidden changes

Do not:
- create a universal persistence layer;
- redesign StoreRoot globally;
- add parent-directory flushing;
- repair Candidate Registry or Local Anchor;
- redesign Trail or Replay;
- begin F3d or F3e;
- change public CLI shape, JSON envelopes, exit codes, protocol semantics, or
  F1 compatibility fixtures;
- delete or weaken an existing direct negative-property test.

## Stop conditions

STOP if:
- `origin/main` differs from `71f79f7c80b2a09921ee59ac4b1acfa3926bf834`;
- a required property cannot be characterized;
- a repair would require redesign outside F3c;
- a required check fails;
- two materially similar attempts fail.

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

Run every focused F3c test explicitly and record it separately.
