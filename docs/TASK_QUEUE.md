# Task Queue

Updated: 2026-08-01

## Current State

Tethers 0.2.0 is published at
`b5546411661dcbcb53e1cf2538eaec594c6f76f2` with annotated tag `v0.2.0`.
Language semantics remain `0.1`.

J17 is complete. Its bounded release history includes:

- J17-V3 runner matcher failure.
- J17-V4 unsupported Rust-total claim.
- J17-V5 successful final reconciliation.
- Final Lucy sign-off: `SIGNED OFF FOR 0.2.0`.
- J17-P1 narrow fetch-ref preflight failure.
- J17-P2 successful publication.

All 17 ROAD_TO_0_2 acceptance claims were proven. Large evidence logs remain in
the retained release evidence and worker records.

## J18 Universal Plug Architecture And Plug Kit

J18B, J18C, J18D, and J18E are accepted. J18F is active and design-only. Plug
functionality remains unauthorised.

- [x] J18A: post-release reset.
- [x] J18B: Universal Plug Architecture (accepted boundary; J18H freeze gate).
- [x] J18C: Tethers Socket v1 protocol and MCP stdio binding (accepted).
- [x] J18D: `.tetherplug` package format (accepted).
- [x] J18E: capability classes, effects, and scopes (accepted at `eb6548c`).
- [~] J18F: lifecycle, outcomes, events, and conformance (pending Lucy review).
- [ ] J18G: security and sandbox threat model.
- [ ] J18H: paper validation against representative integrations.
- [ ] J18I: first Plug Kit implementation roadmap.

The first J18 phase is architecture and paper validation. Plugs remain outside
Tethers Core, which remains application-agnostic. J18F is design-only; J18G is
next only after Lucy accepts J18F.

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
