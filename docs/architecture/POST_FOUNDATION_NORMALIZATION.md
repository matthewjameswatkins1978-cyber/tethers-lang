# Post-Foundation Type-Directed Normalization

Status: **DIRECTION / ARCHITECTURAL GUIDANCE — NOT AUTHORISED IMPLEMENTATION**

Prepared after the post-Foundation inference design discussion, August 2026.

This document sharpens the human-facing inference thread recorded in `POST_FOUNDATION_ROADMAP.md`. It does **not** amend the Foundation Pass, authorise implementation, freeze syntax, or require any particular OCaml module structure. Future work must still re-audit repository reality and produce a bounded implementation packet.

The governing user-facing rule is:

> **Infer what is obvious, what has been pre-decided, but no more.**

Companion rule:

> **Inference may remove ceremony. It may not manufacture intent.**

The central architectural finding is:

> **Tethers does not need general-purpose dimension extraction in its trusted runtime. In most useful cases, Tethers already knows the dimension it expects. The right mechanism is type-directed normalization, not heuristic NLP in the semantic path.**

---

## 1. The corrected pipeline

The syntax parser must remain pure and independent of installed Plugs and runtime capability availability.

The preferred conceptual pipeline is:

```text
TETHERS SOURCE
      |
      v
PURE SYNTAX PARSER
"What was written?"
      |
      v
AST / UNRESOLVED LITERAL
      |
      + expected type from resolved capability/schema
      + explicit language/context inputs where permitted
      |
      v
TYPE-DIRECTED NORMALIZATION
"Does this written value have exactly one accepted meaning of this kind?"
      |
      +-----------------------------+
      |                             |
      v                             v
CANONICAL TYPED VALUE          AMBIGUOUS / UNSUPPORTED
      |                             |
      v                             v
SEMANTIC EVALUATOR             HARD DIAGNOSTIC / ASK BOUNDARY
```

The key separation is:

```text
SYNTAX
What was written?

TYPE / CAPABILITY CONTRACT
What kind of value belongs here?

NORMALIZATION
Does the written value have exactly one accepted meaning of that kind?

SEMANTICS
What does the resulting Tether do?
```

Do not make the parser consult Plug schemas merely to construct the AST. The same source should remain syntactically parseable even when the Plug needed for later validation is unavailable.

---

## 2. Why normalization, not type-directed parsing

A tempting design is to let the parser use the target capability schema while parsing a literal such as `10m`.

That creates the wrong dependency direction. A `.tethers` file could become syntactically unparseable simply because a particular Plug was not installed on the current machine.

Instead, parsing should preserve enough lexical/literal information for a later validation/normalization pass.

Conceptually, the parser might preserve something equivalent to:

```text
Literal "10m"
```

or a structured but unresolved literal candidate such as:

```text
UnitCandidate(number = 10, suffix = "m")
```

The exact AST representation is **not frozen here**.

Later, after capability/schema resolution:

```text
expected type = Duration
literal       = 10m
```

may normalize uniquely to:

```text
Duration(10 minutes)
```

while:

```text
expected type = Distance
literal       = 10m
```

may normalize uniquely to:

```text
Distance(10 metres)
```

The language does not ask, "What can `10m` mean in the universe?"

It asks:

> **Given the meaning already established by the program, does this literal have exactly one accepted interpretation?**

This is a much smaller and more deterministic problem.

---

## 3. Three levels of human-facing interpretation

Not every friendly form belongs in the same mechanism.

### 3.1 Formal literals

These belong directly to the accepted Tethers language surface when designed and versioned:

```text
5 min
90 sec
£20
GBP 20
75%
15:30
3:30 pm
5 km
20 MB
2026-08-08
```

They should be deterministic and offline.

### 3.2 Type-directed normalization

Some written forms may be ambiguous in isolation but uniquely resolvable once the expected type is known.

Example:

```text
10m + expected Duration
→ 10 minutes

10m + expected Distance
→ 10 metres
```

Another example:

```text
20 pounds + expected Money
→ GBP 20

20 pounds + expected Mass
→ 20 lb
```

These examples are architectural illustrations, not frozen syntax.

### 3.3 AI / HQ authoring interpretation

Fuzzy prose, slang, idiom, or conversational intent should normally be translated **before** it becomes authoritative Tethers source.

Examples:

```text
twenty quid
fifty bucks
half five
in a bit
a couple of minutes
sometime tomorrow afternoon
```

AI or HQ may help the human turn those expressions into strict Tethers source, but the trusted semantic path should not silently guess their meaning.

This gives AI/HQ a clear job:

```text
UNSUPPORTED OR AMBIGUOUS HUMAN INTENT
              |
              v
AI / HQ AUTHORING ASSISTANCE
              |
              v
EXPLICIT / TYPE-RESOLVED TETHERS SOURCE
              |
              v
NORMAL TETHERS PIPELINE
```

---

## 4. Interpretation taxonomy

Use the following mental model when designing and testing normalization.

### EXPLICIT

The literal itself uniquely determines the accepted meaning.

Examples:

```text
2026-08-08
15:30
5 km
75%
```

