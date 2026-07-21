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
               | presented to planner
               v
+------------------------------------------+
|  Tethers Planner (OCaml)                 |  DETERMINISTIC
|  (evaluates Conditions, proposes Actions |
|   referencing capability name, version,   |
|   and manifest digest - never executes)   |
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
| Manifest -> planner | Manifest fields form the capability contract. | N/A - manifest is trusted by definition. |
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

The host resolves the manifest by digest. If the digest does not match any
currently installed manifest, execution is denied. A Plan cannot silently
execute against a changed or removed manifest.

---

## 4. Trusted manifest format

### Canonical structure

A manifest is a JSON document stored in the host's trusted manifest store.
Every field below the `manifest_format_version` header is authoritative for
execution. Display-only fields (comments, annotations, authoring hints) are
excluded from the contract digest and must not affect execution behaviour.

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
  "digest": "sha256:...",
  "digest_algorithm": "sha256"
}
```

### Digest computation

The digest is computed over a canonical JSON representation of every
execution-authoritative field. The following fields are **included** in the
canonical form:

1. `capability_name`
2. `capability_version`
3. `input_schema`
4. `output_schema`
5. `effects`
6. `permission_scope`
7. `reversibility`
8. `determinism`
9. `idempotency`
10. `confirmation_policy`
11. `timeout_ms`
12. `retry_policy`
13. `provider` (all sub-fields)
14. `binding` (all sub-fields)

The following fields are **excluded** from the canonical form:

- `digest` (self-referential)
- `digest_algorithm`
- `manifest_format_version`
- `title`
- `description`
- Any field-level `description` sub-keys inside schemas or policies that are
  purely display metadata. Structural schema constraints (`type`, `properties`,
  `required`, `additionalProperties`, `items`) are included; inline
  `description` strings inside schema objects are excluded.

**Canonicalization rules:**

1. Recursively sort all object keys in lexicographic order.
2. Remove excluded fields.
3. Serialize as compact UTF-8 JSON with no trailing newline.
4. Compute SHA-256 over the resulting byte sequence.
5. The digest string is `"sha256:"` followed by the lowercase hex encoding.

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
runtime schema may be a subset of the manifest's fields, generated at request
time from the manifest. The host controls this; the planner does not need to
know the manifest exists. The planner only needs to receive the subset it
already uses.

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
Manifest is presented to planner as an available capability
  (via the existing runtime schema subset)
        |
        v
Planner may propose Actions referencing this capability
        |
        v
Host resolves manifest by digest at execution time
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
  - existing Plans referencing the old digest remain valid
    but can only execute if the old manifest is still installed
        |
        v
Changed contract requires explicit human re-review
  - no automatic reapproval
  - new review produces a new manifest with new digest
  - old digest may be retained for existing Plan execution
    or explicitly revoked
```

### Time-of-check/time-of-use prevention

- A Plan records `manifest_digest` at evaluation time.
- The host resolves the manifest by digest at execution time.
- If the digest does not match any currently installed manifest, execution is
  denied with `manifest_not_found`.
- A Plan created under one manifest version cannot silently execute under a
  different version.
- If a manifest is revoked (removed from the store), all Plans referencing its
  digest become unexecutable.

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
a client-supplied key.

