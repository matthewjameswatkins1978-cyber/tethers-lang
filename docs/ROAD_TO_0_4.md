# Road to Tethers 0.4 - Concurrency

> Concurrency belongs in Tethers semantics. Parallelism mostly belongs in the runtime.

A Tether declares which Actions are independent. The engine does not declare threads, workers, cores, async runtimes, operating-system processes, provider concurrency, or physical start timing.

## Status

**Tethers 0.4 concurrency is COMPLETE, ACCEPTED, and integrated into `main`.**

The accepted integration chain culminated at:

`14b2c65d1a830b4fc0a7a893ee3e72b684b09740`

That chain was subsequently integrated into `main` before the later Portable Workbench 0.2.2 commits.

The old wording in this roadmap that said `origin/main` was still waiting for the 0.4 merge described the pre-integration checkpoint and is no longer current product status.

This document remains the historical roadmap and acceptance summary for the 0.4 concurrency programme.

## Completed sequence

### C1 - Together semantic foundation ✓

Deterministic fan-out / join semantics are complete and accepted.

- `together` expresses independent Actions in one semantic group.
- All group members are attempted before the join is resolved.
- Join success requires all members to succeed.
- A non-success join blocks later Actions.
- First non-success is selected by semantic Runtime Plan order, never physical completion order.
- Tethers without `together` retains the established non-group behaviour.

### C2-A1 - Core-native Together semantic bridge ✓

Complete and accepted. Core carries `Together_origin` into flat source-order Runtime Plan `actions` plus additive non-empty `groups` while preserving Canonical V2 / Rocket meaning.

### C2-A2 - replay ownership + Trail ordering foundation ✓

Complete and accepted.

- replay admission ownership remains coordinator-side;
- Trail semantic placement is distinct from truthful physical durable append order.

### C2-A3 - physical provider overlap ✓

Design and implementation complete and accepted.

Together group members may overlap physically while the coordinator remains the owner of policy, capability resolution, replay, Trail, response, anchors and join semantics.

Same-provider Together members use independent ephemeral provider sessions so physical overlap does not silently serialize through one retained session.

Design artifact:

`docs/concurrency/C2_A3_PHYSICAL_CONCURRENCY_DESIGN.md`

### C3 - bounded concurrency / resource limits ✓

Complete and accepted.

The first bounded scheduler slice limits active Together provider invocations per group while preserving semantic order and existing observable meaning.

Key properties:

- runtime-only waiting state;
- queued wait does not consume provider timeout;
- earliest semantic-order waiting member receives freed capacity;
- a slot remains occupied until trusted Stage C terminalisation;
- fatal replay/Trail failures halt new launches while already-running siblings finish truthfully;
- no worker pool, async runtime, global scheduler or provider-priority machinery was introduced.

### C4 - adversarial concurrency crucible ✓

Complete and accepted.

Informal name: **The Bunny Baptism.**

C4 attacked the C1-C3 implementation with hostile completion order, fast failure beside slow success, replay G1/G2 failures, outcome durability failure, worker panic under N=2 pressure, same-provider inverse completion and repeated stress.

No production concurrency defect was found and no production semantic repair was required.

Channel disconnection could not be physically constructed with the production sender topology without adding forbidden watchdog/channel-lifetime machinery. The existing unexpected-disconnect path was reviewed as fail-closed; this was recorded as a deferred construction rather than falsely described as a runtime test.

### C5 - fresh-agent concurrency proof - RETIRED

C5 was originally intended to prove that a fresh agent could author a multi-capability `together` Tether from ordinary documentation.

That gate was retired because fresh, bounded agents are already the normal Gorilla Bunny development model and the proposed test was largely redundant as a concurrency acceptance gate.

A short exploratory C5 attempt was salvaged after repeated setup loops. It produced useful non-concurrency findings instead:

- a real `check` provider server-name bug;
- an undocumented `core_environment` authoring/runtime requirement;
- scope-binding/configuration usability friction.

The server-name bug was subsequently fixed and accepted at the integration tip above. The remaining findings belong to later usability work rather than concurrency correctness.

## Frozen 0.4 semantic principles

- Source semantics define independence and join meaning, not physical scheduling.
- Runtime Plan remains flat Actions plus additive group metadata, not a second hidden execution graph.
- Semantic member order is deterministic.
- Physical completion and Trail append order may vary without changing semantic meaning.
- Replay and Trail truthfulness remain coordinator-owned.
- Provider scheduling must not leak into source identity or first-non-success selection.
- Already-running effects are reported truthfully even after a fatal trusted-state failure halts future launches.

## What followed 0.4

0.4 is finished and should remain frozen unless real use exposes a defect.

The current product direction is an Agent Essentials usefulness pass:

```text
0.3 Plug extensibility ✓
-> 0.4 concurrency ✓
-> Agent Essentials / real agent use
-> later HQ and human-facing work
```

Current status belongs in:

- `docs/CURRENT_GOAL.md`
- `docs/PROJECT_DASHBOARD.md`

This roadmap is retained as historical acceptance evidence.
