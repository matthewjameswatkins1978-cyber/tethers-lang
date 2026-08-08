# Post-Foundation Semantic Concurrency

Status: **DIRECTION / ARCHITECTURAL GUIDANCE — NOT AUTHORISED IMPLEMENTATION**

Prepared after the post-Foundation semantic-concurrency design discussion, August 2026.

This document sharpens the semantic half of the concurrency work recorded in `POST_FOUNDATION_CONCURRENCY.md` and `POST_FOUNDATION_ROADMAP.md`. It does **not** amend the Foundation Pass, authorise implementation, freeze syntax, or require any particular internal representation. Future work must still re-audit repository reality and produce a bounded implementation packet.

The governing rules remain:

> **The Tether declares semantic independence and dependency. The Plug declares safe overlap constraints. The host decides the actual physical schedule.**
>
> **Nondeterministic execution, deterministic interpretation.**

The central semantic rule is:

> **Wall-clock completion order must never choose semantic meaning.**

A second rule is equally important:

> **Keep the source language structured and readable without unnecessarily freezing the internal semantic model into permanent whole-block barriers.**

---

## 1. No first-to-finish semantics

Physical completion races are allowed. Semantic races are not.

If branches A, B, and C execute concurrently, the host may observe any completion order:

```text
run 1: B, A, C
run 2: C, B, A
```

That observation must not by itself select a branch value, downstream path, primary failure, or Trail order.

Do not introduce semantics equivalent to `Promise.race()` where the fastest physical completion becomes the winner.

If Tethers ever supports a concept such as `any` or `first_successful`, selection must use an explicit deterministic priority, most naturally declaration order unless a better frozen rule is later justified.

Example conceptually:

```text
A declared before B
A and B both eventually succeed
B physically finishes first
semantic winner remains A
```

The runtime may therefore need to wait for higher-priority branches to reach a terminal state before a lower-priority success can become semantically relevant.

For the first concurrency design, the safer choice is probably to omit `any` entirely unless a demonstrated use case requires it.

---

## 2. Structured source, flexible semantic headroom

A structured source form is strongly preferred over exposing arbitrary orchestration-DAG syntax to ordinary Tethers authors.

A future source shape may resemble:

```text
concurrent
    weather.get
    calendar.today
    mail.unread
join
```

This gives humans and AIs an obvious lexical region and a clear explanation surface in HQ.

However, **do not freeze an absolute whole-block barrier as a permanent law merely to obtain determinism.**

A whole-block barrier is an excellent first implementation rule because it is easy to explain, test, replay, and visualize:

```text
all branches reach terminal outcomes
        ↓
join is interpreted
        ↓
next instruction proceeds
```

But determinism does not inherently require every future dependent action to wait for unrelated siblings.

If a later semantic model can state explicit dependencies such as:

```text
D depends on A
D does not depend on B or C
```

then D may be physically eligible to begin after A reaches its required semantic state even while B and C remain in progress, provided:

- no wall-clock race chooses meaning;
- D's inputs are completely determined;
- Plug safety constraints permit the overlap;
- join/failure rules remain deterministic;
- observable side effects are governed by explicit dependency semantics rather than completion timing.

This means the **surface language may remain structured while the compiler/runtime internally lowers it to a dependency representation**.

The project should avoid two opposite mistakes:

1. exposing an Airflow-style arbitrary DAG language merely because the runtime can represent one;
2. permanently serializing independent downstream work because the first syntax used a simple block barrier.

Use structured concurrency first. Preserve room for dependency-aware scheduling later.

---

## 3. Branch identity and canonical ordering

Every branch in a concurrent semantic region requires a stable canonical identity for that exact Tether version.

Semantic ordering must derive from source/declaration structure, never from completion time.

A branch identity may eventually come from:

- an explicit alias;
- a canonical compiler-derived branch ID;
- another deterministic source-derived identity.

Exact syntax is not frozen.

Positional names such as `action_1` may be acceptable as an internal or compatibility fallback within one source version, but they should not be assumed to be the ideal long-term human reference because inserting an earlier branch renumbers later positions.

The essential requirement is:

```text
same Tether version
→ same branch identities
→ same canonical branch ordering
```

---

## 4. Branch outputs: no magic merge

Concurrent branch outputs must remain separately identifiable.

Do not deep-merge arbitrary JSON objects into one shared result scope.

Bad conceptual model:

```text
weather returns { status, temp }
mail returns    { status, unread }
              ↓
magically merge into one object
```

This creates collision and provenance problems.

Preferred conceptual model:

```text
branch weather  → its own outcome/output
branch calendar → its own outcome/output
branch mail     → its own outcome/output
```

A later syntax may offer names such as:

```text
results.weather
results.calendar
results.mail
```

