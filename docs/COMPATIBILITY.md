# Tethers 1.0 Compatibility Contract

Status: living contract
Updated: 2026-09-14

Tethers 1.0 compatibility is compatibility of meaning at the consequential
boundary, not a promise that every internal implementation detail remains
unchanged. The version axes are defined in `VERSIONING.md`.

## Compatibility promise

During Tethers 1.x, a caller may rely on the explicitly supported versions of
the Human Tether language and machine contracts. Unsupported or incompatible
versions fail closed. They are never guessed, silently downgraded, silently
reset, or partially interpreted as though fully understood.

The stable boundary remains:

```text
explicit input -> deterministic Plan -> host authority -> bounded execution -> honest evidence
```

Changing implementation, provider, transport, or human-readable wording must
not silently change that meaning.

## Human Tether source

The supported source language is Tethers `0.1`, defined by `tethers-0.1/SPEC.md`.
Existing valid `.tether` source and Tether Sets using that version must continue
to parse and retain their defined meaning during 1.x, subject to explicit
validation of their declared language version.

New syntax is not implied by a product release. An incompatible language
successor receives an explicit language version and a migration or opt-in
translation path. No hidden aliases, coercions, new branching, or result
chaining are introduced as compatibility conveniences.

## Machine interface classification

| Surface | Classification for the 1.0 audit | Compatibility rule |
| --- | --- | --- |
| Core request/response JSON | VERSIONED | Protocol `0.1` is strict; unsupported versions return structured errors |
| CLI JSON envelope and discovery | VERSIONED | `tethers.cli/1` remains explicit; fields may be added only compatibly and schema changes are versioned |
| Plan JSON | VERSIONED | Preserve ordered Action/group semantics; incompatible shape or meaning gets a new schema |
| Trail/receipt JSON | VERSIONED | Preserve causal distinctions and evidence meaning; readers reject unsupported versions |
| Configuration and persistent state | VERSIONED | Format versions, validation, protected migrations, and backups/pre-migration protection are required |
| Capability manifests | VERSIONED | Validate manifest format, exact name/version, provider identity, binding pins, and digest |
| Plug package and Socket | VERSIONED | Package format and Socket major are explicit; conformance is not trust or permission |
| Policy, scope, approval, replay | VERSIONED | Host owns authority and durable safety state; changes require explicit contract evidence |
| MCP adapter | VERSIONED / EXPERIMENTAL until current-protocol audit | MCP revision is negotiated separately; MCP never becomes a second Tethers evaluator |
| OCaml/Rust module internals | INTERNAL | No compatibility promise for implementation shapes |

The current statuses are deliberately conservative. `VERSIONED` means a
compatibility mechanism exists or is required; it does not mean the 1.0 audit
is complete.

## Human-facing text

Human-readable messages, help prose, explanations, and diagnostics may improve
between compatible releases unless a document explicitly declares a string a
machine API. Machines must use structured output, schema identities, stable
error codes, exit classes, and documented fields instead of scraping prose.

## Fail-closed rules

Consumers and migration tools must reject:

- unsupported language, protocol, schema, package, socket, transport, or
  storage versions;
- malformed, ambiguous, duplicate, or incompletely understood data;
- missing trust, policy, scope, provider pins, or durable replay state;
- a proposed compatibility downgrade that changes semantic meaning.

An error must remain distinguishable from `not_matched`, denial, an unattempted
Action, a definite provider failure, and post-invocation uncertainty.

## Migration

Any incompatible persistent-format change requires:

1. an explicit new format version;
2. a deterministic, reviewable migration;
3. validation before and after migration;
4. pre-migration protection appropriate to the data, such as a backup or
   durable copy where loss would matter; and
5. failure without destructive silent recovery.

No migration may turn unknown data into an empty/default state merely to make a
startup check pass. A failed migration leaves the original state available for
diagnosis or explicit recovery.

## 0.7 to 1.0 compatibility corpus

R6 must build and retain a corpus covering at least:

- valid and invalid `.tether` source;
- Tether Sets;
- configuration and persistent host state where public durability matters;
- trusted Capability manifests and provider bindings;
- Plug package metadata and Socket identity;
- policy, scope, approval, replay, and idempotency state;
- Plan JSON, ordered Actions, Together groups, and ProgramDigest identity;
- Trail and receipt JSON, including failure and uncertainty;
- CLI structured output and exit classes; and
- MCP requests/responses for the supported adapter revision.

Each corpus case records its version, expected classification, exact semantic
meaning, and whether compatibility is byte, structural, or semantic. Historical
worker notes remain evidence of their checkpoint; they are not rewritten to
pretend that an old fixture was already a 1.0 contract.

## Ownership boundary

Core owns deterministic interpretation. The host owns policy, scope, provider
trust, execution, persistence, replay, and Trail truth. A provider or MCP server
cannot redefine compatibility by advertising a different contract at runtime.
