# J24I Exact-Candidate Installation Trust

## Purpose

J24I adds the missing trust authority required by the frozen public installation
request.

The request contract says:

```json
"trust": { "scope": "exact_candidate" }
```

The existing M3 trust authorities do not express that scope:

- publisher trust applies to a signing key and optional package namespace;
- developer approval applies to one semantic package digest;
- neither record is pinned to one candidate identity and candidate-record
  digest.

J24I must not pretend either authority is exact-candidate trust. It adds one
small host-owned record and one new `PackageTrustEvidence` mode that bind the
human decision to one immutable candidate.

J24I does not plan installation, acquire a lock, prepare or launch a provider,
run conformance, create installation approval, copy payloads, publish installed
state, enable a Plug, or add a CLI command.

## Module boundary

Add:

```text
tethers-0.1/host-rust/src/installation_trust.rs
```

Export it from `lib.rs`.

The new module owns only:

- exact-candidate trust record construction and validation;
- immutable persistence through `StoreRoot`;
- exact candidate lookup;
- conversion into `PackageTrustEvidence` through a constructor in `trust.rs`.

It does not own publisher trust, developer approval, signatures, conformance,
installation approval, installed state, or enablement.

## Exact public record

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExactCandidateTrustRecord {
    pub schema_version: u32,
    pub candidate_id: String,
    pub candidate_record_digest: String,
    pub package_id: String,
    pub package_version: String,
    pub semantic_package_digest: String,
    pub raw_archive_digest: String,
    pub provider_id: String,
    pub provider_version: String,
    pub request_schema: String,
    pub trust_scope: String,
    pub approving_authority: String,
    pub created_unix_ms: u64,
    pub record_digest: String,
}
```

No second UUID is introduced. The candidate ID is the record identity.

The record intentionally does not contain conformance, launch-profile,
installation-path, installed, enablement, policy, credential, runtime approval,
Trail, or Anchor authority.

## Exact store seam

```rust
pub struct ExactCandidateTrustStore {
    root: StoreRoot,
}

impl ExactCandidateTrustStore {
    pub fn open(path: &Path) -> Result<Self>;
    pub fn open_existing(path: &Path) -> Result<Self>;

    pub fn create(
        &self,
        candidate: &CandidateRecord,
        request: &InstallationRequest,
        approving_authority: &str,
    ) -> Result<ExactCandidateTrustRecord>;

    pub fn find(
        &self,
        candidate_id: &str,
    ) -> Result<Option<ExactCandidateTrustRecord>>;

