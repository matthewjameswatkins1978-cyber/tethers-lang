# Tethers OCaml Engineering Guide

**Status:** Engineering guidance for Tethers Core and its OCaml protocol surfaces<br>
**Primary audience:** AI coding agents, especially Goose, Cline, Codex, and future implementation agents<br>
**Secondary audience:** Senior OCaml engineers and reviewers<br>
**Repository context:** approved against published `main` at `0718288984773257b5a10785a2d1ed2cfcbcc831`; each future task packet must record its own exact current `origin/main` base.<br>
**Target compiler baseline for this guide:** OCaml `5.5.0`<br>
**Observed local toolchain:** opam `2.5.2`, Dune `3.24.0`, Yojson `2.2.2`<br>
**Dune language declared by the repository:** `3.10`<br>
**Revision:** 1.2<br>
**Last reviewed:** 30 July 2026

---

## 1. Purpose

This guide explains how OCaml should be used to implement the deterministic core of Tethers.

It is not an OCaml tutorial. It is not a replacement for the Tethers specification. It is not permission to redesign the language, move host responsibilities into Core, or replace project contracts with an agent's preferred architecture.

Its purpose is to prevent capable humans and AI agents from writing OCaml that is locally plausible but semantically wrong, especially code that:

- adds permission, execution, provider discovery, retries, storage, or application behaviour to Tethers Core;
- changes the Tethers 0.1 language while pretending to perform a refactor;
- weakens deterministic ordering or allows hidden I/O into evaluation;
- accepts JSON forms or implicit coercions that the 0.1 specification rejects;
- changes stable error codes, envelope shapes, or Trail ordering accidentally;
- treats Yojson trees as the internal domain model;
- invents functions from Rust, F#, Haskell, Base, Core, or another ML environment;
- uses OCaml or Dune features newer than the repository's supported baseline;
- introduces concurrency, effects, functors, GADTs, or other advanced machinery without a concrete present need;
- produces large rewrites whose semantic equivalence is difficult to prove;
- claims tests passed without running the exact required commands.

The required result is direct, idiomatic OCaml whose types, modules, and data flow preserve Tethers' small deterministic architecture.

---

## 2. Fast safety scan for agents

Run this scan before every OCaml edit.

1. Am I adding permission, dispatch, retries, provider calls, storage, clock access, filesystem access, network access, environment access, or live discovery to Tethers Core? **Stop.**
2. Am I changing Tether syntax, operator meaning, Action ordering, error classification, identity construction, or Trail order without an explicit specification task? **Stop.**
3. Am I making an AI call inside a Condition or otherwise introducing hidden semantic judgement? **Stop.**
4. Am I accepting floats, nulls, arrays, arbitrary objects, or implicit conversions because Yojson can represent them? **Stop and check the 0.1 contract.**
5. Am I changing a stable error code, message, response envelope, or fixture merely to make a refactor easier? **Stop.**
6. Am I using a wildcard match over a closed Tethers state where a new constructor should trigger a compiler warning or error? **Stop and make the match explicit.**
7. Am I relying on an OCaml 5.6, Dune 3.11+, Base, Core, Batteries, or third-party API that is not in the active project environment? **Verify first.**
8. Am I placing diagnostics, banners, or logs on stdout used by the JSON-lines or MCP protocol? **Stop.**
9. Am I introducing mutation or concurrency where explicit immutable data flow already expresses the operation? **Stop and justify the change.**
10. Am I rewriting adjacent code merely because I dislike its style? **Leave it alone unless the task authorises that work.**
11. Have I made two materially similar failed attempts? **Stop and return exact evidence instead of burning compute.**
12. Am I about to claim completion without compiler, fixtures, protocol tests, diff inspection, and Git evidence? **Do not claim completion.**

A stopped item is not necessarily forbidden forever. It means the edit has crossed a design, compatibility, or authority boundary that the agent may not invent.

---

## 3. Authority order

Before modifying OCaml code, use the narrowest applicable authority in this order:

1. The current authorised task packet and any explicit design attached to it for scope and acceptance.
2. `tethers-0.1/SPEC.md` for precise Tethers 0.1 language and protocol semantics.
3. `docs/CONSTITUTION.md` for enduring product principles.
4. `docs/architecture/TETHERS_LANTERN_KEEPER_CANONICAL_ARCHITECTURE.md` for the Core, host, provider, AI, and Lantern Keeper boundaries.
5. Specific accepted designs and bridge contracts relevant to the task.
6. `docs/DECISIONS.md` for recorded decisions.
7. `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md` for implementation technique.
8. This guide for OCaml-specific practice.
9. Existing code, fixtures, compiler output, and worker notes as implementation evidence.

The current task packet is authoritative about what may change. It is not allowed to contradict the specification, Constitution, or accepted trust boundaries silently.

Existing code is evidence, not automatic authority. A local pattern may be transitional, incomplete, or superseded by a later accepted decision.

When authorities conflict, an AI must not reconcile them by invention. It must report:

- the exact conflicting statements;
- the affected files and behaviour;
- why implementation cannot safely continue;
- the smallest decision required from Matthew or the designated architect.

### 3.1 Repository authority and modification matrix

| File or path | Role | Normal modification rule |
| --- | --- | --- |
| `tethers-0.1/SPEC.md` | Precise 0.1 semantics | Read-only unless the task explicitly changes the specification |
| `docs/CONSTITUTION.md` | Enduring product principles | Read-only unless Matthew explicitly authorises a constitutional change |
| `docs/architecture/*.md` | Accepted architecture | Read-only unless the task is an architecture revision |
| `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md` | Engineering technique | Read-only during ordinary implementation |
| `docs/CURRENT_CLINE_TASK.md` or current task packet | Immediate scope and acceptance | Modify only through the project-control workflow |
| `tethers-0.1/engine-ocaml/bin/*.ml` | Core, protocol, evaluator, and MCP implementation | Modify only inside the authorised task boundary |
| `tethers-0.1/engine-ocaml/bin/*.mli` | Future explicit module interfaces | Add or modify only when a stable boundary is part of the task |
| `tethers-0.1/engine-ocaml/dune-project` | Dune language and package declaration | Change only through an explicit toolchain/build decision |
| `tethers-0.1/engine-ocaml/bin/dune` | Executable and module graph | Change only when the task requires build graph changes |
| `tethers-0.1/engine-ocaml/tethers_engine.opam` | Package compatibility constraints | Change only through an explicit dependency or compiler policy decision |
| Protocol fixtures | Cross-language behavioural contract | Modify only for an authorised semantic or compatibility change |
| PowerShell verification scripts | Repository-level contract verification | Modify only when the verification contract itself changes |
| Worker notes | Durable implementation evidence | Update exactly as required by the task packet |

Do not revise a contract to fit an implementation.

---

## 4. Where OCaml fits in Tethers

The central division is:

> Lantern Keeper remembers. Tethers coordinates. AI interprets. Matthew decides.

For implementation work, the narrower boundary is:

### Tethers Core, implemented in OCaml, owns

- parsing Tether source;
- validating the language version and structural rules;
- representing Tethers language values and syntax as typed OCaml data;
- matching the Anchor against the supplied event name;
- resolving Conditions against an immutable Fact snapshot;
- stopping on the first false Condition;
- validating planned Actions against the supplied capability descriptions;
- resolving literal and `anchor.*` Action arguments;
- constructing deterministic Action IDs, Plan IDs, and planner-owned idempotency material as specified;
- producing the deterministic evaluation portion of the Trail;
- returning `matched`, `not_matched`, or structured evaluation errors;
- exposing the same evaluator through the JSON-lines executable and the MCP adapter;
- validating Tether source without executing Actions.

### The Rust runtime host owns

- provider configuration and connection;
- manifest admission and trusted provider binding;
- live capability resolution;
- local permission policy and one-shot approval;
- resource-scope enforcement;
- durable replay admission and execution identity;
- serial one-shot provider dispatch;
- deadlines, cancellation, failure, and uncertainty classification;
- host Trail entries and durable audit behaviour;
- result Anchors and causal event queuing;
- Windows persistence and process-supervision proof boundaries.

### OCaml Core must not own

- permission grants or denials;
- provider trust or self-identification;
- provider process management;
- live discovery;
- credentials;
- retries;
- durable replay state;
- wall-clock timestamps;
- filesystem, network, environment, process, database, or random access;
- application-specific rules for Lantern Keeper, GitHub, email, files, music, or AI;
- hidden AI calls inside Conditions;
- a second host execution state machine;
- guessed compatibility with newer language, protocol, or capability versions.

Tethers Core is a deterministic planner. A Plan is a request, not permission.

### 4.1 Core and host responsibility matrix

