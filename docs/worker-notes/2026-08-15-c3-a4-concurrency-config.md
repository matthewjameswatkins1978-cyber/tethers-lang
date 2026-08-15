# Worker Note — C3-A4 External Bounded-Concurrency Configuration

Task: `C3-A4 — External Bounded-Concurrency Configuration`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `C3-A4 Configuration Integration Agent`
Status: `COMPLETE`
Base commit: `ecb9a9bb4f68731f378f055984de91f5399ea5a2`
Implementation checkpoint: `7764ca9921e23bafbb37487f5bc72157b2d575d9`

## Objective

Expose `max_active_together_invocations` as host configuration with sensible defaults and validation, wiring it through `RuntimeConfig` → `PreparedRuntime` → `execute_group_concurrent` wrapper.

## Requested outcome

1. Add ONE optional top-level runtime configuration field `max_active_together_invocations` with serde default 2, validated N >= 1.
2. Wire the validated value through `PreparedRuntime` with read-only accessor.
3. Update `execute_group_concurrent` wrapper to use the configured value.
4. Prove physically that the default N=2 and explicit N=1 control real group execution through the production wrapper.
5. All existing C3-A1/A2/A3 tests remain green.

## Changes made

### `tethers-0.1/host-rust/src/runtime_config.rs`

- Added `DEFAULT_MAX_ACTIVE_TOGETHER_INVOCATIONS` constant (2).
- Added `default_max_active_together_invocations()` serde default function.
- Added `max_active_together_invocations: usize` field to `RuntimeConfig` with `#[serde(default = "default_max_active_together_invocations")]`.
- Added `validate_max_active_together_invocations()` validation: rejects 0 with `RuntimeConfigErrorCode::InvalidValue` and field `/max_active_together_invocations`.
- Added 8 focused tests (A4.1–A4.6c): omission defaults to 2, explicit 1/2/larger accepted, zero rejected, wrong type (string/float/null) rejected.

### `tethers-0.1/host-rust/src/configured_runtime.rs`

- Added `max_active_together_invocations: usize` field to `PreparedRuntime`.
- Added `pub fn max_active_together_invocations(&self) -> usize` read-only accessor.
- Wired `prepare_runtime` to populate the field from `loaded.config.max_active_together_invocations`.
- Added 2 focused tests (A4.7–A4.8): default materialises as 2, explicit value materialises.

### `tethers-0.1/host-rust/src/host_execution.rs`

- Changed `execute_group_concurrent` wrapper to use `service.runtime.max_active_together_invocations()` instead of `member_indexes.len().max(1)`.
- Added `c3a4_harness_with_config` test helper: builds a C3A1-style harness with explicit `max_active_together_invocations` in config.
- Added `c3a4_run_group_via_wrapper` test helper: calls `execute_group_concurrent` (the wrapper), NOT `execute_group_concurrent_with_limit` directly.
- Added 2 physical proof tests (A4.9–A4.10): default-N=2 proof (A+B active simultaneously, C waits), explicit-N=1 proof (A launches, B/C wait, sequential).

## Frozen decisions and invariants

- Scheduler semantics are frozen.
- C3-V1 is NOT authorised.
- No new terminal taxonomy.
- No environment variable fallback, no CLI override, no config hot reload.
- No Trail schema changes for a configuration knob.

## Decisions and assumptions

- Used `#[serde(default = "default_max_active_together_invocations")]` for backward compatibility: existing configs that omit the field continue to parse with default 2.
- Validation rejects 0 with `RuntimeConfigErrorCode::InvalidValue` and exact field pointer `/max_active_together_invocations`.
- No arbitrary maximum enforced; actual launch count is naturally bounded by group width.
- The internal `.max(1)` guard in `execute_group_concurrent_with_limit` is preserved as a defensive measure; external config validation rejects 0 before `PreparedRuntime` exists.
- Physical proof tests use `execute_group_concurrent` (the wrapper), NOT `execute_group_concurrent_with_limit` directly, to prove the configuration value reaches the production wrapper.

## Evidence

- `cargo test -- c3_a4 --test-threads=1`: PASS (12/12 tests passed)
- `cargo test -- c3_a3 --test-threads=1`: PASS (7/7 tests passed)
- `cargo test -- c3_a2 --test-threads=1`: PASS (4/4 tests passed)
- `cargo test -- c3_a1 --test-threads=1`: PASS (3/3 tests passed)
- `cargo fmt -- --check`: PASS (clean formatting)
- `cargo check`: PASS (compiles clean)
- `cargo test -- --test-threads=1`: PASS (1538 passed, 0 failed, 2 ignored)
- `git diff --check`: PASS (LF→CRLF warnings only)
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`: PASS (control-v1/IN_PROGRESS)

## Discoveries

- The `C3A1GroupHarness` struct fields are private but accessible within the same test module, allowing `c3a4_harness_with_config` to construct instances directly.
- The barrier script requires `peer-count` to be set via `set_peer_count()` for N=1 mode; without it, the provider never enters the barrier.
- The `Mutex` import in `dispatch.rs` is unused after the previous C3-A3 synchronisation fix; this is a pre-existing condition not introduced by this task.

## Remaining risks

- None identified. The configuration plumbing is backward-compatible and the existing `execute_group_concurrent_with_limit` internal `.max(1)` guard remains as a defensive measure.

## Smallest next action

- Await Lucy review and acceptance of C3-A4 on published branch `feature/c3-concurrency-config`.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/concurrency/C3_BOUNDED_CONCURRENCY_DESIGN.md`
- `tethers-0.1/host-rust/src/runtime_config.rs`
- `tethers-0.1/host-rust/src/configured_runtime.rs`
- `tethers-0.1/host-rust/src/host_execution.rs`
