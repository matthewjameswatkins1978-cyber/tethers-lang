# Current Implementation Task

Control contract: `1`
Task: `J24K3c2 - Exact recovery destination verifier`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Red`
Route: `OpenCode using Kimi K2.7Code for one bounded repository-reading and security-sensitive Rust filesystem verification package; Lucy performs independent review and routine safe merge`
Base branch: `opencode/j24k3c2-destination-verifier`
Base commit: `e8b80c13728cf45911880b42734cc4f19fe6d73e`
Implementation branch: `opencode/j24k3c2-destination-verifier`
Worker note: `docs/worker-notes/2026-08-05-j24k3c2-destination-verifier.md`
Implementation blueprint: `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
Rust toolchain: `1.97.1`
Accepted main: `d8d827ae9aeab32ef2fbff7653086c6afad3be71`

## Objective

Implement only J24K3c2: one crate-private, read-only verifier that proves the exact final destination named by a validated `InstallationPublicationIntent` matches the intent's precomputed `InstalledPlugRecord`.

The verifier must prove:

- the already-opened install root still exists as one ordinary safe directory;
- the exact destination exists as one ordinary safe directory;
- the complete physical file set is exactly the record's `plug_json`, `payloads`, and `signature_files` paths;
- every expected file has the pinned length and SHA-256 digest;
- every expected file is read-only using the accepted installed-state permission test;
- every traversed entry and ancestor remains reparse-safe.

J24K3c2 also hardens the accepted J24K3c1 observer so that deletion or replacement of an already-opened install or record root cannot be misreported as an empty transaction state.

This package does not revalidate current trust, launch profile, conformance, approval, candidate source, or current suite. It does not audit unrelated `plug-*` destinations, classify recovery, mutate anything, publish a record, remove an intent, acquire the lock, or wire the executor.

## Relevant background and existing behaviour

Accepted `main` is exactly:

```text
d8d827ae9aeab32ef2fbff7653086c6afad3be71
```

J24K3a provides the validated publication intent and its complete precomputed record. J24K3b provides the pure recovery classifier. J24K3c1 observes exact staging, destination, and record presence without destination-content verification.

`InstalledPlugRegistry::load_all()` already verifies installed destinations against records, but it is unsuitable for this recovery seam because it scans all records and couples each record to a final destination. Recovery needs a narrow exact-intent verifier that can run when the destination exists but the record has not yet been published.

The accepted physical installed file universe is the same one used by publication and `load_all()`:

```text
plug_json + payloads + signature_files
```

`capability_manifests` are semantic references to manifest payload evidence and are not a second physical-copy list.

The accepted J24K3c1 root check currently calls `verify_chain`, which rejects unsafe ancestors but permits `NotFound` because that primitive is also used before safe creation. For an already-opened registry, a missing root must fail closed as `installation_recovery_io`, not make exact child paths look absent.

## Required behaviour

1. Add one shared existing-root guard for recovery reads.

In `installed.rs`, add or derive one private helper that accepts an already-canonical registry root and:

- calls `verify_chain`;
- preserves an explicit `unsafe_store_path` unchanged;
- maps every other chain failure to the stable `installation_recovery_io` error;
- uses `symlink_metadata` on the root itself;
- preserves explicit reparse refusal;
- requires the root to exist as one ordinary directory;
- maps missing, inaccessible, or non-directory root state to `installation_recovery_io` without exposing a path or OS error.

Use this guard for both roots in `observe_installation_recovery` and for the install root in the new destination verifier. Do not create or repair a missing root.

2. Add one exact read-only destination-verification seam.

Add a crate-private method on `InstalledPlugRegistry` structurally equivalent to:

```rust
pub(crate) fn verify_installation_recovery_destination(
    &self,
    intent: &InstallationPublicationIntent,
) -> Result<()>;
```

The method must validate the intent first, validate the existing install root, derive only `install_root / intent.destination_relative_path`, and verify that exact destination. It must accept no caller-supplied root, path, filename, expected map, or record other than the validated intent.

A missing destination is `installation_recovery_conflict`, not success and not an empty observation. A present destination must pass `reject_reparse` and be one ordinary directory.

3. Derive one exact expected physical file map from the precomputed record.