| Responsibility | OCaml Core (`engine-ocaml`) | Rust host (`host-rust`) |
| --- | --- | --- |
| Tether parsing and typed syntax | **Exclusive owner** | Must not parse or reinterpret Tether source |
| Condition evaluation | **Exclusive owner** over immutable supplied Facts | Must not re-evaluate Conditions |
| Action planning | **Exclusive owner** of ordered proposed Actions | Validates and executes only an accepted Plan |
| Permission policy | Forbidden | **Exclusive owner** |
| Provider discovery and dispatch | Forbidden | **Exclusive owner** |
| Replay, durability, and filesystem proof | Forbidden | **Exclusive owner** |
| Trail | Deterministic evaluation entries without timestamps | Permission, intent, execution, outcome, and wall-clock entries |
| Application-specific behaviour | Forbidden in grammar or evaluator branches | Exposed through providers, adapters, policy, and Capabilities |

If a change makes both languages independently decide the same semantic fact, the architecture has already drifted. One side must own the decision and the other must consume a typed result.

---

## 5. Why OCaml fits Tethers Core

OCaml is not being used because it is fashionable, obscure, or academically impressive. It fits the specific shape of Tethers Core.

### 5.1 Algebraic data types match a small language

Tethers contains closed sets of concepts:

- values;
- operators;
- parser outcomes;
- Condition outcomes;
- planning outcomes;
- server states;
- protocol response shapes.

OCaml variants model these directly. Pattern matching then makes every supported case visible and lets the compiler expose forgotten cases.

For example:

```ocaml
type value =
  | String_value of string
  | Int_value of int
  | Bool_value of bool
  | Reference of string

type operator =
  | Is
  | Contains
  | Greater_than
  | Greater_than_or_equal
```

This is a better fit than strings such as `"contains"` flowing through the evaluator after parsing.

### 5.2 Immutable data supports deterministic evaluation

Tethers evaluation is a pure transformation from declared inputs to a Plan or explanation.

OCaml's ordinary lists, variants, records, and local bindings are immutable by default. This encourages explicit data flow and makes hidden state less likely.

The same complete deterministic input must produce the same semantic output. OCaml does not guarantee this automatically, but its normal style makes the required design natural.

### 5.3 Pattern matching keeps semantics visible

The Tethers language has exact rules. Pattern matching lets implementation follow those rules without a forest of booleans, casts, and string comparisons.

A reviewer can see which combinations are accepted and which produce an error:

```ocaml
match condition.operator, actual, condition.expected with
| Is, String_value left, String_value right -> left = right
| Is, Int_value left, Int_value right -> left = right
| Is, Bool_value left, Bool_value right -> left = right
| Contains, String_value left, String_value right -> contains left right
| Greater_than, Int_value left, Int_value right -> left > right
| Greater_than_or_equal, Int_value left, Int_value right -> left >= right
| _ -> fail "type_error" (...)
```

### 5.4 Modules suit a small acyclic core

Tethers Core currently has a small module graph:

```text
main
  -> Tethers_evaluator
      -> Tethers_protocol
      -> Tether_parser

Tethers_protocol
  -> Tether_parser

Tethers_mcp_main
  -> Tethers_mcp_server
      -> Tethers_evaluator
      -> Tether_parser
```

OCaml compilation units and future `.mli` interfaces can keep parser, protocol, evaluation, and transport responsibilities separate without building a framework.

### 5.5 Native compilation and a small dependency surface fit a local engine

The current engine compiles to native executables through Dune and depends directly only on Yojson. This keeps the planner process small and operationally simple.

Performance is not the main reason for OCaml, but a typed native core avoids requiring a large runtime or framework for a small deterministic operation.

### 5.6 Process separation reinforces the trust boundary

The OCaml planner and Rust host may run as separate local processes. The versioned JSON protocol and MCP are cables between them.

This separation is useful because it makes the boundary explicit:

- Core receives immutable input and returns a Plan;
- the host decides whether and how the Plan may execute.

Do not erase that separation by moving host policy into OCaml or duplicating language semantics in Rust.

### 5.7 Where OCaml does not fit as well

Rust is the better home for the current effectful host boundaries:

- durable replay;
- process ownership;
- Windows-specific filesystem proofs;
- provider sessions;
- execution authority;
- one-shot approval ownership;
- strict external-call uncertainty handling.

The choice is not “OCaml versus Rust.” The architecture uses each language for the part it expresses and protects best.

---

## 6. Current verified technical baseline

### 6.1 Repository declarations

The package currently declares:

```opam
depends: [
  "ocaml" {>= "5.1.0" & < "6.0.0"}
  "dune" {>= "3.10"}
  "yojson" {>= "2.0.0" & < "3.0.0"}
]
```

The Dune project declares:

```lisp
(lang dune 3.10)
(name tethers_engine)
```

The executable stanza builds:

```text
tethers_engine
  modules: main, tether_parser, tethers_protocol, tethers_evaluator

tethers_mcp_server
  modules: tethers_mcp_main, tethers_mcp_server,
           tether_parser, tethers_protocol, tethers_evaluator
```

### 6.2 Observed local environment

The existing project guide recorded the following environment on 20 July 2026:

- OCaml compiler: `5.5.0`
- opam: `2.5.2`
- Dune: `3.24.0`
- Yojson: `2.2.2`
- PowerShell: `7.6.4`
- local switch directory: `tethers-0.1/engine-ocaml/_opam/`

This guide therefore targets **OCaml 5.5.0**.

### 6.3 Version truth and the reproducibility gap

Agents must understand four different version facts:

1. `ocaml >= 5.1.0 and < 6.0.0` is the package compatibility declaration.
2. OCaml `5.5.0` is the observed project working compiler and the engineering baseline for this guide.
3. Dune `3.24.0` is the observed installed tool, but `(lang dune 3.10)` limits the Dune language features the project declares.
4. Yojson `2.2.2` is the observed installed version, but the package allows any compatible `2.x` release.

The repository currently does not include a locked opam package file proving the exact complete dependency resolution. The `_opam` directory is local environment state, not portable repository truth.

Therefore:

- do not use OCaml APIs introduced after 5.5.0;
- do not use Dune language features introduced after 3.10 unless a separate task raises `(lang dune ...)`;
- do not code from “latest Yojson” documentation without checking the installed package;
- do not quietly tighten or widen compiler and dependency constraints during unrelated work;
- do not claim the exact toolchain is reproducible merely because it exists on one machine.

### 6.4 Approved version policy and enforcement state

Matthew approved `TOOLCHAIN-BASELINE-01` on 30 July 2026 with these OCaml rules:

- exact verified compiler: OCaml `5.5.0`;
- ordinary package compatibility: `>= 5.5.0 and < 5.6.0`;
- Dune language remains `3.10`;
- locked Dune executable: `3.24.0`;
- locked Yojson: `2.2.2`;
- a committed `tethers_engine.opam.locked` records the accepted resolution;
- task and agent invocations supply an explicit absolute OCaml directory-switch path.

The decision is approved but remains unenforced until its implementation packet is accepted. Until the repository files change, agents must distinguish the approved target from the still-broader checked-in package constraint.

Do not independently implement or partially imitate the baseline during another task. Toolchain enforcement is one bounded Amber packet with a publication gate and independent review.

---

## 7. Toolchain discipline on native Windows

Tethers' primary development environment is native Windows. Use PowerShell 7 through `pwsh.exe`.

Do not introduce WSL, Docker, Bash, `jq`, a database, FFI, or a network service merely to make an OCaml edit more convenient.

Do not install or upgrade software without explicit permission.

### 7.1 Explicit directory-switch behaviour

The verified OCaml environment is a directory switch whose contents live beneath an authorised checkout's `_opam` directory. Tethers uses multiple Git worktrees, while `_opam` is ignored. Therefore the current source worktree may legitimately have no `_opam` of its own.

Every task that invokes OCaml tooling must supply the exact absolute directory-switch root, for example:

```text
D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml
```

That path identifies the directory containing `_opam`; it is not the `_opam` path itself.

Do not infer the switch from the current directory. Do not search parent, sibling, or neighbouring worktrees. Do not fall back to a named or global switch. A current-worktree switch may be used only when its exact root was explicitly supplied.

The switch path selects the tool environment. The current Git worktree remains the source tree being built. An agent working in one worktree must not accidentally build the checkout that happens to own the shared switch.

### 7.2 Inspect the explicit switch before coding

Task packets must provide `OcamlSwitchPath`. Use it for every opam, compiler, Dune, and package query:

```powershell
$OcamlSwitchPath = "<absolute authorised directory-switch root>"
$EngineSourcePath = Resolve-Path .\tethers-0.1\engine-ocaml
$ExpectedPrefix = Join-Path $OcamlSwitchPath "_opam"

if (-not [System.IO.Path]::IsPathFullyQualified($OcamlSwitchPath)) {
  throw "OcamlSwitchPath must be absolute"
}
if (-not (Test-Path -LiteralPath $ExpectedPrefix -PathType Container)) {
  throw "The supplied OCaml switch does not contain _opam: $OcamlSwitchPath"
}

opam switch show --switch=$OcamlSwitchPath
opam var prefix --switch=$OcamlSwitchPath
opam exec --switch=$OcamlSwitchPath -- ocamlc -version
opam exec --switch=$OcamlSwitchPath -- ocamlopt -version
opam exec --switch=$OcamlSwitchPath -- dune --version
opam list --switch=$OcamlSwitchPath --installed --short --columns=name,version |
  Select-String -Pattern "^(ocaml|ocaml-base-compiler|dune|yojson)\s"
```