Engine action: canonical resolution.

### TYPE-RESOLVED

The expected type uniquely resolves a form that would otherwise have more than one possible dimension.

Example:

```text
10m + Duration
→ 10 minutes
```

Engine action: normalize during the typed validation/normalization pass.

### CONTEXT-RESOLVED

An **explicit, pre-established semantic context** uniquely resolves the remaining meaning.

Possible examples:

```text
£20 + explicitly established currency interpretation
relative date + explicit temporal context
locale-sensitive form + explicitly selected language/locale policy
```

Engine action: normalize only when the required context is an explicit input covered by the language/runtime contract.

### AMBIGUOUS

More than one accepted interpretation remains after applying the expected type and all permitted explicit context.

Examples may include:

```text
at 6
03/04/2026
```

Engine action: **hard reject with a clear diagnostic**. The deterministic core does not pick the most likely candidate.

### UNSUPPORTED

No accepted Tethers normalization rule exists for the form.

Examples may include:

```text
twenty quid
half five
in a bit
```

Engine action: syntax/validation diagnostic or authoring assistance boundary, depending on where the unsupported form is encountered.

No confidence score enters semantic execution.

---

## 5. No probabilistic semantic authority

General NLP/dimension systems such as Duckling are useful research sources, corpus sources, and edge-case mines.

They should **not** automatically become the authority that decides what committed Tethers source means at runtime.

The issue is broader than whether a particular library is rule-based, classifier-assisted, deterministic in one version, or probabilistic in some internal stage. The semantic problem is ownership:

> **Tethers language meaning must be controlled by the Tethers language/version contract, not by an independently evolving external interpretation engine.**

Therefore the preferred role for Duckling-like systems is:

```text
reference implementation
edge-case catalogue
corpus inspiration
grammar/rule research
negative-test inspiration
```

not:

```text
runtime semantic authority
```

Existing technology should be evaluated before implementation, but reuse must preserve Tethers' ownership of meaning.

---

## 6. Temporal context must be snapshotted

Relative time is a special semantic family because its meaning depends on an explicit reference instant and timezone.

If Tethers eventually supports forms such as:

```text
tomorrow
next Tuesday
```

then ambient machine time must never leak into semantics.

The host should provide an explicit temporal context. The exact ownership/API is not frozen here, but one rule is strong:

> **`now` is an immutable snapshot for one evaluation. It does not tick during that evaluation.**

Example trap:

```text
evaluation begins: 23:59:59.999
capability work crosses midnight
evaluation continues: 00:00:01
```

All semantic uses of `now` within that evaluation must resolve from the original snapshot, not from repeated wall-clock reads.

A future representation may preserve relative meaning in the AST/semantic value:

```text
RelativeDate Tomorrow
```

and resolve it later against explicit temporal context.

Do not prematurely assume relative source must be flattened into an absolute date at parse time.

The distinction between **relative to evaluation** and **relative to authorship/save time** must be explicit if both are ever supported.

---

## 7. Schema evolution must not silently change magnitude

Type-directed normalization introduces a subtle compatibility hazard when capability schemas evolve.

Example:

```text
Plug v1:
timeout : Int
source: timeout = 10

Plug v2:
timeout : Duration
same source: timeout = 10
```

A schema change must not silently reinterpret `10` as ten seconds, ten milliseconds, or another dimensional value unless the Tethers language contract explicitly defines such a default.

Preferred rule:

> **A naked scalar supplied to a dimensional type is rejected unless the language specification explicitly defines a canonical default unit for that dimension.**

Default units, if they ever exist, are language semantics and therefore require deliberate versioned design. They must not be invented by a Plug update.

This protects old source from silent magnitude changes.

---

## 8. Compound literals must be formal or rejected

Humans naturally write compound forms:

```text
5 mins 30 secs
1 GB 500 MB
£10.50
```

Do not let a fuzzy regex silently glue arbitrary compounds together.

For an initial implementation, either:

1. define a formal compound grammar with exact semantics; or
2. reject compound forms outside the deliberately supported literal grammar.

A possible explicit future form might be:

```text
5 min + 30 sec
```

but syntax is not frozen here.

The rule is:

> **Compound meaning must be formal language, not accidental parser generosity.**

---

## 9. Locale and cultural conventions

Locale-sensitive meaning must never come from ambient OS settings.

Potentially hazardous forms include:

```text
03/04/2026
$20
half five
billion
```

A future Tethers design may choose to support an explicit file, Tether Set, evaluation, user-policy, or other locale/context hierarchy. That hierarchy is **not frozen here**.

What is frozen as direction is the boundary:

> **Cultural or locale-dependent interpretation requires explicit semantic context or explicit syntax. Never infer it from the machine on which the Tether happens to run.**

Early versions should prefer culturally stable forms where practical, for example ISO-style dates, explicit currency codes, and explicit am/pm or 24-hour times.

Tethers is not obliged to support every convenient human notation.

---

## 10. Replay and versioning

Normalization rules are part of language meaning.

If a normalization rule changes semantic interpretation, that change must be governed by Tethers language/version compatibility rather than silently arriving through an external library update.

