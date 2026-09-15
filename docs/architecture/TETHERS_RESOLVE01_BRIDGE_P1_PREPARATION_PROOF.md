# Tethers x Resolve01 P1 — Preparation Proof

Status: `IMPLEMENTED ON TASK BRANCH`

Base: `66e8360a7247b60ccc8fb6cb447f796b922afb42`

P1 is a Rust-host evidence substrate. It does not contact Resolve, admit a
guard, execute a provider, alter replay, or change Tethers Core. Preparation is
evidence, never authority.

## Implementation

The implementation is in
`tethers-0.1/host-rust/src/resolve_guard.rs`, exported by `lib.rs`. The current
prepared runtime exposes only an internal `resolve_action_scope` projection;
the existing public `assess_action_scope` uses that same projection, so P1
does not create a second scope assessor.

The public preparation route is:

```text
prepare_resolve_guard_evidence(
    &PreparedRuntime,
    &ProposedAction,
    &ResolvedCapability,
    &ProviderAvailability,
    &PermissionDecision,
) -> PreparedResolveGuard
```

`PermissionDecision::Allow` must already exist as the result of the current
host approval route; P1 does not manufacture or consume approval. The route
also resolves the exact pinned capability again through the supplied current
`ProviderAvailability`, re-evaluates effective policy against the prepared
runtime, and revalidates the complete action against the current capability
input schema. An unavailable provider or current deny/unavailable policy result
fails closed before evidence is produced. `reconstruct_resolve_guard_evidence`
reruns those authorities against current runtime inputs; it does not copy
fields out of the old proof. For an `Ask` result, callers must supply the new
`Allow` produced by the current exact-approval route.

## GuardPreparationProof

`GuardPreparationProof` is privately constructed and contains:

| Field | Authority |
| --- | --- |
| `format_version` | P1 version constant |
| `evaluation_id`, `plan_id`, `action_id` | the current `ProposedAction` identity |
| `capability_name`, `capability_version` | exact `ResolvedCapability` identity |
| `argument_digest` | current validated Action arguments, through existing `approval::digest` |
| `manifest_digest` | exact verified manifest digest on `ResolvedCapability` |
| `provider_identity` | exact resolved provider identity |
| `resolved_scope_digest` | current binding-owned reviewed scope projection |
| `scope_keys` | opaque keys derived from that reviewed scope |
| `binding_digest` | verified provider/binding metadata plus configured scope binding |

The canonical proof projection contains only those identities and digests.
Canonical proof identity uses the existing `approval::digest` authority, which
uses repository JCS canonicalisation and SHA-256. The proof digest is an
identity for evidence and is not a capability token.

## ScopeKey

`ScopeKey` is an opaque typed wrapper around a `sha256:` digest. Its Debug
representation contains only that digest. It is derived only from the
internal `ResolvedActionScope` returned by `PreparedRuntime` after the existing
JSON-pointer extraction and resource-path validation.

The versioned key projections are:

```text
{ format_version: "tethers.resolve.scope-key/1",
  kind: "path_prefix", value: <already resolved path> }

{ format_version: "tethers.resolve.scope-key/1",
  kind: "unrestricted" }
```

PathPrefix and Unrestricted are currently supported. Repository, Calendar,
missing scope bindings, invalid scope values, and `ScopeNotEstablished` are
refused. No raw argument is inspected heuristically and no unsupported form is
hashed or downgraded to unrestricted. Keys are sorted and deduplicated before
being placed in a proof or projection.

`resolved_scope_digest` is distinct from the key: it commits to the complete
reviewed scope projection, including the validated argument pointer and the
canonical sorted/deduplicated allowed-prefix set. The ScopeKey commits only to
the equality dimension Resolve will eventually compare.

## Binding digest

`binding_digest` uses the versioned `tethers.resolve.binding/1` projection. It
commits to capability identity, verified manifest digest, host-configured
provider identity/source, MCP binding kind/server/tool, optional adapter
identity/digest, configured scope binding, the reviewed scope-binding
projection, and the prepared provider launch command, arguments, and working
directory. These are existing manifest/provider/binding facts needed to detect
rebinding. They are included only in the digest preimage: the proof and Resolve
projection contain no provider secret, description, raw argument, raw
credential, or launch path. P1 does not introduce a second policy or trust
object.

## Resume comparison

`compare_resolve_guard_preparation` first rejects unsupported versions and
malformed digest/identity evidence, then compares every frozen field exactly.
It returns bounded typed reasons such as `ArgumentsChanged`,
`ManifestChanged`, `ProviderChanged`, `BindingChanged`, `ScopeChanged`,
`ScopeKeysChanged`, `CapabilityChanged`, `ActionChanged`, `PlanChanged`, and
`EvaluationChanged`. There is no “close enough”, narrowing, widening, stale
proof repair, or old-policy bypass.

## ResolveGuardRequired

`PreparedResolveGuard::required()` contains exactly:

- the `GuardPreparationProofDigest`;
- the Tethers `action_id`; and
- the sorted, duplicate-free opaque `ScopeKey` vector.

It contains no raw arguments, paths, manifest contents, policy/approval
internals, Resolve Goal/Commitment/Claim state, lease/worker state, or
execution authority. This is the bounded host-owned input P2 may later present
to a guard-admission seam. P1 does not define that seam.

## Preserved exclusions

P1 changes no OCaml source, Core AST, Plan schema, Tether syntax, provider
execution, replay state machine, Trail guard events, Result Anchor taxonomy,
Resolve transport, Resolve database, or provider lifecycle. Proof construction
is pure host computation over already prepared state and has no executor input,
so it cannot invoke a provider.
