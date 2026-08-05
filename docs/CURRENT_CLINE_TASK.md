# Current Implementation Task

Control contract: `1`
Task: `J24K3c1 - Read-only exact publication-state observer`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Red`
Route: `OpenCode using DeepSeek Pro V4 for one bounded security-sensitive Windows filesystem observation package; Lucy performs independent review and routine safe merge`
Base branch: `opencode/j24k3c1-recovery-observer`
Base commit: `5fb9efa0f64b88217d677ad36bf1b0595d7d39d7`
Implementation branch: `opencode/j24k3c1-recovery-observer`
Worker note: `docs/worker-notes/2026-08-05-j24k3c1-recovery-observer.md`
Implementation blueprint: `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
Rust toolchain: `1.97.1`
Accepted main: `753724c45500a03f876ca9008f7835d2147e2ea8`

## Objective

Implement only J24K3c1: one private, read-only observer for the exact publication transaction named by a validated `InstallationPublicationIntent`.

The observer reports three already-observed facts:

- whether `.staging-<transaction_id>` is present as one ordinary safe directory;
- whether the exact `destination_relative_path` is present as one ordinary safe directory;
- whether `<installed-record-root>/<transaction_id>.json` is present as one ordinary strictly decoded installed record.

It returns an owned snapshot that can be converted into the accepted J24K3b `InstallationRecoveryObservation` and classified later.

J24K3c1 does not verify destination contents, revalidate current evidence, audit unrelated installed-root entries, classify or execute recovery, clean staging, publish a record, remove an intent, acquire a lock, or wire the executor.

## Relevant background and existing behaviour

Accepted `main` is exactly:

```text
753724c45500a03f876ca9008f7835d2147e2ea8
```

J24K3a provides one strict private publication intent and store. J24K3b provides one pure classifier with four successful dispositions and fail-closed contradictory rows.

`InstalledPlugRegistry` owns private canonical install and record roots. Existing `load_all()` is unsuitable for partial-publication observation because it validates each installed record together with its final destination. During legitimate recovery, a record or destination may temporarily exist without the other.

Existing `StoreRoot`, `verify_chain`, `reject_reparse`, and `strict_json` provide accepted path and decoding primitives. `Path::exists()` is not an accepted observation primitive because it suppresses access errors and does not itself prove ordinary non-reparse entry type.

The frozen architecture requires recovery to distinguish exact staging, destination, and record presence while holding the installation lock. Lock integration remains later work; this package only supplies the read-only observation seam.

## Required behaviour

1. Add one private owned recovery snapshot and bridge to the accepted classifier observation.

