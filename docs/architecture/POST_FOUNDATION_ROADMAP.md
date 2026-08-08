# Tethers Post-Foundation Roadmap

Status: **DIRECTION / WATCHPOINTS — NOT AUTHORISED IMPLEMENTATION**

Prepared after the Foundation Pass design discussions, August 2026.

This document records where Tethers is intended to go after the Foundation Pass reaches F10. It does **not** amend `TETHERS_FOUNDATION_PASS.md`, does not authorise any production work, and does not freeze syntax or implementation details that have not yet been designed against current repository reality.

Its purpose is simpler: future work should not have to reconstruct the project’s direction from old conversations.

The governing architectural instinct is:

> **Preserve optionality at the boundaries. Freeze meaning, not implementation freedom.**

In ordinary language: **do not build ourselves into a corner.**

Companion rules:

> **Commit where commitment gives us something. Preserve freedom where commitment buys us nothing.**
>
> **Do not generalise early. Do not specialise unnecessarily.**
>
> **Leave the future somewhere to stand.**

The post-Foundation programme currently has four major threads:

1. deterministic concurrency and scalable execution;
2. human-facing inference and normalisation;
3. Tethers using Tethers;
4. HQ, the human/AI-facing GUI and workbench.

Several secondary engineering watchpoints sit behind those threads, including large-Tether evaluator scaling, large result payloads, AST caching, and broader Plug extensibility. They remain watchpoints until separately measured or designed.

---

## 1. Deterministic concurrency

### 1.1 Core distinction

Concurrency belongs in the **Tethers semantics**. Parallelism mostly belongs in the **runtime**.

Tethers must be able to express that several operations are independent and may overlap. The host decides how much physical parallelism is actually available on the current machine, runtime, provider, or future execution environment.

The same Tether should keep the same semantic meaning on a small laptop, a 32-core workstation, or a future distributed host. Hardware may change speed. It must not change meaning.

The intended model is:

```text
TETHER
"What is semantically independent or dependent?"

PLUG / CAPABILITY
"What overlap is safe, and at what scope?"

HOST
"Given those rules and the resources available, what will actually run now?"
```

This three-authority split is fundamental.

### 1.2 Tether authority

A Tether describes dependencies and independence.

The important semantic statement is **not**:

```text
actions have an inherent serial order
```

It is:

```text
dependencies constrain execution order
```

That distinction preserves future headroom.

A future Tether may express fan-out and join concepts such as:

```text
              ┌→ weather ─┐
              ├→ calendar ┤
start ────────┼→ email ───┼→ join → continue
              └→ traffic ─┘
```

The syntax is not frozen here. The semantic requirement is that independent work can be represented as independent work and dependencies remain explicit.

### 1.3 Plug authority

The Tether cannot know every physical constraint of every external system. The Plug or capability provider knows its own operational reality.

A Plug must therefore be able to constrain concurrency. Examples of the **kind** of constraint we may need are:

```text
unrestricted / concurrently safe
exclusive
bounded to N concurrent operations
keyed by a resource identity
shared provider limit
rate-limited
```

These are examples, not frozen syntax.

A filesystem capability might allow writes to different files concurrently but serialize writes to the same target. A database Plug might require exclusive migration access. An API provider might allow four concurrent requests. A GPU capability might claim exclusive device ownership.

A Tether may make an action *eligible* for concurrency. It must never be able to override a Plug’s safety constraint.

The host may always execute **less** concurrency than the Plug permits. It must never execute **more**.

### 1.4 Host authority

The Rust host owns execution machinery:

- async I/O;
- task queues;
- worker/resource admission;
- provider limits;
- backpressure;
- process supervision;
- physical scheduling;
- cancellation/cleanup;
- future parallel and possibly distributed execution.

The OCaml semantic side should remain the judge of meaning rather than becoming an increasingly stateful physical scheduler merely to obtain performance.

This continues the intended split:

```text
OCaml = deterministic semantic judge
Rust  = muscular execution machinery
```

### 1.5 Deterministic interpretation

Physical completion order is allowed to vary.

For example:

```text
run 1: weather 12 ms, calendar 80 ms, email 400 ms
run 2: email 90 ms, calendar 140 ms, weather 300 ms
```

That does not require semantic nondeterminism.

The governing rule is:

> **Nondeterministic execution, deterministic interpretation.**

Future concurrency design must ensure:

- semantic outcomes do not depend on which branch wins a race;
- branch identity/order is canonical rather than completion-order dependent;
- join semantics are explicit;
- failure behaviour at joins is explicit;
- sibling completion/failure is not silently lost;
- replay meaning does not depend on timing accidents;
- the Trail may record physical timing/completion observations without making those observations semantic inputs unless explicitly designed otherwise.

