# Decisions
## 2026-08-01: Security, trust, credentials, and sandbox boundary

1. Host policy and provider containment are separate.
2. Providers remain untrusted after signing and conformance.
3. Package signature v1 uses Ed25519 over a domain-separated semantic package digest.
4. Publisher trust comes only from the host trust store.
5. Trust-on-first-use is not permitted.
6. Revoked keys cannot satisfy current trust.
7. Unsigned packages require explicit developer mode.
8. Job Object supervision is not a complete security sandbox.
9. Supervised and isolated provider profiles remain distinct.
10. Third-party production providers require proven isolation.
11. Provider environments are constructed from scratch.
12. Credentials use host-owned profiles and exact session delivery.
13. Credential values enter no durable evidence.
14. Filesystem and network access begin at deny.
15. Security violations quarantine or disable and never authorise retry.
16. No implementation or Tether syntax change is introduced.

## 2026-08-01: Lifecycle, outcome, event, and conformance boundary

1. Installation, session, health, catalogue, binding, operation, replay,
   event-admission and conformance states remain separate.
2. Readiness is capability-specific.
3. Unattempted is not a canonical outcome.
4. Canonical outcomes remain succeeded, failed and uncertain.
5. Ambiguity after invocation remains uncertain.
6. Standard Result Anchors exist only after attempted durable outcomes.
7. Replay authority remains separate from Trail.
8. No replay or restart path authorises another provider call.
9. Plug Anchors require stable source event identity.
10. Durable external-event admission is distinct from operation replay and J11
    causal admission.
11. Acknowledgement follows durable event admission.
12. Cursors do not replace event identity.
13. Conformance is host-orchestrated evidence, not authority or permission.
14. Passing conformance does not approve, install or enable a Plug.
15. No automatic retry or Tether syntax change is introduced.

## 2026-08-01: Capability class, effect, and scope boundary

1. Every capability has exactly one reviewed class.
2. Class, effect, scope, policy, and outcome are distinct.
3. Action, Query, and Anchor target the first Plug architecture.
4. Job, Stream, and Human Task remain reserved.
5. Effects must be complete, conservative, and host-understood.
6. Unknown effects fail closed.
7. Scope is structured, deterministic, and host-readable.
8. Effective scope is the intersection of supported scope, installation grant,
   policy, and resolved target.
9. Scope mappings are explicit and never inferred from prose.
10. Query remains permissioned.
11. Anchor events are admitted only after source and scope validation.
12. Existing 0.2 scope behaviour remains unchanged.
13. Drift cannot silently alter class, effects, or scopes.
14. No automatic retry or Tether syntax change is introduced.

## 2026-08-01: `.tetherplug` package format v1 boundary

1. `.tetherplug` v1 is a narrowly profiled ZIP archive.
2. `plug.json` is the strict root package document.
3. One package contains one provider and multiple related capabilities.
4. Internal package paths use lowercase safe ASCII.
5. Every payload file is indexed with SHA-256 and size.
6. Semantic package digest is the JCS/SHA-256 digest of validated `plug.json`.
7. Raw archive digest remains separate.
8. Capability-manifest digests remain separate.
9. Package, provider, and capability identity axes remain distinct.
10. Packages contain no credentials, effective policy, or installed state.
11. Signatures are detached evidence and grant no permission.
12. Inspection performs no execution.
13. Installation and generated runtime configuration remain host-owned.
14. No automatic updates or dependency installation are introduced.

## 2026-08-01: Tethers Socket v1 and first MCP binding

1. Socket v1 is a semantic normalization contract.
2. The first binding uses standard MCP methods rather than custom methods.
3. The Tethers host is the MCP client and the Plug provider is the MCP server.
4. This is separate from the existing Tethers Core-facing MCP server.
5. JSON-RPC IDs are session correlation, not durable execution identity.
6. Tool discovery remains untrusted.
7. Structured output is authoritative only after trusted-schema validation.
8. `tools/list_changed` is catalogue drift, not a Tethers Anchor.
9. MCP Tasks, progress, resources, and elicitation do not automatically map to
   Tethers capability classes.
