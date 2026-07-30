# Decisions

## 2026-07-30: Canonical OCaml Engineering Guide


## 2026-07-30: Canonical Git, Worktrees and Line-Endings Guide

Decision: `docs/GIT_WORKTREES_AND_LINE_ENDINGS_FOR_AGENTS.md` is the canonical
operational guide for Tethers agents working with Git topology, branch
publication, worktrees, line-ending and encoding investigation, history
recovery, and destructive Git commands. `AGENTS.md` requires it before the
first Git mutation in those task categories.

The guide assists rather than replaces explicit task instructions. It
introduces no `.gitattributes`, `.editorconfig`, EOL policy, or global Git
configuration. Destructive and history-rewriting operations remain available
under explicit task or recovery authority. Unrelated dirty work must be
preserved; agents must not clean up another worktree.

Reason: Multiple worktrees, Windows/Unix text-file differences, and varied
line-ending tool behaviour have caused agent confusion. One practical
PowerShell-first reference avoids repeating the same diagnostic guidance in
every task packet.

## 2026-07-30: Canonical OCaml Engineering Guide

Decision: `docs/OCAML_GUIDE_FOR_AGENTS.md` is the canonical required operating
guide for every OCaml task. It records the Core/host boundary, deterministic
engineering rules, exact worktree-safe `OcamlSwitchPath` contract, verification
expectations, and stop conditions. `AGENTS.md` requires it before the first
OCaml edit.

Reason: The former compact guide no longer carried enough authoritative detail
for the deterministic Core and multi-worktree native-Windows workflow. One
canonical guide prevents competing OCaml instructions.

`TOOLCHAIN-BASELINE-01` is approved but not implemented by this documentation
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
