# Current Implementation Task

Control contract: `1`
Task: `J24K3c4 - Global installed-root consistency auditor`
Owner: `OpenCode`
Status: `READY`
Task colour: `Red`
Route: `OpenCode using DeepSeek Pro for one bounded, read-only Rust filesystem and record-reconciliation package; Lucy performs independent review and routine safe merge`
Base branch: `opencode/j24k3c4-installed-root-audit`
Base commit: `612b43eaa92d2142975d2dd4878561a1f60e4313`
Implementation branch: `opencode/j24k3c4-installed-root-audit`
Worker note: `docs/worker-notes/2026-08-05-j24k3c4-installed-root-audit.md`
Implementation blueprint: `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
Rust toolchain: `1.97.1`
Accepted main: `e95e061e815d69b91b0637d08f84caaa602f1772`

## Objective

Implement only J24K3c4: one crate-private, read-only global installed-root consistency audit.

Given the optional current `InstallationPublicationIntent`, the audit must prove that every direct final-destination namespace entry beginning `plug-` is accounted for by either:

- one validated installed record whose exact destination is `plug-<installed_id>`; or
- the single validated current publication intent and its exact destination.

Any unexplained final destination is a global integrity failure. The audit must not adopt it, delete it, repair it, classify transaction recovery, publish a record, remove an intent, acquire a lock, plan another action, or wire the executor.

## Relevant background and existing behaviour

Accepted `main` is exactly:

```text
e95e061e815d69b91b0637d08f84caaa602f1772
```

J24K3a provides the durable validated publication intent. J24K3b provides the pure recovery classifier. J24K3c1 observes the exact staging, destination, and record paths for one transaction. J24K3c2 verifies the exact current-intent destination file set, hashes, lengths, permissions, and path safety. J24K3c3 revalidates the complete request, candidate, trust, launch, conformance, approval, and installed-record evidence chain.

`InstalledPlugRegistry::load_all()` validates every installed record and the destination named by that record. It does not enumerate the install root to detect an orphan final directory that has no record. The frozen J24K architecture requires a global pass because final destination names are generated installed IDs and cannot be inferred from only the current candidate.

The current `InstalledPlugRecord::validate()` does not itself prove that `installation_relative_path` is exactly `plug-<installed_id>`. J24K3c4 must enforce that recovery identity invariant without changing the public record schema or ordinary `load_all()` behaviour.

## Required behaviour

### 1. Add one narrow crate-private audit seam

Add a method on `InstalledPlugRegistry` structurally equivalent to:

```rust
pub(crate) fn audit_installation_recovery_destinations(
    &self,
    intent: Option<&InstallationPublicationIntent>,
) -> Result<()>;
```

The caller supplies only the optional current validated intent. It must not supply a root, path, record set, allow-list, callback, deletion policy, or mutation capability.

### 2. Validate a supplied intent before filesystem or store access

When `intent` is `Some`, `intent.validate()` must be the first operation. Invalid intent returns the existing stable error:

```text
installation_intent_invalid: installation publication intent is invalid
```

When `intent` is `None`, no synthetic intent is created and the audit proceeds directly to root validation.

### 3. Require both already-opened registry roots to remain safe and present

Reuse the accepted J24K3c2 existing-root guard for both the install root and record root.

- explicit symlink, junction, or reparse refusal remains `unsafe_store_path`;
- missing, inaccessible, replaced, or non-directory registry roots return `installation_recovery_io`;
- do not recreate or repair a root.

### 4. Load and validate the complete installed-record set

Use the accepted installed-state validation boundary rather than creating a second payload validator. `InstalledPlugRegistry::load_all()` may be used directly, or its internal validation may be narrowly factored only if required to preserve stable recovery errors. Public `load_all()` behaviour must not change.

Every accepted installed record must additionally satisfy:

- `installed_id` is a canonical lowercase hyphenated UUID;
- `installation_relative_path` equals exactly `plug-<installed_id>`;
- no two records account for the same final destination path.

A malformed record, invalid record identity, missing tracked destination, payload drift, permission drift, duplicate destination claim, or other contradictory tracked state returns `installation_recovery_conflict` without exposing lower-layer detail. Explicit unsafe path state remains `unsafe_store_path`.

For failures already collapsed by accepted `load_all()` into a structural installed-state error, map conservatively to `installation_recovery_conflict`. Do not redesign public installed-state error semantics solely to distinguish operating-system failure from corruption. The audit's own root and direct-directory access failures use `installation_recovery_io`.

### 5. Cross-check an optional intent against installed records

A validated intent authorizes only `intent.destination_relative_path`.

- The intent destination may be absent. That is valid at this audit stage because J24K3c1 owns exact state observation.
- The intent destination may be present without an installed record. That is valid at this identity-audit stage because J24K3c2 and J24K3c3 own full destination and evidence revalidation before publication.
- If an installed record already claims the intent destination, that record must equal `intent.installed_record` exactly.
- A different record, repinned record, or contradictory claim at the intent destination returns `installation_recovery_conflict`.
- The intent must not authorize any sibling or package-release-equivalent destination.

Do not verify the content of an intent-only destination in this package. Do not repeat J24K3c2.

### 6. Enumerate the install root's direct final namespace

Enumerate direct children of the install root only. Final destinations are direct children and must never be discovered through recursive searching.

For every direct entry whose UTF-8 filename begins with `plug-`:

1. reject symlink, junction, and Windows reparse state using the accepted primitive;
2. require the filename to be exactly `plug-<canonical lowercase hyphenated UUID>`;
3. require an accounted exact final name from either the validated installed-record set or the optional current intent;
4. when the accounted path is present, require it to be one ordinary directory.

A `plug-*` entry with a malformed final name or no accounting record/intent returns:

```text
installation_destination_untracked: installed destination is not tracked by a validated record or current publication intent
```

An exact accounted name that is present as a non-directory returns `installation_recovery_conflict`. Explicit reparse refusal remains `unsafe_store_path`. Direct enumeration or metadata access failure returns `installation_recovery_io`.

A direct filename that cannot be represented as UTF-8 cannot be safely classified and returns `installation_recovery_conflict`.

### 7. Leave non-final private entries outside this audit

Direct entries not beginning with `plug-` are not final destinations and remain outside J24K3c4's classification boundary.

In particular:

- `.staging-<transaction-id>` remains owned by J24K3c1 and the recovery classifier;
- unrelated non-final test fixtures must not be adopted, deleted, renamed, recursively scanned, or treated as final destinations by this package.

This is not permission to accept an untracked `plug-*` entry with a malformed UUID. Every entry in the `plug-` namespace is fail-closed.

### 8. Use stable recovery-facing errors

The new audit exposes only:

```text
installation_intent_invalid: installation publication intent is invalid
installation_destination_untracked: installed destination is not tracked by a validated record or current publication intent
installation_recovery_conflict: installation recovery state conflicts with publication intent
installation_recovery_io: installation recovery state could not be observed
```

Explicit accepted `unsafe_store_path` remains allowed. Do not expose filesystem paths, record-controlled strings, raw JSON, or operating-system diagnostics.

### 9. Add direct tests and complete verification

Add a private test module whose test names begin `j24k3c4`.

Directly prove at minimum:

- empty install and record roots pass with no intent;
- one or more complete validated installed records and destinations pass with no intent;
- a valid intent whose destination is absent passes the identity audit;
- a valid intent destination present without a record passes the identity audit;
- a matching intent, record, and destination pass;
- one untracked canonical final directory fails with `installation_destination_untracked` when no intent exists;
- an authorised intent destination does not excuse a second untracked final directory;
- malformed `plug-*` directory names fail as untracked;
- an untracked canonical `plug-*` ordinary file fails as untracked;
- an accounted canonical destination present as a file fails as recovery conflict;
- final-destination symlink, junction, or reparse state remains `unsafe_store_path`;
- a record whose destination is not exactly `plug-<installed_id>` fails closed after its digest and fixture state are validly rebuilt;
- an intent and existing record that claim the same destination with different complete records fail as recovery conflict;
- a tracked record whose destination is missing or whose installed bytes drift fails as recovery conflict;
- staging and unrelated non-final entries are ignored and remain untouched;
- invalid intent wins before missing roots or orphan destinations can influence the result;
- removal of either already-opened registry root returns `installation_recovery_io`;
- a successful audit leaves exact entry sets, bytes, modification timestamps, and read-only permission state unchanged beneath both registry roots.

Exercise the production method. Do not test only helper functions or source strings.

## Relevant components

- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/installation_recovery_audit_tests.rs`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/installation_recovery.rs`
- `tethers-0.1/host-rust/src/installation_recovery_destination_tests.rs`
- `tethers-0.1/host-rust/src/installation_recovery_observation_tests.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `InstalledPlugRegistry`
- `InstalledPlugRecord`
- `InstallationPublicationIntent`
- `InstalledPlugRegistry::load_all`
- `require_existing_recovery_root`
- `reject_reparse`

