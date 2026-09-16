# Tethers / Resolve01 Reclassification

Status: `R0 AUDIT - FROZEN FOR R1`

P0-P4b is retained as historical implementation. Its original execution
direction is superseded, but the implementation is not called invalid. The
following classification is an audit of the accepted Tethers tree, not a plan
to delete or refactor it in R0.

## Classification vocabulary

| Label | Meaning |
| --- | --- |
| `KEEP-AS-GENERIC` | Useful independent of the Tethers reference Host and suitable as a contract or reusable concept. |
| `KEEP-AS-REFERENCE-HOST` | Correctly belongs to the supplied Rust Host implementation. |
| `REUSE-IN-RESOLVE-HOST` | A candidate for later use by Resolve, subject to a new explicit interface and ownership review. |
| `REQUIRES-REDESIGN` | The current integration shape assumes the reference Host owns an external application's lifecycle. |
| `HISTORICAL-ONLY` | Retain as evidence of the earlier packet or implementation line; do not treat it as current architecture. |
| `DOCS-SUPERSEDED` | Historical wording that must not guide new implementation. |

## Current implementation

| Area | Classification | Rationale |
| --- | --- | --- |
| `tethers-0.1/engine-ocaml/` | `KEEP-AS-GENERIC` | Core parsing, validation, evaluation and Plan semantics remain application-agnostic. |
| `tethers-0.1/SPEC.md` and Core protocol fixtures | `KEEP-AS-GENERIC` | Precise language and protocol authority; no Resolve concepts enter it. |
| `tethers-0.1/host-rust/src/resolver.rs` | `KEEP-AS-REFERENCE-HOST`; `REUSE-IN-RESOLVE-HOST` candidate | Exact capability and provider resolution is generic in meaning, but the current implementation is owned by the reference Host. |
| `tethers-0.1/host-rust/src/manifest.rs` and `installation_trust.rs` | `KEEP-AS-GENERIC`; `REUSE-IN-RESOLVE-HOST` candidate | Manifest identity, trust evidence and validation are reusable contracts; storage and lifecycle remain Host-owned. |
| `tethers-0.1/host-rust/src/runtime_config.rs` and `configured_runtime.rs` | `KEEP-AS-REFERENCE-HOST` | Configuration loading and prepared runtime belong to the supplied Host. |
| `tethers-0.1/host-rust/src/policy.rs` | `KEEP-AS-REFERENCE-HOST`; `REUSE-IN-RESOLVE-HOST` candidate | Policy is a Host authority. Resolve may adopt an explicitly bounded equivalent, not inherit reference-host state implicitly. |
| `tethers-0.1/host-rust/src/approval.rs` | `KEEP-AS-REFERENCE-HOST`; `REUSE-IN-RESOLVE-HOST` candidate | Approval freshness and proof shape may be reusable, but approval ownership stays with the consuming Host. |
| `tethers-0.1/host-rust/src/operational_scope.rs` | `KEEP-AS-GENERIC`; `REUSE-IN-RESOLVE-HOST` candidate | Scope evidence is a transport-independent contract when supplied by the Host. |
| `tethers-0.1/host-rust/src/dispatch.rs` | `KEEP-AS-REFERENCE-HOST` | Durable intent, dispatch ordering and Trail integration are reference-host lifecycle machinery. |
| `tethers-0.1/host-rust/src/replay.rs`, `replay_runtime.rs`, `replay_windows.rs` | `KEEP-AS-REFERENCE-HOST`; `REUSE-IN-RESOLVE-HOST` candidate | Replay reasoning and identity evidence may inform another Host, but replay state cannot be silently shared. |
| `tethers-0.1/host-rust/src/outcome.rs` | `KEEP-AS-GENERIC`; `REUSE-IN-RESOLVE-HOST` candidate | Success, failure and uncertainty are useful Host contract outcomes. |
| `tethers-0.1/host-rust/src/result_anchor.rs` | `KEEP-AS-REFERENCE-HOST`; `REUSE-IN-RESOLVE-HOST` candidate | Result Anchors are a Tethers host evidence mechanism; Resolve must decide its own consumption boundary. |
| `tethers-0.1/host-rust/src/provider.rs`, `stdio_provider.rs`, `installed_provider_executor.rs` | `KEEP-AS-REFERENCE-HOST` | Concrete provider admission, supervision and invocation implement the reference Host. |
| `tethers-0.1/host-rust/src/plug_pack.rs`, package and installation modules | `KEEP-AS-GENERIC`; `REUSE-IN-RESOLVE-HOST` candidate | Plug packaging and contract evidence are independent, while installation state is Host-owned. |
| `tethers-0.1/host-rust/src/host_execution.rs` | `KEEP-AS-REFERENCE-HOST` | The shared execution boundary is the reference Host's lifecycle, not a universal Resolve boundary. |
| `tethers-0.1/host-rust/src/application.rs` and `run_command.rs` | `KEEP-AS-REFERENCE-HOST` | Reference-host application orchestration and CLI. |

