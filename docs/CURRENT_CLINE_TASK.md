# Check Command Provider Server-Name Bugfix

Control contract: `1`

Status: `COMPLETE`

Task colour: `Red`

Owner: `Check Provider Server-Name Bugfix Agent`

Route: `Narrow bugfix — awaiting Lucy review`

Base commit: `7c9f846cf5c7681a919f321faf42657c386d99ca`

Implementation checkpoint: `ed786efbd156bbb4850a5c95077cae226eac5dcb`

Worker note: `docs/worker-notes/2026-08-15-check-server-name-bugfix.md`

Updated: 2026-08-15

## Objective

Fix the Tethers `check` command so MCP provider initialization validates the
provider against the trusted capability manifest binding's `server_name`, not
against the provider's configured identity. Add a regression test proving
provider identity and MCP server name may legitimately differ.

## Relevant background and existing behaviour

- `tethers-0.1/host-rust/src/check_command.rs` initialized MCP providers using
  the provider config identity (`stdio.provider_config.identity`) as the
  expected MCP server name.
- Provider identity (host/configuration identity for selecting/tracking the
  provider) and MCP server name (reported by MCP initialize and constrained by
  the trusted manifest binding) are distinct concepts.
- The normal host run path (`host_execution.rs` `launch_and_initialize_provider`)
  derives expected server name from
  `verified_manifest.manifest().binding.server_name`.
- `ManagedProvider::initialize` enforces the reported server name against the
  expected value and fails closed on mismatch.

## Required behaviour

1. `check` must use the trusted manifest binding `server_name`, not provider
   identity, when initializing MCP providers.
2. Expected server name derivation must mirror the normal host run path.
3. Provider identity must remain unchanged and continue serving its own purpose.
4. Existing server-name validation must remain enforced.
5. Add one focused regression test proving identity and server_name may differ.
6. Preserve the negative trust behaviour when a reported server name does not
   match the trusted manifest binding.

## Relevant components

- `tethers-0.1/host-rust/src/check_command.rs` (production fix and tests)

## Frozen decisions and invariants

- Provider identity and MCP server name remain distinct concepts.
- Do not make them equal merely to satisfy tests.
- Do not weaken or remove server-name validation.
- Do not accept arbitrary reported server names.
- Use trusted manifest-derived server-name evidence.
- Do not invent a second server-name rule; mirror the existing host-run model.
- No production semantic changes outside the check path.

## Acceptance criteria

1. `check` no longer uses provider identity as expected MCP server_name.
2. Expected server name comes from trusted manifest binding data consistent
   with the normal run path.
3. Provider identity remains unchanged and continues serving its own purpose.
4. Existing server-name validation remains enforced.
5. Regression proves identity and server_name may legitimately differ.
6. Wrong reported server_name still fails if covered by existing/narrow test.
7. No unrelated production changes.
8. Existing tests remain green.

## Required verification

1. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- j13a_check_provider --test-threads=1`
2. `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml -- --check`
3. `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml`
4. `cargo check --locked --manifest-path tethers-0.1/host-rust/Cargo.toml`
5. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- --test-threads=1`
6. `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features`
7. `git diff --check`
8. `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`

## Forbidden changes

- No provider protocol, manifest schema, config schema, source language, replay,
  Trail, Runtime Plan, concurrency, or host execution semantic changes
- No C5 salvage files
- No force push, rebase, or merge of main

## Stop conditions

- If the normal run path derives server name by a conflicting rule.
- If a provider can contain trusted manifests with conflicting
  binding.server_name and no deterministic rule exists.
- Discovered production defect.

## Expected pre-existing changes

- `WORKTREE.md`
- `docs/CANONICAL_FORMAT_V2_SPEC_DRAFT.md`
- `docs/performance/CORE_PHASE_A_IMPLEMENTATION_PACKET.md`
- `docs/performance/R1_PERFORMANCE_PROOF.md`
- `docs/performance/core-phase-a/`
- `docs/performance/r1/`
- `docs/worker-notes/2026-08-12-c-core-cheap-structural-fixes.md`
- `docs/worker-notes/2026-08-14-c2a1-together-semantic-bridge.md`
- `scripts/assert-worktree.ps1`
- `tethers-0.1/engine-ocaml/bin/tethers_cb3t_tie_audit.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_rank_avalanche.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_v2_canon_label.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_v2_canon_label_test.ml`