10. No automatic retry is introduced.
11. Anchor delivery and canonical outcome expansion remain J18F decisions.
## 2026-08-01: Universal Plug Architecture boundary

1. The Tethers Socket is a semantic host-provider contract, not a transport.
2. Socket semantics, protocol binding, and transport are separate layers.
3. The first intended stack is MCP 2025-11-25 over local stdio.
4. Tethers Core remains unaware of packages and Plugs.
5. The host owns trust, permissions, credentials, bindings, outcomes, lifecycle,
   and Trail.
6. Vendor-specific translation remains outside the host.
7. Six capability classes are reserved; only Action, Query, and Anchor target
   the first implementation.
8. Job, Stream, and Human Task remain unimplemented.
9. Version axes remain independent.
10. Paper validation is required before architecture acceptance.

## 2026-07-31: J14C real local file move capability proof

Decision: Matthew authorised J14C after J14 publication. J14 proved the execution
machinery with a fixture ping capability, but every accepted row so far ended in
an echoed fixture string. J14C proves that the same machinery performs one
intelligible, externally visible job — moving a real file from a bounded inbox to
a bounded invoices folder — without changing the runtime.

J14C remains inside the existing 0.2 promise of one real local permissioned
execution loop. Host scope binds `/source_path` only because the accepted runtime
currently supports one JSON pointer per capability. The dedicated provider
independently enforces both source and destination confinement, canonical-root
containment, reparse-point inspection, and safety boundaries for traversal,
overwrite, and junction escape. No watcher, GUI, general filesystem API,
overwrite, retry, or production host redesign is included.

The proof harness reports exactly F01–F09: check admission, untouched photo after
non-match, a successful invoice move preserving byte content, public Trail
explaining the move, blocked replay, out-of-scope denial, traversal refusal,
overwrite refusal, and junction-escape refusal. Every row proves one JSON
envelope, matching process and embedded exit codes, exact provider method counts,
exact identity and Result Anchor rules, durable Trail evidence, and no retry.
All effects occur beneath one unique system temporary root with a space and a
non-ASCII character; no repository file is mutated.

Reason: fixture ping proved the execution machinery but not an intelligible
external effect. J14C proves a recognisable real-world effect through the same
public check/run/trail commands and the same trust boundaries. The safety split
between host-side scope on `/source_path` and provider-side confinement for both
paths is a deliberate design decision, not a gap.

## 2026-07-31: J14B negative public integration matrix

Decision: J14B proves the J14 negative matrix through 11 reproducible native
Windows boundary rows using the public check/run commands and one focused Rust
test seam. The `tethers-stdio-fixture.ps1` provider gained three deterministic
failure modes (`run-explicit-error`, `run-invalid-output`, `run-hang-call`) that
exercise executor failure, invalid output, and uncertain timeout without a
production fault switch. Row M06 (post-admission intent failure) uses the
accepted `#[cfg(test)]` test boundary with zero production code change. Row M11
(causal depth) uses the existing debug-only `event-admission-trail-probe`
compiled boundary. The public run CLI exposes `result_anchor` only for
`Completed` outcomes; the `ExecutionServiceResult::Failed` and `::Uncertain`
variants lack the `response` field. The public M07/M08/M09 rows prove envelope
status, process and embedded exit codes, machine codes, trusted execution
identity, exact provider method counts, and durable Trail outcomes. Two focused
test-only Rust tests on the accepted execution seam prove the internal
capability.failed and capability.uncertain Result Anchor kinds, which the
current public Failed and Uncertain result variants do not expose.

J14B completes its implementation claim when all eleven rows pass. J14 becomes
complete only after Lucy independently accepts and the candidate is published.

Reason: the negative matrix must be proved without adding production
fault-injection branches or weakening the existing trust boundaries. The three
new fixture modes, one test-only Rust seam, and the existing debug probe
together cover all eleven negative cases while preserving the invariant that no
hidden runtime bypass exists.

## 2026-07-30: J14A public execution identity and positive scenario

Decision: public run data exposes the exact host-issued execution ID when
replay admission established a trusted identity. Callers and planners cannot
provide or derive that identity. Result Anchor schema remains unchanged. Trail
accepts the returned identity directly. Exact replay returns the same identity
and causes no second effect. The canonical J14A scenario proves check, run,
trail and replay. J14 remains incomplete until the negative matrix is accepted.

