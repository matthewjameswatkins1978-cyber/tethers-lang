# Current Implementation Task

Control contract: `1`
Task: `J24K1 - Explicit current-trust authority foundation`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Red`
Route: `OpenCode using DeepSeek Pro V4 for bounded security-sensitive Rust refactoring; Lucy performs independent review and routine safe merge`
Base branch: `main`
Base commit: `db84c71dc92381921cdc05c62029a1899c13d7f2`
Implementation branch: `opencode/j24k1-current-trust-authority`
Worker note: `docs/worker-notes/2026-08-04-j24k1-current-trust-authority.md`
Implementation blueprint: `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`
Rust toolchain: `1.97.1`
Implementation checkpoint: `f82af8b3c5f0c0f3bf6e6bb0c7c955c8e71a44c0`

## Objective

Introduce the crate-private current-trust authority foundation required by J24K.

The change must allow existing supervised conformance, installation approval, and disabled installation internals to require one explicit current-trust authority while preserving the accepted signed-publisher and unsigned-developer public paths unchanged.

Add an exact-candidate authority implementation backed only by `ExactCandidateTrustStore`.

J24K1 does not add the installation lock, public executor, multi-step driver, publication intent, recovery logic, CLI, or enablement.

## Relevant background and existing behaviour

J24I added immutable exact-candidate installation trust and `PackageTrustEvidence::exact_candidate`.

J24J can reconcile that evidence and return the next legitimate installation action.

The current downstream mutation seams still call:

```rust
PackageTrustEvidence::revalidate_current(
    package_id,
    publisher_trust,
    developer_approvals,
    now_unix_ms,
)
```

That accepted legacy method deliberately refuses `TrustModeEvidence::ExactCandidate` with:

```text
trust_exact_candidate_authority_required
exact-candidate trust requires current installation-trust authority
```

The affected paths include:

- `PreparedSupervisedLaunch::revalidate_current_trust`;
- `PreparedSupervisedLaunch::launch_for_candidate`;
- `run_host_conformance`;
- `InstallationApprovalStore::approve`;
- `InstalledPlugRegistry::install_disabled`, including its final pre-publication revalidation.

J24K1 introduces the explicit authority seam without weakening or replacing accepted trust policy.

Accepted baseline:

```text
Rust             1.97.1
Cargo tests      926 passing minimum before new J24K1 tests
Nextest retries  0
Cargo.lock       D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB
```

## Required behaviour

1. Add a crate-private module:

```text
tethers-0.1/host-rust/src/current_trust.rs
```

2. Define a crate-private trait semantically equivalent to:

```rust
pub(crate) trait CurrentTrustAuthority {
    fn revalidate_current(
        &self,
        candidate: &CandidateRecord,
        evidence: &PackageTrustEvidence,
        now_unix_ms: u64,
    ) -> Result<()>;
}
```

A generic `A: CurrentTrustAuthority + ?Sized` form is acceptable at call sites. Every authority-aware seam must receive an explicit authority reference.

3. Define a crate-private legacy adapter holding only:

```rust
&PublisherTrustStore
&DeveloperApprovalStore
```

Its `revalidate_current` implementation must delegate to the existing `PackageTrustEvidence::revalidate_current` method and preserve all existing signed-publisher and unsigned-developer behaviour and error contracts.

4. Define a crate-private exact-candidate adapter holding only:

```rust
&ExactCandidateTrustStore
```

Its validation order is frozen:

- call `candidate.validate()` and map the existing safe candidate error;
- call `evidence.require_for_candidate(candidate)`;
- require `TrustModeEvidence::ExactCandidate`;
- load the current exact-candidate record with `store.find(candidate_id)`;
- call `record.require_for_candidate(candidate)`;
- compare the evidence candidate ID, candidate-record digest, installation-trust record digest, and approving authority against the current record;
- reconstruct `PackageTrustEvidence::exact_candidate(&record)`;
- require exact equality with the supplied evidence.

5. Exact-candidate stable failures:

Wrong trust mode:

```text
code: trust_exact_candidate_authority_required
message: exact-candidate trust requires current installation-trust authority
```

Current exact record absent:

```text
code: trust_drift
message: exact-candidate installation trust is absent
```

Current record or reconstructed evidence differs:

```text
code: trust_drift
message: exact-candidate installation trust changed
```

Existing exact-store, candidate, and record-validation failures remain unchanged and fail closed.

6. Add crate-private authority-aware forms equivalent to:

```rust
PreparedSupervisedLaunch::revalidate_current_trust_with(...)
PreparedSupervisedLaunch::launch_for_candidate_with(...)
run_host_conformance_with_authority(...)
InstallationApprovalStore::approve_with_authority(...)
InstalledPlugRegistry::install_disabled_with_authority(...)
```

Names may vary narrowly if Rust readability improves, but each seam must require an explicit authority argument.

7. Route every current-trust check inside those internal paths through the supplied authority, including:

- the pre-launch conformance revalidation;
- the revalidation performed immediately by provider launch;
- approval creation;
- installation entry validation;
- the final installation revalidation after staging and before final publication.

No hidden call to the legacy publisher/developer revalidation may remain inside an authority-aware path.

8. Preserve existing public and accepted crate-visible signatures.

Existing methods construct `PublisherDeveloperTrustAuthority` locally and call the authority-aware implementation. Existing callers must not need to change.

9. Add:

```text
tethers-0.1/host-rust/src/current_trust_tests.rs
```

Declare it from `lib.rs` only under `#[cfg(test)]` so tests can exercise crate-private seams without exposing production API.

