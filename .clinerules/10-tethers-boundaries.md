# Tethers Architectural Boundaries

Tethers is a small deterministic behaviour language and Capability protocol.

Authority:

- `docs/CONSTITUTION.md` governs enduring design principles.
- `tethers-0.1/SPEC.md` defines current precise language/protocol semantics.

Core model:

- Hosts provide events, immutable Facts, and Capability schemas.
- Tethers parses, validates, evaluates, and produces an Action Plan.
- Hosts authorise and execute Actions.
- The Trail explains evaluation, authorisation, and execution.

Non-negotiable rules:

1. Tethers plans; hosts execute.
2. Core must remain application-agnostic.
3. Lantern Keeper is a host, not a special language mode.
4. Evaluation uses only explicitly supplied immutable input.
5. AI is an explicit Capability, never hidden authority.
6. Capability schemas describe Effects but do not grant permission.
7. Hosts enforce permissions.
8. Tethers must not claim an Action executed when it only planned it.
9. Identical complete input must produce identical semantic engine output.
10. Version 0.1 remains smaller than a general programming language.

Version 0.1 excludes:

- loops;
- arithmetic;
- user functions;
- parallel Actions;
- branching inside `do`;
- live Fact queries;
- implicit I/O;
- Conditions based on Action results;
- direct Action-result chaining;
- application-specific grammar;
- AI, email, GitHub, scheduling, HQ, and adapter features.

Do not alter grammar, protocol meaning, error semantics, permission boundaries, identity rules, or Trail ownership without an explicit approved task.

If implementation and SPEC.md conflict, stop and report the conflict. Do not silently choose one.
