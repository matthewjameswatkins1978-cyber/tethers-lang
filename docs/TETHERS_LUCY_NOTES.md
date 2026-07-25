# Tethers — Lucy's Project Notes

This is compact orientation for returning to Tethers. It is not the language
specification, Constitution, task queue, implementation standard, or source of
truth for current behaviour. Read the authoritative project files before
changing semantics or code.

## The Idea

Tethers is a small deterministic behaviour language and capability protocol for
connecting events to clear, typed, permissioned Actions.

> Apps provide the sockets. Tethers provides the cables.

Lantern Keeper may host Tethers, but Tethers remains independent:

- Lantern Keeper remembers things.
- Tethers responds to things and creates Plans.
- Hosts, adapters, and providers perform Actions.
- The Trail records what happened and why.

## Design Test

The Tethers language should remain:

- small and elegant;
- predictable and deterministic;
- human-readable and human-writable;
- reliably writable by AI;
- visually editable through HQ;
- permissioned, explainable, and auditable;
- useful without becoming a general-purpose programming language.

Prefer one canonical expression for each language concept. Additional aliases,
spellings, and decorative forms increase ambiguity for humans, AI, formatting,
documentation, and HQ.

This small-language rule does not restrict the implementation languages. OCaml,
Rust, PowerShell, and future implementation code follow
`docs/IMPLEMENTATION_LANGUAGE_STANDARD.md` and should use the language's real
strengths where justified.

## Architectural Boundary

1. A host sends an event, immutable Facts, Capability schemas, and Tether source.
2. The OCaml engine parses, validates, evaluates, and proposes an ordered Plan.
3. The host resolves current trusted capability and policy state.
4. Approved Actions cross a durable intent boundary before execution.
5. The host validates outcomes and appends authorisation and execution Trail
   entries.
6. Known outcomes may generate visible Result Anchors for later evaluation.

The engine plans. It does not secretly perform external Effects or grant itself
permission.

Applications expose Anchors, Facts, and Capabilities. Tethers Core must not
contain application-specific branches or modes. Files, Lantern Keeper, GitHub,
music tools, AI, and other integrations are Capability sets, not grammar
features.

An application Capability is a request through the application's public
judgement boundary. For example, `lantern.memory.record` asks Lantern Keeper to
process material according to its own memory rules; it must not bypass those
rules and mutate storage directly.

## Language Model

```tethers
tether "Name"

anchor
    event.name

when
    visible.fact is "value"
    and numeric.fact greater_than 0

do
    capability.name
        input: visible.fact
```

- Anchor: what starts evaluation.
- Conditions: deterministic tests over supplied Facts.
- Actions: Capability invocations proposed in the Plan.
- Trail: the explanation of reception, evaluation, authorisation, and execution.

The same complete deterministic input should produce the same semantic Plan and
evaluation Trail. AI may be called only as an explicit Capability. Its structured
output becomes visible data that later deterministic rules may inspect.

## Error And Trust Principles

- Preserve correlation identifiers and trustworthy accumulated Trail context
  where the specification requires them.
- No Action is planned after a failed Condition or planning error.
- Model distinct outcomes distinctly; do not merge denied, unavailable, failed,
  cancelled, timed out, and uncertain states.
- Exceptions belong at deliberate boundaries, not as casual general control
  flow.
- Fixtures protect observable 0.1 behaviour. Change a frozen fixture only with
  an authorised semantic decision and corresponding specification update.
- The planner never trusts complete manifests or provider claims.
- Hosts check current manifest, provider, policy, and scope state before dispatch.
- Unknown or unestablished trust state fails closed.

## HQ

HQ is a visual editor and live view of the same underlying Tether as the text.
They must never become separate sources of truth.

HQ should make the Anchor, Conditions, Actions, Effects, permission state, Trail,
preview results, and supported reversal visible. Do not design syntax solely for
visual prettiness, but every language construct needs one unambiguous visual
representation.

## Current Orientation

Verify current details in the repository because they change. The principal
starting points are:

1. `docs/CONSTITUTION.md` — enduring Tethers language principles.
2. `tethers-0.1/SPEC.md` — precise signed-off 0.1 semantics.
3. `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md` — code and language-use standard.
4. `AGENTS.md` — concise operating guidance.
5. `docs/PROJECT_CONTROL.md` — ownership and task-state contract.
6. `docs/DECISIONS.md` — accepted architectural decisions.
7. `docs/CURRENT_GOAL.md` and `docs/PROJECT_DASHBOARD.md` — current state.
8. `docs/OCAML_GUIDE_FOR_AGENTS.md` — verified OCaml environment and cautions.

Current implementation uses OCaml 5.5, Rust, PowerShell 7, Yojson, native Windows
opam, JSON fixtures, and MCP transcript tests. The project-local OCaml switch is
path-bound, so do not casually move its directory.

For language-specific work, consult the exact official documentation rather than
relying on model memory. The controlling semantic documents decide intended
behaviour; the compiler and tests determine whether the implementation actually
satisfies it.

## Collaboration

Matthew owns product intent and final judgement. Current role assignment and
routing live in `docs/PROJECT_CONTROL.md` and `docs/AGENT_WORKFLOW.md`; this
orientation file does not freeze vendors or models.

Ordinary chat Lucy can inspect pushed GitHub state and handle architecture,
review, task compilation, and acceptance checking. Computer-enabled workers are
used when implementation or local machine access is genuinely required. One task
has one owner, and reports are not proof until repository evidence supports them.

## Before Proposing Or Reviewing A Change

Ask:

1. Does it belong in the Tethers language, a Capability, an adapter, or ordinary
   implementation code?
2. Is there already one canonical way to express the language concept?
3. Is behaviour visible, deterministic, typed, permissioned, and explainable?
4. Can HQ represent it without inventing a second model?
5. Can an AI generate the Tether reliably from the specification?
6. Does the implementation use its programming language idiomatically and make
   invalid states difficult?
7. Is each abstraction justified by a current invariant or boundary?
8. Do focused tests and fixtures prove both success and required failure paths?
9. Does the Trail explain the outcome honestly?
10. Has the live source and current authoritative documentation been checked?

## Warning Signs

Pause if a proposal introduces:

- application-specific grammar or modes;
- arbitrary scripting or hidden computation in Tethers;
- invisible AI judgement controlling workflow;
- direct Effects inside the evaluator;
- multiple language spellings for one concept;
- permission checks performed only by the planner;
- a visual model that can drift from source text;
- behaviour absent from the specification and fixtures;
- speculative framework layers with no current invariant or boundary;
- primitive implementation representations chosen only to appear simple;
- advanced implementation technique with no concrete engineering benefit.

## North Star

Tethers should feel less like programming an automation platform and more like
stating a small, inspectable agreement:

> When this happens, if these visible Facts are true, request these permitted
> Actions, and leave a complete Trail.
