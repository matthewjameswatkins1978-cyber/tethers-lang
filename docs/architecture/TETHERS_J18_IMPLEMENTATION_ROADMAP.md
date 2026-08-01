Status: J18I candidate, pending Lucy roadmap review
Architecture freeze: a5fd63593a9d9acd397030ecd2e27b4f318c87fd
Implementation: Not authorised

# Tethers J18 First Plug Kit Implementation Roadmap

This is an executable sequencing proposal, not implementation authority. It
turns accepted J18B-J18H contracts into vertical, reversible packets while
retaining the released Tethers 0.2 path. No packet below is authorised merely
by being listed.

## Frozen route and compatibility

Tethers Core remains deterministic and application-agnostic. The host owns
trust, policy, credentials, dispatch, canonical outcomes, replay, event
admission, conformance and Trail. Providers own vendor translation. Socket
semantics, protocol binding and byte transport remain separate. Action, Query
and Anchor are first-programme classes; Job, Stream and Human Task remain
reserved. Attempted operation outcomes remain exactly `succeeded`, `failed`, and
`uncertain`; unattempted is not an outcome; no automatic retry exists.

The Plug Kit is introduced through a new host-owned path and stable seam. The
existing 0.2 runtime configuration, manifests and fixtures are evidence and
adapters where sound, not silently reinterpreted as `.tetherplug` v1. No user
0.2 configuration is mutated, no legacy path is removed, and the `v0.2.0` tag
never moves. Any migration or dual-read format requires its own reviewed packet.

## Current implementation inventory

| Component | Exact current role | Decision | Known gap / placement |
|---|---|---|---|
| `main.rs` | Binary owns module declarations, argument compatibility, coordinator, event admission, evaluation and dispatch orchestration | extract behind stable seam | Keep CLI/coordinator thin; forbid broad rewrite; Milestone 1 |
| `lib.rs` | Exposes only `child_process`, `cli`, and `engine_stdio` foundations | extend under frozen semantics | Export the smallest application service/socket surface; Milestone 1 |
| `runtime_config.rs` | Strict J12 config for Tether sources, command/args stdio bindings, reviewed manifest paths and pins | reuse as 0.2 adapter | Does not represent installed Plug identities; retain legacy path |
| `configured_runtime.rs` | Loads immutable prepared runtime, verifies manifests, builds provider launch plans and scope bindings | reuse unchanged for parity; extract seam | Preparation is file/config based, not installed-registry based |
| `manifest.rs` | Duplicate-key rejection, strict manifest parsing, canonical digest and semantic validation | reuse unchanged for legacy; supersede only for Plug package model | Current format is not J18D/J18E `plug.json` |
| `trusted_store.rs` | In-memory verified-manifest identity/digest indexes with atomic conflict checks | reuse as pure validation evidence; extend behind store seam | No publisher trust or durable package trust store |
| `provider.rs` | Admits verified manifests against configured provider identity and capability pins | extend under frozen semantics | No installed Plug/provider identity separation |
| `resolver.rs` | Resolves admitted exact capability against host availability snapshot | reuse unchanged in first parity slice | Needs adapter from installed Plug binding later |
| `policy.rs` | Resolves declared, admitted, available capabilities to allow/ask/deny/unavailable | reuse unchanged | Plug policy binding and scopes need a later adapter |
| `approval.rs` | In-memory one-shot Ask proof bound to evaluation, action, arguments, manifest and provider | reuse unchanged initially | Durable/user workflow is later; approval never grants trust |
| `dispatch.rs` | Intent-first durable boundary before provider execution; typed `DispatchReadyAction` | reuse unchanged in parity | Must be reached by installed Plug path without bypass |
| `stdio_provider.rs` | Retained MCP stdio session, initialize, tools/list, invoke, close, fixed request IDs and supervised child | extract semantic Socket boundary; extend pagination | Full J18C pagination, duplicate/schema drift and catalogue invalidation absent |
| `host_execution.rs` | Typed J13B execution service retaining engine/provider sessions and policy/dispatch/outcome flow | extract behind application seam; reuse internals | Public library boundary is not yet complete |
| `outcome.rs` | Pure deadline/diagnostic redaction and exact three attempted outcomes | reuse unchanged | Must remain host-owned for all Plug paths |
| `replay_runtime.rs` | Runtime replay authority trait and lazy file-backed admission | reuse unchanged for operation replay | Separate durable external-event admission required |
| `replay_windows.rs` | Windows reparse-safe, locked, crash-aware replay ledger and causal generation checks | reuse unchanged | It is not the Anchor admission store or Trail |
| `result_anchor.rs` | One host-created Result Anchor after known executor outcome | reuse unchanged | Plug lifecycle must publish it without conflating admission |
| `event_admission.rs` | Process-local exact-ID gate and generation 0..8 limit | reuse for J11 safety; extend only via separate store seam | No restart-durable external admission |
| `event_queue.rs` | Process-local FIFO Result Anchor queue, serial and non-retrying | reuse unchanged | Not durable external-event authority |
| `child_process.rs` | Windows Job Object supervision, bounded protocol lines, stderr tail, timeout-aware reads | reuse and extend launch profiles | Supervision is not hostile-code isolation; no sanitized environment |
| `engine_stdio.rs` | Retained supervised MCP engine session for `tethers.validate` and `tethers.evaluate` | reuse unchanged | Socket seam must not alter engine semantics |
| CLI/check/run commands | Clap `check`, `run`, `trail`, hidden replay/event probes and legacy route; PowerShell scripts drive verification | preserve; add only reviewed Plug CLI later | No inspect/install/enable/conformance/remove workflow |
| manifests/configs/fixtures | J12 JSON manifests, runtime configs, MCP transcripts, local file provider and fixture provider | reuse as parity/adapters/evidence | Never relabel them as `.tetherplug` v1 |
| Rust/OCaml/integration tests | Rust unit/integration suites, OCaml engine tests, PowerShell verification and MCP transcripts | preserve and extend per milestone | No Plug package, trust, provider or lifecycle tests yet |