but exact source syntax is not frozen.

The semantic result should be representable as a **canonical ordered vector/list of branch outcomes**, each carrying its branch identity, rather than relying on physical insertion order into a map.

---

## 5. Minimal first join policy

Avoid a large join-policy DSL in the first design.

Do not begin with concepts such as:

```text
require 2 of 3
threshold 75%
weighted quorum
first 3 successes
retry another sibling
```

The smallest promising first policy is:

1. all ordinary branches are required by default;
2. a branch may potentially be explicitly marked optional;
3. the group/join disposition is derived deterministically from the canonical branch outcomes.

Conceptually:

```text
concurrent
    weather.get
    optional traffic.route
join
```

An optional branch remains fully observable. Optional means only that its failure/uncertainty does not by itself make the group incomplete.

It does **not** mean:

- ignore its error;
- hide its Trail record;
- pretend an uncertain external side effect did not occur;
- permit downstream code to assume a successful payload exists.

This `required + optional` model is promising, not frozen syntax.

---

## 6. Canonical branch outcomes

The semantic join boundary likely needs explicit branch outcome states.

A useful candidate set is:

```text
Succeeded
Failed
Uncertain
Cancelled
```

Meanings:

### Succeeded
The capability completed successfully and produced its declared output.

### Failed
A known failure was returned, with structured error information.

### Uncertain
The host/capability cannot determine whether the external effect completed or what the final state is. Timeouts, connection loss, or ambiguous provider acknowledgement may lead here.

### Cancelled
Execution was deliberately stopped before a normal terminal capability result was obtained.

Exact ADT names and wire shapes are not frozen here.

The important rule is that a join interprets a complete **canonical branch outcome set**, not the order in which callbacks happened to arrive.

---

## 7. Primary failure must be canonical

If a group contains several failed non-optional branches, the concept of a `primary_failure` must never mean "the one that failed first in wall-clock time".

If a primary failure is needed at all, choose it by a deterministic rule such as earliest canonical declaration order among relevant failed branches.

Example:

```text
source order: A, B, C
physical failures arrive: C, B
B and C both failed
primary failure: B
```

The full outcome vector must still preserve C's failure as well.

Do not reduce a multi-branch failure to one lossy error merely for convenience.

---

## 8. Important correction: cancellation is not merely physical

A scouting proposal described sibling cancellation after one branch failure as a free physical optimization.

That is too strong.

If sibling actions can create external side effects, discretionary cancellation can change the observable world:

```text
run 1: sibling finishes sending message before cancellation arrives
run 2: sibling is cancelled before sending message
```

Even if the join later formats both runs canonically, the external effects differ.

Therefore:

> **The host must not treat arbitrary sibling cancellation as semantically invisible.**

A safe staged policy is:

### First implementation
Prefer allowing admitted siblings to reach terminal outcomes rather than introducing automatic fail-fast cancellation.

### Later cancellation, if justified
Cancellation may be allowed only where the relevant capability/operation declares enough semantics to make cancellation safe and meaningful, and where the Tether/group policy explicitly permits that behaviour.

A capability may eventually need to distinguish ideas such as:

```text
cancel-safe before commit
cancellation unsupported
cancellation may leave outcome uncertain
```

Exact contract vocabulary is not frozen.

The semantic record must reflect actual cancellation/uncertainty rather than pretending the cancelled branch never existed.

---

## 9. Timeouts and the clock

The rule is not literally "the clock can never affect execution". External operations often require timeouts.

The stricter and more useful rule is:

> **Wall-clock timing must not choose between otherwise valid semantic branch values merely because one arrived first.**

A configured timeout may legitimately cause a branch to become `Uncertain` or `Failed` according to the capability/host contract.

Because that can affect semantic outcome, timeout policy must not be an undocumented ambient accident. Future design should make clear:

- who owns the timeout policy;
- whether it is capability, Tether, or host configuration;
- what outcome a timeout produces;
- what evidence is recorded for replay/audit.

Do not smuggle host-default timeout changes into semantics without observability.

---

## 10. Join disposition

A join result should not be only a boolean.

Conceptually it may contain:

```text
group disposition
canonical ordered branch outcomes
branch identities
outputs for successful branches
structured errors for failed branches
uncertainty/cancellation information where applicable
```

For a first `required + optional` policy:

```text
Complete
= every required branch succeeded

Incomplete
= at least one required branch did not succeed
```

Optional branches may still be `Failed`, `Uncertain`, or `Cancelled` inside a `Complete` group.

Exact names such as `Complete` / `Incomplete` are not frozen.

---

## 11. Downstream access to branch results

Downstream code must not receive a magically flattened scope.

A downstream reference should identify the branch and then its output.

