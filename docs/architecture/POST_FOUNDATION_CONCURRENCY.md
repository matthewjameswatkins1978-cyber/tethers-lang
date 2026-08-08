# Post-Foundation Concurrency Architecture

Status: **DIRECTION / ARCHITECTURAL GUIDANCE — NOT AUTHORISED IMPLEMENTATION**

Prepared after the post-Foundation concurrency design discussion, August 2026.

This document sharpens the concurrency thread recorded in `POST_FOUNDATION_ROADMAP.md`. It does **not** amend the Foundation Pass, authorise implementation, freeze syntax, or require any particular scheduler/library design. Future work must still re-audit repository reality and produce a bounded implementation packet.

The governing split remains:

> **The Tether declares semantic independence and dependency. The Plug declares safe overlap constraints. The host decides the actual physical schedule.**

Companion rule:

> **Nondeterministic execution, deterministic interpretation.**

The central architectural correction from this discussion is:

> **Safety, capacity, and rate limits are different concerns. Do not collapse them into one `concurrency` knob.**

A second correction is equally important:

> **A Plug safety conflict should constrain scheduling, not automatically change Tether semantics into failure.**

If two semantically independent actions cannot overlap safely, the host may serialize them. That is consistent with the existing rule that the host may always execute less concurrency than the Tether and Plug combination permits. A Tether should fail only if its semantics explicitly require simultaneous overlap and that requirement cannot be satisfied. No such hard-simultaneity syntax is frozen here.

---

## 1. Three authorities

### 1.1 Tether authority: semantic structure

Tethers owns meaning:

- dependencies;
- independence;
- fan-out/fan-in shape;
- joins;
- branch identity;
- join/failure semantics;
- canonical result ordering;
- whether later work depends on earlier results.

Tethers should **not** need to know about:

- mutexes;
- semaphores;
- filesystem paths;
- SQL connection pools;
- GPU memory;
- HTTP sockets;
- thread counts;
- provider rate-limit windows.

The important semantic statement is:

```text
dependencies constrain order
```

not:

```text
actions inherently execute in source order
```

A future source form may express independent branches, but the syntax is deliberately not frozen here.

### 1.2 Plug authority: safe overlap claims

A capability provider understands physical facts Tethers cannot know generically.

Examples:

- two reads of one file may overlap;
- a write and a read of the same file may need exclusion;
- a database migration may conflict with ordinary queries;
- model loading may conflict with inference on the same device;
- two capabilities in one Plug may share a hidden physical resource.

A Plug contract therefore needs a way to describe **resource claims / overlap safety** without turning the Plug into a scheduler.

### 1.3 Host authority: physical admission and scheduling

The Rust host owns machinery such as:

- async execution;
- task queues;
- admission;
- actual lock/semaphore implementation if used;
- worker limits;
- cancellation signalling;
- process supervision;
- provider/resource capacity configuration;
- backpressure;
- scheduling policy.

The host may always choose **less** physical concurrency than is theoretically safe.

Hardware and operational limits may change speed and waiting time. They should not change Tether meaning merely because one machine has fewer cores, fewer connections, or less memory.

---

## 2. Safety, capacity, and rate are different

### 2.1 Safety

Safety answers:

> **May these operations overlap without violating the capability/resource contract?**

This is the strongest candidate for a declarative Plug-owned concurrency contract.

A useful conceptual model is a resource claim:

```text
resource realm
deterministic resource key
access mode
```

For example:

```text
file.read(A)
→ realm: filesystem
→ key: normalized(A)
→ access: shared

file.write(A)
→ realm: filesystem
→ key: normalized(A)
→ access: exclusive
```

Then conceptually:

```text
read A  + read A   → compatible
read A  + write A  → incompatible overlap
write A + write A  → incompatible overlap
write A + write B  → potentially compatible
```

This is an architectural example, not frozen vocabulary or syntax.

### 2.2 Capacity

Capacity answers:

> **How much work can this runtime/provider/device admit at once?**

Examples:

- maximum host tasks;
- two GPUs;
- eight SQL connections;
- limited VRAM;
- four concurrent external requests.

Capacity is generally operational and host-specific. A different machine may legitimately have a different capacity without changing Tether semantics.

The future design may use token/semaphore-style pools, but the exact ownership must be tested against repository reality.

A likely split is:

```text
capability/Plug metadata → what class or quantity of operational resource an action consumes
host configuration      → how much of that resource exists here
host scheduler          → admission policy
```

