# Tethers Host Architecture

Status: `R0 RECOVERED AND FROZEN`

This document freezes the ownership boundary recovered by the R0 architecture
audit. It describes the Tethers product and its reference implementation; it
does not replace the precise language contract in `tethers-0.1/SPEC.md`.

## The ownership model

```text
Tethers Core
  deterministic semantics and Plan construction
        |
        v
Capability contract
  stable operation, effects, scope and binding requirements
        |
        v
compatible Host
  policy, approvals, execution lifecycle, recovery and provider invocation
        |
        v
Plug / Provider
  translation from the contract to a real operation
        |
        v
world or application service
```

The governing rule is:

> Tethers defines deterministic consequential behaviour. The Host owns
> execution of that behaviour.

The word `Host` is therefore a role, not a promise that every consumer must
run the Tethers Rust reference host.

## Tethers Core

Tethers Core is the deterministic semantic authority. The OCaml engine parses,
validates and evaluates Tether input, evaluates Conditions over supplied Facts,
constructs an ordered Plan, and produces the Core protocol response and
semantic Trail.

Core does not read live application state, execute Actions, enforce a host
policy, invoke a provider, manage approvals, or own application recovery. It
does not know whether a Capability means a file operation, a Resolve job, a
Lantern memory operation, or something else.

## Capability contracts

A Capability is a stable semantic operation exposed to a Host. Its contract
defines the operation identity and version, input and output schemas, declared
Effects, scope vocabulary, provider-binding requirements, idempotency and
confirmation expectations. A Capability contract is transport-independent: the
same meaning can be implemented by more than one compatible Host or provider.

Schemas describe. Policies authorise. Hosts enforce. Trails record.

## Host contract

The Host is the application-side enforcement and execution owner. It supplies
the live event and trusted capability configuration, resolves the exact
Capability and provider binding, validates invocation arguments and scope,
evaluates local policy, obtains required approval, records durable intent,
invokes the provider, classifies the result, records recovery state and emits
host-side evidence.

The Host also owns the application lifecycle surrounding the Action. This
includes application state, coordination, leases or jobs where the application
has them, crash recovery, and the decision about how a provider result changes
that application's durable state.

Tethers contracts can constrain the Host, but a proposed Plan is not permission
and a provider result is not a substitute for host-owned observation.

## Tethers reference Host

The Rust implementation under `tethers-0.1/host-rust/` is the Tethers reference
Host. It is a first-class Tethers product component and legitimately provides
policy, scope assessment, approvals, replay, durable intent, provider
execution, Trails, Result Anchors, recovery and Plug lifecycle.

The reference Host is the supplied general-purpose implementation of the Host
contract. Its ability to execute Actions is not an architectural error.

It is not, however, a mandatory coordinator for every application that uses
Tethers. Its internal modules and execution state must not be treated as an
external application's job, goal, worker, lease, acceptance or recovery model
unless that application explicitly chooses to delegate those responsibilities.

## External application Host

An external application such as Resolve01 may implement the Host contract
itself. In that shape:

```text
Resolve intent / job lifecycle
        |
        v
Tethers semantics and capability contract
        |
        v
Resolve Host policy / executor / recovery
        |
        v
Resolve-owned or compatible Plug/provider
```

Resolve remains responsible for its own jobs, goals, worker lifecycle,
coordination, application authority, acceptance, consequential execution and
recovery. It may use Tethers Core, capability contracts, or selected generic
machinery through an explicitly defined interface. It must not be forced to
become a client of the Tethers reference Host merely because that Host exists.

This relationship does not make Resolve a Tethers language feature. Resolve
concepts such as Goals, Commitments, Claims, leases, epochs and worker identity
remain outside Core and outside generic Capability meaning.

## Plug

A Plug is a package or binding that connects one or more Capability contracts
to a provider implementation. A Plug may contain manifests, provider payload,
binding metadata and conformance evidence according to the current Plug
contract. It is not permission, policy, an application lifecycle, or a claim
that the Tethers reference Host owns the surrounding application.

A Plug may be consumed by the reference Host, an external Host such as Resolve,
or another compatible Host, subject to that Host's own admission, trust, scope
and policy checks. Cross-host Plug portability is an architectural intent, not a
claim that every current Plug is portable today.

## Provider

A Provider is the concrete effect adapter or transport implementation behind a
Capability binding. It translates a Host-approved, validated invocation into a
real operation and returns a result that the Host validates and classifies.

The provider owns transport details and provider-specific failure mapping within
the Host contract. It does not grant itself permission, replace Host policy,
rewrite Tethers semantics, or own the surrounding application's durable
coordination state.

## Three valid Host shapes

### Shape A: reference Host

```text
small application
      |
      v
Tethers reference Host
      |
      v
Plug / Provider
```

The application delegates the Host responsibilities to the supplied Rust
runtime.

### Shape B: Resolve as Host

