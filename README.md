<div align="center">
  <img src="assets/tethers-icon.png" alt="Tethers icon" width="160" />
  <h1>Tethers</h1>
  <p><strong>Deterministic execution control for AI, automation, tools, and services.</strong></p>
  <p><strong>Let AI improvise about intent. Keep real-world execution typed, scoped, permissioned, and provable.</strong></p>
  <p>
    <a href="QUICKSTART.md">Quick start</a>
    ·
    <a href="docs/PROJECT_OVERVIEW.md">How it works</a>
    ·
    <a href="tethers-0.1/SPEC.md">Language specification</a>
    ·
    <a href="docs/PLUG_AUTHORING.md">Build a Plug</a>
    ·
    <a href="https://github.com/matthewjameswatkins1978-cyber/tethers-lang/releases/tag/tethers-v0.5.8">Tethers 0.5 release</a>
  </p>
</div>

AI is very good at deciding what it wants to do. The dangerous part begins one millisecond later, when that decision becomes a file edit, a Git operation, a network call, a deployment, a message, a database mutation, or some other real effect.

Tethers is the deliberately exact layer between **wanting** and **doing**.

```text
AI / application intent
          |
          v
   deterministic Plan
          |
          v
Capability contract + policy + scope
          |
          v
    approved execution
          |
          v
 result / uncertainty / causal Trail
```

A model can change. A prompt can change. A provider can change. The execution contract does not have to become guesswork with them.

> **Make things happen. Keep the receipts.**

## Why Tethers?

Most automation stacks blur several different questions together:

```text
What do we want to happen?
What operation actually exists?
Is this caller allowed to do it here?
Did we really attempt it?
Did it definitely succeed?
What should happen next?
```

Tethers keeps those questions separate on purpose.

A **Tether** deterministically turns an event and immutable Facts into a typed Action Plan. **Capabilities** define the operations that exist. **Policies and scopes** decide what is allowed. The **host** executes admitted work through **Plugs**. **Result Anchors** make outcomes visible to later behaviour. The **Trail** records the causal story.

That separation gives Tethers a few useful properties that are unusually hard to bolt on after the fact:

- **Deterministic meaning.** The same complete semantic input produces the same Plan. Core does not secretly consult time, randomness, the filesystem, the network, or a model.
- **Typed capabilities instead of wishful tool calls.** Inputs, outputs, Effects, versions, provider bindings, confirmation rules, retry contracts, and scope can be part of the trusted operation contract.
- **Authority is not intent.** A planner can request an operation, but cannot grant itself permission to perform it.
- **Fail closed.** Missing trust, malformed data, ambiguous versions, scope mismatch, unavailable durable state, and unsafe uncertainty do not quietly become permission.
- **Human approval is first-class.** Work can stop at an explicit approval boundary instead of hiding a confirmation prompt inside an agent loop.
- **Uncertainty stays honest.** "No final response" is not rewritten into "definitely failed, retry it." Tethers distinguishes failure from post-invocation uncertainty.
- **Concurrency without semantic races.** `together` can overlap independent provider calls while preserving deterministic member order, join meaning, replay identity, and Trail position.
- **Evidence is part of the product.** The Trail is not decorative logging. It distinguishes proposal, authority, intent, attempt, result, uncertainty, replay, and follow-up causality.
- **Providers stay replaceable.** Application-specific behaviour belongs in Plugs and Capability contracts instead of leaking into the language core.

The result is not another agent with opinions. It is a small, boring-in-the-best-way execution substrate underneath agents and automation.

## Where Tethers fits

### Under AI agents, not instead of them

Tethers does not contain an LLM loop and does not compete with coding agents, assistants, planners, or orchestration frameworks.

```text
agent decides what it wants
          |
          v
       Tethers
          |
          | deterministic meaning
          | trusted capabilities
          | policy + scope
          | replay + evidence
          v
      real systems
```

The AI is free to reason creatively. Tethers becomes strict only where creativity would be a liability: contracts, authority, consequential execution, replay, and evidence.

### Beside MCP, not as a replacement for it

