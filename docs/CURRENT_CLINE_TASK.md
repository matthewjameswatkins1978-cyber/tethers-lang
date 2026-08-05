# Current Implementation Task

Control contract: `1`
Task: `J24K3c3 correction - exact trust equality and evidence hygiene`
Owner: `OpenCode`
Status: `IN_PROGRESS`
Task colour: `Red`
Route: `OpenCode using Kimi K2.7Code for one bounded correction to its J24K3c3 authority-chain package; Lucy performs independent review and routine safe merge`
Base branch: `opencode/j24k3c3-evidence-revalidator`
Base commit: `7b148070a04a3af30ffe0165c35ea56e499b36a5`
Implementation branch: `opencode/j24k3c3-evidence-revalidator`
Worker note: `docs/worker-notes/2026-08-05-j24k3c3-correction.md`
Implementation blueprint: `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
Rust toolchain: `1.97.1`
Accepted main: `6cbcbaf8bfa9c67f274b503061187ae51a08b080`
Reviewed OpenCode tip: `88a911772aef7511262a365cccbcbdf3b0ad4ae9`
Reviewed implementation checkpoint: `727a20944270a5c71484f8c1728c339d0d7f1dbf`

## Objective

Correct three narrow independent-review findings in the otherwise sound J24K3c3 read-only recovery evidence-chain revalidator.

Do not redesign the package. Preserve its context, ordering, store selection, candidate-byte revalidation, exact-candidate authority, launch/conformance/approval chain, installed-record derivation, stable errors, read-only boundary, and all later-work exclusions.

## Independent-review findings

### 1. The frozen packet requires literal trust-object equality

The current implementation compares only `PackageTrustEvidence.evidence_digest` at three recovery boundaries:

- reconstructed trust versus `intent.installed_record.trust_evidence` in `revalidate_trust`;
- approval trust versus reconstructed trust in `InstallationApprovalRecord::require_for_recovery`;
- installed-record trust versus reconstructed trust in `InstalledPlugRecord::require_for_recovery`.

SHA-256 coverage remains mandatory, but the frozen J24K3c3 contract explicitly requires exact `PackageTrustEvidence` equality. Enforce full value equality at all three boundaries. Do not remove validation or digest checks performed by the accepted types.

### 2. Recovery-facing error translation must not carry lower-layer messages

`map_candidate_error` currently constructs `unsafe_store_path` using the candidate layer's `error.message`. Preserve the explicit unsafe-path code, but replace the copied message with one fixed recovery-owned safe message. Do not expose the candidate message, a filesystem path, package text, JSON, or OS diagnostics.

Also map failure from `current_suite_digest()` into an allowed stable recovery-facing error rather than allowing a lower `record_invalid` or canonicalisation error to escape. No clock, callback, fault-injection seam, or public API may be added.

The only ordinary recovery-facing errors remain:

```text
installation_intent_invalid: installation publication intent is invalid
installation_intent_evidence_stale: installation publication evidence is no longer current
installation_recovery_io: installation recovery state could not be observed
```

Explicit `unsafe_store_path` remains allowed with one fixed safe message.

### 3. The successful read-only test must prove the metadata claim it makes

`j24k3c3_success_leaves_stores_quarantine_and_permissions_unchanged` currently compares loaded store values and byte hashes, but does not compare modification timestamps or permission state.

Strengthen the production-entry-point test to snapshot and compare, for every relevant file beneath:

- candidate registry root;
- quarantine root;
- exact-trust store;
- launch-profile store;
- conformance store;
- approval store;

At minimum capture:

- normalized relative path and entry type;
- file bytes or SHA-256 digest;
- `modified()` timestamp;
- `permissions().readonly()`.

The snapshot must be taken immediately before and after one successful call to `revalidate_installation_recovery_evidence`. Do not alter the files to prepare the assertion. Directory entry sets must remain exact.

### 4. Test names and evidence must describe what they prove

The closed enums `InstallationTrustScope` and `InstallationTargetState` currently expose only `ExactCandidate` and `Disabled`. The tests named:

- `j24k3c3_non_exact_trust_scope_fails_stale`;
- `j24k3c3_non_disabled_target_fails_stale`;

do not construct invalid states and instead pass the valid chain.

Rename or replace them so the names honestly describe the closed accepted variants they prove. Do not add enum variants, unsafe representation tricks, deserialisation bypasses, public constructors, or architecture changes merely to manufacture impossible typed states. Record this closed-enum limitation accurately in the worker note and exact test count.

## Relevant background and existing behaviour

The J24K3c3 recovery evidence-chain revalidator is fully implemented and reviewed at `88a911772aef7511262a365cccbcbdf3b0ad4ae9`. Independent review identified three narrow findings:

1. Trust comparison at three recovery boundaries uses only `evidence_digest` rather than full `PackageTrustEvidence` equality as the frozen packet requires.
2. `map_candidate_error` copies the candidate layer's error message into the recovery-facing `unsafe_store_path` error, and `current_suite_digest()` failure is not mapped to a safe recovery error.
3. The successful read-only test compares logical values and hashes but not modification timestamps or permissions, and two closed-enum test names claim invalid states the type system prevents.

The existing accepted stores, candidate-byte revalidation, trust-authority, launch, conformance, approval, and installed-record checks remain sound.

## Relevant components

- `tethers-0.1/host-rust/src/installation_recovery_evidence.rs` — entry point, trust revalidation, error mapping.
- `tethers-0.1/host-rust/src/installation_recovery_evidence_tests.rs` — 44 focused tests including the metadata-preservation test and the misleading closed-enum tests.
- `tethers-0.1/host-rust/src/installed.rs` — `InstallationApprovalRecord::require_for_recovery`, `InstalledPlugRecord::require_for_recovery`.
- `docs/CURRENT_CLINE_TASK.md` — this packet.
- `docs/worker-notes/2026-08-05-j24k3c3-correction.md` — correction worker note.

## Frozen decisions and invariants

1. SHA-256 coverage remains mandatory, but exact `PackageTrustEvidence` object equality must be enforced at all three recovery boundaries.
2. Only the three stable recovery-facing error codes are permitted: `installation_intent_invalid`, `installation_intent_evidence_stale`, `installation_recovery_io`. Explicit `unsafe_store_path` is allowed with one fixed safe message.
3. `InstallationTrustScope` and `InstallationTargetState` are closed enums exposing only their accepted variants; no new variants, unsafe representation tricks, or deserialisation bypasses may be added.
4. No production mutation, no destination verification, no recovery classification or cleanup, no publication, no intent removal, no locking, no planner or executor wiring.
5. Cargo.lock must remain byte-identical.
6. The evidence-chain architecture, intent-first ordering, store selection, and authority policy are frozen.

## Required behaviour

1. Preserve `intent.validate()` as the first production operation.
2. Preserve every accepted request, candidate, trust-authority, launch, conformance, approval, and installed-record check.
3. Compare full `PackageTrustEvidence` values at all three recovery chain boundaries.
4. Preserve all accepted validation and cryptographic digest coverage.
5. Translate candidate unsafe state to `unsafe_store_path` with one fixed safe recovery-owned message.
6. Ensure `current_suite_digest()` cannot leak a non-recovery-facing error.
7. Strengthen the successful no-mutation test to compare bytes, entry sets, modification timestamps, and read-only permissions across all evidence roots.
8. Rename the two misleading closed-enum tests without changing accepted public types.
9. Retain every other J24K3c3 test and production semantic.
10. Perform no production mutation and no later recovery work.

## Acceptance criteria

1. `revalidate_trust` requires reconstructed trust to equal the complete intent-record trust value.
2. `InstallationApprovalRecord::require_for_recovery` requires complete approval trust equality.
3. `InstalledPlugRecord::require_for_recovery` requires complete installed-record trust equality.
4. Candidate unsafe translation copies no lower-layer message or detail.
5. Current-suite calculation returns only an allowed recovery-facing error on failure.
6. Existing intent-first ordering and all authority-chain checks remain unchanged.
7. The successful read-only test proves exact entry sets, bytes, modification timestamps, and read-only permission state across all relevant roots.
8. Closed-enum test names no longer claim to exercise impossible invalid variants.
9. All existing focused negative cases remain green.
10. Focused Nextest passes with zero retries.
11. J24K3c2, J24K3c1, J24K3b, J24K3a, J24K2, J24I, J24H, J24J, and M3 lifecycle regressions remain green.
12. Full `just verify` and the task packet checker pass, subject only to the already-documented intermittent Windows handle-contention rule.
13. Cargo.lock remains byte-identical and only permitted files change.
14. The worker note records exact corrected test counts, implementation checkpoint, verification, discoveries, remaining risks, and final remote tip.

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
git log --oneline --decorate -12
```

