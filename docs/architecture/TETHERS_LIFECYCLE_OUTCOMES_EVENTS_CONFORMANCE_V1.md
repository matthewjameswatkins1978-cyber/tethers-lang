# Tethers Lifecycle, Outcomes, Events and Conformance v1

Status: Accepted J18F lifecycle contract
Accepted by Lucy: 2026-08-01
Final architecture freeze: Requires J18H paper validation
Contract generation: 1
Implementation: Not authorised

## 1. Central distinctions

Package trust, host-owned installation, provider-session lifecycle, health,
catalogue, capability binding, policy, operation attempt, canonical outcome,
replay, external-event admission, conformance, and Trail are separate state
families. They are not compressed into one generic status. Trust is evidence;
installation is admitted material; health is observation, not authority; policy
permits a bounded request; attempt records whether the provider boundary was
crossed; outcomes report honest knowledge; replay and event admission prevent
unsafe repetition; conformance records bounded tests; Trail records history.

## 2. Installation lifecycle

The host-owned conceptual sequence is: package received; archive inspected
without execution; validated; extracted into quarantine; manifests and
compatibility validated; test configuration created; provider launched in
isolated test mode; conformance run; provider stopped; evidence reviewed;
installation approved; exact bindings created; present but disabled; explicitly
enabled; operational use. Refused, failed conformance, disabled, degraded,
unavailable, quarantined, removal pending, and removed are side conditions, not
frozen serialized names. Conformance success does not approve, install, or enable
a Plug. Failure before approval creates no active binding.

## 3. Provider session lifecycle

One session is one bounded running instance:

```text
stopped -> starting -> establishing -> discovering -> ready
        -> stopping -> stopped
```

Startup, establishment, discovery, protocol, process, deadline, and forced
shutdown failures remain distinguishable. Restart creates a new process identity
where available, a new Socket session identity, and new initialization and
discovery evidence. It never resumes or retries an incomplete invocation.

## 4. Provider observations

Session lifecycle records phase. Health is `healthy`, `degraded`, or
`unavailable`; ping is liveness evidence only. Catalogue is `unknown`, `current`,
`stale`, or `invalid`; `notifications/tools/list_changed` makes it stale.
Binding is `exact and available`, `stale`, `missing`, `drifted`, `disabled`,
`unavailable`, or `quarantined`. A healthy provider does not make a stale
binding invocable.

## 5. Operational readiness

Readiness is capability-specific. It requires enabled installation, current
package and manifest pins, established session, exact protocol negotiation,
complete current discovery, matched provider identity, present operation,
compatible live schemas, exact binding, credentials/configuration, resolvable
scope, permitting policy, no quarantine, and current conformance evidence. One
failed capability does not fail every capability, although shared compromise may
quarantine all bindings.

## 6. Operation lifecycle

The conceptual route is: deterministic Plan; exact capability/provider
resolution; live discovery and pin revalidation; arguments and scope; policy;
approval; replay admission; durable intent; monotonic deadline; armed
invocation; provider boundary; provider observation; trusted validation;
canonical classification; durable outcome; replay-terminal publication; Result
Anchor when authorised by durable evidence; Trail presentation. These stages
need not each be serialized.

## 7. Attempt boundary

Before provider invocation may begin, work is `unattempted`; once invocation may
have begun, it is `attempted`. Unattempted is not a fourth canonical outcome.
Denied, approval-required, unavailable, stale binding, replay blocked, replay
persistence unavailable, invalid scope, pre-boundary deadline, cancellation,
and process failure before the boundary are unattempted dispositions. They make
no provider call, canonical outcome, or standard Result Anchor, though redacted
admission/refusal evidence may exist.

## 8. Canonical outcomes

Attempted operations have exactly `succeeded`, `failed`, and `uncertain`.
Succeeded requires attempted invocation, trustworthy timely final success,
required structured output, trusted-schema validity, and contract fulfilment.
Failed requires trustworthy final evidence of non-fulfilment, including explicit
provider error, trusted refusal, schema-invalid success, proven no-effect
cancellation, or known incomplete work. Uncertain applies when invocation may
have begun but final truth is unavailable: in-flight timeout, process/connection
loss, malformed or late response, protocol interruption, inconclusive
cancellation, unknown partial extent, or unproven provider claims.

Unavailable, denied, approval-required, replay-blocked, audit-failed, and
unattempted are dispositions or audit conditions, not outcomes. Ambiguity after
invocation remains uncertain.

