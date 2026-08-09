# Current Implementation Task

Control contract: `1`
Task packet: `F8-D7+D8+D9 — Dead Local-Notification Host Integration Seam`
Owner: `Codex`
Status: `IN_PROGRESS`
Task colour: `Amber`
Route: `Codex classified the unused local-notification wrapper chain before removal`
Worker note: `docs/worker-notes/2026-08-09-f8-d7-d8-d9-local-notification-cleanup.md`
Base branch: `foundation/f8-d5-d6-d10-exact-approval-cleanup`
Base commit: `aa01766dc269338b07b4302bc70d6dc9ecaf1037`
Implementation branch: `foundation/f8-d7-d8-d9-local-notification-cleanup`
Implementation checkpoint: `PENDING`
OCaml switch path: `N/A`
Rust toolchain: `1.97.1`
Rust change class: `RUST`

## Objective

Remove the dead D7-D9 host-local-notification integration seam only, while
retaining the independent M5 local-anchor admission coordinator and every live
event-processing, protocol, Trail, queue, and acknowledgement contract.

## Relevant background and existing behaviour

The Job A closeout at `aa01766dc269338b07b4302bc70d6dc9ecaf1037` left eight
production-library dead-code warnings: D7-D9 and D11-D15. D7-D9 were written
as an optional host bridge from a local provider notification to existing J11
admission and J10 event processing, but the current Rust tree has never wired
that bridge into a command, provider, socket, public API, or test. The actual
M5 admission module is independently public and directly tested without this
unused application-level seam.

## Classification evidence

`submit_local_root_anchor`, `short_event_digest`, and
`process_local_notification` have references only within their three-function
chain in `src/application.rs`; no Rust production entrypoint, public export,
or test calls the chain.  `local_anchor::LocalAnchorCoordinator` remains a
public module API and is exercised directly by `tests/m5_local_anchor.rs`.
This classifies D7-D9 as one **DEAD SUBSYSTEM**, not as the M5 coordinator or a
live local-provider protocol route.

## Required behaviour

1. Remove D7 `submit_local_root_anchor`, D8 `short_event_digest`, and D9
   `process_local_notification` together, without replacement or suppression.
2. Retain `local_anchor::LocalAnchorCoordinator`, its admission and durable
   completion behavior, and the direct M5 integration test unchanged unless a
   migration is demonstrably necessary.
3. Preserve the live `process_one_event` route, Result Anchor queue behavior,
   generation handling, Trail machinery, provider protocol behavior, and all
   independently reachable event-processing contracts.
4. Reduce the intended production-library dead-code warning count from 8 to 5;
   leave D11-D15 unresolved for their separately classified jobs.
5. Run focused M5 and compiler checks, then exactly one final
   `just verify-agent` after the implementation checkpoint.

## Relevant components

### AUTHORISED PATHS
- `tethers-0.1/host-rust/src/application.rs` — remove the dead D7-D9 chain

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-f8-d7-d8-d9-local-notification-cleanup.md`

## Frozen decisions and invariants

- The independently public M5 `LocalAnchorCoordinator` is retained. Its
  generation-zero root-anchor identity, durable admission/restart/duplicate/
  conflict behavior, acknowledgement ordering, and terminal completion record
  remain owned by `local_anchor.rs`.
- Do not recreate a wrapper under a new name or wire a new local-provider
  production entrypoint in this cleanup.
- Do not alter provider JSON-RPC notifications, `process_one_event`, queue
  semantics, Result Anchor behavior, Trail behavior, policy, dispatch, or
  protocol fields.
- No `#[allow(dead_code)]` added merely to silence warnings. No D11-D15 work.

## Acceptance criteria

1. Exact Rust searches show zero occurrences of D7-D9 after removal.
2. The source diff is limited to removal of the classified three-function
   chain; the public M5 coordinator and its integration test are retained.
3. A full-target locked `cargo check` reports exactly five intended remaining
   production-library warnings, with no D7-D9 warning.
4. Focused M5 integration evidence confirms admission, restart, duplicate,
   conflict, acknowledgement, and completion behavior remain available at the
   surviving coordinator seam.
5. Formatter output is limited to the authorised Rust path, whitespace checks
   pass, and Clippy remains clean apart from existing non-F8 diagnostics.
6. One final `just verify-agent` passes after the implementation checkpoint.
7. Closeout changes only the packet and worker note; the completed branch is
   normally pushed with matching remote SHA and clean status.

## Required verification

1. `rg "submit_local_root_anchor|short_event_digest|process_local_notification" tethers-0.1/host-rust --type rust` after removal.
2. `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml --test m5_local_anchor --all-features --locked`.
3. `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked`.
4. `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all`, immediate diff inspection, then `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check` and `git diff --check`.
5. `cargo clippy --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features --locked`.
6. `just verify-agent` once after the implementation checkpoint, then full
   range-diff, remote-equality, and clean-status inspection.

## Formatting and checkpoint sequence

The only authorised Rust path is `tethers-0.1/host-rust/src/application.rs`.
Before the implementation checkpoint run the mutating formatter command above
and inspect its immediate diff. STOP if rustfmt changes an unauthorised Rust
path.

## Completion and publication

After the documentation-only closeout commit, normally push this branch to
`origin`. No force-push, merge, rebase, direct `main` update, or pull request
is authorised.

## Forbidden changes

- No removal or modification of `local_anchor.rs` coordinator behavior or its
  integration tests absent demonstrated necessity.
- No D11-D15 cleanup, dead-code suppression, OCaml, fixture, build, protocol,
  dependency, CI, lint-policy, merge, amend, tag, force-push, direct `main`,
  or pull-request change.

## Stop conditions

STOP if a real production caller, public external contract, necessary test
migration that weakens evidence, architectural choice, unauthorised formatter
output, or untrustworthy verification is found. Do not begin Job C without a
successful Job B verification and pushed tip.

## Expected pre-existing changes

None.
