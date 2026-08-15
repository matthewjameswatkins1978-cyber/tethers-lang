# C3 Bounded Concurrency Design

DESIGN CANDIDATE — NOT IMPLEMENTATION AUTHORITY

Status: design artifact — awaiting Lucy acceptance

Updated: 2026-08-15

## 0. Frozen Inputs

C2-A3a is complete and merged to `main`. The A3a architecture is the frozen
foundation:

- Serial Stage A preparation for every Together member in Runtime Plan order.
- `std::thread::scope` workers for provider invocation.
- mpsc result delivery to coordinator.
- Coordinator-owned ReplayAdmission (`Rc<ReplayLedger>`, not Send).
- Coordinator-owned Trail (single writer).
- Ephemeral trusted `RetainedProviderSession` worker path.
- Sequential non-Together path preserved unchanged.

C3 does not redesign any of these. C3 adds a bounded launch window on top of
the existing A3a architecture.

## 1. Resource Being Bounded

C3-A bounds exactly one resource:

**active Together provider invocations within one group execution**

Internal parameter:

```text
max_active_together_invocations: usize = N
```

Requirements: `N >= 1`

For the first C3 slice, N is **group-local**. It applies to one Together group
evaluation, not across evaluations or across the host.

### Explicitly out of scope for C3-A

These are possible later C3 work, not part of this design:

- host-wide scheduler
- global semaphore
- worker pool
- adaptive concurrency
- priority / weights
- fairness policy
- CPU accounting
- RAM accounting
- request-rate limiting
- API quotas
- provider-specific limits

## 2. Stage A Remains A3-Compatible

Stage A remains serial and runs for every semantic Together member in Runtime
Plan order:

1. scope assessment
2. policy / permission decision
3. capability resolution
4. provider availability check
5. replay admission (`admit`)
6. replay G0 (`publish_intent`)
7. durable Trail intent (`prepare_and_record`)

Prepared members may then wait for physical launch capacity. Bulk G0 is
intentional.

### Why G0 without G1 is valid

A member may truthfully have:

```text
G0 = yes
durable intent = yes
G1 = no
provider invocation = no
```

while waiting for capacity. This is valid crash/recovery evidence because:

- G0 (`publish_intent`) records that the coordinator committed to attempting
  this member. After a crash, recovery knows the member was admitted and
  intended.
- G1 (`publish_armed`) records that the member crossed the provider invocation
  boundary. Its absence means the provider was never touched.
- The combination `G0=yes, G1=no` is unambiguous: the member was admitted and
  intended but never armed. Recovery can safely re-attempt or classify as
  Unattempted without uncertainty about whether an external effect occurred.
- This matches the existing A3a serial path where G0 is published in Stage A
  and G1 is published in Stage B immediately before worker launch. The only
  change is that the gap between G0 and G1 may now include a capacity wait.

G0 without G1 is **not** an ambiguous external-effect state. It is a clear
pre-invocation record.

## 3. Internal PREPARED_WAITING Condition

### Definition

An internal runtime condition, conceptually named `PREPARED_WAITING`, meaning:

- Stage A succeeded for this member
- member has durable Trail intent
- replay G0 exists
- no provider timeout has started
- G1 has not occurred
- no provider worker is active for this member
- member is waiting solely for physical launch capacity

### What it is NOT

- NOT a source-language state
- NOT a Runtime Plan semantic
- NOT a new Trail terminal status
- NOT a new replay lifecycle state
- NOT visible to the Tethers language or the host application

It is runtime scheduling state only — internal to the coordinator's launch
window logic.

### Representation recommendation

The existing `GroupMemberState` enum in the A3a implementation already carries
`Prepared` as a state. The design recommends **extending** the existing
`Prepared` variant (or introducing a narrow sub-variant) to distinguish:

- `Prepared` — Stage A complete, ready for launch (may be immediately eligible)
- `PreparedWaiting` — Stage A complete, waiting for capacity

This avoids a second independent truth. The member state IS the capacity
signal. A separate `LaunchWindow` helper may track the count of active members,
but its value must be derivable from, or invariant-checked against,
`GroupMemberState`.

If implementation evidence later proves a small `LaunchWindow` helper is cleaner
for counting, its state must remain a cached projection of `GroupMemberState`,
not an independent mutable truth.

## 4. State-Derived Capacity

### Concept

```text
active_count =
    number of members that have crossed G1 / launch
    and have NOT successfully completed coordinator terminalisation

available = N - active_count
```

### Which states count as active

A member counts as **active** from the moment G1 is published (worker launch)
until the coordinator has successfully completed Stage C terminalisation for
that member:

