# C2-A3 Physical Concurrency Boundary Design

Status: design artifact — NOT implementation authority

Updated: 2026-08-14

## 1. Current Execution Ownership Map

```text
HostExecutionService
  owns: Runtime (policy, requirements, trusted store, providers)
  owns: HashMap<String, RetainedProviderSession>   (one per provider)
  owns: ApprovalStore
  borrows: PreparedEvaluationInput (anchor event, replay root)
  borrows: &mut dyn ReplayAuthority

execute_plan(items, actions, trail, execute_action closure)
  borrows: &mut dyn Trail                        (single writer)
  closure captures: &mut self (HostExecutionService)
                    &mut provider_sessions
                    &mut approvals
                    input
                    replay_authority

For each PlanItem (Sequential or Group):

  execute_one_action:
    1. scope assessment          (Runtime — shared read)
    2. policy evaluation         (Runtime — shared read)
    3. denied/unavailable/ask    (early return)
    4. capability resolution     (Runtime — shared read)
    5. provider session lookup   (get_mut from HashMap)
    6. catalogue refresh         (session — mutable borrow)
    7. ProviderSessionExecutor   (borrows &mut session)
    8. execute_shared_boundary:
       a. bridge pin verification
       b. replay admission       (admit — creates ReplayAdmission)
       c. replay G0 intent      (publish_intent)
       d. dispatch intent        (prepare_and_record → Trail append)
       --- boundary: preparation complete, invocation not yet begun ---
       e. deadline preparation   (clock.now() — per-member, not shared)
       f. replay G1 armed       (publish_armed)
       g. presentation entry    (response trail push)
       h. provider invocation   (executor.execute_classified)
       i. outcome classification
       j. Trail outcome append
       k. replay G2 terminal    (publish_terminal)
       l. result anchor write
       m. response update

GroupJoinEntry appended after all members terminal.
```

Key ownership observations:

- `RetainedProviderSession` takes `&mut self` for `tools_call` — serial MCP protocol.
- `ReplayAdmission` holds `Rc<ReplayLedger>` — not Send.
- `FileTrail` is a single-writer append + flush + sync_data.
- `response: Value` is mutated throughout — shared presentation state.
- `ProviderSessionExecutor` borrows `&mut RetainedProviderSession`.

## 2. Concurrency Blockers

| Component | Current Ownership | Classification | Reason |
|-----------|------------------|----------------|--------|
| ProviderSessionExecutor | borrows `&mut RetainedProviderSession` | A — MUST CHANGE | Cannot overlap calls through one session |
| RetainedProviderSession | `HashMap` in HostExecutionService | B — CAN REMAIN SERIAL | Sessions remain coordinator-owned |
| ReplayAdmission | `Rc<ReplayLedger>` — not Send | C — CAN REMAIN UNCHANGED | Admission stays coordinator-owned |
| FileTrail | `&mut dyn Trail` — single writer | C — CAN REMAIN UNCHANGED | Coordinator appends outcomes |
| response: Value | `&mut Value` — mutable | C — CAN REMAIN UNCHANGED | Coordinator owns response |
| CapabilityExecutor trait | `&mut self` on execute | A — MUST CHANGE | Workers need independent invocation |
| Policy/resolution | Runtime shared read | C — CAN REMAIN UNCHANGED | Prepared before overlap |
| ApprovalStore | `&mut` in HostExecutionService | C — CAN REMAIN UNCHANGED | Prepared before overlap |
| Deadline clock | `ProductionMonotonicClock` | C — CAN REMAIN UNCHANGED | Created per-invocation in STAGE B, not shared across group |
| ResultAnchorWriter | `&mut dyn ResultAnchorWriter` | C — CAN REMAIN UNCHANGED | Coordinator writes anchors |
| Provider catalogue | Runtime shared read | C — CAN REMAIN UNCHANGED | Refreshed per-session |

## 3. Chosen Concurrency Unit

**ONLY the external provider invocation portion of Together members becomes concurrent.**

Everything that must remain deterministically coordinated stays on a single coordinator:

- policy / permission decisions
- capability resolution
- provider availability checks
- replay admission (G0 intent, G1 armed, G2 terminal)
- durable Trail writes (intent, outcome, join)
- result anchor writes
- response presentation
- join evaluation
- deadline establishment (per-member, immediately before launch)

The coordinator is the single source of truth. Workers are stateless invocation carriers that establish independent ephemeral provider sessions.

## 4. Staged Execution Model

### STAGE A — Serial Deterministic Preparation

For every Together member in Runtime Plan order:

1. Scope assessment
2. Policy / permission decision (Deny → early Err; Ask → approval; Unavailable → early Err)
3. Capability resolution
4. Provider availability check
5. Replay admission (`admit`)
6. Replay G0 intent (`publish_intent`)
7. Durable Trail intent (`prepare_and_record`)

All of this is coordinator-owned, serial, deterministic.

**Deadline and G1 are NOT prepared in STAGE A.** These must happen immediately before each member's worker launch (STAGE B). See §4a for why.

**If member B fails preparation (policy denied, unavailable, replay blocked), member A is still eligible for invocation.** The "all members are attempted" rule applies at the provider invocation boundary, not the preparation boundary. A preparation failure is an honest early classification — the member was attempted through preparation and classified before invocation.

### STAGE B — Physical Provider Invocation

Once all eligible members are prepared (STAGE A complete):

For each armed member, immediately before worker launch:

1. **Establish deadline start** (`clock.now()`) — per-member, not shared across group
2. **Calculate remaining deadline** from member's `timeout_ms`
3. **Final pre-invocation deadline check** — if expired, classify as Unattempted (do not launch)
4. **Replay G1 armed** (`publish_armed`) — immediately before launch, per-member
5. **Launch worker** in scoped thread

This preserves the accepted meaning: a member's provider execution timeout does not begin merely because unrelated siblings are still being prepared. The deadline starts only when that specific member is about to invoke.

**Critical ordering for each member:**

```
G0 intent (STAGE A) → durable Trail intent (STAGE A) → deadline start (STAGE B) → G1 armed (STAGE B) → provider invocation (STAGE B)
```

No worker may cause an effect before the coordinator has successfully published G1 for that specific member.

### STAGE C — Durable Result Collection

As provider results physically become available (coordinator receives from workers):

1. Classify outcome (Succeeded / Failed / Uncertain).
2. Append truthful member outcome promptly in **physical completion order** (coordinator-owned Trail).
3. Publish replay terminal state (G2) for each member.
4. Write Result Anchor if semantically required.
5. Preserve each member's SemanticPosition.

**Physical Trail append order is durable append order, not provider completion order.** The coordinator appends outcomes in the order it receives worker results. Two providers completing almost simultaneously may have their results received in an order that does not exactly match physical completion. This is truthful: the Trail records when the coordinator learned about each completion. Do not sort Trail physically after the fact.

### STAGE D — Join

After every member has a terminal Trail record:

1. Append `GroupJoinEntry` with join semantic position.
2. Evaluate all-success join.
3. Continue or stop later Actions.

## 5. Provider Session Decision

### Provider concepts: session, process, identity

| Concept | Definition | Ownership |
|---------|-----------|-----------|
| Provider identity | Capability-resolved name (e.g. `github.com/org/repo/provider`). Determined by `resolve_exact_capability`, not by process count. | Capability resolution — immutable per action |
| Session | A `RetainedProviderSession` that owns one `ManagedProvider` which owns one `SupervisedChild`. Carries: request ID sequence, catalogue freshness flag, cached catalogue. | Coordinator-owned `HashMap<String, RetainedProviderSession>` |
| Process | One OS child process launched by `ManagedProvider::launch`. Each session holds exactly one process. | Session-owned (via `ManagedProvider`) |

Key invariant: **Provider identity is intentionally independent of process instance.** Two sessions with the same provider identity are two independent child processes running the same provider binary. The Tethers contract does not require a 1:1 mapping between identity and process.

### Retained process-local state question

`RetainedProviderSession` maintains per-session state:

- `next_request_id: u64` — monotonic JSON-RPC request ID sequence
- `catalogue_stale: bool` — dirty flag blocking invocation until rediscovery
- `catalogue: Option<SocketCatalogue>` — cached tool catalogue
- `catalogue_change_observed: bool` (in `ManagedProvider`) — notification flag

These are session-local concerns, not provider-identity state. The MCP protocol is stateless per `tools/call` — each invocation is an independent JSON-RPC request/response.

**However:** Tethers does not guarantee that provider implementations are stateless between calls. A provider could retain process-local state (in-memory caches, connection pools, external state). If a provider relies on retained state between serial Actions, using ephemeral sessions for Together members would break that assumption.

**Accepted A3a rule:** Sequential Actions continue using retained sessions exactly as today. Together members may use independent ephemeral sessions only where provider binding semantics make process-instance independence valid. This is safe for C2-A3a because:

1. Together members are independent actions — they do not share provider state by definition.
2. The serial path retains its sessions for post-group catalogue state.
3. If a provider implementation requires inter-call state, that is a provider-specific concern, not a Tethers semantic — and the serial path (which uses retained sessions) already handles it.

### Correct provider worker path

The current serial provider path is:

```
RetainedProviderSession::establish(SocketEstablishment{...})
  → ManagedProvider::launch(command, args, working_dir)
  → ManagedProvider::initialize(protocol, server_name)
  → catalogue_stale = true, catalogue = None

refresh_prepared_catalogue(prepared, session)
  → session.discover()
    → refresh_notification_state()
    → invalidate_catalogue()
    → provider.list_tools_paginated()
    → validate catalogue-change
    → catalogue_stale = false, catalogue = Some(...)
  → validate_prepared_discovery(catalogue, prepared)

session.tools_call(tool_name, arguments, remaining)
  → refresh_notification_state()
  → require_fresh_catalogue()           ← BLOCKS if stale
  → provider.tools_call_with_timeout()
  → observe catalogue-change notifications
```

The proposed A3a worker path **must preserve this exact contract:**

```
Worker receives: PreparedProvider (config, identity, trusted bindings), tool_name, arguments, deadline_remaining

Worker establishes ephemeral session:
  1. ManagedProvider::launch(command, args, working_dir)
  2. ManagedProvider::initialize(protocol_version, server_name)
     → catalogue_stale = true, catalogue = None

Worker discovers catalogue:
  3. session.discover()
     → refresh_notification_state()
     → invalidate_catalogue()
     → provider.list_tools_paginated()
     → catalogue_stale = false, catalogue = Some(...)

Worker validates trusted binding:
  4. verify required operation remains the trusted resolved operation
     (validate_prepared_discovery equivalent against PreparedProvider bindings)

Worker invokes:
  5. session.tools_call(tool_name, arguments, deadline_remaining)
     → refresh_notification_state()
     → require_fresh_catalogue()        ← passes (catalogue is fresh)
     → provider.tools_call_with_timeout()

Worker observes notifications:
  6. observe catalogue-change notifications (informational only for ephemeral session)

Worker returns:
  7. Result<Value, ProviderDiagnostic>
     or classified error

Worker closes:
  8. session.close()
     → ManagedProvider shutdown child process
```

**Why this is safe:** The worker uses the same `RetainedProviderSession::establish` + `discover` + `tools_call` path as the serial coordinator. No contract is bypassed. The ephemeral session is established, discovered, validated, invoked, and closed — identical to a serial session, but scoped to a single invocation.

**Do NOT call `ManagedProvider::tools_call` directly.** Always go through `RetainedProviderSession` to honour the Socket freshness contract.

**Session lifecycle in workers (corrected):**

```text
Worker receives: PreparedProvider, tool_name, arguments, deadline_remaining
Worker establishes: RetainedProviderSession::establish(SocketEstablishment{
    command, args, working_directory, protocol_version, server_name, identity
})
Worker discovers: refresh_prepared_catalogue(prepared, &mut session)
Worker invokes: session.tools_call(tool_name, arguments, deadline_remaining)
Worker returns: Result<Value, ProviderDiagnostic>
Worker closes: session.close()
```

