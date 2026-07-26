# J09 Durable Replay Protection Design

Status: authoritative Red implementation design
Date: 2026-07-26
Base: `main` at `e679338e2887510d907d3b1c77eaf7a922dfad37`

## Contract and scope

J09 makes one host-owned execution identity a durable, single-use authority.
Once the host admits an identity, no path may make another provider call for
that logical execution, including after process restart. J07 deadline/
uncertainty and J08 uncertain Result Anchor intent were absorbed into accepted
J06; they are historical provenance only.

J09 adds no retry, compensation, recovery executor, approval restoration,
planner/OCaml, manifest, provider, MCP protocol, or J10 queue work. It does not
reconstruct a ledger from Trail entries, import safety-branch material, or make
the Trail the replay authority.

The accepted J06 Trail remains the audit record. The J09 ledger is a separate,
host-owned admission and replay authority. A Trail write never replaces a
required ledger transition, and a ledger write never implies that a provider
call happened.

## Terms and host-owned identity lifecycle

`AnchorEventId` is the existing input Anchor's event ID. It is a logical
execution reference, not an execution identity and not a permission. The host
must validate it as a non-empty canonical event ID before it evaluates a
dispatchable Action. A caller can supply neither an `ExecutionId` nor a replay
path, ledger record, or recovered identity.

`LogicalExecutionKey` is the SHA-256 digest of RFC 8785/JCS bytes for:

```json
{"format":"tethers-logical-execution-v1","anchor_event_id":"<AnchorEventId>"}
```

Only the digest is persisted. A distinct Anchor event ID is ordinary new work;
reusing an Anchor event ID means resuming or repeating the same logical
execution. A caller that chooses a new Anchor event ID is proposing new work,
not supplying a replacement execution identity. J11's later event
deduplication may add its own event rule, but J09 does not wait for or alter it.

`ExecutionId` is an opaque, canonical lower-case UUID v4 prefixed `exec_`. The
host creates it exactly once, inside the successful fresh identity-claim
operation described below. Its constructor is private to the host admission
module. The current public tuple-style `dispatch::ExecutionId` construction is
therefore an implementation seam to remove in J09, not a caller contract.

`ExecutionBinding` contains only the exact non-secret dispatch proof fields:

- evaluation ID and Action ID;
- capability name and semantic version;
- manifest digest and provider identity; and
- `argument_digest`, the SHA-256 digest of RFC 8785/JCS bytes of the complete
  resolved non-secret arguments.

`binding_digest` is SHA-256 over the canonical complete binding. Both the
individual fields and the digest are validated; the digest is not a substitute
for field-by-field comparison. Credentials, dispatch-injected secrets, raw
arguments, paths, payloads, stderr, and stacks never enter the ledger.

The identity lifecycle is fixed as follows.

1. The host reads the existing Anchor event ID at input admission, validates it,
   and derives `LogicalExecutionKey`. It creates no UUID yet.
2. Fresh ordinary resolution, schema, scope, manifest/provider pins, and
   policy are evaluated. The host then builds the exact `ExecutionBinding`.
3. For an ordinary Allow, or an approved Ask resume, the host calls
   `admit_or_recover(LogicalExecutionKey, ExecutionBinding)` before durable
   intent. Under the logical-key lock it either recovers an existing claim or
   creates the one fresh host UUID and its immutable claim.
4. The returned private `HostExecutionAdmission` carries the host-issued
   `ExecutionId`, the validated binding, and the still-held identity exclusion.
   Only this private value may reach the J09-aware intent boundary.
5. On restart or a repeated input, the same Anchor event ID derives the same
   logical key. The host reads the immutable claim, validates every binding
   field, recovers the stored host UUID, and selects its ledger state. It never
   generates a replacement UUID for an existing logical key.
6. A different binding for an existing logical key is
   `ReplayBindingMismatch` and fails closed. It neither consumes approval nor
   reaches intent, provider, outcome, or Result Anchor handling.
7. Ordinary new work must carry a fresh Anchor event ID, which yields a new
   logical key and permits one fresh host-created UUID. Terminal identities are
   never reused, overwritten, or reassigned.
8. J12 and J13 preserve this lifecycle by passing the already-existing Anchor
   event ID unchanged into the local host admission seam. `check` performs no
   admission. `run` calls host admission; a repeat uses the same input Anchor
   event ID. `trail` may look up by the host-derived logical-key digest. No
   execution UUID crosses the planner, manifest, provider, or MCP boundary,
   and no new public configuration field is required.
9. The current implementation seam is `main.rs` before
   `authorise_and_execute_inner`, which already has the input event ID and the
   fully resolved Action. J09 inserts host admission there and passes the
   private admission into a narrowed dispatch preparation API. It must stop
   deriving an execution identity from planner `evaluation_id`.