### Confirmed gaps

There is no explicit application Socket seam, installed-Plug registry, package
inspector/quarantine installer, semantic package signature/trust implementation,
sanitised provider environment, conformance evidence invalidation store,
supervised-versus-isolated launch profile, Credential Manager profile store,
durable external-event admission authority, installed Plug Action/Query/Anchor
machine contract, packaged File/PDF reference provider, or Plug lifecycle CLI.
The current discovery path does not prove complete pagination, duplicate
detection, schema drift and catalogue invalidation. No arbitrary third-party
hostile-code containment is present.

## Milestone 1: Socket seam and 0.2 parity

**Result:** The existing host execution path is callable through a reusable
application/library seam, while `check`, `run`, `trail`, replay, Result Anchor,
Trail, policy, outcomes and released behaviour remain unchanged. A semantic
Socket wraps retained MCP stdio sessions and proves full paginated discovery,
duplicate detection, schema drift and catalogue-change invalidation. No package
is installed.

**Contracts:** J18B host boundary; J18C Socket v1, MCP 2025-11-25 stdio,
initialize/discover/invoke/observe/close lifecycle, request identity, no batch,
serial actions and no retry; J18F outcomes, replay, events, conformance and
Trail; J11 dispatch intent and Result Anchor ordering.

**Reuse:** `host_execution.rs`, `configured_runtime.rs`, `policy.rs`,
`approval.rs`, `dispatch.rs`, `outcome.rs`, replay modules, Result Anchor,
event queue, `child_process.rs`, `engine_stdio.rs`, `stdio_provider.rs`,
existing CLI and all 0.2 tests. Extract only the coordinator/application seam
and Socket trait/implementation. Do not rewrite `main.rs` broadly.

**Anticipated modules:** application service interface, Socket session wrapper,
catalogue snapshot/invalidation value types, and transcript fixtures. No package
or installed identity store.

**Packets:** P1-SOCKET-PARITY (Green, first implementation packet): extract the
smallest host service and prove byte-for-byte/semantic 0.2 parity. P2-SOCKET-
BOUNDARY (Amber): place retained stdio behind semantic Socket operations without
changing engine/provider lifecycle. P3-DISCOVERY-CATALOGUE (Amber): pagination,
duplicate, schema drift and catalogue invalidation evidence.

**Owner/colour:** Luna/OpenCode Green for P1; Luna/OpenCode Amber for P2/P3;
Lucy review at the stable seam.

**Dependencies:** none beyond accepted J18 contracts and released 0.2 tests.

