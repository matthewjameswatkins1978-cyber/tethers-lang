# The Tethers Constitution

Tethers is a small deterministic behaviour language for connecting events to clear, typed and permissioned Actions.

Its purpose is to make useful software behaviour understandable, controllable and auditable by humans and AI.

## 1. Remain small

Tethers must remain substantially smaller than a general-purpose programming language.

New syntax must earn its place. A useful Capability does not automatically require a new language feature.

## 2. One concept, one expression

Each supported idea has one canonical name, one syntax and one representation.

Tethers rejects unnecessary aliases, alternative spellings and decorative variations.

Humans, AI, HQ and the formatter should all produce the same canonical form.

## 3. Applications provide the abilities

Applications expose typed events, Facts and Capabilities.

Tethers connects them without containing application-specific knowledge.

There are no file, music, AI, GitHub or Lantern Keeper modes. These are Capability sets, not language features.

## 4. Tethers plans; hosts execute

Tethers parses, validates, evaluates and produces an Action Plan.

The host authorises and executes that Plan.

A proposed Action must never be recorded as though it already happened.

## 5. Evaluation is deterministic

The same complete input must produce the same semantic Plan and evaluation Trail.

Tethers must not secretly read time, randomness, files, networks, environment variables or live application state.

Changing information must be supplied explicitly as event data or immutable Facts.

## 6. Permissions remain visible

Capability schemas describe possible Effects.

Policies authorise them.

Hosts enforce them.

Trails record them.

A Tether may request authority, but it cannot grant authority to itself.

## 7. AI is explicit

AI may be invoked through declared Capabilities.

AI results become visible data that later rules can inspect.

AI must not exercise hidden control over workflow, permissions or policy.

## 8. Every outcome is explainable

The Trail records what arrived, what matched, what failed, what was authorised and what executed.

Errors should retain all trustworthy context available when they occur.

Nobody should be left wondering why something moved, changed, stopped or contacted another system.

## 9. Text and HQ share one truth

The textual rule and HQ are two views of the same underlying Tether.

They must never drift into separate representations.

HQ makes Tethers easier to write; it does not create a second language.

## 10. Human clarity comes first

A person should be able to identify, at a glance:

- what wakes the Tether;
- what Facts it checks;
- what Actions it requests;
- what Effects those Actions may have.

Clever compression is rejected when it damages clarity.

## 11. Ordinary machinery remains ordinary code

Tethers is for behaviour that owners may reasonably want to inspect, alter, disable or audit.

Database internals, algorithms, rendering, byte manipulation and ordinary program flow belong in normal implementation code.

## 12. Usefulness must justify complexity

Tethers exists to do genuinely useful work, not merely to demonstrate elegant language design.

Every addition must improve at least one of:

- clarity;
- safety;
- predictability;
- composability;
- practical usefulness.

If an addition makes the language harder to understand without clearly improving those qualities, it does not belong.

---

## The governing test

Before adding anything, ask:

> Does this make useful behaviour clearer, safer and more predictable without making Tethers unnecessarily larger?

If the answer is not clearly yes, leave it out.

> Apps provide the sockets.
>
> Tethers provides the cables.
>
> HQ is the mixing desk.
