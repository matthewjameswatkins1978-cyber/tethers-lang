# Worker Note

Task: `F8-D7+D8+D9 — Dead Local-Notification Host Integration Seam`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `aa01766dc269338b07b4302bc70d6dc9ecaf1037`

Implementation checkpoint: `f138496db8339875971159e50617286d80aea63a`

## Requested outcome

Removed the unused D7-D9 local-notification wrapper chain while retaining the
independently public M5 local-anchor admission coordinator and its direct
integration evidence.

## Changes made

- `tethers-0.1/host-rust/src/application.rs`
  - Removed `submit_local_root_anchor` (D7), `short_event_digest` (D8), and
    `process_local_notification` (D9) as one dead three-function chain.
  - Removed the former production-level `Trail` import and retained it only in
    the existing test modules that call Trail trait methods on `RecordingTrail`.
  - Did not modify `local_anchor.rs`, `tests/m5_local_anchor.rs`, live
    `process_one_event` callers, provider protocol paths, queue behavior, or
    any D11-D15 target.

## Decisions and assumptions

- Classification is **DEAD SUBSYSTEM**: complete Rust searches found D7-D9
  only at their definitions and internal chain calls. No entrypoint, public
  export, test, provider, socket, or command reaches the chain.
- The public `local_anchor::LocalAnchorCoordinator` is the surviving M5 seam.
  Its direct integration test proves durable admission, restart, duplicate,
  conflict, acknowledgement, generation-zero root anchor, and completion
  behavior without the unused wrapper.
- The two test-scoped `Trail` imports preserve existing approval Trail
  assertions. They introduce no product behavior and prevent a new unused
  production import warning.

## Evidence

- `rg "submit_local_root_anchor|short_event_digest|process_local_notification" tethers-0.1/host-rust --type rust` — PASS: zero matches after removal.
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --test m5_local_anchor --all-features --locked` — PASS: 1/1 M5 integration test.
- `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked` — PASS: exactly five remaining production-library warnings (D11-D15); no D7-D9 warning.
- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all` — only the authorised `application.rs` changed; immediate diff inspected.
- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`, `git diff --check`, and `cargo clippy --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked` — PASS; only pre-existing project-wide Clippy warnings remain.
- `just verify-agent` — PASS (134s): task packet, formatter, cargo check,
  full Cargo tests, Rust agent tools, dependency policy/advisories, and
  Nextest. Nextest: 1592 passed, 2 skipped.

## Discoveries

- Removing the dead production wrapper exposed that the `Trail` trait import
  had been incidentally shared with `application.rs` test modules. Scoping the
  import to those test modules retains their existing trait-method assertions
  and yields the expected five library warnings.
- The ignored OCaml engine outputs restored during Job A remained available for
  this job's retained-engine test paths; no further machine or tracked-file
  repair was needed.

## Remaining risks

- None known within Job B scope. D11-D15 remain intentionally unresolved for
  their independently classified later jobs.

## Smallest next action

Begin Job C's independent D11 authorisation-wrapper classification from this
pushed Job B tip.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `tethers-0.1/host-rust/src/application.rs`
- `tethers-0.1/host-rust/src/local_anchor.rs`
- `tethers-0.1/host-rust/tests/m5_local_anchor.rs`
