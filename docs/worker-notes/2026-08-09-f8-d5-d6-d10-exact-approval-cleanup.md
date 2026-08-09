# Worker Note

Task: `F8-D5+D6+D10 — Dead Exact-Approval Translation/Resume Layer`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `17b60df43b6c32ec8040952e4f7b1a99eb16b1d3`

Implementation checkpoint: `5a728d6956a288cafa69a84d0be50f7ffafaa1ea`

## Requested outcome

Removed the dead exact-approval translation/resume layer D5, D6, and D10 while
retaining the state, fresh-precheck, replay-authority, dispatch, and Trail
contracts at their existing surviving seams. The repaired local engine build
allowed the final `just verify-agent` regression to complete.

## Changes made

- `tethers-0.1/host-rust/src/application.rs`
  - Removed `HumanApprovalDecision` and
    `record_human_approval_decision` (D5/D6).
  - Removed `resume_and_execute_exact_approval` (D10).
  - Retained `resume_and_execute_exact_approval_with_authority` and the
    test-only replay adapter; the former remains the execution seam that owns
    fresh precheck, replay admission, approval consumption, dispatch, and Trail
    ordering.
  - Moved approved-state tests directly to `ApprovalStore::decide`, retaining
    explicit authorisation Trail records where the test asserts the full event
    order.
  - Moved the missing-root replay test to the retained authority seam with an
    explicit `FileReplayAuthority::new(None)`.
  - Retained the Trail-failure invariant at `precheck_exact_approval`: a fresh
    deny invalidates the approved record before the injected invalidation Trail
    write fails.

## Decisions and assumptions

- Classification was **DEAD SUBSYSTEM**: all references to D5, D6, and D10
  were definition/test references in `application.rs`; there were no production
  callers in the Rust tree.
- The direct `ApprovalStore::decide` migration keeps terminal denial and
  cancellation state-transition coverage without recreating a host translation
  wrapper.
- The retained `_with_authority` seam is deliberately different from D10 and
  remains protected by its explicit `#[allow(dead_code)]` testable-architecture
  designation.

## Evidence

- `rg "HumanApprovalDecision|record_human_approval_decision|fn resume_and_execute_exact_approval\\(" tethers-0.1/host-rust --type rust` — PASS: zero matches.
- `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked` — PASS: exactly 8 remaining production-library warnings (D7-D9 and D11-D15); no D5/D6/D10 warning.
- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all` — only `tethers-0.1/host-rust/src/application.rs` changed; immediate diff inspected.
- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check` — PASS.
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked j05_` — PASS: 4 J05 tests.
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked j09_runtime_39_approved_ask_missing_root_consumes_zero_approvals` — PASS: 1 test.
- `cargo clippy --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked` — PASS (existing non-F8 Clippy warnings remain).
- First `just verify-agent` attempt — correctly exposed the local missing-engine prerequisite: packet (0.8s), formatter (1.1s), and cargo check (4.1s) passed before five retained-engine tests could not find the executable.
- `pwsh -NoProfile -File .github/scripts/check-tethers-toolchains.ps1 -OcamlSwitchPath "D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml"` — PASS: the existing switch is OCaml 5.5.0, Dune 3.24.0, Yojson 2.2.2, and matches the pinned lock/toolchain contracts.
- `opam exec --switch="D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" -- dune build` from this worktree's `tethers-0.1/engine-ocaml` — PASS: regenerated ignored `_build/default/bin/main.exe` and `tethers_mcp_main.exe`; no tracked source/configuration changed.
- Final `just verify-agent` — PASS (103s): packet checker, formatter, full Cargo check/test, Rust toolchain check, dependency licences/bans/sources, dependency advisories, and nextest. Nextest: 1592 passed, 2 skipped.
- `git diff --check` — PASS before the implementation checkpoint.

## Discoveries

- The source worktree can lack the ignored engine `_build` output while the
  existing pinned directory switch remains valid. Rebuilding the current
  worktree through that switch restores the retained-engine Rust test
  prerequisite without changing tracked product files.

## Remaining risks

- None known within Job A scope. D7-D9 and D11-D15 remain intentionally
  unresolved for their separately classified jobs.

## Smallest next action

Begin Job B's independent local-notification seam classification from this
accepted Job A tip.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `tethers-0.1/host-rust/src/application.rs`
- `foundation/f8-d5-d6-d10-exact-approval-cleanup`
