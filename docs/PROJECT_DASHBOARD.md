# Tethers Project Dashboard

Updated: 2026-09-02

## Current product direction

**Agent Essentials: make Tethers immediately useful to AI agents.**

The foundation phase is no longer the bottleneck. The current opportunity is to expose and populate what already exists.

## What is already integrated

### Language and Core

- Human Tether 0.1 semantics.
- Explicit `together` fan-out/join syntax.
- typed Tethers Core.
- Human AST to Core lowering.
- Core validation.
- production Core evaluation path.
- Canonical Format V2 / program digest machinery.

### Host/runtime

- policy and authority boundary;
- generic operational scope evidence;
- trusted capability manifests and provider binding;
- durable intent;
- replay protection;
- Trail;
- Result Anchors;
- FIFO generated-event queue;
- provider supervision;
- Together physical overlap and bounded concurrency.

### Plug platform

- deterministic `.tetherplug` packaging;
- read-only package inspection;
- public conformance;
- stage/install;
- enable/disable with scope;
- installed Plug listing;
- PDF Tools reference Plug;
- Text Stats fresh-agent authoring proof;
- Evil Bunny adversarial provider proof.

### Portable workbench

Portable 0.2.2 provides a separate small authority surface:

```text
request -> ALLOW / ASK / DENY
```

for Windows x64 and Linux x64 musl.

It is **not** the full Tethers runtime.

## 0.5 release state

The practical 0.5 release line now combines the accepted Rocket V3 foundation
with the Agent Essentials discovery and provider work. Rocket keeps frozen V2
identity as its authority and selects exact implementations by runtime shape;
the exhaustive reference remains available for bounded differential evidence.
The native host is still versioned `0.2.2` for compatibility, while `0.5` is
the product release line.

The implementation checkpoint is `bf645c94b96dd100ad0f4580583b32f54bf7049f`.
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

The side-effect-free plan surface and richer Trail query ergonomics remain
follow-on work; the 0.5 release does not pretend they are already public.

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

Historical worker notes and roadmap checkpoints remain valid historical evidence. The fix is to repair living current-truth documents rather than rewriting history.

## Version map

| Thing | Version/status |
| --- | --- |
| Human Tether language/protocol | `0.1` |
| Reference host Cargo package | `0.2.2` |
| Portable workbench | `0.2.2` |
| Public Plug authoring milestone | `0.3` complete |
| Together/concurrency milestone | `0.4` complete |
| Practical release line | `0.5` Rocket portfolio + Agent Essentials |

## Engineering posture

- Do not invent another concurrency gate.
- Do not redesign canonicalisation without a demonstrated defect.
- Do not turn Core into a catalogue of applications.
- Prefer semantic Capabilities over generic shell escape hatches.
- Build useful Plugs and run Tethers in real agent workflows.
- Let actual use expose the next missing abstraction.

## Best current reading order

1. `README.md`
2. `QUICKSTART.md`
3. `docs/PROJECT_OVERVIEW.md`
4. `tethers-0.1/SPEC.md`
5. `docs/PLUG_AUTHORING.md`
6. `docs/SECURITY.md`
7. `docs/CONSTITUTION.md`

Deep architecture and historical proof remain under `docs/architecture/`, `docs/concurrency/`, `docs/perf/`, `docs/review/`, `docs/foundation-pass/`, and `docs/worker-notes/`.
