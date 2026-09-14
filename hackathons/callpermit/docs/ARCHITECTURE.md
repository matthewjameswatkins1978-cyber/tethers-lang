# CallPermit architecture

```mermaid
flowchart TD
  U[Matthew's bounded delegation] --> T[Tethers 0.7 request and response]
  T --> E[Frozen Authority Envelope + digest]
  E --> C[CallPermit task compiler]
  C --> F[CALL-E 0.7 SDK or deterministic fake]
  F --> R[Structured provider evidence]
  R --> X[Deterministic reconciliation against SAME envelope]
  X --> A[Accepted within authority]
  X --> D[Decision Card to Matthew]
  D --> N[New envelope and new atomic call later]
```

The fake and official SDK modules occupy the same narrow transport slot. The
official adapter is the only module that imports `@call-e/calle`; it is an
effect boundary and is never used by deterministic acceptance. Tethers is
pre-call authority; it is not a callback or interceptor inside the conversation.

The call record carries the envelope digest. Therefore a changed authority
cannot retroactively authorise an existing call, and a later envelope produces
a different atomic call identity.
