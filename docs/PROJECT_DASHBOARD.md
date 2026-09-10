# Tethers Project Dashboard

Updated: 2026-09-10

## Product state

**Tethers has crossed the line from foundation project to practical AI-facing execution platform.**

The published 0.6 release combines the deterministic language/Core, trusted host/runtime, Plug system, bounded concurrency, replay/evidence machinery, and the Agent Essentials surfaces needed for an unfamiliar agent to plan, execute, and inspect the system.

The next bottleneck is not missing architecture. It is product coherence, repository/release alignment, broader practical Capabilities, and evidence from real workloads.

## The product in one picture

```text
AI / application intent
          |
          v
   deterministic Tether / Plan
          |
          v
 trusted Capability contract
      + policy + scope
          |
          v
      Tethers Host
          |
          v
       Plug/provider
          |
          v
 real effect + result/uncertainty
          |
          v
     causal Trail
```

The point is not to make AI deterministic. The point is to make the **consequential boundary around AI** deterministic, bounded, and inspectable.

## Latest published release

Latest GitHub release: **Tethers 0.6**
Tag: `tethers-v0.6.0`

Published release assets include Windows x64 and Linux x64 musl bundles. The
current asset filenames use the `tethers-0.6.0-*` form.

The 0.6 tagged source includes:

- native host discovery;
- trusted Capability listing and inspection;
- installed Plug inspection;
- side-effect-free preview;
- bounded Trail receipt projection;
- agent workspace/text/hash/patch reference capabilities;
- structured Git, argv-only process, and named verification reference capabilities;
- `tethers-bench`;
- the wider trusted host/runtime and Plug platform;
- the separately versioned Portable Workbench 0.2.2.

## Repository state

The Tethers 0.6 implementation ancestry is now reconciled into `main`, with
the published `tethers-v0.6.0` tag carrying the current product line. The
stable `tethers plan` command is present on the authoritative development
line.

## What is already proven

### Language and Core

- Human Tether 0.1 semantics.
- Explicit `together` fan-out/join syntax.
- Typed Tethers Core.
- Human AST to Core lowering.
- Core validation.
- Production Core evaluation path.
- Canonical Format V2 / ProgramDigest identity machinery.
- Rocket V2 exactness work and Rocket V3 typed refinement/portfolio work in the 0.5 release line.

### Host/runtime

- Policy and authority boundary.
- Generic operational scope evidence.
- Trusted Capability manifests and provider binding.
- Durable intent.
- Replay protection.
- Trail.
- Result Anchors.
- FIFO generated-event queue.
- Provider supervision.
- Together physical overlap and bounded concurrency.

### Plug platform

- Deterministic `.tetherplug` packaging.
- Read-only package inspection.
- Public conformance.
- Stage/install.
- Enable/disable with scope.
- Installed Plug listing.
- Reference and adversarial conformance evidence.
- Agent-oriented workspace and coding reference providers in the 0.6 tagged source.

### Agent-facing product surface

The published 0.6 release adds the practical path a cold agent needs:

```text
describe Tethers
    -> list trusted Capabilities
    -> inspect exact contracts
    -> inspect installed Plug state
    -> preview proposed work
    -> run admitted work
    -> inspect bounded Trail evidence
```

That was the previous Agent Essentials target. It should now be treated as shipped product surface, not future work.

### Portable workbench

Portable Workbench 0.2.2 remains a deliberately smaller surface:

```text
request -> ALLOW / ASK / DENY
```

It does not execute the requested action. It is useful when an existing agent or script only needs a deterministic fail-closed authority decision.

## Current gaps

## 0.6 release state

The practical 0.6 release line now combines the accepted Rocket V3 foundation
with the Agent Essentials discovery and provider work. Rocket keeps frozen V2
identity as its authority and selects exact implementations by runtime shape;
the exhaustive reference remains available for bounded differential evidence.
The native host is still versioned `0.2.2` for compatibility, while `0.6` is
the product release line.

The implementation checkpoint is `f3b2da5693c7eb61526a1cfc692983ba49ba6b8a`.
Windows packaging and bounded local evidence are complete; Linux packaging,
the hosted release URL, signatures, and physical installation remain external
acceptance facts until the tagged workflow proves them.

## Current gap

