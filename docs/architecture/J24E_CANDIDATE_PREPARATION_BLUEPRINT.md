# J24E Candidate Preparation Blueprint

Status: implementation blueprint for the authorised J24E packet

## Why J24E stops before the CLI

J24D completed the public enable/disable loop. The next architectural boundary is
package intake, but a single public `plug install` command would force one worker
to combine archive inspection, quarantine, candidate identity, trust,
conformance, installation approval, installed publication and rollback.

J24E therefore adds one host-owned candidate-preparation service and no CLI.
J24F can then expose a thin `plug stage` adapter without reopening archive or
quarantine design.

A prepared candidate is:

- inspected without execution;
- extracted into immutable quarantine;
- recorded under one immutable candidate identity;
- untrusted;
- unapproved;
- uninstalled;
- disabled and non-operational;
- unable to create a binding, provider session, policy permission, event or Trail.

## Exact public Rust seam

Create `tethers-0.1/host-rust/src/candidate_preparation.rs` with this narrow
shape:

```rust
use std::path::Path;

use crate::candidate::CandidateRecord;
use crate::package::PackageError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidatePreparationDisposition {
    Created,
    Existing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidatePreparation {
    pub candidate: CandidateRecord,
    pub disposition: CandidatePreparationDisposition,
}

pub fn prepare_installation_candidate(
    host_data_root: &Path,
    package_path: &Path,
) -> Result<CandidatePreparation, PackageError>;
```

Export only the module through `src/lib.rs`. Do not export helper functions.

## Frozen host layout

The service derives exactly these children below an existing host data root:

```text
candidates/
quarantine/
```

The host data root itself must already exist as an absolute ordinary directory
with a safe non-reparse path chain. The service never creates the host data
root. The two children may be created only after package inspection succeeds.

J24E does not create or inspect:

```text
install/
installed-records/
enablements/
trust/
conformance/
approvals/
```

## Required orchestration order

Implement in this order. Do not reorder mutation before validation.

```text
1. Validate absolute package path and existing absolute host data root.
2. Inspect the package through package::inspect.
3. Record whether candidates/ and quarantine/ existed before this call.
4. Open CandidateRegistry for candidates/ and quarantine/.
5. Load and validate every existing candidate once.
6. Detect an exact previously prepared archive before extraction.
7. Detect conflicting semantic evidence before extraction.
8. Extract through candidate::extract_to_quarantine.
9. Publish through CandidateRegistry::create.
10. Return Created, or Existing for an exact replay.
```

No provider process, conformance harness, trust authority or installed registry
may be reached.

## Exact replay rule

An existing candidate is reusable only when exactly one loaded candidate has the
same `raw_archive_digest` and all report-pinned evidence agrees:

- package ID and version;
- semantic package digest;
- source size;
- provider ID and version;
- launch path, arguments and working directory;
- capability operation namespace;
- selected platform;
- `plug.json` evidence;
- payload evidence;
- signature evidence and `signatures_present`;
- capability evidence;
- inspection report version and evidence digest.

Return that validated candidate with disposition `Existing`. Perform no
extraction, file write, timestamp update or record creation.

Zero matches continues to preparation. More than one raw-archive match is
`candidate_conflict`. One raw-archive match with disagreeing evidence is
`record_invalid`.

## Semantic conflict preflight

Before extraction, refuse any existing candidate with the same package ID and
package version but a different semantic package digest. Preserve the existing
`semantic_conflict` code.

Different raw archives with the same semantic digest are not automatically the
same candidate because detached signature evidence may differ. Only exact raw
archive replay receives `Existing` treatment.

## Failure cleanup

The low-level extractor already removes incomplete `.staging-*` directories.
J24E additionally owns these narrow cleanup duties:

- if candidate publication fails after a final quarantine directory exists,
  remove only that newly returned quarantine directory;
- never remove a pre-existing candidate or quarantine directory;
- if this call created `candidates/` or `quarantine/`, remove that child only when
  it is empty after rollback;
- never recursively remove the host data root;
- verify the final quarantine directory is still confined beneath the canonical
  quarantine root before removal;
- if cleanup itself fails, return `candidate_rollback_failed` rather than claiming
  a clean refusal.

The rollback error message may mention the original stable error code, but must
not disclose an absolute path.

## Error construction and mapping preparation

J24E returns `PackageError`; J24F will map it into the CLI envelope.

