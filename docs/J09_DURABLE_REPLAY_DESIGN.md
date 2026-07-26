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
the Trail the replay authority. Its only configuration addition is the
host-local reference-host storage authority defined below; it changes none of
the Tethers protocol, request or response JSON, planner output, manifest,
provider contract, or public Tethers language configuration.

The accepted J06 Trail remains the audit record. The J09 ledger is a separate,
host-owned admission and replay authority. A Trail write never replaces a
required ledger transition, and a ledger write never implies that a provider
call happened.

## Terms and host-owned identity lifecycle

`AnchorEventId` is the existing input Anchor's event ID. Together with the
planner's `EvaluationId` and one proposed `ActionId`, it identifies one logical
execution; it is not itself an execution identity or a permission. The host
must validate all three as non-empty canonical IDs before it admits a
dispatchable Action. A caller can supply neither an `ExecutionId` nor a replay
path, ledger record, or recovered identity.

`LogicalExecutionKey` is the SHA-256 digest of RFC 8785/JCS bytes for:

```json
{
  "format": "tethers-logical-execution-v1",
  "anchor_event_id": "<AnchorEventId>",
  "evaluation_id": "<EvaluationId>",
  "action_id": "<ActionId>"
}
```

Only the digest is persisted. The same exact event/evaluation/Action tuple
means resuming or repeating the same logical execution. A different Action in
one ordered Plan, a different evaluation of one event, or a different Anchor
event ID is a distinct logical execution and receives a distinct claim and
host UUID. A caller that chooses a new tuple is proposing new work, not
supplying a replacement execution identity. J11's later event deduplication
may add its own event rule, but J09 does not wait for or alter it.

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

1. The host reads and validates the existing Anchor event ID. After planner
   evaluation it validates that evaluation ID and each proposed Action ID. For
   each Action it derives `LogicalExecutionKey` from that exact three-ID tuple.
   It creates no UUID yet.
2. Fresh ordinary resolution, schema, scope, manifest/provider pins, and
   policy are evaluated for that Action. The host then builds the exact
   `ExecutionBinding`.
3. For an ordinary Allow, or an approved Ask resume, the host calls
   `admit_or_recover(LogicalExecutionKey, ExecutionBinding)` before durable
   intent. Under the logical-key lock it either recovers an existing claim or
   creates the one fresh host UUID and its immutable claim.
4. The returned private `HostExecutionAdmission` carries the host-issued
   `ExecutionId`, the validated binding, and the still-held identity exclusion.
   Only this private value may reach the J09-aware intent boundary.
5. On restart or a repeated input, the same exact event/evaluation/Action tuple
   derives the same logical key. The host reads the immutable claim, validates
   every binding field, recovers the stored host UUID, and selects its ledger
   state. It never generates a replacement UUID for an existing logical key.
6. A different binding for an existing logical key is
   `ReplayBindingMismatch` and fails closed. It neither consumes approval nor
   reaches intent, provider, outcome, or Result Anchor handling.
7. Ordinary new work must carry a fresh logical tuple: a fresh Anchor event ID,
   a different evaluation ID, or a different Action ID. Each yields a new
   logical key and permits one fresh host-created UUID. Terminal identities are
   never reused, overwritten, or reassigned.
8. J12 and J13 preserve this lifecycle by passing the already-existing Anchor
   event ID unchanged into the local host admission seam and using the
   evaluation ID and Action ID already returned by the planner. `check`
   performs no admission. `run` admits each planned Action independently; a
   repeat uses the same exact tuple. `trail` may look up by the host-derived
   logical-key digest. No execution UUID crosses the planner, manifest,
   provider, or MCP boundary. J12/J13 may carry the host-local
   `--host-data-root` option unchanged, without adding a planner, protocol,
   manifest, or provider message.
9. The current implementation seam is `main.rs` before
   `authorise_and_execute_inner`, which has the input event ID, planner
   evaluation ID, and proposed Action. J09 inserts host admission there for
   each Action and passes the private admission into a narrowed dispatch
   preparation API. It must stop deriving an execution identity from planner
   `evaluation_id`.
10. The J09 implementation must prove both that a completed execution,
    restarted with the preserved exact tuple, makes zero provider calls, and
    that a simulated restart cannot accidentally regenerate a new UUID for the
    same tuple or collide with a sibling Action. No protocol or product decision
    remains: this uses existing host input, planner evaluation, and Action seams
    only.

## Host data root and explicit provisioning authority

The reference host has one explicit, host-local storage authority for normal
execution:

```text
--host-data-root <ABSOLUTE_PATH>
```