### 1.6 Concurrency decision gate

After F10, concurrency should receive an architecture/repository pass before HQ implementation goes too far.

If the current code shows concurrency can be introduced as a bounded change, likely on the scale of a day or two of implementation packages, it should probably be implemented before the main HQ build.

If it instead requires broad protocol migration, replay redesign, language surgery, result identity changes, or a large persistence redesign, do **not** force it through simply because it is desirable. Freeze enough of the semantic shape that HQ and other work do not obstruct it, then defer the expensive machinery.

The goal is not to build a motorway immediately. The goal is to avoid building the town hall across the only sensible route for one.

---

## 2. Human-facing inference and normalisation

Tethers is intended to be unusually easy for ordinary humans **and** AIs to write correctly.

That requires reducing ceremony without introducing guessing.

The governing rule is:

> **Infer what is obvious, what has been pre-decided, but no more.**

A useful companion formulation is:

> **Inference may remove ceremony. It may not manufacture intent.**

### 2.1 Friendly outside, strict inside

Human-facing input may be forgiving. The deterministic semantic core should not be fuzzy.

The intended shape is:

```text
human-facing expression
        ↓
obvious conventional meaning?
        ↓
previously established context/policy?
        ↓
normalise
        ↓
strict canonical typed meaning
        ↓
deterministic evaluation
```

If meaning is not obvious and has not been established in advance:

```text
ASK / REJECT / REQUIRE CLARIFICATION
```

Never silently choose a merely plausible interpretation.

### 2.2 Sources of valid inference

Inference may come from three places:

1. **explicit meaning in the expression**;
2. **obvious conventional notation**;
3. **previously established context or policy**.

Examples:

```text
5 min          → duration
£20            → GBP money value
75%            → percentage
3 retries      → integer count with retry meaning
3:30 pm        → clock time
```

Previously established context might include:

```text
default currency = GBP
timezone = Europe/London
week starts Monday
```

Once such context is deliberately established, later shorthand may rely on it.

By contrast:

```text
run at 6
```

is ambiguous unless an earlier rule has already resolved what `6` means in that context.

### 2.3 Everyday semantic families

The normalisation architecture should eventually have a coherent place for ordinary human concepts such as:

- durations;
- clock times;
- dates and relative dates;
- currencies and money;
- counts;
- percentages;
- data sizes;
- distances and other units;
- bounded quantities;
- common comparative forms such as `at least`, `before`, and `after` where the grammar supports them.

This is **not** an instruction to add every conceivable unit immediately. The important architectural requirement is that these concepts are handled by one deliberate normalisation approach instead of scattered special-case guesses throughout the evaluator, HQ, Plugs, and host.

### 2.4 AI-first authorship

AI authorship makes this design more important, not less.

An AI may generate very large Tethers containing hundreds or thousands of values, conditions, actions, joins, and capability calls. Common human notation should remain easy to generate, but an AI must not rely on private intuition that another model or runtime could interpret differently.

The same normalisation contract should serve humans and AIs.

### 2.5 HQ relationship

HQ should use the same interpretation rules rather than inventing a GUI-only language.

For clear input, HQ can quietly show the understood meaning:

```text
Run after: 5 minutes
           ✓ duration
```

For ambiguous input, HQ should surface the ambiguity:

```text
Run at: 6
        ? 6:00 am or 6:00 pm?
```

That makes ambiguity pleasant to resolve without moving uncertainty into the deterministic core.

---

## 3. Tethers using Tethers

The long-term plan is **not** to rewrite the trusted implementation in Tethers.

The plan is to let suitable policy, coordination, and workflow increasingly be expressed as Tethers/Tether Sets while the small trusted machinery remains in Rust and OCaml.

### 3.1 Trusted machinery remains machinery

Likely permanent trusted machinery includes:

- parsing and semantic evaluation;
- capability permission enforcement;
- process supervision;
- persistence primitives;
- Trail/replay machinery;
- cryptographic/trust boundaries;
- capability execution;
- resource admission;
- host safety boundaries.

Those are mechanisms, not policy scripts.

### 3.2 Suitable self-application

Good self-application candidates are higher-level coordination jobs such as:

- selecting relevant verification suites from declared repository changes;
- explaining why a particular verification plan is required;
- Plug admission workflow coordination;
- development/release preparation;
- release-candidate checks;
- maintenance workflows;
- later, bounded parts of Tethers’ own operational policy.

The exact list must be justified against the language that actually exists at the time.

### 3.3 Start in observer/adviser mode

The first self-hosting slice should be deliberately weak in authority.

Example:

```text
repository changed
      ↓
Tether receives declared facts
      ↓
selects relevant verification plan
      ↓
explains the plan
      ↓
host/human performs authorised work
      ↓
Trail records why
```

This dogfoods the language without letting it rewrite its own trust boundary.

### 3.4 Self-hosting safety rules

Self-application must preserve:

- no self-granted capabilities;
- no hidden shell escape hatch;
- no ability for a Tether to redefine its own permission boundary;
- explicit approval where required;
- complete Trail evidence;
- external kill/off switch;
- host-enforced limits independent of the Tether being executed.

Self-hosting should make Tethers one of Tethers’ most demanding customers, not turn the project into an ouroboros with root access.

---

## 4. HQ: GUI, editor, explorer and human understanding layer

HQ is a major product phase, not simply a visual skin over the CLI.

The central rule is:

> **The GUI must never become the real language.**

A Tether remains a Tether. HQ is an editor, explainer, visualiser, runner, and inspector over that same underlying semantic object.

### 4.1 Source remains real

HQ should preserve a clean relationship with source:

- source can be written directly;
- AI can author source directly;
- HQ can open and understand it;
- visual edits round-trip back to valid Tethers source where supported;
- saving through HQ must not create an opaque second representation that only HQ understands.

This matters particularly because AI authors may generate Tethers far larger than a human would ever construct by clicking boxes.

HQ must therefore solve a more interesting problem than visual programming:

> **Make machine-generated Tethers intelligible to humans.**

### 4.2 HQ must anticipate concurrency

Concurrency architecture should be understood before HQ’s underlying editor model is frozen.

HQ may eventually need to represent:

- dependencies;
- independent branches;
- fan-out;
- joins;
- branch failures;
- capability concurrency constraints;
- semantic order versus physical completion order.

Even if concurrency implementation is deferred, HQ must not assume that every Tether is fundamentally a single serial list.

### 4.3 HQ must anticipate inference

HQ should accept and explain natural values through the same normalisation contract as the language.

It should not force humans to fill in machine-centric forms such as:

```text
TYPE: Duration
VALUE: 300000
UNIT: Milliseconds
```

when `5 minutes` is already unambiguous.

### 4.4 First useful vertical slice

The preferred first HQ slice is vertical rather than a collection of disconnected screens:

```text
open a Tether
      ↓
show source
      ↓
understand / validate
      ↓
show structure
      ↓
run with event/facts
      ↓
show matched / not-matched / error
      ↓
show resulting plan
      ↓
show Trail / explanation
```

That already creates a useful Tethers workstation.

Later layers can include:

- visual structure editing;
- capability/Plug browser;
- condition editor;
- large-Tether navigation;
- run history;
- Trail explorer;
- concurrency visualisation;
- self-hosting workflow inspection;
- AI-assisted authoring and explanation.

### 4.5 Design workflow

HQ should use Gem/Gen heavily for UI exploration because this is exactly the kind of problem where many competing visual/interaction ideas are useful.

The intended collaboration is:

```text
Matthew → product intent, feel, usefulness, what is confusing
Gem/Gen → broad UI exploration and prototypes
Lucy → semantic/architectural adjudication and contract freeze
implementation agents → bounded accepted slices
```

UI exploration may proceed in parallel with backend architecture work because it can remain non-authoritative and non-production until shared contracts are understood.

---

## 5. Preserved optionality and extensibility

This principle governs all four post-Foundation threads.

Things that should become rigid are the parts that give Tethers its identity:

- what a Tether means;
- what determinism promises;
- what permissions mean;
- what replay promises;
- what a capability is allowed to claim;
- what constitutes an explicit dependency;
- what ambiguity is allowed to become semantic meaning.

Things that should retain implementation freedom include:

- how work is scheduled;
- how much work runs concurrently;
- how a host maps concurrency to cores/processes/services;
- how capabilities are physically provided;
- how results are represented internally;
- how parsing/evaluation are cached or accelerated;
- how large payloads are stored or referenced;
- whether a future host is local or distributed.

The design question before freezing a boundary should be:

```text
What assumption are we making?

Is it fundamental to Tethers,
or merely convenient for today’s implementation?

If the world changes in five years,
can this part be replaced without changing what a Tether means?
```

The goal is not infinite flexibility. Keeping every option open creates abstraction soup. The goal is to avoid unnecessary irreversible choices.

---

## 6. Secondary engineering watchpoints

These are not separate authorised phases. They are reminders to preserve headroom and measure before acting.

### 6.1 Large-Tether evaluator scaling

AI-first authorship means very large Tethers are valid intended usage.

Future measurement should revisit:

- repeated OCaml list tail appends;
- effect accumulation/deduplication;
- linear capability lookup per action;
- large condition/action/capability counts;
- accidental quadratic behaviour.