Use existing stable codes whenever possible:

- package inspection codes unchanged;
- `unsafe_destination` for unsafe roots or path chains;
- `candidate_io` for ordinary candidate storage I/O;
- `record_invalid` for inconsistent exact-replay evidence;
- `semantic_conflict` for same release with different semantic evidence;
- `candidate_conflict` for ambiguous exact-archive evidence;
- `candidate_rollback_failed` only when cleanup cannot restore the bounded
  candidate/quarantine boundary.

Do not add a generic error wrapper or erase the original stable code.

## Near-code algorithm

```rust
pub fn prepare_installation_candidate(
    host_data_root: &Path,
    package_path: &Path,
) -> Result<CandidatePreparation, PackageError> {
    require_absolute_existing_file(package_path)?;
    require_absolute_existing_safe_directory(host_data_root)?;

    // Inspection must precede all candidate-store creation.
    let report = package::inspect(package_path)?;

    let candidate_root = host_data_root.join("candidates");
    let quarantine_root = host_data_root.join("quarantine");
    let candidate_root_existed = candidate_root.exists();
    let quarantine_root_existed = quarantine_root.exists();

    let registry = match CandidateRegistry::open(&candidate_root, &quarantine_root) {
        Ok(registry) => registry,
        Err(error) => {
            cleanup_new_empty_roots(...)?;
            return Err(error);
        }
    };
    let existing = match registry.load_all() {
        Ok(records) => records,
        Err(error) => {
            cleanup_new_empty_roots(...)?;
            return Err(error);
        }
    };

    match exact_replay(&existing, &report)? {
        Some(candidate) => {
            return Ok(CandidatePreparation {
                candidate,
                disposition: CandidatePreparationDisposition::Existing,
            });
        }
        None => {}
    }

    refuse_semantic_conflict(&existing, &report)?;

    let quarantined = match extract_to_quarantine(&report, &quarantine_root) {
        Ok(value) => value,
        Err(error) => {
            cleanup_new_empty_roots(...)?;
            return Err(error);
        }
    };

    match registry.create(&quarantined) {
        Ok(candidate) => Ok(CandidatePreparation {
            candidate,
            disposition: CandidatePreparationDisposition::Created,
        }),
        Err(error) => {
            rollback_new_quarantine(&quarantine_root, &quarantined.directory)?;
            cleanup_new_empty_roots(...)?;
            Err(error)
        }
    }
}
```

This is structural guidance, not permission to duplicate candidate validation.
`package::inspect`, `extract_to_quarantine`, `CandidateRegistry::open`,
`CandidateRegistry::load_all` and `CandidateRegistry::create` remain the sole
low-level authorities.

## Test fixture recipe

Use the accepted PDF package builder with deliberately non-executable provider
bytes. A successful candidate preparation from bytes such as
`b"not-an-executable-provider"` proves the service never launches the payload.

Use two packages built with different provider byte strings to create the same
package ID/version with different semantic package digests. This provides the
semantic-conflict fixture without hand-editing ZIPs or `plug.json`.

Snapshot tests must record relative entry names and SHA-256 file digests beneath
the host root.

Required scenarios:

1. Valid PDF package creates one candidate record and one immutable quarantine
   subtree, returning `Created`.
2. The exact same archive returns the same candidate ID with `Existing` and a
   byte-for-byte identical host tree.
3. Same package release built from different provider bytes fails
   `semantic_conflict` before extraction and changes nothing.
4. Malformed package fails before `candidates/` or `quarantine/` exists.
5. Missing, relative, non-directory or unsafe host root fails without creating
   it or either child.
6. Corrupt existing candidate evidence fails closed before extraction and
   changes nothing.
7. Candidate and quarantine roots remain distinct and ordinary.
8. Candidate output pins the exact PDF package, provider, platform and
   `pdf.inspect@1` evidence.
9. No install, installed-record, enablement, trust, conformance or approval path
   is created.
10. The standalone rollback helper removes only a newly created confined
    quarantine directory and newly created empty roots.

## J24F handoff contract

After J24E is accepted, J24F should need only:

```rust
let result = candidate_preparation::prepare_installation_candidate(
    &host_data_root,
    &package,
);
```

J24F will own Clap syntax, `tethers.cli/1` mapping and the public command name
`plug stage`. It must not reopen package inspection, candidate matching,
quarantine cleanup or registry semantics.
