# Worker Note

Task: `F8-D7+D8+D9 — Dead Local-Notification Host Integration Seam`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `IN_PROGRESS`

Base commit: `aa01766dc269338b07b4302bc70d6dc9ecaf1037`

Implementation checkpoint: `PENDING`

## Requested outcome

Classify and, only if safe, remove the unused D7-D9 local-notification wrapper
chain while retaining the independently public M5 local-anchor coordinator.

## Pre-implementation evidence

- A full Rust reference search found D7-D9 only at their own definitions and
  internal chain calls in `src/application.rs`; no production entrypoint,
  `lib.rs` export, or test invokes them.
- `LocalAnchorCoordinator` is exported from `local_anchor.rs` and directly
  covered by `tests/m5_local_anchor.rs`, which preserves durable admission,
  restart, duplicate, conflict, acknowledgement, and terminal completion
  behavior without the D7-D9 chain.
- `process_one_event` has live production callers independently of the dead
  wrapper chain. The D7-D9 implementation does not own a provider protocol
  endpoint.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `tethers-0.1/host-rust/src/application.rs`
- `tethers-0.1/host-rust/src/local_anchor.rs`
- `tethers-0.1/host-rust/tests/m5_local_anchor.rs`