MCP is useful for exposing tools and resources. Tethers addresses a different layer: what a requested operation *means*, whether it is authorised in this exact context, how it is scoped, how execution is replayed safely, and what evidence survives afterwards.

The reference architecture can bind providers through MCP stdio while keeping Tethers semantics, trust, and authority above the transport.

### More structured than a shell escape hatch

A shell command can do almost anything, which is precisely the problem when the caller is autonomous. Tethers prefers named semantic Capabilities with reviewed contracts and bounded scope. Generic process execution can still exist where justified, but it does not have to be the universal escape valve.

### More than a policy engine

The small portable workbench *is* deliberately just a deterministic authority boundary. Full Tethers goes further: language semantics, planning, trusted Capability resolution, Plug lifecycle, execution, durable intent, replay, Result Anchors, bounded concurrency, and causal Trails.

## When should I use it?

Tethers earns its keep when consequences matter more than saving a few lines of glue code.

Good fits include AI coding workers that need bounded repository access, local automation that must explain why it changed something, tool ecosystems where providers can drift, workflows where duplicate effects are dangerous, operations that need explicit human approval, and multi-step behaviour where success, failure, and uncertainty must lead to different visible next events.

It is probably **not** the right tool for ordinary internal algorithms, rendering code, byte manipulation, trivial scripts with no meaningful authority boundary, or anything where the execution contract would be more complicated than the problem itself.

## A Tether is deliberately small

```tethers
tether "Morning brief"

anchor
    morning.started

when
    ready is true

do
    together
        weather.fetch
            location: anchor.location

        calendar.fetch
            day: anchor.day

        email.fetch
            account: "main"

    brief.compose
        format: "short"
```

Read it literally: when `morning.started` arrives and the supplied Facts say `ready is true`, request three independent typed operations, join them, then request `brief.compose`.

The engine does not improvise hidden state. The host does not treat the Plan as permission. Physical completion order cannot silently redefine the program.

> **Concurrency may change when work happens. It must not change what the program means.**

## Capabilities and Plugs

A **Capability** is a versioned typed operation. A trusted manifest can describe:

- canonical name and version;
- title and description;
- strict input and output schemas;
- Effects;
- permission scope;
- reversibility and determinism;
- idempotency;
- confirmation policy;
- timeout and retry contract;
- provider identity and protocol binding.

A **Plug** packages a provider and one or more related Capabilities. The public lifecycle is intentionally explicit:

```text
author
  -> pack
  -> inspect
  -> conform
  -> stage
  -> install
  -> enable with scope
  -> execute
```

Those steps are different because they prove different things. A Plug can conform to its declared protocol without thereby gaining trust, installation, enablement, credentials, unrestricted scope, or permission to execute.

> **Conformance is evidence, not permission.**

The reference Plug programme includes benign examples and adversarial providers specifically to keep that boundary honest.

## Results are events, not hidden mutable state

Known provider outcomes can produce standard Result Anchors:

```text
capability.succeeded
capability.failed
capability.uncertain
```

These are causal events with identities. They can wake later Tethers through a host-owned FIFO event queue instead of recursively smuggling action results back into hidden workflow state.

That gives multi-step behaviour a visible shape:

```text
external event
    -> Tether A
    -> Capability call
    -> Result Anchor
    -> Tether B
```

A provider reporting success is also not automatically the same thing as an independent observation of the outside world. Tethers keeps request, provider result, and later observation distinct. [`docs/BUNNY_AND_COOKIES.md`](docs/BUNNY_AND_COOKIES.md) is the friendliest worked explanation of that idea.

## The portable authority workbench

Sometimes you do not need the full host. You just need one small question answered deterministically:

```text
may this requested action proceed?
```

The portable `tethers` workbench returns:

```text
ALLOW
ASK
DENY
```

and never performs the requested action itself.

```powershell
.\tethers.exe doctor --json
.\tethers.exe check --action git.status --json
.\tethers.exe check --action git.push --explain
.\tethers.exe check --action git.force_push --json
```

That makes it useful as a tiny fail-closed authority boundary inside an existing agent, script, CI worker, or local workbench. See [`tethers-0.1/portable-rust/README.md`](tethers-0.1/portable-rust/README.md).

## The full 0.5 product surface

