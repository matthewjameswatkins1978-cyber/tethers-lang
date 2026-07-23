# Tethers — Project Guidance for Coding Agents

Read this file, `docs/AGENT_WORKFLOW.md`, `docs/CONSTITUTION.md`, `tethers-0.1/SPEC.md`,
`docs/MCP_PLAN.md`, `docs/CURRENT_GOAL.md`, and `docs/TASK_QUEUE.md` before
making changes.

`docs/CONSTITUTION.md` governs enduring Tethers design principles.
`tethers-0.1/SPEC.md` defines the current precise 0.1 language and protocol
semantics.
`docs/MCP_PLAN.md` records the approved post-0.1 MCP direction: Tethers owns
its MCP interface directly in OCaml, while Lantern Keeper remains a host and
capability provider.
For OCaml implementation tasks, also read the task-relevant section of
`docs/OCAML_GUIDE_FOR_AGENTS.md`.
For optional project orientation, `docs/TETHERS_LUCY_NOTES.md` can help recover
the conceptual model, but it is not an authoritative specification.

## Project definition

Tethers is a small deterministic behaviour language and capability protocol for connecting applications through clear, typed, permissioned rules.

Friendly description:

> Apps provide the sockets. Tethers provides the cables.

A Tether expresses:

> When this event happens, check these known facts, then propose these permitted actions.

Tethers is not:

- a general-purpose programming language;
- Lantern Keeper’s database language;
- Lantern Keeper’s internal implementation language;
- an AI prompting language;
- an integration catalogue;
- an application-specific workflow engine;
- an authority that grants permissions;
- the component that executes Actions.

Tethers is a deterministic planner.

## Core architecture

The initial reference architecture is:

```text
Lantern Keeper or another host
    sends:
    - Tether source and version
    - event
    - immutable Fact snapshot
    - available Capability schemas
        ↓
Tethers Core — OCaml
    - parses the Tether
    - validates structure and types
    - evaluates Conditions
    - produces an Action Plan
    - records the evaluation Trail
        ↓
Host application — initially Rust
    - checks real permissions
    - executes approved Actions
    - records results
    - appends the execution Trail
    - emits result events where appropriate
```

For Lantern Keeper:

```text
Lantern Keeper knows things.
Tethers responds to things.
Capabilities do things.
```

Lantern Keeper is the first serious host, but Tethers must not contain Lantern Keeper-specific language features or branches.

Never add logic resembling:

```text
if application == "Lantern Keeper"
```

Lantern Keeper integration belongs in Capability schemas, host code, and adapters.

## Canonical vocabulary

Use these terms consistently:

| Term       | Meaning                                                   |
| ---------- | --------------------------------------------------------- |
| Tether     | One behavioural rule                                      |
| Tether Set | An installable collection of related Tethers              |
| Anchor     | The event that wakes a Tether                             |
| Fact       | Immutable input available to Conditions                   |
| Condition  | Deterministic test over a Fact                            |
| Action     | A requested Capability invocation                         |
| Capability | A typed operation exposed by a host or adapter            |
| Effect     | An external consequence declared by a Capability          |
| Plan       | Ordered Actions proposed by Tethers                       |
| Trail      | Causal record of evaluation, authorisation, and execution |
| Host       | Application supplying input and enforcing policy          |
| Adapter    | Component exposing another system as Capabilities         |
| HQ         | Future visual editor, tester, and Trail inspector         |

Do not casually introduce synonyms such as trigger, command, operation, workflow node, or execution log when one of the canonical terms already applies.

## Example Tether

```tethers
tether "Record completed software task"

anchor
    coding.task_completed

when
    project.type is "software"
    and task.changed_files greater_than 0

do
    lantern.task.record
        project: anchor.project
        task: anchor.task
```

Meaning:

1. A `coding.task_completed` event arrives.
2. Tethers checks the supplied Facts.
3. If the project is software and files changed, Tethers proposes `lantern.task.record`.
4. The host decides whether the Action is permitted.
5. The host executes it through the declared Capability.
6. The complete decision and result are added to the Trail.

The Action does not write directly into Lantern Keeper’s database. It submits a request through Lantern Keeper’s public Capability. Lantern Keeper retains control over classification, deduplication, confidence, provenance, retention, rejection, and storage.

## Determinism

The complete deterministic input is:

```text
(
    protocol version,
    language version,
    Tether source and version,
    event envelope,
    immutable Fact snapshot,
    Capability schemas
)
```

Given the same complete input, Tethers must produce the same semantic Action Plan and evaluation Trail.

The engine must never secretly read:

- the system clock;
- environment variables;
- random values;
- the filesystem;
- the network;
- the host database;
- live application state;
- undeclared configuration.

If time or changing state matters, the host must supply it explicitly as event data or Facts.

