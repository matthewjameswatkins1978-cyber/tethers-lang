# Current Implementation Task

Control contract: `1`
Task: `J24K3c3 - Exact recovery evidence-chain revalidator`
Owner: `OpenCode`
Status: `READY`
Task colour: `Red`
Route: `OpenCode using Kimi K2.7Code for a second measured, bounded repository-reading and security-sensitive Rust authority-chain package; Lucy performs independent review and routine safe merge`
Base branch: `opencode/j24k3c3-evidence-revalidator`
Base commit: `374cb57ba50e685e3fe8716ecd6f2166a6f6e9b5`
Implementation branch: `opencode/j24k3c3-evidence-revalidator`
Worker note: `docs/worker-notes/2026-08-05-j24k3c3-evidence-revalidator.md`
Implementation blueprint: `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
Rust toolchain: `1.97.1`
Accepted main: `6cbcbaf8bfa9c67f274b503061187ae51a08b080`

## Objective

Implement only J24K3c3: one crate-private, read-only revalidator that proves the complete evidence chain behind one validated `InstallationPublicationIntent` is still current and exactly matches the current typed `InstallationRequest`.

The revalidator must prove, in order:

- the intent is valid before any store access;
- the typed request still represents exact-candidate trust, explicit non-isolated supervised conformance consent, disabled installation, and the same candidate ID as the intent;
- exactly one current candidate exists and its quarantine bytes are revalidated through the accepted candidate boundary;
- the exact-candidate trust record still exists and binds that candidate;
- reconstructed `PackageTrustEvidence` is exactly the intent record's trust evidence and remains current through `ExactCandidateTrustAuthority`;
- one exact launch profile remains pinned to the candidate;
- one exact passed conformance record remains pinned to the candidate, trust, launch profile, and current conformance suite;
- one exact installation approval remains fully pinned to the candidate, reconstructed trust, launch profile, conformance, and re-reviewed capability manifests;
- the precomputed `InstalledPlugRecord` remains an exact derivation of that complete chain.

This package does not verify destination files, audit unrelated installed roots, classify recovery, mutate state, clean staging, publish a record, remove an intent, acquire a lock, or wire the executor.

## Relevant background and existing behaviour

Accepted `main` is exactly:

```text
6cbcbaf8bfa9c67f274b503061187ae51a08b080
```

J24K3a supplies a strict publication intent containing one complete precomputed installed record. J24K3b classifies already-observed recovery facts. J24K3c1 observes the exact staging, destination, and installed-record paths. J24K3c2 verifies the exact final destination's file set, lengths, hashes, permissions, and path safety.

The frozen J24K architecture requires recovery to revalidate the request and complete authority chain before publishing a record for an already-present destination. A structurally valid intent is not sufficient: its host-owned digests can still be internally consistent while naming stale or incorrectly repinned evidence.

Existing accepted seams already provide the authority:

- `CandidateRegistry::load_all` and `revalidate_candidate` for exact candidate and quarantine bytes;
- `ExactCandidateTrustStore::find` and `ExactCandidateTrustRecord::require_for_candidate`;
- `PackageTrustEvidence::exact_candidate`;
- `ExactCandidateTrustAuthority::revalidate_current`;
- `LaunchProfileEvidence::require_for_candidate`;
- `ConformanceEvidence::require_current` with `current_suite_digest()`;
- `InstallationApprovalRecord::validate` and the existing manifest-review machinery in `installed.rs`;
- `InstalledPlugRecord::validate` and the accepted physical/capability fields.

Do not create a parallel trust or validation model. Compose and narrowly extend these accepted seams.

## Required behaviour

1. Add one narrow crate-private recovery evidence context and entry point.

Add a private module such as `installation_recovery_evidence.rs` with a context structurally equivalent to:

```rust
pub(crate) struct InstallationRecoveryEvidenceContext<'a> {
    pub quarantine_root: &'a Path,
    pub candidates: &'a CandidateRegistry,
    pub exact_trust: &'a ExactCandidateTrustStore,
    pub launch_profiles: &'a LaunchProfileEvidenceStore,
    pub conformance: &'a ConformanceEvidenceStore,
    pub approvals: &'a InstallationApprovalStore,
}
```

Provide one seam structurally equivalent to:

```rust
pub(crate) fn revalidate_installation_recovery_evidence(
    request: &InstallationRequest,
    intent: &InstallationPublicationIntent,
    context: &InstallationRecoveryEvidenceContext<'_>,
) -> Result<()>;
```

The context contains read-only references only. It contains no installed registry, destination path, lock, callback, clock injection, cleanup capability, publisher/developer fallback authority, or mutation seam.

2. Validate the intent and typed request before reading stores.

`intent.validate()` must be the first operation.

Then require the typed request to contain exactly the accepted fixed values:

- schema `tethers.plug-install/1`;
- candidate ID equal to `intent.candidate_id` and `intent.installed_record.source_candidate_id`;
- `InstallationTrustScope::ExactCandidate`;
- `allow_non_isolated_supervised_execution == true`;
- `InstallationTargetState::Disabled`.

A caller can construct the public typed request fields directly, so do not assume successful parsing happened earlier.

3. Load and revalidate exactly one candidate.

Use `CandidateRegistry::load_all()` and require exactly one record whose candidate ID equals the request and intent candidate ID. Do not select by package release or semantic digest.

Require the candidate's package ID, package version, semantic digest, raw archive digest, provider identity, platform, architecture, launch declaration, physical evidence, and capability evidence to match the precomputed installed record where those fields are represented.

Call the accepted `revalidate_candidate(candidate, context.quarantine_root)` boundary after loading it. Recovery must not rely only on historical candidate JSON.

4. Reconstruct and revalidate current exact-candidate trust.

Load the exact trust record through `ExactCandidateTrustStore::find(candidate_id)` and require it for the candidate.

Reconstruct `PackageTrustEvidence::exact_candidate(&record)`, require it for the candidate, and require exact equality with `intent.installed_record.trust_evidence`.

Construct `ExactCandidateTrustAuthority` from the supplied exact-trust store and call `revalidate_current` with the candidate and reconstructed evidence. Do not add publisher/developer fallback, optional authority, global authority, or cached authority state.

5. Load the exact approval, conformance, and launch-profile records.

Load the installation approval by `intent.installed_record.installation_approval_id` and require its digest to equal `installation_approval_digest`.

Load conformance by `intent.installed_record.conformance_evidence_id` and require its digest to equal `conformance_evidence_digest`.

Require the approval's conformance ID and digest to equal both the loaded conformance and the installed record.

Require the approval and conformance to pin the same launch-profile evidence digest. Load exactly that launch profile from `LaunchProfileEvidenceStore::load_all()`.

Missing evidence, duplicate/malformed store state, ID mismatch, digest mismatch, or disagreement between the approval and conformance pins must fail closed.

6. Revalidate launch and conformance freshness.

Call `launch.require_for_candidate(candidate)`.

Call `conformance.require_current(candidate, reconstructed_trust, launch, &current_suite_digest()?)`.

This must require a passed disposition and the current suite digest. Invalidated, failed, interrupted, stale-suite, stale-trust, stale-launch, payload, or capability drift is not recoverable here.

7. Revalidate the complete installation approval chain.

Add one narrow crate-private approval method or equivalent helper in `installed.rs` that:

- calls `approval.validate()`;
- requires candidate ID, package ID/version, semantic and raw archive digests, source size, payload evidence, provider identity/version, launch path/arguments/working directory, launch-profile label/limitation/digest, trust evidence, conformance ID/digest, and all corresponding pins to equal the loaded current evidence;
- reconstructs the accepted reviewed capabilities from the revalidated quarantine directory using the existing manifest-review machinery;
- requires exact equality with `approval.reviewed_capabilities`.

Do not weaken or duplicate manifest verification. Reuse the existing `reviewed_capabilities` implementation or a narrowly extracted equivalent within `installed.rs`.

8. Revalidate the precomputed installed record as an exact chain product.

Add one narrow crate-private installed-record method or equivalent helper that calls `record.validate()` and requires exact equality with the current candidate/evidence chain for:

- installed/source candidate identity and destination identity already pinned by the intent;
- package ID/version, semantic and raw archive digests;
- `plug_json`, payloads, signature files, and capability manifests;
- reconstructed trust evidence;
- installation approval ID/digest;
- conformance evidence ID/digest;
- provider ID/version, launch path/arguments/working directory, launch-profile label;
- platform and architecture;
- exact disabled bindings derived from every candidate capability, with no missing, extra, reordered, enabled, or repinned binding.

Do not refresh `created_unix_ms`, installed ID, destination path, or record digest. Recovery validates and later publishes the precomputed record unchanged.

9. Use stable recovery errors without leaking details.

Use only these recovery-facing errors:

```text
installation_intent_invalid: installation publication intent is invalid
installation_intent_evidence_stale: installation publication evidence is no longer current
installation_recovery_io: installation recovery state could not be observed
```

Preserving explicit accepted `unsafe_store_path` is allowed. Translate the candidate layer's explicit unsafe-destination/reparse refusal to `unsafe_store_path` rather than treating it as stale evidence.

Map genuine candidate/store read or access failures to `installation_recovery_io`. Map missing, malformed, duplicate, contradictory, invalid, stale, or mismatched evidence to `installation_intent_evidence_stale`.

Do not expose filesystem paths, package-controlled strings, raw JSON, lower-layer messages, or OS diagnostics.

10. Add direct production tests and complete full verification.

Add a private test module whose test names are prefixed `j24k3c3`.

Directly prove at minimum:

- one complete valid current evidence chain passes;
- invalid intent is rejected before request or store state can influence the result;
- each invalid typed-request field fails stale, including candidate mismatch;
- missing and tampered candidate evidence fail stale;
- quarantined candidate byte mutation fails stale;
- missing, changed, or differently authorised exact-candidate trust fails stale;
- reconstructed trust evidence must exactly equal the intent record's trust evidence;
- missing, mismatched, or candidate-stale launch profile fails stale;
- missing, mismatched, non-passed, or old-suite conformance fails stale;
- missing or digest-mismatched approval fails stale;
- approval trust, launch, conformance, candidate, provider, payload, or reviewed-capability drift fails stale;
- installed-record package, physical evidence, capability, trust, approval, conformance, provider, launch, platform, architecture, or disabled-binding drift fails stale after recomputing its record and intent digests;
- unrelated valid evidence records do not satisfy or replace the exact pinned records;
- explicit unsafe candidate/store path state remains unsafe rather than stale or absent;
- genuine inaccessible/read failure maps to `installation_recovery_io` where an accepted deterministic fixture is available;
- successful revalidation leaves every store entry, quarantine byte, timestamp, and permission unchanged.

Exercise the production entry point. Do not test only private comparison helpers or source strings.

## Relevant components

- `tethers-0.1/host-rust/src/installation_recovery_evidence.rs`
- `tethers-0.1/host-rust/src/installation_recovery_evidence_tests.rs`
- `tethers-0.1/host-rust/src/installation_publication_intent.rs`
- `tethers-0.1/host-rust/src/installation_request.rs`
- `tethers-0.1/host-rust/src/candidate.rs`
- `tethers-0.1/host-rust/src/installation_trust.rs`
- `tethers-0.1/host-rust/src/current_trust.rs`
- `tethers-0.1/host-rust/src/launch_profile.rs`
- `tethers-0.1/host-rust/src/conformance.rs`
- `tethers-0.1/host-rust/src/installed.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `InstallationRequest`
- `InstallationPublicationIntent`
- `CandidateRegistry`, `CandidateRecord`, `revalidate_candidate`
- `ExactCandidateTrustStore`, `ExactCandidateTrustAuthority`, `CurrentTrustAuthority`
- `PackageTrustEvidence::exact_candidate`
- `LaunchProfileEvidenceStore`, `LaunchProfileEvidence::require_for_candidate`
- `ConformanceEvidenceStore`, `ConformanceEvidence::require_current`, `current_suite_digest`
- `InstallationApprovalStore`, `InstallationApprovalRecord`
- `InstalledPlugRecord`, `DisabledBindingRecord`

