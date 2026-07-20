# Tethers 0.1 — Semantic Baseline

## 1. Definition

Tethers is a deterministic planner. It accepts a Tether, an event, an immutable
fact snapshot, and capability schemas. It returns either a validated action plan
or an explanation of why no plan was produced.

Tethers does not execute actions, grant permissions, query live state, or store
application data.

## 2. Canonical vocabulary

| Term | Meaning |
| --- | --- |
| Tether | One behavioural rule |
| Tether Set | An installable collection of Tethers |
| Anchor | The event that wakes a Tether |
| Fact | Immutable input available to Conditions |
| Condition | Deterministic test over a Fact |
| Action | A requested capability invocation |
| Capability | A typed operation exposed by a host or adapter |
| Effect | An external consequence declared by a capability |
| Plan | Ordered Actions proposed by Tethers |
| Trail | Causal record of evaluation, authorisation, and execution |
| Host | Application supplying inputs and enforcing policy |
| Adapter | Component exposing another system as Capabilities |
| HQ | Later visual editor, tester, and Trail inspector |

## 3. Determinism

The deterministic input is:

```text
(language version,
 Tether source and version,
 event envelope,
 fact snapshot,
 capability schemas)
```

The same input must produce byte-equivalent semantic output. Object key order
is not semantically meaningful.

The engine must not read the clock, environment, filesystem, network, random
source, host database, or any undeclared state. If time or another changing
value matters, the host supplies it in the event or Fact snapshot.

## 4. Evaluation lifecycle

1. Parse the Tether source.
2. Validate its language version and structure.
3. Compare the Anchor with the incoming event name.
4. Resolve every Condition from the supplied Facts.
5. Stop on the first false Condition.
6. Validate each Action against an available Capability schema.
7. Resolve constant and `anchor.*` Action arguments.
8. Return an ordered plan and required Effects.

Conditions cannot inspect results that do not yet exist. Capability results are
reported by the host and may be emitted as new events for another Tether.

## 5. Minimal grammar

```text
tether      := 'tether' STRING NEWLINE anchor conditions actions
anchor      := 'anchor' NEWLINE INDENT NAME NEWLINE
conditions  := 'when' NEWLINE condition (NEWLINE condition)*
condition   := INDENT ['and'] PATH OPERATOR VALUE
actions     := 'do' NEWLINE action (NEWLINE action)*
action      := INDENT NAME NEWLINE argument+
argument    := INDENT INDENT NAME ':' VALUE_OR_PATH
```

Supported 0.1 operators:

- `is`
- `contains`
- `greater_than`
- `greater_than_or_equal`

Supported values:

- quoted strings
- integers
- `true` and `false`
- dotted references beginning with `anchor.`

No loops, arithmetic, user functions, mutation, implicit I/O, or hidden
coercion are permitted.

## 6. Action semantics

Actions are ordered. The host executes them sequentially and stops on the first
failure. Version 0.1 does not pass one Action's result directly into another
Action; result-dependent behaviour must be expressed through a subsequent
event and Tether.

Each planned Action has:

- a stable position-derived `action_id`
- an idempotency key derived from the evaluation and action IDs
- resolved arguments
- declared Effects copied from its Capability schema

A plan is a request, not permission.

## 7. Capabilities and policy

Capability schemas describe names, versions, inputs, outputs, and Effects.
They do not grant authority.

```text
Schemas describe.
Policies authorise.
Hosts enforce.
Trails record.
```

The reference host authorises a plan only when every required Effect is in its
configured allow-list. Production hosts must additionally enforce resource
scope at the execution boundary.

## 8. Trail ownership

Tethers writes deterministic reception and evaluation entries. It does not put
wall-clock timestamps in those entries.

The host appends authorisation and execution entries, including timestamps,
results, failures, and resulting event IDs. Both parts share `evaluation_id`
and `plan_id`.

## 9. Identity and replay

- `event_id` identifies a host event.
- `evaluation_id` identifies evaluation of one Tether version against one event.
- `plan_id` identifies the resulting plan.
- `action_id` identifies a position in that plan.
- `idempotency_key` is `evaluation_id/action_id`.

Hosts must persist successful idempotency keys before acknowledging completion
for externally significant Actions.

## 10. Versioning

The request declares a protocol version and language version. Tethers and
Capabilities declare versions. An engine must reject incompatible major
versions rather than guess.

## 11. Error policy

Malformed source, missing Facts, unknown Capabilities, missing inputs, type
mismatches, and incompatible versions are evaluation errors. A false Condition
is not an error; it produces a successful `not_matched` result with no plan.

## 12. Project constitution

1. Tethers plans; hosts execute.
2. Applications expose Capabilities; Core contains no application-specific logic.
3. Evaluation uses immutable supplied Facts, never hidden live state.
4. The same complete input produces the same plan.
5. AI is an explicit Capability, never invisible authority.
6. Schemas describe, policies authorise, and hosts enforce.
7. Every decision and Effect belongs to a causal Trail.
8. Actions are safely identifiable for replay and deduplication.
9. The language remains smaller than a general programming language.
10. Behaviour that need not be inspected, changed, disabled, or audited belongs in ordinary code.