**Evidence:** Rust unit and integration regression, MCP transcript and
pagination tests, real Windows child-process tests, policy/approval tests,
Result Anchor/Trail ordering, full Rust and OCaml suites, and unchanged CLI
check/run demonstrations.

**Stop:** any 0.2 semantic or lifecycle regression, batch/retry behaviour,
engine/Core coupling, or need to redesign frozen Socket semantics.

**Rollback:** revert the seam and Socket commits; the legacy binary path remains
the release path until parity is accepted.

**Exclusions:** package parsing, installation, signatures, trust, credentials,
File/PDF providers, new CLI and durable external-event stores.

## Milestone 2: Package inspection, quarantine and installed identity

**Result:** A `.tetherplug` can be inspected without execution, strictly
validated, selected for Windows x86_64, semantically digested, and extracted only
to quarantine. Host-owned immutable installed material is disabled by default.
Package, installed, provider and capability identities remain distinct.

**Contracts:** J18D package paths, `plug.json`, payload index and semantic digest;
J18E identities/classes/effects/scopes; J18G install integrity, unsigned
developer mode and refusal. Installation grants no permission and no provider
is launched from Downloads, source ZIP or quarantine.

**Reuse:** `manifest.rs` parser/digest techniques, `trusted_store.rs` conflict
semantics, path validation patterns from `replay_windows.rs`, existing duplicate
JSON tests and host data-root conventions. Do not reinterpret legacy manifests.

**Anticipated modules/stores:** archive inspector, package metadata/value types,
payload selector, quarantine extractor, installed identity registry interface,
and immutable disabled installation record. Store schema is a separate packet.

**Packets:** P4-PACKAGE-INSPECT (Red): strict archive and semantic validation.
P5-QUARANTINE-PATHS (Red): safe extraction and Windows path/reparse handling.
P6-INSTALLED-IDENTITY (Amber): immutable disabled material and identity
separation, with registry schema packet preceding durable code.

**Owner/colour:** Codex Terra High/Red for archive and path attacks; DeepSeek Pro
V4/Amber for registry integration; Lucy architecture review.

**Dependencies:** Milestone 1 seam; J18D/J18G frozen documents; no trust launch.

**Evidence:** parser and duplicate-key tests; archive/path adversarial tests for
archive bombs, traversal, symlink/junction/reparse, case collision,
duplicate-entry and TOCTOU; package/install lifecycle tests; disabled-state and
no-execution proofs.

**Stop:** ambiguous archive semantics, path escape, digest/identity conflation,
execution during inspection/extraction, or need to change J18D.

**Rollback:** quarantine and registry code can be removed without touching the
legacy runtime or user 0.2 configuration.

**Exclusions:** Ed25519 verification, publisher trust, credentials, provider
launch, enablement, File/PDF payloads and public install CLI.

## Milestone 3: Trust, launch and conformance gate

**Result:** Host-owned publisher trust and Ed25519 package verification gate a
revalidated exact payload launch. The environment is constructed from scratch,
the existing Job Object supervision is retained, resource limits are bounded,
and the supervised profile is visibly labelled. Conformance evidence is pinned
to all required identities and does not enable a Plug.

**Contracts:** J18G signature, trust, revocation, developer mode, credentials,
sandbox honesty and conformance; J18C exact executable/argument launch and
Socket/protocol pins; J18D digest; J18F conformance invalidation. No AppContainer
or hostile-code isolation claim.

**Reuse:** `child_process.rs`, `stdio_provider.rs`, `provider.rs`,
`trusted_store.rs`, `resolver.rs`, existing process tests and MCP fixtures.
Extend launch configuration behind a profile boundary; do not weaken Job Object
supervision or use shell/PATH lookup.

**Anticipated modules/stores:** publisher trust store, revocation state,
conformance evidence store, launch profile/environment builder, payload
revalidation and optional Credential Manager metadata store. Each durable store
gets its own schema/crash/recovery packet.

**Packets:** P7-TRUST-SIGNATURE (Red): semantic digest, Ed25519 and publisher
trust/revocation. P8-LAUNCH-PROFILE (Red): exact launch, clean environment,
supervision and visible supervised label. P9-CONFORMANCE-GATE (Red): pinned
evidence, invalidation and disabled-state gate. P10-CREDENTIAL-METADATA
(Amber, optional): profile metadata only; no secret delivery in the first slice.

