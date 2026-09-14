# Tethers Road to 1.0

Status: living programme contract
Updated: 2026-09-14
Starting product: Tethers 0.7.0
Accepted preflight main: `cd6cc23f2f0d7248bb31a5b650862f26699b2a65`

## Meaning of 1.0

> **Tethers 1.0 means the consequential boundary is stable.**

That means users can understand what Tethers promises at the boundary between
intent and real effects: deterministic meaning, typed capabilities, explicit
authority, bounded execution, honest outcomes, compatibility rules, and durable
evidence. It does not mean that every planned interface exists, that
development stops, that every internal version becomes 1.0, or that every
experimental interface becomes permanent.

Tethers 1.0 is a stability contract, not a feature quota. The current preflight
found no Core correctness blocker. The remaining work is primarily contract
audit, portability, release control, compatibility, security/recovery proof,
and real workload evidence.

## Release train

| Release | Promise |
| --- | --- |
| 0.7 | Proven practical foundation: deterministic Core, host boundary, Plugs, replay/Trail, Agent Core, and Windows x64 runtime |
| 0.8 | Contract, portability, verification, MCP, and release hardening |
| 0.9 | Compatibility rehearsal, security/recovery gauntlet, real workloads, and release rehearsal |
| 1.0 | Stable supported public platform at the consequential boundary |

## Programme packages

These are outcome and gate descriptions, not a claim that later implementation
details have already been designed.

| Package | Outcome | Acceptance gate |
| --- | --- | --- |
| R1 — 1.0 Contract Freeze | Public meaning, version axes, compatibility, support, deprecation, non-goals, and classifications are explicit | Contract documents agree and no implementation semantics change |
| R2 — Verification and Release Control | Aggregate verification is trustworthy, the historical checkpoint mismatch is repaired, warning baseline is recorded, and compatibility corpus machinery exists | One authoritative release-verification route distinguishes product, optional, and external results |
| R3 — Linux Native Parity | Linux x86-64 host and engine behaviour are proven | Code, build, package, clean install, runtime, supervision, recovery, CI, release, and documentation gates pass |
| R4 — MCP Current-Protocol Pass | Tethers’ MCP adapter is deliberately compatible with the current supported external revision | Version negotiation, transport, tool, security, and regression evidence pass without creating a second evaluator |
| R5 — Unified Release Pipeline | Supported releases are reproducible, provenance-aware, checksummed, documented, and clean-machine tested | Release artifacts, manifests, SBOM/provenance, permissions, and verification agree |
| R6 — Compatibility and Migration | The 0.7-to-1.0 corpus and any required migrations are exercised | Existing supported inputs remain valid or receive explicit, protected migrations |
| R7 — Security and Recovery Gauntlet | Trust drift, approval, replay, uncertain effects, persistence failure, provider failure, and recovery boundaries are challenged | No silent authority bypass, duplicate effect, false success, or destructive recovery is accepted |
| R8 — Real Workload Proof | Tethers is used on representative consequential local workflows | Receipts and failure/uncertainty evidence show useful, bounded operation |
| R9 — 0.9 Full Release Rehearsal | The release process is run end to end against the intended support matrix | Rehearsal artifacts, installation, upgrade, rollback, and compatibility results are reviewable |
| R10 — 1.0 Release Candidate and Final Audit | Final public contract, support, provenance, security, recovery, and documentation audit | All 1.0 blockers are closed and requirements have evidence or an explicit owner decision |

## Public-contract inventory

This table is a planning inventory, not an automatic stability promise. A
surface becomes stable only when its meaning and compatibility evidence are
audited by the relevant package.

