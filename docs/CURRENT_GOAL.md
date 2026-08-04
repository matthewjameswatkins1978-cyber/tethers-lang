# Current Goal

Updated: 2026-08-04

## Goal

Extend the accepted public Plug lifecycle from safe package intake toward one
simple but high-assurance installation operation without changing Tethers 0.1
language semantics, putting Plugs into Tethers Core, or collapsing trust,
conformance, approval, installation, enablement, policy, replay, Anchor, and
Trail authorities.

## Accepted baseline

Tethers 0.2.0 remains the accepted and published baseline. The annotated
`v0.2.0` tag remains at
`b5546411661dcbcb53e1cf2538eaec594c6f76f2`; language semantics remain 0.1.

The Universal Plug architecture remains frozen at
`a5fd63593a9d9acd397030ecd2e27b4f318c87fd`. Existing host modules already
provide package inspection, safe quarantine and candidate identity, publisher
and developer trust, supervised launch evidence, conformance, installation
approval, installed-disabled publication, enablement histories, operational
scopes, installed PDF execution, and durable lifecycle authorities.

The accepted public Plug surface contains:

- `plug inspect`;
- `plug list`;
- `plug disable`;
- permission-file `plug enable`;
- `plug stage`.

The accepted package-intake sequence is:

```text
plug inspect
→ plug stage
→ immutable quarantine
→ reusable candidate identity
```

Staging grants no trust, approval, installation, permission, or operational
availability.

## Accepted installation foundations

J24G adds the strict public installation-request contract:

```json
{
  "schema": "tethers.plug-install/1",
  "candidate_id": "<canonical UUID>",
  "trust": { "scope": "exact_candidate" },
  "conformance": {
    "allow_non_isolated_supervised_execution": true
  },
  "installation": { "target_state": "disabled" }
}
```

The request is bounded, duplicate-aware, read-only, and permits only one exact
candidate, explicit non-isolated supervised execution, and disabled
installation.

J24H adds durable launch-profile evidence plus non-creating read-only store
openings. A later invocation can inspect existing candidate, trust,
launch-profile, conformance, approval, and installed evidence without creating
empty roots merely by planning.

## Active increment

J24I adds exact-candidate installation trust.

The existing publisher-trust and developer-approval records are deliberately
not treated as equivalent:

- publisher trust is wider than one candidate;
- developer approval is pinned to a semantic package digest rather than one
  candidate record.

J24I adds one immutable record bound to:

```text
candidate ID
+ candidate record digest
+ package and provider identity
+ semantic package digest
+ raw archive digest
+ approving authority
```

It also adds an exact-candidate `PackageTrustEvidence` mode for the future
planner. That mode must fail current-authority execution revalidation until the
future locked executor explicitly supplies the exact trust store.

After J24I, the remaining reviewed sequence is:

```text
J24J  read-only installation reconciliation planner
J24K  host installation lock and gate executor
J24L  thin public plug install CLI
```

The user-facing operation may eventually be one command, but its internal gates
remain separate and independently testable.

## Frozen installation shape

```text
validated installation request
→ exact candidate validation
→ exact-candidate trust
→ durable supervised launch profile
→ supervised conformance
→ installation approval
→ atomic installed publication
→ present_disabled
```

A failed installation may leave completed immutable gate evidence, but it must
never leave a Plug falsely or partially installed.

Exact replay must reuse current matching evidence and return the same installed
identity without rerunning provider code or rewriting records. A different
candidate must never be mistaken for replay, even when package release text is
the same.

## Frozen boundaries

- Tethers Core remains deterministic and application-agnostic.
- Plugs remain outside the language Core.
- Package inspection and planning never execute payloads.
- Candidate identity remains distinct from installed identity.
- Publisher trust, semantic-digest developer approval, and exact-candidate trust
  remain distinct authorities.
- The installation request grants no publisher-wide trust.
- Supervised execution remains explicitly non-isolated.
- A read-only planner creates no directory, record, scratch path, process, or
  lock.
- A plan is advice, never authority; the executor must replan after acquiring
  the host installation lock.
- Candidate bytes must be reverified before and after conformance and before
  installed publication.
- Every immutable evidence record is atomically published through audited store
  primitives.
- Installed state is always `present_disabled`.
- Only a separate exact current enablement record creates operational
  availability.
- Installation never grants operational scope, policy, credentials, runtime Ask
  approval, Anchor admission, or Trail authority.
- No public download/update registry, network listener, OAuth, arbitrary
  third-party enablement, or Tether language change belongs to this increment.

## Active development posture

Current operating mode: **Gorilla Coding**.

- Lucy: architecture, packet compilation, independent review, and routine safe
  merges.
- OpenCode: implementation programme.
- Luna: bounded Green and ordinary Amber implementation.
- HY3: mechanical, repetitive, low-risk implementation.
- DeepSeek Pro V4: thicker cross-module integration under frozen contracts.
- Matthew: product authority, ideas, priorities, and human judgement.
- Cline and Goose are not used.
- Active prototype tree: `tethers-0.1/`.
- Required Rust toolchain: 1.89.0 with `--locked`.
- Required automation shell where applicable: PowerShell 7 (`pwsh.exe`).

DeepSeek editing rule: after an exact `oldString` replacement failure, reread
the current file and create a fresh smaller patch. Never repeat the identical
failed edit, and stop after two materially different failed attempts rather than
rewriting a file wholesale.

## Authoritative references

- Enduring principles: `docs/CONSTITUTION.md`
- Language and protocol semantics: `tethers-0.1/SPEC.md`
- Universal Plug architecture:
  `docs/architecture/TETHERS_UNIVERSAL_PLUG_ARCHITECTURE.md`
- J24E candidate preparation:
  `docs/architecture/J24E_CANDIDATE_PREPARATION_BLUEPRINT.md`
- J24F public staging:
  `docs/architecture/J24F_PLUG_STAGE_CLI_BLUEPRINT.md`
- J24G request contract:
  `docs/architecture/J24G_INSTALLATION_REQUEST_CONTRACT.md`
- J24H evidence-access foundation:
  `docs/architecture/J24H_INSTALLATION_EVIDENCE_ACCESS_FOUNDATION.md`
- J24I exact-candidate trust:
  `docs/architecture/J24I_EXACT_CANDIDATE_INSTALLATION_TRUST.md`
- Lifecycle contract:
  `docs/architecture/TETHERS_LIFECYCLE_OUTCOMES_EVENTS_CONFORMANCE_V1.md`
- Capability bridge and host trust contract: `docs/CAPABILITY_BRIDGE.md`
- Accepted design decisions: `docs/DECISIONS.md`
- Current task: `docs/CURRENT_CLINE_TASK.md`
- Short project status: `docs/PROJECT_DASHBOARD.md`
- Detailed queue: `docs/TASK_QUEUE.md`
- Evidence and reviews: `docs/worker-notes/`
