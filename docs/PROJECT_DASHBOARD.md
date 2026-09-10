# Tethers Project Dashboard

Updated: 2026-09-10

## Product state

**Tethers has crossed the line from foundation project to practical AI-facing execution platform.**

The published 0.5 release combines the deterministic language/Core, trusted host/runtime, Plug system, bounded concurrency, replay/evidence machinery, and the first Agent Essentials surfaces needed for an unfamiliar agent to inspect and use the system.

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

Latest GitHub release: **Tethers 0.5**  
Tag: `tethers-v0.5.8`

Published release assets include Windows x64 and Linux x64 musl bundles. The current asset filenames use the `tethers-0.5.0-*` form.

The 0.5 tagged source includes:

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

## Repository state warning

The documentation merge now sits at the tip of `main`, but it is documentation-only. The implementation ancestry beneath it still comes from the earlier September 1 checkpoint whose implementation tip was `21bb7442fa9f8442db98e193eb4954096f356678`.

The published `tethers-v0.5.8` tag points to later implementation source at commit `2a2fe3986805905a90aa48ad83e95d79f0357b04`.

Therefore:

```text
published 0.5 implementation source != implementation currently reachable from main
```

Use the tagged release source when verifying published 0.5 commands and packaging. Do not infer that an absent command in the implementation currently reachable from `main` was absent from the published 0.5 bundle.

Reconciling this branch/release ancestry should be treated as repository hygiene before the next product release.

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
- Agent-oriented workspace and coding reference providers in the 0.5 tagged source.

### Agent-facing product surface

The published 0.5 release adds the practical path a cold agent needs:

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

The most important gaps are now practical rather than architectural:

1. **Default-branch/release coherence** - make it obvious which source represents the shipped product.
2. **Version presentation** - reduce confusion between language `0.1`, host/portable `0.2.2`, milestone labels `0.3`/`0.4`, practical release `0.5`, tag `tethers-v0.5.8`, and `0.5.0` asset filenames.
3. **Real-world adoption evidence** - use Tethers on ordinary agent jobs and keep the receipts.
4. **Broader useful Plugs** - add semantic everyday capabilities where actual use demonstrates demand.
5. **Cold-agent ergonomics** - keep testing whether an unfamiliar agent can discover, inspect, preview, execute, and audit without private project knowledge.
6. **Packaging and platform clarity** - state feature/platform boundaries exactly, especially where durability or containment behaviour differs.

## Version map

| Thing | Current meaning |
| --- | --- |
| Human Tether language semantics | `0.1` |
| Reference-host Cargo package | `0.2.2` compatibility/package axis |
| Portable Workbench | `0.2.2` |
| Public Plug authoring milestone | `0.3` complete |
| Together/concurrency milestone | `0.4` complete |
| Practical product release | `0.5` |
| Latest public GitHub tag | `tethers-v0.5.8` |
| Current 0.5 asset filename line | `tethers-0.5.0-*` |

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

For the exact 0.5 Agent Essentials command surface, use the source and manuals under the `tethers-v0.5.8` tag until the implementation ancestry reachable from `main` is reconciled with the published release.

Historical roadmaps, reviews, performance notes, foundation evidence, and worker notes remain valuable records of their checkpoints. They are not living product-status documents.
