# Tethers Project Guidance For Coding Agents

## Start Here

Before changing the repository, read:

1. `docs/PROJECT_CONTROL.md`
2. `docs/AGENT_WORKFLOW.md`
3. `docs/CURRENT_CLINE_TASK.md`
4. `docs/PROJECT_DASHBOARD.md`

Before changing implementation code, also read
`docs/IMPLEMENTATION_LANGUAGE_STANDARD.md` and the task-relevant language guide.

Then read only the authoritative documents, code, tests, and worker notes named
by the current task packet. Do not load the complete project archive by default.

For OCaml work, read the task-relevant section of
`docs/OCAML_GUIDE_FOR_AGENTS.md`.

For every Rust task, read `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md` before the
first edit. Treat its fast safety scan, trust-boundary rules, subprocess
supervision rules, verification commands, stop conditions, and worker-note
schema as required operating guidance. Record the guide under the task's
required reading or worker-note evidence; do not merely rely on remembered
chat context.

## Current Operating Mode

**Gorilla Coding 🦄**

- Lucy in ordinary chat controls architecture, task compilation, GitHub-visible
  review, acceptance, and continuation.
- Cline is the default implementation owner for ordinary Green and Amber work.
- Codex handles Red implementation or sign-off, difficult local failure,
  Git/environment/recovery, and machine-required diagnosis.
- Matthew may paste Cline's concise report to Lucy as the normal return handoff.
- Copilot is not part of the active route.

No implementation agent invents or begins the next task. Lucy controls
continuation.

## Project Definition

Tethers is a small deterministic behaviour language and capability protocol for
connecting applications through clear, typed, permissioned rules.

> Apps provide the sockets. Tethers provides the cables.

A Tether means:

> When this event happens, check these known facts, then propose these permitted
> actions.

Tethers is a deterministic planner. It does not grant permission and does not
execute Actions.

## Authority Order

Use the narrowest applicable authority:

1. `docs/CONSTITUTION.md` for enduring Tethers design principles.
2. `tethers-0.1/SPEC.md` for precise 0.1 language and protocol semantics.
3. `docs/DECISIONS.md` for accepted design decisions.
4. `docs/CAPABILITY_BRIDGE.md` for the manifest, trust, and host bridge contract.
5. `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md` for implementation technique and
   language use. It never overrides product semantics or trust boundaries.
6. The current task packet for frozen scope and acceptance criteria.
7. Code, tests, fixtures, Trails, compiler output, and Git for implementation
   evidence.

`docs/TETHERS_LUCY_NOTES.md` is optional orientation, not specification.
Agent reports are claims until repository evidence verifies them.

## Core Boundary

```text
Host application
    supplies event + immutable Facts + Capability schemas + Tether source
        ↓
Tethers Core — OCaml
    parses, validates, evaluates, and proposes an ordered Plan
        ↓
Host application
    resolves policy, records durable intent, executes approved Actions,
    validates results, and appends host Trail entries
```

Keep these responsibilities separate:

```text
Schemas describe.
Policies authorise.
Hosts enforce.
Trails record.
```

Tethers Core must remain application-agnostic. Do not add Lantern Keeper,
GitHub, email, files, music, AI, or other product-specific grammar or branches.
Those belong in Capabilities, adapters, host policy, or host code.

## Canonical Vocabulary

Use these terms consistently:

| Term | Meaning |
| --- | --- |
| Tether | One behavioural rule |
| Tether Set | A collection of related Tethers |
| Anchor | Event that wakes a Tether |
| Fact | Immutable input available to Conditions |
| Condition | Deterministic test over a Fact |
| Action | Requested Capability invocation |
| Capability | Typed operation exposed by a host or adapter |
| Effect | External consequence declared by a Capability |
| Plan | Ordered Actions proposed by Tethers |
| Trail | Causal record of evaluation, authorisation, and execution |
| Host | Application supplying input and enforcing policy |
| Adapter | Component exposing another system as Capabilities |
| HQ | Future visual editor, tester, and Trail inspector |

Avoid casual synonyms when a canonical term applies.

## Non-Negotiable Invariants

- Given the same complete deterministic input, Core produces the same semantic
  Plan and evaluation Trail.
- Core does not secretly read the clock, environment, filesystem, network,
  database, live state, randomness, or undeclared configuration.
- Time and changing state must arrive explicitly as event data or Facts.
- A Plan is a request, not permission.
- The planner never inspects or trusts complete capability manifests.
- Current manifest and provider pins must be checked before dispatch.
- Structured scope without a host/binding-owned assessment fails closed.
- Do not infer argument-to-resource mappings without an approved binding or
  adapter contract.
- AI judgement is an explicit Capability Action whose structured result becomes
  visible data for a later Anchor. It never runs invisibly in Conditions.
- Actions are ordered and initially dispatched serially.
- No automatic retry until idempotency is proved end to end.
- Tethers must not claim that an Action happened when it only proposed it.
- Do not change 0.1 syntax or semantics without an explicit design gate.

## Current Language Shape

A Tether contains one Anchor, zero or more Conditions, and one or more Actions.
The current precise syntax is defined only by `tethers-0.1/SPEC.md`.

Do not use this guidance file as a substitute for the specification. In
particular, do not invent loops, arithmetic, functions, hidden coercion,
parallel Actions, branching inside `do`, or direct Action-result chaining.

## Working Rules

Before work:

1. Confirm packet state, owner, route, worker-note path, base commit, and expected
   pre-existing changes.
2. Before the first edit, confirm the exact worktree root, branch, `HEAD`, status,
   expected base, and any packet-named external toolchain paths. Do not assume
   ignored directories such as `_opam` exist in every worktree. Stop only when the
   worktree, branch, base, or required toolchain genuinely differs; otherwise
   continue without a separate preflight report.
3. Run the task-packet checker.
4. Read only packet-named context and task-relevant code.
5. Stop if another owner already has the task `IN_PROGRESS`.

During work:

- Keep the change bounded to the packet.
- Preserve unrelated and user-authored changes.
- Fix demonstrated defects, not speculative future problems.
- Use the implementation language idiomatically and to its appropriate depth.
  Do not make production code primitive merely so Matthew can read it; explain
  the design outside the code instead.
- Do not add dependencies or alter safety boundaries merely to make tests pass.
- Prefer focused tests that prove one required behaviour or failure branch.
- Stop when requirements conflict or a missing design decision blocks safe work.
- After two materially similar failed attempts, stop and return exact evidence
  plus one smallest unresolved question.

After work:

1. Run the required formatter, compiler, focused checks, relevant regression
   suite, integration scripts, and whitespace checks.
2. Inspect the complete diff and final Git status.
3. Write the worker note at the exact path named by the packet.
4. Update the packet to `COMPLETE` or `BLOCKED` with honest evidence.
5. Return the concise report defined by `docs/CLINE_HANDOFF.md`.
6. Stop. Do not select, compile, authorise, or begin the next task.

Do not commit, push, merge, amend, tag, publish, install, or open a pull request
unless the current task explicitly authorises it.

## Development Environment

- Active prototype tree: `tethers-0.1/`.
- Primary development environment: native Windows.
- Required automation shell: PowerShell 7 (`pwsh.exe`).
- Native Windows opam is the preferred OCaml setup.
- Do not introduce WSL, Docker, Bash, jq, a database, FFI, network service, or
  message broker merely for convenience.
- Do not install software without Matthew's explicit permission.

## Control Check

Run before handoff and before claiming completion:

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
```

The goal is not to use the most agents or produce the most documentation. The
goal is the least total compute and Matthew effort per accepted, correct change.
