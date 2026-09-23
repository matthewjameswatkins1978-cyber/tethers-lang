# Tethers Versioning Policy

Status: living contract
Updated: 2026-09-23

Tethers has several legitimate version axes. They are not interchangeable and
must not be collapsed into one product number.

## Version axes

| Axis | Current value | Meaning and compatibility owner |
| --- | --- | --- |
| Product version | `0.8.0` | Practical native product/release identity; release control owns its public lifecycle; R2 added the public `tethers gate` surface and `tethers.authority/1` |
| Human Tether language | `0.1` | Source-language semantics defined by `tethers-0.1/SPEC.md`; Core owns meaning |
| Core/wire protocol | `0.1` | Tethers request/response protocol; strict version checks at the boundary |
| CLI envelope/discovery | `tethers.cli/1` | Machine-readable native host command envelope and discovery shape |
| Authority protocol | `tethers.authority/1` | Authority Gate request/response protocol over local stdio; independent of the product version; exact schema match at the boundary, never inferred from product version |
| Plan | `tethers.plan/1` | Stable machine-facing side-effect-free Plan projection |
| Trail/receipt | `tethers.trail/1` | Host-owned causal evidence and its bounded receipt projection |
| Capability manifest | `1.0` | Manifest schema/contract format, distinct from capability version |
| Capability identity | name plus explicit capability version | Operation compatibility and exact provider/manifest binding |
| Plug package | format `1` | `.tetherplug` package structure and generated payload evidence |
| Plug Socket | major `1` | Provider protocol/socket compatibility boundary |
| MCP protocol support | Tethers currently targets `2025-11-25`; external latest is `2026-07-28` | Transport/adapter compatibility, owned by the MCP adapter; R4 must choose support explicitly |
| Persistent host state/configuration | `tethers.project/1` and format-specific versions | Host data durability and migration owner; no silent reset or downgrade |
| Portable Workbench | `0.2.2` | Separate small authority façade with its own compatibility promise |
| Benchmarker | `tethers.benchmarker/1`, tool `0.5.0` | Verification-tool identity, not product identity |

## Product versus semantic compatibility

Product identity answers “which Tethers release is this?” Semantic compatibility
answers “does this preserve the meaning of a supported Human Tether and Core
contract?” A product may carry an older language or wire version while adding
compatible host functionality. A package, transport, or storage format may
change independently when its explicit version and migration rules say so.

| Compatibility kind | Rule |
| --- | --- |
| Product identity | Use SemVer for the public product once the 1.0 API is defined |
| Semantic compatibility | Preserve the defined meaning of supported language/Core constructs |
| Wire compatibility | Negotiate or validate explicit protocol/schema versions; do not infer from product version |
| Package compatibility | Validate package/socket/manifest formats and digests before use |
| Storage compatibility | Use explicit format versions and protected migrations |
| Transport compatibility | Negotiate the external transport/protocol revision separately from Tethers semantics |

## 1.x rule

> Tethers 1.x continues to understand the public language and protocol versions
> declared supported at 1.0. Incompatible successors receive a new explicit
> version rather than silently changing the meaning of an existing version.

Human Tether language `0.1` is not renumbered merely because the product reaches
1.0. An incompatible successor must be named and negotiated explicitly.

Backward-compatible public additions use a minor product release; compatible
bug fixes use a patch release; incompatible public changes use a major release,
consistent with [SemVer 2.0.0](https://semver.org/). Major version zero remains
pre-1.0 development and does not itself promise a stable public API.

## Machine-readable discovery

`tethers describe --json` is the preferred discovery point for native product
and supported-surface information. The current implementation exposes the
`tethers.cli/1` envelope, product version, and feature discovery. A later
bounded audit should ensure it also exposes, or links to, the supported Human
Tether language, Core/wire protocol, Plan, Trail, Plug package/socket, and MCP
support revisions without implying more than the product actually guarantees.

This policy does not change CLI semantics in R1.

## Versioning rules for maintainers

- Every machine contract has an explicit schema or protocol identity where it
  crosses a process, package, persistence, or public CLI boundary.
- Internal OCaml module names, Rust module shapes, helper records, test output,
  and diagnostic prose have no compatibility promise unless explicitly listed.
- A producer must not emit a newer or incompatible version under an old
  identity.
- A consumer must reject unsupported major/incompatible versions rather than
  guess, partially interpret, silently downgrade, or silently reset them.
- Changes to Action ordering, Plan meaning, Trail truth, authority, replay, or
  uncertainty are semantic changes even if a JSON field shape stays the same.
- Release manifests, checksums, provenance, and package identity bind an
  artifact to the exact source and format that produced it.
