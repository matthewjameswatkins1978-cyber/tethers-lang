# Tethers x Resolve01 P4b — End-to-End Guarded Lifecycle & Recovery

Status: implementation record on `codex/tethers-resolve-p4b-lifecycle-recovery`

Tethers starting SHA: `63aa8e21bfa359ad444d8b32b3007b398fa9002e`

Accepted Resolve S3 SHA: `b450305e72815d33357359c6db641d315d6b2975`

## Boundary

P4b proves the complete guarded lifecycle by composing the accepted Tethers
authorities. It adds no new runtime authority and does not contact Resolve from
the lifecycle evidence module.

```text
current Tethers resolution
  -> durable intent
  -> one explicit Resolve admission attempt
  -> durable admission evidence
  -> replay G1
  -> at most one provider invocation
  -> durable classified outcome and Result Anchor
  -> explicit exact Resolve outcome delivery
  -> Delivered or durable Conflict
```

Preparation remains evidence, admission remains a coordination fact, and a
provider outcome remains Tethers truth. Neither a proof nor an admission grants
permission by itself.

## Existing authorities used

P4b uses the real serial guarded boundary in
`host-rust/src/application.rs`, `TestReplayAuthority` for deterministic replay
failure/recovery, `RecordingTrail` for deterministic durable-evidence failure
injection, `P2TestAdapter` for the closed admission seam, the existing counting
executor for provider effects, and `FileResolveOutcomeDeliveryStore` plus
`ResolveOutcomeDeliveryCoordinator` for durable delivery recovery.

There is deliberately no P4b state machine. Recovery behaviour is the
composition already established by replay, durable intent, admission evidence,
outcome classification and the P4 delivery journal.

## Lifecycle and recovery contract

| State or boundary | Required P4b result |
| --- | --- |
| No durable intent | No admission and no provider effect; a fresh run is required. |
| Intent failure | Stop before admission; no provider effect. |
| Intent durable, admission rejected | Record the rejection; no provider effect. |
| Intent durable, admission indeterminate | Preserve indeterminate coordination; no provider effect and no automatic admission retry. |
| Admission accepted, admission evidence fails | Fail closed before G1/provider. |
| Admission evidence durable, pre-G1 failure | No provider effect; the old admission is not silently reused. |
| G1 established, provider truth unknown | Preserve `Uncertain`/manual-resolution semantics; never blindly rerun the provider. |
| Provider outcome durable, delivery pending | Redeliver the exact recorded outcome; never rerun the provider. |
| Delivery indeterminate | Permit only explicit exact redelivery of the durable request. |
| Delivered | Terminal for the journal identity, including after restart. |
| Conflict | Durable coordination failure; do not rewrite the Tethers outcome or retry automatically. |

Provider effects are bounded by the existing replay identity and the P4b tests
assert no more than one call per attempted execution. Rejection and
indeterminacy are checked before the executor. Provider failure and uncertainty
are recorded as distinct Tethers outcomes after exactly one provider attempt.

## Crash-boundary matrix

The packet names twelve consequential boundaries. The focused test suite keeps
one explicit entry for each and exercises the available deterministic seam:

| Boundary | Evidence seam | Expected recovery |
| --- | --- | --- |
| Before durable intent | injected intent Trail failure | no admission/provider |
| After intent / before request | durable-intent path | fresh explicit attempt |
| After request / before response | adapter error | indeterminate, zero provider |
| After admitted / before admission evidence | injected guard Trail failure | fail closed, zero provider |
| After admission evidence / before G1 | replay `Armed` failure | zero provider |
| After G1 / before provider | replay/recovery authority | no blind retry |
| During provider | counting executor failed/uncertain result | one effect, durable distinct outcome |
| After provider / before durable outcome | outcome Trail failure | provider truth preserved, audit failure surfaced |
| After durable outcome / before delivery | pending delivery journal | exact explicit redelivery |
| After delivery request / before response | first delivery adapter failure | indeterminate, then exact retry |
| After recorded / before Delivered persistence | delivery journal reopen tests | durable transition or fail closed |
| After Delivered persistence | terminal journal tests | no further delivery/provider call |

The matrix is a recovery evidence inventory, not a claim that production code
contains unsafe crash hooks at every instruction boundary.

## Together barrier

The existing Together preparation path completes guard admission and durable
admission evidence before stage-B invocation. The P4b guarded serial evidence
also verifies the same ordering at the single-action boundary. Rejected or
indeterminate admission makes zero provider calls. A pre-G1 failure makes zero
provider calls. There is no partial provider effect before the barrier and no
automatic attempt to repair a failed admission.

## Outcome journal and identity

The accepted delivery journal is keyed by the trusted Tethers action reference
and the P1 preparation digest. It persists the classified outcome and bounded
delivery state. `Pending` may move to `Delivered`, or to `Indeterminate` and
back to an explicit retry path; a semantic mismatch becomes durable `Conflict`.
`Delivered` is terminal on reopen. Corrupt or contradictory history fails
closed. Delivery never has a provider handle and cannot invoke a provider.

The lifecycle identity chain is therefore:

```text
ExecutionId
  + current preparation identity
  + guard/admission evidence
  + provider identity
  + durable classified outcome
  + outcome-delivery journal identity
```

Old replay state after restart is not treated as a fresh admission. A current
explicit attempt must establish current Tethers truth and current coordination
evidence.

## Evidence and exclusions

The P4b Rust module is test-only composition evidence. The only production
change is a test-only fault-injection field on `RecordingTrail`; the application
module only wires the test module. No OCaml/Core, syntax, Plan, policy
semantics, replay semantics, provider execution ordering, Result Anchor
taxonomy, Resolve database access, scheduler, generic retry engine, distributed
transaction or new live transport is introduced.

The accepted P3 transport and Resolve S3 contract remain the live boundary for
real cross-system smoke testing. P4b's local delivery-restart test proves the
Tethers-side journal property directly; a two-process Tethers/Resolve restart
claim requires the separately provisioned Resolve S3 service/emulator and is
recorded as evidence only when run in that environment.