**Owner/colour:** Codex Terra High/Red for cryptography, Windows launch and
process boundaries; DeepSeek Pro V4/Amber for evidence integration; Lucy review.

**Dependencies:** Milestones 1-2; independent trust and store design packets.

**Evidence:** cryptographic vectors and trust lifecycle tests; real Windows
child/Job Object tests; clean-environment and exact-argument tests; package,
install, trust and conformance lifecycle; payload TOCTOU revalidation; proof
that passing conformance leaves the Plug disabled.

**Stop:** any claim that supervision is isolation, secret leakage, shell/PATH
execution, trust-on-first-use, conformance-as-permission, or frozen J18G change.

**Rollback:** disable the new launch gate and retain the accepted 0.2 path; trust
and conformance stores are removable before any Plug is enabled.

**Exclusions:** AppContainer completion, hostile-code containment, operational
credential delivery, network providers and third-party enablement.

## Milestone 4: File Tools Action/Query vertical slice

**Result:** One credential-free, no-network Windows File Tools provider is
packaged as `.tetherplug` and runs the complete installed-Plug path: inspect,
trust/developer approval, install, discovery, binding, policy, intent, one
invocation, canonical outcome, replay terminal, Result Anchor and Trail.
Bounded read/metadata Query and exact move Action refuse overwrite and enforce
path scope. Unattempted, failed and uncertain outcomes are demonstrated.

**Contracts:** J18C Socket/MCP binding; J18D package; J18E Action/Query, effects
and scopes; J18F lifecycle, outcome, replay, Result Anchor, Trail and
conformance; J18G credential-free supervised reference constraints. Capability
names and schemas are frozen only by this milestone's dedicated implementation
packets, not by this roadmap.

**Reuse:** Milestones 1-3; `policy.rs`, `approval.rs`, `dispatch.rs`,
`outcome.rs`, replay, Result Anchor, event queue, `child_process.rs`, current
local file provider and J14C disposable filesystem fixtures as adapters/evidence.

**Anticipated modules:** packaged reference provider, installed Plug binding
adapter, scope evaluator, deterministic filesystem conformance fixtures and
demo harness.

**Packets:** P11-FILE-CONTRACT (Amber): freeze names/schemas and scope contract.
P12-FILE-PROVIDER (Amber): package the no-network provider and deterministic
fixtures. P13-FILE-END-TO-END (Amber): complete lifecycle and outcome/Trail/replay
demonstration.

**Owner/colour:** Luna/OpenCode Green/Amber under frozen contracts; DeepSeek Pro
V4/Amber for cross-module integration; Lucy review of schemas and trust boundary.

**Dependencies:** Milestones 1-3 and dedicated P11 contract approval.

**Evidence:** provider conformance, policy/scope/approval, deterministic
filesystem tests, package/install/trust lifecycle, replay restart, Result Anchor
and Trail ordering, full regression, and a disposable Windows end-to-end demo.

**Stop:** network/credential access, path escape, overwrite ambiguity, outcome
misclassification, replay bypass, or schema change outside P11.

**Rollback:** remove the File Tools Plug path/provider while leaving 0.2 config,
manifests, fixtures and CLI unchanged.

**Exclusions:** PDF parsing, Anchors, network listeners, arbitrary providers,
unrestricted file access and final release claims.

## Milestone 5: Durable local Anchor and lifecycle completion

**Result:** A bounded local reference source produces provider-stable events
through a host-owned durable admission authority separate from replay, Trail and
the per-invocation J11 gate. Restart proves admitted, duplicate,
identity-conflict, rejected and admission-uncertain paths; acknowledgement occurs
only after durable admission. Install/enable/disable/restart/remove are visible,
with no listener, unbounded stream or operation outcome created by admission.

**Contracts:** J18F external identity, durable admission, acknowledgement,
generation 0 root Anchors and generation 0..8 causal limits; J18G host ownership;
J18B separation of coordination, policy, Trail and replay. A separate design
packet must freeze the store and source identity before implementation.

**Reuse:** `event_admission.rs` for pure per-invocation safety and generation
checks, `event_queue.rs` for serial Result Anchors, `result_anchor.rs`,
`replay_runtime.rs`/`replay_windows.rs` only for operation replay evidence, and
existing event-admission Trail tests. Do not merge durable stores.