However, **V1 does not automatically require arbitrary token pools**. A bounded first implementation may begin with simpler global/per-Plug/per-provider limits if that proves sufficient.

Do not build a generic resource-accounting language merely because semaphores are familiar.

### 2.3 Rate limits

Rate limits answer:

> **How many operations are permitted over a period of time?**

Examples:

```text
60 requests / minute
1000 requests / day
```

This is not the same as safety or instantaneous capacity.

Do not put provider rate-window semantics into the deterministic Tethers language merely to prevent HTTP 429 responses.

A Plug/provider implementation may internally pace/back off/retry according to its own declared operational policy. A future host-level rate facility may be justified later, but only if there is a demonstrated cross-Plug scheduling benefit.

The clock entering an operational scheduler is not itself forbidden. The crucial rule is that **operational waiting/rate policy must not silently become semantic meaning**.

---

## 3. Resource identity

For safety claims to work, resource identity must be:

- deterministic;
- inspectable;
- derived from canonical/normalized arguments where practical;
- stable enough that two capabilities referring to the same physical resource can agree;
- separate from arbitrary host memory addresses or process-local identities.

A likely conceptual identity is:

```text
realm + key
```

Examples:

```text
filesystem + canonical path
database-instance + schema/table/device identity
gpu-device + model/device resource
queue-provider + queue identity
```

The exact granularity matters. Overly fine claims create a distributed borrow checker. Overly coarse claims destroy useful concurrency.

Therefore:

> **Claim at the smallest resource boundary the Plug can state simply and reliably, not at arbitrary field/sub-object depth.**

Do not support claims such as "exclusive access to `.user_id` inside object A`" in the first design.

---

## 4. Cross-capability conflicts

Different capabilities in the same Plug may conflict even when their names differ.

Examples:

```text
file.read(A)
file.write(A)
```

or:

```text
database.query
database.migrate
```

The resource-claim model handles this more naturally than per-capability booleans because both actions can resolve to the same resource realm/key with different access modes.

This is one reason a simple field such as:

```text
concurrent: true
```

is inadequate.

---

## 5. Cross-Plug conflicts

Two separate Plugs may ultimately touch the same physical resource.

Examples:

- two filesystem implementations reaching the same path;
- two database Plugs reaching the same PostgreSQL instance;
- two cloud-provider Plugs sharing one account quota;
- separate capabilities sharing one GPU device.

A Plug should not be trusted to globally invent physical identity by arbitrary string convention.

A likely future model is:

```text
Plug declares local resource claim
        ↓
host/provider configuration maps local realm to deployment resource identity
        ↓
host compares compatible/incompatible claims
```

This preserves portability. The same Plug package can run in different deployments without hardcoding one machine's database URL, filesystem root, or device identity into language semantics.

Exact mapping rules are not frozen.

---

## 6. Important correction: safety conflicts do not automatically mean semantic failure

A scout proposal suggested:

> if two actions are declared concurrent but their Plug safety claims conflict, fail the group rather than serialize them.

That conflicts with the existing Tethers concurrency model unless the source explicitly requires simultaneous execution.

Our current intended meaning is closer to:

```text
Tether: these actions have no semantic dependency
Plug:   these two particular resource claims may not overlap safely
Host:   therefore schedule them without overlap
```

For example:

```text
branch A: file.read(X)
branch B: file.write(X)
```

If the Tether only says A and B are independent, the host may serialize them because physical overlap is unsafe.

That is **not** the same as discovering a semantic dependency between them. It is a physical scheduling restriction.

This preserves an important promise:

> **Changing hardware, capacity, or conservative safety policy should usually change scheduling/performance before it changes semantic outcome.**

A hard concurrency violation should exist only if future Tethers semantics deliberately provide a construct meaning something like:

```text
these operations must overlap physically
```

No such construct is currently required or authorised.

This distinction also keeps capability contract hardening safer. If Plug v2 tightens a resource claim from shared to exclusive, an old Tether should usually become slower/more serialized rather than suddenly fail solely because the Plug learned to be safer.

---

## 7. Atomic admission and deadlock avoidance

If a branch needs several safety/resource claims, the host must avoid naive incremental acquisition that can deadlock.

Conceptually, admission should be equivalent to:

```text
compute complete claim set
        ↓
canonicalize/order claims
        ↓
admit only when complete compatible set is obtainable
        ↓
dispatch branch
```