Extend `installation_recovery.rs` with a crate-private type structurally equivalent to:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InstallationRecoverySnapshot {
    pub staging_present: bool,
    pub destination_present: bool,
    pub installed_record: Option<InstalledPlugRecord>,
}
```

Provide a narrow crate-private bridge equivalent to:

```rust
pub(crate) fn as_observation<'a>(
    &'a self,
    intent: &'a InstallationPublicationIntent,
) -> InstallationRecoveryObservation<'a>;
```

The snapshot contains facts and an optional owned record only. It contains no paths, store handles, mutable references, callbacks, authorities, clocks, or cleanup capability. Do not alter the existing classifier matrix or dispositions.

2. Add one exact read-only observer on `InstalledPlugRegistry`.

Provide a crate-private method structurally equivalent to:

```rust
pub(crate) fn observe_installation_recovery(
    &self,
    intent: &InstallationPublicationIntent,
) -> Result<InstallationRecoverySnapshot>;
```

The method must validate the intent first and then inspect only these exact paths derived from that validated intent:

- install root child `.staging-<transaction_id>`;
- install root child `destination_relative_path`;
- record root child `<transaction_id>.json`.

Do not expose the private roots through broad accessors. Do not accept caller-supplied paths or names.

3. Observe exact directories without following or mutating unsafe state.

For staging and destination, absence means only `symlink_metadata` returned `NotFound` for the exact child after the accepted root chain was rechecked.

When present, each exact entry must:

- pass `reject_reparse`;
- be an ordinary directory;
- remain inside the already-canonical accepted root by construction;
- be reported only as `true` or `false` without directory enumeration.

A file, symlink, junction, reparse point, or other non-directory entry at either exact path must fail closed. Do not create missing roots or children. Do not use `Path::exists()` as the decisive observation.

4. Observe the exact installed-record file without destination coupling.

The exact record path is `<record-root>/<transaction_id>.json`.

Absence means only exact-path `NotFound` and returns `None`.

When present, the entry must:

- pass `reject_reparse`;
- be one ordinary file;
- be read without mutation;
- use duplicate-key-refusing `strict_json` into `InstalledPlugRecord`;
- return the decoded record without requiring its destination to exist.

J24K3b remains responsible for `InstalledPlugRecord::validate` and exact equality with the intent record. Malformed JSON, duplicate keys, unknown fields, non-file state, or unreadable state must fail closed rather than appear absent.

5. Use stable safe observation errors.

Use only:

```text
installation_intent_invalid: installation publication intent is invalid
installation_recovery_conflict: installation recovery state conflicts with publication intent
installation_recovery_io: installation recovery state could not be observed
```

Preserving accepted `unsafe_store_path` for symlink, junction, or Windows reparse refusal is allowed.

Map malformed or non-ordinary exact entries to `installation_recovery_conflict`. Map genuine metadata, root-chain, read, or access failures to `installation_recovery_io`, except explicit unsafe-path refusal. Do not expose filesystem paths, package-controlled text, raw JSON, or OS error text in stable messages.

6. Add direct filesystem observation tests and complete full verification.

Add a private test module with test names prefixed `j24k3c1`.

Directly prove:

- empty exact transaction state;
- staging only;
- destination only;
- exact record only;
- all three facts represented together without classification or mutation;
- snapshot-to-observation bridging preserves exact booleans and record reference;
- malformed, duplicate-key, and unknown-field record JSON fail closed;
- exact record path as a directory fails closed;
- exact staging path as a file fails closed;
- exact destination path as a file fails closed;
- symlink or Windows junction/reparse refusal using accepted repository fixture conventions;
- unrelated install-root and record-root entries are not mistaken for this transaction and are not deleted;
- observation leaves all files, directories, and bytes unchanged;
- an invalid intent is rejected before any exact path state can influence the result.

Run the full required verification, not a narrower substitute.

## Relevant components

- `tethers-0.1/host-rust/src/installation_recovery.rs`
- `tethers-0.1/host-rust/src/installation_recovery_observation_tests.rs`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/m3_store.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `InstalledPlugRegistry`
- `InstallationPublicationIntent::validate`
- `InstallationRecoveryObservation`
- `StoreRoot::path`
- `verify_chain`, `reject_reparse`, and `strict_json`

`installation_publication_intent.rs` and `m3_store.rs` are accepted references and are not permitted edit targets.

## Frozen decisions and invariants

- J24K3c1 observes one exact validated-intent transaction only.
- The observer is read-only and performs no cleanup, publication, adoption, repair, or deletion.
- The observer does not classify recovery; it returns facts for J24K3b.
- Absence is only exact-path `NotFound`, never a suppressed generic error.
- Exact present staging and destination entries must be ordinary safe directories.
- Exact present record state must be one ordinary safe strict-JSON file.
- The observer does not require a record's destination to exist.
- Record semantic validation and equality remain in J24K3b.
- Unrelated entries are not scanned, adopted, rejected, or deleted here; global installed-root audit remains later work.
- Destination contents, lengths, hashes, permissions, and evidence freshness remain later work.
- Existing public installation behaviour and `load_all()` remain unchanged.
- No public API, dependency, Cargo configuration, Cargo.lock, CLI, prompt, output, enablement, operational-scope, packaging, release, or OCaml change is permitted.

## Acceptance criteria

1. The snapshot and observer seams are crate-private.
2. The snapshot contains only exact observed facts and an optional owned installed record.
3. The snapshot bridges into the accepted classifier observation without changing facts.
4. The observer validates the intent before inspecting transaction state.
5. Only exact intent-derived staging, destination, and record paths are inspected.
6. No broad root accessor or caller-supplied path seam is introduced.
7. Empty exact state returns both booleans false and no record.
8. Exact ordinary staging and destination directories are reported independently.
9. Exact ordinary strict record JSON is returned without destination coupling.
10. Non-ordinary, malformed, duplicate-key, unknown-field, symlink, junction, or reparse exact state fails closed.
11. Generic filesystem and read failures are not treated as absence.
12. Stable errors contain no unsafe path or package-controlled detail.
13. Unrelated root entries remain untouched and are not mistaken for the intent transaction.
14. Observation performs no filesystem mutation and preserves existing bytes and entries.
15. Tests directly exercise the production observer and classifier bridge.
16. Focused Nextest runs with zero retries and all new `j24k3c1` tests pass.
17. J24K3b, J24K3a, J24K2, J24J, and representative M3 regressions remain green.
18. Full `just verify` and the task packet checker pass.
19. Cargo.lock remains byte-identical and only permitted files change.
20. The task packet and worker note contain exact commands, counts, checkpoint SHA, discoveries, risks, and final remote tip.

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
  -E 'test(j24k3c1)'

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k3c1 `
  --locked

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