The accepted evidence/store modules are references. Only the narrow extension points listed below may be edited.

## Frozen decisions and invariants

- J24K3c3 is crate-private and read-only.
- Intent validation is the first operation.
- The current typed request is mandatory authority and must exactly match the intent candidate.
- Candidate bytes are revalidated through the accepted quarantine boundary.
- Exact-candidate trust has no publisher/developer fallback.
- Historical package-trust evidence is not current authority.
- Launch profile, conformance, approval, and installed-record pins must form one exact chain.
- Conformance must be passed against the current suite.
- Reviewed capabilities must be reconstructed from current revalidated manifests.
- The precomputed installed record is validated unchanged; no identity or timestamp is recomputed.
- Destination verification remains J24K3c2 and is not repeated here.
- Global installed-root audit, recovery classification, mutation, cleanup, publication, intent removal, lock integration, planner, and executor wiring remain later work.
- Existing public APIs and ordinary installation behaviour remain unchanged.
- No dependency, Cargo configuration, Cargo.lock, CLI, prompt, output, enablement, operational-scope, packaging, release, or OCaml change is permitted.

## Acceptance criteria

1. One crate-private read-only entry point accepts only the typed request, validated intent, and narrow evidence context.
2. Intent validation precedes request and store access.
3. Every fixed request field and candidate identity is checked explicitly.
4. Exactly one candidate is loaded by candidate ID and its quarantine bytes are revalidated.
5. Current exact-candidate trust is reconstructed and revalidated without fallback.
6. Reconstructed trust evidence exactly equals the intent installed record's trust evidence.
7. Approval, conformance, and launch evidence are loaded only through exact pinned IDs/digests.
8. Launch evidence remains bound to the exact candidate.
9. Conformance remains passed and current for candidate, trust, launch, and current suite.
10. Approval pins and reviewed capabilities remain exact and current.
11. The precomputed installed record exactly matches candidate, trust, launch, conformance, approval, platform, architecture, capabilities, files, and disabled bindings.
12. Missing, malformed, duplicate, contradictory, stale, or mismatched evidence fails closed.
13. Genuine read/access failure is not reported as stale or absent.
14. Stable errors contain no unsafe lower-layer detail.
15. No destination or unrelated installed-root state is inspected.
16. Revalidation performs no mutation.
17. Direct tests exercise the production revalidator.
18. Focused Nextest passes with zero retries and all `j24k3c3` tests pass.
19. J24K3c2, J24K3c1, J24K3b, J24K3a, J24K2, J24I, J24H, J24J, and representative M3 regressions remain green.
20. Full `just verify` and the task packet checker pass.
21. Cargo.lock remains byte-identical and only permitted files change.
22. The task packet and worker note contain exact commands, counts, checkpoint SHA, discoveries, risks, and final remote tip.

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
  -E 'test(j24k3c3)'

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
  --test j24i_exact_candidate_installation_trust `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --test j24h_installation_evidence_access `
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
- No edit to `installation_publication_intent.rs`, `installation_request.rs`, `candidate.rs`, `installation_trust.rs`, `current_trust.rs`, `launch_profile.rs`, `conformance.rs`, `m3_store.rs`, `installation_recovery.rs`, `installation_execution.rs`, or destination-verifier production semantics.
- No publisher/developer trust fallback, optional authority, global authority, cached authority, or new trust schema.
- No destination file verification, global installed-root audit, recovery classification, cleanup, deletion, publication, repair, adoption, intent removal, lock, planner, or executor wiring.
- No mutation in production code.
- No public API or broad store/root accessor.
- No dependency, Cargo configuration, Cargo.lock, CLI, packaging, release, enablement, operational-scope, or OCaml change.
- No unrelated refactor.
- No files outside the permitted set.

Permitted files:

- `tethers-0.1/host-rust/src/installation_recovery_evidence.rs` new;
- `tethers-0.1/host-rust/src/installation_recovery_evidence_tests.rs` new;
- `tethers-0.1/host-rust/src/installed.rs` only for narrow crate-private approval and installed-record chain validation helpers;
- `tethers-0.1/host-rust/src/lib.rs` only to register the new private production and test modules;
- `docs/CURRENT_CLINE_TASK.md`;
- `docs/worker-notes/2026-08-05-j24k3c3-evidence-revalidator.md`.

## Stop conditions

Stop as `BLOCKED` only if complete read-only evidence revalidation requires changing an accepted evidence schema, public API, dependency, Cargo.lock, trust policy, intent/classifier type, or production filesystem mutation; or if full verification still fails after one evidence-led correction.

Do not stop for failed LSP, a stale local ref, one ineffective Nextest filter, or the need to build focused host-owned test fixtures from accepted records.

## Expected pre-existing changes

None. The branch is expected to be clean at handoff. The worker-note scaffold commit named by `Base commit` changes only the new worker note; this task-packet commit changes only `docs/CURRENT_CLINE_TASK.md`.
