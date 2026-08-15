# Worker Note: C3-A1 Minimal Bounded Launch Window

Task: `C3-A1 — Minimal Bounded Launch Window`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `C3-A1 Implementation Agent`

Status: `COMPLETE`

Base commit: `d01e41ec00c89b548e6641e9c7661c7ffd2ccfe8`

Implementation checkpoint: `e2ca3676e88ba69e3e13b6ef1ca3d0d82083a0bf`

## Requested outcome

Implement the minimal bounded launch window for Together group execution as frozen
by accepted design C3-D1/D2, bounding active provider invocations to an
internal/test-injectable parameter `max_active_together_invocations: usize` (N >= 1)
without altering Together source semantics, Runtime Plan semantic identity,
member SemanticPosition, replay G0/G1/G2 meaning, Trail truthfulness, terminal
taxonomy, deterministic final non-success selection, or sequential Action behaviour.

## Changes made

- `tethers-0.1/host-rust/src/host_execution.rs`:
  - Added internal helper functions `count_active_members` and `has_prepared_members`
    to derive active capacity directly from `GroupMemberState` (`Launched` and
    `Transitioning` states count as active; `PreparationTerminal`, `Prepared`, and
    `Terminal` do not).
  - Parameterised group concurrency via `execute_group_concurrent_with_limit` accepting
    `max_active_together_invocations: usize` (N >= 1).
  - Wrapped existing `execute_group_concurrent` to default to full group width
    (`member_indexes.len().max(1)`), preserving A3a full-width behavior for all
    existing production callers and tests.
  - Implemented bounded launch loop inside `std::thread::scope`:
    - Stage A prepares all members serially in Runtime Plan order (scope, policy,
      capability resolution, provider check, replay admission, G0 intent, Trail intent).
    - Stage B admits earliest semantic-order `Prepared` waiting member while
      `active_count < max_active`, establishing per-member monotonic deadline start
      (`clock.now()`), calculating remaining duration, publishing G1 `publish_armed()`,
      and spawning worker in scoped thread.
    - Stage C receives `WorkerResult` on coordinator mpsc channel, executes complete
      `execute_boundary_invoke_only` (durable `OutcomeEntry`, G2 `publish_terminal`,
      presentation updates, Result Anchor), and transitions `GroupMemberState` to
      `Terminal`.
    - Capacity is released exclusively upon `GroupMemberState` transition to `Terminal`.
    - Fail-closed launch halt (`launches_halted = true`) prevents any subsequent
      launches on trusted Stage C persistence failure or G2 publication failure.
    - Stage D evaluates `GroupJoin` and semantic non-success selection after all
      members reach terminal state.
  - Added `C3A1GroupHarness` and 3 focused barrier tests proving:
    - `c3_a1_n1_limits_active_invocations_to_at_most_one`: N=1 never exceeds 1 active provider.
    - `c3_a1_n2_limits_active_invocations_to_at_most_two_and_reaches_two`: N=2 reaches 2 active providers without exceeding 2.
    - `c3_a1_full_width_preserves_full_overlap`: N=3 with 3 members preserves full physical overlap.
- `docs/concurrency/C3_BOUNDED_CONCURRENCY_DESIGN.md`:
  - Updated document header status to record Lucy acceptance.
- `docs/CURRENT_CLINE_TASK.md`:
  - Updated active task packet to C3-A1 specification and completion state.

## Decisions and assumptions

- Capacity is strictly derived from `GroupMemberState` without an independent mutable counter.
- Per-member deadline establishment and G1 publication occur at launch time, isolating queue wait from provider timeout.
- Capacity is released only after complete Stage C execution and transition to `Terminal`.
- Fail-closed latch stops new launches immediately on trusted persistence or G2 failure.
- `execute_group_concurrent` preserves A3a full-overlap behavior by defaulting to group width.

## Evidence

- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml -- --check` passed with 0 errors.
- `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml` passed with 0 errors.
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- c3_a1 --test-threads=1` passed: 3 passed, 0 failed.
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- --test-threads=1` passed: 1515 unit tests + 20 integration test binaries passed, 0 failed, 2 ignored.
- `git diff --check` passed with 0 whitespace errors.
- `.github/scripts/check-tethers-task-packet.ps1` verified: `PASS task packet consistency (control-v1/COMPLETE): base d01e41e, HEAD ...`

## Discoveries

None.

## Remaining risks

None known within packet scope. C3-A2/A3/A4 remain NOT authorised.

## Smallest next action

Submit C3-A1 implementation to Lucy for architectural review and acceptance.

## References

- `tethers-0.1/host-rust/src/host_execution.rs`
- `docs/concurrency/C3_BOUNDED_CONCURRENCY_DESIGN.md`
- `docs/concurrency/C2_A3_PHYSICAL_CONCURRENCY_DESIGN.md`
