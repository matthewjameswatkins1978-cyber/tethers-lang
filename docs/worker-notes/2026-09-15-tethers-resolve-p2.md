# Tethers x Resolve01 P2 Worker Note

Task: `TETHERS x RESOLVE01 / P2 - Guard Admission Boundary`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `Codex`
Status: `COMPLETE`
Base commit: `fab258663af363b6e5902ea3bbe5d3dd8de61678`
Implementation checkpoint: `be2606e3efb68be07a9a79283df9b6a922e2f3b2`

## Requested outcome

Implement one bounded, non-transport Resolve guard-admission seam over the
accepted P1 preparation evidence. Resolve may reject or admit coordination, but
Tethers remains the sole permission and execution authority.

## Starting state

- Starting accepted main SHA: `fab258663af363b6e5902ea3bbe5d3dd8de61678`
- Branch: `codex/tethers-resolve-p2-guard-admission`
- Worktree: `D:\The Next Thing\Tethers Lang - Resolve P2 Guard Admission`
- Owner: Codex

## Scope

Rust host guard-admission boundary, focused tests, architecture note, current
task packet and this worker note. No OCaml/Core, syntax, Plan, policy, replay,
provider semantics, Resolve transport or database changes.

## Initial evidence

- Fresh branch was created from fetched `origin/main`.
- Canonical main and `origin/main` matched at the starting SHA.
- Repository tool diagnostic passed.
- Inherited P1 packet checker passed before the P2 packet was installed.

## Changes made

- Added the opaque `ResolveGuardRef`, closed `ResolveGuardAdmission` result
  set, single `ResolveGuardAdapter`, and controlled current-authority context
  in `resolve_guard.rs`.
- Added exact guard-admission evidence to the existing sealed Trail boundary.
- Added explicit guarded serial and Together host routes. Ordinary execution
  continues to pass no guard context and is unchanged.
- Added distinct `GuardRejected` and `GuardIndeterminate` host/CLI outcomes;
  existing provider and Result Anchor outcome taxonomies are unchanged.
- Added focused serial and Together tests proving durable-intent ordering and
  zero provider calls on rejection/failure, plus one admitted serial call.
- Added the P2 architecture note and replaced this packet with the bounded P2
  task contract.

## Implementation checkpoint

The first coherent implementation was committed as
`be2606e3efb68be07a9a79283df9b6a922e2f3b2`.

## Decisions and assumptions

- Guarded mode must be an explicit host-owned execution choice and remain
  disabled for ordinary execution.
- The adapter must expose only `Admitted`, `Rejected` and `Indeterminate`.
- The existing replay/durable-intent and provider boundaries remain the only
  execution authorities.
- P2 deliberately preserves the frozen order: replay admission, durable
  intent, guard admission, then deadline/G1/provider. An unarmed intent after a
  guard refusal remains the existing fail-closed manual-resolution recovery
  state; P2 does not invent replay cleanup semantics.
- Before editing, inspect whether current Trail and configuration contracts can
  carry bounded guard evidence without a schema mutation.

## Evidence

- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all`: PASS.
- `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked`: PASS.
- `just check`: PASS with `-D warnings`.
- `just warning-ratchet`: PASS; only the existing duplicate-target warning is
  reported by the compiler outside the ratchet baseline.
- `cargo test ... p2_ --all-features --locked`: 4 focused tests PASS.
- `pwsh -NoProfile -File scripts/check-compatibility-corpus.ps1`: PASS.
- Explicit toolchain diagnostic with the authorised existing switch:
  `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`: PASS.
- `pwsh -NoProfile -File scripts/verify-tethers.ps1 -OcamlSwitchPath
  'D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml'`: PASS after a
  first overlapping-Dune attempt failed with `Error: RPC server not running.`
  The retry built the current engine from the task source SHA and passed the
  complete OCaml/Rust, fixture, MCP and compatibility aggregate.
- Independent Gemini review completed with model `gemini-3.8-flash`, thinking
  level `high`, status `completed`, complete response. Its crash/replay and
  optional-context concerns were checked against the frozen P2 order and
  existing replay contract; no change was made that would violate P2's
  explicit no-replay-semantics-change boundary. The response did reinforce
  documenting fail-closed manual resolution and testing malformed boundaries.

## Discoveries

The accepted P1 module already rebuilds current capability, schema, scope,
policy and approval evidence. The serial and Together host paths expose the
existing durable-intent/replay/provider seams that must be integrated without a
second execution boundary.

## Remaining risks

The guarded route is intentionally host-selected and has no configuration or
wire-protocol selector in P2. The existing strict runtime configuration schema
therefore remains unchanged. A future consumer must select the explicit host
function; ordinary execution cannot opt in from manifest or provider data.

## Smallest next action

The implementation checkpoint is committed. The clean-tree authoritative
verification and final branch publication remain the closeout steps.

## References

- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P0_SEAM_AUDIT.md`
- `docs/architecture/TETHERS_RESOLVE01_BRIDGE_P1_PREPARATION_PROOF.md`
- `tethers-0.1/host-rust/src/resolve_guard.rs`

## Verification

## Stop conditions encountered

None at task start.