```text
Resolve01
  application Host and executor
      |
      v
Tethers Core / Capability contracts
      |
      v
Resolve-owned Plug / Provider
```

Resolve owns the application lifecycle and invokes only the Tethers contracts
it has deliberately adopted. The Tethers reference Host is optional.

### Shape C: future application Host

```text
future application Host
      |
      v
Tethers semantics / Capability contracts
```

An application may embed or independently implement the necessary Host
contract. Tethers remains useful as a deterministic semantic and contract
authority without absorbing that application's state machine.

## Lantern consequence

Lantern Keeper already exposes genuine memory and claim service surfaces. R0
makes no Lantern change and adds no Lantern Plug. The next design question is:

> What Lantern Capability should Tethers define, and how should a Host execute
> that Capability through a Plug?

That is a Capability and Host integration question. It is not a reason for
Tethers Core or the reference Host to become Lantern's execution owner.

## Boundaries retained by R0

- P0-P4b remains preserved historical implementation, frozen for architectural
  recovery. Its original execution-direction assumption is superseded.
- No OCaml/Core, Plan, syntax, replay, provider ordering, outcome taxonomy or
  Resolve/Lantern production behaviour changes are part of R0.
- The reference Host remains standalone and fully legitimate.
- Resolve remains standalone and may be a Host without importing the reference
  Host's application lifecycle.
- Authority is not inferred from a Capability name, Plug description or
  provider output. The selected Host must perform its own current checks.

## Shape D: external executor + Tethers Authority Gate

Status note (R2): this section is appended after the frozen R0 content. The
`R0 RECOVERED AND FROZEN` Status line and every section above are unchanged.
The catalogue above said "three" shapes; R2 adds a fourth without revising
the first three.

R2 adds a delegated-authority Host shape in which the external application
keeps physical execution and application lifecycle while delegating the
authority decision to a Tethers-owned Authority Gate:

```text
external Host
  executor / provider invocation / outcome observation / application lifecycle
        |
        v  tethers.authority/1 (local stdio)
Tethers Authority Gate
  policy evaluation / approval / replay admission / durable intent
        |
        v
Tethers Core / Capability contracts
```

Two delegated variants are legitimate side by side:

- **Full external Host (R0/R1).** The external Host owns policy and approval
  itself and consumes only the side-effect-free `tethers.plan/1` proposal.
  Nothing is delegated; Tethers grants nothing.
- **Delegated-authority Host (R2).** The external Host delegates authority,
  policy, approval, replay admission and durable intent to the Tethers
  Authority Gate over `tethers.authority/1`, while retaining physical
  execution, provider invocation, outcome observation and application
  lifecycle.

The governing rule does not change between the variants: Tethers defines
deterministic consequential behaviour and — where delegated — authorises it;
the Host executes it. The Authority Gate is non-executing. It reports
`provider_invocations: 0` on every surface, never dispatches the Action, and
its PREPARE result always carries `authorizes_dispatch: false`. Delegation
moves the authority decision into Tethers; it does not move execution there.

Shape D specifics:

- **PREPARE uses Tethers Core.** The Gate invokes
  `HostExecutionService::plan_only` with the canonical absolute `--engine`
  path; the caller supplies run-input only and never an authoritative Plan
  (a `plan` field is refused with `prepare.caller_plan_forbidden`).
- **Durable restart reconciliation exists and is typed.** `status` shares one
  canonical Trail×replay reconciliation with late `outcome`: healthy armed
  unresolved commits (`COMMITTED_OUTCOME_INCOMPLETE`), observed terminal
  outcomes (`TERMINAL_KNOWN`), and explicit `recovery_required` disagreements
  (including `terminal_replay_incomplete` — Trail terminal with replay still
  armed — plus `missing_replay_claim`, `replay_claim_without_trail_intent`,
  `terminal_classification_mismatch`, `terminal_outcome_digest_mismatch`,
  `trail_malformed`, `trail_unavailable`, `replay_unavailable`,
  `scan_truncated`).
  `durable_reconciliation.state` is `healthy|recovery_required|unavailable`
  and is distinct from process `healthy`. Late `outcome` refuses whenever the
  durable view is untrustworthy — incomplete or with authorities that
  disagree for the target.
- **Successful results are schema validated.** A `succeeded` result is
  checked against the trusted capability output schema
  (`validation::validate_output`); an invalid result is recorded as `failed`
  with `reason_code: result_validation_failed` and terminal replay `Failed`.

Neither variant makes the other mandatory. A Host that implements its own
policy (Shapes A–C, R0/R1) remains fully valid; a Host that delegates only
the authority seam while keeping its own executor (Shape D) is equally
valid. Authority is still never inferred from a Capability name, Plug
description, provider output, or a proposed Plan.

The R2 contract is frozen in
`docs/architecture/TETHERS_R2_EXTERNAL_AUTHORITY_GATE.md`. The machine
protocol is `docs/tethers.authority.1.md`.
