<div align="center">
  <img src="assets/tethers-icon.png" alt="Tethers icon" width="160" />
  <h1>Tethers</h1>
  <p><strong>A small deterministic coordination language and capability platform for software, tools, services, and AI.</strong></p>
  <p>
    <a href="QUICKSTART.md">Quick start</a>
    ·
    <a href="docs/AGENT_QUICKSTART.md">Agent quickstart</a>
    ·
    <a href="tethers-0.1/SPEC.md">Language specification</a>
    ·
    <a href="docs/PLUG_AUTHORING.md">Plug authoring</a>
  </p>
</div>

Tethers is built around a simple idea:

> **Describe useful behaviour clearly, keep authority separate from intent, and leave trustworthy evidence of what actually happened.**

A Tether connects an event and immutable Facts to typed Capability requests. The deterministic OCaml engine decides what the program means and produces a Plan. The Rust host resolves trusted Capabilities, applies policy and scope, records durable intent, executes approved work through Plug providers, validates results, and records the causal Trail.

The portable `tethers` workbench is one deliberately smaller surface of the same project. It answers local authority questions with `ALLOW`, `ASK`, or `DENY`. It is useful, but it is not the whole of Tethers.

## The whole machine

```text
                         deterministic meaning
Event + Facts + Tether  -------------------------->  Tethers Core
                                                        |
                                                        v
                                                     Action Plan
                                                        |
                                      schemas describe | policies authorise
                                                        v
                                                 Tethers Host
                                         / trust / scope / replay /
                                        / durable intent / execution /
                                               |              |
                                               v              v
                                           Capability      Trail
                                               |
                                               v
                                              Plug
                                               |
                                               v
                                            Provider
                                               |
                                               v
                                         outside system
                                               |
                                               v
                                          Result Anchor
                                               |
                                               +----> may wake another Tether
```

The important boundaries are deliberate:

```text
Tethers language says what behaviour is requested.
Capability contracts say what operations exist and what Effects they may have.
Policies and scopes say what may happen here.
The host enforces authority and execution.
Plugs connect application-specific providers without putting vendor logic in Core.
Trails record what was proposed, authorised, attempted, and observed.
```

## A real Tether

This fixture exists in the repository and demonstrates `together`:

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

The three members of `together` are semantically independent. The runtime may overlap their provider calls physically, subject to its bounded concurrency rules. The later `brief.compose` Action does not become executable until the group has joined.

Physical completion order does **not** redefine program meaning. Semantic member order, group membership, join behaviour, Trail position, replay identity, and first-non-success selection remain deterministic.

That distinction is one of the central Tethers design laws:

> **Concurrency may change when work happens. It must not silently change what the program means.**

## Tethers is not an agent framework

Tethers does not contain an LLM loop and does not try to replace coding agents, assistants, planners, or orchestration frameworks.

It is designed to sit underneath them.

```text
AI / agent / ordinary application
              |
              | intent
              v
           Tethers
              |
              | typed, scoped, permissioned execution
              v
         real capabilities
```

An AI may decide that it wants to inspect a repository, run a test, move a file, call a service, or request another AI judgement. Tethers gives those operations explicit contracts, authority boundaries, deterministic planning, execution evidence, and visible uncertainty.

AI itself is just another explicit Capability when used inside a Tether. It does not receive hidden control over Conditions, policy, or permission.

## Capabilities and Plugs

A **Capability** is a versioned typed operation. Its trusted manifest can describe:

- canonical name and version;
- title and description;
- strict input and output schemas;
- Effects;
- permission scope;
- reversibility;
- determinism;
- idempotency;
- confirmation policy;
- timeout and retry contract;
- provider identity and binding.

A **Plug** packages one provider and one or more related Capabilities. Plug-specific meaning stays outside the generic host.

The implemented public Plug lifecycle includes packaging, inspection, conformance, staging, installation, enablement, disablement, and listing. Conformance proves behaviour against a declared contract; it does not itself grant permission or durable trust.

The reference Plug programme has already proved the public boundary with PDF Tools, Text Stats, and the adversarial Evil Bunny provider suite.

> **Deep Plug, narrow subject. Wide workflow, Tether.**

## Tethers Core is more serious than the surface syntax

Human Tether source lowers into a typed semantic Core. Core uses distinct identity types for programs, origins, Facts, roles, capabilities, branches, groups, batches, and item templates rather than treating them as interchangeable strings.

