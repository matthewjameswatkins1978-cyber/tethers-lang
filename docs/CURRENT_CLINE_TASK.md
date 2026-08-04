# Current Implementation Task

Control contract: `1`
Task: `J24K3b - Pure publication recovery-state classifier`
Owner: `OpenCode`
Status: `READY`
Task colour: `Red`
Route: `OpenCode using DeepSeek Pro V4 for one bounded security-sensitive Rust state-machine package; Lucy performs independent review and routine safe merge`
Base branch: `opencode/j24k3b-recovery-classifier`
Base commit: `02ff3a9f6475d6ee243ab8fe662a4d3bb74d1b73`
Implementation branch: `opencode/j24k3b-recovery-classifier`
Worker note: `docs/worker-notes/2026-08-04-j24k3b-recovery-classifier.md`
Implementation blueprint: `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
Rust toolchain: `1.97.1`
Accepted main: `9402a9f5d312c3523cc81fd2682431056fe55d97`

## Objective

Implement only J24K3b: one private, pure, typed classifier for the validated-current-intent portion of the frozen J24K recovery matrix.

The classifier receives:

- one validated `InstallationPublicationIntent`;
- whether its exact staging directory has already been observed;
- whether its exact final destination has already been observed;
- an optional installed record already selected for the intent transaction.

It returns one typed recovery disposition naming the next required recovery path, or fails closed when the observed state is contradictory.

J24K3b performs no filesystem access and no mutation. It does not discover paths, load stores, revalidate evidence, verify destination bytes, audit the installed root, clean staging, publish records, remove intent, acquire a lock, plan installation, or wire the executor.

## Relevant background and existing behaviour

Accepted `main` is exactly:

```text
9402a9f5d312c3523cc81fd2682431056fe55d97
```

J24K3a is accepted on main and provides a private, strictly validated `InstallationPublicationIntent` containing the exact precomputed `InstalledPlugRecord`.

The frozen J24K recovery matrix says:

```text
intent + no staging + no destination + no record
    -> remove intent only

intent + staging + no destination + no record
    -> remove staging, then remove intent after successful cleanup

intent + no staging + destination + no record
    -> fully revalidate and publish the exact precomputed record, then remove intent

intent + no staging + destination + matching record
    -> verify destination and record, then remove completed intent

intent + any staging + no destination + record
    -> fail closed

intent + staging + destination + any record state
    -> fail closed

intent + no staging + destination + mismatched record
    -> fail closed
```

Malformed or torn intent is already rejected by J24K3a before classification.

Absence of a current intent and untracked final-directory detection require installed-root observation and global audit. They are deliberately deferred to later J24K3 work and are not represented as a successful J24K3b classification.

The output is a required recovery route, not proof that evidence or destination verification has succeeded.

## Required behaviour

### 1. Private module and seam

Add:

```text
tethers-0.1/host-rust/src/installation_recovery.rs
tethers-0.1/host-rust/src/installation_recovery_tests.rs
```

Register privately in `lib.rs`:

```rust
mod installation_recovery;
#[cfg(test)]
mod installation_recovery_tests;
```

Do not add a public re-export, CLI seam, feature flag, test bypass, or global state.

### 2. Typed pure observation

Add a crate-private observation structurally equivalent to:

```rust
#[derive(Debug, Clone, Copy)]
pub(crate) struct InstallationRecoveryObservation<'a> {
    pub intent: &'a InstallationPublicationIntent,
    pub staging_present: bool,
    pub destination_present: bool,
    pub installed_record: Option<&'a InstalledPlugRecord>,
}
```

The exact field names may be narrowed for clarity, but call sites must use named facts rather than an opaque tuple or bit mask.

The observation contains already-observed facts only. It contains no filesystem paths, store handles, clocks, authorities, candidate data, mutable references, or callbacks.

### 3. Typed recovery dispositions

Add a crate-private enum structurally equivalent to:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InstallationRecoveryDisposition {
    RemoveIntentOnly,
    RemoveStagingThenIntent,
    RevalidateDestinationThenPublishRecord,
    VerifyCompletedPublicationThenRemoveIntent,
}
```

