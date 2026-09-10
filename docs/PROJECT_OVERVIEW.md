# Tethers Project Overview

Status: current system overview  
Updated: 2026-09-01

This document explains the **current architecture as a whole**. Historical roadmaps and worker notes remain evidence of how individual pieces were designed and proved, but this is the better place to understand what Tethers now is.

## 1. What Tethers is

Tethers is a small deterministic behaviour language surrounded by a typed capability, trust, execution, and evidence platform.

Its job is not to be a general-purpose programming language and not to be an AI agent framework.

Its job is to turn explicit intent into bounded, inspectable work.

```text
event + immutable Facts + Tether
              |
              v
       deterministic meaning
              |
              v
          Action Plan
              |
              v
 trust + policy + scope + durable intent
              |
              v
      Capability execution
              |
              v
 result / uncertainty / Result Anchor
              |
              v
             Trail
```

The key separation is:

```text
Tethers language: what behaviour is requested.
Capability contract: what operation exists and what Effects it may have.
Policy and scope: whether this exact operation may happen here.
Host/runtime: enforcement, durable intent, execution, replay, and evidence.
Plug/provider: application-specific implementation.
Trail: causal truth about what was proposed and what actually happened.
```

## 2. The deterministic centre

The OCaml side owns deterministic program meaning.

Current responsibilities include:

- parsing Human Tether source;
- validation;
- Anchor matching;
- Condition evaluation over supplied immutable Facts;
- typed Action planning;
- Human Tether AST to Tethers Core lowering;
- Core validation;
- semantic canonicalisation;
- program digest identity;
- Core to Runtime Plan bridging;
- evaluation/protocol responses.

Core does not secretly read:

- wall clock;
- random source;
- filesystem;
- network;
- environment;
- live database;
- provider state.

Changing information must arrive as explicit event data, Facts, Capability projections, or other declared runtime input.

## 3. Human Tether syntax versus Core

The public human-facing language is intentionally small.

The precise source-language contract lives in [`../tethers-0.1/SPEC.md`](../tethers-0.1/SPEC.md).

The current surface has:

- one Anchor;
- zero or more Conditions;
- one or more Actions;
- explicit `together` fan-out/join groups.

It deliberately does not become arbitrary scripting.

Tethers Core is richer than the current source surface. Its typed vocabulary includes distinct semantic identities and structures for:

- programs;
- origins;
- Facts;
- capabilities;
- groups;
- branches;
- roles;
- batches;
- item templates.

That richness gives future and internal semantics a stable home without forcing every structure into the human language immediately.

**Do not equate "represented in Core" with "currently exposed in source syntax or supported on every runtime bridge".**

## 4. Semantic identity and canonicalisation

Tethers does not treat raw internal IDs or container order as semantic meaning.

Canonical Format V2 gives validated Core programs stable identity through a frozen byte encoding and SHA-256 digest.

Important properties include:

- raw IDs are not semantic identity;
- representation/storage order does not define meaning;
- multiplicity is preserved;
- semantic scalar values are preserved;
- canonicalisation fails closed when validation or deterministic work budgets fail;
- independent implementations/oracles are used as differential evidence.

This means a program can survive representation changes without casually changing what Tethers considers the program to be.

## 5. Plans are requests, not permission

The deterministic engine produces a Plan.

The Plan may name Actions and Together groups, but it is still only a request.

The host owns the consequential boundary:

1. resolve the exact capability;
2. verify trusted manifest/provider evidence;
3. establish effective policy and scope;
4. resolve approval requirements;
5. establish replay state;
6. record durable intent before effectful execution;
7. invoke the provider;
8. classify the outcome;
9. validate structured output;
10. persist trustworthy result/replay evidence;
11. append host Trail evidence;
12. emit a Result Anchor when appropriate.

The planner cannot approve its own work.

## 6. Capability contracts

A Capability is identified by name and version and is backed by a trusted manifest.

The manifest can cover:

- title and description;
- strict input schema;
- strict output schema;
- Effects;
- permission scope;
- reversibility;
- determinism;
- idempotency;
- confirmation policy;
- timeout;
- retry contract;
- provider identity;
- protocol binding.

Discovered provider metadata is not automatically trusted.

The host compares live provider/discovery state with reviewed trusted evidence. Drift or ambiguity fails closed rather than silently changing the operation beneath an existing Plan.

## 7. Plugs

A Plug is the public integration unit that brings one provider and one or more related Capabilities into Tethers.

The generic host must not grow vendor-specific branches such as:

```text
if provider is GitHub
if provider is PDF
if provider is email
```

That application-specific meaning belongs inside the Plug/provider and its manifests.

The implemented Plug lifecycle includes:

```text
pack
inspect
conform
stage
install
enable
disable
list
```

The boundaries matter:

- **pack** creates deterministic package evidence;
- **inspect** treats a package as hostile read-only data;
- **conform** executes the conformance contract under explicit supervision;
- **stage/install** create host-owned lifecycle state;
- **enable** binds operational scope;
- **disable** removes operational availability without erasing historical evidence.

Conformance does not equal trust, installation, enablement, or permission.

The 0.3 public Plug authoring programme was proved with:

- PDF Tools reference Plug;
- Text Stats fresh-agent authoring proof;
- Evil Bunny adversarial provider suite.

## 8. Provider execution and Tethers Socket

The host-provider architecture separates:

```text
Tethers semantic Socket
    -> protocol binding
    -> transport
    -> provider
```

The implemented reference path uses an MCP stdio binding.

The Socket is a semantic contract, not a claim that every provider must use the same process layout forever.

The provider remains untrusted at the protocol boundary. Host-side execution verifies the identities and contracts that matter instead of trusting whatever the provider reports.

## 9. Together and bounded physical concurrency

The surface language can explicitly declare independent Actions with `together`.

Example:

```tethers
do
    together
        weather.fetch
            location: anchor.location

        calendar.fetch
            day: anchor.day

    brief.compose
        format: "short"
```

The semantic rules are deterministic:

- group membership comes from source meaning;
- members have stable semantic order;
- later Actions wait for the group join;
- all members must terminalise before the join resolves;
- first non-success selection follows semantic member order.

The Rust host may overlap Together member provider calls physically.

The accepted 0.4 runtime adds:

- physical provider overlap;
- independent provider sessions where required;
- bounded active concurrency;
- earliest-semantic-member admission when capacity frees;
- truthful completion of already-running work after fatal trusted-state failures;
- semantic Trail position separate from physical append/completion order.

No worker pool, async runtime, or global scheduler was required to establish these semantics.

Concurrency therefore remains an execution strategy under a deterministic language contract.

## 10. Replay, durable intent, and uncertainty

Tethers treats externally significant execution as something that must survive awkward failure boundaries.

The host uses durable intent and replay state so a crash or repeated request does not casually become a duplicate external effect.

The system distinguishes states such as:

- completed success;
- completed failure;
- uncertain post-invocation state;
- approval required;
- replay requiring manual resolution;
- persistence unavailable;
- unattempted work.

A timeout or lost final response after invocation is not automatically a safe retry.

Tethers currently follows the rule:

> **No automatic retry unless idempotency is proved end to end for the relevant contract.**

## 11. Result Anchors and multi-step behaviour

Known provider outcomes may produce standard Result Anchors:

```text
capability.succeeded
capability.failed
capability.uncertain
```

These carry evaluation, Action, capability, manifest, provider, correlation, causation, and generation evidence.

Generated Result Anchors enter a host-owned FIFO queue.

The queue is intentionally serial at the evaluation level:

- no recursive immediate re-entry;
- children append behind already-waiting siblings;
- stable FIFO order;
- bounded causal generation/admission rules.

This is separate from Together provider concurrency. Tethers can overlap independent provider calls inside a group while still processing generated follow-up events through a stable event queue.

## 12. Trail

The Trail is not decorative logging.

It is causal evidence shared across deterministic evaluation and effectful host execution.

