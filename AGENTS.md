# Tethers Project Guidance For Coding Agents

## Mandatory startup gate

Some coding agents automatically load this `AGENTS.md` but do not automatically
load files merely because they are named inside it. A filename is not an import.

Before any repository mutation, immediately use the available file-reading tool
to read these files in full:

1. `docs/PROJECT_CONTROL.md`
2. `docs/AGENT_WORKFLOW.md`
3. `docs/CURRENT_CLINE_TASK.md`
4. `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md`
5. `docs/GIT_WORKTREES_AND_LINE_ENDINGS_FOR_AGENTS.md`

OpenCode's project configuration also names these files through `opencode.json`.
That is additional protection, not permission to assume they were loaded. The
agent must still verify the effective instruction context and explicitly read any
missing file.

Before editing, report:

- the detected repository root;
- the current branch and exact `HEAD`;
- the instruction files loaded automatically;
- the mandatory documents read explicitly;
- the current task owner, status, and risk colour;
- the authorised files and forbidden changes;
- every stop condition;
- the exact repeated-failure rule.

Do not edit until this startup report is complete. A task packet may require
additional reading but may not remove this gate.

After completing the report and before any task-specific tool assumption, run
`pwsh -NoProfile -File scripts/check-dev-tools.ps1`. It is the single
repository-owned diagnostic for `rg`, `fd`, `jq`, `yq`, `gh`, `just`, `git`, and
`pwsh`; stop and report a missing tool rather than guessing. The diagnostic is
read-only. If User PATH changed, start a fresh application process before
treating its result as the environment seen by Codex, OpenCode, VS Code, or a
terminal.

`docs/PROJECT_DASHBOARD.md` is a Matthew-facing summary, not implementation
authority and not part of the automatic OpenCode instruction set. Read it only
when the task concerns project reporting or the packet explicitly names it.

Before changing implementation code, also read the task-relevant language guide.
Then read only the authoritative documents, code, tests, and worker notes named
by the current task packet. Do not load the complete project archive by default.

For every OCaml task, read `docs/OCAML_GUIDE_FOR_AGENTS.md` before the first
edit. Treat its safety scan, Core/host boundary, explicit-switch contract,
verification commands, stop conditions, and worker-note schema as required
operating guidance. Record the guide under the task's required reading or
worker-note evidence; do not merely rely on remembered chat context.

For every Rust task, read `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md` before the
first edit. Treat its fast safety scan, trust-boundary rules, subprocess
supervision rules, verification commands, stop conditions, and worker-note
schema as required operating guidance. Record the guide under the task's
required reading or worker-note evidence; do not merely rely on remembered
chat context.

For Git topology, branch publication, worktree, line-ending or encoding
investigation, history recovery, or destructive Git tasks, read
`docs/GIT_WORKTREES_AND_LINE_ENDINGS_FOR_AGENTS.md` before the first Git
mutation. It supports the task packet; it does not replace its authority.

## Current Operating Mode

**Gorilla Bunny Coding Shop 🦍🐇**

- Matthew supplies product direction, taste, priorities and final human
  judgement. He may be the short copy/paste relay when that keeps him visibly
  in the loop.
- Lucy controls architecture, task compilation, evidence review, acceptance,
  routing and improvement of the shop itself.
- Gem is Lucy's peer technical sparring partner when a difficult or
  consequential decision benefits from a second senior technical view.
- Agents and tools are replaceable specialists selected for task fit, risk,
  local-machine needs and economics; no particular coding agent has a permanent
  role.

Historical filenames, branches, packets and worker notes may retain earlier
agent names. They do not define the active route.

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

1. Complete the mandatory startup report above.
2. Confirm packet state, owner, route, worker-note path, base commit, and expected
   pre-existing changes.
3. Before the first edit, confirm the exact worktree root, branch, `HEAD`, status,
   expected base, and any packet-named external toolchain paths. Do not assume
   ignored directories such as `_opam` exist in every worktree. Stop only when
   the worktree, branch, base, or required toolchain genuinely differs.