Names may be improved only if they retain these exact semantics.

No successful disposition may imply that cleanup, evidence revalidation, destination verification, record publication, or intent removal has already occurred.

### 4. Pure classifier

Provide a crate-private function structurally equivalent to:

```rust
pub(crate) fn classify_installation_recovery(
    observation: InstallationRecoveryObservation<'_>,
) -> Result<InstallationRecoveryDisposition>;
```

It must:

1. validate the publication intent first;
2. return `installation_intent_invalid` if the intent is invalid;
3. validate a present installed record;
4. treat an invalid installed record as `installation_recovery_conflict`;
5. treat a present record as matching only when it is exactly equal to `intent.installed_record` after validation;
6. return the exact successful disposition for the four legitimate rows;
7. return `installation_recovery_conflict` for every other combination;
8. perform no I/O, mutation, clock read, UUID generation, hashing beyond validation already required by the supplied records, or nondeterministic work.

Classification order must not accidentally allow a broad row to swallow a contradictory row. In particular:

- staging plus destination always conflicts;
- any present record without a destination conflicts;
- a destination plus a mismatched or invalid record conflicts;
- only destination plus the exact matching record reaches the completed-publication disposition.

### 5. Stable safe errors

Use only:

```text
installation_intent_invalid: installation publication intent is invalid
installation_recovery_conflict: installation recovery state conflicts with publication intent
```

Do not leak lower-layer installed-record validation codes, record fields, package-controlled text, or raw data.

J24K3b does not emit `installation_intent_evidence_stale`, `installation_destination_untracked`, or I/O errors because it performs no evidence revalidation, global audit, or I/O.

### 6. Direct matrix tests

Tests must exercise the production classifier directly and use the `j24k3b` prefix.

Prove all four legitimate rows:

1. intent only -> `RemoveIntentOnly`;
2. intent plus staging only -> `RemoveStagingThenIntent`;
3. intent plus destination only -> `RevalidateDestinationThenPublishRecord`;
4. intent plus destination plus exact matching record -> `VerifyCompletedPublicationThenRemoveIntent`.

Prove fail-closed contradictory rows at minimum:

5. record without destination and without staging;
6. staging plus record without destination;
7. staging plus destination without record;
8. staging plus destination plus matching record;
9. destination plus a different valid installed record;
10. destination plus an invalid installed record.

Also prove:

11. invalid intent is reported as `installation_intent_invalid` before state classification;
12. a valid but unequal installed record with the same installed ID still conflicts, proving equality is not ID-only or digest-field-only;
13. classification does not alter the supplied intent or record;
14. repeated classification of the same observation is deterministic;
15. no successful disposition exists for absence of intent.

Do not use source-string inspection as a substitute for behavioural tests.

## Relevant components

- `tethers-0.1/host-rust/src/installation_recovery.rs`
- `tethers-0.1/host-rust/src/installation_recovery_tests.rs`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `InstallationPublicationIntent::validate`
- `InstalledPlugRecord::validate`

`installation_publication_intent.rs` and `installed.rs` are accepted references, not permitted edit targets.

## Frozen decisions and invariants

- J24K3b classifies only the matrix rows where one validated current intent is present.
- Intent absence is represented by not calling this classifier, not by a successful `NoIntent` disposition.
- Untracked final-directory detection belongs to later installed-root audit work.
- Staging and destination presence are already-observed exact transaction facts; J24K3b does not discover or verify paths.
- A matching record means validated exact equality with the complete embedded precomputed record.
- Record ID, record digest, candidate ID, or destination alone are insufficient to establish a match.
- The classifier returns a required route, not authority to mutate.
- Every non-frozen combination fails closed.
- No filesystem access, evidence access, global audit, lock, mutation, executor integration, or public API belongs to J24K3b.
- No dependency, Cargo configuration, Cargo.lock, CLI, prompt, output, enablement, operational-scope, packaging, release, or OCaml change is permitted.

## Acceptance criteria

