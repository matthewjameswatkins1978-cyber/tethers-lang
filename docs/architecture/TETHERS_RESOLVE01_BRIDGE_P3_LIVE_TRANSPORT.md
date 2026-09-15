# Tethers x Resolve01 P3 — Live Guard Transport & Outcome Delivery

Status: implementation record on `codex/tethers-resolve-p3-live-transport`

Accepted Resolve contract: `b450305e72815d33357359c6db641d315d6b2975`

Resolve contract source: `C:\dev\resolve-ai\docs\TETHERS_GUARD_PROTOCOL.md`

## Boundary

P3 implements the transport beneath the accepted P2 `ResolveGuardAdapter` and
the explicit delivery of an already durable Tethers provider outcome. The
transport is coordination only:

```text
Tethers current authorities
  -> P1 preparation / P2 durable intent
  -> one Resolve admission request
  -> existing G1 and provider execution
  -> durable Tethers outcome and Trail/anchor truth
  -> explicit Resolve outcome delivery
```

Resolve never grants Tethers permission. The ordinary unguarded host path does
not construct this client and remains unchanged.

## Implementation

The client is `tethers-0.1/host-rust/src/resolve_transport.rs`:

- `ResolveGuardHttpClient` implements both the accepted P2 admission adapter
  and the P4 outcome-delivery adapter.
- `ureq` 3.4.2 with only its `rustls` feature supplies synchronous HTTPS; the
  client has no cookies, browser behaviour, or generic retry layer.
- `ResolveGuardTransportConfig` is host-owned. It accepts an HTTPS base URL,
  bridge key, and bounded timeout. `new_for_local_test` permits HTTP only for
  loopback tests. Environment construction uses
  `TETHERS_RESOLVE_BASE_URL`, `RESOLVE_TETHERS_BRIDGE_KEY`, optional
  `TETHERS_RESOLVE_TIMEOUT_MS`, and explicit `TETHERS_RESOLVE_LOCAL_MODE=1`
  for loopback HTTP tests.
- Timeout defaults to five seconds and is bounded to 100 ms–30 s. Request and
  response bodies are bounded at 16 KiB. Redirects are disabled. Production
  endpoints must use HTTPS.
- The bridge key is sent only as `X-Resolve-Tethers-Key`; it is absent from
  `Debug`, `Display`, errors, Trail and verification evidence.

## Admission protocol

The client sends exactly one POST to:

```text
/internal/tethers/v1/guard/admit
```

The body is the accepted v1 semantic object containing only
`protocol_version`, the P2 opaque `guard_ref` digest, the P2 action identity,
the P1 preparation digest, and the already sorted/deduplicated P2 ScopeKeys.
The client validates the digest shape, ASCII action bound, maximum 64 ScopeKeys,
strict ordering and the 16 KiB body bound before sending.

The expected request digest is independently computed with Tethers'
`approval::digest` JCS/SHA-256 authority. A response is accepted only when its
protocol is exactly `resolve.tethers-guard/1`, its digest is valid and equals
that independently computed value, its reason is from the bounded v1 set, and
its decision/reason pair is coherent. The closed mapping is exactly
`ADMITTED`, `REJECTED`, or `INDETERMINATE`; adapter/transport errors become P2
`Indeterminate`. There is no automatic admission retry.

Responses are parsed as UTF-8 through the existing duplicate-member-rejecting
manifest JSON parser. Exact object fields are required. Unknown fields,
duplicate members, unsupported versions, unknown decisions, wrong digests,
oversized bodies, malformed JSON and non-success HTTP responses fail closed.

## Outcome delivery

The client sends exactly one POST to:

```text
/internal/tethers/v1/outcome
```

The request contains only protocol version, the trusted Tethers action
reference, the trusted P1 preparation digest and the existing classified
outcome `SUCCEEDED`, `FAILED`, or `UNCERTAIN`. The expected delivery digest is
computed independently over the complete semantic request and must match the
Resolve response. `RECORDED` and `ALREADY_RECORDED` both confirm delivery;
`CONFLICT` is returned as a distinct bounded acknowledgement.

The existing P4 journal now keys records by `(action_ref,
preparation_digest)`. It persists only those identities, the classified
outcome, and delivery state. An explicit redelivery may use the journal after
restart; it has no provider handle and cannot rerun a provider. A conflict is
persisted as coordination failure and never changes the Tethers outcome.

`application::deliver_resolve_outcome` requires the caller to supply the
trusted action reference and preparation digest. It does not derive authority
from an execution-result string or from Resolve. Delivery is available only
after the existing durable outcome/Trail/anchor path has completed.

## Failure and security properties

- Missing URL/key, TLS failure, timeout, connection failure, authentication
  failure, non-success status, malformed/oversized/invalid-UTF-8 response,
  protocol mismatch and digest mismatch cannot reach a provider; P2 maps
  attempted transport failure to `Indeterminate`.
- The request carries no raw guard reference, arguments, paths, manifests,
  policy, provider payload, Resolve state or secret.
- Configuration is never sourced from Tether source, a provider, a Plug or
  action arguments. Without explicit host selection, Resolve calls are zero.
- Provider execution, replay/G1 ordering, outcome classification and Result
  Anchor taxonomy are unchanged. P3 adds no Resolve database access, SDK,
  Core/OCaml change, syntax, public capability or live generic coordination
  framework.

## Evidence

`resolve_transport` unit tests exercise exact admission serialization and
digest verification, strict malformed responses, duplicate members, unknown
fields/versions, invalid UTF-8, outcome acknowledgement and secure endpoint /
redirect policy. Existing P2 tests continue to own provider-call ordering and
negative admission proof. Existing P4 conformance continues to own journal
restart, exact redelivery and provider non-rerun proof.

A real cross-process Resolve smoke requires the accepted Resolve service plus
its Firestore emulator and a Resolve-created guard. Those are environment
prerequisites, not a Tethers-side database substitute; their availability is
recorded in the P3 worker note and closeout rather than inferred from fake
server tests.
