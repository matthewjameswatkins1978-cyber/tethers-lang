# Tethers Universal Plug Architecture

Status: `J18B candidate architecture, pending Lucy acceptance`

This document defines the large architectural boundary for Universal Plugs. It
does not freeze the Socket wire contract or package format, and does not
authorise implementation. J18C through J18I define and accept the precise
contracts that follow this boundary.

## 1. Purpose

An outside system may connect to Tethers when it can clearly describe its
identity, capabilities, effects, input and output data, communication binding,
success, failure and uncertainty behaviour, and how Tethers may control,
constrain and verify it.

A Plug remains outside Tethers Core. Adding or removing a Plug must not insert
vendor-specific behaviour into the language, planner, or core policy semantics.

## 2. Architectural Law

The outbound semantic route is:

```text
Tether source
-> deterministic Core planning
-> host capability resolution
-> host policy and permission
-> host durable intent
-> Tethers Socket
-> protocol binding
-> transport
-> Plug provider
-> outside system
```

The return route is:

```text
outside result or event
-> Plug provider
-> protocol binding
-> Tethers Socket
-> host validation and admission
-> canonical outcome or Anchor
-> Trail
-> optional further deterministic Tether evaluation
```

This is a semantic architecture, not a claim that every layer must be a
separate process.

## 3. Socket, Binding, and Transport

The **Tethers Socket** is the versioned semantic contract between a Tethers
host and an outside provider. It defines categories including identity,
capability discovery, invocation, result delivery, event delivery, lifecycle,
health, error mapping, compatibility, cancellation declarations, and
retry-safety declarations.

The Socket is not a programming language, vendor API, package archive,
marketplace, network server, or particular byte transport. Its precise
operations and schemas belong to J18C.

A **protocol binding** encodes Socket operations into an existing or future
protocol. The first intended binding is `MCP 2025-11-25`.

A **transport** carries protocol messages. The first intended transport is
`local stdio`. MCP is not itself casually treated as a byte transport. The
initial stack is:

```text
Tethers Socket semantics
-> MCP 2025-11-25 binding
-> local stdio transport
```

Future bindings or transports must not change capability meaning.

## 4. Layer Ownership

### Tethers Core, OCaml

Core owns parsing and validation of Tether 0.1, deterministic Anchor and
Condition evaluation, deterministic Action planning, capability requirement
projection, Core Trail entries, and stable plan identities required by the
current contract.

Core does not own package installation, provider discovery, credentials, live
provider truth, final permission, process management, network access, retries,
vendor APIs, Plug lifecycle, or resource enforcement. Core must not know that a
capability came from a Plug.

### Tethers Host, Rust

The host owns package inspection and admission, trusted manifest verification,
capability and provider binding, live capability resolution, policy and
permission, credential references and brokering, scope assessment, durable
intent, dispatch, canonical outcome classification, replay protection, Result
Anchors, host Trail entries, installation and removal, provider lifecycle and
health, conformance orchestration, compatibility decisions, quarantine, and
disablement.

### Socket Binding and Transport Adapters

These adapters own generic protocol and transport translation. Host-side
adapters remain generic and reviewed. Vendor-specific translation remains
outside the host, normally inside the Plug provider. The host must not grow
branches such as `if provider is GitHub`, `if provider is Notion`, or
`if device is printer`.

### Plug Provider

A provider owns communication with one outside system or related family,
vendor-specific APIs and protocols, conversion between outside data and
declared Socket data, declared capability implementation, provider-local
validation, honest result and event reporting, and applicable provider-side
safety checks.

It cannot grant itself permission, edit host configuration, redefine canonical
outcomes, silently expand scope, put secrets in Tether source, or bypass host
Trail recording.

### CLI, Shell, and HQ

These are projections and control interfaces. They may display, request, and
submit installation or policy intent, but are not authorities for permission,
package trust, credential access, outcome truth, or Trail truth.

## 5. Canonical Concepts

- **Plug package:** portable development or installation unit containing provider
  material, capability manifests, metadata, tests, documentation, and optional
  signatures. Exact `.tetherplug` structure belongs to J18D.
- **Installed Plug:** host record for an admitted package on one machine; it is
  distinct from the portable package.
- **Provider:** process or service implementing capabilities. A package may
  contain or reference one provider, and a provider may expose several related
  capabilities.
- **Capability:** versioned, schema-described bounded operation or event source.
- **Capability manifest:** a claim describing a capability; it does not prove
  provider identity, availability, or permission.
- **Binding:** host-owned connection between an installed Plug, provider
  identity, exact capability identity and version, reviewed manifests and
  digests, configuration, policy, scopes, and credential references.