1. The recovery module and all of its seams are crate-private.
2. The observation uses named typed facts and contains no path or mutable execution capability.
3. The disposition enum has exactly the four legitimate validated-intent recovery routes.
4. The classifier validates intent before classifying state.
5. Invalid intent maps to the stable `installation_intent_invalid` contract.
6. Present installed records are validated and lower-layer errors do not escape.
7. Matching requires exact full-record equality with the embedded precomputed record.
8. Each of the four legitimate matrix rows returns the exact required disposition.
9. Every contradictory matrix row fails as `installation_recovery_conflict`.
10. Staging plus destination cannot be classified successfully under any record state.
11. Record without destination cannot be classified successfully under any staging state.
12. The classifier performs no I/O or mutation and is deterministic.
13. Tests directly exercise the production seam and cover all required legitimate and contradictory cases.
14. Focused Nextest runs with zero retries and all new `j24k3b` tests pass.
15. J24K3a, J24K2, J24J, and representative M3 regressions remain green.
16. Full verification passes with at least the accepted 991-lib-test baseline plus new tests.
17. Cargo.lock remains byte-identical and only permitted files change.
18. The task packet and worker note are updated with exact commands, counts, discoveries, checkpoint SHA, and final remote tip.

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
git log --oneline --decorate -6
```

The focused Nextest expression may be adjusted once only if discovery reports a different exact test name. Record exact run, pass, skip, and retry counts.

OpenCode LSP is not a gate. Do not spend task time diagnosing empty LSP results.

Cargo.lock must remain:

```text
D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB
```

No full-verification failure is acceptable after `$PSHOME` is prepended to PATH. A pre-existing Windows handle-contention failure must be identified precisely, rerun serially, and must pass before handoff; do not silently dismiss it.

## Forbidden changes

- No edit to `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`.
- No edit to `installation_publication_intent.rs`, `installed.rs`, `m3_store.rs`, or `installation_execution.rs`.
- No intent-store change, filesystem observation, directory enumeration, path derivation, reparse check, installed-record lookup, or store opening.
- No evidence revalidation, candidate lookup, trust lookup, launch-profile lookup, conformance lookup, approval lookup, or current-suite check.
- No destination file-set, length, hash, read-only, permission, or path-safety verification.
- No installed-root audit or untracked-final detection.
- No staging cleanup, intent removal, destination deletion, record publication, adoption, rollback, repair, or any other mutation.
- No lock acquisition or assertion, executor context field, action wiring, planner change, or replacement of `installation_publication_deferred`.
- No successful `NoIntent`, `NothingToDo`, `UntrackedDestination`, `EvidenceStale`, or generic `Conflict` disposition that broadens the exact matrix.
- No public module, public API, serialization schema, dependency, Cargo configuration, Cargo.lock, CLI, prompt, terminal styling, enablement, operational-scope, packaging, release, or OCaml change.
- No files outside the permitted set.

Permitted files:

- `tethers-0.1/host-rust/src/installation_recovery.rs`;
- `tethers-0.1/host-rust/src/installation_recovery_tests.rs`;
- `tethers-0.1/host-rust/src/lib.rs`;
- `docs/CURRENT_CLINE_TASK.md`;
- `docs/worker-notes/2026-08-04-j24k3b-recovery-classifier.md`.

## Stop conditions

Stop as `BLOCKED` only if:

- exact classification requires filesystem observation, installed-root audit, evidence revalidation, or mutation;
- the accepted intent or installed-record types cannot be referenced privately without changing their accepted modules;
- exact full-record equality cannot be used without altering an accepted schema;
- implementation requires a public API, dependency, Cargo.lock change, executor integration, or an out-of-scope file;
- required verification still fails after one evidence-led correction.

Do not stop for failed LSP, one ineffective Nextest filter, a stale local branch ref, or a failed broad text replacement. Reread the current file and make one smaller evidence-led correction.

## Expected pre-existing changes

None. The branch is expected to be clean at handoff. The documentation scaffold commit named by `Base commit` adds only the worker note; the task-packet commit after it changes only `docs/CURRENT_CLINE_TASK.md`.