The implementation may use atomic admission, canonical lock ordering, a central scheduler, or another equivalent mechanism.

Do not freeze a specific Rust primitive here.

The semantic requirement is simply that deadlock-prone physical lock acquisition must not leak into Tether meaning.

---

## 8. Retries

Retries belong to a branch/action lifecycle, not to a second independent semantic branch unless explicitly authored that way.

A safe default principle is:

> **Retries of one logical branch do not overlap with each other.**

Whether safety claims remain physically held during backoff is an implementation question, not yet a semantic rule.

Holding an exclusive claim for a long retry delay might unnecessarily block unrelated work. Releasing and reacquiring may be safe for some capabilities but not others.

Therefore do **not** freeze the scout's stronger claim that locks/tokens must always remain held until the entire retry lifecycle completes.

Retry safety should eventually interact with capability contract facts such as:

- idempotency;
- retry-safe operation;
- uncertainty of previous attempt;
- resource claim needed for each attempt.

---

## 9. Cancellation

Cancellation is primarily physical, but its observable result must still have deterministic semantic treatment.

If a join is already destined to fail, the host may wish to cancel expensive sibling work when:

- the Tether's join semantics permit it;
- the capability supports cancellation;
- cancelling cannot create an invalid uncertainty about side effects.

Do not assume "branch failed, therefore cancel all siblings" universally.

A sibling may already have produced an external side effect, may not support cancellation, or may be required for complete evidence.

The host may send cancellation as an optimization. The semantic layer must still receive an explicit canonical outcome such as succeeded/failed/cancelled/uncertain according to whatever outcome model is eventually frozen.

---

## 10. Compensation / undo

A scout proposal suggested compensation should always take the exact same safety claims as the original action.

That is too strong to freeze generically.

A compensation may:

- touch the same resource in a different access mode;
- touch additional resources;
- invoke a different capability entirely;
- be unsupported;
- be impossible after an external side effect.

Therefore:

> **Compensation must declare its own ordinary capability/resource claims.**

Do not special-case it as "same locks as original" unless a particular capability contract explicitly says so.

---

## 11. Trail, audit, and replay

The semantic Trail/replay contract must remain independent of completion races.

Canonical branch/result identity should come from semantic structure, not completion order.

A future semantic record may need:

```text
branch identity
canonical branch order
final branch outcome
join outcome
cancellation/uncertainty where semantically relevant
```

Operational telemetry may additionally record:

```text
ready_at
admitted_at
started_at
completed_at
resource wait reason
physical worker/provider used
```

The exact split between Trail and lower-level host audit log is not frozen.

The key rule is:

> **Timing observations may explain execution without becoming the source of semantic ordering.**

Dynamic capacity changes should ordinarily affect admission/wait time only.

---

## 12. Contract evolution

Concurrency-related Plug contracts can evolve.

We should distinguish:

### Safety claim changes

A Plug may discover that previously permitted overlap was unsafe and tighten a claim.

Preferred default effect:

```text
same semantic Tether
→ more conservative scheduling
→ same intended outcome
```

not automatic semantic failure.

### Capacity changes

A host/provider may change from 8 slots to 4.

Preferred effect:

```text
same semantic Tether
→ longer execution
→ same intended outcome
```

### Behavioural contract changes

If a capability's actual effects, permissions, outputs, failure contract, or semantic guarantees change, that is broader capability-version compatibility and may legitimately invalidate a Tether. Do not disguise such changes as "just concurrency".

---

## 13. Pressure-test examples

### Filesystem

Conceptually:

```text
read(A)  → shared filesystem:A
read(B)  → shared filesystem:B
write(A) → exclusive filesystem:A
write(B) → exclusive filesystem:B
```

The host may overlap operations with compatible claims.

### SQL database

Conceptually:

```text
query       → shared db/schema claim
migration   → exclusive db/schema claim
```

Exact resource granularity must be chosen by the Plug contract, not hardcoded into Tethers.

### HTTP/API

Many remote APIs need no Tethers-level safety claim because the provider owns data consistency.

The host/Plug may still impose operational capacity limits.

Rate policy should remain separate.

### Email

Ordinary send operations may have no mutual safety claim. SMTP/provider connection capacity and rate rules are operational.

Idempotency/duplicate-send semantics are a separate capability concern and must not be mistaken for concurrency capacity.

### GPU

A GPU Plug may need device/model safety claims plus operational capacity based on available resources.