Reason: the public run command previously did not expose the
execution ID required by the trail command, breaking the run-to-trail
round-trip. The typed ExecutionServiceResult boundary now carries trusted
identity evidence without changing replay ID generation, storage format,
or the Result Anchor schema.

## 2026-07-30: J13C public Trail inspection boundary

Decision: `tethers-reference-host trail` accepts exactly
`--trail <ABSOLUTE_PATH>` and `--execution-id <exec_UUID>`. The existing
`replay::ExecutionId::parse` remains authoritative.

Inspection is read-only and uses only the supplied Trail file. It does not
search, inspect replay storage, repair records, start an engine or provider, or
execute a Tether. Matching top-level `execution_id` entries retain their file
order and original lexical JSON form.

Malformed or ambiguous Trail content fails closed as `audit_failed`; zero
matching entries returns `not_found`. J13 becomes complete only after this
command is accepted.


## 2026-07-30: Toolchain baseline enforcement

Tethers uses Rust 1.89.0 with edition 2021 for the reference host and OCaml
5.5.0 for deterministic Core. Cargo declares MSRV 1.89 and the repository
selects the verified Rust toolchain through `rust-toolchain.toml`. The OCaml
package supports the 5.5 minor series while a committed opam lock records the
exact verified Dune 3.24.0 and Yojson 2.2.2 resolution. Dune project language
remains 3.10.

The PowerShell preflight is non-mutating. It disables rustup automatic
installation process-locally, proves the pinned Rust toolchain and components
are already installed, and restores the previous environment value. OCaml
checks require an explicit absolute directory-switch path and never search
worktrees or fall back to a global switch. Toolchain upgrades require a
separate decision.

## 2026-07-30: Canonical OCaml Engineering Guide

Decision: `docs/OCAML_GUIDE_FOR_AGENTS.md` is the canonical required operating
guide for every OCaml task. It records the Core/host boundary, deterministic
engineering rules, exact worktree-safe `OcamlSwitchPath` contract, verification
expectations, and stop conditions. `AGENTS.md` requires it before the first
OCaml edit.

Reason: The former compact guide no longer carried enough authoritative detail
for the deterministic Core and multi-worktree native-Windows workflow. One
canonical guide prevents competing OCaml instructions.

`TOOLCHAIN-BASELINE-01` is now implemented and enforced: the repository contains `rust-toolchain.toml`, MSRV 1.89, tightened OCaml range, and committed `tethers_engine.opam.locked`
decision. The future bounded implementation task alone may change compiler or
dependency constraints, create locks, add the non-mutating preflight, or alter
toolchain-enforcement files. Windows binary-mode stdio and compiler warning
policy remain separate excluded scopes.

## 2026-07-29: J13B Host Execution Service Architecture

Decision: Extract host execution machinery from `main.rs` into a dedicated
`host_execution.rs` application-service module. Move the `CapabilityExecutor`
trait to `executor.rs` for reuse. Extend `EngineSession` with `evaluate_tether`
using `tools/call` with `tethers.evaluate`.

The service accepts an immutable `PreparedRuntime`, retained engine session,
and typed `PreparedEvaluationInput` values with explicitly supplied
`evaluation_id`. It applies all existing capability resolution, scope, policy,
replay, durable-intent and dispatch boundaries.

No public `run` command. No evaluation-ID derivation rule. The future CLI
layer will map the typed `ExecutionServiceResult` to the frozen status and
exit-code vocabulary.

Reason: The extraction keeps the CLI boundary thin. The retained sessions
prevent repeated process launches. All safety gates (replay admission, durable
intent, provider invocation) remain inside the service, enforced by the
compiler.

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

## 2026-07-24: J03 Four-Outcome Host Policy Contract

Decision: Tethers 0.2 policy resolution is host-owned and returns exactly one
of `allow`, `ask`, `deny`, or `unavailable` for each proposed Action. This
decision freezes the contract for J04 and J05; it adds no runtime behaviour.

### Effective-policy inputs

The host evaluates a proposed Action against these explicit inputs only:

1. the selected Tether Set's exact declared capability requirement (name and
   version);
2. the planned Action's `evaluation_id`, `plan_id`, `action_id`, capability
   name/version, resolved non-secret arguments, and bridge pins;
3. the current trusted-manifest-store resolution for that exact capability,
   including verified `manifest_digest`, host-assigned `provider_identity`,
   live provider binding, manifest input schema, permission scope, and
   `confirmation_policy`;
4. host-local policy rules, keyed by exact capability name and version, plus
   their configured resource scope; and
5. for a resumed Ask only, one host-issued approval record bound as specified
   below.

Capability schemas describe. Policies authorise. Hosts enforce. Trails record.
The planner, a Tether Set, a manifest, a discovered provider, and AI are never
policy authorities.

### Fail-closed precedence

Evaluate in this order. The first matching rule wins:

1. Missing, malformed, or non-matching Action identity; missing required
   bridge pin; Action capability/version not exactly declared by the selected
   Tether Set; failed input-schema validation; or permission-scope violation
   -> `deny` with a precise host reason.
2. No admitted manifest; revoked, stale, or mismatched manifest digest;
   provider identity/binding mismatch; or provider absent from the current
   availability snapshot -> `unavailable`. This is a binding fact, not a
   policy override.
3. An exact host-local `deny` rule -> `deny`.
4. On an Ask resume only, a host-issued approved record whose complete proof
   still matches returns one `allow` after the host atomically consumes it.
   This is a confirmation of this exact Action, not a standing Allow.
5. Manifest `per_call_required: true`, or an exact host-local `ask` rule ->
   `ask`.
6. An exact host-local `allow` rule whose configured scope covers the Action
   -> `allow`.
7. Every omitted, malformed, ambiguous, unsupported, or out-of-scope policy
   rule -> `deny`.

Exact name/version rules override the host default. The host default is
`deny`. A local Allow cannot bypass a mandatory per-call confirmation, schema
validation, scope enforcement, or current binding proof. A current binding is
checked before every dispatch, including an Ask resume. A valid one-shot human
approval satisfies that single mandatory confirmation; it does not bypass any
other check or authorise another Action.

### Ask proof and one-shot decision

`ask` creates a host-local pending approval record. Its proof contains these
literal fields and their SHA-256/JCS digests:

- `evaluation_id`, `plan_id`, and `action_id`;
- capability name and semantic version;
- `argument_digest`: `sha256:` plus SHA-256 of RFC 8785/JCS bytes of the
  complete resolved non-secret Action arguments;
- `manifest_digest`; and
- `provider_identity`.

The host also computes `approval_binding_digest` as SHA-256/JCS over that
complete proof object with `approval_format_version: "1"`. J05 must compare
the constituent fields and the binding digest; the digest is not a substitute
for field-by-field checking. Credential values injected at dispatch are not
part of the Action, either digest, the approval record, or the Trail.

Only a host-recognised human decision endpoint may mark this exact pending
record approved or denied. AI, a Tether, a provider, a manifest, and a caller
cannot self-approve. The UI/API for that endpoint is deliberately deferred.

An approval is valid only until the first matching resume attempt. The host
first re-evaluates every non-approval input, including current binding, schema,
scope and Deny. Only if those checks pass and the ordinary result is Ask does
it atomically consume the matching approval and issue the one Allow proof for
intent preparation. Any changed proof field, changed binding, policy or scope
failure, explicit cancellation/denial, or host-process restart discards the
record. A consumed approval is never restored if intent recording fails; a
later attempt must begin a new Ask. No standing or reusable approval is
created by this contract.

### Dispatch and Trail contract

All host entries include the Action identifiers, capability name/version,
manifest digest and provider identity when known, a reason code, and a
redacted argument summary or `argument_digest`; they never record credentials.

