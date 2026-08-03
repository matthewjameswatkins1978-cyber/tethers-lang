# J24F Plug Stage CLI Blueprint

Status: implementation blueprint for the authorised J24F packet

## Purpose

J24E accepted one host-owned candidate-preparation service:

```rust
candidate_preparation::prepare_installation_candidate(
    host_data_root,
    package_path,
)
```

That service already owns hostile package inspection, safe quarantine,
validated candidate registry access, exact archive replay, semantic-conflict
preflight and bounded rollback.

J24F exposes that accepted service through one thin public command. It must not
reopen or duplicate any package, archive, quarantine, replay, candidate identity
or rollback decision.

## Exact command

```text
tethers-reference-host plug stage \
  --host-data-root <ABSOLUTE_PATH> \
  --package <ABSOLUTE_TETHERPLUG_PATH>
```

`stage` means: inspect the package, place its verified contents in immutable
quarantine and publish or reuse one installation-candidate identity.

It does not mean trust, approve, install, enable, launch or make available.

## Required source shape

The implementation should be limited to:

1. one `Stage` variant in `cli::PlugCommand`;
2. one `application.rs` route;
3. one `plug_command::run_stage` adapter;
4. focused CLI syntax tests;
5. one compiled-binary integration test file.

The production adapter should structurally resemble:

```rust
pub fn run_stage(host_data_root: &Path, package_path: &Path) -> PlugCommandResult {
    if !host_data_root.is_absolute() {
        return invalid_usage("/host-data-root", "--host-data-root must be absolute");
    }
    if !package_path.is_absolute() {
        return invalid_usage("/package", "--package must be absolute");
    }

    match candidate_preparation::prepare_installation_candidate(
        host_data_root,
        package_path,
    ) {
        Ok(prepared) => stage_success(prepared),
        Err(error) => stage_error(error),
    }
}
```

This pseudocode is intentionally close to the desired implementation. The
worker may use small private formatting and mapping helpers inside
`plug_command.rs`, but must not add another application service or public type.

## Exact success envelope

Both newly created and exact-replay candidates return `status: "ok"`, exit `0`
and command `"plug stage"`.

The data object contains exactly one key, `candidate`:

```json
{
  "candidate": {
    "candidate_id": "<UUID>",
    "disposition": "created",
    "state": "quarantined_installation_candidate",
    "package_id": "tethers.pdf-tools",
    "package_version": "1.0.0",
    "semantic_package_digest": "sha256:...",
    "raw_archive_digest": "sha256:...",
    "provider_id": "tethers-pdf-provider",
    "provider_version": "1.0.0",
    "platform": {
      "os": "windows",
      "architecture": "x86_64"
    },
    "capabilities": [
      {
        "name": "pdf.inspect",
        "version": 1,
        "manifest_digest": "sha256:...",
        "operation": "pdf_inspect"
      }
    ],
    "created_unix_ms": 0
  }
}
```

The only disposition strings are:

- `created` for `CandidatePreparationDisposition::Created`;
- `existing` for `CandidatePreparationDisposition::Existing`.

Capabilities must be sorted by `(name, version, operation)` before output even
if the accepted package currently contains one capability.

The public candidate object must contain exactly these keys:

```text
candidate_id
disposition
state
package_id
package_version
semantic_package_digest
raw_archive_digest
provider_id
provider_version
platform
capabilities
created_unix_ms
```

Each capability object must contain exactly:

```text
name
version
manifest_digest
operation
```

## Forbidden public fields

Do not expose:

```text
quarantine_relative_path
source_size_bytes
plug_json
payloads
signature_files
signatures_present
launch_path
launch_arguments
provider_working_directory
capability_operation_namespace
inspection_report_format_version
inspection_evidence_digest
record_digest
absolute package path
absolute host path
```

Do not expose trust, approval, conformance, installed, enablement, policy,
provider-session, replay, event, Anchor or Trail evidence.

## Exact error mapping

CLI syntax owns only absolute-path usage checks:

| condition | status | exit | code | field |
|---|---:|---:|---|---|
| relative host data root | `invalid_cli_usage` | 2 | `invalid_cli_usage` | `/host-data-root` |
| relative package path | `invalid_cli_usage` | 2 | `invalid_cli_usage` | `/package` |

After those two checks, call the J24E service exactly once and preserve its
stable `PackageError.code` and message.

Map service errors as follows:

| error code | status | exit |
|---|---:|---:|
| `archive_read` | `unavailable` | 4 |
| `candidate_io` | `unavailable` | 4 |
| `candidate_rollback_failed` | `failed` | 6 |
| `clock` | `failed` | 6 |
| every other `PackageError` code | `invalid_data` | 3 |

Do not infer status from error-message text. Do not replace specific codes with a
generic stage error.

Every command invocation must emit exactly one JSON envelope line. The process
exit code must equal the envelope `exit_code`.

## Mutation boundary

The CLI adapter itself performs no filesystem operation other than whatever is
performed inside `prepare_installation_candidate`.

It must not:

- call `package::inspect` directly;
- open `CandidateRegistry`;
- inspect or delete quarantine paths;
- load candidate records;
- calculate any digest;
- construct a `CandidateRecord`;
- launch a provider;
- access trust, conformance, approval, installed or enablement stores.

A first successful invocation may create only the J24E-authorised
`candidates/` and `quarantine/` trees.

An exact replay must return `existing`, reuse the same candidate ID and change no
relative path or byte beneath the host root.

## Compiled-binary evidence

Use `pdf_tools::build_reference_package` with deliberately non-executable
provider bytes. Put packages outside the host data root so host-tree snapshots
cover only service-owned state.

The J24F integration file should prove:

1. first stage succeeds with `created`, the exact public allowlist and no
   forbidden field;
2. the result pins the PDF package, provider, Windows x86_64 platform and
   `pdf.inspect@1` capability;
3. the candidate record and immutable quarantine subtree exist, while no other
   lifecycle path exists;
4. exact replay succeeds with `existing`, the same candidate ID and an identical
   relative-path/SHA-256 snapshot;
5. malformed package returns `invalid_data`, exit `3`, its exact inspector code
   and creates no candidate path;
6. missing package returns `unavailable`, exit `4`, code `archive_read` and
   creates no candidate path;
7. same release with different semantic evidence returns `invalid_data`, exit
   `3`, code `semantic_conflict` and changes no byte;
8. corrupt existing candidate evidence returns `invalid_data`, exit `3`, the
   existing `record_invalid` code and changes no byte;
9. relative host and package arguments return exit `2` with the exact field
   pointer before service mutation;
10. on Windows, junction-backed package input returns `invalid_data`, exit `3`,
    code `unsafe_destination` and creates no candidate path;
11. malformed command shapes are rejected by Clap with process exit `2`;
12. every parsed envelope has process/envelope exit-code parity.

## Exclusions

No `plug install`, trust, developer approval, publisher trust, signature,
revocation, launch profile, conformance, installation approval, installed
publication, enablement, disablement change, removal, update, download or public
registry work.

No dependency, package format, candidate schema, CLI envelope schema, capability
identity, Tether syntax, OCaml, release, tag or version change.