Do not assume "shared lock + VRAM tokens" is universally sufficient before examining real GPU/provider models.

### Queue

The remote queue usually owns ordering/atomicity. A producer capability may require no safety claim, while local connection capacity remains operational.

### Lantern Keeper / SurrealDB

Database-level operations may be concurrency-safe because SurrealDB provides its own transaction/isolation semantics. Administrative/schema/backup operations may require stronger claims. The future Plug contract should describe this rather than teaching Tethers about SurrealDB specifically.

---

## 14. Minimum first design target

The smallest useful future design should first prove whether Tethers needs only:

```text
semantic dependency graph
+
Plug-declared deterministic safety/resource claims
+
coarse host concurrency limits
```

before adding generic token pools, cross-provider quotas, dynamic resource weights, or distributed resource arbitration.

A likely staged path is:

### Stage A — semantic concurrency

- independent branches;
- deterministic join;
- canonical branch identity/order;
- host can run branches concurrently when safe.

### Stage B — Plug safety claims

- deterministic resource identity;
- shared/exclusive or similarly tiny access vocabulary;
- host prevents unsafe physical overlap;
- safe conflict results in serialization unless semantics explicitly say otherwise.

### Stage C — bounded host capacity

- global/per-provider/per-Plug limits as demonstrated necessary;
- preserve semantic invariance.

### Later only if evidence requires

- generic resource token pools;
- host-level rate scheduling;
- distributed resource identity;
- weighted/fractional admission;
- priority/fairness policy;
- sophisticated cancellation trees.

---

## 15. Ownership guidance

```text
OCaml / Tethers semantics
- dependency/independence meaning
- branch identity
- deterministic joins
- canonical result interpretation
- no knowledge of mutexes/semaphores/resource keys

Plug / capability contract
- deterministic declarations of physical overlap safety
- resource claim derivation from normalized arguments where needed
- capability cancellation/retry/idempotency facts where separately defined

Rust host
- scheduling
- admission
- physical lock/semaphore machinery
- operational capacity
- worker/provider configuration
- backpressure
- cancellation signalling
- telemetry

HQ / authoring tools
- visualize semantic branches/dependencies
- may explain why execution was serialized
- should not teach users lock internals as if they were core language concepts
```

---

## 16. Things deliberately not to build yet

Do not prematurely add:

- a general Rust-style borrow checker to Tethers;
- field/sub-object resource borrowing;
- distributed locking;
- clock-window rate scheduling in the core;
- arbitrary user-authored mutex declarations;
- source syntax for semaphores;
- fractional resource mathematics;
- automatic cross-Plug global identity without explicit host mapping;
- completion-race semantics;
- mandatory physical overlap;
- scheduler policy embedded in OCaml semantics.

The rule remains:

> **The Plug describes constraints. It does not become the scheduler.**

And:

> **The host exploits available parallelism. It does not invent Tether meaning.**

---

## 17. Post-Foundation design gate

Before implementation, the concurrency architecture pass must inspect current repository reality and answer at least:

1. how action dependencies/order are represented today;
2. whether current plan/action structures accidentally imply serial semantics;
3. where capability contract schemas are resolved;
4. whether normalized arguments are available before host admission;
5. the smallest deterministic resource-claim representation;
6. whether shared/exclusive is sufficient for the first real Plug set;
7. how resource keys can be derived canonically;
8. how cross-Plug deployment identity would be configured if needed;
9. whether V1 needs token pools at all;
10. what coarse host limits already exist or can be added independently;
11. failure/cancellation/join semantics;
12. Trail versus host telemetry ownership;
13. retry/idempotency interaction;
14. behaviour under very large AI-authored fan-out;
15. whether implementation remains bounded enough to do before HQ.

No implementation is authorised merely by this note.

---

## Summary

The useful result from the concurrency scouting is not "put locks and semaphores in Tethers."

It is:

> **Tethers declares the semantic graph. Plugs declare deterministic overlap-safety constraints. The Rust host performs physical admission and chooses how much parallelism to exploit.**

Safety, capacity, and rate remain distinct.

Resource claims are a promising tiny vocabulary for Plug safety, but their exact shape is not frozen.

Most importantly, a physical safety conflict should normally cause **less overlap**, not a different semantic outcome. That preserves the project’s central concurrency promise: the same Tether can run on different machines, providers, and resource budgets without its meaning being rewritten by the scheduler.