Cargo.lock must remain:

```text
D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB
```

If the already-documented `m3_lifecycle` Windows handle-contention failure occurs, identify the exact same failure, rerun that test serially, and require it to pass. Do not relabel a new failure as pre-existing.

## Forbidden changes

- No edit to the frozen architecture.
- No edit to accepted request, candidate, trust, launch-profile, conformance, store, intent, recovery classifier, execution, or destination-verifier modules beyond the already-permitted narrow `installed.rs` helpers.
- No enum variants, unsafe representation construction, parser bypass, public API, schema, dependency, Cargo configuration, Cargo.lock, CLI, packaging, release, enablement, operational-scope, or OCaml change.
- No destination verification, global audit, recovery classification, cleanup, publication, intent removal, lock, planner, or executor wiring.
- No production mutation.
- No unrelated refactor or broad test-fixture framework.

Permitted files:

- `tethers-0.1/host-rust/src/installation_recovery_evidence.rs`;
- `tethers-0.1/host-rust/src/installation_recovery_evidence_tests.rs`;
- `tethers-0.1/host-rust/src/installed.rs` only for the existing narrow recovery helpers;
- `docs/CURRENT_CLINE_TASK.md`;
- `docs/worker-notes/2026-08-05-j24k3c3-correction.md`.

## Stop conditions

Stop as `BLOCKED` only if literal trust equality, safe error translation, or metadata-preservation evidence requires changing an accepted public type, evidence schema, dependency, Cargo.lock, trust policy, or production mutation; or if full verification still fails after one evidence-led correction.

Do not stop for failed LSP, a stale local ref, renaming the two misleading tests, or the documented intermittent Windows handle-contention fixture.

## Expected pre-existing changes

The branch contains the complete reviewed J24K3c3 implementation and evidence at `88a911772aef7511262a365cccbcbdf3b0ad4ae9`, followed by the correction worker-note scaffold at the `Base commit`. No production correction has yet been applied.