Object key order is not semantically meaningful.

## Minimal language shape

A Tether has:

```text
Tether
    Anchor
    Conditions
    Actions
```

The Trail is produced while the rule is evaluated and executed; it is not written as part of the rule.

Version 0.1 operators:

```text
is
contains
greater_than
greater_than_or_equal
```

Version 0.1 values:

- quoted strings;
- integers;
- booleans;
- dotted references beginning with `anchor.`.

Version 0.1 deliberately excludes:

- loops;
- arithmetic;
- user-defined functions;
- arbitrary mutation;
- implicit I/O;
- hidden type coercion;
- parallel Actions;
- branching inside `do`;
- Conditions that inspect Action results;
- direct Action-result chaining.

Do not expand this list without an explicit semantic decision.

## Evaluation lifecycle

The engine must:

1. Parse the Tether.
2. Validate the protocol and language versions.
3. Validate the Tether’s structure.
4. Compare the Anchor with the incoming event.
5. Resolve Conditions using only the immutable Fact snapshot.
6. Stop evaluation on the first false Condition.
7. Validate each Action against an available Capability schema.
8. Resolve constants and `anchor.*` references.
9. Produce an ordered Action Plan.
10. Report the Effects required by that Plan.
11. Produce the deterministic evaluation portion of the Trail.

A false Condition is not an error. It produces `not_matched` and no Plan.

Malformed source, missing Facts, unknown Capabilities, missing arguments, incompatible versions, and type mismatches are errors.

## Action semantics

Actions are ordered.

The host executes them sequentially and stops on the first failure.

Version 0.1 does not allow one Action to inspect the result of an earlier Action. Result-dependent behaviour should use a new visible event:

```text
Action completes
    ↓
Host records result
    ↓
Host emits result event
    ↓
Another Tether evaluates that event
```

Example:

```tethers
tether "Pause after serious architectural conflict"

anchor
    coding.review_completed

when
    review.architecture_conflict is true
    and review.confidence greater_than_or_equal 85

do
    project.pause_task
        task: anchor.task
```

AI may produce the review, but the AI does not secretly determine the workflow response. Its result becomes visible data, and a Tether applies known policy to it.

## Capability model

Applications expose typed Capabilities.

Conceptual example:

```tethers
capability file.move
    version: "1.0.0"
    description: "Move a file into another folder"

    inputs
        file: File
        destination: Folder

    effects
        filesystem.read
        filesystem.write

    reversibility: reversible
```

Capability schemas describe:

- name;
- version;
- inputs;
- outputs;
- Effects;
- reversibility.

Capability schemas do not grant permission.

The language must not gain separate file, music, Lantern Keeper, GitHub, email, or AI modes. These are Capability sets, not grammar features.

## Permissions

Keep these responsibilities separate:

```text
Schemas describe.
Policies authorise.
Hosts enforce.
Trails record.
```

Tethers may report that a Plan requires:

```text
filesystem.read
filesystem.write
lantern.write
network.access
```

Only the host decides whether those Effects are allowed for that Tether, user, project, resource, and execution.

A Plan is a request, not permission.

Production hosts must enforce resource scope at the execution boundary. A Capability schema claiming safe behaviour must not be treated as proof that an adapter is safe.

## Identity and replay

The protocol uses:

- `event_id` — identifies the incoming host event;
- `evaluation_id` — identifies one Tether evaluation;
- `plan_id` — identifies the resulting Plan;
- `action_id` — identifies an Action’s position within the Plan;
- `idempotency_key` — prevents accidental duplicate execution.

The 0.1 idempotency key is:

```text
evaluation_id/action_id
```

Externally significant hosts should persist successful idempotency keys atomically with the external Effect whenever possible.

Retries must not duplicate emails, file moves, memory records, or other significant Effects.

## Trail ownership

The Trail has four stages:

1. Reception
2. Evaluation
3. Authorisation
4. Execution

Tethers writes deterministic Reception and Evaluation entries.

The host appends Authorisation and Execution entries, including:

- permission decisions;
- timestamps;
- Action starts;
- results;
- failures;
- retries;
- resulting event IDs.

Tethers must not claim an Action happened when it only proposed it.

Do not add wall-clock timestamps to deterministic engine output. Host timestamps belong to host-generated Trail entries.

## Reversibility

Do not describe all Actions as undoable.

Use these distinctions:

- `reversible` — the host can reliably restore the previous state;
- `compensatable` — another Action may counteract the Effect;
- `irreversible` — no meaningful automatic reversal exists.

“Undo support” means declared reversal or compensation, not magical rollback.

## Tether Sets

Tether Sets provide project-specific behaviour without hard-coding project modes into Lantern Keeper.

Possible software-project sets:

- Software Project Stewardship
- Architecture Protection
- Coding Handover
- Milestone Audit

