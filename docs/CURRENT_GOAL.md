# Current Goal

Updated: 2026-08-01

## Goal

Independently sign off the completed Tethers 0.2.0 release candidate and, only
after a `SIGNED OFF FOR 0.2.0` verdict, publish the exact accepted commit to
`main` and create the annotated `v0.2.0` tag.

## Current Accepted Baseline

The accepted baseline through the release candidate includes:

- J05 exact one-shot approval.
- J06-J09 honest outcomes, uncertainty, and durable replay.
- J10-J11 serial Result Anchor continuation, event deduplication, and depth eight.
- J12-J14C public runtime routes and the real bounded file move.
- J15 consolidated release matrix.
- J16 clean checkout, builds, restart, and replay proof.
- J16D-F1 deterministic Ctrl+C classification repair.
- J17A1 product identity `0.2.0`.
- J17A2 release-candidate notes.

Latest accepted candidate checkpoint:
`58affc8c30ddfa9284933a5e38f598dad573f4dd`

## Immediate Definition Of Done

1. Verify the exact candidate from the accepted native Windows checkout.
2. Map every ROAD_TO_0_2 release-acceptance claim to evidence.
3. Return exactly `SIGNED OFF FOR 0.2.0` or `NOT SIGNED OFF`.
4. Only after sign-off, fast-forward `main` to the exact accepted commit.
5. Create annotated tag `v0.2.0` at that exact commit.
6. Verify remote main and tag targets.

## Next Authorised Work

J17 independent 0.2.0 release sign-off only.

- No further feature implementation is authorised before J17.
- Product identity is `0.2.0`.
- Language semantics remain `0.1`.
- Release notes remain a candidate until J17.
- Main and tags are still untouched.

## Frozen Boundaries

- Tethers Core remains deterministic and application-agnostic.
- Tethers Core has no built-in knowledge of Lantern Keeper, MCP business
  meanings, AI, memory, or provider-specific effects.
- Capability schemas describe; host policy authorises; hosts enforce; Trails
  record.
- The planner never inspects or trusts complete manifests.
- Manifest and provider pins are checked against current trusted state before
  dispatch.
- Structured scope without a host/binding-owned assessment fails closed.
- Do not infer argument-to-resource mappings without an approved adapter or
  binding contract.
- No hidden AI judgement inside deterministic Condition evaluation.
- No automatic retries until idempotency is proved end to end.
- The signed-off Tethers 0.1 syntax and semantics remain unchanged unless an
  explicit language-design gate authorises a change.

## Active Development Posture

Current operating mode: **Gorilla Coding**.

- Lucy: architecture, task compilation, and independent review.
- Luna on OpenCode: bounded Green and ordinary Amber implementation.
- DeepSeek Pro V4: thicker middle implementation requiring review.
- Codex Terra High: Red work, machine failures, and release gates.
- Matthew: product authority and report-routing bridge.
- Active prototype tree: `tethers-0.1/`.
- Required automation shell: PowerShell 7 (`pwsh.exe`).

## Authoritative References

- Enduring design principles: `docs/CONSTITUTION.md`
- Current 0.1 language and protocol semantics: `tethers-0.1/SPEC.md`
- Joint target architecture:
  `docs/architecture/TETHERS_LANTERN_KEEPER_CANONICAL_ARCHITECTURE.md`
- Capability bridge and host trust contract: `docs/CAPABILITY_BRIDGE.md`
- Accepted design decisions: `docs/DECISIONS.md`
- Current task state: `docs/CURRENT_CLINE_TASK.md`
- Short Matthew-facing status: `docs/PROJECT_DASHBOARD.md`
- Dependency-ordered programme: `docs/ROAD_TO_0_2.md`
- Detailed queue and completed milestones: `docs/TASK_QUEUE.md`
- Evidence and reviews: `docs/worker-notes/`