    pub fn load_all(
        &self,
    ) -> Result<Vec<ExactCandidateTrustRecord>>;
}
```

`open` delegates to `StoreRoot::open`.

`open_existing` delegates to `StoreRoot::open_existing` and never creates a
missing root.

## Record construction

`create` must validate before publishing:

1. validate the candidate with the existing candidate authority;
2. require the typed request schema to equal `tethers.plug-install/1`;
3. require the request candidate ID to equal the candidate ID;
4. confirm the typed trust scope is `InstallationTrustScope::ExactCandidate`;
5. require explicit non-isolated supervised-execution approval to be `true`;
6. confirm the typed target is `InstallationTargetState::Disabled`;
7. require a non-empty approving authority;
8. copy the exact candidate identity and evidence fields into the record;
9. set `request_schema` to `tethers.plug-install/1`;
10. set `trust_scope` to `exact_candidate`;
11. obtain `created_unix_ms` through the existing M3 clock authority;
12. calculate `record_digest` over canonical covered bytes;
13. validate the complete record;
14. publish through `StoreRoot::create_json` using `candidate_id` as the ID.

Do not trust public struct construction merely because the request has already
passed the J24G parser. Public string fields can be altered and the approval
boolean can be manually constructed as `false`.

The trust-scope and target-state enums each deliberately expose only one legal
variant. Their exactness is a type-level guarantee. Do not use unsafe code,
transmute, malformed memory, or another impossible fixture merely to fabricate
an alternative enum value for testing.

Do not call `create_json` until every request, candidate, and authority check has
passed.

## Record validation

`ExactCandidateTrustRecord::validate` may remain private, but add:

```rust
pub fn require_for_candidate(
    &self,
    candidate: &CandidateRecord,
) -> Result<()>;
```

Validation must require:

- schema version `1`;
- canonical lowercase hyphenated `candidate_id`;
- valid lowercase `sha256:` values for candidate record, semantic package, raw
  archive, and record digests;
- non-empty package, version, provider, and approving-authority strings;
- request schema exactly `tethers.plug-install/1`;
- trust scope exactly `exact_candidate`;
- a correct canonical record digest.

`require_for_candidate` must compare candidate ID, candidate record digest,
package ID and version, semantic package digest, raw archive digest, and provider
ID and version.

Any disagreement returns:

- code: `installation_trust_candidate_mismatch`
- message: `exact-candidate trust is not bound to this candidate`

## Store identity and loading

Files are named:

```text
<candidate-id>.json
```

`load_all` must:

1. reject `.tmp` entries with:
   - code: `installation_trust_invalid`
   - message: `torn exact-candidate trust record`
2. reject non-JSON entries with:
   - code: `installation_trust_invalid`
   - message: `unexpected exact-candidate trust entry`
3. read through `StoreRoot::read`;
4. validate the record;
5. require filename stem equal to `candidate_id`, otherwise:
   - code: `installation_trust_invalid`
   - message: `exact-candidate trust filename mismatch`
6. retain a defensive duplicate-candidate check using:
   - code: `installation_trust_invalid`
   - message: `duplicate exact-candidate trust evidence`
7. sort records by candidate ID.

Because candidate ID is also the filename, two validly named records for the
same candidate are structurally impossible in one flat directory. Duplicate
publication is proved through `record_conflict`; copied evidence under another
filename is a filename mismatch. Do not manufacture an impossible duplicate
fixture by weakening validation order.

`find` must use the validated store view and return at most one matching record.
It must not treat corrupt evidence as absence.

## Stable construction errors

Use existing `M3Error`.

| Condition | Code | Message |
|---|---|---|
| typed request is not valid for the frozen operation | `installation_trust_request_invalid` | `installation request is not valid for exact-candidate trust` |
| approving authority is empty | `installation_trust_invalid` | `approving authority is required` |
| record shape or digest is invalid | `installation_trust_invalid` | `invalid exact-candidate trust record` |
| candidate binding differs | `installation_trust_candidate_mismatch` | `exact-candidate trust is not bound to this candidate` |

Candidate validation failures map to:

- code: `candidate_invalid`
- message containing the existing safe candidate error text.

Do not expose filesystem paths, request JSON, quarantine locations, or raw
platform I/O details in new validation messages. Existing `StoreRoot` I/O errors
remain unchanged.

## Package trust evidence extension

Extend `TrustModeEvidence` in `trust.rs` with:

```rust
ExactCandidate {
    candidate_id: String,
    candidate_record_digest: String,
    installation_trust_record_digest: String,
    approving_authority: String,
}
```

Add:

```rust
pub fn exact_candidate(
    record: &ExactCandidateTrustRecord,
) -> Result<PackageTrustEvidence>;
```

The constructor must validate the record, set the top-level semantic package
digest from it, copy only the four fields above, and calculate the normal
`PackageTrustEvidence` digest.

`PackageTrustEvidence::validate` must validate the new mode fields.

`PackageTrustEvidence::require_for_candidate` must additionally require matching
candidate ID, candidate-record digest, and top-level semantic package digest.

A mismatch uses the existing code and message:

- `trust_candidate_mismatch`
- `trust evidence is not bound to this candidate semantic digest`

Do not alter the serialised fields of existing signed-publisher or
unsigned-developer evidence.

## Deliberate fail-closed current-authority boundary

J24I does not wire the new trust mode into provider execution.

When `PackageTrustEvidence::revalidate_current` encounters `ExactCandidate`, it
must return:

- code: `trust_exact_candidate_authority_required`
- message: `exact-candidate trust requires current installation-trust authority`

This prevents existing conformance, approval, installation, and launch paths
from accidentally treating the new evidence as current before the future locked
executor supplies and revalidates the exact trust store.

The read-only planner may use:

- `ExactCandidateTrustStore::find`;
- `ExactCandidateTrustRecord::require_for_candidate`;
- `PackageTrustEvidence::exact_candidate`;
- `PackageTrustEvidence::require_for_candidate`.

It must not use `revalidate_current` until the executor contract explicitly owns
the exact store authority.

## Required evidence

Add:

```text
tethers-0.1/host-rust/tests/j24i_exact_candidate_installation_trust.rs
```

Tests must prove:

1. a valid candidate and exact request create one exact trust record;
2. the filename is exactly `<candidate-id>.json` and no new UUID appears;
3. load and find round-trip the exact record;
4. open-existing on a missing root leaves it missing;
5. create and all read-only operations preserve unrelated files;
6. a second exact create returns `record_conflict` with no mutation;
7. manually constructed wrong request schema, mismatched candidate ID, false
   supervised-execution approval, and empty authority fail before publication;
8. the single-variant trust-scope and target-state types remain compile-time
   guarantees; no impossible negative enum fixture is required;
9. torn temporary, non-JSON, malformed, and mismatched-filename evidence fail
   closed;
10. copied evidence under another filename is a filename mismatch;
11. a record refuses a different candidate, including one with the same
    semantic digest but a different candidate ID or record digest;
12. exact `PackageTrustEvidence` is deterministic and validates;
13. exact trust evidence accepts only the matching candidate;
14. `revalidate_current` refuses the new mode with
    `trust_exact_candidate_authority_required`;
15. existing publisher and developer trust tests remain green;
16. J24E through J24H regressions remain green;
17. no test launches provider code or creates conformance, installation,
    installed, enablement, policy, Trail, or Anchor state.

Use direct Rust fixtures. Do not add a production test-only constructor.

## Editing recovery discipline

If an exact replacement reports that `oldString` was not found:

1. do not retry the identical replacement;
2. reread the current file;
3. locate the smallest stable surrounding anchor;
4. create a fresh, smaller patch against the latest contents;
5. stop after two materially different failed attempts instead of rewriting the
   whole file.

## Non-goals

J24I does not:

- implement the installation reconciliation planner;
- add host-data-root layout orchestration;
- acquire or define the installation lock;
- mutate publisher trust or developer approval;
- verify publisher signatures;
- prepare or launch a provider;
- run or persist conformance;
- create installation approval;
- copy package payloads;
- publish installed state;
- enable or disable a Plug;
- add `plug install`, another CLI command, or public output;
- change Tethers Core, OCaml semantics, package schema, candidate schema,
  launch-profile schema, conformance schema, installation-approval schema, or
  installed-record schema;
- add dependencies or change lockfiles.

J24J will implement the read-only installation reconciliation planner over J24G,
J24H, and J24I.
