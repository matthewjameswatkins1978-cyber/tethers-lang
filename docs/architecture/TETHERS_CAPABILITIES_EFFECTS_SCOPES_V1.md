# Tethers Capabilities, Effects and Scopes v1

Status: J18E candidate, pending Lucy capability review
Contract generation: 1
Implementation: Not authorised

## 1. Central Distinctions

Capability class describes interaction shape. Effect describes what may be
observed, changed, executed, communicated, or actuated. Scope describes the
bounded targets and limits. Policy decides whether a particular bounded
invocation is `allow`, `ask`, `deny`, or `unavailable`. Outcome reports what is
honestly known after an attempt. None substitutes for another: Action is not
automatically dangerous, Query is not harmless, read does not mean unrestricted,
scope does not grant permission, and provider success does not assign outcome.

## 2. Identity and Class

A capability is `name + positive integer version`, such as
`file.move@1`, `document.extract_text@1`, or `mail.message.send@2`. Each has
exactly one host-reviewed class. Class is never inferred from names, provider
operation names, descriptions, MCP annotations, HTTP methods, transport,
output wording, or runtime provider claims. A read-and-mutate operation is an
Action with all relevant effects; mixed or compound class values are forbidden.

## 3. Six Classes

### Action

An Action requests one bounded external change, execution, or actuation. It
declares at least one mutating, communicative, executing, administrative, or
physical effect, and may also declare reads needed by the operation.

### Query

A Query observes or retrieves information without deliberately changing
persistent business state. It remains permissioned and bounded by scopes,
credentials, output validation, privacy, resource, and cost limits. Deliberate
create, update, delete, send, publish, execute, or actuate operations are not
Queries. Access logs, cache use, quota, and billing are not automatically
mutations but material operational/resource effects must be declared.

### Anchor

An Anchor introduces a bounded outside event. Its contract declares source
identity, event schema and identity rules, ordering, duplicate/replay behaviour,
retention/cursors, authentication, and source scope. A provider notification is
not automatically an Anchor. Creating a subscription is a separate Action or
later host-owned installation operation. Event lifecycle remains J18F.

### Reserved

Job, Stream, and Human Task remain reserved and unimplemented. Job is later
completion work; Stream is continuing/high-volume data requiring bounded
reduction; Human Task is explicit human work or judgement. MCP Tasks, progress,
and elicitation do not automatically become these classes.

## 4. Effect Model

Effects are explicit, complete, conservative, ordered canonically, covered by
the manifest digest, understood by the host, and reviewed before installation.
Unknown effects fail closed; advertisements cannot remove or weaken effects and
providers cannot dynamically introduce trusted effects.

Illustrative host vocabulary includes observation (`data.read`,
`metadata.read`, `state.observe`), mutation (`data.create`, `data.update`,
`data.delete`, `data.move`, `data.copy`), communication (`message.send`,
`content.publish`, `notification.emit`), resource/execution (`process.execute`,
`model.infer`, `compute.consume`, `storage.consume`), administration
(`identity.manage`, `permission.change`, `configuration.change`), and physical
(`device.control`, `physical.actuate`). These are architecture vocabulary, not
an exhaustive registry or machine schema. Arbitrary dotted strings are not
trusted without an accepted effect definition or reviewed adapter.

An effect definition preserves semantic identity, target kind, observational or
mutating nature, locality, persistence, privacy, resource/financial impact,
reversibility, and deterministic or probabilistic behaviour. Existing manifest
reversibility and determinism fields remain authoritative and are not duplicated.

The declared set is the maximum honest description of one invocation. Deliberate
secondary effects, AI probabilistic behaviour, physical effects, cost/quota,
and data transfer must be declared. If observed behaviour exceeds it, evidence
is retained, the binding becomes suspect, and further calls fail closed.

## 5. Structured Scope

Scope is a structured host-readable limit answering which files, repositories,
accounts, workspaces, database objects, recipients, origins, devices, times,
quantities, costs, rates, or classifications are allowed. It is not prose,
provider-executed policy, arbitrary code, or a hidden predicate.

Four layers remain separate:

1. **Capability-supported scope:** reviewed maximum dimensions and constraint
   forms; a claim, not a grant.
2. **Installation grant:** host-owned administrator approval, which may only
   narrow the supported maximum.
3. **Policy constraint:** host policy narrowing the grant for a set, caller, or
   situation.
4. **Resolved target:** actual invocation argument or inbound event identity
   extracted by an explicit reviewed binding.

Effective scope is the intersection of supported scope, installation grant,
active policy constraints, and resolved invocation or event target. A provider
may enforce a stricter boundary but never widen the host result. An empty or
unresolved intersection fails closed.

## 6. Binding and Constraint Rules

The host assesses scope only through an explicit reviewed binding identifying
dimension, trusted argument or event location, comparison rule, normalization,
and any approved adapter. JSON Pointer may identify argument locations.