| Outcome or state | Required host Trail record | Dispatch / result Anchor |
| --- | --- | --- |
| `allow` | `policy_allowed` with exact policy-rule source | May proceed to durable intent; no Anchor until an attempted call has an outcome. |
| `ask` pending | `approval_requested` with the complete approval proof | No intent, executor call, execution outcome, or standard result Anchor. |
| Ask approved then resumed | `approval_granted`, then `approval_consumed` before intent preparation | May proceed only if fresh re-evaluation returns Allow. |
| Ask denied, cancelled, stale, or invalidated | `approval_denied`, `approval_cancelled`, or `approval_invalidated` in the authorisation phase | Not dispatched; no standard result Anchor. |
| `deny` | `policy_denied` with the precise rule/validation/scope reason | Not dispatched; no standard result Anchor. |
| `unavailable` | `capability_unavailable` with the binding-resolution reason | Not dispatched; no standard result Anchor. |

An unresolved or otherwise unattempted Action is never represented as
`succeeded`, `failed`, or `uncertain`. Only a provider call that crossed the
intent/dispatch boundary may later produce a standard `capability.succeeded`,
`capability.failed`, or `capability.uncertain` result Anchor.

### J04/J05 acceptance matrix

1. A declared, exactly resolved, schema-valid, in-scope Action with an exact
   local Allow and no mandatory confirmation returns Allow deterministically.
2. A missing policy rule, malformed policy record, unsupported scope rule, or
   out-of-scope local Allow returns Deny; no intent, executor call, or result
   Anchor occurs.
3. An undeclared capability/version, malformed Action identity or pins, input
   validation failure, and manifest permission-scope violation each return
   Deny with distinct reason evidence.
4. Missing admission, revoked/stale digest, unavailable provider, and provider
   identity/binding mismatch each return Unavailable before dispatch, even
   when a local Allow exists.
5. An exact Deny overrides an exact Allow/default; a mandatory per-call
   confirmation overrides Allow; exact name/version rules cannot authorise a
   different version.
6. An Ask record contains each required proof field and the independently
   reproduced `argument_digest` and `approval_binding_digest`; it contains no
   credential value.
7. While Ask is pending, exactly one `approval_requested` Trail record exists
   and there are zero intent records, executor calls, outcome records, and
   standard result Anchors.
8. A human approval resumes only the same proof once. Changing arguments,
   manifest digest, provider identity, capability version, evaluation/plan/
   Action ID, or either proof digest invalidates it and prevents dispatch.
9. A matching approved resume first rechecks all non-approval gates, then
   consumes the approval before intent preparation; it cannot be reused after
   intent failure, executor failure, completion, or host restart.
10. Human denial, cancellation, expiry by restart, and invalidation each emit
    the specified Trail record and make zero executor calls/result Anchors.
11. Every Allow, Ask, Deny, Unavailable, approval decision, and invalidation
    Trail entry preserves known pins and redacts arguments/credentials.
12. Repeating resolution with the same declared inputs and snapshots yields the
    same outcome and proof digest; policy resolution performs no I/O or
    dispatch.

### J03b: scope assessment boundary for J04

Decision: J04 combines an explicit host-produced scope assessment into the
effective-policy result; it does not infer resource-bearing Action arguments
from names such as `path`, `repository`, or `calendar`.

The scope assessor is host/binding-owned. It receives the verified manifest,
its declared `permission_scope`, the resolved non-secret Action arguments, and
the configured provider binding. It returns exactly one of:

- `within_scope` — the host has checked the declared scope against the resolved
  arguments;
- `scope_violation` — the checked arguments fall outside that scope; or
- `scope_not_established` — no trusted binding-specific assessor exists, the
  required argument is absent/ambiguous, or the assessment cannot be made.

For a structured manifest scope (`path_prefix`, `repository`, or `calendar`),
`scope_violation` and `scope_not_established` both yield `deny` before any
local Allow or Ask rule. `within_scope` continues through normal J03/J03a
precedence. For `Unrestricted`, the manifest's existing mandatory per-call
confirmation applies and no automatic Allow is created from scope.

The policy resolver must not accept a Plan-supplied boolean as proof of scope.
The caller supplies a host-owned assessment object; J04 defines and tests the
policy boundary, while a later binding/adapter task implements concrete
path/repository/calendar extraction. This is fail closed without inventing a
generic argument-name convention or changing the manifest format.

## 2026-07-28: J12 Runtime Configuration Foundation