Confirm that the switch root resolves to the supplied canonical absolute path and that its prefix resolves to `<OcamlSwitchPath>\_opam`. On Windows, compare canonical paths case-insensitively.

The approved baseline is OCaml `5.5.0`, Dune `3.24.0`, and Yojson `2.2.2`. If the explicit switch differs, stop. Do not create, install, update, repair, relink, or replace it without separate authorisation.

### 7.3 Build the current worktree through the explicit switch

Run Dune from the current worktree source directory while selecting tools from the explicit switch:

```powershell
$OcamlSwitchPath = "<absolute authorised directory-switch root>"
$EngineSourcePath = Resolve-Path .\tethers-0.1\engine-ocaml

Push-Location $EngineSourcePath
try {
  opam exec --switch=$OcamlSwitchPath -- dune build
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
finally {
  Pop-Location
}
```

Compile after each coherent edit rather than making a large speculative batch. Compiler diagnostics are evidence about the actual language, switch, and module graph.

### 7.4 Do not alter global opam state casually

Avoid unrelated commands such as:

```powershell
opam update
opam upgrade
opam switch set <other-switch>
opam install <new-package>
```

These may change dependency resolution or machine state outside the task.

Use read-only inspection first. Any install, pin, compiler change, or lock-generation step requires task authority.

---

## 8. Governing OCaml rule

Use OCaml fully, but selectively.

Use a language feature when it makes Tethers semantics more accurate, a state harder to misuse, a module boundary clearer, or a proof easier to review.

Do not avoid a suitable feature merely because it is advanced or unfamiliar to Matthew. Explain the feature in the task packet or worker note instead of making production code primitive.

Do not use a feature merely because it is elegant in isolation.

Use this decision order:

1. Preserve specified Tethers behaviour and compatibility.
2. Preserve the Core and host boundary.
3. Make invalid language or evaluation states difficult to construct.
4. Keep effects and mutable state outside deterministic evaluation.
5. Preserve semantic ordering.
6. Use OCaml idiomatically.
7. Minimise conceptual surface area.
8. Optimise only after measurement.
9. Prefer brevity only when it removes noise rather than meaning.

The target is not beginner OCaml. The target is not clever OCaml. The target is code whose structure matches the Tethers domain and can be continued by another senior engineer or capable AI.

---

## 9. Model the language, not the input file

The parser should convert text into a typed internal representation. Later stages should not repeatedly inspect raw source strings to rediscover semantics.

Current core language types include:

```ocaml
type value =
  | String_value of string
  | Int_value of int
  | Bool_value of bool
  | Reference of string

type operator =
  | Is
  | Contains
  | Greater_than
  | Greater_than_or_equal

type condition = {
  fact : string;
  operator : operator;
  expected : value;
  source : string;
}

type action = {
  capability : string;
  arguments : (string * value) list;
}

type tether = {
  title : string;
  anchor : string;
  conditions : condition list;
  actions : action list;
}
```

This is the correct direction: syntax is parsed once into domain types.

### 9.1 Distinguish states with variants

Use variants when outcomes are mutually exclusive.

Current examples:

```ocaml
type condition_result =
  | Conditions_matched of int * Yojson.Safe.t list
  | Condition_not_matched of int * Yojson.Safe.t list
  | Condition_error of string * string * Yojson.Safe.t list

type action_planning_result =
  | Actions_planned of Yojson.Safe.t list * string list * Yojson.Safe.t list
  | Action_planning_error of string * string * Yojson.Safe.t list
```

Future improvements may replace raw strings or Yojson values inside these variants with stronger types, but such work must be bounded and fixture-protected.

Do not replace variants with combinations such as:

```ocaml
{ matched : bool; error : string option; trail : ... }
```

That form permits contradictory states.

### 9.2 Keep trusted and untrusted representations separate

A useful design direction is:

```text
untrusted JSON request
-> structurally decoded request
-> validated versions and identities
-> parsed Tether AST
-> validated capability descriptions
-> deterministic evaluation state
-> typed Plan representation
-> encoded response JSON
```

The current code still constructs some response structures directly as Yojson. Do not launch a broad rewrite solely to remove every Yojson value.

When an authorised task naturally touches a boundary:

- decode once;
- validate once;
- convert to a project type;
- keep Yojson at the protocol edge;
- do not pass arbitrary JSON trees through the semantic core when a closed type is known.

### 9.3 Records should carry meaning

Use records for groups of named values that travel together and have one invariant.

Avoid long tuples once the meaning or order becomes unclear.

A tuple is acceptable for a very local pair such as `(name, value)`. It is a poor long-term representation for several identities, sequence numbers, and Trail values whose order must be remembered.

### 9.4 Constructors and invariants

When a type requires validation, expose a function that performs the validation rather than allowing callers to build invalid values freely.

Once `.mli` files are introduced, keep raw record fields private where invalid construction would weaken a language or protocol invariant.

---

## 10. Parser engineering

The parser implements a frozen, intentionally small 0.1 grammar. It is not a general parser-combinator playground.

### 10.1 Preserve the declared grammar

The specification currently defines:

```text
tether      := 'tether' STRING NEWLINE anchor conditions actions
anchor      := 'anchor' NEWLINE INDENT NAME NEWLINE
conditions  := 'when' NEWLINE condition (NEWLINE condition)*
condition   := INDENT ['and'] PATH OPERATOR VALUE
actions     := 'do' NEWLINE action (NEWLINE action)*
action      := INDENT NAME NEWLINE argument+
argument    := INDENT INDENT NAME ':' VALUE_OR_PATH
```

Supported Condition operators are exactly:

- `is`
- `contains`
- `greater_than`
- `greater_than_or_equal`

Supported literal values are:

- quoted strings;
- integers;
- `true` and `false`.

Action arguments may also contain dotted references beginning with `anchor.`.

Do not invent:

- arithmetic;
- loops;
- functions;
- inline branching;
- implicit Action chaining;
- arrays or object literals;
- null;
- floats;
- aliases for operators;
- optional commas or alternative indentation;
- hidden coercions;
- comments unless explicitly designed;
- escape syntax not defined by the specification.

### 10.2 Parse once, reject early

Structural errors belong at the parser or request-validation boundary.

Examples include:

- malformed section order;
- wrong indentation;
- missing `do` section;
- missing Actions;
- malformed arguments;
- duplicate argument names within one Action;
- `anchor.*` references used as Condition expected values;
- unsupported literal forms;
- unknown operators.

Do not allow malformed source to reach evaluation and then fail as a different error.

### 10.3 Preserve exact error contracts

Existing parse error codes and messages are observable behaviour protected by fixtures.

A mechanical refactor must not casually change:

- capitalisation;
- punctuation;
- spacing;
- path wording;
- whether the error is minimal or correlated;
- the order in which malformed conditions are detected.

An authorised semantic change may revise an error contract, but it must update the specification and fixtures deliberately.

### 10.4 Indentation and line endings are semantic

The parser currently requires exact four-space and eight-space indentation.

Do not run a formatter over embedded Tether fixture strings without checking that their spaces remain exact.

Do not trim leading whitespace before indentation validation.

Tether source must be split on `\n`. After splitting, remove at most one terminal `\r` from each line before tokenisation and indentation checks. Preserve every leading space exactly. This accepts both LF and CRLF source without turning tabs, two-space indentation, trailing interior carriage returns, or other malformed whitespace into valid syntax.

The current project helper follows the required shape:

```ocaml
let remove_terminal_cr line =
  let length = String.length line in
  if length > 0 && line.[length - 1] = '\r' then
    String.sub line 0 (length - 1)
  else
    line

let source_lines source =
  source
  |> String.split_on_char '\n'
  |> List.map remove_terminal_cr
```

Line-ending normalisation must never trim or rewrite semantically significant leading indentation.

### 10.5 Strings and escaping

The current parser recognises a quoted string by its first and final double quote and takes the interior substring. It does not currently implement a general escape language.

An agent must not assume that `\"`, Unicode escapes, multiline strings, or JSON string rules automatically apply to Tether source.

Adding string escapes is a language design task, not a local parser improvement.

### 10.6 Avoid accidental quadratic behaviour in growth work

The current 0.1 source size is small, so simple list operations are acceptable. However, agents should recognise operations such as repeated `@` append and repeated `List.mem` scans.

Do not optimise them speculatively.

If real profiling or larger Tether Sets demonstrate a cost:

- preserve order explicitly;
- prefer accumulator plus `List.rev` for linear construction;
- use a set or map only when its ordering and comparison semantics are specified;
- add property or scale tests before and after the change.

Performance work must not alter error order or Trail order.

---

## 11. Deterministic evaluation

The deterministic input is:

```text
(language version,
 Tether source and version,
 event envelope,
 Fact snapshot,
 capability schemas)
```

The same input must produce byte-equivalent semantic output. JSON object-key order is not semantically meaningful, but array ordering and declared sequence are meaningful.

### 11.1 Core must remain pure with respect to external state

Evaluation must not read:

- wall-clock time;
- monotonic time;
- environment variables;
- filesystem state;
- network state;
- process state;
- database state;
- random values;
- host configuration not supplied in the request;
- provider availability not supplied by the host.

If time or changing state matters, the host supplies it in the event or Facts.

### 11.2 Preserve evaluation order

The 0.1 lifecycle is:

1. Parse Tether source.
2. Validate protocol and language versions.
3. Extract stable identities.
4. Compare the Anchor with the incoming event name.
5. Evaluate Conditions in source order.
6. Stop on the first false Condition.
7. If all Conditions match, plan Actions in source order.
8. Validate every Action against the supplied capability description.
9. Resolve arguments.
10. Return ordered Actions, required Effects, and deterministic Trail entries.

A refactor that evaluates all Conditions eagerly and then finds the first false one later may alter which error appears and which Trail entries exist. That is a semantic change.

### 11.3 False is not an error

A false Condition returns a successful `not_matched` result with no Plan.

Do not turn it into an exception or error response.

### 11.4 Missing and mistyped data are errors

A missing Fact or incompatible operator/value combination is an evaluation error.

Once reliable identities and an Anchor match exist, these errors use the correlated response shape and include the deterministic Trail accumulated so far.

### 11.5 Action results do not feed later Actions directly

Tethers 0.1 does not allow one Action's result to become an argument to the next Action in the same Plan.

Multi-step behaviour uses result Anchors:

```text
Anchor A
-> deterministic evaluation
-> Action: ai.judge
-> host/provider result
-> capability.succeeded Anchor
-> deterministic evaluation of another Tether
```

Do not add an evaluator-local result environment or hidden imperative pipeline.

### 11.6 Exact versions, not “latest”

The Core validates declared protocol and language versions exactly according to the contract.

It must reject incompatible versions rather than guessing an upgrade path.

Do not apply semantic-version compatibility rules unless the specification explicitly defines them for that boundary.

---

## 12. Capability descriptions and Action planning

Core receives capability descriptions from the host. It does not discover providers and does not trust complete manifests as permission.

### 12.1 What Core may use

For planning, the current protocol includes:

- capability name;
- exact capability version;
- input names and primitive types;
- declared Effects;
- bridge pins required by the current bridge contract when present.

Core uses this information to validate and construct a Plan.

### 12.2 What Core must not infer

Core must not infer:

- provider permission;
- provider identity truth;
- resource scope;
- live availability;
- retry safety;
- credential access;
- whether a destructive operation is acceptable;
- whether an AI recommendation should be obeyed.

Schemas describe. Policies authorise. Hosts enforce. Trails record.

### 12.3 Capability names are unique within a request

Duplicate capability names are rejected before evaluation.

Actions address capabilities by name, so a request containing two descriptions with the same name is ambiguous even if versions differ.

Do not “choose the newest” or “choose the first.” Reject the invalid request.

### 12.4 Validate both missing and extra arguments

For each Action:

- every required input must be present;
- every supplied argument must exist in the capability input description;
- every resolved value must match the declared primitive type;
- every `anchor.*` reference must resolve;
- argument ordering in emitted JSON must preserve the accepted contract where fixtures rely on it.

### 12.5 Preserve Action order and stable identities

Actions are planned in source order.

Current 0.1 identity material is position-derived. A change that filters, sorts, deduplicates, or parallelises Action planning can change Action IDs and idempotency material.

Do not reorder Actions for convenience.

### 12.6 Required Effects preserve first occurrence order

The current evaluator deduplicates required Effects while preserving first occurrence order.

Do not replace this with an unordered set whose output order varies.

If a set is used internally for membership performance, retain a separate ordered accumulator or deterministic final ordering that matches the contract.

---

## 13. Trail engineering

The Trail is product behaviour, not debug logging.

### 13.1 Ownership

Core owns deterministic reception and evaluation entries.

The host appends permission, intent, dispatch, outcome, timestamp, and result-event entries.

Core entries must remain timestamp-free.

### 13.2 Sequence is semantic

Trail entries contain a sequence number and appear in causal order.

Do not:

- prepend and forget to reverse;
- sort entries after construction;
- evaluate branches in parallel;
- write a broad helper that renumbers entries differently;
- omit earlier successful entries when returning a later correlated error.

### 13.3 Error Trail boundaries

Before reliable identities exist, decoding and parse failures return a minimal error envelope with no evaluation Trail.

After identities and an Anchor match are established, supported Condition and Action-planning errors return a correlated envelope containing:

- all known identities;
- `plan: null`;
- the Trail accumulated before the failure;
- exactly one failure entry of the correct kind.

Do not fabricate identities for a minimal error. Do not discard known identities for a correlated error.

### 13.4 Trail is not a dumping ground

Do not put these into Core Trail entries:

- stack traces;
- raw provider output;
- credentials;
- local absolute paths;
- environment state;
- complete files or conversations;
- wall-clock timestamps;
- diagnostic backtraces.

The planner Trail explains deterministic evaluation.

---

## 14. JSON and Yojson boundaries

Tethers currently uses `Yojson.Safe.t` for protocol trees.

### 14.1 Supported Tethers value subset

Yojson can represent more than Tethers 0.1 accepts.

Current Tethers values are limited to:

- strings;
- integers;
- booleans;
- `anchor.*` references in Tether source before resolution.

Do not admit floats, arrays, objects, nulls, tuples, variants, or Yojson extensions into the language merely because the library supports them.

### 14.2 Parse at the boundary

The preferred shape is:

```text
Yojson.Safe.t
-> field extraction and structural validation
-> project types
-> semantic evaluation
-> explicit response encoding
-> Yojson.Safe.t
```

Avoid repeated calls to `Yojson.Safe.Util.member` deep inside semantic code when a record or variant can express the accepted state.

### 14.3 Missing versus null

`Yojson.Safe.Util.member` returns `Null` for a missing object field. This can blur the distinction between an absent field and an explicit JSON `null`.

For fields where the contract distinguishes missing and null, inspect the association list directly with `List.assoc_opt` or use a dedicated decoder.

Do not accidentally accept a missing required field because it looked like an optional null.

### 14.4 Duplicate JSON keys

Association-list parsing may preserve duplicate object members. Later lookup often returns one occurrence and can hide ambiguity.

Where the controlling contract requires duplicate-key rejection, validate duplicates before ordinary lookup or conversion.

Do not assume Yojson has made an ambiguous object trustworthy.

### 14.5 Object key order is not semantic

Do not compare JSON object association-list order as Tethers meaning.

Fixtures should compare objects semantically unless a byte-level protocol or canonicalisation contract explicitly requires encoded order.

Array order remains significant for:

- Actions;
- Trail entries;
- Conditions represented in arrays;
- ordered Effects where specified;
- protocol sequences.

### 14.6 Encoding, Windows channel mode, and stdout

The JSON-lines executable reads one request per line and writes one compact response per line.

The MCP executable reads one JSON-RPC message per line and writes protocol responses to stdout. MCP stdio messages are newline-delimited and must not contain embedded newlines.

On native Windows, OCaml text-mode output translates `\n` to `\r\n`. Both protocol entry points must therefore configure protocol stdin and stdout before the first read or write:

```ocaml
let configure_protocol_stdio () =
  In_channel.set_binary_mode stdin true;
  Out_channel.set_binary_mode stdout true
```

Call this at startup in `main.ml` and `tethers_mcp_main.ml`. Leave stderr available for ordinary diagnostic text.

Binary-mode input has one consequence that agents must not miss: when a client sends CRLF framing, line input may retain the terminal `\r` before `\n`. The transport reader must remove at most one terminal `\r` from the framed protocol line before passing it to `Yojson.Safe.from_string`. This transport normalisation is separate from Tether-source line normalisation inside a JSON string.

Do not hash raw JSON-lines or MCP framing bytes as JCS input. Canonical digests belong to explicitly selected parsed JSON values serialised by the one canonicalisation authority. CRLF transport translation must therefore be prevented for protocol consistency, but it must not be confused with canonical JSON hashing.

Rules:

- stdout contains protocol messages only;
- diagnostics go to stderr;
- use UTF-8 JSON without a byte-order mark;
- write exactly one compact JSON value followed by one `\n`;
- flush stdout after each complete response;
- never print startup banners;
- never pretty-print multiline protocol JSON;
- do not use `Format.printf` or `print_endline` for debugging on the protocol channel;
- preserve request IDs exactly at the MCP boundary.

