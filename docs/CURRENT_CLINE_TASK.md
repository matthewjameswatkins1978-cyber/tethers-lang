# Current Implementation Task

Control contract: `1`
Task: `J24K3a - Private publication intent record and atomic persistence`
Owner: `OpenCode`
Status: `READY`
Task colour: `Red`
Route: `OpenCode using DeepSeek Pro V4 for one bounded security-sensitive Rust persistence package; Lucy performs independent review and routine safe merge`
Base branch: `opencode/j24k3a-publication-intent-store`
Base commit: `bff2d53a7951b8f32bbdfdfa62a67091a7f018cb`
Implementation branch: `opencode/j24k3a-publication-intent-store`
Worker note: `docs/worker-notes/2026-08-04-j24k3a-publication-intent-store.md`
Implementation blueprint: `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
Rust toolchain: `1.97.1`
Accepted main: `904be6c25c74832a6a5197e6e2ae0cdc798f9d45`

## Objective

Implement only J24K3a: one private, digest-covered publication-intent record and one private atomic single-record persistence store at:

```text
<executor-state-root>/installation-intent/current.json
```

The intent must pin one exact precomputed `InstalledPlugRecord`, its digest, candidate identity, destination identity, and publication transaction identity.

J24K3a does not construct the installed record, stage or rename payloads, publish an installed record, classify recovery state, inspect the installed root, recover anything, or alter the J24K2 executor.

## Relevant background and existing behaviour

Accepted `main` is exactly:

```text
904be6c25c74832a6a5197e6e2ae0cdc798f9d45
```

J24K2 is accepted and currently refuses `PublishDisabledInstallation` with `installation_publication_deferred` before mutation.

The current legacy installed path in `installed.rs`:

1. revalidates current evidence;
2. creates and verifies a staging directory;
3. copies exact files and marks them read-only;
4. renames staging to `plug-<installed_id>`;
5. constructs and validates `InstalledPlugRecord`;
6. publishes the immutable installed record.

A crash between steps 4 and 6 can leave an invisible final destination.

Existing accepted persistence machinery in `m3_store.rs` already provides:

- absolute and reparse-safe `StoreRoot` opening;
- canonical JSON;
- duplicate-key-refusing strict JSON;
- exclusive `create_new` temporary writes;
- `write_all` and `sync_all`;
- same-directory atomic rename;
- deterministic entry enumeration;
- immutable create-only semantics.

`InstalledPlugRecord::validate` already verifies its complete record digest, including `created_unix_ms`. Recovery must later publish that exact record without refreshing its timestamp or identity.

The frozen J24K architecture is authoritative. Do not edit it.

## Required behaviour

1. Private module and seam

Add:

```text
tethers-0.1/host-rust/src/installation_publication_intent.rs
tethers-0.1/host-rust/src/installation_publication_intent_tests.rs
```

Expose only inside the crate:

```rust
mod installation_publication_intent;
#[cfg(test)]
mod installation_publication_intent_tests;
```

The record, store, and methods required by future J24K packages are `pub(crate)`, not public. Do not add a public re-export, CLI seam, test bypass, or feature flag.

2. Exact intent record

Add a serde record structurally equivalent to:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct InstallationPublicationIntent {
    pub schema_version: u32,
    pub transaction_id: String,
    pub candidate_id: String,
    pub destination_relative_path: String,
    pub installed_record: InstalledPlugRecord,
    pub installed_record_digest: String,
    pub intent_digest: String,
}
```

Use the precomputed installed record's `installed_id` as the publication `transaction_id`. Do not mint a second unrelated UUID.

Provide a crate-private constructor equivalent to:

```rust
pub(crate) fn from_precomputed_record(record: InstalledPlugRecord) -> Result<Self>;
```

The constructor must:

- validate the supplied installed record;
- copy, not regenerate, its `installed_id`, `source_candidate_id`, `installation_relative_path`, `record_digest`, and `created_unix_ms`;
- perform no clock read;
- generate no UUID;
- compute `intent_digest` over canonical covered bytes with only `intent_digest` cleared.

Validation must require:

- schema version exactly `1`;
- canonical lowercase hyphenated UUID transaction identity;
- `transaction_id == installed_record.installed_id`;
- `candidate_id == installed_record.source_candidate_id`;
- `destination_relative_path == installed_record.installation_relative_path`;
- destination exactly `plug-<transaction_id>`;
- destination is one safe relative path component with no `/`, `\\`, `.`, `..`, absolute prefix, or alternate spelling;
- `installed_record.validate()` succeeds;
- `installed_record_digest == installed_record.record_digest`;
- `intent_digest` equals SHA-256 of canonical covered bytes.