Decision: The frozen J12 local runtime configuration is a single strict JSON
file that selects one Tether Set, its ordered source files, exact capability
requirements, explicitly configured stdio provider bindings, reviewed manifest
paths with mandatory pinned digests, scope bindings, and exact local policy
rules. Packet 1 implements parsing, validation, and materialisation only;
Packet 2 owns runtime wiring and live scope assessment.

### Frozen JSON shape

```json
{
  "format_version": "0.1",
  "tether_set": {
    "id": "example.local",
    "version": "1",
    "tethers": [
      {
        "id": "record-completed-task",
        "version": "demo-v1",
        "source_path": "tethers/record-completed-task.tether"
      }
    ],
    "capability_requirements": [
      {
        "name": "lantern.task.record",
        "version": 1,
        "reason": "Record a completed task"
      }
    ]
  },
  "providers": [
    {
      "id": "lantern-local",
      "display_name": "Lantern Local",
      "transport": {
        "kind": "stdio",
        "command": "pwsh.exe",
        "args": ["-NoProfile", "-File", "providers/lantern.ps1"],
        "protocol_version": "2025-11-25"
      },
      "capabilities": [
        {
          "name": "lantern.task.record",
          "version": 1,
          "manifest_path": "manifests/lantern-task-record.json",
          "pinned_digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
          "scope_binding": {
            "kind": "path_prefix",
            "argument_json_pointer": "/path"
          }
        }
      ]
    }
  ],
  "policy": {
    "default": "deny",
    "rules": [
      {
        "name": "lantern.task.record",
        "version": 1,
        "decision": "allow"
      }
    ]
  }
}
```

### Design invariants

1. **One configuration file, one selected Tether Set.**
2. **Explicit configuration, not discovery.** No provider discovery.
3. **Only stdio transport.**
4. **Every provider capability must pin a reviewed manifest digest.**
5. **Scope binding: binding-owned extraction, manifest-owned authority.**
6. **Default deny with exact per-capability rules.** No wildcards.
7. **Relative path resolution** against the config file's parent directory.
8. **No secrets, interpolation, or package management.**
9. **No J13 commands.** Packet 2 wires runtime behaviour.
10. **Structured scope kinds fail closed.** Only `path_prefix` in Packet 1.
11. **Duplicate-key rejection is shared** via `manifest::parse_value_no_dupes`.

### Packet 2 boundaries (superseded by Packet 2 implementation)

The boundaries listed in Packet 1 were corrected by Packet 2. See below.

## 2026-07-28: J12 Runtime Preparation And Scope Closure

Decision: J12 Packet 2 completes the local runtime foundation by turning a
LoadedRuntimeConfig into a complete, immutable PreparedRuntime. Packet 2 owns
filesystem loading, manifest verification, admission, scope assessment, and
launch-plan construction. It performs no provider launch, engine invocation,
dispatch, or Trail writing.

### Global exact-identity uniqueness

Every exact capability identity `(name, version)` must appear under exactly one
configured provider. Duplicate exact identities across providers are rejected
whether or not `scope_binding` is present.

Reasons:
- requirements identify capabilities by exact name and version;
- the TrustedManifestStore indexes by exact name and version;
- the configuration has no provider-selector field on requirements;
- silently choosing between providers would be non-deterministic authority.

### Asset confinement

Tether and manifest paths originate relative to the configuration directory.
Every source and manifest path is resolved against the canonical config
directory, canonicalised, and required to remain beneath it. `../` escapes,
directories, missing files, and unreadable or invalid text (including NUL)
are rejected.

### Manifest verification and admission

For each configured provider capability, the reviewed manifest file is read,
verified via `manifest::verify_manifest`, cross-checked against configuration
(name, version, provider identity, pinned digest), scope-binding compatibility
is validated, and the VerifiedManifest is admitted through the existing
`provider::admit_provider_manifest` boundary. No direct TrustedManifestStore
insertion bypass exists.

### Scope-binding compatibility

- PathPrefix manifest: scope_binding is required, kind must be PathPrefix,
  allowed_prefixes come only from the verified manifest.
- Unrestricted manifest: scope_binding must be absent.
- Repository or Calendar manifest: UnsupportedPermissionScope (fail closed).
- Unexpected binding on Unrestricted manifest: UnexpectedScopeBinding.
- Missing binding on PathPrefix manifest: MissingScopeBinding.