Scope is never inferred from argument names, prose, filenames in descriptions,
provider documentation, model judgement, runtime output, or a provider safety
claim. Unknown mappings fail closed. Reserved constraint families include exact
identifier, approved set, hierarchical prefix, numeric range, quantity limit,
time window, origin/domain set, and structured selector interpreted by an
approved host adapter. Serialized field names are not frozen here, and regular
expression policy or executable user predicates are not introduced.

Multiple dimensions all must pass. A missing target or one failed dimension
cannot be compensated by another.

## 7. Accepted 0.2 Compatibility

Existing 0.2 manifest and runtime behaviour remains valid. The implemented
bounded `path_prefix` scope and existing explicit `unrestricted` vocabulary are
preserved until a deliberate implementation decision. Unrestricted is an
explicit maximum, never an implicit default; it grants no permission, bypasses
no policy or confirmation, and should be refused where the host cannot reason
about effects. J18E does not claim broader scope machinery is implemented.

## 8. Class-Specific Scope

Action mutable, communicative, execution, and physical targets must resolve
before dispatch. An unresolved target is unavailable or denied unless a
deterministic approved adapter assesses it.

Query scopes cover source, account, collection, identifier, fields or
classification, output quantity, time range, cost, and quota as relevant.
Read-only remains permissioned.

Anchor scopes cover provider/source, account/workspace/device, event types,
filters, cursor/replay range, event volume, and retention. The host validates
source and scope before admitting an Anchor; lifecycle remains J18F.

## 9. Policy, Confirmation, and Safety

Class, effects, and effective scope are policy inputs, not policy decisions.
The existing four-outcome vocabulary remains authoritative. Confirmation binds
capability identity/version, manifest digest, provider binding, arguments,
resolved scope, declared effects, and durable execution identity where relevant.
Approval for one scope is not approval for a wider scope and does not survive
relevant manifest, provider, effect, or scope drift.

Reversibility, determinism, idempotency, confirmation, and scope remain
independent. Idempotency does not authorise retry; no automatic retry is
introduced. Privacy, remote transfer, publication, model inference, persistent
storage, financial, quota, compute, and rate consequences must be declared and
bounded where feasible. Credentials remain host-owned; J18F and J18G govern
lifecycle, outcomes, secrecy, and sandbox enforcement.

## 10. Versioning and Drift

A new capability version is required for class, effect semantics or set,
supported scope dimensions, binding meaning, trusted input/output contract,
target interpretation, or breaking compatibility changes. Any authoritative
manifest change requires a new manifest digest. Live discovery drift never
silently changes class, effects, or scopes; the binding is revalidated or made
unavailable.

Core receives only the accepted planning projection required by Tether 0.1. It
does not read complete manifests, package details, full policy, credentials,
live provider truth, or dispatch authority. J18E adds no syntax, condition
semantics, or planner judgement.

## 11. First Envelope and Examples

The intended first Plug Kit may target Action, Query, Anchor, the existing
bounded path-prefix scope, and only later-proven deterministic scope additions.
File Tools and PDF Tools remain examples; Anchor delivery is J18F. J18E alone
authorises no implementation.

Examples remain non-executable: file move is Action with `data.read` and
`data.move` plus source/destination prefixes and no overwrite; PDF extraction
is Query with `data.read` and `compute.consume` plus input, size, page, and
output limits; issue search is Query with approved account/repository and result
limits; email is Action with send/read effects and sender, recipient,
attachment, size, and rate limits; remote AI requires model/compute and remote
transfer effects with provider, classification, token/cost, and output limits;
smart-lock control is Action with physical/security effects, exact device,
operation, time window, and explicit approval. Hard-real-time certification is
outside Tethers' promise.

## 12. Refusal and J18H

Refuse or mark unavailable for unknown class/effect, incomplete or contradictory
effects, unsupported dimensions, ambiguous targets, missing bindings,
non-deterministic comparison, empty effective scope, out-of-scope invocation,
provider drift, undeclared provider effects, unbounded physical/security risk,
or unrepresentable privacy/cost consequences.

J18H must validate that every required integration is representable without
syntax changes, vendor policy in Core, prose authority, trusted annotations,
hidden effects, unbounded permission, or false certainty. Final freeze remains
gated on J18H.

## 13. Acceptance

J18E passes review when class, effect, scope, policy, and outcome are distinct;
each capability has one reviewed class; Action, Query, and Anchor are clear;
reserved classes remain unimplemented; effects are complete and fail closed;
scope layers intersect through explicit deterministic bindings; Query remains
permissioned; Anchor admission is bounded; 0.2 path-prefix behaviour is intact;
drift cannot silently alter bindings; Core remains application-agnostic; no
retry, syntax change, implementation, or machine schema is introduced.