Possible music-project sets:

- Song Version Tracking
- Mix Decision Memory
- Export Archiving
- Creative Direction Review

Possible research-project sets:

- Source Verification
- Contradiction Detection
- Evidence Confidence
- Research Digest

Tether Sets are future-facing. Do not implement package management or a Set marketplace during 0.1.

## What belongs in Tethers

Use Tethers where behaviour should be:

- visible;
- configurable;
- disableable;
- permissioned;
- explainable;
- auditable;
- replaceable;
- triggered across component boundaries.

Use ordinary Rust, OCaml, TypeScript, or other implementation code for:

- database internals;
- byte conversion;
- ordinary function calls;
- UI rendering;
- algorithms;
- low-level error handling;
- performance-critical machinery;
- internal operations required for the host to function.

Decision test:

> Would a project owner reasonably want to inspect, change, disable, or audit this behaviour?

If yes, it may belong in Tethers.

If not, it probably belongs in ordinary code.

## Tethers 0.1 goal

Version 0.1 must prove one complete round trip:

```text
Rust host
    sends event + Facts + Capability schemas + Tether
        ↓
OCaml engine
    parses + validates + evaluates
        ↓
OCaml engine
    returns Action Plan + evaluation Trail
        ↓
Rust host
    authorises + executes mock Capability
        ↓
Rust host
    appends execution Trail
```

Success criteria:

- textual Tether parses;
- Anchor matching works;
- supported Conditions evaluate correctly;
- missing or incorrectly typed inputs fail clearly;
- Action arguments resolve correctly;
- required Effects are reported;
- host permission checking works;
- mock Action execution works;
- idempotency works;
- golden response fixture passes;
- complete Trail is visible.

Do not add AI, GitHub, email, scheduling, adapters, package management, visual diagrams, or HQ before this round trip is reliable.

## Implementation structure

Expected prototype areas:

```text
tethers-0.1/
    README.md
    SPEC.md
    examples/
    protocol/
    engine-ocaml/
    host-rust/
    scripts/
```

Primary implementation:

```text
engine-ocaml/bin/main.ml
host-rust/src/main.rs
```

Protocol fixtures:

```text
protocol/request.json
protocol/expected-response.json
```

The engine communicates using newline-delimited JSON over standard input and output.

Do not introduce an FFI, local network service, database, or message broker for 0.1.

## Windows development

The primary current development machine is Windows.

Provide native PowerShell entry points for development tasks. Unix shell equivalents may remain for portability, but Windows verification must not require Bash or jq unnecessarily.

Native Windows opam is the preferred OCaml setup. Do not introduce WSL or Docker solely to compile this prototype unless Matthew explicitly chooses that route.

Do not install software without explicit permission.

## Working rules for Codex

Before every task:

1. Read this file.
2. Read `docs/CONSTITUTION.md`.
3. Read `tethers-0.1/SPEC.md`.
4. Read `docs/MCP_PLAN.md`.
5. Read `docs/CURRENT_GOAL.md`.
6. Read `docs/TASK_QUEUE.md`.
7. Inspect Git status.
8. Preserve unrelated and user-authored changes.

For OCaml implementation tasks, read the relevant section of
`docs/OCAML_GUIDE_FOR_AGENTS.md` and consult the linked official documentation
before relying on model memory for unfamiliar syntax, APIs, or tooling.

During work:

- Keep each implementation step under approximately 10 minutes.
- Do not expand scope to fill available time.
- Fix clear defects, not speculative future problems.
- Do not silently change language semantics.
- Do not add dependencies without justification.
- Keep Core application-agnostic.
- Maintain deterministic output.
- Keep protocol fixtures synchronized with intentional changes.
- Prefer small tests proving one behaviour.
- Do not replace working foundations to make a demonstration easier.
- Never modify permissions or safety boundaries merely to make a test pass.

After work:

1. Run the relevant tests.
2. Report exact results, including blocked tests.
3. List files changed.
4. Update `docs/CURRENT_GOAL.md`.
5. Update `docs/TASK_QUEUE.md`.
6. State the smallest useful next task.

## Project constitution

The authoritative project constitution is `docs/CONSTITUTION.md`. It governs
enduring design principles. `tethers-0.1/SPEC.md` remains authoritative for the
current precise 0.1 language and protocol semantics.

## Final scope warning

The main project risk is not whether Tethers can support many applications. It can.

The risk is allowing attractive future uses to enlarge the language before its core semantics are proven.

Files, GitHub, email, music software, calendars, AI agents, Home Assistant, and Lantern Keeper are demonstrations of the Capability model—not version 0.1 requirements.

The Core should remain stubbornly ignorant:

> It knows rules, values, schemas, Plans, and Trails.
> It does not know what an invoice, song, repository, email, or memory is.

```