### Binding-owned scope assessor

`assess_action_scope` is a pure method that locates the exact PreparedCapability
using all four identity pins (capability name, bridge version, provider
identity, manifest digest), then applies PathPrefix assessment with configured
JSON Pointer extraction and segment-precise prefix matching. It returns
WithinScope, ScopeViolation, or ScopeNotEstablished. No I/O occurs.

### Planner capability descriptors

Deterministic descriptors are derived only from verified manifests, sorted by
(name, version). Input schemas are converted to scalar types only (string,
boolean, number). Live bridge pins are omitted; J13 adds them after provider
admission and availability.

### Provider launch plans

PreparedProvider carries literal command, arguments, protocol version,
ProviderConfig, canonical config directory as working directory, and verified
capability manifests. No provider is launched in J12.

### PreparedRuntime immutability

PreparedRuntime exposes read-only access to all fields. No mutation can alter
admitted identities, manifests, policy, or Tether order after construction.

### J12/J13/J14 boundaries (corrected)

- J12 Packet 2: filesystem loading, manifest verification/admission, scope
  assessment, launch-plan construction, PreparedRuntime assembly.
- J13: public check/run/trail commands, provider launch and live availability
  snapshots, OCaml engine invocation, Anchor+Facts assembly with
  PreparedRuntime, Trail location and printing.
- J14: actual configured provider capability call, intent, dispatch, validated
  output and Result Anchor through the public route, complete positive and
  negative integration matrix.

### Open Decisions

- Whether future documentation should live at the workspace root, inside
  `tethers-0.1/`, or both.

## 2026-07-28: J13A Process Supervision And Check Command

### Final Candidate 2.1 Corrections

1. Ctrl+C while reading stdin belongs to exit 10 (interrupted), not exit 3
   (invalid_data). Malformed, over-limit or multi-document stdin belongs to
   exit 3. J13A does not implement run/stdin yet, but the shared outcome model
   encodes this distinction.

2. Check ordering is:
   CLI/path validation -> J12 runtime preparation -> retained engine startup
   -> ordered validation of every Tether -> retained provider startup and
   availability verification -> result envelope -> child cleanup.

3. Future trail input uses `--execution-id <exec_UUID>` parsed with the
   existing `ExecutionId::parse` format. Not implemented in J13A.

4. After an Action invocation boundary, interruption classifies the outcome as
   uncertain (exit 7) and attempts required durable recording.
   - recording succeeds -> exit 7
   - required outcome, replay or result-event recording fails -> exit 8
   Exit 8 means a succeeded, failed or uncertain classification exists but
   required durable recording failed. J13A encodes this vocabulary; J13B
   implements Action execution.

5. There is no automatic fallback to the legacy positional parser. The legacy
   route is available only through the explicit hidden subcommand `__legacy`.
   Unknown commands, including "runn", return exit 2 and never enter legacy
   execution.

### Explicit --engine Ownership

The `--engine` option on the `check` command points to the OCaml MCP engine
executable (`tethers_mcp_main.exe`). The host owns the engine lifecycle:
launch, initialize, per-Tether validation, and shutdown. The engine is a
retained session (one launch per check command), not a per-request ephemeral
process.

### Engine-Before-Validation Ordering

The engine is started before any Tether validation. All Tethers are validated
in declared order through a single retained engine session. Providers are not
started until all Tethers validate successfully.

### One Request Per Tether

Each Tether receives exactly one `tethers.validate` MCP request through the
engine session. No `tethers.evaluate` request is permitted during check.

### Fixed Process Timeouts

Production constants:
- startup / initialize / request timeout: 10 sec
- normal graceful close wait: 2 sec
- maximum protocol line: 8 MiB
- retained stderr tail: 64 KiB

Tests may inject shorter timeout values through test-only constructors.

### Windows Job Object Ownership

Every child process is assigned to a Windows Job Object configured with
`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`. On host exit or panic, closing the Job
Object terminates descendant processes as well as the direct child. Job
assignment is best-effort: when the parent is in a restrictive job (common in
test harnesses), assignment may silently fail and the child still runs.