For optional or unsuccessful branches, the language must make absence/failure explicit enough that a downstream reference cannot silently pretend a successful value exists.

Future design needs to decide whether this is represented through:

- explicit outcome matching;
- guarded references;
- a result/option-like type;
- another small deterministic construct.

Do not add that machinery until a concrete first concurrency syntax is designed.

---

## 12. Semantic concurrency versus physical concurrency

The intended boundary is:

```text
SEMANTIC CONCURRENCY — OCaml / language meaning

- branch identity
- declaration/dependency structure
- required vs optional meaning
- deterministic join reduction
- canonical result order
- canonical failure selection
- result/output relationships

            ↓ contract boundary

PHYSICAL CONCURRENCY — Rust / host machinery

- actual async execution
- ready/admission queues
- Plug safety/resource claims
- coarse/dynamic capacity
- physical start/finish timing
- worker/provider availability
- cancellation signalling when semantically permitted
- telemetry
```

Physical scheduling may vary freely inside those semantic constraints.

---

## 13. Trail and operational telemetry

Do not derive Trail order from completion order.

A semantic concurrency record may eventually include:

- group/region identity;
- canonical branch order;
- branch identities;
- required/optional status;
- final canonical branch outcomes;
- canonical group disposition;
- canonical primary failure if such a field exists;
- branch outputs/errors where appropriate.

Operational telemetry may separately include observations such as:

```text
ready_at
admitted_at
started_at
completed_at
waited_for_resource
physical worker/provider
```

These observations can explain performance and real execution without becoming the canonical semantic ordering.

> **Timing may explain the run. Timing must not secretly define the run.**

---

## 14. Replay

Replay must consume the canonical semantic observations/outcomes rather than re-running wall-clock races to rediscover meaning.

The replay contract should eventually make it possible to reconstruct:

```text
which branches existed
which outcomes each branch had
which were required/optional
how the deterministic join reduced them
which downstream semantic path followed
```

Do not make replay depend on reproducing physical latency, thread scheduling, provider timing, or resource admission order.

---

## 15. Structured concurrency as the likely first implementation

The first implementation should strongly prefer the smallest comprehensible semantic shape:

```text
explicit concurrent region
→ fixed canonical branches
→ all branches produce terminal outcomes
→ deterministic join
→ continue
```

This is intentionally conservative.

It provides:

- simple source syntax;
- obvious HQ visualization;
- easy tests;
- easy Trail explanation;
- clean implementation bounds;
- no arbitrary user-authored DAG language.

But the implementation should avoid baking in assumptions that make dependency-aware execution impossible later.

A useful internal question is:

```text
Are we implementing a first structured concurrency surface,
or permanently asserting that all future semantic dependencies are barriers?
```

Prefer the former.

---

## 16. Things deliberately not to build yet

Do not build unless later evidence demands them:

- `Promise.race` / fastest-wins semantics;
- arbitrary graph syntax;
- quorums and weighted joins;
- threshold success DSLs;
- dynamic branch creation;
- completion-order result IDs;
- automatic deep object merging;
- mandatory fail-fast sibling cancellation;
- user-visible mutex/semaphore syntax;
- clock-driven semantic winner selection;
- distributed semantic scheduling.

---

## 17. Post-Foundation semantic-concurrency design gate

Before implementation, inspect repository reality and answer:

1. what action/source ordering currently means;
2. how actions/results are identified today;
3. whether current result references can name branch-specific outputs;
4. what minimal syntax can express one concurrent region;
5. whether the first implementation should use an absolute barrier;
6. how internal representation can preserve future dependency-aware scheduling;
7. whether `optional` is sufficient for the first failure policy;
8. exact branch outcome states and their relationship to existing error/result types;
9. deterministic primary-failure selection;
10. how uncertainty is represented today;
11. whether sibling cancellation should be omitted initially;
12. how timeouts are configured and evidenced;
13. how canonical branch ordering reaches Trail/replay;
14. how optional branch outputs are referenced safely;
15. behaviour under very large AI-authored concurrent regions;
16. whether concurrency can remain a bounded post-Foundation implementation before HQ.

---

## 18. Summary

The preferred first semantic model is deliberately small:

```text
structured concurrent region
+
canonical branch identities/order
+
required branches with optional exceptions
+
explicit branch outcome vector
+
deterministic join reduction
```

The host may execute branches in any physically sensible order allowed by Plug safety and available capacity.

The semantic core must never choose meaning because one packet, thread, process, or provider happened to finish first.

Most importantly:

> **Structured concurrency is the preferred human surface. Explicit dependencies are the underlying semantic truth. Do not confuse a simple first barrier with a permanent requirement to serialize future independent work.**

And:

> **Cancellation that can change external effects is semantic policy, not merely a scheduler optimization.**
