# Tethers quick start

This guide teaches the **whole Tethers mental model** first, then shows the smaller portable workbench.

If you remember only one sentence, use this one:

> **A Tether deterministically proposes typed work; the host decides whether that work may run, executes approved Capabilities through Plugs, and records what actually happened.**

## 1. Start with a Tether

A Tether is deliberately small:

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

Read it as:

```text
when folder.received_file happens
and the supplied immutable Facts say this is an invoice PDF
request file.move with these explicit inputs
```

The OCaml engine does not read the filesystem to discover those Facts. The host supplies the event, Facts, Tether source, and approved Capability projections as explicit input.

## 2. A Plan is not permission

Tethers Core parses, validates, evaluates, and plans.

It does **not** grant itself permission and does not secretly perform the external effect.

The boundary is:

```text
Tether
  -> deterministic Plan
  -> host policy + scope + trust
  -> approved execution
```

Keep this phrase in your head:

```text
Schemas describe.
Policies authorise.
Hosts enforce.
Trails record.
```

## 3. Capabilities describe the operations

A Tether Action names a Capability.

A trusted Capability manifest can carry the exact contract an integration needs:

- name and version;
- input and output schemas;
- Effects;
- scope;
- reversibility and determinism;
- idempotency;
- confirmation requirements;
- timeout/retry contract;
- provider identity and binding.

Application-specific behaviour belongs behind Capabilities and Plugs, not in Tethers Core.

A file tool, Git tool, PDF tool, AI model, email system, or physical device should therefore become a Capability set rather than a new Tethers language mode.

## 4. Plugs connect real systems

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

In particular:

> **Conformance is evidence, not permission.**

A conforming package is not automatically installed, enabled, trusted for every resource, or allowed to execute every call.

See [`docs/PLUG_AUTHORING.md`](docs/PLUG_AUTHORING.md) for the full authoring contract.

## 5. Independent work can be declared with `together`

The current 0.1 surface includes an explicit fan-out/join construct:

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

The three group members are semantically independent.

The accepted reference runtime may overlap their provider invocations physically, with bounded concurrency. The later Action waits for the group join.

What physical scheduling must **not** change:

- source meaning;
- Action identity;
- group membership;
- semantic member order;
- replay identity;
- Trail semantic position;
- join meaning;
- first-non-success selection.

That is why Tethers can have concurrency without letting race timing become language semantics.

## 6. Results become visible events

A successful provider call is not silently fed into hidden mutable program state.

Known outcomes can produce standard Result Anchors:

```text
capability.succeeded
capability.failed
capability.uncertain
```

A Result Anchor carries causal identities and may wake another Tether.

The host drains generated Result Anchors through a stable FIFO event queue rather than recursively re-entering evaluation on the current stack.

This gives multi-step behaviour a visible shape:

```text
external event
    -> Tether A
    -> Capability call
    -> Result Anchor
    -> Tether B
```

For a friendly worked example, read [`docs/BUNNY_AND_COOKIES.md`](docs/BUNNY_AND_COOKIES.md).

## 7. The Trail is part of the product

Tethers distinguishes:

- what event arrived;
- what Facts were supplied;
- what matched;
- what Plan was proposed;
- what authority decision was made;
- what durable intent was recorded;
- what provider was called;
- what result or uncertainty was observed;
- what Result Anchor was produced.

That causal evidence is the Trail.

A proposal is not recorded as an execution, and an uncertain call is not renamed as a clean failure merely because that would be easier to handle.

## 8. Try the portable workbench

The portable workbench is the easiest binary to try, but remember that it is a **small authority façade**, not the full host/runtime.

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

Portable decision exit codes are scriptable:

| Code | Decision |
| ---: | --- |
| `0` | `ALLOW` |
| `10` | `ASK` |
| `20` | `DENY` |

Invocation/configuration failures use separate codes. An operational error never means `ALLOW`.

## 9. Know which surface you are using

Tethers currently has several related surfaces:

### Human Tether language

Defined precisely by [`tethers-0.1/SPEC.md`](tethers-0.1/SPEC.md).

### OCaml Core

Typed semantic representation, validation, canonicalisation, and deterministic planning.

### Rust reference host

Trust, policy, scopes, Plug lifecycle, durable intent, replay, provider execution, Result Anchors, Trails, and bounded Together concurrency.

### Portable workbench

Small self-contained ALLOW / ASK / DENY authority tool for scripts and agents.

Do not infer the limits of the full platform from the portable workbench, and do not infer new user-facing syntax merely because Core has a richer internal vocabulary.

## 10. Where to go next

- [`README.md`](README.md) - the full project story.
- [`docs/PROJECT_OVERVIEW.md`](docs/PROJECT_OVERVIEW.md) - architecture and current implementation boundaries.
- [`tethers-0.1/SPEC.md`](tethers-0.1/SPEC.md) - exact language semantics.
- [`docs/PLUG_AUTHORING.md`](docs/PLUG_AUTHORING.md) - how to build a Plug.
- [`docs/SECURITY.md`](docs/SECURITY.md) - current trust and sandbox limits.
- [`docs/CONSTITUTION.md`](docs/CONSTITUTION.md) - enduring design principles.

## 11. Install the 0.5 bundle

Download the Windows x64 or Linux x64 musl bundle from the [0.5 release
record](docs/TETHERS_0_5_RELEASE.md), verify the adjacent SHA-256 file, and
extract it. The native host is under `bin/`; the small ALLOW / ASK / DENY
workbench is under `portable/`. The bundle includes the agent quickstart and
security manual.

The Windows bundle is reproducible locally:

```powershell
pwsh -NoProfile -File .\scripts\package-tethers-release.ps1 -Target windows-x64
```

Linux x64 musl packaging is performed by the pinned GitHub Actions workflow.
That is a CI verification claim, not a claim that a Windows machine is a Linux
build host.

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
