# ADR: Host Execution Ownership

Status: `ACCEPTED BY TETHERS R0`

Date: 2026-09-16

## Context

Tethers has two related but distinct concerns: deterministic behaviour and the
application lifecycle that turns an admitted Action into an external effect.
The repository also contains a substantial Rust reference Host, while external
applications such as Resolve01 have their own jobs, coordination and recovery.

The R0 audit was required because the completed P0-P4b Resolve line had been
read as though the Tethers reference Host had to remain the execution owner for
Resolve. That interpretation conflicts with the intended Host role and makes a
consumer application depend on another application's lifecycle.

## Original Tethers model

Tethers Core parses, validates, evaluates and plans. Capability contracts define
stable operations. A Host resolves current bindings, applies policy and
approval, records intent, executes through a Plug/provider, classifies outcomes
and owns recovery. The supplied Rust reference Host is a complete implementation
of that Host contract.

## How the reference-host interpretation drifted

Because the reference Host implements policy, replay, durable intent, provider
execution, Trails, Result Anchors and recovery, later integration work treated
its internal execution boundary as the universal boundary. The implementation
was then extended around a reference-host lifecycle rather than distinguishing
generic Tethers machinery from the application Host role.

This was an architectural reading error, not evidence that the reference Host
cannot execute or that its accepted implementation history is invalid.

## P0 assumption

P0-P4b assumed a Resolve Guard Adapter inside the Tethers Rust Host would
coordinate Resolve admission immediately before the reference Host's existing
provider execution, followed by delivery of the Tethers outcome to Resolve.
That contract was internally explicit and its implementation may contain
reusable proof, identity, validation, transport, replay, outcome and recovery
machinery.

Its implicit premise that the Tethers reference Host is the required execution
owner for an external Resolve application is superseded.

## Consequences of that assumption

- Resolve was modelled as a coordinator around a Tethers-owned execution
  lifecycle instead of as an application Host.
- Resolve-specific coordination concepts were placed beside reference-host
  execution seams.
- Reusing generic Tethers evidence in Resolve would require distinguishing it
  from reference-host lifecycle state.
- The Tethers and Resolve repositories risked acquiring competing ideas about
  who owns application authority, recovery and consequential execution.

## Corrected decision

Tethers Core owns deterministic semantics and Plan construction. Capability
contracts own stable operation meaning. The Host owns application enforcement and
execution. The Tethers Rust reference Host remains a first-class, supported
Host implementation, but it is not mandatory architecture for an external
application.

Resolve01 may be the Host for its own use of Tethers. Resolve owns its job,
goal, worker, coordination, application authority, acceptance, consequential
execution and recovery lifecycle. It may consume Tethers Core and capability
contracts, and may reuse explicitly generic machinery through a later bounded
interface. It does not become a client of the Tethers reference Host by default.

The error was not that the Tethers reference host can execute. It can and
should.

The error was treating that particular Host implementation as the mandatory
execution owner for an external application that already owns its execution
lifecycle.

## Treatment of existing P0-P4b work

P0-P4b is retained as:

```text
HISTORICAL IMPLEMENTATION
FROZEN FOR ARCHITECTURE RECOVERY
```

Its original execution-direction contract is superseded. No component is
removed, reverted, squashed, hidden or rewritten by R0. Each relevant area is
classified in `docs/recovery/TETHERS_RESOLVE_RECLASSIFICATION.md`.

The existing work is a mixture of reference-host implementation, reusable
generic evidence and integration-specific material. Future packets must make a
fresh design decision about the target Host before reusing it.

## Compatibility implications

Existing standalone Tethers users and the Tethers reference Host retain their
current architecture and behaviour. Existing P0-P4b history and documents
remain available as historical records; they are not silently presented as the
current universal Resolve architecture.

Future external Hosts need a documented contract for the exact Tethers
semantics or generic evidence they consume. A Plug is portable only where its
Capability contract, binding and Host interface are compatible. R0 does not
claim cross-host Plug portability or change any current wire protocol.

## Future integration rule

Start with the smallest interface by which the external application Host
consumes useful Tethers machinery. Keep application lifecycle and consequential
execution in that Host. Keep Tethers Core application-agnostic. Keep Plugs and
providers behind Capability contracts. Do not move Resolve Goals,
Commitments, Claims, leases, epochs or worker identity into Core or generic
Tethers contracts.
