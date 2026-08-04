Task: `J24I - Exact-candidate installation trust`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `712ae4d27a969375e7b2b8980b2e17c5d26e3377`

Implementation checkpoint: `bd16b01349d3db2bec4cf2406d02d8567a4a079c`

## Correction (J24I review)

Independent review found three connected validation gaps, corrected in
`bd16b01`:

### 1. Canonical candidate UUID validation

`ExactCandidateTrustRecord::validate` now uses `Uuid::parse_str` with a
canonical hyphenated form check. Non-UUID, uppercase, and non-canonical
UUID values are rejected.

### 2. Record validation before evidence construction

`PackageTrustEvidence::exact_candidate()` begins with `record.validate()?;`
and ends with `evidence.validate()?;` before returning. An invalid or
altered trust record cannot produce package-trust evidence.

### 3. Self-validation in require_for_candidate

`ExactCandidateTrustRecord::require_for_candidate` begins with
`self.validate()?;` so a manually constructed record with matching visible
fields but invalid record digest cannot pass candidate matching.

### 4. ExactCandidate evidence field validation

`PackageTrustEvidence::validate` now checks the ExactCandidate mode:
canonical UUID candidate_id, lowercase sha256: candidate_record_digest,
installation_trust_record_digest, semantic_package_digest, and non-empty
approving_authority.

`is_valid_candidate_id` and `is_sha256_digest` helpers added to `trust.rs`.

### 5. Corrected and added tests

The altered-record test now expects failure at construction with
`installation_trust_invalid`. Eight new tests prove rejection of:
non-UUID candidate ID with recomputed digest, uppercase UUID with
recomputed digest, mismatched evidence via require_for_candidate, and
fabricated evidence with invalid candidate ID / candidate-record digest /
installation-trust digest / empty authority / invalid semantic digest, each
with recomputed outer digest.

## Requested outcome

Add the missing exact-candidate trust authority required by the frozen
installation request. J24I adds one immutable trust record pinned to candidate ID
and candidate-record digest, extends PackageTrustEvidence with one exact-candidate
mode, and deliberately refuses current-authority revalidation.

## Changes made

- `src/installation_trust.rs`: Added `ExactCandidateTrustRecord` with frozen
  fields (candidate_id, candidate_record_digest, package_id/version,
  semantic_package_digest, raw_archive_digest, provider_id/version,
  request_schema, trust_scope, approving_authority, created_unix_ms,
  record_digest). Candidate ID is the record identity; no second UUID.
  `ExactCandidateTrustStore` delegates open/open_existing to StoreRoot,
  validates the candidate + request schema + candidate ID match + supervised
  execution approval + target state before publication through
  `StoreRoot::create_json(candidate_id, &record)`. `validated_view` builds
  a sorted scan rejecting torn, non-JSON, malformed, filename-mismatched, and
  duplicate evidence. `find` and `load_all` use the validated view.
  `require_for_candidate` binds to candidate ID, candidate-record digest,
  package ID/version, semantic digest, raw digest, and provider ID/version.

- `src/trust.rs`: Extended `TrustModeEvidence` with `ExactCandidate`
  variant (candidate_id, candidate_record_digest,
  installation_trust_record_digest, approving_authority). Added
  `PackageTrustEvidence::exact_candidate(record)` constructor with
  deterministic evidence digest from the validated record. Extended
  `validate` (works for all modes), `require_for_candidate` (additionally
  checks candidate_id and candidate_record_digest for ExactCandidate mode).
  `revalidate_current` fails closed for `ExactCandidate` with code
  `trust_exact_candidate_authority_required` and message `exact-candidate
  trust requires current installation-trust authority`.

- `src/lib.rs`: Exported `installation_trust` module.

- `tests/j24i_exact_candidate_installation_trust.rs`: 22 focused integration
  tests proving creation, filename identity, round-trip, missing-root
  preservation, unrelated-file preservation, record_conflict rejection, wrong
  schema/mismatched ID/false approval/empty authority refusals, torn/non-JSON/
  malformed/filename-mismatch fail-closed, different-candidate refusal (same
  semantic digest, different ID), PackageTrustEvidence determinism and
  validation, exact candidate binding, altered-record rejection,
  revalidate_current refusal, corrupt-evidence-not-absence, and empty-store
  round-trip.

- `docs/CURRENT_CLINE_TASK.md`: Status transitions.

## Decisions and assumptions

- Single-variant trust-scope and target-state enums are compile-time
  guarantees per the blueprint. No unsafe or impossible negative enum fixture
  was required.

- The defensive duplicate-candidate check in `validated_view` is structurally
  unreachable through normal store operations because candidate ID is the
  filename identity, but it satisfies the blueprint requirement.

- `require_for_candidate` in PackageTrustEvidence checks both
  semantic_package_digest (for all modes) and, for ExactCandidate mode,
  candidate_id + candidate_record_digest. A mismatch uses the existing code
  `trust_candidate_mismatch` with the existing message.

- No existing signed-publisher or unsigned-developer serialised fields or
  behaviour were changed.

## Evidence

- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` — PASS.
- `cargo +1.89.0 fmt --all -- --check` — PASS.
- `cargo +1.89.0 test installation_trust --locked` — 31 + 22 passed.
- `cargo +1.89.0 test trust --locked` — 31 passed.
- `cargo +1.89.0 test --test j24i_exact_candidate_installation_trust --locked` — 22 passed.
- `cargo +1.89.0 test installation_request --locked` — 16 passed.
- `cargo +1.89.0 test --test j24g_installation_request --locked` — 16 passed.
- `cargo +1.89.0 test launch_profile --locked` — 19 passed.
- `cargo +1.89.0 test --test j24h_installation_evidence_access --locked` — 19 passed.
- `cargo +1.89.0 test candidate_preparation --locked` — 17 passed.
- `cargo +1.89.0 test --test j24e_candidate_preparation --locked` — 17 passed.
- `cargo +1.89.0 test --test j24f_plug_stage_cli --locked` — 6 passed.
- `cargo +1.89.0 test --all-targets --all-features --locked` — 921 passed,
  5 failed only because `pwsh.exe` was not found (documented environment
  failures).
- `git diff --check` — PASS.

## Discoveries

None beyond the structurally unreachable duplicate-candidate check noted above.

## Remaining risks

The five documented `pwsh.exe not found` full-suite failures remain an
environment limitation. No known J24I implementation risk remains within the
packet scope.

## Smallest next action

Lucy performs the bounded final review of the pushed J24I branch. J24J follows
with the read-only installation reconciliation planner.

## References

- `docs/architecture/J24I_EXACT_CANDIDATE_INSTALLATION_TRUST.md`
- `tethers-0.1/host-rust/src/installation_trust.rs`
- `tethers-0.1/host-rust/src/trust.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `tethers-0.1/host-rust/tests/j24i_exact_candidate_installation_trust.rs`
- Branch: `opencode/j24i-exact-candidate-installation-trust`