| Surface | Current identity | Owner | Current stability | Intended 1.0 status | Compatibility mechanism | Next evidence |
| --- | --- | --- | --- | --- | --- | --- |
| Human Tether source | Language `0.1` | Core | Versioned | Versioned supported language | Explicit language version; fail closed on unsupported versions | R6 corpus and language audit |
| OCaml semantic Core | Internal typed Core | Tethers Core | Internal | Internal | No module-shape promise | Core differential/regression evidence |
| Core JSON protocol | Protocol `0.1` | Core/host boundary | Versioned | Versioned | Explicit protocol version and strict decoding | R6 request/response corpus |
| ProgramDigest/canonical identity | Canonical Format V2, `tethers:v2:sha256:*` | Core | Versioned | Stable meaning with explicit format identity | Digest prefix and frozen semantic encoding | R2/R6 differential corpus |
| Native Rust host CLI | Product `0.7.0`; CLI envelope `tethers.cli/1` | Host | Versioned but needs 1.0 audit | Stable versioned machine surface | Command/schema versions and structured exit classes | R2/R6 CLI corpus |
| CLI JSON schemas | `tethers.cli/1`, discovery/features JSON | Host | Versioned but needs 1.0 audit | Versioned | Schema field/version negotiation | R6 golden outputs |
| Configuration | `tethers.project/1` and runtime configuration formats | Host | Versioned | Versioned with protected migration | Explicit format version, validation, migration | R6 migration tests |
| Plan | `tethers.plan/1` | Core/host | Versioned but needs 1.0 audit | Stable versioned projection | Schema/version and ordered Action semantics | R2/R6 corpus |
| Trail and receipts | `tethers.trail/1`; receipt projection over Trail | Host | Versioned but needs 1.0 audit | Stable evidence projection, versioned | Schema/version; append-only evidence rules | R6 and R7 recovery corpus |
| Policy and scope | Host-owned policy and binding-specific scope | Host | Versioned but needs 1.0 audit | Stable authority boundary | Explicit policy/scope formats; fail closed | R7 authority matrix |
| Approval identity | Host approval proof and one-shot decisions | Host | Versioned | Versioned | Explicit approval format and binding | R7 replay/approval tests |
| Capability manifests | Manifest format `1.0` | Host/Plug contract | Versioned | Versioned | Manifest format, capability name/version, digest, provider pins | R2/R6 manifest corpus |
| Provider binding | MCP stdio and binding pins | Host | Versioned but needs 1.0 audit | Versioned | Binding kind/version, server/tool/provider identity and digest | R4/R7 drift tests |
| Plug package | Package format `1` | Plug host | Versioned | Stable versioned package contract | Package format, digest, payload index, explicit lifecycle | R5 clean-package verification |
| Plug Socket | Socket major `1` | Plug host/provider | Versioned | Stable major boundary | Socket major and explicit protocol binding | R6 compatibility corpus |
| Installation metadata | Host-owned installed Plug state | Host | Versioned but needs 1.0 audit | Versioned | Identity/digest/version and migration rules | R5/R7 lifecycle recovery |
| Persistent host state | Host data/config/workspace state | Host | Versioned but needs 1.0 audit | Versioned with migrations | Format version, validation, protected migration | R6/R7 migration and backup tests |
| Replay/idempotency state | Durable intent and replay records | Host | Versioned but needs 1.0 audit | Stable safety boundary, versioned storage | Explicit identity, outcome, uncertainty, and migration | R7 crash/recovery corpus |
| MCP adapter | Tethers MCP over stdio, currently `2025-11-25` | Core adapter | Versioned/experimental | Versioned adapter; current revision chosen explicitly | Negotiated MCP revision; one canonical evaluator | R4 current-protocol pass |
| Portable Workbench | Product/workbench `0.2.2` | Portable host | Versioned | Separate supported surface with its own promise | Workbench version and policy decision contract | R6 compatibility audit |
| Release archive layout | Windows x64 0.7 full-runtime archive | Release control | Versioned but needs 1.0 audit | Stable per supported target | Manifest, checksums, provenance, documented layout | R5/R9 clean-machine rehearsal |
| Release manifest | Current generated package manifest | Release control | Versioned but needs 1.0 audit | Stable verification evidence | Source SHA, artifact hashes, toolchain/build identity | R5 provenance and reproducibility |

