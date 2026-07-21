# Capability Bridge - M7 Design

Status: design
Date: 2026-07-21
Purpose: safe, generic bridge between discovered MCP tools and trusted Tethers capabilities

This is the universal plug contract. It must work later for MCP, Git, Google,
Obsidian, Lantern Keeper, and other providers without making Tethers itself
understand those applications.

It is a design document. No foreign MCP tool invocation, no automatic execution,
no provider integration, no networking or credential management, no Tethers
syntax or semantic changes, no new dependency, and no production implementation
are part of M7.

---

## 1. Scope and non-goals

### Scope

- Define the trust model between **discovered MCP tools** (untrusted advertisements) and **Tethers capabilities** (trusted, reviewed, installed manifests).
- Define the manifest format that makes an MCP tool a candidate Tethers capability.
- Define discovery, approval, schema-drift, confirmation, timeout, retry, and planning-to-execution handoff rules.
- Define execution Trail additions needed for bridged capability calls.
- Provide worked examples and explicit rejected cases.

### Non-goals (M7)

- No foreign MCP tool invocation.
- No automatic execution of any Action.
- No Google, Git, Obsidian, or Lantern Keeper integration.
- No networking, authentication, or credential management.
- No Tethers syntax or semantic changes.
- No production implementation.
- No new dependency.

---

## 2. Layered architecture

Five layers, separated by explicit trust boundaries:

```
+------------------------------------------+
|  Discovered MCP tool                      |  UNTRUSTED
|  (name, description, inputSchema,         |
|   annotations - all claims, no authority) |
+--------------+---------------------------+
               | compare, review, approve
               v
+------------------------------------------+
|  Trusted Capability Manifest              |  TRUSTED (after review)
|  (canonical name, version, schemas,       |
|   effects, scope, reversibility,          |
|   determinism, idempotency mechanism,     |
|   confirmation, timeout, retry,           |
|   provider identity, binding, digest)     |
+--------------+---------------------------+
               | host projects approved planning fields
               v
+------------------------------------------+
|  Tethers Planner (OCaml)                 |  DETERMINISTIC
|  (evaluates Conditions using approved     |
|   capability projection, copies opaque    |
|   manifest digest into Actions, never     |
|   executes)                               |
+--------------+---------------------------+
               | Plan with manifest digest
               v
+------------------------------------------+
|  Permissioned Host (Rust or other)        |  TRUSTED
|  (resolves exact manifest by digest,      |
|   re-validates arguments + scope,         |
|   obtains confirmation if required,       |
|   dispatches bound MCP call,              |
|   validates result, appends Trail)         |
+--------------+---------------------------+
               | records
               v
+------------------------------------------+
|  Execution Trail                          |  IMMUTABLE RECORD
|  (capability, version, digest, provider,  |
|   execution/attempt IDs, permission,      |
|   confirmation, dispatch, result, status) |
+------------------------------------------+
```

### Trust claims at each boundary

| Boundary | Trusted claim | Untrusted / must be re-verified |
|---|---|---|
| MCP tool discovery -> manifest author | Nothing. All tool metadata is untrusted. | Tool name, description, schemas, annotations. |
| Manifest -> planner | The trusted host supplies an approved capability projection containing the planning-relevant fields and opaque digest. | The planner does not inspect or trust the complete manifest. |
| Planner -> Plan Action | Action references capability name, version, and digest. | Host must still resolve the exact manifest and re-validate. |
| Plan Action -> host | Nothing. Plan is a request, not permission. | Argument values, scope conformance, confirmation status. |
| Host -> remote MCP call | Nothing. Remote server is untrusted at call time. | MCP tool input validity, output conformance, side effects. |

---

## 3. Generic capability identity

### Identity

A capability is identified by **name + version**:

```text
notes.note.create@1
obsidian.vault.search@2
lantern.memory.store@1
```

- **Capability name**: a dotted path describing a meaningful operation.
  Must not be a raw MCP tool name (e.g., `obsidian_note_create` is a tool name;
  `notes.note.create` is a capability name). Names describe *what* the operation
  does, not *which transport* carries it.
- **Capability version**: a positive integer (`1`, `2`, ...). Monotonically
  increasing. Breaking input/output schema changes require a new version.
- **Identity**: the pair `(name, version)`. Two manifests with the same name
  but different versions are distinct capabilities sharing a lineage.

### Version distinctions

| Version kind | Meaning | Example |
|---|---|---|
| Capability version | Breaking contract revision for one capability | `notes.note.create@2` |
| Manifest format version | Schema version of this manifest document | `"1.0"` |
| MCP protocol version | MCP specification revision spoken by the server | `"2025-11-25"` |

These are independent. A manifest format upgrade does not force a capability
version change. An MCP protocol upgrade does not force either.

### Plan pinning

A proposed Action in a Plan identifies the exact approved contract:

```json
{
  "action_id": "action_1",
  "idempotency_key": "eval_001/action_1",
  "capability": "notes.note.create",
  "capability_version": 1,
  "manifest_digest": "sha256:abc123def456...",
  "arguments": { "title": "Meeting notes", "content": "..." }
}
```

The host resolves the manifest by digest and proves the current provider/tool
binding still matches that pinned contract. If the digest does not match any
currently installed manifest, or if the binding proof fails, execution is
denied. A Plan cannot silently execute against a changed or removed manifest or
provider.

---

## 4. Trusted manifest format

### Canonical structure

A manifest is a JSON document stored in the host's trusted manifest store.
All top-level fields except the digest value itself and exact display metadata
(`title`, `description`) are authoritative for execution and covered by the
contract digest. Display metadata must not affect execution behaviour.

```jsonc
{
  // -- Manifest metadata --
  "manifest_format_version": "1.0",
  "capability_name": "notes.note.create",
  "capability_version": 1,
  "title": "Create a project note",
  "description": "Create a new Markdown note in the project vault with optional frontmatter.",

  // -- Typed contract --
  "input_schema": {
    "type": "object",
    "properties": {
      "title": { "type": "string", "description": "Note title" },
      "content": { "type": "string", "description": "Markdown body" },
      "tags": {
        "type": "array",
        "items": { "type": "string" },
        "description": "Optional tags"
      }
    },
    "required": ["title", "content"],
    "additionalProperties": false
  },
  "output_schema": {
    "type": "object",
    "properties": {
      "path": { "type": "string", "description": "Created file path" },
      "modified": { "type": "boolean", "description": "Whether the file was newly created" }
    },
    "required": ["path", "modified"]
  },

  // -- Security declarations --
  "effects": ["filesystem.write"],
  "permission_scope": {
    "kind": "path_prefix",
    "allowed_prefixes": ["projects/", "daily/"]
  },

  // -- Behaviour declarations --
  "reversibility": "compensatable",
  "determinism": "deterministic",
  "idempotency": {
    "mechanism": "argument_key",
    "argument_name": "idempotency_key",
    "description": "Server deduplicates by idempotency_key argument; repeated calls with the same key produce at most one note."
  },
  "confirmation_policy": {
    "standing_permitted": false,
    "per_call_required": true,
    "description": "Creating notes in project vaults requires per-call confirmation."
  },

  // -- Execution policy --
  "timeout_ms": 10000,
  "retry_policy": {
    "max_retries": 3,
    "backoff_ms": 1000,
    "allowed_on": ["outcome_unknown"],
    "requires_idempotency_proof": true
  },

  // -- Provider identity --
  "provider": {
    "identity": "obsidian-local",
    "display_name": "Obsidian (local vault)",
    "identity_source": "host_configuration",
    "description": "Host-assigned identity; the local Obsidian MCP server provides no globally trustworthy identity. The host pins this provider to a specific configured server instance."
  },

  // -- Binding --
  "binding": {
    "kind": "mcp",
    "server_name": "obsidian",
    "tool_name": "obsidian_create_note",
    "adapter": null
  },

  // -- Contract digest --
  "digest": "sha256:..."
}
```

