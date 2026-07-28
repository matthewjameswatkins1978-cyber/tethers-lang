# Current Implementation Task

Control contract: `1`

Task: `J11 packet 3 compiled host rejection verification`

Status: `IN_PROGRESS`

Task colour: `Red`

Owner: `Goose`

Route: `Goose - J11 packet 3 in fenced local worktree`

Worker note: `docs/worker-notes/2026-07-28-j11-production-rejection-verification.md`

Base branch: `goose/tethers-final-evidence-skill`

Base commit: `85d8bfa3c9e4816f10c8df5afe00bf150574ec58`

Branch: `goose/j11-production-rejection-verification`

## Objective

Add a debug-build-only compiled host diagnostic route that proves J11 rejection
behaviour through the real production admission components:
`EventAdmissionGate`, `ResultEventQueue`, `drain_result_event_queue`, and
`EventDrainOutcome::apply_to_response`.

This is a controlled diagnostic boundary. It does not prove that the normal
public engine route can currently generate Result Anchors. The normal engine
route remains unchanged and continues to use its existing permission-scope and
replay boundaries.

## Relevant background and existing behaviour

Packet 1 delivered `EventAdmissionGate` in `event_admission.rs`. Packet 2 wired
the gate into the host coordinator, creating `EventDrainOutcome` with
`apply_to_response` and the `drain_result_event_queue` helper. The production
drain uses a closure-based evaluation callback, the real admission gate, and
the real `EventDrainOutcome::apply_to_response` to mutate response JSON. The
gate rejects duplicate event IDs and generation 9+ causally. The drain stops
immediately on first rejection; completed follow-ups before rejection are
preserved.

## Relevant components

- `tethers-0.1/host-rust/src/main.rs` – diagnostic subcommand, `run_event_admission_probe`, tests
- `tethers-0.1/host-rust/src/event_admission.rs` – EventAdmissionGate (not modified)
- `tethers-0.1/host-rust/src/event_queue.rs` – ResultEventQueue (not modified)
- `tethers-0.1/host-rust/src/result_anchor.rs` – ResultAnchor type (not modified)
- `tethers-0.1/scripts/test-host-event-admission.ps1` – new compiled-boundary test

## Frozen decisions and invariants

- Diagnostic route compiled under `#[cfg(debug_assertions)]` only.
- Release builds do not contain the diagnostic route.
- The diagnostic uses the real production `EventAdmissionGate`, `ResultEventQueue`, `drain_result_event_queue`, and `EventDrainOutcome::apply_to_response`.
- No duplication of the drain loop or admission logic.
- No environment-variable activation, hidden request fields, network access, engine invocation, policy evaluation, replay, provider dispatch, Trail activity, filesystem state, or sleep/timing behaviour.
- The evaluation callback is deterministic and never enqueues new anchors.
- Initial event `evt/root` generation 0 is always admitted before scenario queue construction.
- This proves the compiled diagnostic route through the production admission helper. It does not prove normal engine-driven Result Anchor generation through the public runtime.
- Packet 4 owns Trail admission and rejection records. J12 owns runnable configuration and legitimate scope establishment.

## Required behaviour

1. Add a debug-build-only diagnostic subcommand `event-admission-probe <SCENARIO>`.
2. The route and all helper code used only by it must compile under `#[cfg(debug_assertions)]`.
3. A release build must not contain the diagnostic route.
4. Use the existing production `EventAdmissionGate`, `ResultEventQueue`, `drain_result_event_queue`, and `EventDrainOutcome::apply_to_response`.
5. Do not duplicate the drain loop or admission logic.
6. For the `duplicate-initial` scenario: reject `evt/root` generation 1 as duplicate; keep `evt/later` in remaining queue; no `follow_up_evaluations`.
7. For the `duplicate-sibling` scenario: evaluate `evt/first` generation 1; reject duplicate `evt/first`; keep `evt/later` in remaining queue.
8. For the `causal-depth` scenario: reject `evt/deep` generation 9 for causal depth; keep `evt/later` in remaining queue; no `follow_up_evaluations`.
9. For the `clean` scenario: evaluate `evt/a` generation 1 and `evt/b` generation 8 in FIFO order; no rejection; empty remaining queue.
10. Add five focused Rust tests beginning with `j11_packet3_` that exercise the diagnostic function.
11. Create a PowerShell compiled-boundary script `test-host-event-admission.ps1` that builds once and invokes the compiled executable for all four scenarios.
12. Fix the pre-existing ESC byte defect in the `drain_result_event_queue` doc comment.

## Acceptance criteria

1. `cargo fmt --check` passes.
2. `cargo check` shows 9 baseline warnings and 0 new.
3. `cargo check --tests` shows 4 baseline warnings and 0 new.
4. `cargo test j11_packet3_ -- --nocapture` passes 5/5.
5. `cargo test j11_ -- --nocapture` passes 24/24 (19 existing + 5 new).
6. `cargo test event_admission -- --nocapture` passes 15/15.
7. `cargo test` full suite passes 512/512.
8. `cargo clippy --all-targets --all-features` passes with baseline warnings only, 0 new.
9. `cargo build` and `cargo build --release` pass.
10. All existing PowerShell scripts pass: `check-fixtures.ps1`, `test-engine.ps1`, `test-mcp-transcripts.ps1`, `test-host-denial.ps1`, `test-host-execution-failure.ps1`, `test-host-result-follow-up.ps1`, `demo.ps1`.
11. `test-host-event-admission.ps1` passes all four scenarios.
12. `check-tethers-task-packet.ps1` passes.
13. `opam exec -- dune build` in engine-ocaml passes.
14. Control-character scan of all four authorised files shows no unexpected characters.
15. `git diff --check` produces no output.

## Required verification

From `tethers-0.1/host-rust`:

```powershell
cargo fmt --check
cargo check
cargo check --tests
cargo test j11_packet3_ -- --nocapture
cargo test j11_ -- --nocapture
cargo test event_admission -- --nocapture
cargo test
cargo clippy --all-targets --all-features
cargo build
cargo build --release
```

From repository root:

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
pwsh -NoProfile -File tethers-0.1/scripts/check-fixtures.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-engine.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-mcp-transcripts.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-host-denial.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-host-execution-failure.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-host-result-follow-up.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-host-event-admission.ps1
pwsh -NoProfile -File tethers-0.1/scripts/demo.ps1
```

From `tethers-0.1/engine-ocaml`:

```powershell
opam exec -- dune build
```

Also run:

```powershell
git diff --check
git diff --name-status
git status --porcelain=v1 --untracked-files=all
```

## Forbidden changes

- No modification of `event_admission.rs`, `event_queue.rs`, or `result_anchor.rs`.
- No modification of `Cargo.toml`, `Cargo.lock`, OCaml code, protocol files, or existing fixtures.
- No modification of existing PowerShell scripts (except the new one).
- No modification of `.agents/skills/tethers-final-evidence/SKILL.md`.
- No Trail schemas or records.
- No environment-variable activation, hidden request fields, network access, engine invocation, policy evaluation, replay, provider dispatch, or filesystem state.
- No duplication of the drain loop or admission logic.
- No new dependencies.

## Stop conditions

- Any existing J10, J11, or `event_admission` test regresses.
- The diagnostic code compiles into a release build.
- The drain loop or admission logic is duplicated.
- A dependency is added.
- The ESC byte defect re-emerges.
- The diagnostic subcommand uses environment variables, network, or policy.

## Expected pre-existing changes

None. The worktree is clean at base commit `85d8bfa3c9e4816f10c8df5afe00bf150574ec58`.
