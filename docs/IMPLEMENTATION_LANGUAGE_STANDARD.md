# Tethers Implementation Language Standard

Status: authoritative for implementation technique

Audience: senior engineers and AI coding agents

## Purpose

Tethers implementation code must use each programming language as a serious
engineering tool. Code is not to be simplified into a beginner-readable subset
for Matthew. Matthew owns the product and architectural judgement; technical
explanations, task packets, worker notes, and reviews carry the burden of making
implementation decisions understandable to him.

The code itself is written for correctness, maintainability, senior review, and
reliable continuation by capable humans and AI.

This standard governs OCaml, Rust, PowerShell, protocol fixtures, and future
implementation languages. It does not govern the user-facing Tethers language;
that is controlled by `docs/CONSTITUTION.md` and `tethers-0.1/SPEC.md`.

Where generic style advice elsewhere conflicts with this document, this document
controls implementation technique. Product semantics, trust boundaries,
accepted decisions, specifications, and task-packet constraints remain higher
authorities.

## Governing Rule

Use the language fully, but selectively.

Choose the language feature, abstraction, or library that expresses the domain
most accurately and safely. Do not avoid an appropriate feature merely because
it is advanced, unfamiliar to Matthew, or harder to explain in chat. Equally,
do not use advanced machinery merely to display fluency.

The target is not elementary code and not clever code. The target is direct,
idiomatic, robust code whose structure matches the problem.

## Decision Order

When several implementations are possible, prefer them in this order:

1. Preserve specified behaviour, trust boundaries, and compatibility.
2. Make invalid states and unsafe transitions difficult or impossible.
3. Represent the domain accurately in types, modules, and interfaces.
4. Use the host language idiomatically.
5. Keep effects, ownership, failure, and ordering explicit.
6. Minimise conceptual surface area and future maintenance cost.
7. Optimise performance when evidence shows it matters.
8. Prefer brevity only when it improves, rather than compresses, understanding.

Local consistency matters, but it does not justify perpetuating a weak pattern.
A better pattern may be introduced through a bounded, tested change.

## Simple Architecture, Capable Implementation

Tethers architecture should remain small and legible. That does not require
primitive implementation code.

Use strong language features to keep the architecture simple:

- algebraic data types or enums for closed states;
- records, structs, and newtypes for meaningful data boundaries;
- exhaustive pattern matching for complete behaviour;
- modules and interfaces for ownership and dependency boundaries;
- traits, signatures, or explicit contracts for replaceable behaviour;
- iterators, folds, combinators, and pipelines when they make data flow clearer;
- immutable values where mutation is unnecessary;
- controlled mutation where it is the clearest or most efficient honest model;
- proof-carrying or typestate-style values where later operations require an
  established invariant.

Do not flatten these ideas into strings, booleans, dictionaries, or loosely
related parameters merely because those forms look simpler.

## Abstraction

An abstraction must pay rent.

Add one when it does at least one of the following:

- names a stable domain concept;
- enforces an invariant;
- isolates an effect or trust boundary;
- removes repeated policy or protocol logic;
- permits independent testing;
- prevents callers from constructing an invalid state;
- makes a likely extension local rather than cross-cutting.

Do not add abstraction solely to reduce line count, imitate a framework, or
prepare for an unspecified future. Do not reject a useful abstraction merely
because the current implementation is small.

Prefer one strong abstraction over several thin forwarding layers. Avoid both
copy-and-paste sprawl and abstraction confetti.

## Types and State

Model states as states, not comments.

- Use variants or enums instead of boolean combinations for mutually exclusive
  outcomes.
- Use distinct types or newtypes when two values share a representation but not
  a meaning.
- Keep constructors private when validity requires checks.
- Carry verified evidence in values when later code must not bypass a proof.
- Keep protocol-facing types distinct from trusted internal types when crossing
  the boundary changes what may safely be assumed.
- Avoid optional fields for states that require different mandatory data; prefer
  a sum type or equivalent representation.

A type is useful when it removes questions from later code.

## Functions and Modules

A function should own one coherent responsibility. It need not be artificially
small.

Split code when a boundary becomes clearer, independently testable, reusable,
or governed by a different invariant. Do not split linear logic into a trail of
one-line wrappers that forces readers to reconstruct the operation across files.

Modules should reflect domain ownership and dependency direction. Keep the graph
acyclic. Public interfaces should expose the smallest capability callers need,
not the internal representation that happened to be convenient.

Use interface files or explicit exported surfaces when a module boundary has
become stable and valuable. Do not create interfaces as empty ceremony.

## Effects and Determinism

Keep pure decisions separate from effects.

- Parse and validate untrusted input before granting it trusted internal form.
- Pass changing state explicitly into deterministic logic.
- Keep clock, randomness, environment, filesystem, network, process, and
  credential access at visible boundaries.
- Do not hide effectful work behind innocent-looking getters or conversions.
- Preserve semantic ordering where Actions, Trail entries, or protocol messages
  depend on it.
- Never trade away a fail-closed boundary to make an implementation convenient.

## Errors

Errors are part of the design.

- Represent expected domain outcomes explicitly.
- Use structured error types with stable machine-readable identity.
- Preserve trustworthy context and causal information.
- Catch errors at deliberate boundaries, not everywhere.
- Do not collapse unavailable, denied, invalid, failed, cancelled, timed out,
  and uncertain outcomes into one generic error.
- Panic, unchecked exception escape, or process termination is acceptable only
  for a genuinely unrecoverable invariant breach or a clearly defined outer
  boundary.
- Error messages should help a senior engineer or AI locate the violated
  contract without exposing secrets.

Existing observable error contracts remain stable unless an authorised task
changes them deliberately.

## Dependencies