Canonical Format V2 gives validated Core programs stable semantic identity that is independent of irrelevant raw identifiers and representation order. The implementation includes independent canonicalisation paths and differential evidence.

The 0.5 release adds Rocket V3 as an exact portfolio seam around that frozen
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

Those are new events with causal identities. They can wake later Tethers without recursive hidden control flow.

A provider saying "I completed the call" is also not automatically the same evidence as an indepent outside observation. Tethers deliberately keeps request, result, and later observation distinct.

See [`docs/BUNNY_AND_COOKIES.md`](docs/BUNNY_AND_COOKIES.md) for the friendliest explanation of that boundary.

## One smaller surface: the portable authority workbench

The self-contained portable workbench is intentionally narrow:

```text
request -> policy match -> ALLOW / ASK / DENY
```

It does not execute the requested operation. The caller acts, asks, or stops.

```powershell
.\tethers.exe doctor --json
.\tethers.exe check --action git.status --json
.\tethers.exe check --action git.push --explain
.\tethers.exe check --action git.force_push --json
```

The same commands work on Linux with `./tethers`.

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

The native host also provides `preview` for a side-effect-free proposed Plan,
and `trail --receipt` for a bounded causal projection over validated Trail
entries. Neither surface requests authority, invokes a provider, or creates a
second persistence store.

## Download the portable workbench

For the full 0.5 host bundle, download the platform asset from the
[Tethers 0.5 release](docs/TETHERS_0_5_RELEASE.md). It contains the native
host, the smaller portable workbench, agent-facing manuals, and SHA-256
evidence. The portable workbench remains separately versioned at 0.2.2 for
compatibility.

## Version map

Several version numbers describe different layers of the project:

| Layer | Current repository truth |
| --- | --- |
| Human Tether language/protocol semantics | `0.1` specification |
| Rust reference-host package version | `0.2.2` |
| Portable workbench | `0.2.2` |
| Public Plug-authoring programme | `0.3` milestone complete and integrated |
| Together/concurrency programme | `0.4` milestone complete and integrated |
| Practical release line | `0.5` — Rocket portfolio and Agent Essentials |

The 0.5 source tree includes three starter Tether Set examples under
[`examples/tether-sets`](examples/tether-sets). They use the existing Tether
language and runtime configuration; they do not introduce a second Set
semantic or permission model.

The 0.3 and 0.4 labels are completed development milestones in this repository. They should not be confused with the portable workbench's release number or the 0.1 language version.

## Security posture

The full reference host has serious trust and execution machinery, but supervised provider execution is **not a hostile-code sandbox**.

Tethers verifies trusted manifests, provider bindings, scopes, durable intent, replay state, output schemas, and causal evidence. It does not claim that arbitrary provider code is isolated from the machine's filesystem, network, credentials, or operating system merely because it is packaged as a Plug.

Read [`docs/SECURITY.md`](docs/SECURITY.md) before treating third-party providers as trusted code.

## Repository map

- `tethers-0.1/engine-ocaml/` - deterministic parser, evaluator, typed Core, validation, canonicalisation, planning, and protocol tools.
- `tethers-0.1/host-rust/` - trusted host, policy, scope, Plug lifecycle, replay, Trail, provider execution, and Together runtime.
- `tethers-0.1/portable-rust/` - small self-contained ALLOW / ASK / DENY workbench.
- `tethers-0.1/protocol/` - capability manifests, protocol cases, fixtures, and transcripts.
- `reference-plugs/` - public Plug examples and adversarial conformance evidence.
- `docs/` - architecture, security, authoring, current state, historical roadmaps, and implementation evidence.

## Read next

For a first pass:

1. [`QUICKSTART.md`](QUICKSTART.md) - learn the full mental model, then try the portable workbench.
2. [`docs/PROJECT_OVERVIEW.md`](docs/PROJECT_OVERVIEW.md) - current system architecture and boundaries.
3. [`tethers-0.1/SPEC.md`](tethers-0.1/SPEC.md) - exact current source-language semantics.
4. [`docs/PLUG_AUTHORING.md`](docs/PLUG_AUTHORING.md) - how Capabilities enter Tethers.
5. [`docs/CONSTITUTION.md`](docs/CONSTITUTION.md) - the enduring design test.
6. [`docs/SECURITY.md`](docs/SECURITY.md) - what Tethers does and does not protect.

Historical roadmaps and worker notes remain valuable evidence of how decisions were proved, but they are not the best front door for understanding the current system.

> **Make things happen. Keep the receipts.**
