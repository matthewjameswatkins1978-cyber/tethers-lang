# Tethers quick start

Tethers is easiest to understand if you keep one separation in your head:

> **AI or an application decides what it wants. Tethers turns consequential work into deterministic Plans, explicit authority, bounded execution, and evidence.**

This guide teaches the whole mental model first, then the smaller portable workbench.

## 1. Start with the problem Tethers solves

A normal agent tool call can collapse several different things into one step:

```text
reason -> call tool -> hope the permission was right -> hope the result means what we think
```

Tethers separates them:

```text
intent
  -> deterministic Plan
  -> trusted Capability contract
  -> policy + scope
  -> execution
  -> result / uncertainty
  -> Trail
```

That is the core value. The agent can stay probabilistic. The boundary where it changes something real becomes explicit and inspectable.

## 2. A Tether is a small behavioural rule

```tethers
tether "Sort received invoices"

anchor
    folder.received_file

when
    file.type is "pdf"
    and file.name contains "invoice"

do
    file.move
        source_path: anchor.source_path
        destination_path: anchor.destination_path
```

Read it literally:

```text
when folder.received_file happens
and the supplied immutable Facts say this is an invoice PDF
request file.move with these explicit inputs
```

The deterministic engine does not secretly read the filesystem to discover those Facts. The host supplies the event, Facts, Tether source, and approved Capability projections as explicit input.

## 3. A Plan is not permission

Tethers Core parses, validates, evaluates, and plans.

It does **not** grant itself permission and does not secretly perform the external effect.

```text
Tether
  -> deterministic Plan
  -> host policy + scope + trust
  -> approved execution
```

Keep this phrase nearby:

```text
Capabilities describe.
Policies authorise.
Hosts enforce.
Trails record.
```

The planner cannot approve its own work.

## 4. Capabilities describe the operations

A Tether Action names a Capability.

A trusted Capability manifest can carry the exact contract an integration needs:

- name and version;
- strict input and output schemas;
- Effects;
- scope;
- reversibility and determinism;
- idempotency;
- confirmation requirements;
- timeout and retry contract;
- provider identity and binding.

Application-specific behaviour belongs behind Capabilities and Plugs, not in Tethers Core.

A file tool, Git tool, PDF tool, AI model, email system, or physical device should therefore become a Capability set rather than a new Tethers language mode.

## 5. Plugs connect real systems

A Plug packages a provider and its Capability manifests.

The public Plug journey is intentionally explicit:

```text
author source
    -> plug pack
    -> .tetherplug
    -> plug inspect
    -> plug conform
    -> stage
    -> install
    -> enable with scope
```

These stages are not aliases for one another.

> **Conformance is evidence, not permission.**

A conforming package is not automatically installed, enabled, trusted for every resource, or allowed to execute every call.

See [`docs/PLUG_AUTHORING.md`](docs/PLUG_AUTHORING.md) for the authoring contract.

## 6. Independent work can be declared with `together`

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

The three group members are semantically independent. A supported runtime may overlap their provider invocations physically, with bounded concurrency. The later Action waits for the group join.

Physical scheduling must not change source meaning, Action identity, group membership, semantic member order, replay identity, Trail position, join meaning, or first-non-success selection.

That is the interesting bit: Tethers can gain useful concurrency without turning race timing into language semantics.

## 7. Results become visible events

A provider result is not silently poured back into hidden mutable workflow state.

Known outcomes can produce Result Anchors such as:

```text
capability.succeeded
capability.failed
capability.uncertain
```

A Result Anchor carries causal identities and may wake another Tether.

```text
external event
    -> Tether A
    -> Capability call
    -> Result Anchor
    -> Tether B
```

The host drains generated Result Anchors through a stable FIFO event queue rather than recursively re-entering evaluation on the current stack.

For a friendly worked example, read [`docs/BUNNY_AND_COOKIES.md`](docs/BUNNY_AND_COOKIES.md).

## 8. The Trail is part of the product

Tethers distinguishes:

- what event arrived;
- what Facts were supplied;
- what matched;
- what Plan was proposed;
- what authority decision was made;
- what durable intent was recorded;
- what provider was called;
- what result or uncertainty was observed;
- what follow-up Result Anchor was produced.

That causal evidence is the Trail.

A proposal is not recorded as an execution. An uncertain call is not renamed as a clean failure just because a retry would be convenient.