### 14.7 Expected failures must cross deliberately

Expected malformed-input and domain failures may reach a protocol handler only as `Tethers_error` or an explicit typed result/variant that the handler converts deliberately. Parser and evaluator helpers must not use `Not_found`, `Failure`, `Invalid_argument`, bounds exceptions, or partial Stdlib functions as expected control flow. Prefer `_opt` APIs, exhaustive matching, checked indexes, or a narrow wrapper that converts the failure at the point where project context is still available.

The outer protocol boundary must still catch unexpected exceptions and convert them to `internal_error`. That catch-all is the final containment boundary for defects, not the normal validation strategy and not proof that ordinary malformed input was handled correctly.

---

## 15. Error design

Errors are part of Tethers' observable contract.

### 15.1 Current project exception

The parser currently defines:

```ocaml
exception Tethers_error of string * string

let fail code message =
  raise (Tethers_error (code, message))
```

This is acceptable for the present small engine because errors cross a deliberate outer boundary and become structured JSON envelopes.

Within parser and evaluator code, every expected rejection must become `Tethers_error` while the helper still knows the stable project code and useful context. A bare `List.assoc`, `Option.get`, unchecked index, or other partial operation must not be allowed to turn ordinary invalid input into a context-poor `internal_error`.

Do not copy the exception everywhere without thought. New local expected alternatives may be clearer as variants or `result` values. Unexpected runtime exceptions remain defects contained only by the outermost protocol catch.

### 15.2 Choose the representation by boundary

Use a variant or `result` when:

- multiple expected outcomes are part of ordinary control flow;
- callers must handle every case;
- the failure carries typed local data;
- the operation composes with other deterministic transformations.

Use `Tethers_error` when:

- the error aborts the current parse or evaluation operation;
- one deliberate protocol boundary catches it;
- the stable machine code and message are preserved;
- converting every helper to `result` would add noise without improving state modelling.

Do not use exceptions to represent a false Condition or other successful `not_matched` outcome.

### 15.3 Stable machine codes

Do not require another component to parse human messages to identify an error.

Preserve codes such as:

- `invalid_json`
- `invalid_request`
- `incompatible_protocol`
- `incompatible_language`
- `parse_error`
- `invalid_capability`
- `missing_fact`
- `type_error`
- `unknown_capability`
- `missing_argument`
- `unknown_argument`
- `missing_reference`
- `internal_error`

A new code is a protocol decision and requires contract tests.

### 15.4 Do not swallow programmer errors silently

A broad wildcard that converts every exception into an expected domain error can hide defects.

Catch only the known exceptions at inner boundaries. Keep one explicit outer containment boundary for unexpected exceptions.

Tests should distinguish deliberate project errors from an accidental `Match_failure`, `Invalid_argument`, or other runtime exception.

### 15.5 Avoid exposing unstable internals

`Printexc.to_string` may be useful at the outer internal-error boundary, but do not let unstable implementation details become a relied-upon public contract.

Do not include secrets, local paths, or raw input dumps in an error message.

---

## 16. Pattern matching and exhaustiveness

Pattern matching is one of the main reasons OCaml fits this project. Use it to expose the full state space.

### 16.1 Avoid security-blind wildcard matches

For closed Tethers states, prefer:

```ocaml
match result with
| Matched plan -> ...
| Not_matched trail -> ...
| Evaluation_error error -> ...
```

Avoid:

```ocaml
match result with
| Matched plan -> ...
| _ -> ...
```

The wildcard can cause a newly added constructor to inherit incorrect behaviour silently.

A wildcard is appropriate when the ignored remainder is truly equivalent and future constructors should intentionally share that handling. State that intention in a comment when it is not obvious.

### 16.2 Do not confuse equality operators

Use structural equality `=` for ordinary Tethers values.

Physical equality `==` compares object identity and is almost never the intended language semantics.

Do not use polymorphic structural comparison casually on values containing functions, abstract values, or representations whose comparison semantics are not part of the contract.

### 16.3 Use compiler warnings as design feedback

Treat non-exhaustive matches, unused values, shadowing, and suspicious partial applications as evidence.

Do not silence a warning globally merely to complete a bounded task.

If a warning is intentionally accepted, explain why and suppress it at the narrowest scope supported by the project policy.

Warning numbers must be checked against the exact compiler manual rather than remembered approximately. In OCaml 5.5, warning 40 is `name-out-of-scope`; open-shadowing is covered by warnings 44 and 45. A copied numeric list with the wrong explanation is worse than no explicit list because it creates false confidence.

Dune's development profile normally treats enabled warnings as fatal. An explicit project warning stanza must preserve `:standard`, be tested against the whole current tree, and avoid accidentally narrowing Dune's existing fatal-warning policy. The exact additional warning set is a separate baseline decision, not something an agent may introduce during an unrelated implementation task.

---

## 17. Modules and interfaces

One `.ml` file creates one capitalised compilation unit:

- `tether_parser.ml` becomes `Tether_parser`;
- `tethers_protocol.ml` becomes `Tethers_protocol`;
- `tethers_evaluator.ml` becomes `Tethers_evaluator`;
- `tethers_mcp_server.ml` becomes `Tethers_mcp_server`.

### 17.1 Keep the graph acyclic

Do not introduce circular dependencies.

If two modules need each other's types, reconsider ownership:

- move the shared type into the lower-level module that owns the concept;
- create a small shared domain module;
- pass a function or value explicitly;
- split protocol encoding from semantic types.

Do not reach for recursive modules without a concrete design need and explicit review.

### 17.2 Use `open` selectively

The project currently uses `open Tether_parser` and `open Tethers_protocol` in bounded modules.

Use `open` when it keeps a small file readable and name provenance remains obvious.

Prefer qualified names when:

- modules are large;
- names may collide;
- the origin of a function or constructor matters to review;
- a trust or protocol boundary should remain visible.

Local opens can keep scope narrow:

```ocaml
let decode json =
  let open Yojson.Safe.Util in
  ...
```

Do not open a large module globally merely to save a few characters.

### 17.3 Introduce `.mli` files when they protect a real boundary

The current project has no `.mli` files.

Add one when it will:

- hide constructors that require validation;
- prevent protocol code from depending on parser internals;
- expose a stable evaluator API shared by JSON-lines and MCP adapters;
- reduce accidental coupling;
- make the module's ownership and invariant clear.

Do not add `.mli` files as empty ceremony that repeats every implementation detail.

A likely future direction is:

```text
Tether_parser.mli
  exposes parsed language types and parse_tether

Tethers_protocol.mli
  exposes validated protocol/domain types and encoders/decoders

Tethers_evaluator.mli
  exposes evaluate_request, validate_source, and response types
```

This is guidance, not authority to restructure the project during unrelated work.

### 17.4 One abstraction must pay rent

Create a module or abstraction when it:

- names a stable Tethers concept;
- enforces an invariant;
- isolates parsing, evaluation, protocol, or transport;
- removes repeated protocol logic;
- permits deterministic unit testing;
- prevents invalid construction;
- keeps a known extension local.

Do not create:

- a service layer that only forwards each call;
- a functor with one implementation and no useful invariant;
- a generic parser framework around four operators;
- a dependency-injection system for pure functions;
- “manager,” “handler,” or “processor” modules with unclear ownership;
- one file per tiny type simply because Rust code is structured that way.

---

## 18. Functional style, recursion, and collections

### 18.1 Immutability by default

Use immutable values unless mutation is the clearest honest model.

Immutable lists and records make evaluation flow easier to reason about and repeat.

### 18.2 Choose the clearest traversal

Use:

- pattern recursion when stop conditions and sequence are central;
- `List.map` for one-to-one pure transformation;
- `List.filter` for selection;
- `List.fold_left` for explicit accumulation;
- `List.iter` for visible effect at a boundary;
- pipelines when they clarify left-to-right data flow.

Do not force every operation into a fold merely to appear functional.

### 18.3 Preserve list order deliberately

Common safe construction:

```ocaml
let rec loop acc = function
| [] -> List.rev acc
| item :: rest -> loop (transform item :: acc) rest
```

Repeated append with `@` is simple but can become quadratic. More importantly, changing between prepend/reverse and append can accidentally reverse semantic order.

Every collection change must be checked against:

- Condition order;
- Action order;
- Trail order;
- Effect first-occurrence order;
- protocol output order.

### 18.4 Mutation is allowed when justified

The MCP server currently uses a `ref` for its simple lifecycle state:

```ocaml
type server_state =
  | Uninitialized
  | Initializing
  | Initialized

let server_state = ref Uninitialized
```

Controlled mutation can be the clearest model for a long-lived transport state.

Keep it:

- local to the owning module;
- represented by a closed variant, not several booleans;
- changed through explicit transitions;
- absent from pure evaluation;
- tested for invalid call order.

Do not spread mutable references through Core.

### 18.5 Arrays, Hashtbl, Map, and Set

