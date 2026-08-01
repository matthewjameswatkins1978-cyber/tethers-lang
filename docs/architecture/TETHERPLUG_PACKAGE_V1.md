# Tethers `.tetherplug` Package Format v1

Status: J18D candidate, pending Lucy package review
Package format version: 1
Implementation: Not authorised

## 1. Purpose and Container

`.tetherplug` is a portable package presented to a Tethers host for inspection.
It may contain one local provider, related capability manifests, provider
payload, conformance material, documentation, assets, licences, notices, and
detached signatures. It is not an installed Plug, permission, policy,
credentials, runtime configuration, Tether Set, update channel, marketplace
listing, result, or Trail. Possession grants no authority.

Version 1 is a narrowly profiled ZIP-compatible archive with the `.tetherplug`
extension. Stored and deflated ordinary-file entries are allowed. Encrypted,
multi-disk, self-extracting, Zip64, symbolic-link, hard-link, device,
junction/reparse-point, alternate-data-stream, nested-package, executable
comment, and executable metadata features are forbidden. ZIP compression,
timestamps, ordering, and platform metadata do not define semantic identity.
An unpacked directory is a development tree, not an installable package.

## 2. Canonical Root and Paths

```text
plug.json
provider/
manifests/
tests/
docs/
assets/
licenses/
signatures/
```

`plug.json` exists exactly once at the archive root. `provider/` contains the
provider payload and `manifests/` contains at least one capability manifest.
Tests reserve J18F conformance material. Docs, assets, licences, and signatures
are optional. There is exactly one provider declaration; it may expose several
capabilities. No file exists outside these root areas and unnecessary empty
directory entries are omitted.

Every archive path is relative, uses `/`, and consists only of lowercase ASCII
segments matching `[a-z0-9][a-z0-9._-]*`. Spaces, empty segments, `.` and `..`,
leading slash, drive letters, colon, NUL/control characters, trailing dots or
spaces, and Windows device names are rejected. Duplicate, case-insensitive,
normalised, file/directory-prefix, escaping, and over-limit paths fail closed.
Payload bytes may be Unicode; package paths remain boring and deterministic.

## 3. Strict `plug.json`

`plug.json` is UTF-8 strict JSON without BOM, duplicate keys, unknown fields,
invalid I-JSON, or over-limit values. It uses RFC 8785 JCS where canonical
ordering is required. Its conceptual sections are:

**Package metadata:** required `package_format_version` (`"1"`), stable
lowercase dotted `package_id`, strict `MAJOR.MINOR.PATCH` `package_version`,
`display_name`, `description`, `publisher`, and `licence`. Display fields are
untrusted presentation. `tethers.file-tools` and `0.1.0` are illustrative
identifiers, not publisher proof.

**Compatibility:** required `socket_major`, `protocol_bindings`, and
`platforms`. The first candidate declares Socket major 1, MCP 2025-11-25,
local stdio, Windows, and x86_64. These are claims until host-checked; product
version never substitutes for package, Socket, binding, or protocol versions.

**Provider:** exactly one declaration with `provider_id`, `provider_version`,
`launch`, `working_directory`, and `capability_operation_namespace`. Provider
identity is distinct from package identity and the declaration does not prove
the launched process identity.

**Capabilities:** a non-empty ordered list of `capability_name`,
`capability_version`, `manifest_path`, `manifest_digest`, and
`provider_operation_name`. Entries sort by name then version. Duplicate
capability identities or provider operation names fail. Manifest paths remain
beneath `manifests/`; package fields cannot override manifest schemas, effects,
scopes, or behaviour.

**Payload index:** a complete path-sorted list of `path`, `sha256`,
`size_bytes`, and `role`. Roles are `provider_executable`, `provider_script`,
`capability_manifest`, `conformance`, `documentation`, `asset`, `licence`, and
`notice`. Every non-signature payload appears once, exists once, and is indexed
once. No unindexed payload is allowed. `plug.json` is not self-indexed.

## 4. Identity and Digests

Package identity, provider identity, capability identity, manifest identity, and
archive identity remain separate. A capability is still semantic `name +
version`; its accepted manifest has its own digest.

Each indexed file has a `sha256:` followed by lowercase hexadecimal payload-file
digest. The semantic package digest is the SHA-256 of the RFC 8785 JCS bytes of
the complete strictly validated `plug.json`, after checking canonical arrays,
all indexed paths, sizes, bytes, and payload digests. The index commits those
payload bytes semantically. The semantic digest is not stored in `plug.json`,
avoiding self-reference.

The host may also record a raw archive digest over the exact `.tetherplug` bytes.
Compression, ordering, or timestamps may change that digest without changing
semantic contents. The raw archive digest is not the semantic package digest.
Neither digest replaces a capability-manifest digest.

Package lineage is `package_id`; human release identity is `package_id +
package_version`; exact package identity also requires semantic package digest.
The same ID and version with different semantic digests is a conflict: refuse or
quarantine pending explicit review, never silently merge or select.

## 5. Provider Launch and Authority

The package may declare a packaged executable under `provider/`, or an indexed
provider script with an approved host interpreter requirement. The host resolves,
pins, and permits the interpreter and never installs it automatically.
Launch declarations use an ordered argument array and package-relative working
directory. They contain no shell command string, concatenation, interpolation,
`cmd /c`, PowerShell `-Command`, or unbounded user fragment. Launch never occurs
during inspection. Sandbox details belong to J18G.