4. Run the task-packet checker.
5. Read only packet-named context and task-relevant code.
6. Stop if another owner already has the task `IN_PROGRESS`.

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
- Solve ordinary implementation problems (compile/test failures, stale fixtures,
  mechanical migration fallout, minor helpers, formatter/lint fallout) locally
  once delegated; reserve `BLOCKED` for a contradictory frozen architecture,
  disproven architectural assumptions, a consequential protected
  architecture/product/security/trust decision, or genuinely unavailable
  tooling/data/credentials.
- After two materially similar failed attempts, stop and return exact evidence
  plus one smallest unresolved question. Repeated failures escalate only when
  they are repeated unsuccessful approaches to the same underlying problem;
  several tests/files exposing one understood migration issue are not separate
  failed attempts.

After work:

1. For a Rust-changing task, before the implementation checkpoint, run the
   packet's Cargo formatter command and inspect its immediate diff. Stop if
   rustfmt changes any file outside the authorised Rust paths. For a non-Rust
   or evidence-only task, run `cargo fmt --all -- --check` only; never use a
   mutating formatter or change Rust source.
2. Run the required compiler, focused checks, relevant regression
   suite, integration scripts, and whitespace checks.
3. Inspect the complete diff and final Git status.
4. Write the worker note at the exact path named by the packet.
5. Update the packet to `COMPLETE` or `BLOCKED` with honest evidence.
6. For `COMPLETE`, push the finished branch normally to `origin`, resolve the
   remote branch HEAD, confirm that it exactly equals local `HEAD`, and confirm
   clean Git status. Include the full remote SHA, equality result, and status in
   the completion report.
7. Return the concise report defined by `docs/CLINE_HANDOFF.md`; that historical
   filename applies to every named implementation owner.
8. Stop. Do not select, compile, authorise, or begin the next task.

Do not merge, amend, tag, publish beyond the required normal branch push,
install, or open a pull request unless the current task explicitly authorises
it. Every `COMPLETE` task requires its finished branch to be pushed normally to
`origin`; this does not authorise force-pushes, direct updates to `main`, or any
other publication.

## Development Environment

- Active prototype tree: `tethers-0.1/`.
- Primary development environment: native Windows.
- Required automation shell: PowerShell 7 (`pwsh.exe`).
- Native Windows opam is the preferred OCaml setup.
- Do not introduce WSL, Docker, Bash, jq, a database, FFI, network service, or
  message broker merely for convenience.
- Do not install software without Matthew's explicit permission.

## Agent toolset

- `rust-analyzer` is a Rust 1.97.1 toolchain component for navigation and LSP
  feedback. Compiler, Clippy, tests, and contracts remain authority when LSP
  feedback differs or becomes stale. Reread or compile when LSP state may be
  out of date.
- `cargo-nextest` provides an alternative test loop for agents. Retries are
  forbidden. Ordinary `cargo test` remains the final completion authority.
- `cargo-deny` is the single accepted dependency-policy gate for licences,
  bans, sources, and advisories. Do not add `cargo-audit`.
- `cargo-machete` is an advisory unused-dependency detector. Treat findings as
  questions, never deletion authority. Never run `cargo machete --fix`.
- Do not add `cargo-semver-checks` without a later decision.
- `scripts/install-rust-agent-tools.ps1` installs the exact frozen toolset.
  `scripts/check-rust-agent-tools.ps1` is a read-only non-mutating checker.
- `scripts/start-opencode-lsp.ps1` is an opt-in launcher for the next OpenCode
  process. It sets `OPENCODE_EXPERIMENTAL_LSP_TOOL=true` and
  `OPENCODE_DISABLE_LSP_DOWNLOAD=true` process-locally and restores previous
  values on exit. Supply `-OpenCodePath`, set process-local `OPENCODE_BIN`, or
  rely on `opencode` already resolved from PATH; the launcher never changes PATH.

## Control Check

Run before handoff and before claiming completion:

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
```

The goal is not to use the most agents or produce the most documentation. The
goal is the least total compute and Matthew effort per accepted, correct change.
