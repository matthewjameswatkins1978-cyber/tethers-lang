# Tethers Rust Engineering Guide for Agents

**Status:** required operational guidance for Rust implementation  
**Primary audience:** Goose, Cline, Codex, and other AI coding agents  
**Secondary audience:** senior Rust engineers and reviewers  
**Baseline:** Rust 1.89.0, edition 2021  
**Scope:** `tethers-0.1/host-rust/` and Rust capability-provider work

This guide does not replace the current task packet, Tethers 0.1 specification,
accepted Red designs, or the implementation-language standard. It tells an
agent how to implement those contracts in Rust without weakening them.

## Fast safety scan

Before every edit, stop and reassess when any answer is yes:

1. Am I moving Tether parsing, Anchor matching, Condition evaluation, or Action
   planning from OCaml into Rust?
2. Am I inventing retries, fallback IDs, approval IDs, permission, compatibility,
   recovery, or a missing product semantic?
3. Am I creating a second execution, replay, permission, Trail, or provider-call
   boundary instead of using the accepted one?
4. Am I replacing accepted Win32 handle, locking, Job Object, or publication
   logic with an ordinary standard-library shortcut?
5. Am I deriving `Clone` on a proof token, exclusion, admission, approval, or
   other authority-bearing value?
6. Am I turning timeout, EOF, malformed framing, or transport loss into known
   provider failure when the request may have crossed the call boundary?
7. Am I refactoring adjacent code merely because it does not meet this guide?
8. Am I using an API without checking Rust 1.89 and the exact locked crate
   version?

A stop item is a design boundary, not necessarily a permanent prohibition.

## Authority order

Use the narrowest controlling authority:

1. Current authorised task packet and attached Red design.
2. `tethers-0.1/SPEC.md`.
3. Accepted architecture and milestone design documents.
4. `docs/CONSTITUTION.md`.
5. `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md`.
6. `docs/DECISIONS.md`.
7. This guide.
8. Existing code patterns.

Existing code is evidence, not automatic authority. If authorities conflict,
report the exact conflict and stop rather than reconciling it silently.

| Path pattern | Ordinary implementation rule |
| --- | --- |
| `tethers-0.1/SPEC.md`, `docs/CONSTITUTION.md` | Read-only unless the task explicitly changes the contract |
| accepted `docs/J*_DESIGN.md` | Read-only unless the exact Red design task authorises revision |
| `tethers-0.1/host-rust/src/` | Change only inside the packet boundary |
| Rust tests and fixtures | Add evidence; never weaken expectations merely to pass |
| `Cargo.toml` | Change only when dependency, feature, package, or compiler work is authorised |
| `Cargo.lock` | Update only through Cargo; never edit manually |
| worker notes | Record exact implementation evidence |

## Architectural boundary

> Lantern Keeper remembers. Tethers coordinates. AI interprets. Matthew decides.

### OCaml Tethers Core owns

- Tether parsing and structural validation;
- language-version validation;
- Anchor matching;
- deterministic Condition evaluation;
- Action planning;
- deterministic planner Trail entries.

### The Rust host owns

- configured provider binding and retained sessions;
- manifest admission and live capability projection;
- exact capability/version resolution;
- execution-boundary schema validation;
- permission and one-shot approval handling;
- durable replay admission;
- serial one-shot provider dispatch;
- deadline and uncertainty classification;
- host Trail entries and result Anchors;
- host-local persistence and Windows proof boundaries.

Rust must not become a second planner or reinterpret Tether source.

## Toolchain and dependency truth

The Tethers host currently declares edition 2021 but does not yet declare an
MSRV or pin `rust-toolchain.toml`. This guide uses Rust 1.89.0 because Lantern
Keeper declares Rust 1.89 and because agents need one known API baseline.

Do not add `rust-version` or a toolchain pin during unrelated work. That is a
separate repository decision.

Before coding:

```powershell
rustup run 1.89.0 rustc --version
rustup run 1.89.0 cargo --version
cargo +1.89.0 metadata --format-version 1 --locked
cargo +1.89.0 tree -e features
```

Use local Rust 1.89 documentation and the locked crate versions. Never code
against “latest” documentation without proving the versions match.

Every new dependency requires:

