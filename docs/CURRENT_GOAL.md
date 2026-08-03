# Current Goal

Updated: 2026-08-03

## Goal

Extend the accepted public Plug lifecycle into safe package intake without
changing Tethers 0.1 language semantics, putting Plugs into Tethers Core, or
collapsing package inspection, quarantine, trust, conformance, installation,
enablement, policy, replay, Anchor and Trail authorities.

## Accepted Baseline

Tethers 0.2.0 remains the accepted and published baseline. The annotated
`v0.2.0` tag remains at
`b5546411661dcbcb53e1cf2538eaec594c6f76f2`; language semantics remain 0.1.

The Universal Plug architecture remains frozen at
`a5fd63593a9d9acd397030ecd2e27b4f318c87fd`. The implementation on `main`
contains strict package inspection, safe quarantine and candidate identity,
trust and supervised-launch evidence, conformance, installation approval,
installed-disabled publication, enablement histories, operational scopes,
installed PDF execution and durable host lifecycle authorities.

The accepted public Plug surface now includes:

- J24A `plug inspect` at
  `13f6a3caffa00904f6357c7975a8a0937a6c2d5c`;
- J24B `plug list` at
  `726c6aa780c6809fce32de39427200217cbad12f`;
- J24C `plug disable` at
  `aac395a522e9d90573870a7f53e00b4fb075a4d7`;
- J24D permission-file `plug enable` at
  `f8c63b907efca1e0f9f1839d542f79221c7298f2`.

Together these commands provide inspection, validated state reporting and an
explicit immutable enable/disable loop. J24D established
`tethers.plug-scope/1` as the first human/automation-facing permission request
format while keeping host-generated integrity evidence internal.

## Active Increment

J24E adds one internal candidate-preparation application service. It composes the
accepted package inspector, quarantine extractor and candidate registry behind
one narrow, idempotent, rollback-aware seam.

J24E deliberately adds no CLI. J24F will later expose a thin `plug stage`
command that calls the J24E service rather than reimplementing archive,
quarantine or candidate rules.

A J24E candidate remains untrusted, unapproved, uninstalled, disabled and
non-operational. It creates no active capability binding, provider session,
policy permission, event, Anchor or Trail.

## Frozen Boundaries

- Tethers Core remains deterministic and application-agnostic.
- Plugs remain outside the language Core.
- Package inspection never executes payloads.
- Candidate/quarantine identity is not installed identity.
- Candidate preparation grants no trust, approval, installation or permission.
- Package, candidate, installed, provider and capability identities remain
  distinct.
- Exact archive replay may reuse one validated candidate but may not rewrite it.
- Same package release with different semantic evidence fails closed before
  extraction.
- Low-level package, quarantine and candidate validation remain owned by their
  existing modules; the application seam only composes them.
- Installation approval remains distinct from runtime Ask approval.
- Installed state remains `present_disabled`; only an exact current enablement
  record creates operational availability.
- Structured scope without a host/binding-owned assessment fails closed.
- Supervised provider execution remains explicitly non-isolated and must not be
  described as hostile-code safe.
- No public registry, download/update path, network listener, OAuth, arbitrary
  third-party enablement or Tether language change belongs to this increment.

## Active Development Posture

Current operating mode: **Gorilla Coding**.

- Lucy: architecture, packet compilation, independent review and routine safe
  merges.
- OpenCode: the implementation programme.
- Luna: bounded Green and ordinary Amber implementation.
- HY3: mechanical, repetitive and low-risk implementation work.
- DeepSeek Pro V4: thicker cross-module integration under frozen contracts.
- Matthew: product authority, ideas, priorities and judgement where human taste
  or intent matters.
- Cline and Goose are not used.
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
- J24E implementation blueprint:
  `docs/architecture/J24E_CANDIDATE_PREPARATION_BLUEPRINT.md`
- Lifecycle contract:
  `docs/architecture/TETHERS_LIFECYCLE_OUTCOMES_EVENTS_CONFORMANCE_V1.md`
- Capability bridge and host trust contract: `docs/CAPABILITY_BRIDGE.md`
- Accepted design decisions: `docs/DECISIONS.md`
- Current task state: `docs/CURRENT_CLINE_TASK.md`
- Short Matthew-facing status: `docs/PROJECT_DASHBOARD.md`
- Detailed queue: `docs/TASK_QUEUE.md`
- Evidence and reviews: `docs/worker-notes/`