10. The J09 implementation must prove both that a completed execution,
    restarted with the preserved Anchor event ID, makes zero provider calls,
    and that a simulated restart cannot accidentally regenerate a new UUID for
    the same Anchor event ID. No protocol or product decision remains: this
    uses the existing host Anchor event-ID seam only.

## Immutable storage model

J09 uses one immutable identity claim plus an immutable, per-identity
generation chain. It has no mutable head pointer, no in-place record edit, and
no scan of Trails to invent state.

The provisioned host data root has this exact layout. `<lk>` and `<eid>` below
are lower-case SHA-256 hex digests, not raw event IDs or arguments. `<nonce>`
is a host-generated lower-case UUID without the `exec_` prefix.

```text
<host-data-root>/replay/v1/
  FORMAT.json
  locks/
    <lk>.lock
  claims/
    <lk>.claim.json
    <lk>.<nonce>.tmp
  chains/
    <eid[0..1]>/
      <eid>/
        g0000000000000000.json
        g0000000000000001.json
        g0000000000000002.json
        g<generation>.<nonce>.tmp
```

`FORMAT.json` is created during explicit host provisioning with create-new
semantics and records `replay_format_version: 1`. J09 never silently creates an
empty established root at startup or lookup. The root must be owned and
writeable only by the host identity; reparse points, network filesystems,
unexpected ACLs, and unsupported volume types are unavailable rather than
best-effort.

The immutable claim is the identity-claim primitive. Its canonical payload
contains:

- `record_kind: "identity_claim"` and `ledger_format_version: 1`;
- logical-key digest, host-created `ExecutionId`, and execution-ID digest;
- the complete redacted `ExecutionBinding` and `binding_digest`; and
- `claim_digest`, SHA-256 of the canonical payload excluding `claim_digest`.

The claim filename is the logical-key digest. It is atomically created without
replacement. A successful claim is durable before any J05 approval consumption,
intent Trail entry, or provider call. An already-existing claim is read and
validated; it is never updated.

Each generation is a separate canonical JSON payload with an enclosing checksum
and these fields:

- `record_kind: "replay_generation"`, format version, logical-key digest, and
  execution-ID digest;
- integer `generation`, encoded in a 16-digit zero-padded filename;
- `state`;
- `predecessor_digest`; and
- `record_digest`, SHA-256 of the canonical payload excluding `record_digest`.

Generation zero has predecessor equal to `claim_digest`. Every later generation
has predecessor equal to the complete preceding generation's `record_digest`.
The allowed chain is exact:

| Generation | State | Required predecessor | Meaning |
| --- | --- | --- | --- |
| 0 | `intent_recorded` | claim digest | durable admission to intent; no effect claim |
| 1 | `invocation_armed` | generation 0 digest | provider call may follow; no effect claim |
| 2 | `succeeded`, `failed`, or `uncertain` | generation 1 digest | J06 final class, with the redacted durable-outcome digest |

There is no generation three and no direct transition around a missing
predecessor. An unattempted J06 path may remain at generation zero; it is still
manual-resolution-only and is not a retry authority. A final generation binds
the digest of the successfully durable J06 outcome entry. It may be published
only after that outcome write succeeds.

To select current state, the host reads the claim and every exact generation
filename in the identity directory while holding the per-logical-key lock. It
checks canonical bytes, checksums, claimed identity and binding, contiguous
numbering, the allowed state sequence, and predecessor digests. The highest
valid contiguous generation is current only if there are no extra, missing, or
invalid records. The claim with no generation is selected as
`claimed_no_state`. `claimed_no_state`, `intent_recorded`, `invocation_armed`,
and `uncertain` require manual resolution. `succeeded` and `failed` are
permanently replay-blocked. No selected state permits a provider retry.

## Claim, publication, concurrency, and crash rules

`admit_or_recover` first acquires an OS-backed exclusive lock on
`locks/<lk>.lock`. Lock ownership is the operating-system lock, never the
existence, age, or contents of the lock file. Windows uses a handle opened with
sharing disabled plus a documented exclusive file/byte-range lock; the lock is
released by handle close or process death. If exclusive cross-process locking
cannot be acquired and proved, the host returns `ReplayPersistenceUnavailable`
before J05 consumption.

Under that lock:

1. A valid existing claim is recovered and its complete binding is compared.
   Any selected state blocks dispatch. A binding mismatch, unreadable claim, or
   incomplete claim blocks before approval consumption.
2. If no claim or keyed temporary exists, the host creates one random
   `ExecutionId`, writes the complete claim to a same-directory temporary file,
   flushes it, atomically publishes it without replacement, and proves the
   publication durable. Only then is the fresh admission returned.
3. A second same-identity writer waits for the first writer's lock or fails
   closed. Once it acquires the lock it observes the claim/chain and is a
   replay block; it never creates another identity or consumes an approval.