The published Tethers 0.5 practical release adds the agent-facing surfaces needed to use the wider platform without bespoke tribal knowledge, including read-only host discovery, trusted Capability listing and inspection, installed Plug inspection, side-effect-free preview, bounded Trail receipts, agent-oriented workspace and coding reference Plugs, and the deterministic `tethers-bench` verification tool.

For commands and source that match the published release, use:

- [Tethers 0.5 release (`tethers-v0.5.8`)](https://github.com/matthewjameswatkins1978-cyber/tethers-lang/releases/tag/tethers-v0.5.8)
- [0.5 Agent Quickstart](https://github.com/matthewjameswatkins1978-cyber/tethers-lang/blob/tethers-v0.5.8/docs/AGENT_QUICKSTART.md)
- [0.5 release notes](https://github.com/matthewjameswatkins1978-cyber/tethers-lang/blob/tethers-v0.5.8/docs/TETHERS_0_5_RELEASE.md)

### Repository state note

The latest published GitHub release is Tethers 0.5, tagged `tethers-v0.5.8`. The default `main` branch currently points to an earlier September 1 checkpoint and does not contain every 0.5 agent-facing command present in the tagged release source. Until that ancestry is reconciled, use the release tag when reproducing the published 0.5 bundle.

This documentation pass deliberately does not pretend otherwise.

## Version map

Tethers currently has several version axes. They describe different things:

| Layer | Current meaning |
| --- | --- |
| Human Tether language semantics | `0.1` specification |
| Reference host Cargo package | `0.2.2` compatibility/package axis |
| Portable authority workbench | `0.2.2` |
| Public Plug-authoring milestone | `0.3` complete |
| Together/concurrency milestone | `0.4` complete |
| Practical product release | Tethers `0.5`, latest tag `tethers-v0.5.8` |

The 0.5 release assets currently use `tethers-0.5.0-*` filenames while the GitHub tag is `tethers-v0.5.8`. That naming is historical release machinery, not a claim that all of these axes are interchangeable. Consolidating version presentation is a product-hygiene task, not a semantic change.

## Security posture

Tethers has serious trust and execution machinery, but supervised Plug execution is **not a hostile-code sandbox**.

The host can verify trusted manifests, provider bindings, scopes, durable intent, replay state, output schemas, deadlines, and causal evidence. None of that magically prevents malicious native provider code from accessing the machine through ordinary operating-system facilities.

Read [`docs/SECURITY.md`](docs/SECURITY.md) before treating third-party providers as trusted code.

## Repository map

- `tethers-0.1/engine-ocaml/` - deterministic parser, evaluator, typed Core, validation, canonicalisation, planning, and protocol tools.
- `tethers-0.1/host-rust/` - trusted host, policy, scope, Plug lifecycle, replay, Trail, provider execution, and Together runtime.
- `tethers-0.1/portable-rust/` - small self-contained ALLOW / ASK / DENY workbench.
- `tethers-0.1/protocol/` - capability manifests, protocol cases, fixtures, and transcripts.
- `reference-plugs/` - reference and adversarial Plug material in checkpoints that contain it.
- `docs/` - architecture, security, authoring, current state, historical roadmaps, and implementation evidence.

## Read next

If you want to **understand the idea**, read [`QUICKSTART.md`](QUICKSTART.md) and [`docs/PROJECT_OVERVIEW.md`](docs/PROJECT_OVERVIEW.md).

If you want to **integrate an AI or script with the small authority layer**, read [`tethers-0.1/portable-rust/AI-INTEGRATION.md`](tethers-0.1/portable-rust/AI-INTEGRATION.md).

If you want to **build capabilities**, read [`docs/PLUG_AUTHORING.md`](docs/PLUG_AUTHORING.md).

If you care about **exact language semantics**, read [`tethers-0.1/SPEC.md`](tethers-0.1/SPEC.md) and [`docs/CONSTITUTION.md`](docs/CONSTITUTION.md).

If you are evaluating Tethers for consequential execution, read [`docs/SECURITY.md`](docs/SECURITY.md) before trusting provider code.

> **AI can be probabilistic. The boundary where it changes the world does not have to be.**