## 9. Try the portable workbench

The portable workbench is the easiest binary to try. It is a **small authority façade**, not the full host/runtime.

It answers:

```text
may this requested action proceed?
```

with:

```text
ALLOW
ASK
DENY
```

Windows:

```powershell
.\tethers.exe version --json
.\tethers.exe doctor --json
.\tethers.exe check --action git.status --json
.\tethers.exe check --action git.push --explain
.\tethers.exe check --action git.force_push --json
```

Linux:

```bash
./tethers version --json
./tethers doctor --json
./tethers check --action git.status --json
./tethers check --action git.push --explain
./tethers check --action git.force_push --json
```

A useful default coding policy can therefore say:

```text
git.status      -> ALLOW
git.push        -> ASK
git.force_push  -> DENY
```

The caller still performs the operation. The workbench only makes the authority decision.

Portable decision exit codes are scriptable:

| Code | Decision |
| ---: | --- |
| `0` | `ALLOW` |
| `10` | `ASK` |
| `20` | `DENY` |

Invocation and configuration failures use separate codes. An operational error never means `ALLOW`.

## 10. The published 0.5 host goes further

The Tethers 0.5 practical release exposes the wider platform to agents through machine-readable discovery and inspection surfaces. The tagged release source includes:

```text
tethers describe --json
tethers capability list --host-data-root <absolute-host-data-root> --json
tethers capability inspect <name> --host-data-root <absolute-host-data-root> --version <version> --json
tethers plug show --host-data-root <absolute-host-data-root> --installed-id <id> --json
tethers preview --config <config.json> --engine <engine> --input <input.json>
tethers trail --trail <trail.jsonl> --execution-id <id> --receipt
```

Discovery and preview are deliberately side-effect-free. They do not start a provider, grant authority, or pretend that a preview is an execution.

Use the source and manuals attached to the published release when following those commands:

- [Tethers 0.5 release (`tethers-v0.5.8`)](https://github.com/matthewjameswatkins1978-cyber/tethers-lang/releases/tag/tethers-v0.5.8)
- [0.5 Agent Quickstart](https://github.com/matthewjameswatkins1978-cyber/tethers-lang/blob/tethers-v0.5.8/docs/AGENT_QUICKSTART.md)

The implementation ancestry currently reachable from `main` is still based on an earlier September 1 checkpoint, so not every 0.5 agent-facing command is present there. That is a repository-state issue, not a reason to blur the difference in the documentation.

## 11. Know which surface you are using

### Human Tether language

Defined precisely by [`tethers-0.1/SPEC.md`](tethers-0.1/SPEC.md). The language version is a semantic axis, not the overall product release number.

### OCaml Core

Typed semantic representation, validation, canonicalisation, deterministic planning, and program identity.

### Rust reference host

Trust, policy, scopes, Plug lifecycle, durable intent, replay, provider execution, Result Anchors, Trails, and bounded Together concurrency.

### Portable workbench

Small self-contained ALLOW / ASK / DENY authority tool for scripts and agents. It remains separately versioned at 0.2.2.

Do not infer the limits of the full platform from the portable workbench, and do not infer new user-facing syntax merely because Core has a richer internal vocabulary.

## 12. Where to go next

- [`README.md`](README.md) - why Tethers exists and where it fits.
- [`docs/PROJECT_OVERVIEW.md`](docs/PROJECT_OVERVIEW.md) - architecture and implementation boundaries.
- [`tethers-0.1/SPEC.md`](tethers-0.1/SPEC.md) - exact language semantics.
- [`docs/PLUG_AUTHORING.md`](docs/PLUG_AUTHORING.md) - how to build a Plug.
- [`docs/SECURITY.md`](docs/SECURITY.md) - current trust and sandbox limits.
- [`docs/CONSTITUTION.md`](docs/CONSTITUTION.md) - enduring design principles.
- [`tethers-0.1/portable-rust/AI-INTEGRATION.md`](tethers-0.1/portable-rust/AI-INTEGRATION.md) - embedding the small authority workbench.

The shortest accurate mental model is:

```text
Events wake Tethers.
Facts make decisions explicit.
Tethers propose Plans.
Capabilities describe operations.
Policies and scopes constrain authority.
Plugs connect providers.
Hosts execute.
Result Anchors continue the story.
Trails keep the receipts.
```
