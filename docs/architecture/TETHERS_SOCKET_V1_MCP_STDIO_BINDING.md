# Tethers Socket v1 MCP Stdio Binding

Status: Accepted J18C contract
Accepted by Lucy: 2026-08-01
Final architecture freeze: Requires J18H paper validation
Implementation: Not authorised

## 1. Direction and Roles

Two MCP directions must remain separate:

```text
Existing Core-facing MCP:
external MCP client -> Tethers OCaml MCP server -> deterministic Core

Universal Plug provider binding:
Tethers Rust host acting as MCP client -> outside Plug provider acting as MCP server
```

The first exposes planning and evaluation. The second reaches outside
capabilities. They share a protocol family but not implementation or authority.

## 2. Standard Mapping

The first binding is MCP `2025-11-25` over local stdio and uses standard methods,
not custom Socket methods:

| Socket operation | MCP operation |
|---|---|
| establish | `initialize`, result, then `notifications/initialized` |
| discover | `tools/list`, all pagination pages |
| invoke | `tools/call` |
| observe_result | matching JSON-RPC result or error |
| observe_catalogue_change | `notifications/tools/list_changed` |
| probe | `ping` |
| close | bounded stdio/process shutdown |

No custom `tethers.socket.invoke`, `tethers.socket.discover`,
`tethers.socket.event`, or `tethers.socket.health` method is required.

## 3. Initialization and Negotiation

The host proposes exactly MCP `2025-11-25`; the provider must negotiate that
version for this first binding and advertise tools capability. Only negotiated
features are used. The host validates the result, compares server information
with the pinned provider expectation, then sends `notifications/initialized`.
No discovery or call precedes successful initialization, and initialization
does not grant permission. The host advertises only client capabilities it
actually implements, never sampling, elicitation, roots, or other unsupported
features.

## 4. Stdio and Credentials

The host launches the exact configured executable and arguments without shell
interpolation. Working directory and environment are host-owned. Stdin and
stdout carry UTF-8 newline-delimited JSON-RPC 2.0 messages; each message is one
line and embedded newlines are forbidden. Provider stdout is protocol-only.
Stderr is captured separately and is never protocol or outcome evidence.

Implementations must impose finite message, schema, tool-count, and nesting
limits. Malformed stdout is a protocol violation. EOF records its operation
stage. Shutdown is bounded and no provider process may survive unnoticed.

Stdio uses no MCP HTTP authorization. Credentials remain host-owned, are not in
Tether source or packages, and are injected only through approved process
environment or a later broker. Environment variables are not copied wholesale
into Trails and diagnostics redact secrets.

## 5. Discovery

`tools/list` pagination is completed before discovery is complete. Cursors are
opaque; looping or repeated cursors fail. Tool names are unique. Bound tools
must have valid input schemas; optional output schemas are recorded. Titles,
descriptions, icons, annotations, and execution hints are untrusted and cannot
determine class, effects, scope, permission, or idempotency.

Live advertisements are compared with trusted manifests and exact bindings.
Drift prevents invocation until revalidated. `notifications/tools/list_changed`
means catalogue drift, not a Tethers Anchor: mark stale, prevent affected new
calls, perform bounded re-discovery, retain exact matches, quarantine changed or
missing bindings, leave additions unapproved, and record the transition.

## 6. Invocation and Results

The host resolves policy, scope, approval, pins, and trusted input arguments
before one `tools/call`. It creates one fresh JSON-RPC request ID unique within
the session. That ID is session-local correlation, not a durable execution ID,
replay key, or idempotency key. Idempotency data is included only through an
approved manifest argument mapping. There is no hidden argument injection,
batching, parallel call, or automatic retry. Late, duplicate, and mismatched
responses are recorded and rejected.

The host preserves raw `content`, raw `structuredContent`, `isError`, protocol
error object, timing, connection, timeout, and cancellation evidence. A JSON-RPC
result means protocol completion only. `isError: true` is provider-declared
execution error evidence. Neither directly assigns the canonical Tethers
outcome. Trusted output-schema validation is required; provider text is
diagnostic and untrusted. Final canonical mapping remains J18F.

## 7. Ping and Shutdown

`ping` proves only that the MCP session responds. It does not prove outside API
readiness, credentials, capability correctness, safety, manifest agreement, or
future success.

Shutdown stops admission, resolves or reports the active call, closes provider
stdin, waits for bounded graceful exit, terminates an uncooperative process,
captures final stderr separately, records closure, and retains Trail and
installation state. There is no invented MCP shutdown request.

## 8. Explicit Non-Mappings

MCP Tasks are not Tethers Jobs. MCP progress notifications are not Tethers
Streams. MCP elicitation is not a Human Task. Resource updates and arbitrary MCP
notifications are not Anchors. They remain unsupported or deferred unless a
future accepted Socket and binding decision defines them.

The host rejects or reports unexpected server-to-client sampling, elicitation,
roots, or similar requests according to the binding. It never silently honours
an unsupported request.

## 9. Acceptance

Acceptance requires standard MCP methods, explicit opposite MCP directions,
complete paginated discovery, trusted-schema validation, separate identity
axes, catalogue drift quarantine, no automatic retry, bounded stdio shutdown,
host-owned credentials, and no implementation or schema change. J18H paper
validation remains mandatory before final architectural freeze.
