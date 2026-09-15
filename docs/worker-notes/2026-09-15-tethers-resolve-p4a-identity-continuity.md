# Tethers x Resolve01 P4a worker note

Task: `Tethers x Resolve01 / P4a - Identity Continuity and Delivery Recovery Hardening`

Task packet: `User-authorized P4a repair packet; Resolve R0 authority is commit 8d42e5b061f86b2b2a2c1949c629654968a550ff`

Owner: `Codex`

Status: `IMPLEMENTATION IN PROGRESS`

Base commit: `9fb8517acd1192e9a1dbee8fa70c0f520d6c0fad`

## Frozen objective

Prove that the host-created replay `ExecutionId` is the exact opaque Resolve
`action_ref` used at both guard admission and outcome delivery. Preserve the
planner `ActionId` as preparation/logical identity evidence. Make the P4
delivery journal monotonic and terminal at `Delivered` without adding a
transport, executor, retry loop, or provider access.

## Changes made

The P2 adapter now receives a private-field
`ResolveGuardAdmissionRequest` constructed only from a real
`DispatchReadyAction`. Its action reference is the ready token's
host-created `ExecutionId`; the adapter no longer receives planner `ActionId`
or a separately supplied action reference.

The P4 delivery store now validates complete per-action histories:

```text
none -> Pending -> Delivered
              -> Indeterminate -> Pending
```

An already durable `Delivered` record is locally idempotent and never calls
Resolve again. Restart and Trail-failure tests preserve that terminal truth.

## Evidence targets

- admission action reference equals replay `DispatchReadyAction.execution_id`;
- outcome delivery uses the same opaque identity;
- ActionId and ExecutionId remain distinct types and values;
- provider execution remains exactly once;
- duplicate Delivered delivery makes zero adapter calls;
- Indeterminate retry is allowed and makes one new adapter attempt;
- malformed journal histories fail closed;
- P2 rejection remains provider-free.

## Explicit exclusions

No HTTP, MCP, A2A, Resolve database, scheduler, background retry loop,
provider retry, replay semantic change, OCaml/Core change, Tether syntax
change, Result Anchor taxonomy change, or P5 Plug functionality.

## References

- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P1_PREPARATION_PROOF.md`
- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P2_GUARD_ADMISSION.md`
- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P4_OUTCOME_DELIVERY.md`
- `docs/J09_DURABLE_REPLAY_DESIGN.md`
- Resolve R0 `8d42e5b061f86b2b2a2c1949c629654968a550ff`
