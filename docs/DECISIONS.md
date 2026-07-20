# Decisions

## 2026-07-20: Preserve The Prototype Archive

Decision: Keep `Tethers-0.1-Prototype.tar.gz` in the workspace.

Reason: The tarball is the original imported artifact and provides a recovery
point for the extracted prototype.

## 2026-07-20: Extract Without Flattening

Decision: Extract the archive as `tethers-0.1/` instead of moving its contents
into the workspace root.

Reason: The archive already contains a clean top-level directory. Preserving it
avoids accidental collisions and keeps the prototype boundary clear.

## 2026-07-20: Use `tethers-0.1/` As The Active 0.1 Tree

Decision: `tethers-0.1/` is the active development tree for the entire 0.1
cycle, not a frozen snapshot. Historical baselines will be preserved through Git
commits and later Git tags, not by copying complete source trees into new
version-numbered folders.

Reason: The verified native Windows opam switch is path-bound to
`tethers-0.1/engine-ocaml`. Moving or renaming the tree would disturb the
working environment, and version history belongs in Git.

## 2026-07-20: Keep The Prototype Source Intact

Decision: Do not edit imported source files during the first integration pass.

Reason: The request is to inspect, extract, integrate, and document. Changing
semantics before verification would mix preservation with implementation.

## 2026-07-20: Document Before Expanding Scope

Decision: Add project-control documents under `docs/`.

Reason: The workspace needs a clear overview, active goal, decision log, and
task queue before compilation or further design work begins.

## 2026-07-20: Give Cline Concise Workspace Rules

Decision: Add `.clinerules/` and `.clineignore` at the primary workspace root
so Cline has concise project-specific operating guidance.

Reason: Cline is the bounded implementation worker for this project and should
receive enough architectural context to avoid dangerous changes without loading
the full project guidance for every mechanical task.

## 2026-07-20: Adopt `docs/CONSTITUTION.md` As The Enduring Constitution

Decision: `docs/CONSTITUTION.md` is the authoritative Tethers constitution and
governs enduring design principles.

Reason: The constitution should exist once as a stable document that other
project guidance can reference concisely. `tethers-0.1/SPEC.md` remains the
authority for current precise 0.1 language and protocol semantics.

## 2026-07-20: Use A Compact OCaml Guide For AI Agents

Decision: Tethers uses `docs/OCAML_GUIDE_FOR_AGENTS.md` plus task-relevant
official OCaml, Dune, opam, and Yojson documentation for OCaml implementation
tasks.

Reason: AI coding agents need verified project-specific OCaml guidance without
loading an entire language manual into every task. The compact guide points to
official documentation for version-specific details, and the compiler plus
Tethers contract tests remain the final authority.

## 2026-07-20: Pre-Evaluation Parse Errors Remain Minimal

Decision: Tether source parse errors (`parse_error`) remain minimal
pre-evaluation errors. The engine returns only `protocol_version`, `status`,
and `error` — no evaluation identifiers, no plan, and no Trail.

Reason: Parsing is part of validating the submitted request; evaluation has
not begun and no evaluation Trail exists. When the Tether source is
syntactically invalid, the request is semantically incomplete and the engine
cannot identify which identities a correlated envelope should carry. Partially
correlated envelopes that contain some identifiers and not others would
introduce three categories of error shape (minimal, partial, full) rather than
the simpler two-category model (minimal pre-evaluation, fully correlated
evaluation/planning). Tethers 0.1 uses only:

1. minimal pre-evaluation errors (request-decoding, version, structural,
   parse);
2. fully correlated evaluation/planning errors (Condition, Action).

## 2026-07-20: Reject Duplicate Action Argument Names

Decision: Each argument name may appear at most once within a single Action.
Duplicate names are rejected as parse errors before evaluation begins.

Reason: Duplicates create ambiguity about which value the Tether author
intended. The host should not silently select one value over another. Rejecting
duplicates during parsing provides a clear, deterministic error before any
evaluation identity or Trail is established. Different Actions may
independently reuse the same argument name without conflict.

## 2026-07-20: Reject Duplicate Capability Names

Decision: Every Capability name must be unique within a request. Duplicate
Capability names are rejected as a minimal pre-evaluation `invalid_capability`
error before any evaluation identifiers, plan, or Trail are established.

Reason: Actions address Capabilities by name. When two entries share the
same name, the engine cannot determine which schema the Tether author
intended. The name is compared without regard to version because a
name+version pair still creates ambiguity for Action lookup. Silent
selection of the first (or last) declaration would mask author error.
Deterministic rejection produces a clear, unambiguous response.

## Open Decisions

- Whether future documentation should live at the workspace root, inside
  `tethers-0.1/`, or both.
- What the first post-baseline implementation milestone should be.
