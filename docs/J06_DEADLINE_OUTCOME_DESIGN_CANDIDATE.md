# J06 Deadline And Outcome Design Candidate

Status: reviewed candidate, not yet implementation authority  
Date: 2026-07-25

## Provenance

The immutable safety branch
`safety/preserve-local-main-20260725` at
`f74999aba9135f0493cf28693ba6444c22388294` contains a J06 design and independent
audit that Codex judged internally coherent and independent of the incomplete
J07 code.

This document preserves the accepted design direction without importing the
safety branch's partial runtime implementation. A later Lucy design gate must
turn it into an authoritative J06 task after J05 is accepted.

## Candidate Contract

- The execution deadline starts only after durable intent exists.
- Failure before provider invocation is unattempted and produces no standard
  Result Anchor.
- Once provider invocation may have occurred, inability to establish a trusted
  final response is `uncertain`, not guessed `failed`.
- Provider-declared failure and schema-invalid successful output are known
  `failed` outcomes.
- A known provider outcome followed by an audit-write failure remains a known
  outcome with a separate audit failure. It does not authorise retry.
- Deadline timing must use a monotonic clock.
- Deterministic tests must inject a controllable clock.
- Durable outcome and Result Anchor reasons cross an explicit redaction boundary.
- No automatic retry is introduced.

## Separation From J07

The partial J07-style changes on the safety branch are reference only and must
not be transplanted. They use wall-clock `SystemTime`, leave the deterministic
test clock unused, pass raw uncertain reasons across durable boundaries, and
contain syntax corruption.

A future J07 implementation must start fresh from the accepted J06 contract on
current `main`.
