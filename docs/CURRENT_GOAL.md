# Current Goal

Updated: 2026-08-01

## Goal

Roadmap the first Tethers Plug Kit without changing Tethers 0.1 language
semantics or weakening the released 0.2 runtime boundaries.

## Accepted Baseline

Tethers 0.2.0 is the accepted and published baseline. J17 is complete; `main`
and the annotated `v0.2.0` tag point to the signed-off commit
`b5546411661dcbcb53e1cf2538eaec594c6f76f2`.

J18A is complete. J18B through J18H are accepted and the Universal Plug
architecture is frozen at `a5fd63593a9d9acd397030ecd2e27b4f318c87fd`. J18I is
active and pending Lucy roadmap review. Implementation remains unauthorised; the
first implementation packet follows only after J18I acceptance.

## J18 Boundaries

- Tethers Core remains deterministic and application-agnostic.
- Plugs remain outside the core.
- Permissions, credentials, canonical outcomes, and Trails remain host-owned.
- The signed-off Tethers 0.1 syntax and semantics remain unchanged.
- The 0.2 runtime boundaries remain fail-closed and intent-first.

## Frozen Boundaries

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
