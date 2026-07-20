# Tethers 0.1 — Semantic Baseline

This SPEC defines the current precise 0.1 language and protocol semantics. The
enduring design principles are governed by `../docs/CONSTITUTION.md`.

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

### 11.1 Request-decoding errors

Errors that occur before reliable request identities are extracted (malformed
JSON, missing protocol/language version, missing structural fields, or
unsupported version values) return a minimal error envelope:

```json
{
  "protocol_version": "0.1",
  "status": "error",
  "error": { "code": "...", "message": "..." }
}
```

No evaluation identifiers, plan, or Trail are included because the engine
has not yet established reliable evaluation context.

### 11.2 Correlated evaluation errors

Once the engine has extracted evaluation, event, and Tether identities and
the Anchor has matched, errors that occur during Condition evaluation return
a correlated error envelope that retains all known identities, `plan: null`,
and the evaluation Trail accumulated so far.

The following Condition-evaluation errors use the correlated envelope:

- **missing Fact** (`missing_fact`): the Fact key is absent from the supplied
  Facts;
- **type error** (`type_error`): the Fact value has an incompatible type for
  the Condition operator (for example, comparing an integer against a string
  with `is`).

In both cases the engine:

- retains `evaluation_id`, `event_id`, `tether_id`, and `tether_version`;
- returns `plan: null`;
- includes reception and Anchor-matched Trail entries;
- includes any Condition Trail entries that precede the error;
- appends exactly one `condition_failed` entry (phase `"evaluation"`, kind
  `"condition_failed"`, outcome `"error"`) whose message preserves the
  original error text.

Other Condition-evaluation error paths are not yet correlated and may still
produce the minimal request-decoding error envelope. The correlated envelope
is extended deliberately, one error path at a time.

### 11.3 Error classification

Malformed source, missing Facts, unknown Capabilities, missing inputs, type
mismatches, and incompatible versions are evaluation errors. A false Condition
is not an error; it produces a successful `not_matched` result with no plan.

## 12. Constitution

The enduring Tethers design principles are recorded in
`../docs/CONSTITUTION.md`. This SPEC remains the authority for the current
precise 0.1 language and protocol semantics.
