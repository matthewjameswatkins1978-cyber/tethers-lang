# CallPermit architecture — offline checkpoint

```mermaid
flowchart TD
  U[Matthew's bounded delegation] --> T[Tethers 0.7 request and response]
  T --> E[Frozen Authority Envelope + digest]
  E --> C[CallPermit task compiler]
  C --> F[Fake CALL-E module - offline only]
  F --> R[Structured provider evidence]
  R --> X[Deterministic reconciliation against SAME envelope]
  X --> A[Accepted within authority]
  X --> D[Decision Card to Matthew]
  D --> N[New envelope and new atomic call later]
```

The fake module occupies the transport slot only. It is intentionally not a
claim that CALL-E is currently integrated live. Tethers is pre-call authority;
it is not a callback or interceptor inside the conversation.

The call record carries the envelope digest. Therefore a changed authority
cannot retroactively authorise an existing call, and a later envelope produces
a different atomic call identity.
