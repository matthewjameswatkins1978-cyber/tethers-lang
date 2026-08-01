# Task Queue

Updated: 2026-08-01

## Current State

Latest accepted release-candidate checkpoint:

`58affc8c30ddfa9284933a5e38f598dad573f4dd`

Accepted work is complete through J17A3:

- [x] All accepted baseline work through J04a.
- [x] J05 exact one-shot approval.
- [x] J06 honest outcome classification.
- [x] J07 deadline and uncertainty handling.
- [x] J08 uncertain Result Anchor.
- [x] J09 durable replay protection.
- [x] J10 serial Result Anchor continuation.
- [x] J11 event deduplication and causal depth eight.
- [x] J12 minimal runnable configuration.
- [x] J13 public `check`, `run`, and `trail` routes.
- [x] J14 complete local scenario.
- [x] J14C real bounded file move.
- [x] J15 consolidated release matrix.
- [x] J16 clean checkout, builds, restart, and replay proof.
- [x] J17A1 product identity.
- [x] J17A2 release notes.
- [x] J17A3 current-state alignment.

## Active Gate

J17 independent 0.2.0 release sign-off

## Remaining 0.2 Queue

- [ ] Final independent native Windows verification.
- [ ] Evidence-backed release verdict.
- [ ] Fast-forward exact accepted commit to `main`, only after sign-off.
- [ ] Create annotated `v0.2.0` tag, only after sign-off.
- [ ] Verify remote main and tag targets.

## Deferred Beyond 0.2

- Lantern Keeper provider integration until it exposes a small stable capability
  surface.
- Safe retry until idempotency is proved end to end.
- Additional providers and automatic discovery.
- Remote transports, OAuth, and network listeners.
- HQ, package management, marketplace, scheduling, and adapters.
- General plugin or AI-agent framework features.
- Cosmetic rename of `tethers-0.1/` while the local opam switch remains
  path-bound.

## Working Rule

The ten-minute implementation-step limit is a runaway brake, not a deadline.
Stop at a coherent recoverable point and return exact evidence rather than rush,
repeat attempts blindly, or invent missing decisions.

## Gorilla Coding Route

```text
Lucy inspects and compiles
-> Luna handles bounded Green and ordinary Amber work
-> DeepSeek Pro V4 handles thicker middle implementation requiring review
-> Codex Terra High handles Red and machine-required gates
-> Matthew returns concise reports to Lucy
```

J17 is the only remaining release gate. Do not begin feature implementation or
release publication before its evidence-backed verdict.
