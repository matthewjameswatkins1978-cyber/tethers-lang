# Worker Note

Task: `F8-D5+D6+D10 — Dead Exact-Approval Translation/Resume Layer`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `BLOCKED`

Base commit: `17b60df43b6c32ec8040952e4f7b1a99eb16b1d3`

Implementation checkpoint: `5a728d6956a288cafa69a84d0be50f7ffafaa1ea`

## Requested outcome

Remove the dead exact-approval translation/resume layer D5, D6, and D10 while
retaining the state, fresh-precheck, replay-authority, dispatch, and Trail
contracts at their existing surviving seams. The job requires a final
`just verify-agent` regression before it can be complete.

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
- `just verify-agent` at implementation checkpoint — BLOCKED at full `cargo test`: packet (0.8s), formatter (1.1s), and cargo check (4.1s) passed; then 1329 passed, 5 failed, 2 ignored because the five retained-engine tests panic with `engine binary not found; build with opam exec -- dune build`.
- `git diff --check` — PASS before the implementation checkpoint.

## Discoveries

- This worktree has no built OCaml engine binary for the retained-engine Rust
  tests. The F8-D5+D6+D10 packet names no authorised absolute OCaml switch or
  current-worktree engine-build command, so worktree safety rules prohibit
  inferring or borrowing a switch from another checkout.
- Because `verify` failed at `cargo test`, the remaining `verify-agent`
  dependencies (toolchain, dependency-policy/advisory, and nextest gates) did
  not run.

## Remaining risks

- The source checkpoint has focused evidence and the intended warning reduction,
  but it is not completion evidence until the final umbrella regression runs in
  a current-worktree environment with the authorised engine binary.

## Smallest next action

Issue a bounded continuation packet that supplies the explicit OCaml switch and
authorises building the current worktree's engine, then rerun the required final
umbrella regression against `5a728d6956a288cafa69a84d0be50f7ffafaa1ea`.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `tethers-0.1/host-rust/src/application.rs`
- `foundation/f8-d5-d6-d10-exact-approval-cleanup`