Tethers currently has more capability than its everyday agent-facing surface makes obvious.

The main gaps are practical:

- capability self-discovery should be easier from the CLI;
- the public Plug ecosystem needs useful everyday providers;
- agent configuration/scope diagnostics need less friction;
- planning and Trail evidence should be easier to query;
- the front-door documentation previously over-emphasised the portable authority façade.

The side-effect-free `tethers plan` surface is now public on the reconciled
main line. It emits `tethers.plan/1` and proves zero provider invocations and
zero Trail execution entries; richer Trail query ergonomics remain follow-on
work.

## Agent Essentials target

A cold AI agent should be able to:

```text
discover Tethers
    -> discover installed Capabilities
    -> inspect exact trusted schemas/effects/scopes
    -> form bounded intent
    -> preview/plan
    -> execute under explicit authority
    -> inspect the Trail/result evidence
```

without bespoke knowledge of the host.

## Documentation audit

The 2026-09-01 documentation audit found several current-truth files had fallen behind implementation:

- README/Quickstart foregrounded ALLOW / ASK / DENY and made Tethers look like only a policy gate.
- `PROJECT_OVERVIEW.md` still described parallel Actions as future work.
- `SECURITY.md` still described Universal Plug execution as architecture-only.
- `CURRENT_GOAL.md`, `PROJECT_DASHBOARD.md`, and `ROAD_TO_0_4.md` still said the accepted 0.4 chain had not reached `main`.

The most important gaps are now practical rather than architectural:

1. **Default-branch/release coherence** - make it obvious which source represents the shipped product.
2. **Version presentation** - reduce confusion between language `0.1`, host/portable `0.2.2`, milestone labels `0.3`/`0.4`, practical release `0.5`, tag `tethers-v0.5.8`, and `0.5.0` asset filenames.
3. **Real-world adoption evidence** - use Tethers on ordinary agent jobs and keep the receipts.
4. **Broader useful Plugs** - add semantic everyday capabilities where actual use demonstrates demand.
5. **Cold-agent ergonomics** - keep testing whether an unfamiliar agent can discover, inspect, preview, execute, and audit without private project knowledge.
6. **Packaging and platform clarity** - state feature/platform boundaries exactly, especially where durability or containment behaviour differs.

The implementation ancestry is now reconciled into the authoritative main line;
the tag remains intact as release evidence and the current gaps are product and
adoption work rather than a missing 0.5 source line.

## Version map

| Thing | Current meaning |
| --- | --- |
| Human Tether language semantics | `0.1` |
| Reference-host Cargo package | `0.2.2` compatibility/package axis |
| Portable Workbench | `0.2.2` |
| Public Plug authoring milestone | `0.3` complete |
| Together/concurrency milestone | `0.4` complete |
| Practical release line | `0.6` Plan + execution + replay/Trail proof |
| Latest public GitHub tag | `tethers-v0.6.0` |
| Current 0.6 asset filename line | `tethers-0.6.0-*` |

These are related but not interchangeable. Future releases should make the public-facing version identity simpler.

## Current engineering posture

- Do not invent another concurrency gate.
- Do not redesign canonicalisation without a demonstrated defect.
- Do not turn Core into a catalogue of applications.
- Prefer semantic Capabilities over generic shell escape hatches.
- Use Tethers in real agent workflows and let friction reveal the next missing abstraction.
- Keep authority, execution outcome, and external observation distinct.
- Keep uncertainty visible rather than retrying optimistically.
- Treat documentation and release/source alignment as part of product correctness.

## Best current reading order

1. `README.md` - why Tethers exists and why you might use it.
2. `QUICKSTART.md` - the operating mental model.
3. `docs/PROJECT_OVERVIEW.md` - whole-system architecture.
4. `tethers-0.1/SPEC.md` - exact Human Tether semantics.
5. `docs/PLUG_AUTHORING.md` - how Capabilities enter the system.
6. `docs/SECURITY.md` - what the trust boundary does and does not guarantee.
7. `docs/CONSTITUTION.md` - the enduring design test.

For the exact 0.6 Agent Essentials command surface, use the source and manuals under the `tethers-v0.6.0` tag.

Historical roadmaps, reviews, performance notes, foundation evidence, and worker notes remain valuable records of their checkpoints. They are not living product-status documents.
