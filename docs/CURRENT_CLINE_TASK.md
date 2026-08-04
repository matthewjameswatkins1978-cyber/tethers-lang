# Current Implementation Task

Control contract: `1`
Task: `J24K3b correction - record validation ordering and final verification`
Owner: `OpenCode`
Status: `READY`
Task colour: `Red`
Route: `OpenCode using DeepSeek Pro V4 for one bounded Rust correction; Lucy performs independent review and routine safe merge`
Base branch: `opencode/j24k3b-recovery-classifier`
Base commit: `e09a16004a9f634e99e39491e2469a6cb5ec337d`
Implementation branch: `opencode/j24k3b-recovery-classifier`
Worker note: `docs/worker-notes/2026-08-04-j24k3b-recovery-classifier.md`
Implementation blueprint: `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
Rust toolchain: `1.97.1`
Accepted main: `9402a9f5d312c3523cc81fd2682431056fe55d97`

## Objective

Correct and finish the existing J24K3b pure recovery-state classifier without changing its matrix or package boundary.

The reviewed implementation at `ef13e1d5a83ea8adea59aafc1557c3e70f69ba6f` is structurally sound, but it validates an installed record only in the destination-plus-record arm. The authoritative contract requires validation of every present installed record immediately after intent validation and before matrix classification.

The earlier handoff also ran `just test-rust` rather than the required full `just verify`. The packet checker failed because the previous packet used nested ordered lists that were counted as 23 required behaviours against 18 acceptance criteria. This replacement packet repairs that coordinator-authored structural defect.

## Relevant background and existing behaviour

Accepted main is:

```text
9402a9f5d312c3523cc81fd2682431056fe55d97
```

The branch already contains:

- one private `InstallationRecoveryObservation`;
- one four-variant private `InstallationRecoveryDisposition`;
- one pure `classify_installation_recovery` function;
- fourteen direct `j24k3b` tests;
- no filesystem access, mutation, public API, dependency, or executor wiring.

The four successful rows remain unchanged:

- intent only -> `RemoveIntentOnly`;
- intent plus staging only -> `RemoveStagingThenIntent`;
- intent plus destination only -> `RevalidateDestinationThenPublishRecord`;
- intent plus destination plus exact matching record -> `VerifyCompletedPublicationThenRemoveIntent`.

Every other combination fails as `installation_recovery_conflict`. Invalid intent fails as `installation_intent_invalid`.

## Required behaviour

1. Validate the intent first, then validate every present installed record before applying any staging, destination, or equality row.

A present record that fails `InstalledPlugRecord::validate` must map to `installation_recovery_conflict`. No lower-layer installed-record error may escape. This must apply even when the observation is already contradictory, including record without destination and staging plus destination.

2. Preserve the existing exact recovery matrix and full-record equality rule.

Do not add, remove, rename, or broaden successful dispositions. A valid present record matches only by exact equality with `intent.installed_record`. Staging plus destination always conflicts. Record without destination always conflicts. Intent validation remains first.

3. Add direct edge tests for the corrected validation path.

Add `j24k3b` tests that supply an invalid installed record in at least these broad conflict states:

- record present without destination;
- staging plus destination plus invalid record.

Both must return only `installation_recovery_conflict`. Retain all existing direct tests.

4. Complete the actual required verification and control evidence.

The corrected packet checker must pass. Full `$env:PATH = "$PSHOME;$env:PATH"; just verify` must pass, not merely `just test-rust`. Update the worker note with exact commands, counts, checkpoint SHA, final remote tip, discoveries, and any unrelated Windows handle-contention rerun.

## Relevant components

- `tethers-0.1/host-rust/src/installation_recovery.rs`
- `tethers-0.1/host-rust/src/installation_recovery_tests.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/installed.rs`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-04-j24k3b-recovery-classifier.md`

The accepted intent and installed modules are references only and must not be edited.

## Frozen decisions and invariants

- J24K3b remains a private pure classifier.
- Intent validation occurs first.
- Every present installed record is validated before matrix classification.
- Invalid or unequal records produce `installation_recovery_conflict`.
- Matching requires validated exact full-record equality.
- The four successful dispositions remain exact and unchanged.
- No filesystem access, path observation, evidence revalidation, global audit, mutation, cleanup, publication, lock, planner, executor integration, or public API belongs here.
- No dependency, Cargo configuration, Cargo.lock, CLI, prompt, output, packaging, release, or OCaml change is permitted.

## Acceptance criteria

1. Intent validation remains the first classifier operation.
2. Every `Some(installed_record)` is validated before any matrix row is selected.
3. Invalid record errors map only to `installation_recovery_conflict`.
4. The four successful rows and disposition enum remain unchanged.
5. Full-record equality remains mandatory for completed-publication classification.
6. Existing fourteen direct tests remain green.
7. Direct tests cover invalid record without destination and invalid record with staging plus destination.
8. The implementation remains pure, deterministic, crate-private, and mutation-free.
9. Focused Nextest passes with zero retries.
10. J24K3a, J24K2, J24J, and M3 regressions remain green.
11. Full `just verify` and the corrected packet checker pass.
12. Cargo.lock remains byte-identical and only permitted files change.
13. The task packet and worker note contain exact final evidence and a clean remote tip.

## Required verification

Run from repository root:

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1

cargo fmt `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all -- --check

cargo nextest run `
  --config-file .config/nextest.toml `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all-features --locked `
  -E 'test(j24k3b)'

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k3b `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k3a `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k2 `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --test j24j_installation_reconciliation `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --test m3_lifecycle `
  --locked

$env:PATH = "$PSHOME;$env:PATH"
just verify

Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256
git diff --check
git status --short
git log --oneline --decorate -8
```

Cargo.lock must remain:

```text
D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB
```

A pre-existing Windows handle-contention failure must be identified precisely, rerun serially, and pass before handoff. Do not replace full `just verify` with a narrower command.

## Forbidden changes

- No edit to the frozen architecture.
- No edit to `installation_publication_intent.rs`, `installed.rs`, `m3_store.rs`, or `installation_execution.rs`.
- No new recovery row or disposition.
- No filesystem access, store opening, path derivation, destination verification, installed-root audit, evidence revalidation, cleanup, publication, deletion, repair, lock, planner, or executor wiring.
- No public API, serialization schema, dependency, Cargo configuration, Cargo.lock, CLI, packaging, release, or OCaml change.
- No unrelated refactor or test-helper expansion beyond what the two direct edge tests require.
- No files outside the permitted set.

Permitted files:

- `tethers-0.1/host-rust/src/installation_recovery.rs`;
- `tethers-0.1/host-rust/src/installation_recovery_tests.rs`;
- `tethers-0.1/host-rust/src/lib.rs` only if genuinely required;
- `docs/CURRENT_CLINE_TASK.md`;
- `docs/worker-notes/2026-08-04-j24k3b-recovery-classifier.md`.

## Stop conditions

Stop as `BLOCKED` only if validating every present record before matrix classification changes a frozen successful row, requires an accepted-module edit, requires I/O or mutation, requires a public API or dependency, or full verification still fails after one evidence-led correction.

Do not stop for failed LSP, a stale local ref, one ineffective Nextest filter, or the already-repaired packet-count issue.

## Expected pre-existing changes

None. The branch is expected to be clean at handoff. The worker-note scaffold commit named by `Base commit` is the correction base; the task-packet commit after it changes only `docs/CURRENT_CLINE_TASK.md`.