10. Focused tests must prove:

- matching exact-candidate evidence and current exact store are accepted;
- exact evidence from one valid store is rejected against a different valid current record for the same candidate;
- absent exact authority is rejected;
- signed-publisher or unsigned-developer evidence is rejected by the exact adapter with the frozen mode error;
- malformed or corrupt exact-store evidence fails closed;
- a distinctive recording/failing authority is invoked by every authority-aware downstream seam and its error is propagated rather than replaced by legacy trust lookup;
- the provider-launch path uses the same supplied authority for its immediate revalidation;
- the installation final revalidation after staging also uses the supplied authority;
- accepted public signed-publisher and unsigned-developer paths remain behaviourally unchanged;
- no production test-only constructor or public authority export is added.

Use direct Rust fixtures and existing store/publication APIs. A small crate-internal test authority under `#[cfg(test)]` is allowed.

## Relevant components

- `tethers-0.1/host-rust/src/current_trust.rs`
- `tethers-0.1/host-rust/src/current_trust_tests.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `tethers-0.1/host-rust/src/trust.rs`
- `tethers-0.1/host-rust/src/installation_trust.rs`
- `tethers-0.1/host-rust/src/launch_profile.rs`
- `tethers-0.1/host-rust/src/conformance.rs`
- `tethers-0.1/host-rust/src/installed.rs`
- `PackageTrustEvidence`
- `TrustModeEvidence`
- `ExactCandidateTrustStore`
- `ExactCandidateTrustRecord`
- `PublisherTrustStore`
- `DeveloperApprovalStore`
- existing `M3Error` and `Result`

## Frozen decisions and invariants

- `CurrentTrustAuthority` and both adapters are crate-private.
- Authority-aware seams require an explicit authority argument. No `Option`, default, implicit fallback, global, static, or thread-local authority is permitted.
- Exact-candidate authority never consults or mutates publisher trust or developer approval.
- Legacy authority never consults exact-candidate installation trust.
- Existing public method signatures and accepted callers remain unchanged.
- `PackageTrustEvidence::revalidate_current` retains its current public behaviour and continues to refuse exact-candidate mode.
- Current trust authority validates authority only. It does not absorb candidate-byte integrity or remove existing candidate/launch revalidation.
- No lock, executor, action loop, publication intent, installed-root recovery, CLI, or enablement work is part of J24K1.
- Accepted evidence schemas, JSON, dependencies, Cargo configuration, OCaml, and Cargo.lock remain unchanged.

## Acceptance criteria

1. The new crate-private authority module compiles without public API expansion.
2. Exact-candidate authority accepts only an exact matching current record and reconstructed evidence.
3. Missing, stale, malformed, wrong-mode, or mismatched exact authority fails closed with the frozen or existing safe error.
4. Every new authority-aware downstream seam requires an explicit authority reference.
5. No authority-aware path silently invokes legacy trust revalidation.
6. Existing public conformance, approval, and installation signatures remain source-compatible.
7. Existing signed-publisher and unsigned-developer behaviour remains green.
8. Tests behaviourally prove authority propagation through conformance launch, approval, installation entry, and final installation revalidation.
9. No lock, executor, recovery journal, CLI, enablement, schema, dependency, or Cargo.lock change appears.
10. Focused Nextest executes J24K1 tests with zero retries.
11. Focused ordinary Cargo J24K1 tests pass.
12. J24I exact-candidate trust and representative existing conformance/installation suites remain green.
13. Final verification retains the accepted baseline plus the new J24K1 tests.
14. The final diff contains only permitted files and Cargo.lock remains byte-identical.

## Required verification

Run from the repository root:

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1

cargo fmt `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all -- --check

