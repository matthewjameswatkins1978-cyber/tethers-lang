# Tethers 0.1 semantic baseline

The `tethers-0.1/` directory is the semantic root of the current Human Tether language and the first reference implementation family. The `0.1` name describes the **language/protocol semantics**, not the overall modern product release number.

For the current product story, start at [`../README.md`](../README.md). For exact Human Tether semantics, this directory's authority is [`SPEC.md`](SPEC.md).

## What lives here

Tethers connects explicit events and immutable Facts to typed Capability requests through small deterministic rules. The wider host/runtime then owns authority, execution, replay, and evidence.

```text
Human Tether + event + Facts + Capability schemas
                    |
                    v
             OCaml Tethers Core
        parse -> validate -> evaluate -> plan
                    |
                    v
               Action Plan
                    |
                    v
              Rust host
       trust + policy + scope + replay
                    |
                    v
          approved Capability work
                    |
                    v
          result / uncertainty / Trail
```

The enduring separation is:

```text
Capabilities describe.
Policies authorise.
Hosts enforce.
Trails record.
```

A Plan is a request, not permission.

## What the 0.1 language currently supports

The current precise source contract is in [`SPEC.md`](SPEC.md). At a high level it includes:

- one Anchor per Tether;
- immutable supplied Facts;
- deterministic Conditions;
- typed Actions;
- explicit `together` fan-out/join groups;
- stable position-derived identities;
- result-dependent continuation through later events rather than hidden mutable Action-result references.

The language deliberately excludes arbitrary scripting features such as loops, arithmetic, user functions, mutation, implicit I/O, and hidden coercion.

This is intentional. Application-specific power belongs in Capabilities and Plugs rather than in ever-growing language syntax.

## What the wider project has added around 0.1

The language version remained small while the surrounding execution platform grew substantially.

Later product milestones added and proved:

- typed Tethers Core and semantic canonicalisation;
- trusted Capability manifests and provider binding;
- policy and operational scope enforcement;
- durable intent and replay protection;
- Result Anchors and a host-owned FIFO follow-up event queue;
- public Plug packaging, inspection, conformance, installation, and enablement;
- physical bounded concurrency for `together` without changing its deterministic semantics;
- a portable ALLOW / ASK / DENY authority workbench;
- agent-oriented discovery, preview, Trail receipts, and practical workspace/coding capabilities in the published 0.5 release line;
- exact Rocket identity/canonicalisation portfolio work and deterministic benchmarking.

Those are product/runtime evolutions around the 0.1 language. They are not reasons to rename the language every time the host gains a new capability.

## Repository map

- `SPEC.md` - current 0.1 language and protocol semantics.
- `protocol/` - request, response, capability, and transcript fixtures.
- `engine-ocaml/` - parser, validator, evaluator, typed Core, canonicalisation, and deterministic planning.
- `host-rust/` - trusted host/runtime, policy, scope, Plug lifecycle, replay, Trail, and provider execution.
- `portable-rust/` - small self-contained authority workbench for scripts and agents.
- `examples/` - Human Tether examples.
- `scripts/` - project verification and demonstration helpers where present in the relevant checkpoint.

## Important distinction: planner versus host

The semantic engine itself does not execute actions, grant permission, query live state, or store application data.

The full product is larger than the deterministic planner because the host/runtime deliberately owns the consequential boundary.

This is not duplication. It prevents the component deciding what a program means from silently becoming the component that grants itself authority to change the world.

## Provider integration

Application-specific behaviour belongs behind Capability contracts and Plug/provider code.

The reference architecture can bind providers over MCP stdio, but MCP is a transport/protocol binding rather than the source of Tethers semantics or authority.

There are no special GitHub, files, email, music, AI, or Lantern Keeper language modes. Those are Capability sets.

## Version note

Several version axes coexist in the wider repository:

| Axis | Meaning |
| --- | --- |
| Human Tether language | `0.1` |
| Reference-host Cargo package | `0.2.2` compatibility/package axis |
| Portable Workbench | `0.2.2` |
| Plug authoring milestone | `0.3` complete |
| Together/concurrency milestone | `0.4` complete |
| Practical product release | `0.5` |

The latest published GitHub product release is tagged `tethers-v0.5.8`. The documentation now sits on `main`, but the implementation ancestry reachable from `main` is still based on an earlier checkpoint, so use the tagged source when reproducing the exact 0.5 agent-facing release surface.

## Read next

- [`../README.md`](../README.md) - why Tethers exists and where it fits.
- [`../QUICKSTART.md`](../QUICKSTART.md) - practical mental model.
- [`SPEC.md`](SPEC.md) - exact 0.1 semantics.
- [`../docs/PROJECT_OVERVIEW.md`](../docs/PROJECT_OVERVIEW.md) - full architecture.
- [`../docs/PLUG_AUTHORING.md`](../docs/PLUG_AUTHORING.md) - Capability/Plug authoring.
- [`../docs/SECURITY.md`](../docs/SECURITY.md) - trust boundary and limits.
- [`portable-rust/README.md`](portable-rust/README.md) - small authority workbench.