Do not optimise these merely because they look suspicious. Measure representative large Tethers first.

### 6.2 AST/source caching

The current evaluator reparses source for each evaluation. A cache keyed by source identity/hash may be possible without changing semantics if hit/miss behaviour produces identical semantic results.

Measure before implementing.

### 6.3 Large result payload policy

Large JSON/result payloads may eventually create avoidable copying and GC pressure across Rust/JSON/OCaml boundaries.

Future options may include selective projection, references/blobs, and explicit payload policies. Do not redesign the wire format without evidence.

### 6.4 Broader Plug extensibility

The preferred principle is:

> **Open the Plug system outward, not inward.**
>
> **Freeze the core. Expand the edges.**

Plugs may grow richer declarations around capabilities, requirements, events, lifecycle, returns, compatibility, metadata, and resource/concurrency constraints without inventing new core language semantics for every external system.

---

## 7. Post-Foundation execution strategy

### 7.1 Finish Foundation first

F7, F8, F9, and F10 remain the current programme. Nothing in this roadmap authorises work before F10 closes Foundation unless Matthew and Lucy explicitly choose to interrupt the programme.

### 7.2 Scout the major threads in parallel

After F10, run bounded architecture/repository scouting in parallel where useful:

```text
A. deterministic concurrency architecture + implementation estimate
B. inference/normalisation architecture recovery
C. Tethers-using-Tethers seam audit + first observer-mode candidate
D. HQ interaction/design exploration
```

These scouting jobs should initially be evidence/design work, not four implementation agents changing adjacent core modules at once.

### 7.3 Reconcile before implementation

Lucy should reconcile the scouting results into one shared architectural picture because the threads intersect:

- concurrency changes what HQ must represent;
- inference changes what HQ accepts and explains;
- self-hosting exercises capabilities, Trail, inference, and policy boundaries;
- HQ exposes all of them to humans;
- AI authorship places scale pressure on all of them.

### 7.4 Likely implementation order

Unless repository evidence changes the decision, the current preference is:

1. **Concurrency**, if the real implementation is bounded.
2. **Basic inference/normalisation machinery**, establishing the rule and boundary before attempting a huge catalogue of human concepts.
3. **First observer-mode Tethers-using-Tethers slice**, to dogfood the language and expose awkward seams.
4. **First HQ vertical slice**, then make HQ a major development stream.

HQ design exploration can begin earlier in parallel, but production HQ contracts should not freeze assumptions that concurrency or inference are about to invalidate.

### 7.5 If concurrency is not bounded

If the architecture pass shows concurrency is a large redesign:

1. define/freeze the minimum semantic shape needed to preserve future concurrency;
2. ensure HQ and Plug contracts leave that shape room;
3. defer heavy runtime implementation;
4. proceed with inference, self-hosting and HQ where they are independent.

### 7.6 Development work may itself be concurrent

Use the same rule for building Tethers that Tethers will eventually use at runtime:

> **Parallelise independent work. Order dependent work.**

Multiple scouting/design jobs may run together.

Implementation jobs may run together only when ownership and assumptions are genuinely independent. Two agents simultaneously changing adjacent OCaml semantic boundaries because both tasks look small is not useful concurrency; it is delayed merge conflict.

---

## 8. What success looks like

The four major threads serve one coherent ambition:

```text
Concurrency
→ Tethers can become LARGE.

Inference
→ Tethers can remain HUMAN.

Tethers using Tethers
→ Tethers becomes EXPRESSIVE enough to govern suitable parts of itself.

HQ
→ all of that remains UNDERSTANDABLE.
```

The implementation behind the language should not become the accidental limiting factor.

A useful summary remains:

> **Tethers should be a small language backed by a big machine.**

Humans should be able to read it.

AIs should be able to write it, potentially at enormous scale.

The runtime should be able to become much more capable without requiring the meaning of existing Tethers to change.

And as the project grows:

> **Let the system become bigger without making the core less understandable.**

---

## 9. Authority and next-step rule

This roadmap is deliberately directional. It is not a substitute for current evidence.

Before each post-Foundation implementation phase:

1. inspect current repository reality;
2. revalidate old assumptions and watchpoints;
3. identify the smallest demonstrated problem or desired semantic addition;
4. freeze the relevant contract;
5. prepare a bounded implementation packet;
6. independently review implementation evidence before acceptance.

No future worker should treat this document as permission to start concurrency, inference, self-hosting, HQ, Plug expansion, caching, payload redesign, or performance work merely because the idea appears here.

The roadmap tells us **where we are trying to go**. Each future packet still has to earn the right to move one step.