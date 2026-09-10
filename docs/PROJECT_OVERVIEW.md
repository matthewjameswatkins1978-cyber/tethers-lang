# Tethers Project Overview

Status: current whole-system overview
Updated: 2026-09-10

Tethers is a deterministic behaviour language surrounded by a typed capability, trust, execution, replay, and evidence platform.

Its most useful role is underneath AI and automation:

> **Let the caller be flexible about intent. Make consequential execution explicit, bounded, and provable.**

Tethers is not a general-purpose programming language and not an AI agent framework. It is the layer that turns explicit intent into inspectable work without allowing the planner, provider, or transport to quietly redefine the rules at execution time.

## 1. The problem it solves

An autonomous tool call often mixes several concerns together:

```text
intent + operation discovery + permission + execution + retry + evidence
```

Tethers separates them:

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
 trusted Capability + policy + scope
              |
              v
      Capability execution
              |
              v
 result / failure / uncertainty
              |
              v
       Result Anchor + Trail
```

The boundaries are deliberate:

```text
Tethers language: what behaviour is requested.
Capability contract: what operation exists and what Effects it may have.
Policy and scope: whether this exact operation may happen here.
Host/runtime: enforcement, durable intent, execution, replay, and evidence.
Plug/provider: application-specific implementation.
Trail: causal truth about what was proposed and what actually happened.
```

A model may be probabilistic. These boundaries do not need to be.

## 2. The deterministic centre

The OCaml side owns deterministic program meaning.

Responsibilities include:

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

Core does not secretly read the wall clock, randomness, filesystem, network, environment, live database, or provider state.

Changing information must arrive as explicit event data, Facts, Capability projections, or other declared runtime input.

This is one of the project's strongest properties: a Tether is not “deterministic except for the bits where it quietly asks the world what is happening.”

## 3. Human Tether syntax versus Core

The public human-facing language is intentionally small. The precise source-language contract lives in [`../tethers-0.1/SPEC.md`](../tethers-0.1/SPEC.md).

The current surface includes:

- one Anchor;
- zero or more Conditions;
- one or more Actions;
- explicit `together` fan-out/join groups.

It deliberately does not become arbitrary scripting.

Tethers Core is richer than the current source surface. Its typed vocabulary includes distinct semantic identities and structures for programs, origins, Facts, capabilities, groups, branches, roles, batches, and item templates.

That richness gives internal and future semantics a stable home without forcing every structure into the Human Tether language immediately.

**“Represented in Core” does not mean “already exposed as source syntax or supported on every runtime bridge.”**

## 4. Semantic identity and canonicalisation

Tethers does not treat raw internal IDs or container order as semantic meaning.

Canonical Format V2 gives validated Core programs stable identity through a frozen byte encoding and SHA-256 digest.

Important properties include:

- raw IDs are not semantic identity;
- representation or storage order does not define meaning;
- multiplicity is preserved;
- semantic scalar values are preserved;
- canonicalisation fails closed when validation or deterministic work budgets fail;
- independent implementations and oracles can be used as differential evidence.

The 0.6 release line extends the exact implementation portfolio around that frozen identity rather than changing the identity format itself. Rocket work changes how exact canonical results are reached and evidenced, not what a valid canonical answer means.

That distinction matters because performance machinery must not quietly become a new semantic specification.

## 5. Plans are requests, not permission

The deterministic engine produces a Plan.

A Plan may name Actions and Together groups, but it is still only a request.

The host owns the consequential boundary:

1. resolve the exact Capability;
2. verify trusted manifest and provider evidence;
3. establish effective policy and scope;
4. resolve approval requirements;
5. establish replay state;
6. record durable intent before effectful execution;
7. invoke the provider;
8. classify the outcome;
9. validate structured output;
10. persist trustworthy result and replay evidence;
11. append host Trail evidence;
12. emit a Result Anchor when appropriate.

The planner cannot approve its own work.

## 6. Capability contracts

A Capability is a versioned typed operation backed by trusted manifest evidence.

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

The host compares live provider or discovery state with reviewed trusted evidence. Drift or ambiguity fails closed rather than silently changing the operation beneath an existing Plan.

This is especially useful for agents because “the tool says it can do X” and “the host has reviewed and authorised capability X version Y under this scope” remain different claims.

## 7. Plugs

A Plug is the public integration unit that brings one provider and one or more related Capabilities into Tethers.

The generic host should not grow application-specific branches such as:

```text
if provider is GitHub
if provider is PDF
if provider is email
```

That meaning belongs inside the Plug/provider and its manifests.

The implemented lifecycle includes:

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
- **conform** exercises the declared provider contract under explicit supervision;
- **stage/install** create host-owned lifecycle state;
- **enable** binds operational scope;
- **disable** removes operational availability without erasing historical evidence.

Conformance does not equal trust, installation, enablement, or permission.

> **Deep Plug, narrow subject. Wide workflow, Tether.**

## 8. Provider execution and transport

The host-provider architecture separates semantic capability identity from the transport used to reach a provider:

```text
Tethers semantic Socket
    -> protocol binding
    -> transport
    -> provider