## 1.0 non-goals

The following are not on the 1.0 critical path unless real workload evidence
later proves one is required:

HQ; full Tethers Shell; Presentation Contract implementation; full Guidance or
Semantic Draft UX; LSP; public Plug marketplace or registry; automatic Plug
downloading or updates; remote MCP deployment; Streamable HTTP requirement;
OAuth implementation; hosted or distributed Tethers; unrestricted shell
capability; embedded LLM runtime; global scheduler; new canonicalisation
generation; new Human Tether syntax without a demonstrated blocker; macOS or
ARM64 release; streaming framework; and a generic long-running job framework.

These may be useful future work. They are not silently promised by 1.0.

## Release classifications

| Class | Meaning |
| --- | --- |
| BLOCKER | Known defect that prevents a truthful 1.0 release, such as Core incorrectness, authority bypass, data corruption, false success, or a supported target that cannot run |
| REQUIREMENT | Work deliberately required before 1.0, without implying that the current product is broken |
| DEBT | Worth improving but not sufficient by itself to withhold an otherwise honest release |
| EXTERNAL | Failure outside current product correctness; it can still block the release process until tooling is repaired |
| FUTURE | Useful work outside the 1.0 promise |

Current examples are Linux native parity and release automation (REQUIREMENT),
licence resolution (REQUIREMENT plus OWNER DECISION), existing non-fatal Clippy
warnings (DEBT), and the historical GARY checkpoint mismatch (EXTERNAL).
The preflight Core blocker is NONE.

## Verification policy

There must eventually be one trustworthy aggregate release-verification command.
The current `just verify` route is not yet that command because the historical
packet checker stops before product verification. R2 owns that repair.

Until then, every report must distinguish:

- product verification;
- optional integration results;
- external tooling results.

Each result is `PASS`, `FAIL`, `SKIPPED WITH REASON`, or `NOT APPLICABLE`.
“Mostly green” is not an acceptance state.

## Research evidence — 2026-09-14

### Semantic Versioning

[Semantic Versioning 2.0.0](https://semver.org/) defines 1.0.0 as the point at
which the public API is defined; compatible additions use MINOR, compatible
bug fixes use PATCH, and incompatible public changes use MAJOR. This supports
the 1.0 stability boundary and the explicit version-axis policy in
`VERSIONING.md`.

### MCP

The [current MCP specification](https://modelcontextprotocol.io/specification/latest)
resolves to revision `2026-07-28`. Tethers currently implements and fixtures
`2025-11-25`. The latest revision changes the base presentation toward
stateless, self-contained requests with per-request metadata, while Tethers'
current adapter uses the earlier connection-scoped initialize/capability
negotiation model. Its transport and tools material also add current metadata,
tool-result, and extension rules. R4 is therefore an explicit compatibility
pass. R1 does not upgrade MCP or create a second evaluator.

### Licensing

The [Apache License 2.0 text](https://www.apache.org/licenses/LICENSE-2.0.html)
includes copyright and patent grants plus redistribution conditions. Apache's
[application guidance](https://www.apache.org/legal/apply-license.html)
describes placing the full text in `LICENSE`, reviewing `NOTICE` and
attribution obligations, and handling source/package notices. Apache-2.0
remains the recommended candidate, but licence selection is still an owner
decision and no licence is added by R1 without approval.

### GitHub release provenance

GitHub's [artifact attestation overview](https://docs.github.com/en/actions/concepts/security/artifact-attestations)
and [attestation how-to](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations)
describe signed build-provenance claims tied to repository, workflow, commit,
and triggering event, and support SBOM attestations verified with GitHub CLI.
R5 should select the least complex reproducible implementation, with explicit
workflow permissions, checksums, manifest/source binding, provenance, SBOM
policy, and clean-machine verification.