### Same-provider overlap

| Question | Answer |
|----------|--------|
| Can two members targeting the SAME provider overlap? | YES — each gets its own child process (independent sessions) |
| Can two members targeting DIFFERENT providers overlap? | YES — independent sessions |
| Does provider identity remain unchanged? | YES — identity is determined by capability resolution, not session count |
| Are there twice as many provider processes? | Only for the duration of the Together group. Ephemeral sessions are closed after invocation. |

**Why independent connections are safe:** The MCP protocol is stateless per tool invocation. Each `tools_call` sends a JSON-RPC request and waits for a response. Multiple concurrent child processes for the same provider identity are independent OS processes with independent stdio channels. The provider process handles each independently — that is the provider's runtime detail, not Tethers' concern.

The coordinator retains its existing `RetainedProviderSession` instances for serial Actions and for post-group catalogue state. The worker sessions are ephemeral invocation-only connections.

## 6. Replay Ownership Decision

**ReplayAdmission remains coordinator-owned. Never moves to workers.**

| Question | Answer |
|----------|--------|
| Does ReplayAdmission move between threads? | NO |
| Must Rc change to Arc? | NO — admission stays on coordinator |
| How is G0 published? | Coordinator publishes G0 in STAGE A, before worker launch |
| How is G1 published? | Coordinator publishes G1 in STAGE B, immediately before each member's worker launch |
| How is G2 published? | Coordinator publishes G2 in STAGE C, after receiving worker result |

The admission lifecycle remains identical to serial execution:

```text
admit() → publish_intent() → [provider invocation] → publish_terminal()
```

The key change is that G1 is published in STAGE B (immediately before worker launch), not in STAGE A. This preserves the accepted meaning: G1 armed means "this member is about to invoke" — and it must not be armed while other members are still being prepared.

**G1 exact relationship to worker launch:** G1 is published immediately before the scoped thread is spawned for that specific member. No worker may cause an effect before the coordinator has successfully published G1 for that specific member. The admission guard retains cross-process exclusion through the call and final publication.

**Why this works:** `ReplayAdmission` uses `Rc<ReplayLedger>` which is !Send. Since the admission never crosses a thread boundary, this is not a problem. The worker never needs the admission — it only needs the `DispatchReadyAction` (which is an owned value), the `PreparedProvider` (for ephemeral session establishment), and the remaining deadline.

## 7. Trail Ownership Decision

**One coordinator-owned Trail writer. Workers never touch Trail.**

| Question | Answer |
|----------|--------|
| Writer ownership | Coordinator — `&mut dyn Trail` |
| Intent durable before provider effect | YES — STAGE A writes intent before STAGE B |
| Outcome durable promptly after completion | YES — STAGE C appends outcome immediately after coordinator receives worker result |
| Physical JSONL order | Durable append order = coordinator receive order |
| SemanticPosition | Deterministic — flat Runtime Plan index + phase |
| GroupJoin | Appended after all member terminal records |

**Physical ordering meaning (corrected):**

Authoritative physical Trail order is **durable append order** — the order in which the coordinator durably recorded each outcome.

Coordinator receive order is the order in which the coordinator observes worker results via channel. This is *usually* the same as durable append order (coordinator receives, then immediately appends). But two providers completing almost simultaneously may have their results received in an order that does not exactly match physical completion — channel scheduling, thread wakeup, and OS scheduling can reverse observation order.

**Trail proves:** "the coordinator durably recorded B before A."

**Trail does NOT necessarily prove:** "B's provider physically completed before A's provider."

Under serial execution, physical order matches semantic order. Under concurrency, it may differ. Both are truthful.

**SemanticPosition preserves program order.** Regardless of physical Trail append order, each member's `action_ordinal` remains its flat Runtime Plan index. This is the deterministic program-order anchor.

## 8. Deterministic Result Rule

The following remain deterministic regardless of physical completion order:

| Output | Deterministic? | Source |
|--------|---------------|--------|
| Semantic Action identity | YES | Flat Runtime Plan index |
| Semantic position | YES | `action_ordinal` = flat index, `phase` = member/action |
| Member ordinal | YES | `member_action_ids` order |
| Join result (success/failure) | YES | All-success test is commutative |
| Later Action eligibility | YES | Join result determines continuation |
| Final plan result classification | YES | Deterministic member selection (see below) |
| Trail physical append order | NO (intentionally) | Reflects coordinator receive order, which may differ from provider physical completion order |

**Final failure selection:** When a join is non-success, the "first non-success member" is selected in **semantic Runtime Plan member order** (the order of `member_action_ids`), NOT physical completion order. This preserves deterministic plan results.

## 9. Failure / Cancellation Rule

### Core rule: Do not cancel siblings

C1 says all members are attempted before join. Physical concurrency does not change this.

### Attempted semantics (corrected)

"Member attempted through the Together fan-out semantics" is broader than "provider invocation attempted."

- **Attempted through preparation:** The member was processed through the preparation pipeline (scope, policy, resolution, replay admission, intent). A preparation failure (policy denied, unavailable, replay blocked, intent write failure) is an honest early classification — the member was attempted and classified before invocation.
- **Attempted through invocation:** The member's provider invocation crossed the invocation boundary (G1 armed, worker launched, `tools_call` sent). This is the stronger claim.
- **Not attempted:** The member failed preparation before reaching the invocation boundary. This is NOT the same as "not attempted through the Together fan-out." The member was attempted through preparation.

### Preparation failures

If member A prepares successfully but member B fails before provider invocation (policy denied, unavailable, replay blocked, intent write failure):

- Member A **is still invoked** (it is eligible for STAGE B).
- Member B is classified as its preparation failure result (Denied, Unavailable, etc.).
- The join evaluates all terminal members.

A preparation failure does not mark a provider invocation as attempted — it never crossed its invocation boundary. But it IS an attempted member through the Together fan-out semantics.

### Provider failures

| Scenario | Behaviour |
|----------|-----------|
| One member success, one explicit failure | Join fails; plan stops |
| One member uncertain | Join fails; plan stops |
| One member replay blocked completed-success | Counted as success for join |
| One member replay blocked completed-failure | Counted as failure for join |
| One member unavailable | Join fails; plan stops |
| One member policy denied | Classified in preparation; still attempted |
| One member approval required | Classified in preparation; member not armed |
| One member deadline before invocation | Classified in preparation; member not armed |
| One provider process lost | Classified as Uncertain |
| Worker panic | Coordinator catches thread panic; classified as Uncertain |
| Coordinator interruption | Honest process limitation — members may be mid-flight |
| Trail outcome write failure | AuditFailed for the plan |
| Replay terminal publication failure | ReplayDispatchResult::PersistenceUnavailable |

### Worker panic handling

Workers run in scoped threads. If a worker panics, the coordinator catches the panic (via `std::panic::catch_unwind` or `JoinHandle`) and classifies the member as Uncertain. This is honest: the provider invocation was attempted but no trustworthy result is available.

## 10. C2 / C3 Boundary

| Concern | C2 | C3 |
|---------|----|----|
| Provider invocation overlap | YES | — |
| Scoped thread per member | YES | — |
| Independent provider connections | YES | — |
| Worker pool sizing | NO | YES |
| CPU accounting | NO | YES |
| Provider quotas | NO | YES |
| Fair scheduling | NO | YES |
| Queue priorities | NO | YES |
| Adaptive concurrency | NO | YES |
| Rate limiting | NO | YES |
| Global resource scheduler | NO | YES |

C2 introduces actual overlap bounded by group size. C3 owns general resource limits.

## 11. C2-A3a Proposed Implementation

### Exact scope

C2-A3a — provider invocation overlap under coordinator ownership.