Build the expected set from:

- `intent.installed_record.plug_json`;
- `intent.installed_record.payloads`;
- `intent.installed_record.signature_files`.

Every expected path must be a non-empty normalized relative path containing only normal components and remaining beneath the exact destination by construction. Duplicate physical paths, absolute paths, parent traversal, current-directory components, prefix/root components, or separator-normalization ambiguity must fail as `installation_recovery_conflict`.

Do not add `capability_manifests` as an independent second physical list. Do not rewrite or repair evidence paths.

4. Verify the entire destination tree without following unsafe entries.

Recursively enumerate the exact destination only.

For every encountered child:

- reject symlinks, junctions, and Windows reparse points through the accepted `reject_reparse` primitive;
- recurse only into ordinary directories;
- collect only ordinary file paths normalized to `/` relative to the destination;
- reject any other entry type;
- map metadata, enumeration, or access failures to `installation_recovery_io` without leaking details.

The collected physical file set must equal the expected set exactly. Missing or extra files fail as `installation_recovery_conflict`. Unrelated entries outside the exact destination must not be scanned.

5. Verify every expected file's immutable evidence.

Immediately before reading each expected file:

- reject reparse state again;
- require one ordinary file;
- read without mutation;
- require the exact pinned `size_bytes`;
- require the exact pinned SHA-256 digest;
- require `metadata.permissions().readonly()` using the existing accepted installed-state rule.

A file-set, type, length, digest, permission, or evidence-path mismatch is `installation_recovery_conflict`. A genuine read, metadata, or access failure is `installation_recovery_io`. Explicit unsafe-path refusal remains `unsafe_store_path`.

6. Add direct production tests and complete full verification.

Add a private test module whose new test names are prefixed `j24k3c2`.

Directly prove:

- one exact valid flat destination passes;
- one exact valid nested destination passes;
- missing destination fails closed;
- destination-as-file fails closed;
- missing expected file fails closed;
- extra file fails closed;
- changed bytes with equal length fail by digest;
- changed length fails closed;
- writable expected file fails closed;
- nested symlink, junction, or reparse entry is refused;
- destination-root symlink, junction, or reparse entry is refused;
- duplicate or unsafe expected physical paths fail closed if they can be constructed through a validly digested intent fixture;
- unrelated sibling destinations remain untouched and are not scanned;
- verification leaves all directory entries, bytes, timestamps, and permissions unchanged;
- invalid intent is rejected before destination state can influence the result;
- missing already-opened install root returns `installation_recovery_io`;
- J24K3c1 observation returns `installation_recovery_io` when either already-opened registry root is removed, rather than reporting all exact state absent.

Retain all accepted J24K3c1 tests and run the full required verification.

## Relevant components

- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/installation_recovery_destination_tests.rs`
- `tethers-0.1/host-rust/src/installation_recovery_observation_tests.rs`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/installation_recovery.rs`
- `tethers-0.1/host-rust/src/m3_store.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `InstalledPlugRegistry`
- `InstallationPublicationIntent::validate`
- `InstalledPlugRecord`
- `PayloadEvidence`
- `verify_chain`, `reject_reparse`, `sha256`

`installation_publication_intent.rs`, `installation_recovery.rs`, and `m3_store.rs` are accepted references and are not permitted edit targets.

## Frozen decisions and invariants

- J24K3c2 is read-only and verifies one exact validated-intent destination only.
- Intent validation is the first operation.
- Already-opened registry roots must still exist as ordinary safe directories.
- Destination verification uses the precomputed record unchanged and does not recompute record identity or timestamps.
- The exact physical file universe remains `plug_json + payloads + signature_files`.
- Exact set, length, digest, read-only permission, and path safety are all mandatory.
- Explicit unsafe path state remains `unsafe_store_path`.
- Other filesystem observation failures use stable `installation_recovery_io` without detail.
- Structural or evidence mismatch uses stable `installation_recovery_conflict`.
- J24K3c1 observation remains fact-only and does not begin destination verification.
- Current authority and evidence freshness remain the next package.
- Global installed-root audit, mutation, cleanup, publication, intent removal, lock integration, planner, and executor wiring remain later work.
- Existing public installation and `load_all()` behaviour remain unchanged.
- No public API, dependency, Cargo configuration, Cargo.lock, CLI, prompt, output, enablement, operational-scope, packaging, release, or OCaml change is permitted.

## Acceptance criteria

1. The destination verifier is crate-private, read-only, and accepts only a validated intent.
2. Intent validation occurs before registry-root or destination inspection.
3. Both J24K3c1 observer roots and the J24K3c2 install root must still exist as ordinary safe directories.
4. Missing or non-directory registry roots return stable `installation_recovery_io`.
5. Explicit root or child reparse refusal remains `unsafe_store_path`.
6. Only the exact intent-derived destination is enumerated.
7. The expected physical set is derived only from `plug_json`, `payloads`, and `signature_files`.
8. Unsafe, ambiguous, or duplicate expected paths fail closed.
9. The complete actual physical file set must equal the expected set.
10. Every expected file must match pinned length and SHA-256 digest.
11. Every expected file must satisfy the accepted read-only permission rule.
12. Non-ordinary, symlink, junction, reparse, missing, extra, writable, or mismatched destination state fails closed.
13. Filesystem access failures do not appear as absence or mismatch and expose no unsafe detail.
14. Verification does not scan siblings or unrelated registry entries.
15. Verification performs no mutation and preserves bytes, entries, timestamps, and permissions.
16. Direct tests exercise the production observer and destination verifier seams.
17. Focused Nextest passes with zero retries and all `j24k3c2` tests pass.
18. All accepted J24K3c1, J24K3b, J24K3a, J24K2, J24J, and representative M3 regressions remain green.
19. Full `just verify` and the task packet checker pass.
20. Cargo.lock remains byte-identical and only permitted files change.
21. The task packet and worker note contain exact commands, counts, checkpoint SHA, discoveries, risks, and final remote tip.

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
  -E 'test(j24k3c2)'

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k3c2 `
  --locked

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
git log --oneline --decorate -10
```

Cargo.lock must remain:

```text
D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB
```

Do not substitute `just test-rust` for full `just verify`. A pre-existing intermittent Windows handle-contention failure must be identified precisely, rerun serially, and pass before handoff.

## Forbidden changes

- No edit to the frozen architecture.
- No edit to `installation_publication_intent.rs`, `installation_recovery.rs`, `m3_store.rs`, `installation_execution.rs`, or accepted evidence modules.
- No current-trust, launch-profile, conformance, approval, candidate-source, or current-suite revalidation.
- No global installed-root or record-root audit.
- No recovery classification, staging cleanup, destination deletion, record publication, intent removal, repair, adoption, lock, planner, or executor wiring.
- No mutation in production verification code.
- No broad root accessor, caller-supplied path, caller-supplied record, or caller-supplied expected-file map.
- No change to public installation, `load_all()`, snapshot, classifier, or intent semantics beyond requiring already-opened registry roots to continue existing during recovery observation.
- No public API, dependency, Cargo configuration, Cargo.lock, CLI, packaging, release, or OCaml change.
- No unrelated refactor.
- No files outside the permitted set.

Permitted files:

- `tethers-0.1/host-rust/src/installed.rs`;
- `tethers-0.1/host-rust/src/installation_recovery_destination_tests.rs`;
- `tethers-0.1/host-rust/src/installation_recovery_observation_tests.rs` only for existing-root regression coverage;
- `tethers-0.1/host-rust/src/lib.rs` only to register the new private test module;
- `docs/CURRENT_CLINE_TASK.md`;
- `docs/worker-notes/2026-08-05-j24k3c2-destination-verifier.md`.

## Stop conditions

Stop as `BLOCKED` only if exact destination verification requires changing an accepted evidence schema, public API, dependency, Cargo.lock, intent or classifier type, current-authority policy, or filesystem mutation; or if full verification still fails after one evidence-led correction.

Do not stop for failed LSP, a stale local ref, one ineffective Nextest filter, or an initial Windows junction fixture command that can be corrected using the accepted repository convention.

## Expected pre-existing changes

None. The branch is expected to be clean at handoff. The worker-note scaffold commit named by `Base commit` changes only the new worker note; this task-packet commit changes only `docs/CURRENT_CLINE_TASK.md`.