Prefer the standard library when it is reliable and sufficient. This is not a
ban on dependencies.

Use a well-maintained dependency when reimplementing the concern would be less
correct, less secure, less portable, or more expensive to verify, particularly
for cryptography, canonicalisation, parsing standards, schema processing, and
protocol support.

Every new dependency must have a concrete purpose, compatible licence, supported
version range, acceptable maintenance posture, and tests at the project boundary.
Do not add a framework where a library or ordinary code is enough.

## Comments and Documentation

Code should not be written as a tutorial, and comments should not translate each
line into English.

Comment:

- why a boundary exists;
- which invariant is being protected;
- why an apparently simpler alternative is unsafe;
- which external contract or standard controls behaviour;
- why ordering, ownership, or lifetime is significant;
- what must remain true during future modification.

Do not comment obvious syntax. Use names and types to explain what the code is.
Use comments to explain why it must be that way.

Task packets and worker notes should explain unfamiliar but appropriate language
features rather than removing them from the implementation.

## Testing and Verification

Test contracts and failure boundaries, not private line-by-line structure.

Use the strongest appropriate combination of:

- focused unit tests for local invariants;
- table-driven or parameterised tests for state matrices;
- golden fixtures for stable protocol behaviour;
- property or generative tests where broad input spaces matter;
- integration tests for process, language, and trust boundaries;
- deterministic-repeat tests where reproducibility is a requirement;
- compile-time restrictions where the type system can prevent misuse.

Every required negative branch needs direct evidence. A broad happy-path test is
not proof of fail-closed behaviour.

Formatting, compiler warnings, static checks, tests, complete diff inspection,
and final repository state are part of implementation, not optional polish.

## OCaml Direction

Use OCaml as OCaml.

- Prefer algebraic data types, records, modules, explicit signatures, exhaustive
  pattern matching, immutable data, and clear recursive or combinator-based
  transformations.
- Use `Result`, `Option`, exceptions, or a dedicated error type according to the
  actual boundary. Expected local alternatives usually belong in data; parse or
  protocol exceptions may remain appropriate when caught at one deliberate
  boundary.
- Use pipelines, higher-order functions, folds, maps, and local operators when
  they make data flow clearer. Avoid dense point-free or combinator-heavy code
  that hides domain transitions.
- Introduce `.mli` interfaces when they protect a meaningful module boundary.
- Functors, first-class modules, GADTs, effects, and other advanced features are
  available when a concrete problem justifies them. Their novelty is neither a
  reason to use them nor a reason to forbid them.
- Consult the exact OCaml 5.5 documentation for unfamiliar or version-sensitive
  behaviour. Do not invent APIs from another ML ecosystem.

## Rust Direction

Use Rust's type and ownership systems to enforce the host's trust boundaries.

- Prefer enums, newtypes, private fields, validated constructors, traits, and
  ownership transfer when they make illegal transitions unrepresentable.
- Use typestate or proof-token patterns for security-sensitive sequencing, such
  as the existing dispatch-ready boundary.
- Use iterators and combinators when they are clearer than manual indexing;
  prefer an explicit loop when it better exposes control flow or failure.
- Avoid broad `Clone`, interior mutability, dynamic dispatch, and shared mutable
  state unless their cost and purpose are concrete.
- Avoid `unwrap` and `expect` in production paths unless impossibility is
  established by a nearby invariant and the failure message names that
  invariant.
- `unsafe` requires an explicit Red design gate, a documented safety argument,
  focused tests, and independent review.

## PowerShell Direction

PowerShell is orchestration, not the home of product semantics.

- Use PowerShell 7 and strict failure handling.
- Make paths, quoting, process exit codes, cleanup, and encoding explicit.
- Use functions for coherent orchestration steps and fail immediately on an
  untrustworthy result.
- Keep business rules in OCaml or Rust unless the rule genuinely belongs to the
  development or verification workflow.
- Scripts must be deterministic where practical, safe with spaces in paths, and
  honest about commands that were not run.

## JSON and Protocol Direction

Treat JSON as an interchange format, not an untyped internal architecture.

- Parse once at the boundary and convert into typed internal values.
- Reject malformed, ambiguous, duplicate, or unsupported input according to the
  controlling contract.
- Never rely on object-key order unless a canonicalisation standard explicitly
  defines encoded bytes.
- Make compatibility changes deliberately and protect them with fixtures.
- Keep trusted host-owned fields distinct from provider-reported or user-supplied
  data.

## AI Coding Rules

AI agents must:

- inspect the live language version, dependencies, local patterns, and task
  contract before proposing syntax or APIs;
- use official language or library documentation when behaviour is unfamiliar;
- write idiomatic production code, not pseudo-code translated mechanically into
  the target language;
- retain appropriate advanced features and explain them in the worker note;
- avoid whole-file rewrites and unrelated formatting for bounded changes;
- state every test, check, or command that was not run;
- distrust their own report until the compiler, tests, fixtures, diff, and Git
  state support it;
- stop rather than invent a missing semantic, permission, or trust decision.

## Review Test

A reviewer should ask:

1. Does the code preserve the specified behaviour and trust boundaries?
2. Does its structure match the domain rather than the input file format?
3. Has the language's type system removed preventable invalid states?
4. Are effects, errors, ordering, and ownership visible?
5. Is each abstraction justified by a present invariant or boundary?
6. Is any complexity accidental, decorative, or speculative?
7. Has any code been made primitive merely to look easier?
8. Do tests prove both success and required failure paths?
9. Could a senior engineer or capable AI continue this code without recovering
   hidden assumptions from chat?

## Final Standard

Elegance is not the fewest lines or the most advanced technique.

Elegance is the smallest coherent set of concepts that expresses the real
system, enforces its boundaries, and remains straightforward to change.