It has no default or fallback. The path is never derived from `TRAIL_PATH`, a
Trail parent, request file, working or executable directory, temporary
directory, environment variable, or provider configuration. `TRAIL_PATH`
remains independent audit storage. A branch that may dispatch must receive a
valid, already-provisioned root before J05 consumption or provider work. An
Allow or approved Ask without it returns `ReplayPersistenceUnavailable` with
zero J05 consumption, intent, provider call, outcome, or standard Result
Anchor. Evaluation-only, unmatched, denied, and pending-Ask paths do not open
or provision replay storage because they cannot invoke a provider.

The only operation allowed to establish storage is:

```text
tethers-reference-host provision-replay <ABSOLUTE_HOST_DATA_ROOT>
```

The supplied host-data root itself must already exist and be absolute. The
provisioner never guesses or creates it. It is a separate authority from normal
execution, which never calls the provisioner or creates a missing directory or
`FORMAT.json` internally. After validating the existing root as described in
the Windows substrate section, the provisioner may create only:

```text
<host-data-root>/replay/
<host-data-root>/replay/v1/
<host-data-root>/replay/v1/FORMAT.json
<host-data-root>/replay/v1/locks/
<host-data-root>/replay/v1/claims/
<host-data-root>/replay/v1/chains/
```

Every directory is created with create-new semantics, then handle-validated;
`FORMAT.json` is published with the frozen no-replace primitive, reopened, and
validated before success. An absent replay subtree may be created. The exact
complete valid v1 structure returns `AlreadyProvisioned` without mutation. A
partial, malformed, unknown, mismatched, or unsupported structure fails closed
without repair, deletion, replacement, or cleanup. Normal admission validates
the complete pre-existing structure and treats missing or incomplete
provisioning as `ReplayPersistenceUnavailable` before approval consumption or
dispatch.

## Immutable storage model

J09 uses one immutable identity claim plus an immutable, per-identity
generation chain. It has no mutable head pointer, no in-place record edit, and
no scan of Trails to invent state.

The provisioned host data root has this exact layout. `<lk>` is the lower-case
SHA-256 hex digest of the canonical event/evaluation/Action tuple; `<eid>` is
the digest of the host UUID. Neither is a raw ID or argument. `<nonce>` is a
host-generated lower-case UUID without the `exec_` prefix.

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

`FORMAT.json` is created only by the explicit provisioner with create-new
semantics and records `replay_format_version: 1`. J09 never silently creates an
empty established root at startup or lookup, and normal execution never treats
Trail storage as this root. The root must be owned and writeable only by the
host identity; reparse points, network filesystems, unexpected ACLs, and
unsupported volume types are unavailable rather than best-effort.

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
- A keyed temporary, unknown filename, partial record, or abandoned lock-file
  contents are evidence that publication cannot be proved. The affected logical
  key fails closed and needs a separately authorised maintenance process; J09
  performs no cleanup.
- A chain directory whose execution-ID digest has no valid claim with that
  execution-ID digest cannot be mapped to an affected logical key. It is
  ledger/root corruption, so the entire ledger fails closed until separately
  authorised maintenance resolves it.
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

### Supported native Windows substrate

J09 0.2 supports only native Windows on a local fixed NTFS volume. It rejects a
network filesystem; a reparse point, symlink, junction, mount-point traversal,
or unsupported volume; and every anomalous or unprovable storage condition.
Unsupported storage returns `ReplayPersistenceUnavailable` before J05 approval
consumption or dispatch.

The proof boundary is the documented Windows/NTFS API contract plus runtime
verification that every required call succeeds. J09 does not claim protection
against storage hardware or firmware that violates the operating system's
completed-write contract.

The runtime may add only these target-specific dependencies, with Cargo choosing
the exact compatible patch versions:

```toml
[target.'cfg(windows)'.dependencies]
windows-sys = { version = "0.61", features = [
    "Win32_Foundation",
    "Win32_Storage_FileSystem",
    "Win32_System_IO",
    "Win32_Security",
    "Win32_System_Threading"
] }

uuid = { version = "1", features = ["v4"] }
```

No other dependency is authorised. `windows-sys` is confined to one small
Windows persistence module whose safe Rust wrappers contain, document, and test
every unsafe Win32 call. `uuid::Uuid::new_v4()` creates host execution IDs and
temporary nonces; every durable UUID is validated as canonical lower-case format.

#### Storage verification

Before ledger use, the Windows backend requires the absolute configured
host-data root, opens and inspects it and every relevant existing parent
component through `CreateFileW`, and then validates the complete provisioned
replay subtree. It uses
reparse-point-safe opening and inspection, rejects any component carrying
reparse-point semantics, and uses `GetVolumeInformationByHandleW` or an
equivalent handle-bound query to require filesystem name `NTFS` and a local
fixed volume. Validation remains handle-bound after path validation; path
strings alone are not authority. A missing root, missing child, relative path,
or substitution at any validation point is `ReplayPersistenceUnavailable`.

#### Owner and DACL validation

