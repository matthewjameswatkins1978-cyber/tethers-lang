# J06 Deadline And Outcome Design

Status: authoritative implementation design  
Date: 2026-07-25  
Base: `main` at `1f984ff3c89c66b5580e8b6e7936b8e41d9db93d`

## Purpose

J06 makes execution timing and provider outcomes truthful.

It defines when an Action becomes attempted, when a deadline begins, how timeout
and transport ambiguity are classified, what becomes durable, and when a Result
Anchor may exist.

This design is the sole J06 implementation authority. The earlier candidate and
the immutable safety branch are reference only.

## Non-goals

J06 does not add automatic retry, compensation, durable approval persistence,
new planner semantics, new manifest fields, new MCP protocol messages, or J07
behaviour.

## Core invariants

1. Durable intent exists before the execution deadline begins.
2. The deadline uses a monotonic clock only.
3. Tests inject a controllable monotonic clock.
4. Before provider invocation begins, the Action is unattempted.
5. Once invocation may have begun, loss of trustworthy final evidence is
   `uncertain`, never guessed `failed`.
6. Provider-declared failure is a known failed outcome.
7. Provider success with schema-invalid output is a known failed outcome.
8. A known provider outcome remains known even if later audit persistence fails.
9. No automatic retry or implicit compensation is authorised.
10. A consumed J05 approval is never restored.
11. Unattempted Actions produce no standard Result Anchor.
12. Durable reasons and Result Anchor reasons cross an explicit redaction
    boundary.

## Clock model

The host owns a `MonotonicClock` abstraction.

Required operations:

- obtain a monotonic instant;
- compute elapsed duration between two monotonic instants;
- compare elapsed duration with the execution deadline.

Production uses a monotonic source such as `std::time::Instant`. Wall-clock
`SystemTime` must not participate in deadline decisions.

A separate wall-clock source may continue to supply `occurred_at` timestamps for
event envelopes. Wall-clock timestamps are descriptive only and must never
change execution classification.

Tests use a deterministic injected clock that advances only when directed by the
test.

## Deadline start

The execution deadline starts immediately after durable intent has been written
and confirmed by the existing dispatch boundary, and before provider invocation
begins.

The deadline must not include:

- planning;
- resolution;
- policy evaluation;
- J05 approval waiting;
- approval consumption;
- failed intent persistence.

The implementation must capture the monotonic start instant only after
`prepare_and_record` has successfully returned dispatch authority.

## Execution state vocabulary

J06 uses these host-level execution classes:

- `Unattempted`
- `Succeeded`
- `Failed`
- `Uncertain`

`Unattempted` means the provider invocation boundary was not crossed.

`Succeeded` means the host obtained a trusted provider success response and the
output passed the trusted output schema.

`Failed` means the host obtained trusted evidence of failure, including:

- an explicit provider-declared error;
- a trusted completed provider response whose declared success output fails the
  trusted output schema.

`Uncertain` means provider invocation may have occurred but the host cannot
establish a trusted final outcome.

Examples include:

- deadline expires while invocation is in flight;
- provider process or connection disappears after invocation may have started;
- response framing is truncated or malformed after invocation;
- protocol interruption prevents proving whether the provider completed;
- the provider returns no trustworthy final response before the deadline.

## Invocation boundary

The provider invocation boundary is crossed immediately before the host hands a
valid `DispatchReadyAction` to the provider adapter or executor operation that
may cause external effects.

The host must record this boundary in volatile orchestration state before making
the call. It need not add a new durable pre-call record beyond existing durable
intent.

Classification rules:

- deadline already expired before this boundary: `Unattempted`;
- failure to construct or enter the provider call before this boundary:
  `Unattempted`;
- any ambiguity after this boundary: `Uncertain` unless trusted final evidence
  proves `Succeeded` or `Failed`.

## Deadline cases

### Expired before invocation

If elapsed time reaches the deadline after durable intent but before the
invocation boundary, execution is `Unattempted`.

Required effects:

- no provider call;
- no execution outcome record claiming success, failure, or uncertainty;
- no standard Result Anchor;
- a durable audit entry may record a redacted `deadline_before_invocation`
  reason.

### Expired during invocation

If the deadline expires after invocation may have begun and no trusted final
response has been established, execution is `Uncertain`.

The host must not relabel this as provider failure.

### Response arrives at or after deadline

The host must define one deterministic acceptance rule:

- a trusted response observed before the monotonic deadline is classified from
  that response;
- a response first observed after the deadline is not accepted as the final J06
  outcome and the Action is `Uncertain`.

Tests must control the clock at the observation boundary.

J06 does not require forcibly killing an in-process provider. Adapters may use
native timeout mechanisms, but classification remains governed by the host's
monotonic deadline and invocation boundary.

## Provider and transport classifications

- explicit provider error response: `Failed`;
- trusted provider success plus valid output: `Succeeded`;
- trusted provider success plus schema-invalid output: `Failed` with
  `result_validation_failed`;
- process death before invocation: `Unattempted`;
- process death after invocation may have begun: `Uncertain`;
- malformed or truncated response after invocation: `Uncertain`;
- protocol interruption after invocation: `Uncertain`;
- adapter setup failure before invocation: `Unattempted`;
- unavailable provider discovered during fresh resolution before intent:
  existing unattempted behaviour;
- unavailable provider discovered after intent but before invocation:
  `Unattempted` with audit evidence and no Result Anchor.

## Durable ordering

The required order is:

1. fresh resolution, policy, binding, schema, scope, and J05 approval checks;
2. J05 approval consume when applicable;
3. durable intent;
4. monotonic deadline start;
5. provider invocation boundary;
6. trusted outcome classification;
7. durable outcome persistence for attempted known or uncertain outcomes;
8. standard Result Anchor creation for attempted outcomes only;
9. response Trail presentation entries.

