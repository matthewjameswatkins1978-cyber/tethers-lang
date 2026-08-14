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
       e. deadline preparation
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
| Deadline clock | `ProductionMonotonicClock` | C — CAN REMAIN UNCHANGED | Created per-invocation |
| ResultAnchorWriter | `&mut dyn ResultAnchorWriter` | C — CAN REMAIN UNCHANGED | Coordinator writes anchors |
| Provider catalogue | Runtime shared read | C — CAN REMAIN UNCHANGED | Refreshed per-session |

## 3. Chosen Concurrency Unit

**ONLY the external provider invocation portion of Together members becomes concurrent.**

Everything that must remain deterministically coordinated stays on a single coordinator:

- policy / permission decisions
- capability resolution
- provider availability checks
- replay admission (G0, G1, G2)
- durable Trail writes (intent, outcome, join)
- result anchor writes
- response presentation
- join evaluation

The coordinator is the single source of truth. Workers are stateless invocation carriers.

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
8. Deadline preparation
9. Replay G1 armed (`publish_armed`)

All of this is coordinator-owned, serial, deterministic.

**If member B fails preparation (policy denied, unavailable, replay blocked), member A is still armed.** The "all members are attempted" rule applies at the provider invocation boundary, not the preparation boundary. A preparation failure is an honest early classification — the member was attempted through preparation and classified before invocation.

### STAGE B — Physical Provider Invocation

Once eligible members are armed:

- Each armed member's provider invocation executes in an independent scope (thread or scoped task).
- No Trail writer ownership in workers.
- No shared mutable response in workers.
- No ReplayAdmission in workers.
- Workers receive: `DispatchReadyAction` + remaining deadline + provider identity + tool name.
- Workers return: raw provider result or classified diagnostic + timing evidence.

### STAGE C — Durable Result Collection

As provider results physically become available (coordinator receives from workers):

1. Classify outcome (Succeeded / Failed / Uncertain).
2. Append truthful member outcome promptly in **physical completion order** (coordinator-owned Trail).
3. Publish replay terminal state (G2) for each member.
4. Write Result Anchor if semantically required.
5. Preserve each member's SemanticPosition.

Physical Trail append order is intentionally allowed to differ from semantic order because it is truthful runtime evidence. Do not sort Trail physically after the fact.

### STAGE D — Join

After every member has a terminal Trail record:

1. Append `GroupJoinEntry` with join semantic position.
2. Evaluate all-success join.
3. Continue or stop later Actions.

## 5. Provider Session Decision

**Chosen: Option 2 — one independent invocation handle per worker.**

Each worker creates an independent MCP stdio connection to the provider, invokes the tool, and returns the result. The connection is dropped after invocation.

| Question | Answer |
|----------|--------|
| Can two members targeting the SAME provider overlap? | YES — each gets its own stdio connection |
| Can two members targeting DIFFERENT providers overlap? | YES — independent connections |
| Should C2 guarantee overlap only when independent sessions exist? | N/A — all invocations get independent connections |
| Would such a limitation alter Tethers semantics? | N/A |
| Does provider identity remain unchanged? | YES — identity is determined by capability resolution, not session count |

**Why not mutex one session:** That would produce concurrency-shaped code with serial provider calls — no actual overlap.

**Why independent connections are safe:** The MCP protocol is stateless per tool invocation. The `tools_call` method sends a JSON-RPC request and waits for a response. Multiple concurrent connections to the same provider process are independent TCP/stdio channels. The provider process itself may or may not handle them concurrently — that is the provider's runtime detail, not Tethers' concern.

**Session lifecycle in workers:**

```
Worker receives: provider_identity, tool_name, arguments, deadline_remaining
Worker creates: ManagedProvider::launch(provider_config)
Worker calls: provider.initialize(protocol, server_name)
Worker calls: provider.tools_call(tool_name, arguments, remaining)
Worker returns: Result<Value, ProviderDiagnostic>
Worker drops: provider (closes child process)
```

The coordinator retains its existing `RetainedProviderSession` instances for serial Actions and for any post-group catalogue state. The worker sessions are ephemeral invocation-only connections.

## 6. Replay Ownership Decision

**ReplayAdmission remains coordinator-owned. Never moves to workers.**

| Question | Answer |
|----------|--------|
| Does ReplayAdmission move between threads? | NO |
| Must Rc change to Arc? | NO — admission stays on coordinator |
| How is G1 published before launch? | Coordinator publishes G1 in STAGE A, before spawning workers |
| How is G2 published after result? | Coordinator publishes G2 in STAGE C, after receiving worker result |

The admission lifecycle remains identical to serial execution:

```text
admit() → publish_intent() → [provider invocation] → publish_terminal()
```

The only change is that "provider invocation" happens in a worker thread while the coordinator holds the admission. The coordinator publishes G2 after the worker returns.

