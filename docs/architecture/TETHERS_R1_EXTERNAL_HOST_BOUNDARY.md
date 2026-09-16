# Tethers R1: External Host Boundary and Resolve Proof

Status: implementation evidence for the R1 external-host boundary

Starting main: `3739d0c06d51569de985053b764af2dc77fc1f2f`

## Decision

The existing public `tethers plan` command is sufficient as the Tethers-owned
side of the smallest external Host seam. R1 therefore adds no protocol, daemon,
FFI, execution framework, Resolve client, or Resolve database access.

The boundary is:

```text
external Host inputs
  -> Tethers `tethers plan --config ... --engine ... --input ...`
  -> versioned `tethers.plan/1` proposal
  -> external Host authority/policy/approval
  -> external Host executor/provider/outcome/ledger
```

Tethers produces a deterministic proposal. It does not grant permission,
invoke a provider, reserve replay, write an execution Trail, or report a
provider outcome.

## Plan contract

The stable command surface is `tethers.cli/1` with `data.schema` equal to
`tethers.plan/1`. The Plan contains the selected evaluation and ordered Action
proposal. The explicit control projection reports:

- `authority.granted: false` and `authority.status: not_requested`;
- `execution.performed: false`;
- `execution.provider_invocations: 0`;
- `execution.policy_evaluated: false`;
- `execution.replay_mutated: false`;
- `execution.trail_execution_entries: 0`.

The existing `fixture.ping` J14 scenario is used as a harmless deterministic
Capability contract. Its provider configuration remains present as trusted
runtime input, but the Plan path uses empty live-provider availability and does
not launch that provider.

## Ownership

Tethers owns Core semantics, source validation, event/Fact evaluation, ordered
Plan production, Action identity, and the versioned Capability contract.

The external Host owns job state, authority, policy, approval, execution
identity, effect identity, provider invocation, outcome, replay/recovery, and
its own durable ledger. A Tethers ActionId is evidence of the proposed Action;
it is not an external execution identity and does not grant authority.

The R1 harness models the external Host with a deliberately tiny local test
executor. Its ALLOW path writes one harmless Resolve-owned marker; ASK before
approval and DENY write none. This is conformance evidence, not production
Resolve code and not a new Host framework.

## Required proof

`tethers-0.1/scripts/test-r1-external-host-boundary.ps1` proves:

1. identical source, event, Facts, trusted Capability projection, and versions
   produce canonical-equivalent Plan evidence;
2. Resolve-only job state changes do not alter the Plan because they are not
   supplied as Tethers Facts;
3. Tethers reports zero provider calls and no authority, policy, replay, or
   execution-Trail effects;
4. the external Host can consume the same Plan under ALLOW, ASK, and DENY;
5. only the external Host's explicit ALLOW path invokes its harmless provider;
6. malformed or tampered Plan evidence is rejected before external execution;
7. the external execution identity remains distinct from Tethers ActionId.

The harness is run after the current reference Host and OCaml engine are
built. It accepts `-OcamlSwitchPath` so the engine can be constructed from an
explicit supported switch; it never selects a historical executable as proof
of current source.

## What R1 does not prove or add

R1 does not implement live Resolve guard admission, transport, durable Resolve
coordination, claims/commitments, recovery protocol, or outcome delivery. It
does not modify OCaml, Core semantics, Tether syntax, provider execution,
replay, Result Anchors, or the historical P0-P4b integration. Those remain
outside this Tethers-first boundary proof.

If a future Resolve implementation needs a field not present in `tethers.plan/1`,
it must identify a host-neutral contract gap and receive a separate design gate;
it must not import the Tethers reference Host's execution lifecycle.