**Anticipated modules/stores:** external-event admission store, local source
adapter, lifecycle registry binding and restart recovery evidence. Exact schema
and event identity remain packet decisions.

**Packets:** P14-ANCHOR-STORE-DESIGN (Red): schema/version, atomicity, crash
recovery, permissions, corruption and rollback. P15-LOCAL-SOURCE-DESIGN (Red):
provider-persisted identity and bounded source contract. P16-ANCHOR-LIFECYCLE
(Red): implementation, restart, acknowledgement and lifecycle integration.

**Owner/colour:** Codex Terra High/Red for durable storage and restart authority;
DeepSeek Pro V4/Amber for bounded source integration; Lucy architecture review.

**Dependencies:** Milestone 4 lifecycle and separate P14/P15 design acceptance.

**Evidence:** durable replay and event-admission restart tests, identity conflict
tests, crash/corruption tests, acknowledgement ordering, Result Anchor/Trail
ordering, lifecycle tests and proof no operation outcome is emitted by admission.

**Stop:** conflated replay/Trail/admission authority, fabricated identity,
ack-before-admission, false success/failure outcome, raw stream or network
listener requirement.

**Rollback:** disable the local source and remove its binding/store adapter;
operation replay, Trail and File Tools remain intact.

**Exclusions:** network webhooks, raw Streams, sensors, Jobs, Human Tasks and
provider-to-provider communication.

## Milestone 6: PDF Tools and first Plug Kit release gate

**Result:** A bounded PDF extraction Query provider is packaged as `.tetherplug`
and treated as hostile parser input. Exact materialisation, page/byte/time/
memory/output limits, disposable scratch, no network and no credentials are
proven. Supervised mode is labelled reference/competition-only; production is
refused without proven isolation. The user workflow covers inspect, install,
conformance, approve, enable, list, disable and remove, with retained 0.2
regression coverage.

**Contracts:** all first-slice J18B-J18G boundaries; J18D package lifecycle;
J18E Query and resource scope; J18F outcomes, conformance and Trail; J18G
supervised/isolation honesty and refusal. No arbitrary third-party safety claim.

**Reuse:** File Tools lifecycle and installed path, package/trust/conformance
stores, Socket, policy, dispatch, replay, Result Anchor, Trail, child
supervision, existing PDF first-envelope evidence and 0.2 regression suites.

**Anticipated modules:** PDF provider package, hostile-input fixtures, bounded
materialisation/limit adapter and public Plug lifecycle command surface. Public
CLI is introduced only through a dedicated packet.

**Packets:** P17-PDF-CONTRACT (Amber): freeze bounded Query schema and limits
through measurement, not guesses. P18-PDF-PROVIDER (Red): hostile parser
provider, package and conformance. P19-PLUG-LIFECYCLE-CLI (Amber): inspect,
install, conformance, approve, enable, list, disable and remove. P20-RELEASE-
EVIDENCE (Red): final independent review and clean-machine/isolated-host gate.

**Owner/colour:** DeepSeek Pro V4/Amber for provider and CLI integration; Codex
Terra High/Red for parser/process and final release verification; Lucy final
architecture review; Matthew final product authority.

**Dependencies:** Milestones 1-5, P17 contract/measurement, and all store
evidence. No version number or release is created by J18I.

**Evidence:** hostile PDF parser tests, materialisation/limit tests, reference
provider conformance, package/install/trust lifecycle, end-to-end File/PDF demo,
full Rust/OCaml regression, and clean-machine or isolated test-host evidence.

**Stop:** parser isolation overclaim, network/credential access, unbounded input,
missing lifecycle evidence, 0.2 regression, or request for arbitrary third-party
enablement. Require final Red review before publication.

**Rollback:** disable PDF and new lifecycle commands while retaining File Tools,
stores and the released 0.2 path; publication is a separate gate.

**Exclusions:** public registry, download/update, remote HTTP, OAuth, network
listeners, Jobs, Streams, Human Tasks, renderers, sensors, printers, MIDI,
smart locks, industrial actuation, unrestricted shell, dependency installation,
AppContainer completion and Tether language changes.

## Packet map

The following packets are proposed future work only. Listing does not authorise
them. Every packet is intended to fit one focused agent run and one review.