- **Installation:** host-owned state and evidence after package inspection,
  review, testing, and approval.
- **Credential profile:** host-owned reference to authentication material;
  credentials never belong in Tether source or a portable package.
- **Effect:** structured declaration of what an operation may affect.
- **Scope:** structured host-readable limit on where, what, whom, when, or how
  much a capability may affect.
- **Trail:** historical causal evidence; removing a Plug must not erase it.

## 6. Capability Classes

The architecture reserves six conceptual classes: **Action**, **Query**,
**Anchor**, **Job**, **Stream**, and **Human Task**. A capability class named
Action is not the same object as a Tether Action. A Tether Action invokes a
capability, which may belong to the Action class.

- **Action:** performs one bounded change.
- **Query:** reads without deliberately changing the outside system. Read-only
  does not mean permission-free.
- **Anchor:** introduces an outside event into Tethers.
- **Job:** starts work that may complete later.
- **Stream:** produces continuing or high-volume information that must be
  reduced to bounded events, checkpoints, or summaries before entering Tethers.
- **Human Task:** requests explicit human work or judgement.

Only Action, Query, and Anchor are candidates for the first implementation. Job,
Stream, and Human Task remain reserved and unimplemented. The current runtime
does not already support any of these Plug classes.

## 7. Outbound Request Flow

For Action and Query capabilities the required sequence is:

1. Core emits a deterministic Plan.
2. The host exact-resolves the capability and provider binding.
3. The host revalidates manifests, digests, live discovery, and compatibility.
4. The host assesses structured scope.
5. The host applies effective policy.
6. Required approval is obtained exactly once.
7. Durable intent is recorded before an effectful attempt.
8. The host sends one Socket request.
9. Binding and transport deliver it to the provider.
10. The provider performs or refuses the outside operation.
11. The host validates returned data.
12. The host classifies the canonical outcome.
13. The host records the Trail.
14. The host emits the appropriate Result Anchor when the current runtime
    contract permits it.

No provider call occurs before the host gates pass. No automatic retry is
introduced.

## 8. Inbound Anchor Flow

The conceptual event route is:

1. An outside event occurs.
2. The provider assigns or reports source and event identity.
3. The binding carries a Socket event.
4. The host authenticates the provider binding.
5. The host validates event schema and source.
6. The host classifies new, duplicate, replayed, or untrusted identity.
7. The host records admission evidence.
8. An accepted event becomes a Tethers Anchor.
9. Core evaluates deterministic Tethers.
10. The host continues through the normal execution boundary.

The exact subscription, acknowledgement, replay cursor, and retention contract
belongs to J18F.

## 9. Host-Owned Invariants

- Packages and provider output are untrusted until validated.
- A signature proves origin or integrity, not permission.
- Manifests describe; they do not authorise.
- Policy and credentials remain host-owned.
- Structured scopes must be host-readable.
- A provider may add safety checks but cannot be the only permission boundary.
- Unknown or unsupported scope mappings fail closed.
- Uncertainty remains uncertainty.
- No automatic retry exists without end-to-end retry-safety proof.
- A transport change must not change capability semantics.
- Historical Trails survive disablement, removal, and package upgrades.
- Plugs cannot communicate directly with other Plugs in the first architecture.
- No Plug may inject hidden AI judgement into deterministic Conditions.
- Probabilistic AI capabilities must declare that property.
- AI outputs return as data for later deterministic evaluation.
- Physical connectivity does not imply hard-real-time or safety certification.

## 10. State Separation

Package trust, installation, provider health, and operation outcome are distinct
state families and must not be merged into one status field.

### Package Trust State

Candidate vocabulary is unsigned development, locally reviewed, publisher
signed, organisation approved, quarantined, and revoked. Exact serialized names
are not frozen here.

### Installation State

The conceptual sequence is:

```text
inspected -> format validated -> compatible -> reviewed -> configured
-> tested -> approved -> enabled
```

Possible side states are disabled, degraded, unavailable, quarantined, and
removed. These transitions are not all implemented.

### Provider Health State

Conceptual states are installed, verified, configured, enabled, healthy,
degraded, unavailable, disabled, and quarantined.

### Operation State

The current intent-first boundary remains authoritative:

```text
planned -> permission resolved -> durable intent -> attempted
-> canonical outcome -> Trail
```

## 11. Outcome Ownership

Providers may return detailed provider codes and safe diagnostic data, but only
the host assigns canonical Tethers outcomes. A provider cannot redefine success,
failure, or uncertainty. Malformed or schema-invalid success data cannot become
success. A timeout does not prove that no outside effect occurred, and
uncertainty cannot silently become failure or success.

