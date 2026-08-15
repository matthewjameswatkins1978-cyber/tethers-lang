# C3-A4 — External Bounded-Concurrency Configuration

Control contract: `1`

Status: `COMPLETE`

Task colour: `Red`

Owner: `C3-A4 Configuration Integration Agent`

Route: `C3-A4 configuration/default/validation — awaiting Lucy review`

Base commit: `ecb9a9bb4f68731f378f055984de91f5399ea5a2`

Implementation checkpoint: `7764ca9921e23bafbb37487f5bc72157b2d575d9`

Worker note: `docs/worker-notes/2026-08-15-c3-a4-concurrency-config.md`

Updated: 2026-08-15

- C3-A1, A2, A3 accepted by Lucy.
- This packet owns ONLY external configuration/default/validation wiring.
- Scheduler semantics are frozen.
- C3-V1 is NOT authorised.

## Objective

Expose `max_active_together_invocations` as host configuration with sensible defaults and validation.

## Relevant background and existing behaviour

- C3-A1 introduced the bounded launch window `max_active_together_invocations` and C3-A2 proved the deadline isolation and G1 boundaries.
- C3-A3 proved the failure boundaries and corrected the audit_failure contamination defect.
- `RuntimeConfig` is strict `#[serde(deny_unknown_fields)]`.
- `PreparedRuntime` is immutable runtime state.
- `execute_group_concurrent` currently defaults to `member_indexes.len().max(1)`.
- `execute_group_concurrent_with_limit` is the accepted bounded engine.

## Required behaviour

1. Add ONE optional top-level runtime configuration field: `max_active_together_invocations`.
2. Freeze default: `DEFAULT_MAX_ACTIVE_TOGETHER_INVOCATIONS = 2`.
3. Validate: N >= 1, reject 0, use existing `RuntimeConfig InvalidValue` error model.
4. Backward compatibility: existing configs that omit the field must continue to parse.
5. `PreparedRuntime` must carry the validated value explicitly with read-only accessor.
6. `execute_group_concurrent` wrapper must use `service.runtime.max_active_together_invocations()` instead of `member_indexes.len().max(1)`.

## Relevant components

- `tethers-0.1/host-rust/src/runtime_config.rs`
- `tethers-0.1/host-rust/src/configured_runtime.rs`
- `tethers-0.1/host-rust/src/host_execution.rs`

## Frozen decisions and invariants

- Scheduler semantics are frozen.
- C3-V1 is NOT authorised.
- No new terminal taxonomy.
- No environment variable fallback, no CLI override, no config hot reload.
- No Trail schema changes for a configuration knob.

## Acceptance criteria

1. Omitted field defaults to 2.
2. Explicit 1 accepted.
3. Explicit 2 accepted.
4. Larger value (e.g. 8) accepted.
5. Zero rejected with `RuntimeConfigErrorCode::InvalidValue` and field `/max_active_together_invocations`.
6. Wrong type rejected (e.g. "2", 2.5, null).
7. Default materialises in `PreparedRuntime` as 2.
8. Explicit value materialises in `PreparedRuntime`.
9. Physical default-N=2 proof using `execute_group_concurrent` wrapper.
10. Physical explicit-N=1 proof using `execute_group_concurrent` wrapper.

## Required verification

1. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_a4 --test-threads=1`
2. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_a3 --test-threads=1`
3. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_a2 --test-threads=1`
4. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_a1 --test-threads=1`
5. `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml -- --check`
6. `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml`
7. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- --test-threads=1`
8. `git diff --check`
9. `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`

## Forbidden changes

- Scheduler semantics
- C3-A1/A2/A3 test semantics
- C3-V1
- New terminal taxonomy
- Worker pools, semaphore, queue settings
- Environment variable, CLI override, config hot reload
- Trail schema changes
- New source-language syntax
- Host-global or provider-aware scheduling

## Stop conditions

- A required production change outside the authorised files is needed.
- A frozen design invariant cannot be satisfied without redesign.
- Repeated failure rule: 2 materially similar failed attempts on the same underlying problem.

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

## Requested outcome

1. Add ONE optional top-level runtime configuration field `max_active_together_invocations` with serde default 2, validated N >= 1.
2. Wire the validated value through `PreparedRuntime` with read-only accessor.
3. Update `execute_group_concurrent` wrapper to use the configured value.
4. Prove physically that the default N=2 and explicit N=1 control real group execution through the production wrapper.
5. All existing C3-A1/A2/A3 tests remain green.
