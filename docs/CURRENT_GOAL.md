# Current Goal

Updated: 2026-07-25

## Goal

Complete the first vertical Tethers 0.2 runtime slice around the verified
manifest store and capability bridge, while preserving the signed-off Tethers
0.1 language and protocol semantics.

## Current Accepted Baseline

The accepted implementation baseline includes:

- verified manifest parsing, digesting, admission, and trusted-store lookup;
- deterministic live capability projection for one Tether Set;
- opaque manifest digest, capability version, and provider identity carried from
  approved projection through planning to dispatch;
- production dispatch requiring a `DispatchReadyAction` created only after
  durable intent preparation;
- configured local stdio MCP provider discovery and admission;
- executor output validation and known-outcome Result Anchors;
- effective host policy outcomes of `allow`, `ask`, `deny`, and `unavailable`;
- fail-closed rejection of stale manifest pins and unestablished structured
  scope.

Latest accepted implementation checkpoint:
`d5ed278d4a2cae5e9ab8a3e1d8700fdcba7ae851`.

J04a effective-policy correction is accepted. J05 and later approval/resume
work remain unauthorised until a separate Red design packet is approved.

## Immediate Definition Of Done

The runtime slice is complete when all of the following are implemented and
independently verified:

1. A configured local provider binding is admitted only through a verified
   manifest and the Trusted Manifest Store.
2. A live capability projection supplies exact capability versions, provider
   identity, and opaque manifest digest to deterministic planning.
3. Every planned bridge Action is resolved through host-owned effective policy
   with exactly one outcome: `allow`, `ask`, `deny`, or `unavailable`.
4. Dispatch is serial, conservative, intent-first, and has no automatic retries.
5. Trail records distinguish authorisation, dispatch, success, failure,
   unavailability, denial, timeout, and uncertain outcome honestly.
6. Known successful and failed outcomes produce standard Result Anchors;
   unattempted Actions do not.
7. AI judgement, when used, is an explicit capability Action whose structured
   result becomes data for a later Anchor. It never runs invisibly inside
   deterministic Condition evaluation.

## Next Authorised Work

Stop after the accepted J04a checkpoint.

The next work must begin with a separate Red design packet for J05 covering the
approval and resume boundary. That design must freeze, before implementation:

- one-shot approval identity and binding;
- approval creation, expiry, invalidation, and consumption;
- resumed Ask precedence;
- denial and cancellation Trail semantics;
- dispatch and Result Anchor behaviour for unattempted Actions;
- replay, stale-plan, and double-consumption failure cases.

No J05 implementation is authorised merely because J04a is complete.

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

- Active prototype tree: `tethers-0.1/`.
- Required automation shell: PowerShell 7 (`pwsh.exe`).
- Keep implementation tasks small, bounded, and owned by one worker.
- Require focused regressions plus the complete relevant verification suite.
- Reserve Lucy/Codex for Red design gates, independent review, difficult local
  diagnosis, and work that genuinely requires direct computer access.
- Do not load the whole project history for routine implementation tasks.

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

## Historical Record

This file is intentionally limited to the present goal, accepted baseline,
frozen boundaries, and next authorised work. Completed milestone narratives,
toolchain setup history, fixture-by-fixture records, and prior verification
results belong in the task queue, decisions, worker notes, dashboard, and Git
history rather than in every agent's default current-context package.
