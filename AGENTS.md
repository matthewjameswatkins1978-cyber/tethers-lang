# Tethers Agent Map

This file is deliberately small. It is a routing map plus permanent Tethers boundaries, not a repository manual. Load deeper context only when the current task requires it.

## AI-first operating rules

1. Make consequential truth explicit.
2. Keep one semantic authority.
3. Keep reasoning bounded and dependency-complete.
4. Prefer simple explicit structure over clever indirection.
5. Make knowledge discoverable, reusable, and load it on demand.
6. Put critical constraints below the model.
7. Verify the actual requirement independently.
8. Let evidence rewrite the method.

Do not preload the repository because the information exists. Start with the smallest dependency-complete working set and expand only when a necessary dependency is missing.

## Start here

Before mutation:

1. Inspect repository root, branch, exact `HEAD`, and `git status`.
2. `git fetch origin --prune` and confirm the task is based on the current authorised base. Historical task branches are evidence, not continuation authority.
3. Read the current task packet and the parts of `docs/PROJECT_CONTROL.md` needed to establish owner, state, authorised scope, base, acceptance, stop conditions, and publication rules.
4. Read only the semantic authorities, code, tests, and guides relevant to the task.
5. Run the task-packet checker before claiming completion:

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
```

If the task packet is stale, contradictory, owned elsewhere, or based on the wrong checkout, stop before mutation.

## Authority map

Use the narrowest relevant authority. Do not load every document by default.

- `docs/CONSTITUTION.md` — enduring Tethers design principles.
- `tethers-0.1/SPEC.md` — precise language/protocol semantics.
- `docs/DECISIONS.md` — accepted design decisions.
- `docs/CAPABILITY_BRIDGE.md` — manifest, trust, and host bridge contract.
- `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md` — implementation technique only; never overrides product semantics.
- current task packet — frozen task scope, acceptance, base and stop conditions.
- code/tests/fixtures/compiler output/Git — implementation evidence.

Optional orientation such as `docs/TETHERS_LUCY_NOTES.md` is not semantic authority.

## Load deeper guidance only when triggered

- OCaml change → read the relevant parts of `docs/OCAML_GUIDE_FOR_AGENTS.md`.
- Rust change → read the relevant parts of `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md`.
- Git topology, worktrees, line endings, encoding, history recovery, or destructive Git work → read `docs/GIT_WORKTREES_AND_LINE_ENDINGS_FOR_AGENTS.md`.
- project reporting → read `docs/PROJECT_DASHBOARD.md` only if the task actually concerns reporting.
- workflow mechanics beyond the task packet → consult `docs/AGENT_WORKFLOW.md` only as needed.

A filename is a route, not an import. Do not repeatedly rediscover stable truth that an authoritative document already states.

## Project identity

Tethers is a small deterministic behaviour language and capability protocol.

> Apps provide the sockets. Tethers provides the cables.

A Tether means: when this event happens, check these known facts, then propose these permitted actions.

Tethers is a deterministic planner. It proposes Actions. It does not grant permission and does not execute them.

## Core boundary

```text
Host application
  supplies event + immutable Facts + Capability schemas + Tether source
      ↓
Tethers Core — OCaml
  parses, validates, evaluates, and proposes an ordered Plan
      ↓
Host application
  resolves policy, records durable intent, executes approved Actions,
  validates results, and appends host Trail entries
```

Keep these meanings separate:

```text
Schemas describe.
Policies authorise.
Hosts enforce.
Trails record.
```

Core remains application-agnostic. Product-specific behaviour belongs in Capabilities, adapters, host policy, or host code.

## Permanent invariants

These are consequential enough to stay in always-loaded context:

- Same complete deterministic input → same semantic Plan and evaluation Trail.
- Core does not secretly read clock, environment, filesystem, network, database, randomness, live state, or undeclared configuration.
- Time and changing state arrive explicitly as event data or Facts.
- A Plan is a request, not permission.
- Current manifest/provider pins must be checked before dispatch.
- Structured scope without a host/binding-owned assessment fails closed.
- Do not infer argument-to-resource mappings without an approved binding or adapter contract.
- AI judgement is an explicit Capability Action whose structured result becomes later visible data; it never runs invisibly inside Conditions.
- Runtime scheduling/concurrency must not change source-level semantic ordering or failure meaning.
- No automatic retry until end-to-end idempotency is proved.
- Do not claim an Action happened when Tethers only proposed it.
- Do not change 0.1 syntax or semantics without an explicit design gate.

For exact syntax and edge semantics, read the specification rather than expanding this file.

## Work packet contract

For substantial work, reason from:

```text
GOAL
SCOPE
RELEVANT SEMANTICS
MUST REMAIN TRUE
ACCEPTANCE CONDITIONS
```

Keep the task bounded. Do not use a local assignment as permission for unrelated refactors, architecture redesign, dependency replacement, stylistic rewrites, or speculative cleanup.

If a necessary semantic dependency is missing, retrieve it. If product meaning is genuinely contradictory or absent, surface the decision instead of inventing it.

After substantial exploration, preserve only durable reusable discoveries in the appropriate authority/evidence surface. Do not turn exploratory logs into permanent agent context.

## Verification and handoff

Use task-specific evidence proportional to consequence. Compilation and inherited tests are useful but do not automatically prove the requested behaviour.

Before completion:

- run the packet-required checks and relevant focused/regression tests;
- inspect the complete diff and final Git status;
- preserve unrelated/user-authored work;
- write/update the task's required evidence or worker note;
- obey the packet's publication rules;
- do not select or begin the next task.

Return concise evidence:

```text
WHAT CHANGED
WHAT WAS VERIFIED
WHAT REMAINS UNVERIFIED
SEMANTIC ASSUMPTIONS
BLOCKERS
DURABLE DISCOVERIES (only if any)
```

"Done" is not evidence.
