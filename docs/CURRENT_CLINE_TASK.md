# Current Implementation Task

Control contract: `1`

Task: `J13A local process supervision and check command`

Status: `COMPLETE`

Task colour: `Green`

Owner: `Goose`

Route: `Goose - J13A in rehearsal worktree`

Worker note: `docs/worker-notes/2026-07-28-j13a-process-check.md`

Base branch: `main`

Base commit: `f100689a35c9b7032193abd4f737c3203815fa4c`

Branch: `goose/j13a-process-check`

## Expected pre-existing changes

None. Starting from clean main at f100689a.

## Objective

Implement the first public J13 route:

```
tethers-reference-host check --config <PATH> --engine <PATH>
```

This packet establishes strict CLI parsing, stable JSON command envelopes,
explicit hidden compatibility routing, caller-relative path resolution,
supervised Windows child-process ownership, one retained MCP engine session,
ordered Tether validation, one retained session per configured provider,
provider initialize/tools-list availability verification, deterministic
check output, and complete process cleanup.

The command must perform no provider capability call, event evaluation, policy
decision, dispatch, Trail write or replay write.

## Acceptance criteria

See full J13A task packet for complete criteria. Key points:

1. `cargo fmt --check` passes
2. `cargo check` and `cargo check --tests` pass
3. `cargo test j12_ -- --nocapture` all 99 pass
4. `cargo test` passes (with known flaky stderr_capture on restricted test env)
5. `cargo clippy --all-targets --all-features` zero new errors
6. `cargo build` and `cargo build --release` succeed
7. `pwsh -NoProfile -File tethers-0.1/scripts/test-j13a-check.ps1` passes
8. `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` passes
9. Legacy scripts (demo, test-host-denial, test-host-execution-failure,
   test-host-result-follow-up) updated to use __legacy
10. `git diff --check` clean

## Forbidden changes

Only the 19 authorised files may change. No J13B, J13C, or J14 behaviour.
No provider tools/call. No tethers.evaluate. No Trail or replay creation.
