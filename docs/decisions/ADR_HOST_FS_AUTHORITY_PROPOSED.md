# ADR: Host Filesystem Authority for Ordinary Mutations (`fs.*@1`)

Status: `PROPOSED`

Date: 2026-10-08

## Context

Phase A (accepted, Tethers `4dd8588`) established Host-bound process
execution: `BindingKind::Host` with an exact `executor_identity`, sequential
PREPARE, and atomic `commit_bundle` under `tethers.authority/2` with an
opaque `composition_digest`.

Omen's Portable Shell P2 needs the filesystem half of the same seam:
ordinary mutations (`cp`, `mv`, `rm`, `mkdir`, `rmdir`, `touch`, `ln`)
physically executed by the admitted Host runtime (Omen), with Tethers owning
capability authority. The existing `file.move@1` / `file.metadata@2`
manifests are MCP-bound reference-provider fixtures with placeholder scopes;
they are **not** production authority for Host execution and must not be
repurposed by reinterpretation.

This ADR proposes the minimum versioned, scoped Host-filesystem authority
contract. Omen owns physical execution; Tethers owns this contract. No
parallel permission mechanism is created inside Omen.

## Proposal

New capabilities, each versioned independently starting at `@1`:

- `fs.copy@1`, `fs.move@1`, `fs.remove@1`, `fs.mkdir@1`,
  `fs.remove_dir@1`, `fs.touch@1`, `fs.link@1`

Each capability reuses manifest format `1.0` with:

- `binding`: `{"kind": "host", "executor_identity": "<exact admitted Host>"}`
- `permission_scope`: `{"kind": "path_prefix", "allowed_prefixes": [...]}` with
  **real admitted roots** (cwd-rooted session scope plus explicitly granted
  roots), never placeholders.
- `confirmation_policy`: `per_call_required: true` for destructive members
  (`fs.remove`, `fs.move` with overwrite, recursive variants); standing
  permission only for narrowly scoped non-destructive members.
- `retry_policy`: `max_retries: 0` unless the capability carries an
  idempotency proof (see replay).
- `effects`: `data.read` / `data.write` / `data.move` / `data.delete` /
  `metadata.read` as applicable; `reversibility` honest per operation
  (`fs.remove` without backup is `irreversible`, never `compensatable`).

### Operation identity

Every authorized mutation carries:

- `operation`: one of `copy | move | remove | make_dir | remove_dir |
  touch | link`.
- `idempotency_key`: caller-supplied, unique per intended effect; the
  existing replay authority refuses reuse (same rule as bundle members).
- `argv_digest`: digest of the exact resolved operation arguments Omen will
  execute; any drift between PREPARE and execution fails closed.

### Source / destination scope

- All paths are absolute at authority time (Omen resolves `cwd`, `~`,
  symlinks-to-parents before PREPARE; Tethers never parses shell syntax).
- `sources[]` and `dest` (where applicable) must each fall under an admitted
  `path_prefix`, else deny.
- Recursive operations (`copy -r`, `rm -r`) declare `recursive: true` and
  are scoped by the admitted root, not by pre-enumerated trees (trees change
  under concurrency; enumeration is Omen's estimate, not authority).

### Overwrite decisions

- `overwrite: refuse | allow` is an explicit authority-time field, never a
  default-inferred behaviour. `fs.copy` / `fs.move` default to `refuse`;
  `allow` requires explicit per-call approval material.
- Omen's preflight estimate (`overwrites[]`) is advisory evidence attached
  to the request; the decision is Tethers'.

### Path identity

- Authority binds the canonicalized absolute path **and** a symlink
  treatment declaration: `follow_parent | no_follow`.
- A path whose parent chain resolves through a symlink after PREPARE is a
  scope violation (TOCTOU): Omen re-verifies identity at the
  last-responsible moment and reports drift instead of executing.

### Replay

- `idempotency_key` + existing replay authority: a committed key can never
  authorize a second physical effect, including across bundles and across
  crashes (same invariant as `commit_bundle` members).
- Read-only identification (`file.metadata`-class) stays outside this
  contract; no new replay surface is created for reads.

### Approval

- ASK approvals are exact authority proofs consumed only by the atomic
  `commit_bundle` path (A8 semantics): all member approvals proven ready
  before any is consumed; consumption is recoverable without partial
  dispatch authority.
- Destructive members always require per-call approval material; the bundle
  carries it, not ambient session permission.

### Truthful partial outcomes

- Atomic authority does not make filesystems atomic. Per-member terminal
  outcomes reuse the A10 vocabulary: `attempted`, `succeeded`, `failed`,
  `uncertain`, plus `not_attempted_due_to_bundle_start_failure` for members
  authorized but never physically attempted.
- Omen reports per-target observations (created / removed / overwritten /
  absent-ok); Tethers records them, never invents them.

## Non-goals

- No `fs.exec`, no permission-mode modelling beyond honest refusal
  (`chmod` stays out until its Windows/Linux semantics can be stated
  truthfully).
- No raw byte transport through Tethers (A12 holds for filesystem content).
- No change to `tethers.authority/1`, `process.execute@1`, `together`
  semantics, or the MCP-bound fixture manifests.

## Acceptance (to be proven before merge)

1. Manifests validate under the existing manifest parser + `kind: host`
   requires exact `executor_identity`.
2. Scope tests: outside-root source/dest deny; placeholder scopes rejected.
3. Overwrite tests: default refuse; `allow` without approval material fails.
4. Replay tests: same `idempotency_key` twice refuses; cross-crash ledger
   recovery refuses reuse.
5. Bundle tests: `fs.*` members commit via `commit_bundle` with
   `composition_digest`; deny-one-member admits zero.
6. Partial-outcome tests: mid-bundle physical failure records truthful
   per-member outcomes including `not_attempted`.
7. Windows + Linux green; authority/1 suite unchanged.

## Consequence

On acceptance under Tethers ownership: Omen pins the accepted revision,
binds P2 mutations through this contract (replacing `RefusedClosed` with
admitted execution), and qualifies P2 per its acceptance list. P2 must never
be presented as functional on scaffolding alone.