It can record:

- reception;
- evaluation;
- planning;
- semantic Action/group position;
- authority decisions;
- durable intent;
- provider attempt;
- result/failure/uncertainty;
- group join;
- replay identity;
- Result Anchor correlation.

Pure deterministic Core entries remain independent of wall-clock time. Host execution entries may include timestamps because the host is the effectful runtime boundary.

## 13. Security boundary

The reference host has strong trust machinery, but supervised Plug execution is not a hostile-code sandbox.

Current security value comes from layers such as:

- strict package/manifest validation;
- host-owned provider identity;
- reviewed capability manifests;
- live binding revalidation;
- scope evidence;
- explicit policy;
- approval boundaries;
- durable intent;
- replay protection;
- protocol/output validation;
- bounded provider deadlines;
- redacted evidence;
- process supervision.

Those controls do not prove that arbitrary provider code is isolated from the machine's filesystem, network, credentials, DLL loading, or operating-system APIs.

See [`SECURITY.md`](SECURITY.md).

## 14. Portable workbench

`tethers-0.1/portable-rust/` is a deliberately smaller, self-contained authority façade.

It answers local policy questions:

```text
request -> ALLOW / ASK / DENY
```

and does not execute the action itself.

It exists because a small deterministic authority binary is useful for scripts and agents even when the full Tethers host/runtime is not being used.

Do not infer the full platform's limits from the portable façade.

## 15. Version map

The repository carries several version axes:

| Axis | Current meaning |
| --- | --- |
| Human Tether language/protocol | `0.1` |
| Reference host Cargo package | `0.2.2` |
| Portable workbench | `0.2.2` |
| 0.3 | completed public Plug-authoring milestone |
| 0.4 | completed Together/concurrency milestone |

These axes are related but not interchangeable.

## 16. Current implementation boundaries

Implemented and integrated:

- deterministic 0.1 source language;
- typed Core and lowering;
- static Core validation;
- canonical program identity;
- production Core evaluation path;
- trusted manifest/capability bridge;
- host policy and scope enforcement;
- provider supervision;
- durable intent and replay machinery;
- Result Anchors and FIFO result-event queue;
- public Plug lifecycle/authoring;
- bounded physical Together concurrency;
- portable 0.2.2 workbench.

Important boundaries that remain:

- Tethers is not a general-purpose scripting language;
- nested `together` is not part of current 0.1 syntax;
- direct arbitrary Action-result references are not the normal 0.1 chaining model;
- not every richer Core structure is exposed by the Human Tether language;
- supervised providers are not hostile-code sandboxed;
- the full reference host still contains Windows-specific durability/containment paths even though the portable workbench ships for Windows and Linux.

## 17. Documentation authority

Use documents by purpose rather than treating every old roadmap as current truth:

1. [`CONSTITUTION.md`](CONSTITUTION.md) - enduring design principles.
2. [`../tethers-0.1/SPEC.md`](../tethers-0.1/SPEC.md) - exact Human Tether 0.1 semantics.
3. [`DECISIONS.md`](DECISIONS.md) - accepted architecture decisions.
4. [`CAPABILITY_BRIDGE.md`](CAPABILITY_BRIDGE.md) - trusted capability/manifest bridge.
5. [`PROJECT_OVERVIEW.md`](PROJECT_OVERVIEW.md) - current whole-system explanation.
6. [`SECURITY.md`](SECURITY.md) - current security claims and limits.
7. [`PLUG_AUTHORING.md`](PLUG_AUTHORING.md) - public Plug author contract.
8. [`CURRENT_GOAL.md`](CURRENT_GOAL.md) and [`PROJECT_DASHBOARD.md`](PROJECT_DASHBOARD.md) - living direction/status.

`ROAD_TO_*`, `worker-notes/`, `review/`, `perf/`, and foundation-pass documents are important implementation history and evidence. Their old "not yet implemented" statements are true for the checkpoint they describe and should not be read as present-day product status unless a living document points to them.
