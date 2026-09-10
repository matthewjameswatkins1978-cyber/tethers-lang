# Current Goal

Updated: 2026-09-10

## Goal

**Turn the Tethers 0.5 practical release from a strong technical proof into an execution layer that AI agents and developers can adopt, understand, and reach for in ordinary work.**

The foundation is no longer the problem. The first Agent Essentials pass is no longer the problem either. The next phase is product coherence, real-world use, and hardening through actual workloads.

## What has already been achieved

The published Tethers 0.5 release line includes the major capabilities that the previous goal document described as future Agent Essentials work:

- deterministic Human Tether semantics and typed Core;
- trusted Capability manifests, policy, scope, provider binding, durable intent, replay, and Trail machinery;
- public Plug packaging, inspection, conformance, staging, installation, enablement, disablement, and listing;
- `together` semantics with bounded physical provider concurrency;
- Result Anchors and the host-owned FIFO result-event queue;
- read-only native host discovery;
- trusted Capability listing and contract inspection;
- installed Plug inspection;
- side-effect-free preview;
- bounded Trail receipt projection;
- agent-oriented workspace/text/hash/patch reference capabilities;
- structured Git, argv-only process, and named verification reference capabilities;
- deterministic `tethers-bench` verification tooling;
- Windows x64 and Linux x64 musl practical release bundles;
- the separate Portable Workbench 0.2.2 authority façade.

The latest published GitHub release is Tethers 0.5, tagged `tethers-v0.5.8`.

## Repository coherence

The Tethers 0.5 implementation ancestry and the current `main` line have now
been deliberately reconciled. The `tethers-v0.5.8` tag remains intact and
reachable, and the deterministic `tethers plan` surface is present on `main`.
The tag remains the source of truth for reproducing the published bundle while
`main` is the authoritative development line.

3. **Planning and evidence ergonomics**
   - stable side-effect-free `tethers plan` / `tethers preview` surfaces;
   - precise configuration/scope diagnostics;
   - easier Trail querying and execution receipts.

The repository-coherence problem is now resolved on the reconciled main line;
the remaining work is product coherence, real-world use, and hardening through
actual workloads.

## Current product direction

### 1. Make the product legible in under a minute

A new reader should quickly understand:

```text
AI / application decides what it wants
          |
          v
       Tethers
          |
          | deterministic Plan
          | trusted Capability contract
          | policy + scope
          | bounded execution
          | replay + evidence
          v
      real systems
```

The front-door story is not “another policy engine” and not “another agent framework.”

Tethers is the deterministic execution substrate between probabilistic intent and consequential effects.

### 2. Reconcile release and version presentation

Tethers currently carries several legitimate version axes:

- Human Tether language semantics: `0.1`;
- reference-host Cargo package: `0.2.2`;
- Portable Workbench: `0.2.2`;
- completed Plug milestone: `0.3`;
- completed Together/concurrency milestone: `0.4`;
- practical product release: `0.5`, latest tag `tethers-v0.5.8`.

The 0.5 release assets currently use `tethers-0.5.0-*` filenames. This can be explained, but it is not a good long-term product experience. Future release work should make the public version identity boring and obvious.

### 3. Use Tethers in real agent work

The next useful evidence should come from ordinary consequential jobs, not another abstract architecture layer.

Prioritise scenarios such as:

- bounded repository inspection and editing;
- structured Git work;
- named verification/test execution;
- deterministic file/text/hash operations;
- explicit approval before remote or destructive actions;
- Trail inspection after a completed job;
- replay and uncertainty behaviour under awkward failures.

Actual use should tell us which missing Capabilities or ergonomics matter.

### 4. Grow practical Plugs without growing Core

The useful expansion point is the Capability/Plug layer.

Good candidates are everyday operations agents repeatedly need, provided they can be given a clear semantic contract and meaningful scope. Avoid turning every operating-system facility into a generic shell-shaped capability simply because that is easy to expose.

> **Deep Plug, narrow subject. Wide workflow, Tether.**

### 5. Prove cold-agent usability

An unfamiliar agent should be able to:

```text
discover Tethers
    -> discover trusted Capabilities
    -> inspect exact contracts and scope
    -> preview intended work
    -> execute admitted work
    -> inspect the resulting evidence
```

without hidden project knowledge and without inventing commands.

This should remain a recurring acceptance test as the product changes.

### 6. Keep consequential semantics boring

Do not reopen solved foundations merely because they are interesting.

Do not add without a demonstrated real-world blocker:

- another Core abstraction layer;
- another canonicalisation scheme;
- a global scheduler;
- an async runtime for its own sake;
- distributed execution;
- an LLM runtime inside Tethers;
- a second policy engine;
- vendor-specific Core semantics;
- new Human Tether syntax that does not solve a proved user problem.

Complexity still has to earn its keep.

## Product positioning

The central claim is now:

> **AI can be probabilistic. The boundary where it changes the world does not have to be.**

Tethers should be the thing an agent reaches for when it wants real work to happen predictably, under explicit authority, with evidence that survives after the model has moved on.

The useful contrast is not “Tethers versus AI.” It is:

```text
AI reasoning             -> flexible, probabilistic, replaceable
Tethers execution layer  -> deterministic, typed, scoped, auditable
```

## Documentation boundary

The living current-truth documents are:

- `README.md`
- `QUICKSTART.md`
- `docs/PROJECT_OVERVIEW.md`
- `docs/SECURITY.md`
- `docs/PROJECT_DASHBOARD.md`
- this file

Historical roadmaps, architecture freezes, reviews, performance notes, and worker notes remain evidence of their checkpoints. Do not rewrite historical “not yet implemented” statements merely because later work completed them.

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
- Product usefulness matters more than architecture for architecture's sake.
