# J24H Installation Evidence Access Foundation

## Purpose

J24H adds the smallest persistence and read-only access seams required before a
safe installation reconciliation planner can exist.

The installation design requires a later invocation to resume after trust or
conformance evidence has already been created. `ConformanceEvidence` pins a
`launch_profile_evidence_digest`, but the complete `LaunchProfileEvidence` is
not currently stored. Without the full evidence object, a later process cannot
revalidate and reuse a passed conformance result safely.

J24H therefore does exactly two things:

1. adds an immutable `LaunchProfileEvidenceStore` using the existing audited
   `StoreRoot` persistence primitive;
2. adds read-only `open_existing` constructors to the evidence stores the future
   planner must inspect without creating any directory.

J24H does not compute an installation plan, grant trust, prepare or launch a
provider, run conformance, approve installation, copy payloads, publish
installed state, acquire a lock, or add a CLI command.

## Frozen public seams

### Candidate registry

Add:

```rust
pub fn open_existing(
    root: &Path,
    quarantine_root: &Path,
) -> Result<Self, PackageError>;
```

It must require both roots to exist as safe ordinary directories and must never
create either root.

### Existing M3 stores

Add this constructor to each type:

```rust
pub fn open_existing(path: &Path) -> Result<Self>;
```

Types:

- `PublisherTrustStore`
- `DeveloperApprovalStore`
- `ConformanceEvidenceStore`
- `InstallationApprovalStore`

Each constructor must delegate to `StoreRoot::open_existing` and perform no
write.

`InstalledPlugRegistry::open_existing` already exists and must not change.

### Launch-profile evidence store

Add to `launch_profile.rs`:

```rust
pub struct LaunchProfileEvidenceStore {
    root: StoreRoot,
}

impl LaunchProfileEvidenceStore {
    pub fn open(path: &Path) -> Result<Self>;

    pub fn open_existing(path: &Path) -> Result<Self>;

    pub fn create(
        &self,
        evidence: &LaunchProfileEvidence,
    ) -> Result<()>;

    pub fn load_all(&self) -> Result<Vec<LaunchProfileEvidence>>;
}
```

Do not change the `LaunchProfileEvidence` schema or the behaviour of
`PreparedSupervisedLaunch`.

## Launch-profile record identity

A launch-profile evidence file is named from its existing content digest:

```text
<64-lowercase-hex>.json
```

The filename stem is the suffix after `sha256:` in
`profile_evidence_digest`.

The suffix must contain exactly 64 lowercase hexadecimal characters.

Example:

```text
launch-profiles/
  0123456789abcdef...64-hex-characters.json
```

Do not introduce a second UUID or timestamp. The evidence already has a stable
content identity.

## Launch-profile store behaviour

### `open`

Use `StoreRoot::open`. This is the executor-facing constructor and may create
the configured store root.

### `open_existing`

Use `StoreRoot::open_existing`. It must fail if the root is absent, unsafe, or
not a directory. It must never create the root.

### `create`

1. call `evidence.validate()`;
2. derive and validate the 64-character record identity from
   `profile_evidence_digest`;
3. call `StoreRoot::create_json` with that identity;
4. do not overwrite or replace existing evidence;
5. return `record_conflict` when the exact destination or temporary file already
   exists, through the existing `StoreRoot` behaviour.

`StoreRoot::create_json` remains the sole atomic-write authority. Do not add a
second temporary-file implementation.

### `load_all`

For every entry:

1. reject `.tmp` entries with:
   - code: `launch_profile_store_invalid`
   - message: `torn launch-profile evidence`
2. reject non-`.json` entries with:
   - code: `launch_profile_store_invalid`
   - message: `unexpected launch-profile store entry`
3. load through `StoreRoot::read`;
4. call `LaunchProfileEvidence::validate()`;
5. require the filename stem to equal the validated digest suffix, otherwise:
   - code: `launch_profile_store_invalid`
   - message: `launch-profile filename mismatch`