**Why this works:** `ReplayAdmission` uses `Rc<ReplayLedger>` which is !Send. Since the admission never crosses a thread boundary, this is not a problem. The worker never needs the admission — it only needs the `DispatchReadyAction` (which is an owned value) and the remaining deadline.

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

**Physical ordering meaning:**

When result messages cross a channel (worker → coordinator), the coordinator receives them in physical completion order. The coordinator appends outcomes in that receive order. This means Trail physical order = **coordinator receive order** = durable append completion order.

This is truthful: the Trail physically records when the coordinator learned about each completion. Under serial execution this matches semantic order; under concurrency it may differ. Both are truthful.

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
| Trail physical append order | NO (intentionally) | Reflects physical completion timing |

**Final failure selection:** When a join is non-success, the "first non-success member" is selected in **semantic Runtime Plan member order** (the order of `member_action_ids`), NOT physical completion order. This preserves deterministic plan results.

## 9. Failure / Cancellation Rule

### Core rule: Do not cancel siblings

C1 says all members are attempted before join. Physical concurrency does not change this.

### Preparation failures

If member A prepares successfully but member B fails before provider invocation (policy denied, unavailable, replay blocked, intent write failure):

- Member A **is still invoked** (it is already armed).
- Member B is classified as its preparation failure result (Denied, Unavailable, etc.).
- The join evaluates all terminal members.

"Attempted" means: the member was processed through the preparation pipeline. A preparation failure is an honest classification, not a skipped attempt.

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
- Group members all prepared deterministically in STAGE A.
- Replay admission remains coordinator-owned.
- Trail remains coordinator-owned.
- Provider invocation executes in scoped threads (STAGE B).
- Results returned to coordinator via channel.
- Coordinator persists terminal evidence (STAGE C).
- GroupJoin remains after all terminal (STAGE D).
- Deterministic semantic result selection.
- No general resource scheduler.

### Files likely affected

| File | Change |
|------|--------|
| `plan_execution.rs` | Add concurrent group execution path (new function or branch) |
| `host_execution.rs` | New worker function for provider invocation; modified `execute_one_action` to return armed member context |
| `application.rs` | New worker result type; modified `execute_boundary_impl` to separate preparation from invocation |
| `socket.rs` | New function to create ephemeral provider connection for workers |
| `executor.rs` | New worker-scoped executor type (not trait change) |

### Ownership changes

- `DispatchReadyAction` — already owned, passed to worker by value.
- Worker receives: `DispatchReadyAction`, provider config, tool name, deadline.
- Worker returns: `WorkerResult` (action_index, semantic_position, provider result/diagnostic, timing).
- Coordinator retains: Trail, ReplayAdmission, response, approvals, anchor writer.
- `ProviderSessionExecutor` — no change to existing type; workers use a new ephemeral executor.

### Explicit non-goals

- General resource scheduling (C3)
- Worker pool sizing beyond group membership
- Provider rate limiting
- Retry logic
- Cancellation propagation
- Execution DAG
- Nested Together
- Physical concurrency for sequential Actions
- Trail schema changes
- Replay identity changes
- Canonical V2 / Rocket changes

## 12. Required A3a Tests

### 1. TWO MEMBERS ACTUALLY OVERLAP

Use controlled provider barriers/counters proving B starts before A completes. Two providers with a shared test barrier: provider A blocks until provider B has started, then both complete.

### 2. ALL MEMBERS ATTEMPTED

Failure of one member does not prevent another invocation. Prepare member A with a working provider and member B with a failing provider. Verify both are invoked and the join evaluates correctly.

### 3. DETERMINISTIC FINAL FAILURE

Different physical completion orders yield the same semantic final member failure selection. Run the same plan twice with controlled timing; verify the same member is selected as the "first non-success" in semantic order.

### 4. TRAIL PHYSICAL ORDER TRUTH

Outcome append order reflects coordinator receive order, not semantic sorting. Verify that Trail entries are appended in the order the coordinator receives worker results.

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

Explicitly test or document whether two calls to same provider can physically overlap in A3a. With independent connections: YES.

### 12. DIFFERENT PROVIDERS

Prove expected overlap behaviour with two different providers.

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

## 14. Unresolved Blockers

None. The design is implementable within existing architecture constraints.

The primary architectural risk is the ephemeral provider connection lifecycle: creating a new MCP stdio connection per invocation adds process launch overhead. This is acceptable for C2-A3a because:

1. It is the smallest safe boundary.
2. The overhead is a runtime cost, not a semantic change.
3. C3 can introduce connection pooling or session reuse if evidence shows it matters.

## 15. Platform Considerations

The concurrency mechanism (scoped threads) is platform-neutral. The replay persistence (`replay_windows.rs`) remains Windows-specific — this is an existing platform limitation, not a new A3a limitation. A3a does not change which platforms are supported.

---

This document is a design artifact. It does not authorise implementation. C2-A3a implementation requires a separate approved task packet.
