# Tethers R1 external Host boundary

Task: `TETHERS R1 / External Host Boundary & Resolve Proof`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `IN_PROGRESS`

Base commit: `3739d0c06d51569de985053b764af2dc77fc1f2f`

Implementation checkpoint: `WORKTREE`

## Requested outcome

Prove the smallest host-neutral Tethers boundary for Resolve01. Tethers must
produce a deterministic side-effect-free `tethers.plan/1` proposal while an
external Host owns authority, policy, approval, execution identity, provider,
outcome, replay, recovery, and its own ledger.

## Changes made

- Replaced the completed R0 task packet with the R1 control packet and explicit
  implementation scope.
- Added `docs/architecture/TETHERS_R1_EXTERNAL_HOST_BOUNDARY.md` describing the
  existing Plan contract, ownership split, and exclusions.
- Added `tethers-0.1/scripts/test-r1-external-host-boundary.ps1`, a repository-
  owned conformance harness using the existing J14 `fixture.ping` scenario.
- No production Rust, OCaml, Core, Plan, provider, replay, outcome, Resolve,
  Lantern, or P0-P4b implementation files were changed.

## Decisions and assumptions

The existing `tethers plan` command is sufficient. Its `plan_only` path uses
the real Core engine with empty live-provider availability and stops before
policy, replay, execution Trail mutation, provider launch, and dispatch. The
external-host portion is deliberately a tiny local test simulation, not a new
protocol or Host framework. A Tethers `action_id` is retained as proposal
identity; the simulation creates a distinct Resolve-owned execution/effect
identity.

The explicit supported switch used for verification was:
`D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`.

## Evidence

- Fresh R1 worktree started at `3739d0c06d51569de985053b764af2dc77fc1f2f`,
  equal to fetched `origin/main`.
- R1 harness: 6 cases passed, 87 assertions.
- Harness proof: identical Plan evidence, zero Tethers provider calls/effects,
  external ALLOW one call, ASK before approval zero calls, ASK resumed after
  explicit approval exactly once, DENY zero calls, and tampered Plan evidence
  rejected before the external marker/provider boundary.
- Rust Plan tests: 4 passed, 0 failed.
- `cargo check --all-targets --all-features --locked`: passed.
- `just verify`: PASS, including current engine provenance, OCaml tests, Rust
  and cross-language tests, MCP transcript suite, protocol fixture sanity, and
  compatibility corpus.

## Discoveries

The Plan action contract names the operation `capability`, not
`capability_name`; the harness validates the repository's actual stable shape.
The existing fixture provider configuration remains in the trusted runtime
input, but the Plan path never launches it and reports zero invocations.

## Remaining risks

The R1 proof does not implement live Resolve guard admission, transport,
durable coordination, outcome delivery, or recovery. Those are later Resolve
work and must not be imported into Tethers or the reference Host Plan seam.
The repository retains the known duplicate-target warning as accepted debt;
the warning ratchet passed without adding a new warning class.

## Smallest next action

Complete independent review and the final scoped diff/status gate. If accepted,
push this Tethers branch for review; Resolve implementation remains a separate
follow-on task.

## References

- `docs/architecture/TETHERS_HOST_ARCHITECTURE.md`
- `docs/decisions/ADR_HOST_EXECUTION_OWNERSHIP.md`
- `docs/recovery/TETHERS_RESOLVE_RECLASSIFICATION.md`
- `tethers-0.1/host-rust/src/plan_command.rs`
- `tethers-0.1/host-rust/src/host_execution.rs`
- `tethers-0.1/scripts/test-r1-external-host-boundary.ps1`