### Digest computation

The digest algorithm is fixed to SHA-256. The manifest does not include a
`digest_algorithm` field. Algorithm agility is deliberately deferred until a
real need exists; adding it later would be a new manifest-format decision.

The digest is computed over an RFC 8785 JSON Canonicalization Scheme (JCS)
canonical representation of the manifest after removing only the digest value
and exact top-level display metadata. The following fields are **included** in
the canonical form:

1. `manifest_format_version`
2. `capability_name`
3. `capability_version`
4. `input_schema` (complete object, including all nested keys)
5. `output_schema` (complete object, including all nested keys)
6. `effects`
7. `permission_scope`
8. `reversibility`
9. `determinism`
10. `idempotency`
11. `confirmation_policy`
12. `timeout_ms`
13. `retry_policy`
14. `provider` (all sub-fields)
15. `binding` (all sub-fields)

The following fields are **excluded** from the canonical form:

- `digest` (self-referential)
- top-level `title`
- top-level `description`

No nested schema or policy fields are excluded. In particular, `input_schema`
and `output_schema` are digested completely, including nested `description`
keys, annotations, examples, defaults, constraints, and every object beneath
them. This avoids treating any nested schema text as non-authoritative by
accident.

**RFC 8785/JCS requirements:**

1. The manifest must be valid I-JSON before canonicalization.
2. Duplicate object keys are rejected, recursively, before any semantic
   validation or digest computation.
3. String data is preserved as Unicode; canonicalization must not normalize,
   rewrite, or escape it except as required by JCS serialization.
4. Numbers must be representable within the IEEE-754-compatible JSON number
   constraints required by JCS/I-JSON. Values outside that range are invalid
   manifest input.
5. Object properties are sorted recursively according to RFC 8785 ordering by
   UTF-16 code units, not by host-language string ordering if that differs.
6. Primitive serialization must be ECMAScript-compatible as required by JCS.
7. The canonical byte output is UTF-8 with no extra whitespace.
8. Compute SHA-256 over those canonical bytes.
9. The digest string is `"sha256:"` followed by the lowercase hex encoding.

C1b1 must first verify a maintained Rust JCS implementation against official
RFC 8785 examples and test vectors. Do not authorise a casual homemade
canonicalizer or fallback. If no suitable implementation is verified, stop for a
separate design decision before implementing manifest digests.

**C1b1 result (2026-07-21):** Columbo selects
`serde_json_canonicalizer` 0.3.x for C1b2 canonical byte generation, subject to
pinning an exact compatible crate version during the implementation task. The
reviewed version was 0.3.2. Evidence: its current crate metadata identifies it
as an RFC 8785/JCS implementation with MIT licensing; its source uses
`ryu-js` for ECMAScript-compatible number serialization and sorts object keys by
decoded UTF-16 code units; its test suite includes RFC-text cases and the
cyberphone `json-canonicalization` reference corpus. A disposable local
experiment against RFC sample output, recursive object sorting, and non-BMP
UTF-16 key ordering passed.

Division of responsibility:

- C1a2 remains responsible for strict JSON parsing, recursive duplicate-key
  rejection, trailing-token rejection, and authoritative unknown-field
  rejection before canonicalization.
- C1b2 must pass only the already strict-parsed and digest-filtered
  `serde_json::Value` to `serde_json_canonicalizer::to_vec`.
- Columbo must enforce manifest I-JSON and number-domain constraints before
  canonicalization where `serde_json` or the canonicalizer do not reject them
  explicitly.
- C1b2 must add project golden vectors proving all digest-covered manifest
  fields affect the canonical bytes and digest.

Rejected C1b1 alternatives:

- `serde_jcs` 0.2.0: API shape is usable and the crate is active enough to
  compile on the current Rust toolchain, but `serde_json_canonicalizer` was
  created specifically because `serde_jcs` had open RFC-compatibility concerns;
  prefer the crate that documents and tests the intended RFC conformance more
  directly.
- `json-canon` 0.1.3: RFC 8785 API is plausible, but it documents problematic
  handling for NaN/Infinity on struct input and has older dependencies.
- `canon-json` 0.2.1: useful formatter implementation, but it delegates more
  responsibility to caller-side serializer setup and is less direct than a
  `to_vec(&serde_json::Value)` API for Columbo's strict-parsed value pipeline.
- `jcs-canonicalize` 0.2.1: includes conformance tests, but it wraps
  canonicalization with a CLI and SHA-256 helper. Columbo C1b2 should keep
  canonicalization and digest calculation explicit rather than adopting a crate
  that bundles hashing into its public story.

### Strict JSON parsing

Manifest parsing must reject duplicate keys in every object recursively,
including arbitrary nested objects inside `input_schema` and `output_schema`.
This is an observable requirement, independent of implementation mechanism.
Do not claim that `serde_json::StreamDeserializer` or any other parser provides
recursive duplicate-key rejection automatically until that behaviour has been
verified. The C1a2 implementation task must choose and prove the mechanism.

A host verifying a digest recomputes it from the stored manifest's
authoritative fields and compares it with the `digest` value. A mismatch means
the manifest has been altered or corrupted.

### Distinction from the runtime capability schema

The existing Tethers 0.1 runtime capability schema (supplied per
evaluation request in the `capabilities` array) remains unchanged:

```json
{
  "name": "lantern.task.record",
  "version": "1.0.0",
  "inputs": { "project": "string", "task": "string" },
  "effects": ["lantern.write"],
  "reversibility": "compensatable"
}
```

The runtime schema continues to serve its existing role: telling the planner
what input types, effects, and reversibility a capability declares so the
planner can validate Action arguments and report `required_effects`.

The **trusted manifest** is a separate, richer, host-installed artifact. The
trusted host constructs the capability registry/view supplied as deterministic
evaluator input. For each approved bridge-backed capability, that input includes
the planning-relevant capability projection plus the exact `manifest_digest` as
opaque contract identity.

The planner does not inspect or trust the manifest. It receives an approved
capability projection containing an opaque manifest digest and copies that digest
into each proposed bridge Action. A missing digest means a bridge-backed
capability Action is invalid and cannot execute.

Compatibility boundary:

- Existing 0.1 capability requests and Plans remain unchanged.
- Bridge-backed capability planning requires the future additive
  capability-input and Action/Plan fields described here.
- M7 specifies that future extension but does not implement it.
- Until those fields exist, Tethers cannot produce executable bridge-backed
  Plans with digest pinning.

