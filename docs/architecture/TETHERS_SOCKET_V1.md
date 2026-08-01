# Tethers Socket v1

Status: J18C candidate, pending Lucy protocol review
Implementation: Not authorised

## 1. Identity and Purpose

Tethers Socket major version: **1**.

Socket version is independent of Tethers product and language versions, package
format, capability manifest, MCP protocol, provider, and capability versions.
The host must select an explicit compatible version; it must never silently use
the newest version.

Socket v1 is the semantic boundary between a trusted Tethers host and an
outside provider. It normalizes provider and capability observation, invocation,
result observation, catalogue change, health, and session closure. It does not
install packages, approve packages, decide permissions or scope, store
credentials, classify canonical outcomes, retry, or evaluate Tethers.

## 2. Layers and Ownership

```text
Tethers Core (OCaml)
  -> deterministic Plan
Tethers host (Rust)
  -> trust, binding, policy, durable intent, Socket
Socket v1
  -> protocol binding
MCP provider binding
  -> local stdio transport
Plug provider
  -> outside system
```

Core remains unaware of packages, Plugs, providers, and Socket details. The
host owns bindings, credentials, permission, scope, lifecycle, validation,
canonical outcomes, replay protection, and Trails. Protocol and transport
translation is generic; vendor-specific translation remains in the provider.

## 3. Semantic Operations

These names are semantic operations, not wire method names:

- `establish` creates one session and records the expected binding, provider
  observation, Socket and binding compatibility, negotiated features, session
  identity, start time, executable identity, and known configuration identity.
- `discover` obtains every page of the complete live capability advertisement.
  Discovery grants neither permission nor installation.
- `invoke` carries one already-authorised request to one exact binding.
- `observe_result` returns raw protocol and provider observations for host
  validation and canonical classification.
- `observe_catalogue_change` marks previous discovery stale; it never changes
  bindings automatically.
- `probe` supplies liveness evidence only. It does not prove readiness,
  authentication, safety, permission, or capability correctness.
- `close` ends the session without deleting installation or Trail history.

## 4. Semantic Records

The following are conceptual records, not executable schemas.

**Provider Binding Expectation** is host-owned and contains installed Plug
identity, package digest, provider identity/version expectation, launch identity,
Socket version, protocol binding, transport, platform, approved configuration,
and credential-profile references.

**Provider Observation** contains untrusted session identity, implementation
name/version, negotiated protocol version, advertised protocol capabilities,
process identity where available, start/observation times, and health evidence.
It is not cryptographic identity.

**Capability Binding Expectation** binds Tethers capability name/version,
trusted manifest digest, provider identity, provider operation name, trusted
input/output schemas, class, effects, scopes, policy, credential reference, and
compatibility requirements.

**Capability Observation** contains provider operation name, display claims,
advertised schemas, annotations, execution declarations, observation time, and
session. Descriptions and annotations are untrusted claims.

**Invocation Command** contains durable host execution identity, action or
operation identity, exact binding, arguments, host deadline, approval evidence,
manifest/provider pins, and idempotency data only when an accepted binding
defines its explicit transmission mapping.

**Invocation Observation** preserves dispatch state, transport correlation, raw
result/error, provider error state, structured data, diagnostics, timing,
timeout/cancellation, connection state, and response-validation state. It does
not assign a canonical outcome.

**Catalogue Change Observation** records only that live advertisement may have
changed. **Health Observation** records evidence and time without promoting
liveness into trust or permission.

## 5. Identity and Discovery Laws

Durable Tethers execution IDs identify host executions. Socket session IDs
identify one provider session. MCP JSON-RPC request IDs are session-local
transport correlation values. Provider operation names identify bindings.
Tethers capability identity is semantic `name + version`. None substitutes for
another. A provider restart creates a new session. A transport request ID is
never a replay or idempotency key; an idempotency token is transmitted only by
an explicitly reviewed manifest mapping.

Discovery is untrusted live observation. Installed manifests remain trusted.
All pages must be consumed; cursors are opaque, repeated or looping cursors
fail; duplicate operation names, malformed schemas, and missing bound
operations fail closed. Unknown additions remain unavailable. Removed or
changed bound operations become stale or unavailable. Catalogue change triggers
re-discovery, never automatic acceptance. Dispatch requires an exact trusted
binding match. No class, effect, scope, permission, or idempotency property may
be inferred from provider descriptions or annotations.

## 6. Invocation, Result, and Output Laws

Host gates complete before invocation. Durable intent precedes an effectful
attempt. Socket v1 sends at most one provider request, never retries, preserves
the response, and never emits a Tethers Result Anchor. The first binding is
serial: one active invocation per session, no batching, parallel calls, or
hidden restart queue.

JSON-RPC success, provider tool success, schema-valid output, and canonical
Tethers success are distinct. JSON-RPC errors, provider errors, transport loss,
deadline expiry, invalid output, canonical failure, and canonical uncertainty
are also distinct observations. A protocol response does not prove the outside
effect succeeded; timeout or connection loss does not prove it did not. The host
assigns canonical outcomes. Final mapping is a J18F decision.

Trusted host manifests define authoritative output schemas. Provider
`structuredContent`, when present, is untrusted until validated against the
trusted schema. Successful typed results use `structuredContent`; text is
diagnostic or compatibility material and never authoritative deterministic
result data. Missing or malformed required structured data cannot be canonical
success and may require uncertainty after dispatch.

## 7. Capability Boundaries and Extensions

Socket v1 reserves Action, Query, Anchor, Job, Stream, and Human Task. Action
and Query invocation are first-slice candidates. Anchor delivery is deferred to
J18F. Job, Stream, and Human Task remain reserved and unimplemented.

Catalogue notifications, resource updates, and arbitrary notifications are not
automatically Anchors. MCP Tasks are not Tethers Jobs, progress notifications
are not Tethers Streams, and MCP elicitation is not a Human Task.

Unknown server requests and notifications do not become Tethers events.
Unsupported negotiated features are unused. Provider extensions require a
future explicit binding decision, and unknown fields never grant authority.

## 8. State, Installation, Removal, and Validation

Package trust, installation state, provider health, operation state, canonical
outcome, and Trail history are separate state families. Installation and removal
are host-owned J18B boundaries: inspect, validate, verify, review, configure,
bind, test, approve, enable, disable or quarantine, and remove without erasing
historical Trails. Removal stops active providers, prevents new calls, removes
active bindings, and retains evidence and explicit credential decisions.

No Socket operation grants trust. No provider can edit host policy, broaden
scope, redefine outcomes, or bypass Trail recording.

## 9. J18H Paper Validation

Before final freeze, J18H must paper-validate local files, PDF, GitHub, email,
SQL, cloud drive, remote and local AI, webhook, video renderer, sensor stream,
printer, MIDI, smart lock, industrial machine, and human approval queue. Each
case must identify class, binding/transport, effects, scopes, credentials,
success, uncertainty, retry safety, cancellation, restart survival, Trail
evidence, and refusal boundary without changing language semantics or adding
vendor logic to Core.

## 10. Acceptance Boundary

Acceptance requires separation of Core, host, Socket, binding, transport,
provider, and outside system; independent version axes; fail-closed discovery
and drift; honest outcomes; host-owned authority; preserved Trails; reserved
unsupported classes; and successful J18H paper validation. This document is a
candidate only and authorises no implementation.