All fields, including the full embedded record and its `created_unix_ms`, must be covered by the intent digest.

3. Private single-record store

Add a crate-private store equivalent to:

```rust
pub(crate) struct InstallationPublicationIntentStore {
    root: StoreRoot,
}
```

It accepts an absolute executor-state root and owns exactly its `installation-intent` child.

Provide narrow `open` and `open_existing` forms following existing store conventions. They must use accepted `StoreRoot` and path-safety primitives rather than duplicating or weakening them.

The intent root may contain only:

```text
current.json
```

or be empty.

4. Atomic create without overwrite

Provide a create method that accepts a validated intent and publishes exactly:

```text
installation-intent/current.json
```

Use existing canonical, temporary-write, `sync_all`, and same-directory rename machinery, preferably `StoreRoot::create_json("current", ...)`.

Creation must:

- succeed only when the intent root is empty;
- never overwrite a valid current intent;
- never replace, ignore, or delete malformed, torn, duplicate, or unknown state;
- leave the existing bytes unchanged on conflict;
- leave no temporary file after ordinary success.

5. Strict load and fail-closed entry handling

Provide:

```rust
pub(crate) fn load(&self) -> Result<Option<InstallationPublicationIntent>>;
```

Loading must return:

- `Ok(None)` only for an empty, valid intent root;
- `Ok(Some(intent))` only for one ordinary non-reparse `current.json` whose strict JSON and record validation succeed.

Refuse as `installation_intent_invalid`:

- `.current.tmp` or any other `.tmp` entry;
- any unknown file or directory;
- multiple entries;
- malformed JSON;
- duplicate JSON keys;
- unknown JSON fields;
- mismatched filename or identity;
- invalid embedded installed record;
- changed duplicated fields or digests.

Never treat malformed or torn state as absence.

6. Exact-match removal only

Provide an exact-match removal seam structurally equivalent to:

```rust
pub(crate) fn remove_if_matches(
    &self,
    expected: &InstallationPublicationIntent,
) -> Result<bool>;
```

It must:

- validate `expected`;
- call the same strict `load` path;
- return `Ok(false)` if the valid root is empty;
- refuse a different current transaction or any unequal intent without removing it;
- reject reparse or non-ordinary `current.json` immediately before deletion;
- remove only the exact matching `current.json` and return `Ok(true)`;
- never delete temporary files, unknown entries, directories, staging payloads, destinations, installed records, or the intent root itself.

Use stable safe errors:

```text
installation_intent_invalid: installation publication intent is invalid
installation_intent_conflict: installation publication intent conflicts with current state
installation_intent_io: installation publication intent could not be persisted
```

Preserving the accepted `unsafe_store_path` code for path/reparse refusal is allowed. Do not expose filesystem paths, package-controlled text, or raw JSON in stable messages.

## Relevant components

- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/installation_publication_intent_tests.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/m3_store.rs`
- `InstalledPlugRecord::validate`
- `StoreRoot::{open, open_existing, entries, read, create_json}`
- `canonical`, `sha256`, `strict_json`, `verify_chain`, `reject_reparse`

`installed.rs` and `m3_store.rs` are references, not permitted edit targets.

## Frozen decisions and invariants

- J24K3a is one private persistence concept only.
- The complete installed record is precomputed before the intent layer receives it.
- The installed record's `installed_id` is the transaction identity.
- The intent constructor preserves the installed record byte-for-byte and does not refresh `created_unix_ms`.
- Canonical digest coverage includes every intent field except the cleared `intent_digest` field itself.
- One fixed `current.json` exists or no intent exists.
- Valid existing intent is never overwritten.
- Malformed, torn, unknown, duplicate, or unsafe state is never treated as empty.
- Removal requires exact equality with the currently loaded validated intent.
- Existing `StoreRoot` safety and atomic-create semantics are reused, not forked.
- No executor integration, recovery decision, destination verification, installed-root audit, staging, rename, installed-record publication, or cleanup belongs to J24K3a.
- No public API, dependency, schema outside the private intent, Cargo configuration, Cargo.lock, CLI, prompt, output, enablement, operational-scope, packaging, release, or OCaml change is permitted.

## Acceptance criteria

1. The new intent module is private and exposes only crate-private seams needed by later J24K packages.
2. A valid precomputed `InstalledPlugRecord` produces a valid intent without changing its identity, digest, or `created_unix_ms`.
3. The record uses `installed_id` as transaction identity and validates every duplicated identity field.
4. Intent digest validation detects changes to transaction, candidate, destination, embedded record, explicit record digest, and embedded creation time.
5. Destination identity is exactly one safe `plug-<transaction_id>` relative component.
6. Atomic create writes canonical `current.json`, leaves no ordinary-success temp file, and never overwrites existing state.
7. Empty-root load returns `None`; exact valid current intent round-trips as strict equality.
8. Malformed JSON, duplicate keys, unknown fields, invalid embedded record, changed digests, and changed duplicated fields fail closed.
9. Torn temp, unknown entry, directory entry, and multiple-entry states fail closed rather than appearing absent.
10. Exact-match removal deletes only the matching validated current intent; absent returns false.
11. Mismatched removal refuses and preserves the original bytes.
12. Path, symlink, and Windows reparse safety remain at least as strict as existing `StoreRoot` behaviour.
13. Tests exercise the new production behaviour directly through the crate-private seam; source inspection or adjacent store tests are not substitutes.
14. Focused Nextest runs with zero retries and all new `j24k3a` tests pass.
15. J24K2, J24J, and M3 lifecycle representative regressions remain green.
16. Full verification passes with at least the accepted 966-test baseline plus new tests.
17. Cargo.lock remains byte-identical and only permitted files change.
18. The task packet and worker note are updated with exact commands, counts, discoveries, checkpoint SHA, and final branch tip.

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
  -E 'test(j24k3a)'

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
git log --oneline --decorate -5
```

The focused Nextest expression may be adjusted once if discovery reports a different exact test name. Record exact run, pass, skip, and retry counts. Do not repeat ineffective filters blindly.

OpenCode LSP is not a gate. Do not spend task time diagnosing empty LSP results.

Cargo.lock must remain:

```text
D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB
```

No full-verification failure is acceptable after `$PSHOME` is prepended to PATH.

## Forbidden changes

- No edit to `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`.
- No edit to `installed.rs` or `m3_store.rs`.
- No installed-record constructor extraction or publication refactor.
- No executor context field, action wiring, replacement of `installation_publication_deferred`, or planning change.
- No recovery-state enum, recovery matrix, evidence revalidation, destination verification, installed-root scanning, or global integrity audit.
- No staging directory, payload copy, destination rename, installed-record publication, adoption, deletion, rollback, or repair.
- No public module, public record/store API, test-only production bypass, optional fallback, global state, or feature flag.
- No second transaction UUID unrelated to the precomputed `installed_id`.
- No dependency, Cargo configuration, tool configuration, Cargo.lock, CLI, prompt, terminal styling, enablement, operational-scope, packaging, release, or OCaml changes.
- No files outside the permitted set.

Permitted files:

- `tethers-0.1/host-rust/src/installation_publication_intent.rs`;
- `tethers-0.1/host-rust/src/installation_publication_intent_tests.rs`;
- `tethers-0.1/host-rust/src/lib.rs`;
- `docs/CURRENT_CLINE_TASK.md`;
- `docs/worker-notes/2026-08-04-j24k3a-publication-intent-store.md`.

## Stop conditions

Stop as `BLOCKED` only if:

- a valid `InstalledPlugRecord` cannot be embedded, strictly deserialized, and revalidated without changing `installed.rs`;
- existing `StoreRoot` cannot safely implement one fixed atomic `current.json` record without changing `m3_store.rs`;
- exact-match removal cannot be implemented without a broad or unsafe delete seam;
- safe implementation requires executor integration, recovery classification, destination inspection, installed-root scanning, schema outside the private intent, a new dependency, or an out-of-scope file;
- required verification still fails after one evidence-led correction.

Do not stop for failed LSP, one ineffective Nextest filter, a stale local branch ref, or a failed broad text replacement. Reread the current file and make one smaller evidence-led correction.

## Expected pre-existing changes

None. The branch is expected to be clean at handoff. The documentation scaffold commit named by `Base commit` is the implementation base; the task-packet commit after it changes only `docs/CURRENT_CLINE_TASK.md`.