---

## 5. Tool discovery and approval

### Lifecycle

```
MCP server starts
        |
        v
Host discovers tools via tools/list
        |
        v
Host records discovered tool metadata
  (name, description, inputSchema, annotations)
  ALL UNTRUSTED
        |
        v
Human or authorised process reviews
  discovered claims against:
  - actual tool behaviour
  - security implications
  - required scope
        |
        v
Trusted manifest is authored, reviewed, and installed
  with pinned contract digest
        |
        v
Host produces an approved capability projection
  (planning fields plus opaque manifest_digest)
        |
        v
Projection is supplied as deterministic planner input
        |
        v
Planner may propose Actions referencing this capability
  and copying the same manifest_digest
        |
        v
Host resolves exactly that digest before execution
```

### Discovery rules

- MCP tool descriptions, annotations, and schemas are **advertising claims**.
  They carry zero authority.
- MCP annotations such as `readOnlyHint`, `destructiveHint`, `idempotentHint`
  may assist human review but **must never** override or auto-populate manifest
  fields.
- Discovery never grants permission, authority, or planner visibility.
- A tool with no installed trusted manifest is invisible to the planner.
- Auto-generation of manifests from discovery is forbidden. Every manifest must
  be explicitly authored or reviewed by a trusted party.

---

## 6. Schema and identity drift

### Contract digest coverage

The contract digest covers every field listed in section 4 Digest Computation. If any
of those fields changes, the digest changes and the manifest is considered a
different contract.

### Drift lifecycle

```
MCP server emits notifications/tools/list_changed
        |
        v
Host re-discovers tools
        |
        v
Host compares each discovered tool's current schema
  with the installed manifest's pinned fields
        |
        v
Match: no action; capability remains available
Mismatch: capability becomes UNAVAILABLE
  - planner cannot propose new Actions for this capability
  - no existing Plan may dispatch an undispatched Action through
    the changed provider
        |
        v
Changed contract requires explicit human re-review
  - no automatic reapproval
  - new review produces a new manifest with new digest
  - old Plans must be re-evaluated against the newly approved
    capability projection
```

A currently installed old manifest is not, by itself, enough to execute an old
Plan. Before every dispatch, the host must prove that the currently bound
provider/tool still matches the exact contract and provider binding pinned by
the Action's `manifest_digest`. If current discovery or trusted binding state
differs, the capability becomes unavailable immediately: no new Plan may use it,
and no existing Plan may dispatch an undispatched Action through that changed
provider.

An old Plan may execute only if the host has an immutable old provider binding,
versioned adapter, isolated endpoint, or equivalent host-verifiable proof that
the exact old contract remains available. Retaining an old manifest document
alone is not proof.

### Time-of-check/time-of-use prevention

- A Plan records the `manifest_digest` supplied in deterministic evaluator input.
- The host resolves the manifest by digest at execution time.
- If the digest does not match any currently installed manifest, execution is
  denied with `manifest_not_found`.
- A Plan created under one manifest version cannot silently execute under a
  different version.
- If a manifest is revoked (removed from the store), all Plans referencing its
  digest become unexecutable.
- If the digest can be found but current discovery or trusted binding state no
  longer proves the pinned contract and provider binding, dispatch is denied
  with `manifest_binding_mismatch`.
- Re-review and installation of a new manifest creates a new digest; it never
  silently repairs or upgrades an old Plan. The old Plan must be re-evaluated
  against the newly approved capability projection.

### Multi-action Plan invalidation

If drift or revocation is detected during execution, completed Actions remain
recorded. Any currently dispatched Action follows the normal
`completed`/`failed`/`outcome_unknown` rules. Every undispatched Action using the
invalidated manifest must be denied. The host must not continue dispatching
merely because the Plan was previously authorised.

---

## 7. Typed inputs and outputs

### Pre-dispatch validation

Before dispatching an MCP tool call, the host must:

1. Validate every Action argument against the manifest's `input_schema`.
2. Reject unknown arguments (the schema's `additionalProperties` governs this).
3. Reject missing required arguments.
4. Reject type mismatches.
5. Construct the MCP `tools/call` arguments from the validated Action arguments.

This is a re-validation: the planner already validated against the runtime
schema, but the host validates against the full manifest schema at execution
time.

### Post-dispatch result validation

After receiving an MCP tool result, the host must:

1. If the MCP call returned an error (MCP `isError: true`), record it as a
   provider error - not a schema-validation failure.
2. If the MCP call succeeded (`isError: false`), extract `structuredContent`
   if present, or the first text content item if not.
3. If the manifest declares an `output_schema`, validate the extracted result
   against it.
4. If `output_schema` is present and the result fails validation, record
   `action_failed` with `result_validation_failed`.
5. If the manifest declares no `output_schema` (or the MCP tool provides none
   usable), the tool **requires a separately reviewed typed adapter** before it
   can become a trusted capability. Unstructured text must not silently become
   trusted structured data.

Manifest review must reject effectively unconstrained `output_schema` values.
Examples include empty schemas, schemas that allow all JSON values, or schemas
whose constraints are too weak to make the provider result trustworthy for
later host use. This does not mean every output must be an object with
properties. Concrete primitive schemas, arrays with constrained items, enums,
and structured-object schemas may all be valid when they precisely describe the
provider result. Unstructured provider output still requires a reviewed typed
adapter.

### Error classification at execution

| Failure | Trail outcome |
|---|---|
| Argument validation failure | `action_failed` with `argument_validation_failed` |
| MCP transport/protocol error | `action_failed` with `provider_error` |
| MCP tool error (`isError: true`) | `action_failed` with `provider_error` |
| Result validation failure | `action_failed` with `result_validation_failed` |
| Timeout before dispatch | `action_failed` with `timeout` |
| Timeout after dispatch | `outcome_unknown` (see section 11) |
| Unexpected result shape (no usable schema) | Rejected at manifest review time; tool requires typed adapter |

---

## 8. Effects and permission scope

### Effects

Effects are explicit security-relevant string declarations. They use a
namespace convention:

```text
filesystem.read
filesystem.write
network.access
notes.read
notes.write
calendar.read
calendar.write
email.send
git.read
git.write
lantern.write
```

The host's permission policy checks `required_effects` from the Plan against
its configured allow-list. This is the existing 0.1 effect-authorisation model
and remains unchanged.

### Permission scope

Permission scope is an enforceable structured constraint, distinct from
effects. While effects declare *what kind* of operation is occurring (e.g.,
`filesystem.write`), scope declares *where* or *on what resources*.

Scope is declared in the manifest and enforced by the host at execution time:

```json
{
  "permission_scope": {
    "kind": "path_prefix",
    "allowed_prefixes": ["projects/", "daily/"]
  }
}
```

```json
{
  "permission_scope": {
    "kind": "repository",
    "allowed_repositories": ["org/repo-name"]
  }
}
```

```json
{
  "permission_scope": {
    "kind": "calendar",
    "allowed_calendars": ["work", "personal"]
  }
}
```

### Scope enforcement

At execution time, the host must:

1. Extract the relevant argument values from the Action (e.g., `path`, `repository`, `calendar`).
2. Check them against the manifest's `permission_scope`.
3. If the Action's arguments fall outside the allowed scope, deny execution
   with `scope_violation`.

### Scope and discovery

- A discovered MCP tool cannot broaden its own scope.
- Scope is declared in the trusted manifest only.
- If a capability's scope cannot be represented in the structured scope
  language, the manifest must either:
  - set `permission_scope` to `null` (meaning "no scope restriction; per-call
    confirmation is mandatory"), or
  - defer to a provider-specific scope validator (future extension point).

### Provider-specific scope validators

The generic envelope supports `kind`-specific scope definitions. Each `kind`
implies a validation function that the host must implement for that provider
class. The design does not specify every future provider's scope language; it
defines the envelope and requires that each binding kind document its scope
validation rules.

---

## 9. Confirmation

### Confirmation policy in the manifest

The manifest declares whether standing approval is **permitted** and whether
per-call confirmation is **always required**:

```json
{
  "confirmation_policy": {
    "standing_permitted": false,
    "per_call_required": true,
    "description": "Creating notes requires per-call confirmation."
  }
}
```

```json
{
  "confirmation_policy": {
    "standing_permitted": true,
    "per_call_required": false,
    "description": "Reading notes may receive standing approval after initial review."
  }
}
```

```json
{
  "confirmation_policy": {
    "standing_permitted": false,
    "per_call_required": false,
    "description": "Purely read-only, safe for automatic dispatch without confirmation."
  }
}
```

### Standing approval is separate host-controlled state

**A manifest must not grant its own standing authority.** The
`confirmation_policy` only declares what the manifest author believes is
acceptable. Actual standing approval is a separate host-controlled record:

```jsonc
{
  "standing_approval": {
    "manifest_digest": "sha256:abc123...",
    "approved_scope": { "kind": "path_prefix", "allowed_prefixes": ["projects/"] },
    "approved_by": "matthew",
    "created_at": "2026-07-21T10:00:00Z",
    "revoked_at": null,
    "reason": "Daily note creation in project vaults is low-risk."
  }
}
```

A standing-approval record is bound to:
- the **exact manifest digest** (a changed manifest voids the approval);
- the **approved scope** (which may be narrower than the manifest's declared scope);
- the **approving identity**;
- creation and optional revocation timestamps.

If a standing approval exists for a manifest digest and the Action's arguments
fall within the approved scope, the host may skip per-call confirmation.
Otherwise, and always when `per_call_required` is `true`, the host must obtain
confirmation before dispatch.

### Confirmation contents

When confirmation is required, the host must present:

- the capability name and version;
- the resolved Action arguments;
- the declared effects;
- the permission scope;
- the manifest digest;
- whether standing approval exists but the current call falls outside its scope.

### Denial and cancellation

- Confirmation denied -> `action_failed` with `confirmation_denied`.
- Confirmation cancelled (e.g., timeout waiting for user) -> `action_failed` with `confirmation_cancelled`.
- Both produce explicit Trail entries.

---

## 10. Determinism, idempotence, and reversibility

These are three separate properties. Do not use one as a substitute for another.

### Determinism

| Value | Meaning |
|---|---|
| `"deterministic"` | Same inputs produce the same outputs. Safe to cache, safe to retry for read-only. |
| `"non_deterministic"` | Outputs may vary. Host must not cache results or assume repeatability. |

Declared in the manifest as:

```json
{ "determinism": "deterministic" }
```

### Idempotency

Idempotency describes whether repeating the same request avoids duplicate
side effects. The manifest must specify a **concrete mechanism**, not merely a
label.

#### Mechanism: `argument_key`

The capability accepts a dedicated idempotency-key argument. The host supplies
a stable key derived from the evaluation and action identity. The provider
(MCP server) deduplicates by this key.

```json
{
  "idempotency": {
    "mechanism": "argument_key",
    "argument_name": "idempotency_key",
    "key_source": "evaluation_id/action_id",
    "description": "The host passes evaluation_id/action_id as the idempotency_key argument. The server guarantees at-most-once semantics for repeated calls with the same key."
  }
}
```

#### Mechanism: `server_dedup`

The capability's provider guarantees idempotency internally without requiring
a client-supplied key. A manifest description alone is not proof. The host must
have trusted host/provider/adapter evidence describing the deduplication key,
scope, and lifetime, and that evidence must be pinned by the manifest binding.

```json
{
  "idempotency": {
    "mechanism": "server_dedup",
    "dedup_key": "provider request id derived from stable message-id",
    "dedup_scope": "provider account and target collection",
    "dedup_lifetime": "at least 24 hours",
    "evidence": "adapter contract review obsidian-local@2026-07-21",
    "description": "The reviewed adapter proves provider-side deduplication for the declared key, scope, and lifetime."
  }
}
```

#### Mechanism: `none`

The capability provides no idempotency guarantee.

```json
{
  "idempotency": {
    "mechanism": "none"
  }
}
```

### Idempotency and retry

- Effectful automatic retries require a concrete idempotency mechanism.
  `retry_policy.requires_idempotency_proof: false` cannot bypass or skip this
  rule.
- If `mechanism` is `"none"`, automatic retry is **forbidden** for writes or
  any other effectful Action. The host may only retry read-only, deterministic,
  idempotency-safe calls.
- If `mechanism` is `"argument_key"`, the host must supply the key before
  dispatch and may retry on `outcome_unknown` (see section 11).
- If `mechanism` is `"server_dedup"`, the host may retry on `outcome_unknown`
  only when the trusted manifest binding pins evidence for the deduplication
  key, scope, and lifetime.
- The word `"conditional"` alone is insufficient. A manifest must specify the
  exact mechanism and, for `argument_key`, the argument name and key source.
  Without a concrete reviewed mechanism, automatic retry remains forbidden.

### Reversibility

| Value | Meaning |
|---|---|
| `"reversible"` | The host can reliably restore the previous state. |
| `"compensatable"` | Another action may counteract the effect (e.g., delete the created note). |
| `"irreversible"` | No meaningful automatic reversal exists (e.g., sent email). |

Reversibility is declared in the manifest and reported by the planner (existing
0.1 behaviour). It informs host policy about confirmation requirements and undo
presentation but does not, by itself, enable automatic rollback.

---

## 11. Timeout, retries, and unknown outcomes

### Timeout

The manifest declares a `timeout_ms`:

```json
{ "timeout_ms": 10000 }
```

Two distinct timeout boundaries exist:

1. **Pre-dispatch timeout**: the host's own deadline for validating arguments,
   obtaining confirmation, and preparing the MCP call. If exceeded, the Action
   is `action_failed` with `timeout` and the remote server is never contacted.

2. **Post-dispatch timeout**: the deadline for the remote MCP server to respond
   after the call has been sent. If exceeded, the outcome is **`outcome_unknown`**
   - the remote server may have completed the operation, may still be processing,
   or may never have received it.

### Outcome unknown

`outcome_unknown` is a distinct execution status. It is not `action_failed`.
It means: "We do not know whether the operation succeeded or failed."

The Trail must record:

```json
{
  "sequence": 8,
  "phase": "execution",
  "kind": "action_dispatched",
  "outcome": "outcome_unknown",
  "message": "MCP call to notes.note.create timed out after dispatch (10000ms); outcome unknown",
  "timestamp": "2026-07-21T10:30:00Z"
}
```

### Retry rules

Conservative defaults:

1. **No automatic retry for writes or other effectful Actions** unless the
   manifest's `idempotency.mechanism` is `"argument_key"` or `"server_dedup"`
   with reviewed, concrete proof of safety.
2. Retry is **only** permitted for `outcome_unknown`. A confirmed `action_failed`
   must not be automatically retried.
3. Each retry attempt uses a stable `execution_id` with an incrementing
   `attempt_id` (e.g., `exec_001/attempt_1`, `exec_001/attempt_2`).
4. All attempts are recorded in the Trail.
5. The host must respect `retry_policy.max_retries` and `retry_policy.backoff_ms`.
6. If the manifest's idempotency mechanism is `"none"`, retry remains
   forbidden for effectful Actions regardless of `max_retries` or
   `requires_idempotency_proof`.

```json
{
  "retry_policy": {
    "max_retries": 3,
    "backoff_ms": 1000,
    "allowed_on": ["outcome_unknown"],
    "requires_idempotency_proof": true
  }
}
```

---

## 12. Planning-to-execution handoff

The complete conceptual sequence:

1. **The trusted host constructs deterministic evaluator input** from approved
   manifests. Existing 0.1 capabilities use the unchanged runtime schema shape.
   Bridge-backed capabilities use the future additive capability projection:
   planning-relevant fields plus opaque `manifest_digest`.
2. **Tethers evaluates** the Tether, Conditions, and Action arguments against
   the capability projections supplied by the host. The planner does not inspect
   or trust complete manifests.
3. **A Plan is produced** containing proposed Actions. Each bridge Action
   references the capability name, capability version, and the **manifest
   digest** that was supplied in the deterministic evaluator input.
4. **The host receives the Plan.** It resolves each Action's manifest by digest
   from the trusted manifest store and proves the currently bound provider/tool
   still matches the pinned contract and provider binding.
5. **The host validates** each Action's arguments against the full manifest
   `input_schema`.
6. **The host checks scope** by evaluating the Action's arguments against the
   manifest's `permission_scope`.
7. **The host checks confirmation policy.** If `per_call_required` is `true`
   or no standing approval covers this call, the host obtains explicit
   confirmation.
8. **The host dispatches** the bound MCP call using the manifest's `binding`
   fields.
9. **The host awaits the result** within `timeout_ms`.
10. **The host validates the result** against the manifest's `output_schema`
   (if present).
11. **The host appends execution Trail entries** - at minimum:
    - `plan_authorised` / `plan_denied`
    - `action_started`
    - `action_dispatched`
    - `action_completed` / `action_failed` / `outcome_unknown`
    - Each with capability, version, digest, provider identity, execution/attempt IDs.

The original deterministic planning Trail (reception, evaluation phases) is
preserved unchanged. The host appends authorisation and execution phases.

---

## 13. Execution Trail

### New execution-stage information

The execution Trail entries must record, where applicable:

| Field | Description | Required |
|---|---|---|
| `capability_name` | Resolved capability name | Always |
| `capability_version` | Resolved capability version | Always |
| `manifest_digest` | Digest of the manifest used for execution | Always |
| `provider_identity` | Host-assigned provider identity from manifest | Always |
| `execution_id` | Stable identifier for this execution (one per Action) | Always |
| `attempt_id` | Incrementing attempt within one execution | Always |
| `permission_decision` | `authorised` or `denied` | Authorisation phase |
| `confirmation_decision` | `confirmed`, `denied`, `cancelled`, or `standing` | When confirmation policy applies |
| `dispatch_state` | `dispatched`, `not_dispatched` | Execution phase |
| `result_validation` | `passed`, `failed`, `skipped` (no output schema) | On completion |
| `status` | `completed`, `failed`, `denied`, `outcome_unknown` | Always |
| `timestamp` | Host wall-clock time (not in planner output) | Execution phase |

For stale-Plan or schema-drift denial, the Trail must record that the Action was
not dispatched, including the pinned `manifest_digest`, the current provider
identity if known, and the denial reason (`manifest_not_found`,
`manifest_revoked`, or `manifest_binding_mismatch`). For multi-action Plans, the
Trail preserves completed Actions and records denied entries for every
undispatched Action using the invalidated manifest.

### Redaction rules

- Do not record credentials, tokens, API keys, or secrets.
- Do not record full request/response payloads unless explicitly configured
  for debugging and scoped to a non-production environment.
- Record argument values only to the extent needed for audit (capability,
  scope, and intent). The host's logging policy controls this.

### Credential handling

Tether source and Plans never contain, request, or supply credential values.
Manifest schemas may describe credential-shaped inputs when the provider tool
expects such an argument, but the actual credential value is injected only by
Columbo from trusted host credential storage at dispatch. The injected value is
never part of deterministic planner input, never appears in the Plan, and must
not be recorded in the Trail.

Secret-like-value scanning of Tether source, manifests, Plans, and Trails is a
defence-in-depth check. It does not replace the architectural rule that
credentials live outside declarative artifacts. When a secret-like literal is
found in a Tether or Plan, the rejection must tell the author to remove the
value and rely on host credential injection. It must not suggest renaming,
re-encoding, obfuscating, or otherwise disguising the value.

### Example execution Trail entry

```json
{
  "sequence": 8,
  "phase": "execution",
  "kind": "action_completed",
  "outcome": "completed",
  "message": "notes.note.create completed: created 'Meeting notes' at projects/lantern/meeting-notes.md",
  "capability_name": "notes.note.create",
  "capability_version": 1,
  "manifest_digest": "sha256:abc123...",
  "provider_identity": "obsidian-local",
  "execution_id": "exec_001",
  "attempt_id": "exec_001/attempt_1",
  "timestamp": "2026-07-21T10:30:01Z"
}
```

---

## 14. Worked examples

### Example 1: Read-only capability - `obsidian.note.read`

#### Discovered MCP tool (untrusted)

```json
{
  "name": "obsidian_read_note",
  "description": "Read a note from the Obsidian vault",
  "inputSchema": {
    "type": "object",
    "properties": {
      "path": { "type": "string", "description": "Path to the note" }
    },
    "required": ["path"]
  },
  "annotations": {
    "readOnlyHint": true
  }
}
```

#### Trusted manifest (after review)

```json
{
  "manifest_format_version": "1.0",
  "capability_name": "obsidian.note.read",
  "capability_version": 1,
  "title": "Read an Obsidian note",
  "description": "Read the Markdown content of a note in the Obsidian vault.",
  "input_schema": {
    "type": "object",
    "properties": {
      "path": { "type": "string" }
    },
    "required": ["path"],
    "additionalProperties": false
  },
  "output_schema": {
    "type": "object",
    "properties": {
      "content": { "type": "string" },
      "frontmatter": { "type": "object" }
    },
    "required": ["content"]
  },
  "effects": ["filesystem.read"],
  "permission_scope": {
    "kind": "path_prefix",
    "allowed_prefixes": ["projects/", "daily/", "archive/"]
  },
  "reversibility": "reversible",
  "determinism": "deterministic",
  "idempotency": {
    "mechanism": "none",
    "description": "Read-only; no side effects to deduplicate."
  },
  "confirmation_policy": {
    "standing_permitted": true,
    "per_call_required": false
  },
  "timeout_ms": 5000,
  "retry_policy": {
    "max_retries": 3,
    "backoff_ms": 500,
    "allowed_on": ["outcome_unknown"],
    "requires_idempotency_proof": false
  },
  "provider": {
    "identity": "obsidian-local",
    "display_name": "Obsidian (local vault)",
    "identity_source": "host_configuration",
    "description": "Host-assigned identity for the local Obsidian MCP server."
  },
  "binding": {
    "kind": "mcp",
    "server_name": "obsidian",
    "tool_name": "obsidian_read_note",
    "adapter": null
  },
  "digest": "sha256:1a2b3c4d..."
}
```

#### Approved capability projection (planner input)

```json
{
  "name": "obsidian.note.read",
  "version": 1,
  "inputs": { "path": "string" },
  "effects": ["filesystem.read"],
  "reversibility": "reversible",
  "manifest_digest": "sha256:1a2b3c4d..."
}
```

#### Proposed Tethers Action (in Plan)

```json
{
  "action_id": "action_1",
  "idempotency_key": "eval_001/action_1",
  "capability": "obsidian.note.read",
  "capability_version": 1,
  "manifest_digest": "sha256:1a2b3c4d...",
  "arguments": {
    "path": "projects/lantern/architecture.md"
  },
  "effects": ["filesystem.read"]
}
```

#### Host checks

1. Resolve manifest by digest `sha256:1a2b3c4d...` -> found.
2. Prove the current `obsidian_read_note` provider binding still matches the
   pinned contract -> passes.
3. Validate `path` against `input_schema` -> passes.
4. Check scope: `projects/lantern/architecture.md` has prefix `projects/` -> within scope.
5. Confirmation: `per_call_required` is `false`; standing approval exists for
   this digest and scope -> skip confirmation.
6. Dispatch `tools/call` with `obsidian_read_note` and `{"path": "projects/lantern/architecture.md"}`.
7. Result: `structuredContent` with `content` and `frontmatter` -> validate against `output_schema` -> passes.

#### Trail outcome

```json
{ "sequence": 6, "phase": "authorisation", "kind": "plan_authorised", "outcome": "authorised", "capability_name": "obsidian.note.read", "capability_version": 1, "manifest_digest": "sha256:1a2b3c4d...", "provider_identity": "obsidian-local", "execution_id": "exec_001", "attempt_id": "exec_001/attempt_1", "timestamp": "2026-07-21T10:30:00Z" },
{ "sequence": 7, "phase": "execution", "kind": "action_started", "outcome": "started", "execution_id": "exec_001", "attempt_id": "exec_001/attempt_1", "timestamp": "2026-07-21T10:30:00Z" },
{ "sequence": 8, "phase": "execution", "kind": "action_completed", "outcome": "completed", "capability_name": "obsidian.note.read", "capability_version": 1, "manifest_digest": "sha256:1a2b3c4d...", "provider_identity": "obsidian-local", "execution_id": "exec_001", "attempt_id": "exec_001/attempt_1", "timestamp": "2026-07-21T10:30:01Z" }
```

---

### Example 2: Scoped writing capability - `notes.note.create`

#### Discovered MCP tool (untrusted)

```json
{
  "name": "obsidian_create_note",
  "description": "Create a new note",
  "inputSchema": {
    "type": "object",
    "properties": {
      "title": { "type": "string" },
      "content": { "type": "string" },
      "tags": { "type": "array", "items": { "type": "string" } },
      "idempotency_key": { "type": "string" }
    },
    "required": ["title", "content"]
  },
  "annotations": {
    "destructiveHint": true
  }
}
```

#### Trusted manifest (after review)

```json
{
  "manifest_format_version": "1.0",
  "capability_name": "notes.note.create",
  "capability_version": 1,
  "title": "Create a project note",
  "description": "Create a new Markdown note in the project vault.",
  "input_schema": {
    "type": "object",
    "properties": {
      "title": { "type": "string" },
      "content": { "type": "string" },
      "tags": { "type": "array", "items": { "type": "string" } },
      "idempotency_key": { "type": "string" }
    },
    "required": ["title", "content"],
    "additionalProperties": false
  },
  "output_schema": {
    "type": "object",
    "properties": {
      "path": { "type": "string" },
      "modified": { "type": "boolean" }
    },
    "required": ["path", "modified"]
  },
  "effects": ["filesystem.write"],
  "permission_scope": {
    "kind": "path_prefix",
    "allowed_prefixes": ["projects/", "daily/"]
  },
  "reversibility": "compensatable",
  "determinism": "deterministic",
  "idempotency": {
    "mechanism": "argument_key",
    "argument_name": "idempotency_key",
    "key_source": "evaluation_id/action_id",
    "description": "Server deduplicates by idempotency_key argument. The host supplies evaluation_id/action_id as the key."
  },
  "confirmation_policy": {
    "standing_permitted": false,
    "per_call_required": true,
    "description": "Creating notes requires per-call confirmation."
  },
  "timeout_ms": 10000,
  "retry_policy": {
    "max_retries": 3,
    "backoff_ms": 1000,
    "allowed_on": ["outcome_unknown"],
    "requires_idempotency_proof": true
  },
  "provider": {
    "identity": "obsidian-local",
    "display_name": "Obsidian (local vault)",
    "identity_source": "host_configuration",
    "description": "Host-assigned identity for the local Obsidian MCP server."
  },
  "binding": {
    "kind": "mcp",
    "server_name": "obsidian",
    "tool_name": "obsidian_create_note",
    "adapter": null
  },
  "digest": "sha256:9f8e7d6c..."
}
```

#### Approved capability projection (planner input)

```json
{
  "name": "notes.note.create",
  "version": 1,
  "inputs": {
    "title": "string",
    "content": "string",
    "tags": "array"
  },
  "effects": ["filesystem.write"],
  "reversibility": "compensatable",
  "manifest_digest": "sha256:9f8e7d6c..."
}
```

#### Proposed Tethers Action (in Plan)

```json
{
  "action_id": "action_1",
  "idempotency_key": "eval_002/action_1",
  "capability": "notes.note.create",
  "capability_version": 1,
  "manifest_digest": "sha256:9f8e7d6c...",
  "arguments": {
    "title": "Architecture Decision: Capability Bridge",
    "content": "## Context\n\n...",
    "tags": ["architecture", "mcp"]
  },
  "effects": ["filesystem.write"]
}
```

#### Host checks

1. Resolve manifest by digest `sha256:9f8e7d6c...` -> found.
2. Prove the current `obsidian_create_note` provider binding still matches the
   pinned contract -> passes.
3. Validate arguments against `input_schema` -> passes. Note: `idempotency_key`
   is declared in the schema but not supplied in the Tether Action arguments.
   The host must inject it before dispatch using `key_source`
   (`evaluation_id/action_id` -> `eval_002/action_1`).
4. Check scope: the manifest declares `path_prefix` with `projects/` and
   `daily/`. The Tether supplies `title` and `content` but no explicit `path`.
   The MCP tool derives the path from the title. The host must determine the
   resulting path from the tool's documented behaviour, or the manifest must
   declare a `path` input directly. If the host cannot verify scope before
   dispatch, the manifest must require per-call confirmation and present the
   inferred path to the user.
5. Confirmation: `per_call_required` is `true` -> host obtains explicit
   confirmation. Confirmation prompt includes: capability, arguments, effects,
   manifest digest, inferred path.
6. User confirms.
7. Host injects `idempotency_key: "eval_002/action_1"` into the MCP call arguments.
8. Dispatch `tools/call` with `obsidian_create_note` and the augmented arguments.
9. Result: `{"path": "projects/lantern/architecture-decision-capability-bridge.md", "modified": true}` -> validate against `output_schema` -> passes.

#### Trail outcome

```json
{ "sequence": 6, "phase": "authorisation", "kind": "plan_authorised", "outcome": "authorised", "capability_name": "notes.note.create", "capability_version": 1, "manifest_digest": "sha256:9f8e7d6c...", "provider_identity": "obsidian-local", "execution_id": "exec_002", "attempt_id": "exec_002/attempt_1", "timestamp": "2026-07-21T10:45:00Z" },
{ "sequence": 7, "phase": "authorisation", "kind": "action_confirmed", "outcome": "confirmed", "confirmation_decision": "confirmed", "execution_id": "exec_002", "attempt_id": "exec_002/attempt_1", "timestamp": "2026-07-21T10:45:05Z" },
{ "sequence": 8, "phase": "execution", "kind": "action_started", "outcome": "started", "execution_id": "exec_002", "attempt_id": "exec_002/attempt_1", "timestamp": "2026-07-21T10:45:05Z" },
{ "sequence": 9, "phase": "execution", "kind": "action_completed", "outcome": "completed", "capability_name": "notes.note.create", "capability_version": 1, "manifest_digest": "sha256:9f8e7d6c...", "provider_identity": "obsidian-local", "execution_id": "exec_002", "attempt_id": "exec_002/attempt_1", "timestamp": "2026-07-21T10:45:06Z" }
```

---

## 15. Explicit rejected cases

### Case 1: Discovered tool with no trusted manifest

A Plan references `github.issue.create@1` but no manifest with that name and
version is installed. The host denies execution: `unknown_capability` at the
manifest resolution stage. The discovered MCP tool's existence does not grant
it planner visibility or execution authority.

### Case 2: Manifest/tool schema mismatch

The installed manifest declares `input_schema` requiring `{"repository": "string", "title": "string"}`. The discovered MCP tool has changed its schema to require `{"owner": "string", "repo": "string", "title": "string"}`. The host marks the capability unavailable. No new Plan may use it, and no existing Plan may dispatch an undispatched Action through the changed provider. Re-review is required; the new manifest receives a new digest, and old Plans must be re-evaluated against the newly approved capability projection.

### Case 3: Changed server or provider identity

The manifest binds to `server_name: "obsidian"` with `provider.identity: "obsidian-local"`. The MCP server is replaced with a different Obsidian server instance. The host configuration changes the server identity. The manifest must be re-reviewed and re-installed with a new digest reflecting the new provider identity. Old Plans referencing the old digest cannot execute against the new server unless the host has an immutable old binding or equivalent proof for the exact old contract.

### Case 4: Missing usable output schema without a reviewed typed adapter

A discovered MCP tool returns unstructured text (no `structuredContent`, no
declared `outputSchema`). The review process cannot produce a `output_schema`
for the manifest. The tool requires a separately reviewed typed adapter that
parses the unstructured text into a structured result. Without an adapter, the
tool cannot become a trusted capability. This is rejected at manifest review
time, not at execution time.

### Case 5: Action outside permission scope

The manifest's `permission_scope` allows `path_prefix: ["projects/"]`. A Tether
Action proposes `path: "secrets/passwords.md"`. The host checks scope at
execution time: `"secrets/"` is not within `["projects/"]`. The host denies
execution with `scope_violation`. Confirmation cannot override scope.

### Case 6: Required confirmation denied

The manifest requires `per_call_required: true`. The host presents the
confirmation prompt. The user denies. The host records `action_failed` with
`confirmation_denied`. The Action is not dispatched.

### Case 7: Non-idempotent write after an outcome-unknown timeout

The manifest declares `idempotency.mechanism: "none"`. The host dispatches a
write call. The call times out after dispatch -> `outcome_unknown`. The
`retry_policy.requires_idempotency_proof` is `true`. Since `mechanism` is
`"none"`, there is no idempotency proof. Retry is **forbidden**. The host
records `outcome_unknown` and stops. The Trail preserves the uncertainty.

### Case 8: Remote annotations claiming safety contrary to the manifest

The MCP tool advertises `annotations: {"readOnlyHint": true}`, but the
installed manifest declares `effects: ["filesystem.write"]`. The manifest
controls. Discovery annotations are untrusted. The host enforces the manifest's
declared effects and permission scope.

### Case 9: Plan referencing a stale manifest digest

A Plan was created when manifest digest `sha256:abc123` was installed. The
manifest is later updated (digest becomes `sha256:def456`) and the old version
is removed from the store. The Plan references `sha256:abc123`. The host cannot
resolve the digest -> `manifest_not_found`. Execution is denied. The Plan must
be re-evaluated against the current manifest set.

### Case 10: Old digest with changed provider binding

A Plan references an installed old digest, but the currently connected
provider/tool no longer matches its pinned contract. Dispatch is denied unless
an immutable old binding or equivalent proof is available. Retaining the old
manifest document alone does not prove that the Action can still execute safely.

### Case 11: Credentials or secrets supplied through a Plan

A Tether Action argument contains `api_key: "sk-..."`. This is not a bridge
design rejection; it is a Tether authoring error. Credentials must never appear
in Tether source, Plans, or Trails. Manifest schemas may describe
credential-shaped inputs, but values are supplied only by Columbo from trusted
host storage at dispatch time, keyed by provider identity and binding. The Plan
never sees them. The rejection message must tell the author to remove the value
and rely on host credential injection; it must not suggest renaming or
re-encoding the value.

---

## 16. Columbo C1 implementation boundary (complete)

Columbo C1 is the first implementation pass for manifest parsing, validation,
and digesting. All five C1 tasks are complete.

C1 final checkpoint: `34330b3` — feat: validate Columbo manifest semantics

Completed C1 tasks:

1. **C1a1: data types and structured error model** ✓
2. **C1a2: strict parsing, unknown-field handling, and recursive duplicate-key
   rejection** ✓
3. **C1b1: investigate and verify the JCS implementation/dependency** ✓
   — selected `serde_json_canonicalizer` 0.3.x.
4. **C1b2: canonicalisation, SHA-256, and official/golden vectors** ✓
5. **C1c: semantic and cross-field validation** ✓

Three settled C1c invariants:

- Null `permission_scope` → `per_call_required` must be `true`.
- `output_schema` must not be empty `{}` or boolean `true`.
- `idempotency.mechanism` is `"none"` + effectful effects + `max_retries > 0`
  is invalid.

Reserved error codes awaiting future implementation: `InvalidEffects`,
`InvalidIdempotency`, `ContainsCredentials`, `DigestMismatch`.

## 17. Columbo C2 — Trusted Manifest Store

C2 establishes the boundary between:

- structurally and semantically valid manifests (C1);
- manifests whose declared digest has been verified (C2a);
- manifests admitted to the trusted store (C2b).

The trusted store must never accept an ordinary parsed `TrustedManifest` or
unchecked JSON directly. Its insertion boundary must require the verified
representation produced by C2a.

C2 does not establish provider trust merely because a manifest's digest
matches. Digest verification proves content identity and integrity relative to
the declared digest; it does not prove who authored, supplied or authorised the
manifest.

Planned task boundaries:

- **C2a** — Verify declared manifest digest.
- **C2b** — Store verified manifests with identity and digest indexes,
  including insertion conflicts, idempotency, and retrieval semantics.
  (C2c is merged into C2b; insertion semantics cannot be implemented
  independently of conflict and duplicate detection.)

### 17.1 C2a — Verify declared manifest digest

#### Input

Original manifest JSON text containing a supplied top-level `digest` field.

#### Verification pipeline

1. Duplicate-aware strict JSON parsing (C1a2).
2. Authoritative field, type, and enum validation (C1a2).
3. C1c semantic cross-field validation.
4. Accepted IEEE-754/I-JSON number-domain enforcement.
5. RFC 8785/JCS canonicalisation of the original strict-parsed `Value` after
   excluding only top-level `digest`, `title`, and `description`.
6. SHA-256 calculation over the canonical bytes.
7. Supplied-digest format validation.
8. Constant, deterministic comparison of supplied and calculated digest.
9. Production of a `VerifiedManifest` only after equality succeeds.

#### Digest syntax

- A supplied top-level `digest` is mandatory for C2a verification.
- Syntax: `sha256:` followed by exactly 64 lowercase hexadecimal characters.
- Uppercase hexadecimal is rejected rather than normalised.
- Leading/trailing whitespace is rejected rather than trimmed.
- Other algorithm prefixes are rejected.
- No `digest_algorithm` field is introduced.
- The supplied digest value is excluded from its own canonical digest input.
- Top-level `title` and `description` remain excluded.
- Verification never overwrites or repairs a supplied digest.

#### Error mappings

| Condition | Error code | Field pointer |
|---|---|---|
| `digest` missing | `MissingField` | `/digest` |
| `digest` not a string | `InvalidType` | `/digest` |
| `digest` syntax malformed | `InvalidValue` | `/digest` |
| Digest mismatch | `DigestMismatch` | `/digest` |

#### Verified representation (`VerifiedManifest`)

A new type carrying at minimum:

- `capability_name`: `String`
- `capability_version`: `u32`
- `verified_digest`: `String`
- `manifest`: `TrustedManifest` (the fully validated manifest data)

The `VerifiedManifest` type exists so that C2b insertion and C2c retrieval
signatures can accept `VerifiedManifest` rather than `TrustedManifest`, making
unverified insertion impossible at compile time.

`VerifiedManifest` must not contain canonical bytes or the original JSON/Value
unless a later task demonstrates a concrete need. The `verified_digest` field
is always the calculated digest, never the supplied string copied verbatim
without verification.

#### Trust boundary

A matching digest means:

- the manifest content matches its declared content identity;
- canonical bytes reproduce the declared SHA-256 value.

It does **not** mean:

- the provider is trusted;
- the manifest is authorised for installation;
- credentials may be supplied;
- the capability may be dispatched;
- Actions may execute;
- approval is granted.

Use "verified manifest" for the C2a result. Reserve "trusted store" for the
host-controlled collection that admits verified manifests under later C2b/C2c
rules.

#### C2a scope exclusions

C2a must not implement:

- manifest storage or persistence;
- MCP discovery or schema-drift checking;
- capability registry or projection;
- provider identity or provider trust;
- credential scanning, loading, or injection;
- dispatch, networking, approvals, or Action execution;
- Trail writing;
- OCaml changes;
- broader semantic validation;
- algorithm agility.

#### Later C2 boundaries (defined here, not implemented)

**C2b — Store verified manifests:**

- Insertion accepts only `VerifiedManifest` (the C2a type).
- Primary identity key: `(capability_name, capability_version)`.
- Retrieval by exact digest.
- Retrieval by exact name/version.
- Initial implementation is deterministic in-memory unless canonical documents
  already require persistence.

**C2c — Insertion conflicts, idempotency, and retrieval semantics:**

C2c will settle:

- Reinsertion of the same identity and same digest.
- Same identity with a different digest.
- Digest collision or inconsistent index state.
- Replacement/version policy.
- Deterministic error mappings.
- Retrieval and conflict tests.

C2c conflict policy is not settled here. The unresolved choices are:

- Whether same-identity-same-digest reinsertion quietly succeeds or explicitly
  signals idempotency.
- Whether same-identity-different-digest is rejected unconditionally or
  permitted under a policy switch.
- Whether digest collision (same digest, different identity) is an error or
  merely a warning.

### Remaining deferred items (beyond C2)

- MCP discovery adapter and schema-drift checker.
- Capability registry/projection supplied to the planner as deterministic input.
- Host dispatcher, provider-specific scope validators, credential injection,
  result validation, and execution Trail writer.

### Unresolved questions

1. **Scope validation before dispatch for path-derived capabilities**: When the
   Tether Action does not supply a path directly (e.g., the MCP tool derives
   the path from a title), how does the host verify scope before dispatch?
   Options: (a) require the manifest to declare a `path` input and have the
   Tether supply it; (b) have the manifest declare a path-derivation rule the
   host can execute; (c) require per-call confirmation that presents the
   inferred path. This is a material design decision for implementation.

2. **Provider identity when MCP has no trustworthy identity mechanism**: MCP
   `initialize` returns `serverInfo: { name, version }` but this is
   self-reported and mutable. The design correctly uses host-assigned
   `provider.identity` with `identity_source: "host_configuration"`. The
   host is responsible for mapping a configured MCP server connection to a
   stable identity. If a user reconfigures the MCP server endpoint without
   updating the provider identity, the host must detect the mismatch (e.g.,
   via a separate configuration fingerprint) and flag it.

3. **Adapter identity and version in the manifest**: The `binding.adapter`
   field is `null` for direct MCP bindings. When an adapter is required
   (e.g., for tools without usable output schemas), the adapter must have its
   own identity and version, and these must be included in the digest
   computation. The exact adapter manifest format is not designed here.

---

## Design principles (recapitulation)

- One generic capability contract; transport-specific bindings beneath it.
- MCP tools advertise; trusted manifests authorise.
- Tethers plans; the host executes.
- The host distrusts both the proposed Action and the remote server enough to revalidate them.
- Least privilege.
- Deny by default.
- Deterministic planning.
- Explicit execution uncertainty.
- No ambient authority.
- No credential values in declarative artifacts.
- Keep the format small enough for humans and AI systems to inspect.
