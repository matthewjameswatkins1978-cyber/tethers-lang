# Current Goal

Updated: 2026-08-03

## Goal

Complete the first public Tethers Plug lifecycle command surface over the
accepted Plug Kit backend without changing Tethers 0.1 language semantics,
putting Plugs into Tethers Core, or weakening host-owned trust, permissions,
conformance, enablement, outcome, replay, Anchor, and Trail boundaries.

## Accepted Baseline

Tethers 0.2.0 remains the accepted and published baseline. The annotated
`v0.2.0` tag remains at
`b5546411661dcbcb53e1cf2538eaec594c6f76f2`; language semantics remain 0.1.

The Universal Plug architecture remains frozen at
`a5fd63593a9d9acd397030ecd2e27b4f318c87fd`. The implementation now on `main`
includes the reusable Socket/application seam, package inspection and
quarantine candidate identity, trust and supervised launch evidence,
conformance and installed-disabled state, File Tools, durable local Anchor
admission, the bounded PDF contract/provider/package, operational scope,
conformance, and installed PDF execution.

J24A is accepted at `13f6a3caffa00904f6357c7975a8a0937a6c2d5c`. It exposes the
existing hostile-data package inspector through the strictly read-only public
command `plug inspect --package <PATH>`.

## Active Increment

J24B adds the next smallest public lifecycle surface: a strictly read-only
`plug list --state-root <ABSOLUTE_PATH>` command. It reports validated installed
identity and current enablement state without creating directories, changing
records, launching providers, or granting availability.

Mutating lifecycle commands remain unauthorised until their own reviewed
packets: candidate creation/quarantine, conformance, approval,
installed-disabled publication, enablement, disablement, and removal.

## Frozen Boundaries

- Tethers Core remains deterministic and application-agnostic.
- Plugs remain outside the language Core.
- Capability schemas describe; host policy authorises; hosts enforce; Trails
  record.
- Package inspection, installed records, and enablement histories retain their
  existing sole authorities; CLI adapters must not duplicate their parsers or
  validation rules.
- Installation approval is distinct from runtime Ask approval.
- Installed state remains `present_disabled`; only an exact current enablement
  record can create operational availability.
- Structured scope without a host/binding-owned assessment fails closed.
- No hidden AI judgement enters deterministic Condition evaluation.
- No automatic retry exists without end-to-end idempotency proof.
- Supervised provider execution remains explicitly non-isolated and must not be
  described as hostile-code safe.
- No public registry, download/update path, network listener, OAuth, arbitrary
  third-party enablement, or Tether language change is part of the first Plug
  Kit.

## Active Development Posture

Current operating mode: **Gorilla Coding**.

- Lucy: architecture, task compilation, and independent review.
- OpenCode: bounded Green and ordinary Amber implementation.
- Codex: Red work, machine failures, recovery, and release gates.
- Matthew: product authority and report-routing bridge.
- Active prototype tree: `tethers-0.1/`.
- Required Rust toolchain: 1.89.0 with `--locked`.
- Required automation shell where applicable: PowerShell 7 (`pwsh.exe`).

## Authoritative References

- Enduring design principles: `docs/CONSTITUTION.md`
- Current 0.1 language and protocol semantics: `tethers-0.1/SPEC.md`
- Universal Plug architecture:
  `docs/architecture/TETHERS_UNIVERSAL_PLUG_ARCHITECTURE.md`
- First Plug Kit roadmap:
  `docs/architecture/TETHERS_J18_IMPLEMENTATION_ROADMAP.md`
- Lifecycle contract:
  `docs/architecture/TETHERS_LIFECYCLE_OUTCOMES_EVENTS_CONFORMANCE_V1.md`
- Capability bridge and host trust contract: `docs/CAPABILITY_BRIDGE.md`
- Accepted design decisions: `docs/DECISIONS.md`
- Current task state: `docs/CURRENT_CLINE_TASK.md`
- Short Matthew-facing status: `docs/PROJECT_DASHBOARD.md`
- Detailed queue: `docs/TASK_QUEUE.md`
- Evidence and reviews: `docs/worker-notes/`