```

The reference architecture includes MCP stdio binding material. MCP is therefore a useful provider protocol, not the place where Tethers delegates its language semantics, policy, replay rules, or evidence model.

The provider remains untrusted at the protocol boundary. Host-side execution verifies the identities and contracts that matter instead of trusting whatever a provider advertises live.

## 9. Together and bounded physical concurrency

The surface language can explicitly declare independent Actions with `together`.

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

The host may overlap Together member provider calls physically.

The accepted concurrency work includes physical provider overlap, bounded active concurrency, deterministic admission when capacity frees, truthful terminalisation of already-running work after trusted-state failures, and semantic Trail position separate from physical completion order.

The useful result is simple:

> **Concurrency may change when work happens. It must not silently change what the program means.**

## 10. Replay, durable intent, and uncertainty

Tethers treats externally significant execution as something that must survive awkward failure boundaries.

Durable intent and replay state prevent a crash, repeated request, or lost response from casually becoming a duplicate external effect.

The host distinguishes states such as:

- completed success;
- completed failure;
- uncertain post-invocation state;
- approval required;
- replay requiring manual resolution;
- persistence unavailable;
- unattempted work.

A timeout or lost final response after invocation is not automatically a safe retry.

> **No automatic retry unless idempotency is proved end to end for the relevant contract.**

This is intentionally more conservative than agent loops that treat every failure-looking response as another opportunity to “try again.”

## 11. Result Anchors and multi-step behaviour

Known provider outcomes may produce standard Result Anchors:

```text
capability.succeeded
capability.failed
capability.uncertain
```

These carry causal evidence and may wake later Tethers.

Generated Result Anchors enter a host-owned FIFO queue. The queue is intentionally serial at the evaluation level, with no recursive immediate re-entry and stable admission order.

This is separate from Together provider concurrency. Tethers can overlap independent provider calls inside a group while processing generated follow-up events through a stable causal queue.

A provider result is also not automatically equivalent to an independent observation of the outside world. The architecture can preserve that distinction instead of pretending that “command accepted” and “physical state confirmed” are the same fact.

## 12. Trail

The Trail is not decorative logging.

It is causal evidence shared across deterministic evaluation and effectful host execution.

It can record reception, evaluation, planning, semantic Action/group position, authority decisions, durable intent, provider attempt, result/failure/uncertainty, group join, replay identity, and Result Anchor correlation.

Pure deterministic Core entries remain independent of wall-clock time. Host execution entries may include timestamps because the host is the effectful runtime boundary.

The 0.6 release line also includes a bounded receipt projection over validated Trail evidence for agent-friendly inspection without introducing a second persistence store.

## 13. Agent-facing 0.6 surface

The published Tethers 0.6 practical release adds a machine-oriented front door so an unfamiliar agent does not need private knowledge of the host.

The tagged release source includes surfaces for:

```text
describe the host
    -> list trusted Capabilities
    -> inspect exact Capability contracts
    -> inspect installed Plug state
    -> preview proposed work without effects
    -> run admitted work
    -> inspect bounded Trail receipts