Before mutating the root, and after creating every replay directory, the
provisioner opens its directory handle with `READ_CONTROL`, obtains owner and
DACL through `GetKernelObjectSecurity`, and obtains the current process user
SID through `OpenProcessToken` plus `GetTokenInformation(TokenUser)`. It uses
caller-owned buffers only; no path-based security lookup is authority.
`CreateWellKnownSid` may create comparison SIDs for LocalSystem and Builtin
Administrators.

The security descriptor must be valid, have a present non-null DACL, and have
an owner SID equal to the current process user SID. Write-capable allow ACEs
may grant write authority only to that current user, LocalSystem, or Builtin
Administrators. An unrelated principal may have read-only authority, but may
not receive write, append, create-child, delete, change-permission,
take-ownership, or full-control authority. Malformed, unsupported, or
unprovable ACE forms, owner mismatch, null or absent DACL, or an impermissible
write grant fails closed as `ReplayPersistenceUnavailable`. J09 never rewrites
or repairs the operator's ACL.

#### Cross-process exclusion

The backend opens the lock file through `CreateFileW`, obtains OS-backed
exclusive access with `LockFileEx` over a documented byte range, and holds the
handle for the lock lifetime. A competing process may wait or fail closed, but
it must never proceed concurrently. The admission process holds that exclusion
through at least the provider boundary and, where practical, through final
generation publication.

#### Temporary-file publication

For every immutable claim or generation, the backend must create a uniquely
named temporary file with `CREATE_NEW` in the verified destination directory,
open it with `FILE_FLAG_WRITE_THROUGH`, write complete canonical bytes, and
require `FlushFileBuffers` success. It renames the still-open temporary handle
with `SetFileInformationByHandle` using `FileRenameInfo` or `FileRenameInfoEx`
with replacement disabled; source and destination must remain in the same
verified NTFS volume and directory. It then requires a second
`FlushFileBuffers`, reopens, and validates the final file through the validated
root before treating publication as durable.

The durability argument is that `FILE_FLAG_WRITE_THROUGH` is the documented
NTFS mechanism for flushing metadata changes including rename,
`FlushFileBuffers` confirms complete contents, and handle-based no-replace
rename supplies the atomic namespace operation. J09 does not require or pretend
to perform generic POSIX-style parent-directory `fsync` on Windows.

The backend must not use replace-capable rename,
`MOVEFILE_REPLACE_EXISTING`, `ReplaceFile`, copy-and-delete, hard links,
ordinary `std::fs::rename` as the durability primitive, an administrator-only
volume flush, or best-effort cleanup after ambiguous publication. Any failed or
unprovable call returns `ReplayPersistenceUnavailable`; no J05 approval is
consumed and no dispatch is allowed.

## Dispatch ordering and J05/J06 interaction

The normal and Ask-resume orders are fixed.

1. Validate the Anchor event ID, evaluation ID, and Action ID; derive one
   logical key per exact tuple; perform fresh resolution, pins, schema, scope,
   and ordinary policy evaluation for that Action.
2. For a fresh Ask request, create only the J05 pending approval. It creates no
   execution identity and makes no replay claim.
3. For Allow, and for an approved Ask resume after all J05 fresh checks, require
   a valid already-provisioned absolute `--host-data-root`, then acquire replay
   admission under the logical-key lock. Missing, relative, unproven, existing,
   blocked, corrupt, or mismatched identities stop here before J05 consumption.
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
4. The same exact event/evaluation/Action tuple across restart recovers the
   same host UUID.
5. A completed recovered tuple makes zero provider calls and no duplicate
   Result Anchor.
6. A simulated host restart cannot accidentally generate a new UUID for the
   same exact tuple.
7. One event with `action_1` and `action_2` creates two distinct logical keys,
   claims, and host UUIDs.
8. Replay of `action_1` recovers its original UUID.
9. Replay of `action_1` never collides with `action_2`.
10. Different evaluations of one Anchor event produce distinct logical keys and
    host UUIDs.
11. Existing exact tuple plus changed binding fails before approval consumption,
    intent, or provider work.
12. Claim is durable before J05 consumption; crash after claim before generation
   zero selects `claimed_no_state` and is manual-only.
13. Generation zero is immutable, follows the claim digest, and precedes Trail
    intent and every provider call.
14. Generation one is immutable, follows generation zero, and precedes every
    possible provider call.
15. Final generation follows generation one, includes the durable J06 outcome
    digest, and precedes the standard Result Anchor.
16. Only the exact `0 -> 1 -> 2` chain and final-state vocabulary are accepted.
17. Current state is selected by full contiguous-chain validation, never a
    mutable head pointer.
18. Gap, duplicate number, invalid predecessor, checksum failure, malformed
    record, unsupported version, or unexpected extension fails closed.
19. A crash after generation zero before Trail intent makes zero calls on
    restart and writes no compensating Trail entry.