cargo nextest run `
  --config-file .config/nextest.toml `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all-features --locked --lib `
  -E 'test(j24k1)'

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --lib j24k1 `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --test j24i_exact_candidate_installation_trust `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --test m3_lifecycle `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --test j23c2_pdf_conformance `
  --locked

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --test j23c3_installed_pdf_execution `
  --locked

$env:PATH = "$PSHOME;$env:PATH"
just verify

Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256
git diff --check
git status --short
```

The focused Nextest filter may be adjusted once if Nextest reports the crate-internal test name differently. Do not repeat an ineffective filter blindly. Record the exact executed test count.

OpenCode LSP is not a gate. Do not spend task time diagnosing empty LSP output. Use `rg`, compiler diagnostics, Cargo tests, and Nextest.

Cargo.lock must remain:

`D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB`

If the five previously documented `pwsh.exe` environment failures remain after prepending `$PSHOME`, record them exactly and prove they are unchanged. No other full-verification failure is acceptable.

## Forbidden changes

- No public `CurrentTrustAuthority` API.
- No `Option` authority argument, implicit fallback, global state, thread-local state, or mutable policy switch.
- No change to `PackageTrustEvidence` or `TrustModeEvidence` schemas.
- No weakening of exact trust, candidate, launch-profile, conformance, approval, or installed validation.
- No installation lock or lock file.
- No `execute_next_installation_action` implementation.
- No internal multi-mutation loop.
- No installation publication intent, recovery matrix, installed-root repair, or orphan deletion.
- No CLI, prompt, terminal output, enablement, operational-scope, packaging, release, or OCaml work.
- No dependency, Cargo configuration, tool configuration, or Cargo.lock changes.
- No production test-only constructors.
- No files outside the permitted set.

Permitted files:

- `tethers-0.1/host-rust/src/current_trust.rs`;
- `tethers-0.1/host-rust/src/current_trust_tests.rs`;
- `tethers-0.1/host-rust/src/lib.rs`;
- `tethers-0.1/host-rust/src/launch_profile.rs`;
- `tethers-0.1/host-rust/src/conformance.rs`;
- `tethers-0.1/host-rust/src/installed.rs`;
- `docs/CURRENT_CLINE_TASK.md`;
- `docs/worker-notes/2026-08-04-j24k1-current-trust-authority.md`.

The frozen architecture file is already present and must not be edited by the implementation worker.

## Stop conditions

Stop as `BLOCKED` only if:

- Rust visibility rules cannot support a crate-private test module without public API expansion;
- an existing downstream seam cannot be split into public legacy wrapper plus crate-private authority-aware implementation without changing accepted behaviour;
- exact-candidate authority cannot be proven current using the accepted store and evidence pins;
- safe implementation requires schema, dependency, lock, executor, recovery, CLI, enablement, or out-of-scope changes;
- required verification still fails after one evidence-led correction.

Do not stop for failed LSP, an unavailable optional tool, one ineffective Nextest filter, or one failed exact text replacement. Reread the current file, make one smaller evidence-led correction, and continue.

## Expected pre-existing changes

The branch already contains documentation-only preparation commits for:

- `docs/architecture/J24K_LOCKED_GATED_INSTALLATION_STEP_EXECUTOR.md`;
- `docs/CURRENT_CLINE_TASK.md`;
- `docs/worker-notes/2026-08-04-j24k1-current-trust-authority.md`.

Treat those as expected task scaffolding. Do not revert them.