The current Tethers 0.2 normative outcomes remain `succeeded`, `failed`, and
`uncertain`. Future categories such as cancelled, timed out, unavailable,
rejected, or partially completed remain J18F candidates and must not silently
change the current vocabulary.

## 12. Version Axes

These axes remain independent: Tethers product version, Tethers language
version, Tethers Socket version, Plug package-format version,
capability-manifest version, capability identity and version, provider identity
and version, protocol-binding version, and underlying protocol version.

Compatibility must be explicit. Product version is not a substitute for Socket
or package compatibility, and “latest” must not be selected silently.

## 13. Installation Boundary

The architectural installation flow is: inspect without executing; validate
package structure; verify digests and available signatures; check platform,
Socket, and binding compatibility; display capabilities, effects, privacy, and
resource needs; collect scopes and policy; create host-owned credential
references; create exact pinned bindings; start only in isolated test mode; run
conformance checks; show evidence; activate only after explicit approval; and
record installation and approval in the Trail.

The package never edits host configuration itself. Command syntax and exact
package schemas are deferred.

## 14. Removal Boundary

Removal identifies dependent Tethers, prevents new calls, handles or reports
active operations, stops the provider, removes active bindings, retains
historical Trails, keeps or separately removes credentials according to explicit
choice, removes package files, and proves that no active capability remains.
Removal is not historical erasure.

## 15. Adapter Rule

Generic Socket protocol bindings and byte transports are host-owned
infrastructure. Vendor-specific API, device, or command translation lives in a
provider outside the host. Unusual systems may be reached through a dedicated
provider or gateway. The host must not become a catalogue of vendor-specific
adapters.

REST, database, serial, MIDI, and industrial gateways are future edge providers
or reviewed generic bindings, not host branches.

## 16. First Implementation Envelope

The intended first envelope is native Windows, a local provider process, MCP
2025-11-25 over stdio, Action/Query/Anchor classes only, one package format,
multiple capabilities per Plug, host-owned inspection and installation,
checking and conformance testing, disablement and removal, automatic digest
pinning, host-generated runtime configuration, and friendly permission setup.

File Tools is the reference Plug and PDF Tools is the competition Plug. This
document alone does not authorise implementation. J18C through J18I must define
and accept the precise contracts first.

## 17. Explicitly Deferred Scope

Deferred scope includes a marketplace, public registry, automatic downloads,
automatic updates, remote HTTP providers, OAuth implementation, hardware
implementation, Jobs, Streams, Human Tasks, dependency installation, payment
capabilities, unrestricted shell execution, certified industrial control,
Plug-to-Plug direct communication, multi-user service, and general AI-agent
framework behaviour.

## 18. Unsuitable or Impossible Systems

Tethers must refuse or defer connection when no usable API or protocol exists,
connection requires bypassing security, stable operation or event identity
cannot be established, outcomes cannot be reported honestly, effects cannot be
bounded or permissioned, hard-real-time guarantees are required, safety
certification forbids the controller, licensing forbids automation or
redistribution, or a required driver or gateway does not exist.

A future Human Task capability may represent a manual step instead of
pretending direct control exists.

## 19. Paper-Validation Obligation

J18H must validate these examples on paper: local file tool, PDF processor,
GitHub service, email service, SQL database, cloud drive, remote AI model, local
AI model, webhook source, long-running video renderer, live sensor stream,
printer, MIDI instrument, smart lock, industrial machine, and human approval
queue.

For every example J18H must answer technical possibility, capability class,
protocol binding and transport, effects, scopes, authentication, success,
uncertainty, retry safety, cancellation, restart survival, Trail evidence, and
refusal boundary. The architecture is not frozen until these examples can be
described without changing Tether language semantics or inserting vendor logic
into Core.

## 20. Architecture Acceptance Criteria

- No example requires editing Tether syntax.
- No vendor-specific policy branch enters Core or host policy.
- Permissions and credentials remain host-owned.
- Canonical outcomes remain honest.
- Manifests remain schema-validated claims rather than authority.
- External effects are structured and declared.
- Duplicate and replayed events are addressed.
- Job and Stream shapes are reserved without false implementation claims.
- Physical effects carry explicit risk boundaries.
- Installation and removal are deterministic.
- Historical Trails survive removal.
- Unsupported systems fail honestly.
- The first implementation remains small enough for competition delivery.

The first implementation should be small. The architecture should not be
small-minded. Tethers is building one reliable socket, not a growing pile of
special cables.