Packages contain no passwords, keys, tokens, cookies, private keys,
credential-profile values, approved scopes, effective policy, environment
values, machine paths, or installation IDs. The host owns credentials,
configuration, bindings, permissions, and generated runtime configuration.

Capability manifests remain separate authoritative documents. Each has its own
format, identity, JCS/SHA-256 digest, schemas, effects, and scopes. The host
validates and pins the manifest during installation and later pins the exact
binding. Any mismatch fails closed. Existing retry fields do not authorise
automatic retry; no automatic retry exists without explicit end-to-end proof.

## 6. Detached Signatures

`signatures/` reserves detached signature envelopes over the semantic package
digest. Signatures are optional, excluded from semantic identity, and may be
added or removed without changing semantic package contents. They identify the
digest claimed, remain untrusted until cryptographic and institutional checks,
and prove neither permission nor safety. J18G defines algorithms, key identity,
trust, revocation, envelope, and publisher policy. Malformed signatures are
reported and ignored or quarantined according to host policy; they never bypass
inspection or approval.

## 7. Inspection and Extraction

Inspection performs no execution. The host records raw archive digest and size,
reads the ZIP directory without extraction, applies archive and path rules,
finds exactly one `plug.json`, strictly validates it and its ordering, compares
archive entries with the complete index, checks every size and SHA-256, computes
semantic package digest, checks compatibility, validates every capability
manifest, inspects signatures as evidence, and produces an inspection report.

After successful inspection and explicit continuation, extraction occurs only
into a new host-owned quarantine directory. The host rechecks destination paths,
rejects links and reparse points, chooses restrictive permissions, verifies
bytes again, ignores ZIP executable trust, and never launches from the archive.
Final installation is a host-owned transaction after J18F conformance and
approval. Exact sandbox and rollback mechanics belong to J18G.

## 8. Package versus Installed Plug

A package contains portable claims and payload. An installed Plug is host-owned
state containing semantic and raw archive digests, signature evidence,
extraction location, exact launch identity, verified manifests and live
bindings, approved scopes and policy, credential references, conformance
evidence, enabled/disabled and health state, and installation/approval Trail
evidence. None is written back to the package. The package never edits host
configuration; the host generates runtime configuration and binding records.

## 9. Upgrade, Removal, and Refusal

V1 provides identity, not updates. Every new version is separately inspected,
hashed, tested, approved, and bound. There are no automatic downloads, updates,
dependency installation, silent rebinding, or migration to latest. Removal acts
on installed state: prevent calls, stop the provider, remove active bindings and
payload, preserve package/manifest evidence and historical Trails, and handle
credentials by explicit choice. Deleting an archive is not removal.

Refuse or quarantine malformed or unsupported archives, unsafe/colliding paths,
missing/duplicate/unindexed entries, size or digest mismatches, invalid package
identity/version, duplicate capabilities/operations, incompatible platform,
Socket or binding, missing provider payload, unsupported launch, manifest
failure, same-version digest conflict, or resource-limit exhaustion.

## 10. First Envelope and Deferred Scope

The first envelope is Windows x86_64, one local provider, several related
capabilities, Socket 1, MCP 2025-11-25 over local stdio, a packaged executable
or approved interpreter, and host-owned inspect/test/approve/enable/disable/
remove. File Tools is the reference Plug and PDF Tools the competition Plug.
J18D authorises no implementation.

Dependencies, remote providers or payloads, registries, marketplaces, updates,
deltas, multi-provider packages, untested operating systems, native selection,
OAuth, drivers, unrestricted shell, payment, Plug-to-Plug dependencies,
embedded Tether Sets or policy, executable signature trust, and install scripts
are deferred. J18F defines conformance contents and semantics; J18G defines
security and sandbox details.

## 11. Illustrative Package (Not a Schema)

Illustrative only. Not a machine schema and not an implementation fixture.

```text
tethers.file-tools@0.1.0
plug.json
provider/file-tools.exe
manifests/file.move-v1.json
manifests/file.read-v1.json
tests/conformance.json
docs/readme.md
licenses/mit.txt
```

The optional `signatures/` area appears only when the package contains detached signature material; empty directory entries are omitted.

An illustrative fragment is deliberately non-executable:

```json
{
  "package_format_version": "1",
  "package_id": "tethers.file-tools",
  "package_version": "0.1.0",
  "display_name": "File Tools",
  "description": "Illustrative package metadata",
  "publisher": "Example publisher",
  "licence": "MIT",
  "socket_major": 1,
  "protocol_bindings": [{"protocol": "MCP", "version": "2025-11-25", "transport": "stdio"}],
  "platforms": [{"os": "windows", "architecture": "x86_64"}]
}
```

The fragment omits required provider and payload-index fields and is not a
complete package document.

## 12. Acceptance and J18H Boundary

J18D passes review only when the ZIP profile, strict root document, safe paths,
single provider, indexed payloads, three digest distinctions, detached-signature
limits, no-execution inspection, quarantine extraction, host-owned installation,
manifest authority, refusal rules, preserved Trails, and small first envelope
are explicit. J18H paper validation remains required before final freeze.