- a task-authorised requirement;
- a reason existing dependencies and `std` are insufficient;
- Rust 1.89 compatibility;
- justified feature flags;
- compatible licence and maintenance posture;
- boundary tests;
- review of transitive and platform risk.

Do not update unrelated dependencies.

## Governing Rust rule

Use Rust fully, but selectively.

Prefer, in order:

1. specified behaviour and trust boundaries;
2. invalid states made unrepresentable;
3. accurate domain types;
4. explicit effects, ownership, ordering, and failure;
5. ordinary Rust idioms;
6. minimal conceptual surface;
7. measured optimisation.

The target is not beginner Rust and not clever Rust. It is direct production
Rust another senior engineer or capable AI can continue without recovering
hidden assumptions from chat.

## Trust transitions belong in types

A host execution is a sequence of increasingly trusted states:

```text
untrusted bytes
→ decoded DTO
→ correlated planner response
→ resolved Action
→ schema-validated Action
→ permission decision
→ replay admission
→ durable intent
→ invocation armed
→ classified provider outcome
→ durable outcome
→ result Anchor
```

Do not represent the whole sequence with `serde_json::Value`, `String`, and
booleans.

Use validated newtypes for identities and digests. Keep constructors private
when canonical form matters. Prefer `TryFrom`, `FromStr`, or named validated
constructors over infallible `From<String>`.

Use proof-carrying values where later operations require evidence:

```rust
pub struct SchemaValidatedAction {
    action: PlannedAction,
    capability: TrustedCapability,
    arguments: ValidatedArguments,
}

pub struct HostExecutionAdmission {
    execution_id: ExecutionId,
    binding: ExecutionBinding,
    exclusion: LogicalExecutionExclusion,
}
```

A provider-call API must not accept a merely parsed Action.

Do not derive `Clone` automatically. Authority, held exclusion, one-shot
approval, admission, or consumed state may need to move exactly once.

## Closed states are enums

Use enums for mutually exclusive states:

```rust
pub enum PermissionDecision {
    Allow(PermissionGrant),
    Ask(ApprovalRequest),
    Deny(DenialReason),
    Unavailable(UnavailabilityReason),
}

pub enum ProviderOutcome {
    Succeeded(ValidatedOutput),
    Failed(ProviderFailure),
    Uncertain(UncertaintyReason),
}
```

`uncertain` is not failure. It means an effect may have occurred but the host
cannot prove the result.

Known provider outcome and audit completion are separate facts. A final Trail
write failure must not rename provider success or failure.

Match security-sensitive enums exhaustively. Avoid wildcard arms that hide new
states.

## Protocol DTOs and Serde

Serde DTOs describe input shape. They do not automatically become trusted host
state.

Convert at deliberate boundaries:

```text
PlannerResponseDto → CorrelatedPlannerResponse
ProviderManifestDto → VerifiedManifest
ProviderResponseDto → ValidatedProviderOutput
```

Contain `serde_json::Value` at OCaml/MCP compatibility adapters. Validate once,
then convert to typed internal values.

Use `#[serde(deny_unknown_fields)]` only when the controlling contract requires
specification-exact rejection. For explicitly forward-compatible third-party
envelopes, validate required and security-relevant fields while permitting only
contract-authorised additive metadata.

An unknown field may never affect identity, digest, provider selection,
permission, scope, replay, or execution without a versioned contract.

Reject duplicate JSON keys before converting to `Value` or a map that overwrites
them. Use fixed structs, a custom visitor, or `MapAccess` tracking. Once
duplicates collapse, ambiguity can no longer be proved.

## Canonical JSON and hashing

There must be one canonicalisation and SHA-256 authority.

Do not hash:

- `HashMap` iteration order;
- debug or pretty JSON;
- display strings;
- an incidental struct layout;
- raw provider JSON when the contract requires a selected projection.

Preserve existing JCS/RFC 8785 field names, projections, and digest formats.
Changing a digest input is a storage/protocol migration, not cleanup.

Validate both underlying binding fields and the digest when the accepted design
requires both.

JSON/JCS forbids `NaN` and infinities. Prefer integers or explicitly specified
fixed-precision decimal representations for security-sensitive canonical
payloads. If finite floats are genuinely allowed, validate finiteness and test
negative zero, exponent forms, and precision boundaries.

## Execution ordering

For an allowed Action or approved Ask resume, preserve this order:

1. Validate Anchor event, evaluation, and Action identities.
2. Derive the logical execution key.
3. Resolve exact capability and provider.
4. Validate manifest/provider pins, schema, and resource scope.
5. Calculate policy.
6. For fresh Ask, create no replay identity until approved resumption.
7. Admit or recover replay under the logical-key exclusion.
8. Consume one-shot approval only after fresh replay admission.
9. Publish `intent_recorded`.
10. Append and durably flush Trail intent.
11. Start the monotonic deadline.
12. Publish `invocation_armed`.
13. Cross the provider boundary at most once.
14. Classify `succeeded`, `failed`, or `uncertain`.
15. Append and durably flush Trail outcome.
16. Publish the terminal replay generation.
17. Create the standard result Anchor.

Having all operations in a different order is still wrong.

There must be one accepted execution seam. Do not duplicate dispatch, replay,
permission, or Trail ordering in a new service.

## Planner response handling

Only a fully validated `matched` response may reach replay or provider code.

- `matched`: validate protocol and all required correlation, then continue.
- `not_matched`: return no actions and make zero replay/provider calls.
- `error`: return a distinct planner-error result.
- missing or unknown status, malformed error, or correlation mismatch:
  `invalid_data`.

Required correlation includes protocol version, evaluation ID, event ID,
Tether ID, and Tether version. Do not repair missing fields locally and then
pretend the planner response was valid.

## Provider admission

A provider manifest describes claims. It does not prove identity or grant
permission.

Effective capability is the intersection of:

```text
verified manifest
AND configured local binding
AND live retained connection
AND exact Tether Set requirement
AND local policy
```

Keep admitted trust separate from current availability. Compare live MCP
`tools/list` evidence with prepared capabilities before admitting callability.

Use exact versions. Do not select “latest”, silently coerce versions, or apply
compatibility guessing absent an explicit contract.

## Permission and approval

`Ask` is not `Allow`.

Ask stops before provider dispatch and exposes only accepted public fields,
including existing evaluation/Action identities and a redacted reason. Never
fabricate an approval ID.

Bind approval to exact work: capability, provider, manifest digest, arguments,
resource scope, and Action identity. Any change requires fresh evaluation.

On approved resume, repeat fresh checks, obtain replay admission, then consume
the approval. A replay block must not consume approval merely to discover the
block. A consumed approval is never silently restored.

## Replay and durability

Replay protection is durable execution authority, not an in-memory set and not
a reconstruction from Trail.

Logical execution identity uses the exact Anchor event ID, planner evaluation
ID, and Action ID. The host creates the opaque execution UUID exactly once. No
execution UUID crosses planner, manifest, provider, or MCP boundaries.

The accepted immutable chain is:

```text
claim
→ generation 0: intent_recorded
→ generation 1: invocation_armed
→ generation 2: succeeded | failed | uncertain
```

There is no mutable head, in-place update, inferred repair, generation three,
or automatic retry.

`claimed_no_state`, `intent_recorded`, `invocation_armed`, and `uncertain` are
manual-resolution-only and block another provider call. Corrupt, partial,
unexpected, or unprovable state fails closed.

Replay ledger and Trail are separate authorities. Neither substitutes for the
other.

## Windows proof boundaries

The accepted replay backend targets verified native Windows on local fixed
NTFS. It includes handle-bound path validation, reparse-point rejection, DACL
and owner checks, OS-backed exclusion, write-through temporary creation,
`FlushFileBuffers`, handle-based no-replace rename, reopen, and validation.

Do not replace that proof with:

- `path.exists()` followed by create;
- generic `std::fs::rename`;
- replace-capable rename;
- lock-file existence;
- best-effort cleanup;
- standard-library locking alone.

Rust 1.89 standard file locking does not automatically prove the complete J09
contract.

Keep Win32 `unsafe` inside the smallest target-specific module behind safe
types. Every unsafe block must document pointer validity, buffer ownership,
handle lifetime, structure initialisation, checked return values, and the safe
invariant produced. New unsafe requires an explicit Red gate and independent
review.

## Subprocess and STDIO supervision

The current host uses `std::process` behind `SupervisedChild`. Preserve it.

A retained engine or provider process requires one owner responsible for:

- piped stdin/stdout/stderr;
- bounded protocol messages;
- bounded waiting;
- separately drained or captured stderr;
- flushing complete protocol writes;
- EOF, process exit, malformed framing, and oversized output;
- graceful close followed by forced termination;
- direct-child reaping;
- joining reader threads;
- descendant termination on Windows through the accepted Job Object;
- cleanup on drop or interruption.

Do not bypass `SupervisedChild` with ad hoc `std::process::Command`. Do not add a
second supervisor.

Never use a blocking full-stream read or `wait_with_output` for a retained
protocol child whose lifetime and other pipe are not already bounded.

If a future authorised task introduces Tokio, `kill_on_drop(true)` is only one
piece. It does not solve descendants, backpressure, framing limits, reaping,
session shutdown, or failure-versus-uncertainty classification. Do not add Tokio
only for that convenience.

Process cleanup does not prove a request was never delivered.

## Deadlines and uncertainty

Actions remain serial. Do not spawn one task per Action or parallelise a Plan.

Use one monotonic deadline and propagate the exact remaining duration. Do not
restart a fresh timeout in each layer or use wall-clock time for elapsed limits.

If expiry occurs before the provider boundary, make zero calls.

After possible delivery:

- provider-declared structured error: known `failed`;
- timeout, EOF, malformed framing, or transport loss: normally `uncertain`;
- only typed evidence proving rejection before send may be unattempted or known
  failure.

Dropping a future is local cancellation, not proof of no external effect.

Avoid holding ordinary locks across `.await`. The accepted replay exclusion is
a deliberate security boundary, not a model for general shared state.

## Error design

Use structured domain errors with stable machine identity. `thiserror` is
appropriate for libraries and domain modules. `anyhow` belongs at outer binary,
startup, test-helper, or orchestration boundaries.

Do not use `Result<T, String>` for policy, replay, protocol, dispatch, or
admission. Do not classify errors by parsing display text.

Production code must not use `unwrap` or `expect` for malformed input, I/O,
provider, policy, replay, or protocol outcomes. An `expect` is acceptable only
when a nearby tested invariant proves impossibility and the message names that
invariant.

Panic is not a provider failure mode.

## Effects, Trail, tracing, and redaction

Clock, filesystem, process, network, environment, credential, and randomness
access belong at visible boundaries. Do not hide effects in getters,
conversions, formatting, equality, hashing, or constructors that appear pure.

The Tethers Trail is product audit behaviour. `tracing` logs are operational
diagnostics. Neither substitutes for the other.

Do not place credentials, raw arguments, raw files, full provider stderr,
stacks, absolute local paths, unbounded outputs, or complete conversations in
replay or ordinary Trail records. Prefer stable IDs, digests, safe summaries,
and references. Secrets should not derive ordinary `Debug`.

## Modules and abstraction

An abstraction must pay rent by naming a stable domain concept, enforcing an
invariant, isolating an effect/trust boundary, removing repeated policy, enabling
independent testing, or preventing invalid construction.

Avoid managers, factories, repositories, forwarding services, plugin systems,
or trait objects without a present reason.

Start with modules in the existing package. Split a crate only when dependency,
testing, reuse, or release boundaries justify it.

Do not refactor untouched non-compliant code merely to satisfy this guide.
Record it under `Discoveries` and leave it untouched unless the packet includes
it. This prevents ordinary work from becoming a roaming renovation project.

Do not copy a weak pattern into new code.

## Testing

Every required negative branch needs direct evidence.

Use:

- unit tests for constructors, transitions, correlation, redaction, and digests;
- table-driven permission/outcome/replay matrices;
- golden JSON/JSONL fixtures for stable protocol and durable records;
- determinism tests across repeated runs and object insertion order;
- crash-cut tests around every replay and Trail boundary;
- concurrency tests proving one same-key claim and zero duplicate calls;
- native Windows tests for NTFS, reparse, ACL, lock, flush, rename, and process
  death;
- provider tests distinguishing declared failure, pre-call expiry, and
  post-boundary uncertainty.

Mocks do not prove Win32/NTFS behaviour. A broad happy path does not prove
fail-closed branches. Do not automatically bless fixtures because code changed.

## Required verification

From `tethers-0.1/host-rust/`, unless the packet is stricter:

```powershell
cargo +1.89.0 fmt --all -- --check
cargo +1.89.0 check --all-targets --all-features --locked
cargo +1.89.0 clippy --all-targets --all-features --locked -- -D warnings
cargo +1.89.0 test --all-targets --all-features --locked
cargo +1.89.0 build --locked
cargo +1.89.0 build --release --locked
git diff --check
git status --short --branch
```

Also run every task-packet fixture, engine, MCP transcript, public acceptance,
task checker, complete-diff, and external-toolchain command.

If a command cannot run, report the exact command and reason. Do not claim full
verification.

## Goose operating protocol

Before editing:

1. Confirm repository, worktree, branch, status, and full commit.
2. Read the packet, specification/design sections, this guide, affected code,
   focused tests, `Cargo.toml`, and `Cargo.lock`.
3. Identify all operations that can consume approval, mutate replay, write
   Trail, launch a process, or call a provider.
4. Name the negative path the task must prove.
5. State the intended file boundary.

During implementation:

- make the smallest coherent change;
- compile after the first structural step;
- preserve the one accepted execution seam;
- add focused negative evidence with the code;
- avoid whole-file rewrites and unrelated formatting;
- do not invent IDs, retries, policy, fallback, or compatibility;
- do not weaken a type to satisfy the borrow checker;
- do not clone authority to satisfy ownership;
- do not convert structured errors to strings for convenience;
- do not suppress Clippy globally.

When ownership fails, move ownership at the real boundary, shorten borrows, or
split states. Use `Arc` only for genuine shared ownership, `Mutex` only for
genuine shared mutation, and boxing only for justified indirection/object
safety. Never use `unsafe` to escape an ordinary borrow-checker problem.

Stop and report when authorities conflict, a semantic or permission rule is
missing, provider failure cannot be classified honestly, persistence cannot be
proved, new unsafe or a new dependency appears necessary, a second execution
boundary would result, or scope broadens materially.

## Worker note schema

Use this schema unless project control is stricter:

```markdown
# Worker Note

- **Task Packet:** `<name / ID / path>`
- **Owner:** `<agent or engineer>`
- **Status:** `COMPLETE | PARTIAL | BLOCKED`
- **Base Commit:** `<full hash>`
- **Final Commit:** `<full hash or NOT COMMITTED>`
- **Branch / Worktree:** `<branch and path>`

## Files Modified
- `<path>`

## Behavioural Result
<Externally observable result.>

## Invariants Preserved
- `<invariant>`

## Negative Tests Added or Updated
- `<test or fixture>` — `<failure branch proved>`

## Commands Executed
- `<exact command>` — `PASS` (`<count/result>`)

## Unrun Checks and Reason
- `None`
  
or:
- `<exact command>` — `NOT RUN`: `<reason>`

## Discoveries
- `<new fact, not an invented decision>`

## Remaining Risks
- `None`
  
or:
- `<specific risk and boundary>`

## Recommended Next Action
<One smallest concrete action.>
```

Do not omit sections by assumption. Use full hashes, exact commands, and named
tests. `COMPLETE` is forbidden when a required command was not run unless the
packet explicitly marks it optional. Discoveries do not authorise extra work.

## Review checklist

A reviewer must establish:

- Rust remains host, not second planner.
- Planner correlation is complete before dispatch.
- Exact versions, provider bindings, schema, scope, and policy are enforced.
- Ask, Deny, Unavailable, Failed, Unattempted, Uncertain, and Audit failure stay
  distinct.
- Replay identity and immutable ordering remain exact.
- No retry or duplicate provider call is possible.
- DTOs are separated from trusted domain types.
- Authority-bearing values are not cloneable without justification.
- `serde_json::Value` is contained at adapters.
- Windows and subprocess proof boundaries remain intact.
- Secrets and raw payloads are redacted.
- Every required failure path has direct evidence.
- Rust 1.89 and the lockfile were honoured.
- The worker note reports actual commands and repository state.

## Definition of done

A Rust task is complete only when the requested behaviour exists, accepted
ordering and trust boundaries remain intact, invalid states are prevented or
rejected explicitly, required negative branches have focused evidence, no
unrelated semantic/dependency change is hidden, Rust 1.89 verification passes,
the complete diff and Git state were inspected, and the worker note is
reproducible and honest.

Compiler success is necessary. It is not sufficient.
