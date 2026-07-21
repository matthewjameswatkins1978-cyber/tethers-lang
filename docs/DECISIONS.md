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

## 2026-07-21: Columbo C1 Complete — Begin C2 Trusted Manifest Store

Decision at that checkpoint: Columbo C1 was complete at `34330b3`, and the
next set was C2 — Trusted Manifest Store, beginning with C2a — Verify declared
manifest digest. C2 is now complete at `25ab2bb`; see the later C2 decision
entry below.

C1 final state:
- C1a1: data types and structured error model ✓
- C1a2: strict parsing, duplicate-key rejection ✓
- C1b1: JCS dependency verified (`serde_json_canonicalizer` 0.3.x) ✓
- C1b2: canonicalisation, SHA-256, golden vectors ✓
- C1c: semantic cross-field validation ✓

C2 original planned task order:
- C2a: Verify declared manifest digest
- C2b: Store verified manifests with identity and digest indexes
- C2c: Define insertion conflicts, idempotency, and retrieval semantics

C2a verifies that a manifest's supplied top-level `digest` matches the
digest computed from its authoritative fields. It produces a
`VerifiedManifest` type that C2b insertion requires, making unverified
insertion impossible at compile time. Digest verification proves content
identity and integrity; it does not prove provider trust, authorisation,
or permission to dispatch.

Digest syntax: `sha256:` followed by exactly 64 lowercase hex characters.
Uppercase, whitespace, and other algorithm prefixes are rejected.

Reason: C2 establishes the boundary between valid manifests and verified
ones before any storage, trust, or dispatch decision.

## 2026-07-21: Columbo C2 Complete — Begin Joint Runtime Slice

Decision: Columbo C2 is complete at checkpoint `25ab2bb`. The Trusted Manifest
Store now admits only `VerifiedManifest`, indexes manifests by exact
`(capability_name, capability_version)` identity and verified digest, supports
idempotent reinsertion, returns deterministic identity/digest conflict errors,
and leaves both indexes unchanged on every rejection.

C2 final state:
- C2a: declared digest verification and `VerifiedManifest` boundary ✓
- C2b: verified manifest store with identity and digest indexes ✓
- C2c: insertion conflicts, idempotency, and retrieval semantics merged into
  C2b ✓

The next Tethers target is the vertical runtime slice defined by the joint
Tethers/Lantern Keeper canonical architecture. Provider admission, live
capability projection, exact version resolution, effective policy, serial
dispatch, result Anchors, and execution Trail writing are one coherent route,
not independent architectural products. Tethers remains a general coordination
and behaviour layer; it has no built-in knowledge of Lantern Keeper, memory,
AI, or MCP-specific business meanings. AI judgement is an explicit capability
Action whose structured result can become a later Anchor.

Reason: The accepted joint architecture keeps the already implemented manifest
verification/store baseline and directs future work toward one real
Anchor-to-Plan-to-permission-to-execution-to-Trail slice.

## 2026-07-21: Adopt Joint Tethers/Lantern Keeper Canonical Architecture

Decision:
[`architecture/TETHERS_LANTERN_KEEPER_CANONICAL_ARCHITECTURE.md`](architecture/TETHERS_LANTERN_KEEPER_CANONICAL_ARCHITECTURE.md)
is the synchronized joint architectural contract and build foundation for
Tethers plus Lantern Keeper. The same file is installed in both repositories
and must be updated in both places for future architectural edits.

The report does not replace `tethers-0.1/SPEC.md` for exact current language
semantics or `docs/CAPABILITY_BRIDGE.md` for the current manifest/capability
contract. It governs the joint build order and ownership boundaries:
Tethers coordinates, Lantern Keeper remembers, AI interprets through explicit
capabilities, and Matthew remains the final authority.

Reason: The two projects need one shared target architecture without turning
Tethers into a memory engine or Lantern Keeper into a general workflow engine.

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
2. **Manifest -> planner**: The trusted host supplies an approved capability
   projection containing planning-relevant fields and an opaque manifest digest.
   The planner does not inspect or trust the complete manifest.
3. **Planner -> Plan Action**: A bridge Action references capability name,
   version, and the same manifest digest supplied in deterministic evaluator
   input. Host must still resolve the digest and prove the pinned provider/tool
   binding before dispatch.
4. **Plan Action -> host**: Nothing trusted. Plan is a request, not permission.
5. **Host -> remote MCP call**: Nothing trusted. Remote server is untrusted at
   call time.

The contract digest is fixed to SHA-256 over RFC 8785/JCS canonical bytes.
It covers `manifest_format_version`, capability name and version, complete input
and output schemas, effects, permission scope, reversibility, determinism,
idempotency mechanism, confirmation policy, timeout and retry policy, provider
identity (host-assigned, not self-reported), binding kind, server name, MCP tool
name, and adapter identity/version. Only the digest value itself and exact
top-level display metadata (`title`, `description`) are excluded. The manifest
does not carry `digest_algorithm`; algorithm agility is deferred until a real
need exists.

Manifest parsing must reject duplicate keys in every object recursively,
including arbitrary nested `input_schema` and `output_schema` objects. C1b1 must
verify a maintained Rust RFC 8785/JCS implementation against official examples
and test vectors before C1b2 implements canonicalization and digesting. If no
suitable implementation is verified, implementation stops for a separate design
decision rather than using a casual homemade fallback.

C1b1 selected `serde_json_canonicalizer` 0.3.x, reviewed at 0.3.2, for C1b2
JCS canonical byte generation. Columbo will use it only after C1a2 strict
parsing has rejected duplicate keys recursively and after C1b2 has removed
non-digest fields from the manifest value. Columbo remains responsible for
manifest I-JSON constraints, for rejecting values outside the accepted number
domain, and for adding project golden vectors before computing SHA-256 digests.
Rejected alternatives were `serde_jcs`, `json-canon`, `canon-json`, and
`jcs-canonicalize` for weaker direct fit, documented concerns, or bundled
hashing scope.

A manifest's `confirmation_policy` declares what is acceptable but does not
grant standing authority. Actual standing approval is separate host-controlled
state bound to the exact manifest digest, approved scope, approving identity,
and creation/revocation information.

Idempotency requires a concrete mechanism (`argument_key`, `server_dedup`, or
`none`), not merely the word `"conditional"`. For `argument_key`, the manifest
must name the argument and key source. For `server_dedup`, the trusted
host/provider/adapter evidence must describe the deduplication key, scope, and
lifetime, pinned by the manifest binding. Without a concrete reviewed mechanism,
automatic retry is forbidden for effectful Actions; `requires_idempotency_proof:
false` cannot bypass that rule.

Tether source and Plans never contain or supply credential values. Manifest
schemas may describe credential-shaped inputs, but Columbo injects actual
credential values only from trusted host storage at dispatch. Secret-like-value
scanning is defence-in-depth; rejections must tell authors to remove the value,
not rename or re-encode it.

Output schemas must reject effectively unconstrained schemas. They do not all
need to be objects with properties; concrete primitive, array, enum, and
structured-object schemas may all be valid. Unstructured provider output
requires a reviewed typed adapter.

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
credential values never appear in declarative artifacts.

## Open Decisions

- Whether future documentation should live at the workspace root, inside
  `tethers-0.1/`, or both.