### One JSON Envelope

Every invocation emits exactly one compact JSON document to stdout. There is
no timestamp in the envelope. Diagnostics go only to stderr.

### No Timestamp

The base envelope shape is:
```json
{
  "schema": "tethers.cli/1",
  "command": "check",
  "status": "ok",
  "exit_code": 0,
  "data": {},
  "error": null
}
```

No `timestamp` or `timestamp_unix_ms` field exists in the CLI envelope.

### Hidden __legacy Gate With No Fallback

The legacy positional parser is reachable only through the explicit hidden
subcommand `__legacy`. Unknown commands, unknown subcommands, misspelled
commands (e.g., "runn"), and absent commands all return exit 2 and emit a
JSON envelope. There is no automatic fallback or fuzzy matching.

### Execution Identity Remains exec_<UUID>

The `ExecutionId` type continues to use `exec_<UUID>` format. Future J13B/J13C
will use `--execution-id` for trail input.

### Audit Failure Precedence

When an Action has been invoked and the outcome is uncertain, the host attempts
durable recording. If recording succeeds, exit 7 (uncertain). If recording
fails, exit 8 (audit_failed) takes precedence over the uncertain classification.

### J13A/J13B/J13C/J14 Boundary

- J13A: check command with engine and provider availability verification.
- J13B: typed host execution service extraction, run command.
- J13C: trail command with `--execution-id`.
- J14: actual provider capability call proof, complete integration matrix.

J13A implements no provider capability call, event evaluation, policy decision,
dispatch, Trail write, or replay write.

## 2026-07-29: Tether Source Line Endings Belong To The OCaml Parser

Decision: Tether source accepts LF, CRLF, and mixed LF/CRLF line endings at
the OCaml parser boundary. After splitting on LF, the parser removes only one
terminal CR that belongs to CRLF, then applies the existing blank-line test.

Reason: Structural parser lines must be compared without a CRLF artefact while
preserving the source language's indentation. The parser never globally trims
returned lines: leading spaces, action indentation, argument indentation, and
all other source characters remain significant. Rust hosts pass source through
unchanged and do not perform line-ending normalization.

## 2026-07-29: J13B Packet 2 Public Run Boundary

Decision: `tethers-reference-host run` has exactly five public options:
`--config`, `--engine`, `--input`, `--trail`, and `--host-data-root`. Its input
is exactly `{format_version,evaluation_id,tether,event,facts}`, where
`format_version` is `"1"`, `tether` is `{id,version}`, and `event` is
`{id,name,data}`. Duplicate keys and unknown fields fail. The supplied
evaluation ID and event ID are preserved exactly; they are never generated,
normalised, or replaced.

One invocation admits one external generation-zero event with host-owned
correlation equal to the event ID and no causation, durably records that
admission, and evaluates exactly one configured Tether selected by exact ID
and version. Source, capabilities, policy, scope, pins, approval, causal,
replay, and execution identity are host-owned and never accepted in public
input.

Ask reuses the exact approval-request seam and durable Trail record with a
process-local approval store. Public output contains only the evaluation ID,
Action ID, and redacted reason; it never exposes an approval ID or resume
route. Typed service results map only to the frozen CLI outcome vocabulary and
its matching exit codes, including distinct replay and unattempted outcomes.

## 2026-07-30: Canonical Git, Worktrees and Line Endings Guide

Decision: docs/GIT_WORKTREES_AND_LINE_ENDINGS_FOR_AGENTS.md is the canonical
operational guide for Git topology, worktrees, branch publication, line endings,
encoding investigation, and authorised recovery. It supports task packets and
does not replace their authority.

The guide adds no .gitattributes, editor, EOL, or Git-configuration policy.
Recovery tools remain available only under explicit task or recovery authority.
Unrelated dirty work belongs to its existing owner and must be preserved.

The canonical text was recovered unchanged from the earlier Work-mode branch
docs/git-worktrees-line-endings-guide at
3e958ceba22bbeed1937b1fa62fa3054fab1596b. The later Goose duplicate at
e63a90d0587c918a07dc2697db6c0f1dace77872 is not authority for this guide.