6. reject a repeated `profile_evidence_digest` with:
   - code: `launch_profile_store_invalid`
   - message: `duplicate launch-profile evidence`
7. sort the returned records by `profile_evidence_digest`.

Do not silently ignore temporary, unknown, malformed, mismatched, or duplicate
evidence.

## CandidateRegistry read-only opening

`CandidateRegistry::open_existing` must preserve every safety rule of `open`
without calling `create_safe_dir_all` or any other mutating function.

Required order:

1. reject lexically identical registry and quarantine paths with existing code
   `registry_invalid`;
2. call `verify_existing_chain` for both roots;
3. inspect both roots with `fs::symlink_metadata`;
4. require both entries to be directories;
5. canonicalise both roots;
6. verify both canonical chains;
7. reject roots that resolve to the same location;
8. return the registry.

Use existing `PackageError` conventions:

- absent, unreadable, or metadata/canonicalisation failure maps through the
  existing `candidate_io` path;
- unsafe links or Windows reparse points remain `unsafe_destination`;
- non-directory roots use `registry_invalid` with the message
  `registry and quarantine roots must be directories`.

Do not weaken or alter `CandidateRegistry::open`, `create`, or `load_all`.

## Read-only guarantee

Every `open_existing` constructor must be observationally read-only:

- no directory creation;
- no file creation;
- no deletion;
- no rename;
- no permission change;
- no evidence update;
- no timestamp or record regeneration by Tethers code.

A missing root must remain missing after failure.

Opening and loading an existing valid store must leave a recursive byte snapshot
unchanged.

## Required evidence

Add one focused integration file:

```text
tethers-0.1/host-rust/tests/j24h_installation_evidence_access.rs
```

The tests must prove:

1. `LaunchProfileEvidenceStore::create` and `load_all` round-trip one valid
   evidence object exactly.
2. The launch-profile filename is the digest suffix and contains no UUID or
   timestamp identity.
3. `open_existing` and `load_all` change no byte or path.
4. A second `create` of identical evidence fails with `record_conflict` and
   changes no byte.
5. A `.tmp` entry, non-JSON entry, mismatched filename, malformed evidence, and
   duplicate digest evidence each fail closed with the frozen code/message.
6. Missing launch-profile, publisher-trust, developer-approval, conformance, and
   installation-approval roots fail through `store_io` and remain absent.
7. Existing empty roots opened through every new M3 `open_existing` constructor
   remain byte-identical.
8. Missing candidate and quarantine roots fail without creating either root.
9. Existing candidate and quarantine roots open read-only and `load_all`
   preserves a byte snapshot.
10. A non-directory candidate root is rejected with the frozen
    `registry_invalid` message.
11. Unsafe symbolic-link paths are rejected where the platform permits the
    fixture.
12. On Windows, at least one real junction-backed store root is rejected and no
    path is created beneath its target.
13. Existing lifecycle and J24E through J24G tests remain green.

A valid launch-profile fixture may be constructed directly in the integration
test. Compute its digest by cloning the evidence, clearing
`profile_evidence_digest`, canonicalising with the already available
`serde_json_canonicalizer`, and hashing with the already available `sha2` crate.
Do not add a production test-only constructor.

## Suggested implementation order

1. Add `CandidateRegistry::open_existing` and its focused tests.
2. Add the four one-line M3 store `open_existing` constructors.
3. Add the `LaunchProfileEvidenceStore` and private digest-suffix helper.
4. Add launch-profile store refusal tests.
5. Add cross-store recursive snapshot tests.
6. Run all focused and full verification.

## Non-goals

J24H does not:

- consume `InstallationRequest`;
- add `installation_plan.rs`;
- select trust evidence;
- create developer approval;
- verify package signatures as part of planning;
- prepare a supervised launch;
- create scratch space;
- run a provider;
- run conformance;
- create installation approval;
- install or enable a Plug;
- add a lock;
- add `plug install` or another CLI command;
- change evidence schemas;
- change package or candidate schemas;
- change dependencies or lockfiles.

J24I will use these read-only seams to implement the installation reconciliation
planner.