Use them when the real problem justifies them.

Before replacing lists, define:

- required ordering;
- key comparison;
- duplicate policy;
- deterministic encoding;
- performance evidence;
- how error order is preserved.

`Hashtbl` iteration order must not leak into semantic output.

A `Map` or `Set` ordering must be intentionally compatible with the contract, not merely deterministic by accident.

---

## 19. Advanced OCaml features

OCaml 5.5 provides modules, functors, first-class modules, GADTs, locally abstract types, effects, domains, and other advanced tools.

They are available. They are not default decorations.

### 19.1 Functors

Use a functor when there are genuinely multiple implementations sharing one compile-time contract and the parameterisation enforces useful structure.

Do not use a functor merely to mock a pure function or because a Rust version uses a trait.

### 19.2 First-class modules

Use first-class modules when implementation selection must occur at runtime and modules carry the right interface.

Do not use them where a small variant of known implementations is clearer.

### 19.3 GADTs

A GADT may be justified if it makes an important invalid semantic combination unrepresentable, for example a future typed expression representation where operator and operand types are linked.

Do not introduce a GADT to encode the current four operators unless the simpler variant model has demonstrated a real safety limitation.

### 19.4 Effect handlers

OCaml effects are not a licence to hide I/O or control flow inside Tethers evaluation.

Tethers Core must remain externally pure. A future internal implementation technique using effects would require a concrete design proving that:

- no hidden external state enters evaluation;
- ordering remains exact;
- errors and Trail entries remain explicit;
- the code is easier to verify than the direct version.

The current engine does not need effect handlers.

### 19.5 Domains and parallelism

OCaml 5 domains enable parallel execution. Tethers 0.1 does not need parallel evaluation or Action planning.

Do not introduce domains, thread pools, or parallel maps into Core.

Parallelism can alter:

- first-error behaviour;
- deterministic Trail order;
- Action order;
- resource use;
- reproducibility.

A future performance task must prove that semantics remain exact.

### 19.6 Polymorphic variants and objects

Use ordinary variants for closed Tethers states.

Polymorphic variants are useful when open composition is a real requirement. OCaml objects are useful for particular structural subtyping problems. Neither is justified merely to reduce declarations.

### 19.7 PPX and code generation

Do not add PPX dependencies automatically for JSON derivation, lenses, visitors, or tests.

A PPX may be appropriate when it removes substantial repeated boundary code and its generated behaviour is understood, version-compatible, reviewable, and fixture-protected.

Dependency and build complexity must be part of the decision.

---

## 20. MCP adapter rules

The MCP server is an adapter around Tethers Core. It does not redefine Tethers semantics.

### 20.1 One evaluator authority

`tethers.evaluate` must call the same `Tethers_evaluator.evaluate_request` used by the direct engine path.

Do not build a second parser or evaluator inside the MCP server.

`tethers.validate` should use the shared parser boundary and return validation information. It must not create a separate grammar.

### 20.2 MCP state is transport state

The server lifecycle currently uses:

```text
Uninitialized -> Initializing -> Initialized
```

Calls before initialization return an MCP error. Unknown notifications may be ignored according to the accepted protocol behaviour.

Do not let MCP lifecycle state affect deterministic Tether evaluation beyond whether the adapter accepts the call.

### 20.3 Preserve JSON-RPC identity

MCP response IDs must preserve the request ID exactly, including string, integer, or null forms allowed by the controlling protocol.

Do not coerce IDs to strings or generate replacements.

### 20.4 Protocol errors versus Tethers errors

Distinguish:

- JSON-RPC request/method/argument errors;
- MCP lifecycle errors;
- a successful `tools/call` whose structured content is a Tethers `error` envelope;
- unexpected internal adapter failure.

Do not flatten every error into `isError: true` or one JSON-RPC code without checking the accepted transcript fixtures.

### 20.5 stdout and stderr

MCP stdout is protocol-only.

Unexpected line-processing diagnostics go to stderr. A malformed line must not crash the server loop unless the accepted outer process policy says otherwise.

Do not print OCaml backtraces, progress messages, or debug values to stdout.

### 20.6 Supported protocol versions

The live server currently lists explicit MCP protocol versions.

Do not add, remove, or guess support for another revision without reviewing the official specification and updating transcript tests.

A protocol version string is a contract, not marketing metadata.

---

## 21. Dependencies

The current direct OCaml dependency is Yojson.

### 21.1 Standard library first, not standard library only

Prefer the standard library when it is reliable and sufficient.

Use a maintained dependency when reimplementation would be less correct, less secure, less portable, or more expensive to verify.

Do not add Base, Core, Batteries, containers, parser generators, PPX packages, or an async runtime simply because an agent is more familiar with them.

### 21.2 Every dependency must answer

1. Which accepted requirement does it satisfy?
2. Why are Stdlib and existing dependencies insufficient?
3. Does it support OCaml 5.5.0?
4. Does it work with Dune language 3.10 and the current build shape?
5. Which exact version is tested?
6. What transitive dependencies and PPX/build steps does it add?
7. What is its licence and maintenance status?
8. How is its behaviour tested at the Tethers boundary?
9. Does it affect native Windows support?
10. Does it enlarge the trusted parsing or protocol surface?

### 21.3 Inspect exact installed packages through the supplied switch

```powershell
$OcamlSwitchPath = "<absolute authorised directory-switch root>"
opam list --switch=$OcamlSwitchPath --installed --columns=name,version
opam show --switch=$OcamlSwitchPath yojson
```

Do not update unrelated packages while implementing a bounded task. Do not omit `--switch` and allow opam to select an environment from ambient directory or global state.

### 21.4 Locking is approved but remains packet-scoped

`TOOLCHAIN-BASELINE-01` authorises a committed `tethers_engine.opam.locked` generated from the explicit verified switch. That decision does not authorise other tasks to generate, refresh, or repair the lock.

Ordinary OCaml tasks consume the accepted lock and report mismatch. Only a task packet that explicitly owns dependency locking may change it.

---

## 22. Dune engineering

### 22.1 Installed Dune versus Dune language

The installed tool is observed as Dune `3.24.0`.

The repository declares:

```lisp
(lang dune 3.10)
```

Agents may use the installed executable, but build files must remain valid under the declared Dune language unless a task explicitly raises it.

Do not copy a stanza from current online documentation without checking when each field was introduced.

### 22.2 Keep the module graph explicit

The current `bin/dune` lists modules for each executable.

When adding a module:

- add it only to the executable or library that owns it;
- avoid accidentally linking transport-only modules into the direct engine;
- preserve shared evaluator modules between both executables;
- compile immediately to expose missing or duplicate module ownership.

### 22.3 Do not create a library solely for ceremony

A shared internal library may become justified when:

- Core has stable interfaces;
- direct engine, MCP server, tests, and future embedding all need it;
- executable module lists become error-prone;
- independent unit tests need a clean target.

Until then, the explicit shared module lists are small and understandable.

### 22.4 Tests and aliases

The repository currently relies heavily on cross-language fixture and PowerShell scripts.

A future task may add Dune-native unit tests or cram tests. Do not assume `dune runtest` proves the existing external protocol suite unless those tests are actually wired into Dune.

### 22.5 Formatting

Do not add `ocamlformat` or reformat the whole engine during a small semantic task unless the project explicitly adopts a formatter policy.

Preserve local style. Keep diffs reviewable.

### 22.6 Warning policy is repository policy

Dune supports compiler flags through `(flags (:standard ...))` on executables, libraries, or an `env` stanza. Do not replace `:standard`; doing so discards Dune's selected defaults.

Before adopting explicit warning flags:

1. inspect the effective development flags with the installed Dune;
2. build the unchanged tree and record every warning;
3. verify every proposed warning number or mnemonic against OCaml 5.5;
4. prefer mnemonic names in documentation, while recognising that compiler flags still form a versioned policy;
5. enable additional warnings before deciding which are fatal;
6. apply the same policy to both OCaml executables and future tests;
7. never weaken a fixture or suppress a warning merely to make adoption green.

The approved `TOOLCHAIN-BASELINE-01` packet explicitly excludes warning policy. This guide does not silently enact it.

---

## 23. Testing strategy

The compiler passing is necessary. It is not sufficient.

Tethers tests must prove semantic contracts and negative branches.

### 23.1 Test layers

Use the strongest relevant combination of:

- parser unit tests for syntax and structural rejection;
- evaluator unit tests for deterministic state transitions;
- fixture tests for exact protocol envelopes;
- MCP transcript tests for lifecycle, IDs, tool schemas, and adapter behaviour;
- repeated-evaluation tests for determinism;
- Rust host tests when a planner response or bridge field changes;
- end-to-end PowerShell scripts for native Windows process integration;
- property or generative tests when the input space justifies them.

### 23.2 Required negative evidence

A happy path does not prove fail-closed behaviour.

Relevant negative cases include:

- incompatible protocol version;
- incompatible language version;
- malformed Tether section order;
- wrong indentation;
- duplicate Action arguments;
- duplicate capability names;
- missing Fact;
- Condition type mismatch;
- unknown capability;
- missing Action argument;
- unknown Action argument;
- missing `anchor.*` reference;
- invalid bridge-pin combinations;
- MCP call before initialization;
- unknown MCP method or tool;
- malformed MCP arguments;
- stdout contamination;
- repeated identical input producing different semantic output.

### 23.3 Determinism tests

For identical input, verify:

- same status;
- same identities;
- same Action order;
- same argument values;
- same required Effects order;
- same Trail sequence and messages;
- no timestamps in Core entries;
- semantically equivalent JSON objects.

Do not use wall-clock time or random IDs in Core tests.

### 23.4 Fixture discipline

Fixtures are contracts.

Do not update expected output merely because the implementation changed.

When a fixture changes, the worker note must state:

- which authorised semantic changed;
- why the old fixture is no longer correct;
- which specification or decision authorises the new output;
- whether Rust host or MCP consumers were checked.

### 23.5 Small refactors need equivalence evidence

For a mechanical extraction or module move:

- run the full relevant fixture suite before and after;
- inspect object and array ordering;
- check error code and message stability;
- check minimal versus correlated error shapes;
- check stdout/stderr behaviour;
- inspect the complete diff.

---

## 24. Standard verification commands

The current task packet controls the exact required command set. The following is the normal OCaml-oriented baseline.

### 24.1 Project control

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
```

### 24.2 OCaml build

The task packet must provide the exact absolute switch root:

```powershell
$OcamlSwitchPath = "<absolute authorised directory-switch root>"
$EngineSourcePath = Resolve-Path .\tethers-0.1\engine-ocaml