Replay/audit design should preserve enough evidence to answer:

```text
What source expression was written?
What expected type was applied?
What explicit context was used?
What canonical value was produced?
Which language/normalization rules governed the interpretation?
```

A future Trail may therefore benefit from preserving both the original human-facing literal and the canonical interpreted value where that interpretation matters to auditability.

Exact Trail shape is not frozen here.

Adding support for a previously unsupported spelling may be backward-compatible. Changing the meaning of an already valid expression is much more serious and may require a language-version boundary.

---

## 11. Small initial dimension set

Do not turn Tethers into an NLP project.

A deliberately small first normalization set should be sufficient to prove the architecture.

Candidate families for a future bounded design pass:

```text
Duration
Money
Percentage
Absolute clock time
```

Possible next families after evidence/design:

```text
Distance / measurement units
Data size
Absolute dates
Relative dates/times
Counts / bounded quantities
```

This list is directional, not authorised scope.

Relative dates/times deserve special treatment because they introduce temporal-context and replay questions.

---

## 12. Representative design corpus

The eventual design/implementation pass should test both accepted and deliberately rejected forms.

### Likely straightforward candidates

```text
5 min
five minutes
90 seconds
£20
75%
seventy-five percent
3:30 pm
15:30
5 km
five kilometres
20 MB
2026-08-08
```

Whether word-number forms such as `five minutes` live in the parser proper or a deterministic normalizer remains an implementation/design question.

### Type-resolved candidates

```text
10m + expected Duration
10m + expected Distance
20 pounds + expected Money
20 pounds + expected Mass
```

These are examples for semantic testing, not frozen source syntax.

### Context-resolved candidates

```text
$20 + explicit currency context
relative date + explicit timezone and immutable evaluation-now snapshot
```

### Ambiguous/reject candidates

```text
6
at 6
03/04/2026
20M
```

unless a future formal rule and required explicit context make one interpretation unique.

### Unsupported authoring-assistance candidates

```text
twenty quid
fifty bucks
half five
Friday week
a couple of minutes
a gig
in a bit
```

The point of the corpus is not maximal linguistic coverage. It is to prove that the boundary between convenience and guessing remains crisp.

---

## 13. HQ and AI authoring contract

HQ and AI authors may be substantially more conversational than the core language.

That freedom must terminate before authoritative source semantics begin.

HQ should be able to explain normalization, for example:

```text
Run after: 5 minutes
           ✓ Duration
```

or expose ambiguity:

```text
Run at: 6
        ? Please choose 06:00 or 18:00
```

AI authoring may translate:

```text
twenty quid
```

into an explicit canonical Tethers form after obtaining or using already-established context.

But HQ/AI convenience must not create hidden semantics that plain-source Tethers cannot represent.

The GUI remains an editor/explainer over real Tethers, not a second language.

---

## 14. Ownership guidance

Current architectural preference:

```text
OCaml / semantic side
- owns Tethers language meaning
- owns deterministic normalization semantics
- owns ambiguity rejection semantics

Rust host
- supplies explicit runtime/evaluation facts where the contract permits them
- does not silently decide what source literals mean
- continues to own execution machinery

Plug/capability contract
- supplies expected types and declared constraints
- does not redefine Tethers literal semantics at runtime

HQ / AI authoring
- may interpret fuzzy human intent
- must emit or confirm strict Tethers meaning before authoritative execution
```

Do not freeze exact module placement until the post-Foundation repository audit.

The important boundary is ownership of meaning, not which source file contains a helper function.

---

## 15. Post-Foundation design gate

When Foundation is complete, the normalization architecture pass should answer from current repository reality:

1. what literal information the parser should preserve;
2. where schema/type resolution currently occurs and where normalization cleanly fits;
3. the smallest first dimension set;
4. which accepted spellings belong in formal grammar versus deterministic normalization;
5. exact ambiguity diagnostics;
6. explicit context ownership and precedence;
7. immutable temporal snapshot semantics;
8. replay/Trail evidence requirements;
9. normalization language-version compatibility;
10. performance behaviour for very large AI-authored Tethers;
11. which Duckling/other corpora and rules are useful as research/test inputs without becoming runtime semantic dependencies.

No implementation is authorised by this document.

The future packet should remain bounded and prove one best normalization path before adding a broad catalogue of human expressions.

---

## 16. Summary

The desired result is not "Tethers understands English."

It is:

> **Tethers understands a deliberately bounded set of human-friendly literals exactly.**

The parser stays pure.

Plug schemas provide expected types without becoming parser dependencies.

Type-directed normalization removes ceremony where the program already supplies enough meaning.

Explicit context may resolve only what has been deliberately pre-decided.

Ambiguity is rejected, not ranked.

AI/HQ handles genuinely fuzzy language before it becomes authoritative Tethers source.

External NLP systems remain useful research material without owning language semantics.

This preserves the original principle:

> **Infer what is obvious, what has been pre-decided, but no more.**

And it sharpens the broader Tethers rule:

> **Freeze meaning, not implementation freedom.**