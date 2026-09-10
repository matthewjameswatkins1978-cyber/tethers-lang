# AI integration contract

This document describes integration with the **portable Tethers Workbench 0.2.2**, where the caller owns execution and Tethers owns the authority decision.

It does not describe the full Tethers host/runtime, which can execute approved Capabilities through Plugs and record replay/Trail evidence.

## The contract

An agent or workbench may reason however it likes about what it wants to do. Before a governed operation is executed, the integration constructs a canonical Tethers request, invokes the workbench, validates the response schema, and only then interprets the result.

```text
agent intent
    -> canonical request
    -> Tethers Workbench
    -> validate response
    -> ALLOW / ASK / DENY
    -> caller acts, asks, or stops
```

No wrapper may implement a second policy engine.

That rule is important. If a wrapper quietly says “Tethers denied this, but my own fallback thinks it is probably safe,” the integration has destroyed the authority boundary it was supposed to provide.

## Safe caller behaviour

The caller should use the following logic:

1. Construct a request that matches the documented schema.
2. Resolve resource paths and containment before requesting authority where required by the integration.
3. Invoke the workbench as a subprocess.
4. Validate the complete response schema and known decision vocabulary.
5. Treat exact `ALLOW` as permission to continue with the caller-owned operation.
6. Treat `ASK` as a requirement for explicit human authority before execution.
7. Treat `DENY` as a stop.
8. Treat malformed output, unknown decisions, invocation failure, timeout, missing policy state, or any other operational ambiguity as a stop.

> **An error is never an implicit ALLOW.**

The decision exit codes are intentionally separate from invocation/configuration failures so shell and CI integrations do not need to guess whether a failed process means permission.

## Decision order

The workbench decision order is fixed:

1. request, schema, and action validation;
2. manifest and policy validation;
3. manifest and scope narrowing;
4. global hard denies;
5. first matching policy rule;
6. policy default.

Conceptually the authority layers are:

```text
GLOBAL HARD -> WORKBENCH DEFAULT -> PROJECT -> JOB
```

Later layers may narrow authority. They do not broaden a hard deny.

`ASK` requires a real human approval step. It is not a softer spelling of `ALLOW`.

## Provenance, explanation, and audit

`decision_id` and `policy_sha256` provide deterministic provenance.

`--trace` adds rule and condition outcomes without echoing condition values.

`--audit` adds an optional JSONL decision record and fails closed if it cannot append.

These surfaces are useful because the agent should not have to reconstruct why a decision occurred from its own conversational memory.

## Translators are not executors

The official translators under `plugs/` cover common host request surfaces. They construct Tethers requests only.

They do not:

- execute the requested operation;
- grant permission;
- widen scope;
- replace policy evaluation;
- prove that an external effect occurred.

Path resolution and symlink/junction containment are performed before evaluation where the integration contract requires them. A host that cannot resolve a path reliably must deny the operation rather than guessing.

## When to use the full Tethers host instead

Use the portable workbench when your existing caller already owns execution and only needs a small deterministic authority boundary.

Use the full Tethers host/runtime when you need Tethers to own more of the consequential path, including deterministic Tether planning, trusted Capability resolution, Plug lifecycle, durable intent, replay protection, provider execution, Result Anchors, bounded Together concurrency, and causal Trail evidence.

The two surfaces are related, but they solve different-sized problems.

For the whole product story, see [`../../README.md`](../../README.md). For exact portable commands and exit codes, see [`docs/CLI.md`](docs/CLI.md).