- Sequential Actions unchanged (serial, byte-compatible).
- Only Together group provider calls overlap.
- Group members prepared deterministically in STAGE A (scope, policy, resolution, replay admission, G0 intent, Trail intent).
- Deadline + G1 armed established per-member in STAGE B, immediately before worker launch.
- Replay admission remains coordinator-owned.
- Trail remains coordinator-owned.
- Provider invocation executes in scoped threads (STAGE B), each worker establishing an independent ephemeral `RetainedProviderSession`.
- Results returned to coordinator via channel.
- Coordinator persists terminal evidence (STAGE C).
- GroupJoin remains after all terminal (STAGE D).
- Deterministic semantic result selection.
- No general resource scheduler.

### Files likely affected

| File | Change |
|------|--------|
| `plan_execution.rs` | Add concurrent group execution path (new function or branch) |
| `host_execution.rs` | New worker function for provider invocation; modified `execute_one_action` to return prepared context (not armed) |
| `application.rs` | New worker result type; modified `execute_boundary_impl` to separate preparation from invocation; deadline + G1 moved to per-member launch |
| `socket.rs` | New function to create ephemeral provider connection for workers (using existing `RetainedProviderSession::establish` + `refresh_prepared_catalogue`) |
| `executor.rs` | New worker-scoped executor type (not trait change) |

### Ownership changes

- `DispatchReadyAction` — already owned, passed to worker by value.
- Worker receives: `DispatchReadyAction`, `PreparedProvider` (for ephemeral session establishment), tool name, deadline.
- Worker establishes: `RetainedProviderSession::establish` → `refresh_prepared_catalogue` → `session.tools_call` → `session.close`.
- Worker returns: `WorkerResult` (action_index, semantic_position, provider result/diagnostic, timing).
- Coordinator retains: Trail, ReplayAdmission, response, approvals, anchor writer.
- `ProviderSessionExecutor` — no change to existing type; workers use a new ephemeral executor that follows the full Socket establishment/discovery/invocation path.

### A3a proposal — revised answers

| # | Question | Answer |
|---|----------|--------|
| 1 | What state is prepared serially before fan-out? | Scope, policy, resolution, replay admission, G0 intent, Trail intent (STAGE A). Deadline + G1 are NOT prepared serially. |
| 2 | When exactly does each member's deadline start? | In STAGE B, immediately before that member's worker launch, after all other members are prepared. Per-member `clock.now()`. |
| 3 | When exactly is G1 written? | In STAGE B, immediately before that member's worker launch, after deadline establishment. Per-member. |
| 4 | What exact object/config enters the worker? | `DispatchReadyAction` (owned), `PreparedProvider` (cloned — contains config, identity, trusted bindings), tool name, deadline remaining. |
| 5 | How does the worker establish a trusted provider invocation path? | `RetainedProviderSession::establish(SocketEstablishment{...})` → `refresh_prepared_catalogue(prepared, session)` → `session.tools_call(tool_name, args, remaining)` → `session.close()`. |
| 6 | Does it launch a new process? | YES — each worker launches an independent child process via `ManagedProvider::launch`. Process is closed after invocation. |
| 7 | Does same-provider overlap remain supported? | YES — two workers can launch two child processes for the same provider identity. |
| 8 | Are retained provider session semantics preserved? | YES — the serial path retains its `RetainedProviderSession` instances for serial Actions and post-group state. Workers use independent ephemeral sessions. |
| 9 | What worker result returns to the coordinator? | `WorkerResult` containing: action_index, semantic_position, `Result<Value, ProviderDiagnostic>`, timing evidence. |
| 10 | What does Trail physical order actually prove? | "The coordinator durably recorded B before A." It does NOT prove "B's provider physically completed before A's provider." SemanticPosition preserves program order. |

## 12. Required A3a Tests

### 1. TWO MEMBERS ACTUALLY OVERLAP

Use controlled provider barriers/counters proving B starts before A completes. Two providers with a shared test barrier: provider A blocks until provider B has started, then both complete.

### 2. ALL MEMBERS ATTEMPTED

Failure of one member does not prevent another invocation. Prepare member A with a working provider and member B with a failing provider. Verify both are invoked and the join evaluates correctly.

### 3. DETERMINISTIC FINAL FAILURE

Different physical completion orders yield the same semantic final member failure selection. Run the same plan twice with controlled timing; verify the same member is selected as the "first non-success" in semantic order.