| State | Counts as active? |
|-------|-------------------|
| Preparing (Stage A) | NO |
| Prepared (waiting for capacity) | NO |
| PreparedWaiting (capacity-blocked) | NO |
| Armed (G1 published, worker launching) | YES |
| Running (worker in flight) | YES |
| ResultReceived (worker result in coordinator, Stage C pending) | YES |
| Terminalised (Stage C complete, G2 published) | NO |

The exact terminal boundary is: **after the coordinator has durably written the
OutcomeEntry, published G2, processed required anchor/response, and transitioned
the member to its terminal state**. Only then does `active_count` decrease.

### Why state-derived, not counter-based

A separate mutable `active_slots` counter creates two independent truths:

1. `GroupMemberState` — the authoritative member lifecycle
2. `active_slots` — a separate count that must stay in sync

If these diverge (due to a bug, panic recovery, or early return), the system
could launch workers it should not or block workers it should launch.

By deriving capacity from `GroupMemberState`, there is one truth. A cached
count helper is acceptable for performance, but it must be resettable from
the authoritative state and checked against it.

## 5. Admission Order

When capacity is available, launch the **earliest semantic-order
PREPARED_WAITING member**.

Semantic order is the Runtime Plan member order (`member_action_ids` order),
which is deterministic and independent of physical completion timing.

### Example

```text
Plan members: A B C D E
N = 2

Initial:   A and B launch (first two eligible)
B finishes: C launches next (earliest waiting)
A finishes: D launches next (earliest waiting)
C finishes: E launches next
D finishes: (no more waiting)
E finishes: all terminal → join
```

### No provider-aware skipping in C3-A

C3-A does not inspect provider identity when selecting the next member. If A
and B are both PREPARED_WAITING and A is earlier in semantic order, A launches
regardless of whether A and B share the same provider.

Same-provider members already use independent ephemeral `RetainedProviderSession`
child processes from A3a. Provider-specific capacity or rate constraints belong
to later C3 work.

## 6. Exact Launch Boundary

For each member selected for available capacity:

```text
capacity eligibility established
→ deadline starts (clock.now(), per-member)
→ remaining timeout calculated
→ existing final pre-invocation deadline check
→ replay G1 publish_armed
→ worker launch
→ possible provider effect
```

### The provider timeout MUST NOT run while PREPARED_WAITING

The timeout measures provider execution time, not queue wait time. A member
that waits 30 seconds for capacity and then invokes a provider that responds
in 1 second must not be classified as timed out.

### Required invariant

```text
G0 (Stage A)
→ durable Trail intent (Stage A)
→ arbitrary capacity wait (no timeout running)
→ deadline start (Stage B, per-member clock.now())
→ final deadline check (Stage B)
→ G1 (Stage B)
→ worker launch (Stage B)
→ possible provider effect (Stage B)
```

### No queue timeout

Do not introduce a queue timeout. Do not introduce:

- `AbortedBeforeInvocation`
- `QueueDeadlineExpired`
- any new terminal taxonomy

The existing `Unattempted` semantics cover a genuine pre-invocation deadline
failure at the actual launch boundary (the member was selected, the deadline
was established, and it was already expired). This is the same as A3a — the
only difference is that the gap between Stage A and Stage B may include a
capacity wait during which no timeout runs.

## 7. Slot Release / Active-Count Release Point

A member remains active after `WorkerResult` arrives at the coordinator.

Capacity becomes reusable ONLY after successful coordinator Stage C
terminalisation:

```text
worker result received
→ classify outcome (Succeeded / Failed / Uncertain)
→ durable OutcomeEntry written
→ replay G2 terminal publication
→ required anchor/response processing (per existing A3 contract)
→ member terminal transition (GroupMemberState → terminal variant)
→ active_count decreases
→ next waiting member may launch
```

### Exact terminal point

The member is considered terminal (and capacity released) at the point where:

1. The OutcomeEntry is durably written to Trail.
2. G2 is published.
3. The member's `GroupMemberState` has transitioned to its terminal variant.

All three must be complete. An in-memory `WorkerResult` without durable evidence
is not sufficient.

### Safety invariant

No queued provider effect may be launched merely because an in-memory
`WorkerResult` arrived while trusted durable evidence for that result has not
yet been completed. This prevents:

- launching a new worker based on a result that would be lost on crash
- double-launching if recovery replays the member as non-terminal

## 8. Failure Boundaries

### A. Normal provider failure

Worker returns a failed or uncertain provider result.

- Coordinator truthfully terminalises the member (Failed, Uncertain, etc.).
- Capacity becomes reusable (Stage C complete).
- Queued siblings continue.
- Group evaluates all terminal members for join.

### B. Worker panic

Existing A3a `catch_unwind` / `JoinHandle` path maps panic to `WorkerResult`
with exact `Uncertain` semantics.

After truthful terminalisation:

- capacity becomes reusable
- queued siblings continue
- no coordinator hang

