<div align="center">
  <img src="assets/tethers-icon.png" alt="Tethers icon" width="160" />
  <h1>Tethers</h1>
  <p><strong>Deterministic execution control for AI, automation, tools, and services.</strong></p>
  <p><strong>Let AI improvise about intent. Keep real-world execution typed, scoped, permissioned, and provable.</strong></p>
  <p>
    <a href="QUICKSTART.md">Quick start</a>
    ·
    <a href="docs/AGENT_QUICKSTART.md">Agent quickstart</a>
    ·
    <a href="docs/PROJECT_OVERVIEW.md">How it works</a>
    ·
    <a href="tethers-0.1/SPEC.md">Language specification</a>
    ·
    <a href="docs/PLUG_AUTHORING.md">Build a Plug</a>
    ·
    <a href="https://github.com/matthewjameswatkins1978-cyber/tethers-lang/releases/tag/tethers-v0.6.0">Tethers 0.6 release</a>
  </p>
</div>

AI is very good at deciding what it wants to do. The dangerous part begins one millisecond later, when that decision becomes a file edit, a Git operation, a network call, a deployment, a message, a database mutation, or some other real effect.
**Tethers is the execution boundary between an AI's intentions and your computer.**

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
## Tethers 0.7.0

- Tethers product version: 0.7.0
- Status: release candidate implementation
- Language semantics: 0.1
- Release notes: [`docs/releases/v0.7.0.md`](docs/releases/v0.7.0.md)

Tethers 0.7 keeps the language deliberately small and adds a discoverable
host-owned Agent Core for ordinary local repository work. Core plans; hosts
authorise and execute; providers perform effects; Trails record what happened.

Production Core is tested with OCaml suites and reference oracles. The separate
Rocq research tree explores machine-checked specifications; it is not a claim
that production Tethers is formally verified or extracted from Rocq.

## Cold start

From a repository, a new AI can discover the boundary with:

```powershell
tethers --help
tethers init
tethers doctor
tethers capability list
tethers capability inspect workspace.read
```

The Agent Core exposes bounded `workspace`, structured local `git`, and direct
`exec` operations. Workspace replacement requires an expected preimage hash;
Git remote mutation is not exposed; exec never inserts a shell. `threadmoth`
is an optional explicit integration and is never hidden inside `workspace.replace`.

The normal Windows full-runtime package contains `tethers.exe` and its sibling
`tethers-engine.exe`; Rust, Cargo, OCaml, opam and Dune are development tools,
not end-user prerequisites. A portable host can inspect and plan without
claiming the full local execution surface.

## Repository Map

Tethers uses a layered set of authoritative and operational documents:

- `docs/CONSTITUTION.md` records the enduring Tethers language principles.
- `tethers-0.1/SPEC.md` defines the current precise 0.1 language and protocol
  semantics.
- [`docs/architecture/TETHERS_LANTERN_KEEPER_CANONICAL_ARCHITECTURE.md`](docs/architecture/TETHERS_LANTERN_KEEPER_CANONICAL_ARCHITECTURE.md)
  is the joint architectural contract and build foundation for Tethers and
  Lantern Keeper.
- `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md` defines how senior engineers and AI
  agents use OCaml, Rust, PowerShell, protocol formats, and future implementation
  languages.
- `docs/MCP_PLAN.md` records the approved post-0.1 direction for an OCaml Tethers
  MCP interface.
- `docs/OCAML_GUIDE_FOR_AGENTS.md` gives version-specific OCaml environment and
  project guidance.
- `docs/PROJECT_CONTROL.md` defines task ownership, evidence, worker notes, and
  review.
- `docs/AGENT_WORKFLOW.md` defines the current **Gorilla Coding 🦄** route.
- `docs/CLINE_HANDOFF.md` is the current worker-neutral Gorilla handoff guide
  (historical filename).
- `docs/TASK_PACKET_TEMPLATE.md` and `docs/WORKER_NOTE_TEMPLATE.md` define the
  two durable sides of each implementation handoff.
- `docs/PROJECT_DASHBOARD.md` is Matthew's short current-state view.
- `docs/releases/v0.7.0.md` records the current 0.7.0 release-candidate gates;
  older `ROAD_TO_0_2.md` material is retained as historical project context.

Current operating route:

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

Canonical Format V2 gives validated Core programs stable semantic identity that is independent of irrelevant raw identifiers and representation order. The implementation includes independent canonicalisation paths and differential evidence.

The 0.6 release carries Rocket V3 as an exact portfolio seam around that frozen
identity. Common success paths use the proven path solver; typed refinement and
the exact V2 search remain available for broader shapes, and the exhaustive
reference engine remains available for bounded differential checks. Backend
selection changes runtime and evidence counters only, never the payload or
digest.

The repository also includes `tethers-bench`, a deterministic Rocket
benchmarker with human and machine-readable output, environment context,
backend/resource counters, and before/after comparison. It is intended to be
an explicit AI-toolbelt verification check, not an ad hoc release script. See
[`docs/TETHERS_BENCHMARKER.md`](docs/TETHERS_BENCHMARKER.md).

Core's vocabulary is intentionally richer than the current human-facing 0.1 syntax. Do not assume that every Core structure is already exposed as source syntax or supported on every runtime bridge. The current public language surface is defined by [`tethers-0.1/SPEC.md`](tethers-0.1/SPEC.md).

## Result Anchors and uncertainty

Provider execution is not collapsed into a boolean.

Known runtime results are represented distinctly, including success, failure, and uncertainty. Successful, failed, and uncertain provider outcomes may produce standard Result Anchors such as:

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

For the native host and installed trusted Plugs, begin with the zero-knowledge
discovery surface:

```text
tethers describe --json
tethers capability list --host-data-root <absolute-host-data-root> --json
tethers capability inspect <name> --host-data-root <absolute-host-data-root> --version <version> --json
```

These commands are read-only and expose trusted contracts, scopes, provider
bindings, and availability without starting providers. See the
[agent quickstart](docs/AGENT_QUICKSTART.md) for the complete discovery path.

The native host provides `plan` as the stable machine-facing, side-effect-free
Plan surface, while `preview` remains the compatible human-oriented preview.
Both evaluate Core without requesting authority, invoking a provider, or
creating a Trail execution entry. `trail --receipt` provides a bounded causal
projection over validated Trail entries.

## Download the portable workbench

For the full 0.5 host bundle, download the platform asset from the
[Tethers 0.6 release](docs/TETHERS_0_6_RELEASE.md). It contains the native
host, the smaller portable workbench, agent-facing manuals, and SHA-256
evidence. The portable workbench remains separately versioned at 0.2.2 for
compatibility.

## Version map

Tethers currently has several version axes. They describe different things:

| Layer | Current meaning |
| --- | --- |
| Human Tether language/protocol semantics | `0.1` specification |
| Rust reference-host package version | `0.2.2` |
| Portable workbench | `0.2.2` |
| Public Plug-authoring programme | `0.3` milestone complete and integrated |
| Together/concurrency programme | `0.4` milestone complete and integrated |
| Practical release line | `0.6` — Plan, execution, replay, Trail proof, and Agent Essentials |

The 0.6 source tree includes three starter Tether Set examples under
[`examples/tether-sets`](examples/tether-sets). They use the existing Tether
language and runtime configuration; they do not introduce a second Set
semantic or permission model.

The 0.6 release assets use `tethers-0.6.0-*` filenames and the GitHub tag is
`tethers-v0.6.0`. The language and compatibility package versions remain
separate axes by design.

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
