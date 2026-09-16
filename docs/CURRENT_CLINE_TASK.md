# TETHERS R1 - EXTERNAL HOST BOUNDARY & RESOLVE PROOF

Task: `TETHERS R1 / External Host Boundary & Resolve Proof`

Control contract: `1`

Status: `IN_PROGRESS`

Task colour: `Red`

Owner: `Codex`

Route: `Tethers-first boundary proof: establish whether the existing read-only
Plan interface is sufficient for an external Host such as Resolve01; add only
the smallest repository-owned conformance evidence and documentation required.
Do not alter Resolve, Lantern, Core semantics, Tether syntax, provider
execution, replay, outcome, or the historical P0-P4b implementation.`

Base commit: `3739d0c06d51569de985053b764af2dc77fc1f2f`

Worker note: `docs/worker-notes/2026-09-16-tethers-r1-external-host-boundary.md`

Suggested branch:

`codex/tethers-r1-external-host-boundary`

## Objective

Prove the smallest correct external Host seam. Tethers Core must provide a
deterministic, side-effect-free `tethers.plan/1` Plan and Capability contract;
an external Host such as Resolve01 must retain ownership of policy, authority,
execution identity, provider invocation, outcome, replay, recovery, and its own
ledger. Tethers must not grant permission or execute the provider.

## Relevant background and existing behaviour

R0 was merged at the base commit above. It establishes that Tethers Core owns
deterministic semantics and planning, Capability contracts own stable operation
meaning, and a Host owns application execution lifecycle, policy, approvals,
recovery, and provider invocation. The Tethers reference Host remains a valid
Host implementation but is not mandatory for an external Host.

The existing public `tethers plan` command emits versioned `tethers.plan/1`
data. Its `plan_only` path validates and evaluates the selected Tether through
the real Core engine with empty live-provider availability, then stops before
policy, replay mutation, Trail execution entries, provider launch, and
dispatch. The existing `fixture.ping` scenario is the harmless deterministic
test capability for this proof.

Historical P0-P4b Resolve integration remains preserved history and is not
deleted, reverted, or reused as the external execution owner. No Resolve or
Lantern implementation is in scope.

## Required behaviour

1. Establish fresh fetched `origin/main` and exact R1 starting SHA.
2. Confirm the existing Plan interface and Core boundary before editing.
3. Prove deterministic Plan production for identical source, event, Facts,
   trusted Capability projection, and version inputs.
4. Prove that the Plan path makes zero reference-Host provider calls, performs
   no durable dispatch, mutates no replay state, writes no execution Trail, and
   makes no Resolve transport or guard calls.
5. Prove the external-host lifecycle with a harmless local Resolve-owned test
   executor: the same Plan is consumed under ALLOW, ASK-before-approval, and
   DENY; only the explicitly approved ALLOW path invokes the Resolve-owned
   fake provider.
6. Prove malformed/tampered Plan input fails closed before that executor can
   invoke a provider.
7. Preserve the distinction between Tethers Action identity and external Host
   execution/effect identity.
8. Document the versioned contract, ownership, evidence, exclusions, and any
   extraction or limitation honestly.

## Relevant components

- `tethers-0.1/host-rust/src/plan_command.rs`
- `tethers-0.1/host-rust/src/host_execution.rs`
- `tethers-0.1/host-rust/src/configured_runtime.rs`
- `tethers-0.1/scripts/test-tethers-0.6-plan.ps1`
- `tethers-0.1/scenarios/j14-complete-local/`
- `docs/architecture/TETHERS_HOST_ARCHITECTURE.md`
- `docs/decisions/ADR_HOST_EXECUTION_OWNERSHIP.md`
- `docs/recovery/TETHERS_RESOLVE_RECLASSIFICATION.md`

## Frozen decisions and invariants

Tethers Core is deterministic and side-effect-free. A Plan is a proposal, not
permission. The external Host owns authority, policy, approval, execution,
provider invocation, outcome, replay, recovery, and its own durable ledger.
The reference Host's read-only Plan adapter may be used to expose current Core
semantics, but its execution lifecycle is not imported into Resolve.

The R1 proof uses the existing versioned `tethers.plan/1` contract and the
harmless `fixture.ping` capability. No new protocol, daemon, service, Host
framework, Resolve transport, Resolve database, or Core concept is authorised.
P0-P4b remains preserved historical implementation and is not rewritten.

## Acceptance criteria

1. Fresh current `origin/main` and exact starting SHA are recorded.
2. Existing Plan interface is shown sufficient or the smallest neutral gap is
   documented without inventing a framework.
3. Identical Tethers inputs produce identical Plan evidence.
4. Plan evidence proves zero provider calls, durable dispatch, replay mutation,
   execution Trail entries, and Resolve calls.
5. A local external-Host simulation consumes the same Plan under ALLOW, ASK,
   and DENY with only approved ALLOW invoking its own harmless provider.
6. Malformed or tampered Plan evidence is rejected before external execution.
7. Tethers Action identity remains distinct from external execution identity.
8. Documentation, worker note, task packet, tests, diff review, and Git status
   are complete and honest.

## Authorised files

- `docs/CURRENT_CLINE_TASK.md`
- `docs/architecture/TETHERS_R1_EXTERNAL_HOST_BOUNDARY.md`
- `docs/worker-notes/2026-09-16-tethers-r1-external-host-boundary.md`
- `tethers-0.1/scripts/test-r1-external-host-boundary.ps1`
- narrowly related R1 test fixtures or documentation only if required by the
  conformance harness

## Forbidden changes

No Resolve repository changes; no Lantern changes; no OCaml/Core production
changes; no Tether syntax or Plan semantics changes; no provider execution,
policy, approval, replay, durable intent, outcome, Result Anchor, or Trail
execution changes; no live Resolve transport, guard client, database, or
execution framework; no generic Host SDK; no deletion or rewriting of P0-P4b;
no unrelated cleanup; no direct main update; no force-push.

Production Rust changes are permitted only if the conformance proof identifies
a genuinely host-neutral boundary defect that cannot be demonstrated from the
existing public Plan seam. Otherwise this is an evidence-and-documentation
task.

## Stop conditions

Stop and document extraction rather than inventing if the boundary requires
reference-Host state, a new protocol or execution framework, Core/Plan semantic
changes, Tethers-owned provider execution, Resolve transport/database access,
or a generic Host abstraction. Stop if malformed or untrusted Plan data cannot
be rejected before external execution. Stop after two materially similar
failed implementation attempts against the same boundary issue.

## Required verification

Run the R1 conformance harness, the existing focused Plan regression, task
packet checker, documentation/link checks, `cargo fmt --all -- --check`,
relevant Rust checks, `just verify` where the current environment permits it,
secret scan, `git diff --check`, and a complete diff/status review. Record any
OCaml/toolchain limitation without calling dependent evidence a pass.

## Expected pre-existing changes

None in the fresh R1 worktree. Other occupied worktrees are preserved and are
not imported into this task.