`installation_publication_intent.rs`, `installation_recovery.rs`, and the accepted J24K3c1-c3 modules are reference-only and are not permitted production edit targets.

## Frozen decisions and invariants

- J24K3c4 is crate-private and read-only.
- A supplied intent is validated before any root, store, or filesystem access.
- Every entry in the direct `plug-` namespace must be accounted for globally.
- Installed records account only for exact canonical `plug-<installed_id>` paths.
- One current intent accounts only for its one exact destination.
- Intent-only destination content remains J24K3c2's responsibility.
- Current authority and evidence freshness remain J24K3c3's responsibility.
- Staging observation and recovery classification remain J24K3c1 and J24K3b responsibilities.
- Non-final direct entries are not recursively inspected or mutated.
- No orphan is adopted, deleted, renamed, repaired, or converted into a record.
- Existing public installation and `load_all()` behaviour remain unchanged.
- No public API, dependency, Cargo configuration, Cargo.lock, request, evidence schema, CLI, packaging, release, enablement, operational-scope, or OCaml change is permitted.
- Recovery mutation, cleanup, record publication, intent removal, lock integration, planner, and executor wiring remain later work.

## Acceptance criteria

1. One crate-private method accepts only an optional validated intent.
2. Supplied-intent validation is the first operation.
3. Both already-opened roots must remain ordinary safe directories.
4. All installed records and their tracked destinations remain valid through accepted installed-state validation.
5. Every record destination is exactly canonical `plug-<installed_id>`.
6. The current intent authorizes no destination other than its exact path.
7. A matching record at the intent path must equal the complete precomputed intent record.
8. Every direct `plug-*` entry is canonical, ordinary, safe, and accounted for.
9. Any unexplained or malformed final namespace entry returns `installation_destination_untracked`.
10. Contradictory tracked state returns `installation_recovery_conflict`.
11. Explicit reparse state remains `unsafe_store_path`.
12. Root and direct-audit access failure returns `installation_recovery_io` without detail.
13. Missing intent destination is permitted; missing record-backed destination is not.
14. Staging and unrelated non-final entries are untouched and not recursively audited.
15. Audit performs no mutation and preserves bytes, entries, timestamps, and permissions.
16. Direct tests exercise the production audit seam.
17. Focused Nextest passes with zero retries and all `j24k3c4` tests pass.
18. J24K3c3, J24K3c2, J24K3c1, J24K3b, J24K3a, J24K2, J24J, and M3 lifecycle regressions remain green.
19. Full `just verify` and the task packet checker pass.
20. Cargo.lock remains byte-identical and only permitted files change.
21. The task packet and worker note record exact commands, counts, checkpoint SHA, discoveries, risks, and final remote tip.

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
  -E 'test(j24k3c4)'

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k3c4 `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k3c3 `
  --locked

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
git log --oneline --decorate -12
```

Cargo.lock must remain:

```text
D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB
```

If the documented `m3_lifecycle` Windows handle-contention failure occurs, identify the exact known failure, rerun that test serially, and require it to pass. Do not classify a different failure as pre-existing.

## Forbidden changes

- No edit to the frozen architecture.
- No edit to `installation_publication_intent.rs`, `installation_recovery.rs`, `installation_recovery_evidence.rs`, `installation_execution.rs`, `m3_store.rs`, or accepted destination/observation production semantics.
- No public `load_all()` behaviour change.
- No intent-destination content verification beyond accepted installed-record validation.
- No recursive scan of unrelated install-root entries.
- No staging classification or cleanup.
- No global repair, adoption, deletion, rename, record creation, intent removal, lock, planner, or executor wiring.
- No public API, schema, dependency, Cargo configuration, Cargo.lock, CLI, packaging, release, enablement, operational-scope, or OCaml change.
- No unrelated refactor or broad test framework.

Permitted files:

- `tethers-0.1/host-rust/src/installed.rs` only for the narrow crate-private audit and private helpers;
- `tethers-0.1/host-rust/src/installation_recovery_audit_tests.rs` new;
- `tethers-0.1/host-rust/src/lib.rs` only to register the new private test module;
- `docs/CURRENT_CLINE_TASK.md`;
- `docs/worker-notes/2026-08-05-j24k3c4-installed-root-audit.md`.

## Stop conditions

Stop as `BLOCKED` only if global final-destination accounting requires changing an accepted public type, record schema, intent schema, dependency, Cargo.lock, public `load_all()` semantics, or production filesystem mutation; or if full verification still fails after one evidence-led correction.

Do not stop for failed LSP, a stale local ref, constructing validly redigested installed-record fixtures, or the documented intermittent Windows handle-contention fixture.

## Expected pre-existing changes

None. The branch is expected to be clean at handoff. The worker-note scaffold commit named by `Base commit` changes only the new worker note; this task-packet commit changes only `docs/CURRENT_CLINE_TASK.md`.