20. A crash between generation zero and one is manual-only and makes zero calls.
21. A crash after generation one before/during/after a possible provider call
    is manual-only and never retries.
22. Crash after durable J06 outcome but before final generation is replay
    blocked with no synthetic Anchor.
23. Crash after final generation but before Anchor is replay blocked with no
    duplicate Anchor.
24. Keyed temporary files and partial files fail closed and are never silently
    deleted by J09.
25. A chain directory with no valid matching claim is ledger/root corruption and
    fails the entire ledger closed until separately authorised maintenance.
26. Temporary write, file flush, no-replace publish, directory-durability, read,
    validation, permission, and lock failures each block before dispatch.
27. Two concurrent same-key admissions create exactly one claim and one UUID.
28. A competing writer observes the first claim/chain after exclusion and makes
    zero provider calls.
29. A duplicate generation publication is accepted only when its expected
    immutable bytes and predecessor already validate; any other collision fails
    closed.
30. Cross-process lock loss, unavailable lock primitive, or unsupported volume
    fails before J05 consumption and dispatch.
31. Existing replay-blocked success, failure, uncertain, armed, intent, and
    claim-only states all consume zero additional J05 approvals.
32. Fresh approved Ask consumes exactly once only after fresh replay admission;
    claim, J05, intent, and armed ordering is observable in the test double.
33. J05 consumption failure leaves no dispatch authority; its claim is
    manual-only and approval is not restored.
34. Trail-intent failure after generation zero, armed-publication failure, and
    pre-invocation deadline expiry each make zero provider calls.
35. J06 known success, known failure, and uncertainty write outcome then final
    generation then exactly one corresponding standard Anchor.
36. J06 outcome write failure or final-generation failure preserves the known
    classification where applicable, creates no Anchor, and never retries.
37. Terminal success and failure are permanently replay-blocked; terminal
    records are never reused or overwritten.
38. Incomplete and uncertain states are manual-only; J09 has no automatic
    resolution, compensation, approval restoration, or recovery executor.
39. J12/J13 integration preserves the existing Anchor event ID plus the
    planner-produced evaluation ID and Action ID at host admission; it does not
    alter planner, manifest, provider, or MCP messages.
40. Full Rust, host integration, protocol, OCaml, packet, whitespace, complete
    diff, Windows primitive, and restart tests pass from the accepted J06 base.
41. A supported local fixed NTFS volume is admitted; non-NTFS and remote storage
    are rejected before approval consumption or dispatch.
42. Reparse-point, symlink, junction, mount-point traversal, and unsupported
    volume paths are rejected through handle-bound storage validation.
43. Two processes cannot hold the same logical-key lock concurrently.
44. Destination collision never replaces existing final bytes; competing
    publishers produce exactly one accepted final file.
45. Write, flush, rename, reopen, and final-verification mismatch faults fail
    closed, and a published final file validates after closing/reopening handles.
46. The safe Windows wrapper contains and documents every unsafe Win32 call.
47. Only the authorised target-specific `windows-sys` and `uuid` dependencies
    are added; no generic directory `fsync`, volume flush, hard-link,
    replace-capable rename, or `std::fs::rename` publication path exists.
48. Allow without `--host-data-root` fails before J05 consumption, durable
    intent, dispatch, outcome, or Result Anchor.
49. Approved Ask without `--host-data-root` does not consume its approval.
50. `TRAIL_PATH` is neither used nor inspected as replay-root authority, and
    Trail and replay roots may be completely different paths.
51. A relative host-data-root path is rejected and a missing host-data root is
    never created.
52. Normal execution never provisions replay storage.
53. Explicit provisioning creates exactly the specified complete v1 structure.
54. Repeated provisioning of that exact valid structure returns
    `AlreadyProvisioned` without mutation.
55. Partial provisioning fails closed and is not repaired; unknown files or
    versions fail closed.
56. Host-root owner mismatch fails closed.
57. Null or missing DACL fails closed.
58. Broad or unknown write authority fails closed.
59. A read-only unrelated principal does not automatically fail validation.
60. Write authority for the current user, LocalSystem, and Builtin
    Administrators is accepted.
61. ACL inspection remains handle-bound and reparse substitution between path
    parsing and ACL validation fails closed.
62. Provisioned valid storage permits later replay admission.
63. Existing unmatched, denied, and pending-Ask behaviour remains provider-free
    without opening or provisioning replay storage.

## Implementation boundary

J09 may add focused host replay-ledger and host-admission code, a small
target-specific Windows persistence module, the two authorised target-specific
dependencies, tests, minimal dispatch orchestration, the host-local
`--host-data-root` option, and the separate `provision-replay` operation. It
must not change Tethers Core, OCaml, capability manifests, MCP messages,
provider contracts, Result Anchor queueing, public Tethers language
configuration, or the preserved safety branch.