## P0-P4b integration line

| Area | Classification | Rationale |
| --- | --- | --- |
| `resolve_guard.rs` | `REQUIRES-REDESIGN`; generic proof concepts reusable | The typed guard seam is coupled to the reference-host execution insertion point; opaque evidence and exact comparison may be retained only behind a Host-owned interface. |
| `resolve_transport.rs` | `REQUIRES-REDESIGN`; bounded validation reusable | Live Resolve transport was introduced beneath the reference Host's P2 seam. A future Resolve Host needs an explicit boundary rather than reference-host ownership. |
| `resolve_outcome.rs` | `REUSE-IN-RESOLVE-HOST` candidate; current integration requires redesign | Delivery identity, idempotency and bounded evidence are useful, but outcome ownership belongs to the Host that owns the execution. |
| `lantern_authority.rs` | `REQUIRES-REDESIGN` | The bridge proves authority consultation and receipt delivery, not a general external-Host execution contract or Lantern memory provider. |
| P0-P4b Rust changes as a whole | `HISTORICAL-ONLY` for integration direction | Preserve all code and commits; do not treat the old insertion point as mandatory architecture. |
| `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P0_SEAM_AUDIT.md` | `HISTORICAL-ONLY` | Records the original P0 seam and must be read with this R0 correction. |
| P1/P2/P3/P4/P4b architecture notes and worker notes | `HISTORICAL-ONLY` | Evidence of accepted work against the earlier contract; not rewritten or used as current universal ownership. |

## Existing contracts and examples

| Area | Classification | Rationale |
| --- | --- | --- |
| `docs/CAPABILITY_BRIDGE.md` | `KEEP-AS-GENERIC` | Capability, manifest, scope, binding and provider contract authority. |
| `docs/architecture/TETHERPLUG_PACKAGE_V1.md` | `KEEP-AS-GENERIC` | A Plug is a package/binding, not an application lifecycle. |
| `reference-plugs/` | `KEEP-AS-GENERIC` | Reference and adversarial provider material; no Resolve-specific semantics. |
| `docs/architecture/TETHERS_LANTERN_KEEPER_CANONICAL_ARCHITECTURE.md` | `KEEP-AS-GENERIC` with R0 ownership reading | The distinction between Tethers coordination and Lantern memory remains useful; application Host ownership now controls execution interpretation. |
| `docs/TETHERS_LUCY_NOTES.md` and old Lantern examples | `DOCS-SUPERSEDED` where they imply mandatory Tethers execution ownership | Preserve history, but future work follows the R0 Host model. |

## Later packet boundary

R1 must define the smallest explicit interface by which Resolve, as Host and
executor, consumes useful Tethers machinery. It must not begin by moving the
reference Host's complete lifecycle into Resolve or by adding Resolve concepts
to Core. Lantern Plug work remains paused until the Capability and Host boundary
is explicitly designed.