| Packet ID | Objective / dependencies | Owner / colour | Areas and acceptance evidence | Schema? / public CLI? | Stop / rollback |
|---|---|---|---|---|---|
| P1-SOCKET-PARITY | Smallest `main.rs` extraction; none | Luna / Green | Rust host/application tests; exact 0.2 regression | No / No | parity loss; revert seam |
| P2-SOCKET-BOUNDARY | Semantic Socket over retained stdio; P1 | Luna / Amber | MCP lifecycle/transcript evidence | No / No | lifecycle drift; revert wrapper |
| P3-DISCOVERY-CATALOGUE | Pagination/drift/invalidation; P2 | Luna / Amber | duplicate/schema/catalogue tests | No / No | J18C ambiguity; revert discovery |
| P4-PACKAGE-INSPECT | Strict `.tetherplug` inspection; P3 | Codex / Red | parser/archive adversarial evidence | Yes, dedicated / No | parser ambiguity; discard inspector |
| P5-QUARANTINE-PATHS | Safe extraction; P4 | Codex / Red | traversal/reparse/TOCTOU evidence | No / No | escape; remove extractor |
| P6-INSTALLED-IDENTITY | Disabled immutable registry; P4/P5 | DeepSeek / Amber | identity/store lifecycle evidence | Yes, design first / No |
| P7-TRUST-SIGNATURE | Ed25519/trust/revocation; P6 | Codex / Red | crypto/trust vectors | Yes, design first / No | false trust; disable gate |
| P8-LAUNCH-PROFILE | Clean exact supervised launch; P7 | Codex / Red | Windows process/environment tests | No / No | isolation overclaim; revert profile |
| P9-CONFORMANCE-GATE | Pinned evidence and invalidation; P7/P8 | Codex / Red | conformance lifecycle, disabled-after-pass | Yes, design first / No |
| P10-CREDENTIAL-METADATA | Optional metadata only; P7 | DeepSeek / Amber | confidentiality/store tests | Yes, design first / No | secret delivery request; omit |
| P11-FILE-CONTRACT | Freeze File Action/Query schema; P1-P9 | Luna / Amber | approved contract and scope matrix | Yes / No | semantic disagreement; stop |
| P12-FILE-PROVIDER | Package credential-free provider; P11 | Luna / Amber | provider conformance/filesystem fixtures | Package schema as approved / No |
| P13-FILE-END-TO-END | First runnable Plug; P12 | DeepSeek / Amber | complete lifecycle/outcome/Trail demo | No / No | boundary failure; disable Plug |
| P14-ANCHOR-STORE-DESIGN | Durable admission authority design; P13 | Codex / Red | crash/atomicity/security design review | Yes / No | unresolved store authority; stop |
| P15-LOCAL-SOURCE-DESIGN | Stable local event identity; P14 | Codex / Red | identity/admission contract review | Yes / No | fabricated identity risk; stop |
| P16-ANCHOR-LIFECYCLE | Implement durable Anchor lifecycle; P14/P15 | Codex / Red | restart/admission/ack tests | Yes / No | false admission; disable source |
| P17-PDF-CONTRACT | Measured bounded Query contract; P9/P13 | DeepSeek / Amber | limits and hostile-input matrix | Yes / No |
| P18-PDF-PROVIDER | Hostile-input provider; P17 | Codex / Red | package/conformance/e2e evidence | Package schema as approved / No |
| P19-PLUG-LIFECYCLE-CLI | User workflow; P16/P18 | DeepSeek / Amber | CLI lifecycle tests and docs | No, unless packet says / Yes |
| P20-RELEASE-EVIDENCE | Independent final gate; P19 | Codex / Red | clean host, full suites, complete diff | No / No |

The first implementation packet after acceptance is P1-SOCKET-PARITY only. It
must not parse packages, implement File Tools, or begin security work in the
same packet.

## Worker routing

Luna on OpenCode owns bounded Green work, fixtures, documentation and ordinary
Amber work under frozen interfaces. DeepSeek Pro V4 owns thicker middle
implementation and cross-module integration requiring Lucy review. Codex Terra
High owns Red Windows security/process boundaries, archive/path attacks,
cryptography/trust, durable storage migrations, Git surgery and final release
verification. Lucy owns architecture guard, packet design, review and verdict.
Matthew retains final product authority. Do not route every task to the strongest
worker.