### 4. TRAIL PHYSICAL ORDER TRUTH

Outcome append order reflects coordinator receive order, not semantic sorting. Verify that Trail entries are appended in the order the coordinator receives worker results. Acknowledge that coordinator receive order may not exactly match provider physical completion order.

### 5. SEMANTIC POSITION STABILITY

Positions remain flat Runtime Plan based. Verify `action_ordinal` equals flat index for all members regardless of completion order.

### 6. JOIN AFTER TERMINAL

Join cannot append before all member terminal records. Verify the join Trail entry appears after all member outcome entries.

### 7. REPLAY SAME KEY EXCLUSION

Existing logical-key exclusion remains. Two members with the same logical key: only one gets a fresh admission.

### 8. DISTINCT REPLAY MEMBERS

No global replay serialization introduced. Two members with different logical keys get independent admissions.

### 9. INTENT BEFORE EFFECT

Every member's durable intent exists before its provider begins. Verify Trail intent entries precede provider invocation.

### 10. SERIAL PLAN COMPATIBILITY

Non-Together plans remain physically serial and byte/behaviour compatible with pre-A3a output.

### 11. SAME PROVIDER

Two calls to same provider physically overlap via independent child processes. Each worker establishes its own `RetainedProviderSession` → `ManagedProvider::launch` → `initialize` → `discover` → `tools_call` → `close`.

### 12. DIFFERENT PROVIDERS

Prove expected overlap behaviour with two different providers.

### 13. DEADLINE PER MEMBER (NEW)

Member A's deadline does not begin until STAGE B, after all members are prepared. Verify that preparing member B does not consume member A's timeout. Use a slow preparation for B and verify A's deadline starts fresh.

### 14. PROVIDER SESSION ESTABLISHMENT (NEW)

Verify that each worker establishes the full Socket contract: `establish` → `discover` → `tools_call`. Verify that `require_fresh_catalogue` passes (catalogue is not stale). Verify that the worker does NOT call `ManagedProvider::tools_call` directly.

### 15. G1 BEFORE INVOCATION (NEW)

Verify that G1 is published immediately before each member's worker launch, not in STAGE A. Verify that a member's G1 is published only when that specific member is about to invoke.

## 13. Explicit Non-Goals

- Physical concurrency for sequential Actions
- Worker pool / thread pool sizing (C3)
- Provider rate limiting / quotas (C3)
- Retry logic
- Cancellation propagation
- Execution DAG
- Nested Together
- Trail schema changes
- Replay identity changes
- Canonical V2 / Rocket changes
- Approval redesign
- Result anchor redesign
- Bypassing Socket establishment/discovery/invocation contract
- Shared deadline across group members
- G1 publication before all members are prepared

## 14. Unresolved Blockers

None. The design is implementable within existing architecture constraints.

**Provider process overhead:** Creating a new child process per invocation adds launch overhead. This is acceptable for C2-A3a because:
1. It is the smallest safe boundary.
2. The overhead is a runtime cost, not a semantic change.
3. C3 can introduce connection pooling or session reuse if evidence shows it matters.

**Retained provider state:** Tethers does not guarantee that provider implementations are stateless between calls. For C2-A3a this is safe because:
1. Together members are independent actions — they do not share provider state by definition.
2. The serial path retains its sessions for post-group catalogue state.
3. If a provider requires inter-call state, that is a provider-specific concern handled by the serial path.

**Deadline semantics:** The corrected deadline model (per-member in STAGE B) preserves the accepted meaning that a member's provider execution timeout does not begin merely because unrelated siblings are still being prepared. This is consistent with the serial path where `deadline_start = clock.now()` occurs immediately before provider invocation.

## 15. Platform Considerations

The concurrency mechanism (scoped threads) is platform-neutral. The replay persistence (`replay_windows.rs`) remains Windows-specific — this is an existing platform limitation, not a new A3a limitation. A3a does not change which platforms are supported.

---

This document is a design artifact. It does not authorise implementation. C2-A3a implementation requires a separate approved task packet.
