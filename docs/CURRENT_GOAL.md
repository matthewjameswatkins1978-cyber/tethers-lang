# Current Goal

Updated: 2026-09-01

## Goal

**Turn the completed Tethers foundation into something an AI agent would choose to use in ordinary work.**

The next product phase is an **Agent Essentials usefulness pass**:

- make installed capabilities easy for an unfamiliar agent to discover and inspect;
- expose the existing trusted manifest information through a clean machine-readable CLI;
- build genuinely useful executable Plug packs rather than only proof/reference providers;
- make planning, diagnostics, and Trail evidence easy for agents to consume;
- preserve Tethers as a deterministic execution substrate rather than turning it into another agent framework.

## Baseline already complete

Current `main` already contains the major foundation that this phase should use rather than redesign:

- Core phases 1-9 accepted and cut over to production evaluation;
- Canonical Format V2 / Rocket V2 integrated;
- public Plug authoring and conformance programme (0.3) complete;
- Plug lifecycle support for pack, inspect, conform, stage, install, enable, disable, and list;
- accepted Together semantics;
- physical provider overlap;
- bounded Together concurrency;
- adversarial concurrency crucible;
- Result Anchors and host-owned FIFO result-event queue;
- durable intent, replay, Trail, scope, policy, and provider-binding machinery;
- Portable Workbench 0.2.2 for Windows x64 and Linux x64 musl.

The earlier documentation that said the accepted 0.4 chain was still waiting to reach `main` is obsolete. It was integrated before the later portable-workbench commits now on `main`.

## Active product direction

The Agent Essentials work should prioritise practical agent use:

1. **Self-discovery CLI**
   - describe Tethers;
   - list available Capabilities;
   - inspect the exact trusted Capability contract;
   - inspect installed Plug state without needing the original package.

2. **Real Agent Essentials Plugs**
   - workspace/filesystem/text/patch;
   - Git;
   - process and named verification;
   - structured data;
   - hashes/integrity;
   - archives;
   - bounded HTTP/network;
   - SQLite;
   - read-only system/environment orientation.

3. **Planning and evidence ergonomics**
   - side-effect-free plan/preview surface;
   - precise configuration/scope diagnostics;
   - easier Trail querying and execution receipts.

4. **Cold-agent acceptance**
   - prove that an unfamiliar external client can discover what Tethers can do, inspect a Capability contract, execute harmless bounded work, and inspect the resulting evidence using only public surfaces.

5. **Documentation truth**
   - describe the full platform first;
   - keep the portable ALLOW / ASK / DENY workbench clearly labelled as one smaller façade;
   - distinguish Human Tether syntax from richer Core vocabulary;
   - distinguish authority decisions from execution outcomes.

## Do not reopen the foundation without evidence

This phase should **not** invent new semantics merely because the architecture can support them.

Do not add without a demonstrated blocker:

- another Core abstraction layer;
- another canonicalisation scheme;
- a global scheduler;
- an async runtime for its own sake;
- distributed execution;
- an LLM runtime inside Tethers;
- a second policy engine;
- vendor-specific Core semantics;
- new Human Tether syntax unrelated to a real agent-use problem.

The Core can stay rich while the everyday agent experience becomes simple.

## Product positioning

The intended relationship is:

```text
AI / agent decides what it wants
          |
          v
       Tethers
          |
          | explicit contracts
          | authority + scope
          | bounded execution
          | trustworthy evidence
          v
    real capabilities
```

Tethers should not compete with the agent.

It should become the thing an agent reaches for when it wants real work to happen predictably.

## Documentation boundary

The front-door current-truth documents are:

- `README.md`
- `QUICKSTART.md`
- `docs/PROJECT_OVERVIEW.md`
- `docs/SECURITY.md`
- `docs/PROJECT_DASHBOARD.md`
- this file

Historical roadmaps, architecture freezes, reviews, performance notes, and worker notes remain evidence of their checkpoints. Do not rewrite historical "not yet implemented" statements merely because later work completed them.

## Enduring boundaries

- Human Tether syntax remains small and canonical.
- A Plan remains a request, not permission.
- Core remains capability-agnostic.
- Capabilities describe; policy authorises; host enforces; Trail records.
- Physical scheduling must not alter semantic meaning.
- Provider advertising is not trusted manifest truth.
- Conformance is not permission.
- No automatic effectful retry without end-to-end idempotency proof.
- Supervised provider execution is not a hostile-code sandbox.
- Evidence beats agent confidence.
- Complexity must earn its keep.