## Test and evidence plan

Every milestone must use the strongest applicable layers: pure unit tests;
parser/duplicate-key tests; archive/path adversarial tests; real Windows
child-process and Job Object tests; MCP transcript/pagination tests;
package/install/trust lifecycle tests; policy/scope/approval tests; durable
replay and event-admission restart tests; Result Anchor/Trail ordering tests;
reference-provider conformance; full Rust and OCaml regression; end-to-end File
Tools/PDF demonstrations; and clean-machine or isolated test-host evidence
before release. Milestones 1, 4 and 6 require full regression; Milestones 2 and
3 require adversarial, process and lifecycle evidence; Milestone 5 requires
restart, corruption, admission and acknowledgement evidence.

No performance number is frozen here. A measurement packet must establish any
default byte, page, time, memory, rate or output limits. Limits are host-owned,
bounded and fail closed until measured.

## Durable stores and schemas

The following authorities remain separate: installed Plug registry; publisher
trust store; conformance evidence; credential profile metadata; operation replay;
external-event admission; and Trail. Existing replay and Trail authorities are
not merged with new stores.

Each new durable store requires a schema/version design packet, atomicity and
crash-recovery model, permissions/confidentiality review, migration/rollback
plan, corruption behaviour, and tests proving no automatic retry or false
admission. Installation, trust, conformance, credentials, replay, admission and
Trail may reference one another by immutable identity but must not share
authority.

## First-slice exclusions

Outside the first Plug Kit: public registry/marketplace, automatic download or
update, remote HTTP providers, OAuth, general network egress/listeners,
credential-bearing production integrations, arbitrary third-party enablement,
AppContainer completion unless separately authorised, Jobs, Streams, Human
Tasks, long-running renderers, live sensors, printers, MIDI, smart locks,
industrial actuation, unrestricted shell, interpreter-backed production
providers, dependency installation, Plug-to-Plug communication and Tether
language changes. Supervised execution is not hostile-code isolation.

## Risk register

| Risk | Prevention | Detection | Containment | Owner |
|---|---|---|---|---|
| 0.2 regression during extraction | smallest P1 seam, parity first | full Rust/OCaml/CLI suites | retain legacy path, revert seam | Luna/Lucy |
| package/archive attack surface | strict parser, quarantine, no execution | adversarial archive/path tests | quarantine discard, disable install | Codex |
| identity/digest conflation | distinct types and registry indexes | conflict/property tests | reject and quarantine | Codex |
| false trust from signing/conformance | host trust is separate; conformance no permission | trust/revocation tests | revoke/disable | Codex |
| supervised mode mistaken for isolation | visible labels and refusal text | review/search and launch tests | refuse production use | Lucy/Codex |
| Windows path/reparse escape | handle-based validation, no reparse following | traversal/junction/TOCTOU tests | quarantine and disable | Codex |
| environment/credential leakage | clean environment, deny network, metadata only | child-process and redaction tests | terminate/quarantine; no retry | Codex |
| stale discovery | catalogue snapshot and invalidation | pagination/drift transcripts | stale binding unavailable | Luna |
| provider process survival | Job Object and bounded close | real Windows process tests | kill job, mark unavailable | Codex |
| outcome misclassification | typed attempted-operation boundary | outcome matrix tests | no Result Anchor for unattempted | DeepSeek/Lucy |
| replay/Result Anchor publication failure | intent-first and separate authorities | crash/restart/ordering tests | manual resolution, no retry | Codex |
| external identity conflict | provider-stable identity and durable admission | conflict/restart tests | quarantine or disable source | Codex |
| durable-store corruption | versioning, atomicity and recovery design | corruption injection tests | fail closed, preserve evidence | Codex |
| first-slice scope growth | explicit exclusions and packet gates | Lucy bounded review | stop and defer | Lucy |
| deadline pressure causing shortcuts | Red gates and no release claim | independent final review | delay publication, never weaken boundary | Lucy/Matthew |

## Final gate

J18I does not create a release or version number. After P20, Codex performs the
independent Red review, Lucy accepts or rejects the evidence, and only Matthew's
authority permits publication. The released 0.2.0 reference and tag remain
unchanged throughout. Implementation starts only with a later explicit packet,
whose first candidate is P1-SOCKET-PARITY.