## 9. Partial completion and cancellation

There is no `partially_completed` outcome. Partial data is succeeded only when
the trusted contract defines it as fulfilment and validates it; otherwise known
incomplete work is failed and unknown extent/effect is uncertain. Cancellation
before invocation is unattempted. After invocation it is failed only with proof
of no completed effect, otherwise uncertain. Accepted success cannot be
rewritten. No MCP cancellation mapping is invented; the first Plug need not
implement cancellation.

## 10. Audit failure

Known in-memory classification remains known after audit failure. Audit failure
does not downgrade success or failure to uncertainty. No standard Result Anchor
is created when either required durable publication fails. Replay-terminal
failure never authorises another provider call; incomplete replay state requires
manual resolution. Raw diagnostics remain redacted.

## 11. Result Anchors

The standard names remain `capability.succeeded`, `capability.failed`, and
`capability.uncertain`. The host classifies first. Durable outcome and replay-terminal
publication must both succeed before Result Anchor creation.
Only after both durable publications succeed does it attempt exactly one
standard Result Anchor write. A successful write creates exactly one standard
Result Anchor; a failed write creates none and does not change the canonical
outcome, replay state, or no-retry boundary. Failure of either durable
publication suppresses Anchor creation. A failed publication does not authorise retry.
Unattempted work gets no
standard Result Anchor. The host creates the Anchor, never Socket. It carries
stable host identity, capability/manifest identity, provider identity,
correlation, causation, causal generation, validated result or stable redacted
error, and no raw stderr, secret, token, stack, path, or unreviewed provider
message.

## 12. Replay and restart

Replay authority remains separate from Trail. Existing states are claimed with
no state, intent recorded, invocation armed, succeeded, failed, and uncertain.
Succeeded and failed are permanently replay-blocked. Claimed, intent, armed,
and uncertain require manual resolution. Corrupt or unavailable replay storage
makes dispatch unavailable. No recovered state or new provider session permits
a provider retry. A new event/evaluation/Action tuple is new work, not recovery.

## 13. Inbound Plug Anchors

The route is: approved Anchor and binding; established session; discovery match;
explicit subscription or polling; outside event; provider/source
authentication; identity, schema, and scope validation; durable duplicate
admission; admission evidence; conversion to a Tethers Anchor; J11 admission
and causal checks; durable Trail; deterministic evaluation; acknowledgement or
cursor progression only under the accepted source contract. A provider
notification is not automatically an Anchor. Catalogue notifications remain
lifecycle evidence.

## 14. External event identity

Every Plug Anchor binds installed Plug/provider binding, Anchor capability and
version, exact source identity, source event identity, trusted schema version,
and relevant partition. A source stable ID is preferred. A host adapter may
derive an ID only from immutable canonical fields under an explicit reviewed
rule. Arrival time and independently random delivery IDs are not stable
identity. Without stable identity the source is unsupported.

## 15. Durable external-event admission

Host-owned durable admission across restart is distinct from operation replay,
Trail, and the existing J11 gate. Its conceptual key is exact source binding
plus exact source event identity. Same identity and payload digest is duplicate
redelivery and is not evaluated again. Same identity with a different payload
digest is an identity conflict that quarantines or disables the source binding.
Missing or corrupt admission fails closed; Trail cannot replace admission
authority, and admission does not prove evaluation completed.

## 16. Acknowledgement and cursors

Durable admission precedes acknowledgement that would prevent redelivery. No
acknowledgement follows admission failure. Duplicate redelivery remains safe.
Cursors are opaque source positions, not event identities; rewind does not bypass
deduplication and progression cannot skip unadmitted events. Replay range is
explicitly scoped and approved. The host records whether source order is absent,
total, partitioned, or monotonic and never fabricates order.

## 17. Root Anchors and causal limits

An admitted outside event becomes a root Anchor with stable host ID, reviewed
event name, producer/source binding, root correlation, no causation parent,
generation 0, separate source occurrence and host admission times,
schema-validated facts, source identity, and payload digest. No executable JSON
schema is frozen here.

After external admission, J11 still applies: duplicate IDs are rejected within
the active run; generation 0 through 8 is accepted; generation 9 or greater is
rejected; admission/rejection is recorded before evaluation; write failure stops
evaluation; rejected follow-ups stop the current drain; Result Anchor follow-ups
retain correlation, causation, and generation. External deduplication and J11
causal protection solve different problems.