Generation publication uses the same sequence: create a uniquely named
same-directory temporary file with create-new semantics; write the complete
canonical record; flush the file to durable storage; atomically publish the
final generation filename without replacement; prove the containing-directory
metadata durable; then remove the temporary only after publication has been
verified. The publication operation is idempotent only in this narrow sense: if
the expected final filename already exists with the exact expected digest and
valid predecessor, the current in-flight operation may treat that generation
as already published. A different existing payload, duplicate generation, or
replacement attempt is corruption and fails closed.

A process may not repair, complete, delete, or reuse an identity automatically:

- A crash after claim publication but before generation zero leaves
  `claimed_no_state`; recovery is manual-only.
- A crash after generation zero but before the Trail intent leaves
  `intent_recorded`; recovery is manual-only and does not append a compensating
  Trail entry or retry.
- A crash between generations leaves the preceding state selected and
  manual-only. In particular, `invocation_armed` does not claim an effect but
  prohibits another call.
- A temporary, orphaned identity directory, keyed temporary, unknown filename,
  partial record, or abandoned lock-file contents are evidence that publication
  cannot be proved. The affected logical key fails closed and needs an
  separately authorised maintenance process; J09 performs no cleanup.
- A checksum failure, malformed JSON, unsupported version, generation gap,
  duplicate numbered generation, predecessor mismatch, unexpected terminal
  extension, permission error, or root anomaly is
  `ReplayPersistenceUnavailable`. It produces zero intent, provider calls,
  outcome writes, and standard Result Anchors.

The filesystem remains a trusted host boundary. The lock prevents cooperative
host processes from interleaving transitions; immutable create-new publication
and full-chain validation detect direct concurrent writers rather than trusting
them. A process may hold the lock from admission through final generation
publication. It must not release it between replay admission and provider
boundary.

### Native Windows durability gate

The active implementation environment is native Windows. A J09 Windows backend
is admissible only when it proves every required primitive on the actual local
filesystem:

1. open temporary and final files with no-follow/reparse-safe, create-new
   semantics on the same local volume;
2. write and `FlushFileBuffers` the complete temporary file successfully;
3. atomically create the final name without replacement (not
   `MOVEFILE_REPLACE_EXISTING`, `ReplaceFile`, copy, or a replace-capable
   rename);
4. prove the final name and parent directory metadata durable after publication;
   and
5. acquire and hold the documented cross-process exclusion above.

The implementation may use a tested native primitive such as a same-volume
atomic no-replace link/create only if its no-replace and crash-durability
properties are demonstrated for the supported Windows filesystem. It must not
assume that a Rust `rename`, a best-effort head file, a successful data-file
flush, or a directory handle alone proves durable publication. Where Windows or
the chosen volume cannot prove a required primitive, ledger initialisation,
lookup, claim, or generation publication returns
`ReplayPersistenceUnavailable`; no J05 approval is consumed and no dispatch is
allowed. This is the required fail-closed result, not a portability fallback.

## Dispatch ordering and J05/J06 interaction

The normal and Ask-resume orders are fixed.

1. Validate host input and derive the logical key; perform fresh resolution,
   pins, schema, scope, and ordinary policy evaluation.
2. For a fresh Ask request, create only the J05 pending approval. It creates no
   execution identity and makes no replay claim.
3. For Allow, and for an approved Ask resume after all J05 fresh checks, acquire
   replay admission under the logical-key lock. Existing, blocked, corrupt, or
   mismatched identities stop here.
4. Only a fresh Ask resume with a newly admitted identity atomically consumes
   the J05 approval. Thus a duplicate or blocked identity never consumes a new
   approval merely to discover that replay is forbidden. A consumption failure
   leaves the immutable claim manual-only; the approval is never restored.
5. Publish generation zero `intent_recorded`, then append the existing durable
   Trail intent. Failure of either prevents deadline start and provider call.
6. Start J06's monotonic deadline only after both intent boundaries succeed.
   Immediately before the provider boundary, publish generation one
   `invocation_armed`. Its failure prevents the call.
7. Cross J06's provider invocation boundary and make at most one provider call.
8. Classify with J06 and append its durable outcome. If that write fails, retain
   the J06 in-memory classification but leave generation one replay-blocked;
   make no Anchor and never retry.
9. Publish matching generation two final state, then create the one standard
   Result Anchor. If final publication or Anchor creation fails, make no retry;
   a later replay remains blocked and J09 synthesizes no Anchor.

No unaudited request can reach intent, dispatch, outcome, or a standard Result
Anchor. The ledger never claims a provider effect: `invocation_armed` only says
that a call may follow, while final classes are copied only after J06's durable
outcome. Approval remains one-shot and is not stored in, restored from, or
reused by the replay ledger.

## Fail-closed results