A pre-existing Windows handle-contention failure must be identified precisely, rerun serially, and pass before handoff. Do not replace full `just verify` with `just test-rust` or another narrower command.

OpenCode LSP is not a gate. Do not spend task time diagnosing empty LSP results.

## Forbidden changes

- No edit to `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`.
- No edit to `installation_publication_intent.rs`, `m3_store.rs`, `installation_execution.rs`, or the J24K3b classifier function and dispositions.
- No intent-store opening, intent discovery, or no-intent recovery path.
- No broad install-root or record-root accessor.
- No directory enumeration for staging or destination contents.
- No installed-root global audit or untracked-final detection.
- No destination file-set, length, hash, read-only, permission, or evidence verification.
- No current request, candidate, trust, launch-profile, conformance, approval, or suite revalidation.
- No staging cleanup, intent removal, record publication, destination adoption, deletion, rollback, repair, or any other mutation.
- No lock acquisition, executor context field, action wiring, planner change, or replacement of `installation_publication_deferred`.
- No public API, serialization schema, feature flag, dependency, Cargo configuration, Cargo.lock, CLI, prompt, terminal styling, enablement, operational-scope, packaging, release, or OCaml change.
- No files outside the permitted set.

Permitted files:

- `tethers-0.1/host-rust/src/installation_recovery.rs`;
- `tethers-0.1/host-rust/src/installation_recovery_observation_tests.rs`;
- `tethers-0.1/host-rust/src/installed.rs`;
- `tethers-0.1/host-rust/src/lib.rs`;
- `docs/CURRENT_CLINE_TASK.md`;
- `docs/worker-notes/2026-08-05-j24k3c1-recovery-observer.md`.

## Stop conditions

Stop as `BLOCKED` only if:

- exact read-only observation cannot be added without exposing broad root access;
- safe absence detection cannot distinguish exact `NotFound` from access failure;
- strict exact-record loading without destination coupling requires changing accepted persistence primitives;
- reliable Windows reparse coverage cannot be created using accepted repository fixture conventions;
- implementation requires destination verification, evidence revalidation, global audit, mutation, executor integration, a dependency, Cargo.lock change, or an out-of-scope file;
- full required verification still fails after one evidence-led correction.

Do not stop for failed LSP, a stale local branch ref, one ineffective Nextest filter, or one pre-existing Windows handle-contention failure that passes on the required serial rerun.

## Expected pre-existing changes

None. The branch is expected to be clean at handoff. The worker-note scaffold commit named by `Base commit` adds only `docs/worker-notes/2026-08-05-j24k3c1-recovery-observer.md`; the task-packet commit after it changes only `docs/CURRENT_CLINE_TASK.md`.