A standard Result Anchor must never precede durable outcome persistence.

If durable outcome persistence fails after a known provider result:

- retain the known in-memory classification;
- report a separate audit failure;
- do not downgrade to `Uncertain`;
- do not authorise retry;
- do not create a standard Result Anchor unless the design's required durable
  outcome prerequisite has succeeded.

If durable uncertain-outcome persistence fails, retain `Uncertain` in memory,
report audit failure, and do not authorise retry or create a standard Result
Anchor.

## Result Anchors

J06 extends the existing Result Anchor vocabulary with an uncertain outcome.

Allowed event names:

- `capability.succeeded`
- `capability.failed`
- `capability.uncertain`

A standard Result Anchor exists only for an attempted Action whose durable
outcome record succeeded.

No standard Result Anchor exists for `Unattempted`.

Failure and uncertain reasons must use stable redacted reason codes. Raw process
stderr, transport payloads, paths, credentials, tokens, arguments, stack traces,
or provider-private messages must not cross into durable outcome records or
Result Anchors.

## Redaction boundary

The host owns a pure classification-and-redaction function that converts
internal diagnostics into:

- stable public reason code;
- bounded safe message;
- optional non-sensitive structured metadata explicitly approved by the design.

Required stable codes include at least:

- `provider_error`
- `result_validation_failed`
- `deadline_exceeded`
- `provider_outcome_uncertain`
- `provider_process_lost`
- `provider_protocol_interrupted`
- `provider_response_invalid`
- `audit_write_failed`

Internal diagnostics may remain in process-local logs or test assertions, but
must not be copied verbatim into durable Trail outcome entries or Result Anchor
facts.

## Restart and crash semantics

J06 adds no recovery execution and no automatic retry.

After restart, durable records are evidence only. The host must not infer that an
Action is safe to repeat merely because no final Result Anchor exists.

Interpretation:

- durable intent with no durable final outcome means the previous attempt may
  have crossed invocation and requires later recovery policy, outside J06;
- durable uncertain outcome remains uncertain;
- durable known outcome remains known;
- consumed J05 approval remains consumed and is never reconstructed.

J06 does not implement the future reconciliation workflow.

## Interaction with J05

J05 remains authoritative for approval identity and consumption.

For an approved Ask:

1. resume performs all J05 fresh checks;
2. exact approval is atomically consumed;
3. durable intent is written;
4. J06 deadline starts;
5. provider invocation may begin.

Any later timeout, uncertain outcome, provider failure, audit failure, or crash
must not restore the approval.

## Separation from J07

J07 implementation is forbidden in J06.

Do not transplant partial safety-branch runtime code. In particular, do not use
wall-clock `SystemTime` for deadlines, do not leave an injected test clock
unused, and do not persist raw uncertain reasons.

J07 must begin later from accepted J06 on current `main`.

## Verification matrix

Each numbered case requires an individually identifiable test or explicit
one-to-one mapping.

1. deadline start occurs after durable intent confirmation.
2. planning time does not consume execution deadline.
3. approval waiting does not consume execution deadline.
4. monotonic production clock is used for deadline decisions.
5. deterministic test clock controls elapsed time.
6. deadline before invocation produces `Unattempted`.
7. case 6 makes zero provider calls.
8. case 6 creates no durable attempted outcome.
9. case 6 creates no standard Result Anchor.
10. provider success before deadline with valid output is `Succeeded`.
11. explicit provider error before deadline is known `Failed`.
12. provider success with schema-invalid output is known `Failed`.
13. deadline during invocation is `Uncertain`.
14. process loss after invocation is `Uncertain`.
15. malformed response after invocation is `Uncertain`.
16. truncated response after invocation is `Uncertain`.
17. protocol interruption after invocation is `Uncertain`.
18. no trustworthy response before deadline is `Uncertain`.
19. response observed after deadline remains `Uncertain`.
20. adapter setup failure before invocation is `Unattempted`.
21. unavailable provider before invocation is `Unattempted`.
22. all unattempted cases make zero provider calls where applicable.
23. all unattempted cases create no standard Result Anchor.
24. known success outcome is durably recorded before Result Anchor.
25. known failure outcome is durably recorded before Result Anchor.
26. uncertain outcome is durably recorded before uncertain Result Anchor.
27. successful outcome write failure preserves known success in memory.
28. failed outcome write failure preserves known failure in memory.
29. uncertain outcome write failure preserves uncertainty in memory.
30. outcome-write failure creates no standard Result Anchor.
31. outcome-write failure never authorises retry.
32. deadline uncertainty never becomes guessed provider failure.
33. redaction removes credentials and tokens.
34. redaction removes raw arguments and provider-private payloads.
35. redaction emits stable bounded reason codes.
36. raw diagnostics may remain non-durable only.
37. J05 approval remains consumed after success.
38. J05 approval remains consumed after known failure.
39. J05 approval remains consumed after uncertainty.
40. J05 approval remains consumed after intent or outcome audit failure.
41. restart with intent but no final outcome does not authorise retry.
42. durable known outcome remains known across reconstruction.
43. durable uncertain outcome remains uncertain across reconstruction.
44. no automatic retry occurs for any J06 outcome.
45. no implicit compensation occurs.
46. existing Deny, Ask, Unavailable, identity mismatch, and intent-write failure
    regressions remain unattempted and Anchor-free.
47. existing structured-scope demo remains fail-closed.
48. full Rust, fixture, engine, MCP transcript, denial, execution-failure, demo,
    OCaml build, packet checker, formatting, and diff checks pass.

## Acceptance gate

J06 is acceptable only when the implementation matches this document, all 48
cases have identifiable evidence, and independent Red review finds no route that
turns ambiguity into retry authority or false certainty.