```json
{
  "idempotency": {
    "mechanism": "server_dedup",
    "description": "The server detects duplicate requests by content hash and returns the cached result."
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

- If `mechanism` is `"none"`, automatic retry is **forbidden** for writes.
  The host may only retry read-only, deterministic, idempotency-safe calls.
- If `mechanism` is `"argument_key"`, the host must supply the key before
  dispatch and may retry on `outcome_unknown` (see section 11).
- If `mechanism` is `"server_dedup"`, the host may retry on `outcome_unknown`
  without supplying an additional key.
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

1. **No automatic retry for writes** unless the manifest's `idempotency.mechanism`
   is `"argument_key"` or `"server_dedup"` with a reviewed, concrete proof of
   safety.
2. Retry is **only** permitted for `outcome_unknown`. A confirmed `action_failed`
   must not be automatically retried.
3. Each retry attempt uses a stable `execution_id` with an incrementing
   `attempt_id` (e.g., `exec_001/attempt_1`, `exec_001/attempt_2`).
4. All attempts are recorded in the Trail.
5. The host must respect `retry_policy.max_retries` and `retry_policy.backoff_ms`.
6. If `retry_policy.requires_idempotency_proof` is `true` and the manifest's
   idempotency mechanism is `"none"`, retry remains forbidden regardless of
   `max_retries`.

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

1. **Tethers evaluates** the Tether, Conditions, and Action arguments against
   the runtime capability schemas supplied by the host.
2. **A Plan is produced** containing proposed Actions. Each Action references
   the capability name, capability version, and the **manifest digest** that
   was current at evaluation time.
3. **The host receives the Plan.** It resolves each Action's manifest by digest
   from the trusted manifest store.
4. **The host validates** each Action's arguments against the full manifest
   `input_schema`.
5. **The host checks scope** by evaluating the Action's arguments against the
   manifest's `permission_scope`.
6. **The host checks confirmation policy.** If `per_call_required` is `true`
   or no standing approval covers this call, the host obtains explicit
   confirmation.
7. **The host dispatches** the bound MCP call using the manifest's `binding`
   fields.
8. **The host awaits the result** within `timeout_ms`.
9. **The host validates the result** against the manifest's `output_schema`
   (if present).
10. **The host appends execution Trail entries** - at minimum:
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

### Redaction rules

- Do not record credentials, tokens, API keys, or secrets.
- Do not record full request/response payloads unless explicitly configured
  for debugging and scoped to a non-production environment.
- Record argument values only to the extent needed for audit (capability,
  scope, and intent). The host's logging policy controls this.

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
  "digest": "sha256:1a2b3c4d...",
  "digest_algorithm": "sha256"
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
2. Validate `path` against `input_schema` -> passes.
3. Check scope: `projects/lantern/architecture.md` has prefix `projects/` -> within scope.
4. Confirmation: `per_call_required` is `false`; standing approval exists for
   this digest and scope -> skip confirmation.
5. Dispatch `tools/call` with `obsidian_read_note` and `{"path": "projects/lantern/architecture.md"}`.
6. Result: `structuredContent` with `content` and `frontmatter` -> validate against `output_schema` -> passes.

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
  "digest": "sha256:9f8e7d6c...",
  "digest_algorithm": "sha256"
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
2. Validate arguments against `input_schema` -> passes. Note: `idempotency_key`
   is declared in the schema but not supplied in the Tether Action arguments.
   The host must inject it before dispatch using `key_source`
   (`evaluation_id/action_id` -> `eval_002/action_1`).
3. Check scope: the manifest declares `path_prefix` with `projects/` and
   `daily/`. The Tether supplies `title` and `content` but no explicit `path`.
   The MCP tool derives the path from the title. The host must determine the
   resulting path from the tool's documented behaviour, or the manifest must
   declare a `path` input directly. If the host cannot verify scope before
   dispatch, the manifest must require per-call confirmation and present the
   inferred path to the user.
4. Confirmation: `per_call_required` is `true` -> host obtains explicit
   confirmation. Confirmation prompt includes: capability, arguments, effects,
   manifest digest, inferred path.
5. User confirms.
6. Host injects `idempotency_key: "eval_002/action_1"` into the MCP call arguments.
7. Dispatch `tools/call` with `obsidian_create_note` and the augmented arguments.
8. Result: `{"path": "projects/lantern/architecture-decision-capability-bridge.md", "modified": true}` -> validate against `output_schema` -> passes.

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

The installed manifest declares `input_schema` requiring `{"repository": "string", "title": "string"}`. The discovered MCP tool has changed its schema to require `{"owner": "string", "repo": "string", "title": "string"}`. The digest has changed. The host marks the capability unavailable. No existing Plan can execute against this manifest because the old digest no longer matches any installed manifest. Re-review is required.

### Case 3: Changed server or provider identity

The manifest binds to `server_name: "obsidian"` with `provider.identity: "obsidian-local"`. The MCP server is replaced with a different Obsidian server instance. The host configuration changes the server identity. The manifest must be re-reviewed and re-installed with a new digest reflecting the new provider identity. Old Plans referencing the old digest cannot execute against the new server.

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

### Case 10: Credentials or secrets supplied through a Plan

A Tether Action argument contains `api_key: "sk-..."`. This is not a bridge
design rejection; it is a Tether authoring error. Credentials must never appear
in Tether source, Plans, manifests, or Trails. The host's credential management
is entirely outside the scope of the capability bridge. If a capability
requires authentication, the host supplies credentials from its own secure
store at dispatch time, keyed by provider identity. The Plan never sees them.

---

## 16. Future implementation boundary

The following are the smallest likely implementation pieces when M7 design
transitions to implementation. They are identified here but **not built** as
part of M7.

1. **Manifest parser/validator** - parse the canonical JSON manifest format,
   validate required fields, compute and verify the contract digest.
2. **Trusted manifest store** - filesystem or database store for installed
   manifests, keyed by `(capability_name, capability_version)`, retrievable
   by digest.
3. **MCP discovery adapter** - call `tools/list` on configured MCP servers,
   compare discovered schemas against installed manifest pinned fields, report
   drift.
4. **Schema-drift checker** - compute current contract digest from discovered
   tool metadata, compare against installed manifest digest, flag mismatches.
5. **Capability registry** - the subset of installed manifests presented to the
   planner as runtime capability schemas. Only manifests with a clean drift
   check are included.
6. **Host dispatcher** - resolve manifest by digest, validate arguments, check
   scope, obtain confirmation, bind and dispatch the MCP `tools/call`, validate
   the result.
7. **Provider-specific scope validators** - for each `permission_scope.kind`,
   implement the validation function that checks Action arguments against
   allowed scope values.
8. **Execution Trail writer** - append authorisation and execution Trail entries
   with all bridge-specific fields.

### Unresolved questions

1. **Scope validation before dispatch for path-derived capabilities**: When the
   Tether Action does not supply a path directly (e.g., the MCP tool derives
   the path from a title), how does the host verify scope before dispatch?
   Options: (a) require the manifest to declare a `path` input and have the
   Tether supply it; (b) have the manifest declare a path-derivation rule the
   host can execute; (c) require per-call confirmation that presents the
   inferred path. This is a material design decision for implementation.

2. **Canonicalization of JSON Schema `description` fields**: The current
   exclusion rule removes inline `description` keys from schema objects. If a
   schema uses `description` to carry structural meaning (unusual but possible),
   this could mask drift. Implementation should verify that no discovered MCP
   tools rely on `description` for structural semantics.

3. **Multi-action Plans and manifest resolution**: If a Plan contains five
   Actions referencing the same manifest digest, and the manifest is revoked
   between Action 3 and Action 4, should the host stop or continue? The
   conservative answer is "stop" (the Plan's contract is broken), but the
   host may reasonably want to record completed Actions and deny the remainder.
   This is a host-policy decision, not a bridge-format decision.

4. **Provider identity when MCP has no trustworthy identity mechanism**: MCP
   `initialize` returns `serverInfo: { name, version }` but this is
   self-reported and mutable. The design correctly uses host-assigned
   `provider.identity` with `identity_source: "host_configuration"`. The
   host is responsible for mapping a configured MCP server connection to a
   stable identity. If a user reconfigures the MCP server endpoint without
   updating the provider identity, the host must detect the mismatch (e.g.,
   via a separate configuration fingerprint) and flag it.

5. **Adapter identity and version in the manifest**: The `binding.adapter`
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
- No credentials in declarative artifacts.
- Keep the format small enough for humans and AI systems to inspect.