The coordinator does not distinguish between "provider returned Uncertain" and
"worker panicked" at the capacity level. Both release the slot after Stage C.

### C. Worker channel failure

The A3a architecture uses `std::sync::mpsc` channels scoped to the group
execution lifetime. Under normal worker construction (scoped threads), a
channel failure means either:

1. The worker panicked (handled by B above).
2. The scope was dropped before the worker sent its result (an interruption).

The design states: **channel failure is fail-closed**. If the coordinator
cannot receive a worker result through the channel, it must not assume success
or fabricate evidence. The member should be classified as `Uncertain` (or the
existing appropriate audit failure) and terminalised honestly.

Current A3a structure makes a channel-result hole impossible under normal
scoped-thread construction because the scoped join guarantees all workers
complete before the scope exits. A channel failure implies a scope violation
or panic, both of which are handled.

### D. Stage C trail / durability failure

If the coordinator cannot durably record truthful prior effect evidence
(Trail write failure, G2 publication failure, anchor write failure):

**NO NEW PROVIDER WORKERS MAY BE LAUNCHED AFTER FAILURE BECOMES KNOWN.**

Fail closed through the existing appropriate audit/persistence failure
semantics (`AuditFailed` or `ReplayPersistenceUnavailable`).

Do NOT free capacity and continue launching external effects.
Do NOT flatten this into an ordinary provider `Failed` result.

The rationale: if the coordinator cannot durably record that member A's effect
occurred, it cannot trust its own state about which members are active, which
slots are free, or whether recovery would replay correctly. Launching new
external effects into untrusted state violates Trail truthfulness.

### E. Replay G2 failure

Same trust principle as D:

**after the coordinator knows trusted replay terminalisation has failed,
no additional provider effect launches.**

G2 failure means the replay ledger cannot be trusted to reflect the member's
terminal state. Recovery might re-admit the member or classify it differently.
Launching new effects while the replay state is untrusted creates the same
class of hazard as D.

Document the existing A3a classification/boundary rather than inventing a new
one. Both D and E are existing persistence-failure semantics; C3 inherits them
and adds the constraint that they halt further launches.

## 9. Group Semantics

Failure of any active member does NOT cancel waiting siblings unless a fatal
trusted-state failure (§8D, §8E) prevents safe continuation.

### Normal semantic non-success

```text
A fails (provider failure)
B running
C waiting

→ A terminalises (Failed)
→ B continues
→ C still launches when capacity is available
→ GroupJoin waits for A (terminal), B, C
```

### GroupJoin

GroupJoin remains: **only after every semantic member has reached its
legitimate terminal state.**

Join success iff all members semantically succeed (under existing C1 rules).

### Final group non-success

First non-success in **semantic Runtime Plan / member order**:

- NEVER physical launch order
- NEVER physical completion order
- NEVER queue order beyond its physical scheduling role

This preserves deterministic plan results across different N values and
different physical timing.

## 10. Same-Provider Behaviour

Preserve A3a:

- Together same-provider overlap is allowed through independent ephemeral
  child process / session instances.
- Sequential Actions still use retained provider sessions.
- Provider identity is independent of process instance.
- C3-A global group cap counts both same-provider and different-provider
  workers equally.

Do not create provider-specific serialization in C3-A.

## 11. Semantic Equivalence Across N

### Must remain invariant

For N=1, N=2, N>=eligible member count, the following must remain semantically
equivalent:

- source meaning
- Runtime Plan identity
- semantic member identities
- SemanticPosition (action_ordinal, group_id, member_ordinal, phase)
- terminal classification for equivalent provider outcomes
- join result (success / failure / which members)
- final semantic non-success selection (first non-success in member order)
- later Action gating (join blocks or permits continuation)
- canonical / program identity

### Need NOT be byte-identical

- physical Trail append order
- coordinator observation order of worker results
- timing / timestamps
- child process scheduling / OS thread scheduling

Trail must remain **physically truthful** — it records what the coordinator
observed and when it durably recorded it. Different N values may produce
different physical orders; both are truthful.

### Why this matters

N=1 degenerates to A3a serial behaviour (one worker at a time). N>=group-size
degenerates to A3a full overlap (all workers launch immediately). Both must
produce the same semantic results as each other and as A3a for equivalent
provider outcomes. The bounded window changes only the physical interleaving,
not the semantic contract.

## 12. C3-A Out of Scope

Explicitly deferred to later C3 slices:

- max total Together members (group-size cap)
- max prepared waiting members (queue depth)
- host-wide concurrency across evaluations
- per-provider concurrency quotas
- provider rate limits
- API quota accounting
- priorities / weighted fairness
- adaptive tuning
- CPU/RAM accounting
- persistent worker pools
- queue telemetry
- user-facing configuration schema

