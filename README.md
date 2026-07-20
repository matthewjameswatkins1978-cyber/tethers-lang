# Tethers Lang

Tethers is governed by two complementary documents:

- `docs/CONSTITUTION.md` records the enduring design principles.
- `tethers-0.1/SPEC.md` defines the current precise 0.1 language and protocol
  semantics.
- `docs/MCP_PLAN.md` records the approved post-0.1 direction for an OCaml
  Tethers MCP interface.
- `docs/OCAML_GUIDE_FOR_AGENTS.md` gives version-specific OCaml guidance for AI
  coding agents.

The active prototype tree is `tethers-0.1/`.

## MCP Direction

Tethers owns its MCP interface directly in OCaml. Lantern Keeper is one
connected host and capability provider, not the MCP hub. The first MCP surface
is planner-only over stdio: evaluate a complete Tethers request and return the
existing Plan and Trail envelope without executing Actions.