The host returns only redacted local replay results:

- `replay_blocked_completed_success`;
- `replay_blocked_completed_failure`;
- `replay_requires_manual_resolution`; or
- `replay_persistence_unavailable`.

They are not J06 outcomes and create no standard Result Anchor. They expose no
raw key, payload, path, error, diagnostic, or storage implementation detail.

## Test seam and numbered verification matrix

J09 adds a host-owned `ReplayLedger` abstraction with a file-backed backend and
a deterministic test double. The double models claim, lock, validation, write,
file flush, no-replace publish, directory durability, crash cut points, and
cross-process interleaving. J06's controllable monotonic clock remains the
timing seam. Every item below needs one named focused test or an explicit
one-to-one mapping to an existing regression.

1. Explicit provisioning creates `FORMAT.json` once; lookup never creates an
   established root.
2. Exact layout names use only logical-key and execution-ID digests; durable
   records contain no secrets, raw arguments, paths, payloads, or diagnostics.
3. The host, not the request, creates canonical UUID `ExecutionId`; supplied or
   substituted execution-ID fields are rejected.
4. The same Anchor event ID across restart recovers the same host UUID.
5. A completed recovered identity makes zero provider calls and no duplicate
   Result Anchor.
6. A simulated host restart cannot accidentally generate a new UUID for the
   same Anchor event ID.
7. A new Anchor event ID creates a distinct logical key and fresh UUID.
8. Existing logical key plus changed binding fails before approval, intent, or
   provider work.
9. Claim is durable before J05 consumption; crash after claim before generation
   zero selects `claimed_no_state` and is manual-only.
10. Generation zero is immutable, follows the claim digest, and precedes Trail
    intent and every provider call.
11. Generation one is immutable, follows generation zero, and precedes every
    possible provider call.
12. Final generation follows generation one, includes the durable J06 outcome
    digest, and precedes the standard Result Anchor.
13. Only the exact `0 -> 1 -> 2` chain and final-state vocabulary are accepted.
14. Current state is selected by full contiguous-chain validation, never a
    mutable head pointer.
15. Gap, duplicate number, invalid predecessor, checksum failure, malformed
    record, unsupported version, or unexpected extension fails closed.
16. A crash after generation zero before Trail intent makes zero calls on
    restart and writes no compensating Trail entry.
17. A crash between generation zero and one is manual-only and makes zero calls.
18. A crash after generation one before/during/after a possible provider call
    is manual-only and never retries.
19. Crash after durable J06 outcome but before final generation is replay
    blocked with no synthetic Anchor.
20. Crash after final generation but before Anchor is replay blocked with no
    duplicate Anchor.
21. Keyed temporary files, orphan identity directories, and partial files fail
    closed and are never silently deleted by J09.
22. Temporary write, file flush, no-replace publish, directory-durability, read,
    validation, permission, and lock failures each block before dispatch.
23. Two concurrent same-key admissions create exactly one claim and one UUID.
24. A competing writer observes the first claim/chain after exclusion and makes
    zero provider calls.
25. A duplicate generation publication is accepted only when its expected
    immutable bytes and predecessor already validate; any other collision fails
    closed.
26. Cross-process lock loss, unavailable lock primitive, or unsupported volume
    fails before J05 consumption and dispatch.
27. Existing replay-blocked success, failure, uncertain, armed, intent, and
    claim-only states all consume zero additional J05 approvals.
28. Fresh approved Ask consumes exactly once only after fresh replay admission;
    claim, J05, intent, and armed ordering is observable in the test double.
29. J05 consumption failure leaves no dispatch authority; its claim is
    manual-only and approval is not restored.
30. Trail-intent failure after generation zero, armed-publication failure, and
    pre-invocation deadline expiry each make zero provider calls.
31. J06 known success, known failure, and uncertainty write outcome then final
    generation then exactly one corresponding standard Anchor.
32. J06 outcome write failure or final-generation failure preserves the known
    classification where applicable, creates no Anchor, and never retries.
33. Terminal success and failure are permanently replay-blocked; terminal
    records are never reused or overwritten.
34. Incomplete and uncertain states are manual-only; J09 has no automatic
    resolution, compensation, approval restoration, or recovery executor.
35. J12/J13 integration preserves only the existing Anchor event ID at host
    admission and does not alter planner, manifest, provider, or MCP messages.
36. Full Rust, host integration, protocol, OCaml, packet, whitespace, complete
    diff, Windows primitive, and restart tests pass from the accepted J06 base.

## Implementation boundary

J09 may add focused host replay-ledger and host-admission code, tests, and the
minimal dispatch orchestration needed to carry a private admission. It must not
change Tethers Core, OCaml, capability manifests, MCP messages, provider
contracts, Result Anchor queueing, the public configuration format, or the
preserved safety branch.
