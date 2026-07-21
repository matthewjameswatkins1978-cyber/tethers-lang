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

## 2026-07-20: Tethers Owns Its MCP Interface In OCaml

Decision: MCP connects directly to Tethers. The MCP implementation belongs in
OCaml, in the Tethers project, and must call the same evaluator boundary as the
existing engine. Lantern Keeper is a connected host and capability provider,
not the MCP hub. The first MCP surface is planner-only over stdio: it evaluates
complete Tethers requests and returns the existing Plan and Trail envelope
without executing Actions.

Reason: Tethers is the deterministic planner and should expose that planning
surface directly. Keeping the MCP adapter in OCaml avoids parallel Rust and
OCaml interpretations of the language, preserves the signed-off 0.1 protocol,
and keeps host permission and execution responsibilities outside Tethers Core.

## 2026-07-20: Restrict Condition Expected Values To Literals

Decision: Conditions may only compare Fact values against literal values
(strings, integers, booleans). `anchor.*` references are rejected during
Condition parsing as `parse_error` before evaluation begins.

Reason: Conditions test known Facts against known thresholds. Allowing
`anchor.*` references in Conditions would require the engine to resolve
event data during Condition evaluation, which mixes Fact and event
resolution contexts before the evaluation lifecycle clearly separates
them. Action arguments remain free to use `anchor.*` references because
Action resolution occurs after all Conditions have matched, when the
event data is fully available and the resolution context is unambiguous.

## 2026-07-21: Treat OCaml MCP Libraries As References For The First Server

Decision: Do not add `ocaml-mcp` or `snf_mcp` as a dependency for the first
Tethers MCP server. Use them as reference implementations only. Consider the
smaller OCaml `jsonrpc` package later, after the evaluator boundary and MCP
transcript fixtures exist, if it reduces JSON-RPC plumbing without weakening
Tethers' protocol control.

Reason: `ocaml-mcp` is real and useful, but currently targets MCP
`2025-06-18` while the public MCP specification now redirects to `2025-11-25`,
and its transport/SDK shape brings broad Eio, socket, HTTP, schema-generation,
and OCaml-development-server machinery that Tethers does not need for a narrow
planner-only stdio server. `snf_mcp` proves OCaml MCP stdio practicality,
including Windows CI and shutdown handling, but it vendors MCP code inside a
web-search server with network-heavy dependencies. Tethers should keep the
first server small, deterministic, and application-agnostic.

## 2026-07-21: Capability Bridge Trust Boundary (M7)

Decision: The capability bridge design (`docs/CAPABILITY_BRIDGE.md`) establishes
that MCP tool discovery advertises what a server claims to provide, but
discovered tool metadata and annotations are untrusted. A tool becomes a
candidate Tethers capability only through an explicitly installed, reviewed,
trusted host-side manifest. The Tethers planner may propose a capability Action
but can never execute it. The permissioned host resolves the exact manifest by
digest, re-validates arguments and scope, obtains confirmation where required,
dispatches the bound MCP call, validates the result, and appends execution
Trail entries.

Key trust boundaries:

1. **MCP tool discovery -> manifest author**: Nothing is trusted. All tool
   metadata is untrusted advertising claims.
2. **Manifest -> planner**: Manifest fields form the capability contract.
3. **Planner -> Plan Action**: Action references capability name, version, and
   manifest digest. Host must still re-validate.
4. **Plan Action -> host**: Nothing trusted. Plan is a request, not permission.
5. **Host -> remote MCP call**: Nothing trusted. Remote server is untrusted at
   call time.

The contract digest covers every execution-authoritative manifest field:
capability name and version, input and output schemas, effects, permission
scope, reversibility, determinism, idempotency mechanism, confirmation policy,
timeout and retry policy, provider identity (host-assigned, not
self-reported), binding kind, server name, MCP tool name, and adapter
identity/version. Display-only metadata is excluded.

A manifest's `confirmation_policy` declares what is acceptable but does not
grant standing authority. Actual standing approval is separate host-controlled
state bound to the exact manifest digest, approved scope, approving identity,
and creation/revocation information.

Idempotency requires a concrete mechanism (`argument_key`, `server_dedup`, or
`none`), not merely the word `"conditional"`. For `argument_key`, the manifest
must name the argument and key source. Without a concrete reviewed mechanism,
automatic retry is forbidden.

Provider identity uses host-assigned identity with
`identity_source: "host_configuration"` because MCP `serverInfo` is
self-reported and mutable and therefore insufficient for trustworthy provider
identity.

Reason: This trust boundary governs every future adapter (MCP, Git, Google,
Obsidian, Lantern Keeper, and others). The design ensures Tethers Core remains
application-agnostic while providing a safe, auditable path from untrusted
discovery to trusted execution. The decision applies the established Tethers
architectural rules: Tethers plans, hosts execute; schemas describe, policies
authorise, hosts enforce, Trails record; discovery never grants permission;
credentials never appear in declarative artifacts.

## Open Decisions

- Whether future documentation should live at the workspace root, inside
  `tethers-0.1/`, or both.