## 18. Event replay

Approved source replay requires explicit source and time/cursor scope. Already
admitted identities remain duplicates; unseen historical identities may be
admitted. Replayed external events remain root generation-0 Anchors. Replay does
not alter operation replay rules and cannot fabricate a new identity for admitted
content.

## 19. Conformance purpose and environment

Conformance tests one exact package/provider/capability combination under bounded
conditions. It does not prove publisher trust, permission, production safety,
outside-service correctness, credential authority, absence of malicious
behaviour, future availability, or hard-real-time suitability. Package tests are
untrusted input; the host orchestrates, constrains, observes, and records them.
A provider cannot certify itself.

Run after inspection, from quarantine, before active bindings, with test-only
configuration, test credentials/scopes, J18G controls, no production Tether
Sets/effective policy, and no silent production resources. Effectful tests use a
disposable fixture, sandbox, or test account. No real production effect proves
installation.

## 20. Conformance categories

Categories cover static package checks; launch and protocol; binding agreement;
Action/Query invocation; Anchor identity, duplicate, conflict, scope, replay,
acknowledgement and generation; lifecycle start/establish/discover/ready/stale/
rediscover/shutdown/restart; and Trail/evidence ordering and redaction. They
include exact launch, clean stdout, version/provider identity, complete
discovery, bounded shutdown, schema agreement, no hidden retry, valid and
invalid inputs, one authorised provider call, trusted output validation, honest
timeout/process-loss classification, stable event identity, durable admission
before acknowledgement, durable outcome before Result Anchor, and no leaked
secrets.

## 21. Conformance evidence and invalidation

Evidence is pinned to semantic package digest, payload-file and manifest
digests, launch identity, provider version, Socket major, binding/protocol
versions, host build, platform/architecture, suite and test-config digests,
times, each case result, bounded safe diagnostics, and final pass/fail/
interrupted disposition. It contains no secrets and is immutable historical
evidence.

Evidence is stale or invalid when package, payload, manifest, capability,
launch, Socket, binding, tested platform, suite, or material security boundary
changes. Credential rotation need not erase structural evidence, but readiness
is checked separately. Policy and scope changes require fresh readiness. States
are conceptual: not run, running, passed, failed, interrupted, invalidated.
Failure/interruption creates no approval.

## 22. Activation, upgrade and removal

Activation separately records passed conformance, reviewed evidence,
package/install approval, scopes, policy, credentials, exact bindings, and
enablement. No test result performs another step. Upgrade is a fresh digest and
conformance candidate. Disablement prevents new invocations and event
admissions, stops or reports sessions, and preserves evidence. Removal removes
bindings and payload, stops provider/source, preserves Trail/replay/event-
admission/conformance evidence, and handles credentials by separate choice.

## 23. First envelope and refusal

The first Plug Kit may target one local session, serial Action/Query invocation,
three outcomes, current Result Anchors, existing replay, host conformance, File
Tools, PDF Tools, and Anchor delivery. Anchor delivery remains a first-slice
candidate, but requires a separately authorised implementation task providing
the accepted source contract and host-owned durable external-event admission
authority. J18F alone does not authorise that implementation. Cancellation,
partial-completion extensions, remote providers, Jobs, Streams, Human Tasks,
automatic updates, general subscription management, and multi-provider failover
remain deferred.

Refuse or mark unavailable when lifecycle, binding, attempt boundary, final
truth, replay authority, event identity, duplicate admission, acknowledgement,
ordering, conformance evidence, declared effects, credentials, or physical and
security-sensitive bounds cannot be established honestly.

## 24. J18H obligation

J18H must paper-validate each representative integration against installation,
session, readiness, attempt boundary, canonical outcomes, restart, event identity
where relevant, duplicate/replay, acknowledgement/cursor, conformance,
invalidation, and refusal boundaries. Final architecture freeze remains gated on
J18H.

## 25. Acceptance

Acceptance requires separate state families; capability-specific readiness;
unattempted distinct from exactly succeeded/failed/uncertain; timeout after
invocation uncertain; no Result Anchor for unattempted; durable outcome before
Anchor; replay separate from Trail and never retry-authorising; stable external
identity; durable external admission distinct from J11; acknowledgement after
admission; cursors distinct from identity; generation 0 through 8; host-run
conformance without authority; honest evidence invalidation; no automatic retry;
and no schema, implementation, or Tether syntax change.