```

It also includes agent-oriented workspace/text/hash/patch capabilities, structured Git capabilities, bounded argv-only process execution, named verification checks, and the deterministic `tethers-bench` verification tool.

For exact 0.6 commands and source, use the [`tethers-v0.6.0` release tag](https://github.com/matthewjameswatkins1978-cyber/tethers-lang/releases/tag/tethers-v0.6.0).

That source/release mismatch should be treated as repository hygiene, not hidden by documentation.

## 14. Portable workbench

`tethers-0.1/portable-rust/` is a deliberately smaller, self-contained authority façade.

It answers local policy questions:

```text
request -> ALLOW / ASK / DENY
```

and does not execute the action itself.

It exists because a small deterministic authority binary is useful for scripts and agents even when the full Tethers host/runtime is not being used.

Do not infer the full platform's limits from the portable façade.

## 15. Security boundary

The reference host has strong trust machinery, but supervised Plug execution is not a hostile-code sandbox.

Current security value comes from strict package and manifest validation, host-owned provider identity, reviewed capability manifests, live binding revalidation, scope evidence, explicit policy, approval boundaries, durable intent, replay protection, protocol/output validation, bounded provider deadlines, redacted evidence, and process supervision.

Those controls do not prove that arbitrary native provider code is isolated from the machine's filesystem, network, credentials, dynamic libraries, or operating-system APIs.

See [`SECURITY.md`](SECURITY.md).

## 16. Version map

Tethers currently carries several version axes:

| Axis | Current meaning |
| --- | --- |
| Human Tether language semantics | `0.1` |
| Reference-host Cargo package | `0.2.2` compatibility/package axis |
| Portable workbench | `0.2.2` |
| Public Plug authoring milestone | `0.3` complete |
| Together/concurrency milestone | `0.4` complete |
| Practical product release | `0.6` |
| Latest public GitHub tag | `tethers-v0.6.0` |

The current 0.6 release asset filenames use the `tethers-0.6.0-*` form. The
language and compatibility package versions are related but not interchangeable.

## 17. Where Tethers is strongest

Tethers is especially useful where these distinctions matter:

- intent versus permission;
- trusted contract versus provider advertising;
- scoped capability versus unrestricted tool access;
- proposed Plan versus actual execution;
- first attempt versus replay;
- success versus failure versus uncertainty;
- semantic order versus physical completion order;
- provider result versus later external observation.

If none of those distinctions matter for a task, ordinary code may be the better answer. Tethers is not intended to turn every function call into a ceremony.

## 18. Documentation authority

Use documents by purpose rather than treating every old roadmap as current truth:

1. [`../README.md`](../README.md) - product positioning and front door.
2. [`../QUICKSTART.md`](../QUICKSTART.md) - working mental model.
3. [`CONSTITUTION.md`](CONSTITUTION.md) - enduring design principles.
4. [`../tethers-0.1/SPEC.md`](../tethers-0.1/SPEC.md) - exact Human Tether semantics.
5. [`DECISIONS.md`](DECISIONS.md) - accepted architecture decisions.
6. [`CAPABILITY_BRIDGE.md`](CAPABILITY_BRIDGE.md) - trusted capability and manifest bridge.
7. [`SECURITY.md`](SECURITY.md) - current security claims and limits.
8. [`PLUG_AUTHORING.md`](PLUG_AUTHORING.md) - public Plug author contract.
9. [`CURRENT_GOAL.md`](CURRENT_GOAL.md) and [`PROJECT_DASHBOARD.md`](PROJECT_DASHBOARD.md) - living direction and status.

`ROAD_TO_*`, `worker-notes/`, `review/`, `perf/`, and foundation-pass documents are implementation history and evidence. Their old checkpoint statements should remain historical rather than being silently rewritten into present tense.