Push-Location $EngineSourcePath
try {
  opam exec --switch=$OcamlSwitchPath -- dune build
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
finally {
  Pop-Location
}
```

When useful and supported by the current tree:

```powershell
$OcamlSwitchPath = "<absolute authorised directory-switch root>"
$EngineSourcePath = Resolve-Path .\tethers-0.1\engine-ocaml

Push-Location $EngineSourcePath
try {
  opam exec --switch=$OcamlSwitchPath -- dune build @all
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
finally {
  Pop-Location
}
```

### 24.3 Protocol and fixture checks

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\check-fixtures.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-engine.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-mcp-transcripts.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\demo.ps1
```

Run only scripts that exist at the inspected repository revision, and run every script required by the task packet.

### 24.4 Rust compatibility when protocol changes

```powershell
pwsh -NoProfile -Command '
  Push-Location .\tethers-0.1\host-rust
  cargo test
  exit $LASTEXITCODE
'
```

An OCaml protocol change is not complete if the Rust host no longer accepts or correlates the response.

### 24.5 Diff and Git evidence

```powershell
git diff --check
git status --short
git diff --stat
git diff
```

Do not write “all tests pass” without exact commands and observed results.

If a required command was not run, state the command and the specific reason.

---

## 25. AI implementation workflow

### 25.1 Before editing

1. Confirm the exact worktree root, branch, `HEAD`, status, and expected base commit.
2. Confirm the task owner and packet state.
3. Confirm that no other agent owns the task `IN_PROGRESS`.
4. Run the task-packet checker.
5. Read the specification and only the task-relevant accepted documents.
6. Record the task-supplied absolute `OcamlSwitchPath`, verify that exact directory switch, and inspect its OCaml, opam, Dune, and Yojson versions.
7. Read the affected OCaml modules and their direct callers.
8. Run the focused existing tests before changing code.
9. State the smallest intended change and the invariant it preserves.

### 25.2 During editing

- keep the diff inside packet scope;
- compile after each coherent change;
- preserve existing error and protocol contracts;
- use typed states rather than booleans and strings where the task naturally touches them;
- keep Yojson at boundaries;
- preserve semantic order explicitly;
- add focused negative tests before broad cleanup;
- do not add a dependency, formatter, module framework, or toolchain change casually;
- never discover or silently substitute an OCaml switch; use only the absolute switch root named by the task packet;
- after two materially similar failed attempts, stop and report evidence.

**Non-target compliance clause:** do not refactor existing untouched OCaml merely to make it conform to this guide unless the current task packet explicitly requires that work. Record a concrete non-compliance under `Discoveries` in the worker note, name the affected file and risk, and leave it untouched. A discovery is not self-issued scope.

### 25.3 After editing

1. Run the exact required formatter or style check, if the project has one.
2. Run the OCaml compiler and all focused tests.
3. Run the relevant fixture and MCP transcript suite.
4. Run Rust host tests if any shared protocol changed.
5. Run whitespace checks.
6. Inspect the complete diff, not only changed hunks shown by the agent.
7. Inspect final Git status.
8. Write the worker note with exact commands and results.
9. Mark the task `COMPLETE` or `BLOCKED` honestly.
10. Stop. Do not invent or begin the next task.

### 25.4 Agent report schema

Use the repository's controlling handoff format. Unless it requires a stricter schema, use this exact template:

```markdown
# Worker Note

- **Task Packet:** `<packet name / ID / path>`
- **Owner:** `<agent or engineer>`
- **Status:** `COMPLETE | PARTIAL | BLOCKED`
- **Base Commit:** `<full hash>`
- **Final Commit:** `<full hash or NOT COMMITTED>`
- **Branch / Worktree:** `<branch and exact worktree path>`

## Files Modified

- `<path>`

## Behavioural Result

<What now happens, stated in externally observable terms.>

## Invariants Preserved

- `<determinism, ordering, protocol, error, Core/host, or Trail invariant>`

## Negative Tests Added or Updated

- `<test function or fixture path>` — `<failure branch proved>`

## Commands Executed

- `<exact command>` — `PASS` (`<count or relevant result>`)
- `<exact command>` — `FAIL` (`<relevant result>`)

## Unrun Checks and Reason

- `None`

or:

- `<exact command>` — `NOT RUN`: `<specific reason>`

## Discoveries

- `<new repository fact or non-target non-compliance; not an invented decision>`

## Remaining Risks

- `None`

or:

- `<specific unresolved risk and affected boundary>`

## Recommended Next Action

<One smallest concrete action.>
```

Rules for this schema:

- do not omit a section by silently assuming `None`;
- use full commit hashes;
- name focused tests and fixtures rather than saying only “tests added”;
- give exact commands and observed counts where available;
- `COMPLETE` is forbidden when a required command was not run unless the packet explicitly marks it optional;
- `Final Commit: NOT COMMITTED` is valid and preferable to an invented hash;
- discoveries do not authorise additional implementation;
- distinguish a real unresolved risk from a feature deliberately deferred outside the packet.

Do not write “all tests pass” without the exact command and result.

---

## 26. Common AI failure patterns

### Failure: Moving host logic into Core

**Symptom:** OCaml begins checking permission, provider availability, replay state, credentials, or retries.

**Correction:** Core plans. Rust authorises and executes.

### Failure: Reimplementing semantics in the MCP adapter

**Symptom:** `tethers_mcp_server.ml` contains its own Condition evaluator or response logic.

**Correction:** Call the one shared parser and evaluator.

### Failure: Treating Yojson as the domain model

**Symptom:** semantic functions repeatedly call `member` and switch on arbitrary JSON.

**Correction:** decode into variants and records at the boundary touched by the task.

### Failure: Accepting unsupported JSON values

**Symptom:** floats or nulls reach Tethers values because Yojson supports them.

**Correction:** enforce the 0.1 value subset explicitly.

### Failure: Hidden coercion

**Symptom:** `"1"` compares equal to `1`, or `1` becomes `true`.

**Correction:** preserve exact types and return `type_error`.

### Failure: Wildcard hides a new state

**Symptom:** a newly added variant silently follows `_ -> internal_error` or `_ -> false`.

**Correction:** match closed states explicitly.

### Failure: Physical equality

**Symptom:** code uses `==` for Tethers values or strings.

**Correction:** use structural equality `=` unless object identity is deliberately required.

### Failure: Order lost through a set or hash table

**Symptom:** Actions, Effects, or Trail entries change order across runs.

**Correction:** preserve declared order separately from membership lookup.

### Failure: Exception changes become protocol changes

**Symptom:** a helper rewrite changes the public error message or envelope.

**Correction:** protect observable errors with fixtures and preserve them mechanically.

### Failure: Minimal and correlated errors collapse

**Symptom:** every error contains fake identities, or every error drops known identities and Trail.

**Correction:** preserve the exact boundary at which reliable context exists.

### Failure: Dune feature drift

**Symptom:** build files use a field introduced after Dune language 3.10 because Dune 3.24 is installed.

**Correction:** obey `(lang dune 3.10)` until an explicit build-language task changes it.

### Failure: Compiler drift

**Symptom:** code uses an OCaml 5.6 API while the project baseline is 5.5.0.

**Correction:** check the exact 5.5 manual or local installed docs.

### Failure: Importing another ecosystem mentally

**Symptom:** an agent invents `Result.map_error`, Rust iterator methods, Haskell functions, F# computation expressions, or Base/Core modules.

**Correction:** verify the OCaml 5.5 Stdlib or installed package API.

### Failure: Advanced feature without rent

**Symptom:** functors, GADTs, effects, PPX, or domains appear in a four-case parser task.

**Correction:** use the least complicated feature that accurately protects the present design.

### Failure: Avoiding an advanced feature for the wrong reason

**Symptom:** a useful `.mli`, variant, module type, or GADT is rejected only because Matthew may not read it directly.

**Correction:** write senior-quality implementation and explain it outside the code.

### Failure: Protocol debug output on stdout

**Symptom:** a print statement corrupts JSON-lines or MCP framing.

**Correction:** stdout is protocol-only. Use stderr for bounded diagnostics.

### Failure: Fixture laundering

**Symptom:** expected output is changed until tests pass without an authorised semantic decision.

**Correction:** fixtures judge the code, not the other way around.

### Failure: Whole-file rewrite

**Symptom:** a small change produces hundreds of unrelated formatting and naming edits.

**Correction:** preserve local structure and make the semantic diff inspectable.

### Failure: Unrun tests reported as green

**Symptom:** the handoff says “all tests pass” without exact commands.

**Correction:** list every command and observed result. State unrun checks plainly.

---

## 27. Review checklist

### Architecture

- Does OCaml still implement only deterministic Core and protocol adapters?
- Has any permission, provider, replay, execution, or application-specific logic crossed into Core?
- Is there still one parser and one evaluator authority?
- Does MCP remain an adapter rather than a second implementation?

### Semantics

- Does the code match `tethers-0.1/SPEC.md` exactly?
- Are Anchor, Condition, and Action order preserved?
- Does evaluation stop at the correct point?
- Is false Condition still `not_matched`, not error?
- Are Action results still handled through later events rather than hidden chaining?

### Types

- Are closed choices represented by variants?
- Are records used where tuples have become unclear?
- Are trusted internal values distinct from untrusted JSON where the task touches the boundary?
- Does any wildcard match hide a future constructor?
- Are invalid states constructible without validation?

### Determinism

- Does Core read any external changing state?
- Can hash-table, set, map, or parallel iteration alter output order?
- Are Core Trail entries timestamp-free?
- Do repeated identical inputs produce the same semantic output?

### Parser

- Is indentation preserved exactly?
- Are duplicate arguments rejected?
- Are unsupported values rejected?
- Has string syntax changed accidentally?
- Are parse error codes and messages stable?

### Protocol

- Are minimal and correlated errors still distinct?
- Are all known identities preserved when required?
- Are MCP request IDs preserved exactly?
- Is stdout protocol-only?
- Are JSON object keys treated as unordered and arrays as ordered?

### Toolchain

- Was OCaml 5.5.0 used or explicitly checked?
- Was the local project switch selected?
- Does Dune syntax remain within language 3.10?
- Was the exact Yojson version inspected before using unfamiliar APIs?
- Was any dependency or machine state changed without authority?

### Testing

- Did the OCaml build pass?
- Did focused parser/evaluator tests pass?
- Did fixture checks pass?
- Did MCP transcript tests pass when relevant?
- Did Rust host tests pass after protocol changes?
- Does every required negative branch have direct evidence?
- Was the complete diff inspected?
- Is the final Git state known?

---

## 28. Definition of done

An OCaml task is complete only when:

1. The requested behaviour exists.
2. The 0.1 language and protocol semantics remain correct.
3. The Core and host boundary remains intact.
4. Deterministic ordering remains exact.
5. Invalid or unsupported input is rejected at the correct boundary.
6. Error codes, envelope shapes, and Trail behaviour are preserved or deliberately revised by authority.
7. The code compiles with the OCaml 5.5.0 baseline.
8. Dune files remain valid under language 3.10 unless explicitly changed.
9. Required negative paths have focused tests or fixtures.
10. Relevant JSON-lines, MCP, and Rust-host compatibility checks pass.
11. No unrelated dependency, toolchain, formatting, or architecture change is hidden in the diff.
12. The complete diff and Git status have been inspected.
13. The worker note contains exact reproducible evidence.
14. Every unrun check or unresolved risk is stated plainly.

The compiler passing is necessary evidence. It is not sufficient evidence.

---

## 29. Recorded decisions and deferred work

The following distinguishes approved work from implementation already present in the repository.

### 29.1 TOOLCHAIN-BASELINE-01 is approved, not yet implemented

`TOOLCHAIN-BASELINE-01` is the controlling approved decision for a deliberate split between:

- the package's supported compiler range;
- the exact compiler used for verified development;
- the exact locked dependency resolution;
- the older Dune language understood by project files.

It authorises a committed `tethers_engine.opam.locked`, but only its separately issued Amber implementation task may generate or change that lock. Until then, the checked-in package constraint remains the current repository state. Do not partially enact the approved baseline during another task.

### 29.2 The opam lock is approved but implementation-scoped

The future `tethers_engine.opam.locked` must preserve the exact accepted dependency resolution while leaving the ordinary `.opam` compatibility ranges distinct. Its generation, inspection, and publication belong only to `TOOLCHAIN-BASELINE-01`.

### 29.3 Define a compiler warning policy separately

The repository should eventually decide which warnings are enabled and which are fatal in development or CI.

Do not adopt a broad `-warn-error` setting without first compiling the current tree and deciding how generated, platform-specific, and version-dependent warnings are handled.

### 29.4 Add `.mli` files when Core boundaries stabilise

The highest-value future interfaces are likely the parser, protocol, and evaluator boundaries.

Do not add them all mechanically. Add them in a task that deliberately hides invalid construction and proves both executables still use the same Core.

### 29.5 Add direct OCaml tests without replacing protocol fixtures

Dune-native unit tests would improve local feedback for parser and evaluator invariants.

They should supplement, not replace, the JSON fixture and MCP transcript tests that prove cross-language behaviour.

### 29.6 Keep this guide and the Rust guide paired

The two guides should describe the same boundary from opposite sides:

- OCaml owns deterministic language semantics and planning;
- Rust owns trusted admission, permission, execution, durability, and uncertainty.

When architecture changes, update both guides in one documentation task or link them to one canonical boundary section to prevent drift.

---

## 30. Project references

Read the task-relevant parts of these before OCaml implementation:

- `AGENTS.md`
- `docs/PROJECT_CONTROL.md`
- `docs/AGENT_WORKFLOW.md`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/CONSTITUTION.md`
- `tethers-0.1/SPEC.md`
- `docs/architecture/TETHERS_LANTERN_KEEPER_CANONICAL_ARCHITECTURE.md`
- `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md`
- `docs/DECISIONS.md`
- `docs/CAPABILITY_BRIDGE.md` when bridge fields are involved
- `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md`
- the most recent worker note for the affected area

Current OCaml implementation locations:

```text
tethers-0.1/engine-ocaml/
  dune-project
  tethers_engine.opam
  bin/
    dune
    main.ml
    tether_parser.ml
    tethers_protocol.ml
    tethers_evaluator.ml
    tethers_mcp_main.ml
    tethers_mcp_server.ml
```

---

## 31. Official technical references

Use the exact versioned or installed documentation when behaviour is unfamiliar.

- OCaml 5.5 manual: https://ocaml.org/manual/5.5/
- OCaml 5.5 standard library API: https://ocaml.org/manual/5.5/api/
- OCaml modules: https://ocaml.org/manual/5.5/modules.html
- OCaml patterns: https://ocaml.org/manual/5.5/patterns.html
- OCaml type declarations: https://ocaml.org/manual/5.5/typedecl.html
- OCaml parallel programming and domains: https://ocaml.org/manual/5.5/parallelism.html
- Dune reference: https://dune.readthedocs.io/en/stable/reference/
- Dune tests: https://dune.readthedocs.io/en/stable/tests.html
- opam manual: https://opam.ocaml.org/doc/Manual.html
- opam switches: https://opam.ocaml.org/doc/man/opam-switch.html
- opam lock: https://opam.ocaml.org/doc/man/opam-lock.html
- Yojson Safe API: https://ocaml-community.github.io/yojson/yojson/Yojson/Safe/index.html
- Yojson Safe.Util API: https://ocaml-community.github.io/yojson/yojson/Yojson/Safe/Util/index.html

When documentation for “latest” differs from OCaml 5.5.0, Dune language 3.10, or the installed Yojson version, the project baseline wins until an authorised upgrade task changes it.

---

## Final rule

Tethers Core should feel small because its concepts are small, not because the implementation avoids the language's strengths.

Use OCaml to make the deterministic semantics explicit, typed, exhaustive, and difficult to distort.

Keep everything effectful, permissioned, provider-specific, or application-specific outside Core.
