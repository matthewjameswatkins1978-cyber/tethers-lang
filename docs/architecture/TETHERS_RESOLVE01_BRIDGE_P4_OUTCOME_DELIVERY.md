# Tethers x Resolve01 P4 — Outcome Delivery and Conformance

Status: P4 accepted on main at PR #34; P4a hardening is on its own branch

P4 adds the smallest Tethers-owned delivery seam after the existing durable
provider outcome boundary. It does not add a transport, a message bus, a
Resolve database, a retrying executor, or any change to provider outcome
classification.

## Boundary

The adapter receives exactly:

```text
TethersActionRef
SUCCEEDED | FAILED | UNCERTAIN
```

`TethersActionRef` is the bounded opaque projection of the host-created durable
`ExecutionId` from replay admission. No planner `ActionId` is used as this
reference. `ActionId` names the planned logical action; `ExecutionId` names the
host-admitted execution. Resolve sees `ExecutionId` as its opaque `action_ref`
at both guard admission and outcome delivery.

No Goal, Commitment, Claim,
worker, DAG, policy, Resolve state, provider output, or adapter diagnostic
crosses this boundary. Tethers remains authoritative for classification and
durable outcome publication; Resolve only receives the closed outcome fact.

The delivery coordinator is called only after the existing Trail outcome and
replay terminal publications succeed. It has no executor reference and cannot
rerun a provider. Resolve delivery failure therefore leaves the Tethers
provider outcome and Result Anchor taxonomy unchanged.

## Delivery recovery state

The narrow append-only delivery journal stores the state sequence for each
action reference:

```text
none -> Pending -> Delivered
              -> Indeterminate -> Pending
```

`Pending` is written before the first adapter call. An adapter error records
`Indeterminate`; an explicit retry records `Pending` before calling again; a
successful call records `Delivered`. `Delivered` is terminal delivery state.
Repeating an already-delivered coordination fact is a local idempotent success:
it appends nothing and does not contact Resolve again. Reopening the journal
validates the complete sequence and rejects malformed histories rather than
selecting their last record. A different outcome for an existing action
reference is rejected before the adapter and cannot replace the recorded
provider truth.

The journal is a delivery-recovery state family, not replay authority and not
a generic queue. Existing replay still prevents provider re-execution, while
the existing Trail receives only bounded action-reference digest, outcome and
delivery-state evidence. No secrets or provider payloads are written there.

## Semantic handshake fixture

The cross-project conformance fixture pins the accepted Tethers baseline,
the P4 implementation checkpoint, and Resolve R0 commit
`8d42e5b061f86b2b2a2c1949c629654968a550ff`. It proves:

```text
prepare
 -> externally issued guard
 -> resume
 -> durable intent
 -> Resolve admission
 -> one provider call
 -> durable Tethers outcome
 -> Resolve delivery
 -> exact duplicate delivery
 -> journal reopen/retry
```

The identity-continuity test runs the actual Tethers guarded serial boundary:
P1 preparation, replay admission, durable intent, P2 admission with the
resulting `ExecutionId`, one provider call, durable outcome, and P4 delivery
through a stateful Resolve-contract double. HTTP, MCP, A2A and live
cross-repository transport remain outside P4/P4a.
