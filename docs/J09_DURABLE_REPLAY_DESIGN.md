# J09 Durable Replay Protection Design

Status: authoritative Red implementation design
Date: 2026-07-26
Base: `main` at `e679338e2887510d907d3b1c77eaf7a922dfad37`

## Contract

J09 makes one host-owned execution identity a durable single-use authority. Once an identity reaches durable intent, it must never produce another provider call, including after restart. J07 deadline/uncertainty and J08 uncertain Result Anchor intent were absorbed into accepted J06; they remain historical provenance only. J09 adds no retry, compensation, recovery executor, approval restoration, planner/OCaml, manifest, protocol, provider-idempotency, or J10 queue work. It does not reconstruct a missing ledger from Trail entries or import safety-branch material.

`ExecutionId` is host-created once before dispatch: opaque canonical lower-case UUID v4 prefixed `exec_`. It is never caller-selected, regenerated, or reused. It binds ActionId, evaluation ID, capability name/version, manifest digest, provider identity, and an RFC 8785/JCS SHA-256 digest of non-secret resolved arguments. Same-identity binding mismatch is a security error and blocks. The host owns the configured private data root. Replay state is separate from the Trail at `<host-data-root>/replay/v1/`, never derived from Action data or provider path. One checksum-protected versioned record exists per identity, named by SHA-256 of canonical identity. It contains binding, state, generation and predecessor digest, and only redacted metadata—never credentials, secrets, raw arguments, paths, payloads, stderr, or stacks.

## Persistence and states

Publish through a same-directory temporary file: write complete canonical content and checksum, flush and make the file durable, atomically publish without replacing an existing identity, then make the directory durable where supported. Native Windows implementation must prove equivalent durable atomic publication; otherwise startup and lookup fail closed. Records are never edited in place.

| State | Meaning | Repeat provider call |
| --- | --- | --- |
| `intent_recorded` | durable intent; no trusted final outcome | blocked; manual resolution |
| `invocation_armed` | durable pre-call marker; an effect may follow | blocked; manual resolution |
| `succeeded` | trusted J06 success final | blocked permanently |
| `failed` | trusted J06 known failure final | blocked permanently |
| `uncertain` | durable J06 uncertainty final | blocked; manual resolution |

There is no retryable, compensated, or reusable pending state. Known failed execution is replay-blocked. A new deliberate attempt needs a new identity and, when policy says ask, a new J05 approval. Failed intent recording never permits reuse because partial persistence cannot be excluded.

## Ordering and crash windows

1. Fresh resolution, schema, scope, pin, policy, and J05 checks.
2. Atomically consume J05 approval where applicable.
3. Durably publish `intent_recorded`.
4. Append the existing durable Trail intent.
5. Start J06 monotonic deadline.
6. Durably publish `invocation_armed`.
7. Cross J06 provider invocation boundary and make at most one provider call.
8. Classify with J06.
9. Durably append J06 outcome.
10. Publish matching final replay state.
11. Create one standard Result Anchor only after 9 and 10 succeed.

Every failure/crash fails closed. Before 3 no provider call is allowed and consumed approval stays consumed. After 3, missing final state is manual-only even if the call had not started. After 6, `invocation_armed` never claims an effect happened but never authorises retry. A Trail outcome is audit evidence only if final replay publication fails. After final publication but before Anchor creation, J09 synthesizes no Anchor on replay.

## Restart and duplicate behaviour

Ledger state is authoritative. Missing established root/directory, unreadable storage, malformed/checksum-invalid/partial record, duplicate identity, invalid generation chain, unsupported format, permission anomaly, or binding mismatch is `ReplayPersistenceUnavailable`: startup and lookup fail closed. The host never silently creates an empty ledger on established storage. Explicit pre-execution host provisioning is the only valid creation of empty ledger. `intent_recorded` without trusted final outcome, `invocation_armed`, and `uncertain` are redacted manual-resolution items. J09 provides no state-changing resolution command, reconstruction, retry, compensation, or approval restoration. Future recovery needs a new identity unless separately designed and authorised.

Admission is checked before dispatch preparation and again immediately before provider boundary under single-identity exclusion. A duplicate before call finds `intent_recorded`; one after possible invocation finds `invocation_armed`, `uncertain`, `succeeded`, or `failed`. All make zero calls. The host returns redacted local `replay_blocked_completed_success`, `replay_blocked_completed_failure`, `replay_requires_manual_resolution`, or `replay_persistence_unavailable`. This is not a J06 outcome and produces no standard Result Anchor. Original attempted execution alone may have the J06 Anchor after durable outcome and final replay state.

## J05, J06, and J10 separation

J05 approval consumption occurs before replay intent and is never restored by persistence failure, restart, failure, or uncertainty. J06 remains authoritative for monotonic time and Unattempted/Succeeded/Failed/Uncertain; ledger intermediate states are replay evidence, not effect claims. No retry or compensation is authorised. J09 does not queue/evaluate Result Anchors, deduplicate event IDs, change causal generation, or recurse into events: J10/J11 remain separate.

## Test seam, audit, and verification matrix

J09 adds host-owned `ReplayLedger`: a file-backed implementation plus deterministic test double. The double injects read, validation, write, flush, publish, and directory-durability faults and exposes ordering. J06's controllable monotonic clock remains timing seam. Audit may contain identity, state, pins, and argument digest, but never raw values/private diagnostics.

1. IDs canonical/unique; caller-supplied, malformed, and reused IDs fail.
2. Existing-identity binding mismatch fails closed.
3. Host-owned ledger; no secrets or raw diagnostics.
4. `intent_recorded` durable before Trail intent and call.
5. Trail-intent failure blocks and does not restore J05 approval.
6. Deadline starts after both intent boundaries; no wall clock decides admission.
7. `invocation_armed` durable before provider boundary.
8. Duplicate before invocation: zero calls and Anchors.
9. Crashes after intent or armed marker: manual resolution.
10. J06 success, known failure, uncertainty: outcome then final state then one Anchor.
11. Outcome/final-publication failure: no retry and no Anchor.
12. Crash after outcome/final state: no runnable replay or duplicate Anchor.
13. Completed success/failure permanently blocked; uncertainty manual-only.
14. New deliberate attempts use new identities; cannot overwrite records.
15. Missing, corrupt, partial, unreadable, duplicate, unsupported, or invalid-chain persistence fails closed.
16. Temporary files, collision, and injected durability faults block safely.
17. Deterministic seam proves ordering and redacted audit.
18. No retry, compensation, J10 queueing, planner, manifest, protocol, or safety-branch code.

## Implementation boundary

J09 may add focused host replay-ledger code, tests, and minimal dispatch orchestration. It must not change Tethers Core, OCaml, capability manifests, MCP messages, provider contracts, Result Anchor queueing, or the preserved safety branch.
