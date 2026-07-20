# Tethers — Lucy's Project Notes

This is a compact orientation file for Lucy when returning to Tethers. It is not the language specification, constitution, task queue, or source of truth for current behaviour. Read the authoritative project files before changing semantics.

## The idea

Tethers is a small deterministic behaviour language and capability protocol for connecting events to clear, typed, permissioned actions.

Friendly formulation:

> Apps provide the sockets. Tethers provides the cables.

Lantern Keeper may host Tethers, but Tethers must remain independent of it:

- Lantern Keeper remembers things.
- Tethers responds to things and creates action plans.
- Hosts, plugins, and agents perform actions.
- The Trail records what happened and why.

## Design test

Tethers should remain:

- small and elegant;
- predictable and deterministic;
- human-readable and human-writable;
- reliably writable by AI;
- visually editable through HQ;
- permissioned, explainable, and auditable;
- useful without becoming a general-purpose programming language.

Prefer one clear, canonical way to express each idea. Do not add aliases or alternative spellings merely for convenience. Every extra form increases ambiguity for humans, AI authors, documentation, formatting, and HQ.

## Architectural boundary

The intended execution boundary is:

1. A host sends JSON containing an event, facts, capabilities, and Tether source.
2. The OCaml engine parses, validates, and evaluates it.
3. The engine returns a JSON action plan and Trail.
4. The host independently checks permissions and conflicts.
5. The relevant host or adapter executes each authorised action.

The engine plans; it does not secretly perform external effects.

Applications expose:

- Anchors: events that wake rules;
- Facts: simple visible state inspected by Conditions;
- Capabilities: typed actions with declared inputs, outputs, effects, and permission requirements.

Tethers Core must not contain application-specific branches or modes. Files, Lantern Keeper, GitHub, music tools, AI, and other integrations are capability sets—not grammar features.

An application capability is a request through that application's public judgement boundary. For example, `lantern.memory.record` asks Lantern Keeper to process material according to its own memory rules; it must not bypass those rules and directly mutate storage.

## Language model

The conceptual shape is:

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
- Conditions: facts that must match.
- Actions: capabilities to include in the plan.
- Trail: the engine-generated explanation of reception, evaluation, authorisation, and execution reporting.

The same event, facts, capabilities, permissions, and Tether version should produce the same action plan and evaluation Trail. AI may be called only as an explicit capability. AI output becomes recorded data that later deterministic rules may inspect.

## Error principles

- Errors discovered after evaluation context exists should preserve correlation identifiers and accumulated Trail where the specification says so.
- No Action should be planned after a failed Condition or Action-planning error.
- Expected evaluator outcomes should use explicit OCaml variants where practical.
- Exceptions should remain narrow and be caught at deliberate boundaries; do not use them as casual general control flow.
- Fixtures define and protect observable 0.1 behaviour. Change a frozen fixture only with an intentional semantic decision and corresponding specification update.

## HQ

HQ is a visual editor and live view generated from the same underlying rule representation as the text. Text and diagram must never become separate sources of truth.

HQ should make these immediately visible:

- the Anchor;
- each Condition and its pass/fail path;
- planned Actions;
- required effects and permissions;
- Trail and execution history;
- preview/test results;
- undo availability where the host supports it.

Do not design text syntax solely for visual prettiness, but ensure every language construct has one unambiguous visual representation.

## Current implementation orientation

Verify these details in the repository because they can change:

- active development tree: `tethers-0.1/`;
- engine: OCaml 5.5, Dune, Yojson;
- host/demo executor: Rust;
- native Windows workflow using PowerShell 7;
- local opam switch is path-bound, so do not casually move the tree;
- protocol behaviour is protected by discoverable JSON fixture cases and deterministic-repeat testing;
- the OCaml code has been separated into parser, protocol, and evaluator/I/O responsibilities.

Known project documents to read before substantial work:

1. `docs/CONSTITUTION.md` — enduring values and boundaries.
2. `AGENTS.md` — agent operating rules.
3. `tethers-0.1/SPEC.md` — authoritative current 0.1 semantics.
4. `docs/DECISIONS.md` — accepted architectural decisions.
5. `docs/CURRENT_GOAL.md` — current checkpoint.
6. `docs/TASK_QUEUE.md` — planned work.
7. `docs/OCAML_GUIDE_FOR_AGENTS.md` — verified OCaml environment and house style.

For OCaml-specific work, consult the relevant section of the official OCaml 5.5 manual instead of relying on memory. In particular, revisit variants, pattern matching, exceptions, modules/compilation units, and the exact standard-library APIs touched by the task.

## Collaboration model

Matthew owns product intent and final judgement.

Lucy helps maintain the conceptual model, compress ideas, identify the next bounded milestone, and watch for architectural drift.

Cline with DeepSeek is useful for narrow implementation tasks with explicit scope, files, forbidden changes, tests, and a short timebox. It should normally leave work uncommitted for review.

Codex should inspect the actual diff, correct mistakes or confusion, run proportional verification, update project state when warranted, and handle Git commits. Neither agent's report is proof; trust but verify.

## Before proposing or reviewing a change

Ask:

1. Does this make the language smaller or merely more capable?
2. Is there already one way to express it?
3. Is it visible, deterministic, typed, permissioned, and explainable?
4. Does it belong in Tethers, or in the host/application's normal code?
5. Can HQ represent it without inventing a second model?
6. Can a human understand it at a glance?
7. Can an AI generate it reliably from the specification?
8. Is the behaviour protected by a focused fixture?
9. Does the Trail explain both success and failure?
10. Has the relevant current documentation and actual source been checked?

## Warning signs

Pause if a proposal introduces:

- application-specific grammar or language modes;
- arbitrary scripting or hidden computation;
- invisible AI judgement controlling workflow;
- direct side effects inside the evaluator;
- multiple spellings for the same concept;
- permission checks performed only by the planner;
- a visual model that can drift from source text;
- behaviour not represented in fixtures and the specification;
- abstraction added before the concrete 0.1 behaviour requires it.

## North star

Tethers should feel less like programming an automation platform and more like stating a small, inspectable agreement:

> When this happens, if these visible facts are true, request these permitted actions—and leave a complete Trail.