## 13. Required Implementation Decomposition

### C3-A1 — Minimal bounded launch window

**Objective:** Introduce the `max_active_together_invocations` parameter and
gate worker launch on `available > 0`, launching in semantic order.

**Permitted scope:** Group execution coordinator logic, `GroupMemberState`
extension (if needed), capacity derivation from member state, semantic-order
launch selection.

**Proof target:** N=1 with group=5 produces max active exactly 1, every
eligible member eventually invokes. N=2 produces max active never exceeding 2.
N>=group-size preserves A3a physical overlap behaviour.

### C3-A2 — Core deterministic resource / deadline / G1 crucible

**Objective:** Prove that capacity wait does not consume provider timeout, that
G1 is published per-member at launch (not at preparation), and that the launch
boundary invariant holds.

**Permitted scope:** Deadline establishment timing, G1 publication timing,
timeout isolation proofs.

**Proof target:** Waiting member has G0, durable intent, no G1, no provider
touch. Queue wait longer than member timeout does not consume provider timeout.
Next capacity launches earliest semantic PREPARED_WAITING member.

### C3-A3 — Failure-boundary crucible

**Objective:** Prove that each failure boundary (§8A–§8E) behaves correctly
and that capacity is neither leaked nor unsafely freed.

**Permitted scope:** Failure injection tests, Stage C durability failure
simulation, G2 failure simulation, worker panic recovery.

**Proof target:** Normal provider failure releases slot, next sibling launches.
Worker panic produces exact Uncertain, does not leak capacity, does not hang
coordinator. Stage C failure halts further launches. G2 failure halts further
launches. Physical completion order inversion does not change final semantic
result.

### C3-A4 — External configuration / default / validation surface

**Objective:** Expose `max_active_together_invocations` as host configuration
with sensible defaults and validation.

**Permitted scope:** Configuration schema, default value selection, input
validation, documentation.

**Proof target:** Invalid N rejected. Default N produces correct bounded
behaviour. Configuration is observable in Trail or host metadata if required.

### C3-V1 — Independent final architectural review and full verification

**Objective:** Independent reviewer verifies the entire C3 implementation
against this design, the frozen A3a inputs, and the required proof matrix.

**Permitted scope:** Read-only review, test execution, evidence collection.

**Proof target:** All 14 future-proof matrix items pass. No semantic contract
violations. No trust boundary breaches. No regressions on A3a or C1 tests.

## 14. Required Future Proof Matrix

The following deterministic proofs are required for C3 acceptance:

### 1. N=1, group=5

- max active exactly 1 at all times
- every eligible member eventually invokes
- all five members reach terminal state
- join evaluates all five

### 2. N=2, group=5

- max active never exceeds 2
- observed max reaches 2 (proves the bound is exercised, not trivially satisfied)
- every eligible member invokes
- all five members reach terminal state

### 3. N >= group size

- preserves A3a physical overlap behaviour
- all members eligible for simultaneous launch
- no unnecessary queuing

### 4. Waiting member state

- G0 exists in replay
- durable intent exists in Trail
- G1 absent from replay
- provider untouched (no worker spawned)

### 5. Queue wait longer than member timeout

- member waits for capacity longer than its `timeout_ms`
- when launched, the provider timeout starts fresh
- member is NOT classified as timed out due to queue wait

### 6. Next capacity launches earliest semantic order

- multiple PREPARED_WAITING members exist
- when capacity becomes available, the earliest semantic-order member launches
- not the most recently prepared, not random, not provider-affinity-based

### 7. Normal provider failure

- member A fails (provider returns error)
- next queued sibling B still launches
- join evaluates both A (Failed) and B

### 8. Worker panic

- worker panics during provider invocation
- coordinator catches panic via existing `catch_unwind` / `JoinHandle` path
- member classified as exact Uncertain
- capacity is not leaked (active_count decreases)
- next waiting sibling launches
- no coordinator hang

### 9. Physical completion order inversion

- member B completes before member A (reverse of semantic order)
- final group semantic result is unchanged from the case where A completes first
- first non-success selection is by semantic member order

### 10. GroupJoin timing

- GroupJoin is absent while any member is active OR legitimately waiting
- GroupJoin appears only after every semantic member has reached terminal state
- join cannot be appended prematurely

### 11. Trusted Stage C durability failure

- after Trail / G2 / anchor write failure becomes known
- no additional provider effect launches
- existing active workers may complete (their results are not lost)
- but no NEW workers are launched

### 12. Replay G2 failure

- after G2 publication failure becomes known
- no additional provider effect launches
- same fail-closed principle as §11

---

This document is a design artifact. It does not authorise implementation.
C3-A1 implementation requires a separate approved task packet after Lucy
acceptance of this design.
