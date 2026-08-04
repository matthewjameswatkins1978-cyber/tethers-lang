# M01C1 Engine Session Warning Pilot

Status: frozen implementation blueprint

## Purpose

Exercise the accepted Rust agent toolchain on one small, behaviour-preserving warning cluster before broader warning and repository cleanup.

M01C1 targets warnings whose primary source span is `tethers-0.1/host-rust/src/engine_stdio.rs`. The work must use language-aware reference discovery, focused Nextest feedback, ordinary Cargo completion tests, Cargo-deny policy gates, and cargo-machete evidence.

This is a warning-cleanup pilot, not a redesign of the retained OCaml engine session, subprocess supervision, MCP protocol, timeout policy, or Plug lifecycle.

## Accepted starting baseline

M01B is accepted at `f7e84a467bf77a02f1f1b60cd319c55644dd9bbd`.

The repository baseline is:

```text
Rust             1.97.1
rust-analyzer    toolchain component
cargo-nextest    0.9.140
cargo-deny       0.19.7
cargo-machete    0.9.2
Cargo tests      926 passing
Nextest tests    1133 passing
Nextest retries  0
Cargo.lock       committed
```

## Target warning cluster

Before editing, capture ordinary Clippy output and machine-readable Clippy JSON. Record every warning whose primary span is `src/engine_stdio.rs`, including its lint code, symbol or expression, and line.

Known likely members include:

- the retained `EngineSession::read_timeout` field being written but not read;
- path-reference API linting around `EngineSession::launch`, if emitted by the accepted Clippy version.

The worker must trust the captured Rust 1.97.1 diagnostics rather than assuming this list is complete or forcing a warning that is not emitted.

## Required repair shape

1. Use the OpenCode LSP tool before editing to locate:
   - the definition and all references of `EngineSession::launch`;
   - the definition and all references of `EngineSession::read_timeout`;
   - all calls to the private `read_json` helper.
2. Record the LSP result summary in the worker note.
3. Remove every warning whose primary span is `src/engine_stdio.rs` when it can be fixed without behaviour or protocol change.
4. The retained timeout must become real authority rather than decorative state:
   - one named default duration must be used for initialization and stored on the session;
   - validation and evaluation reads must use the stored session timeout;
   - the effective default remains exactly ten seconds.
5. If Clippy emits `ptr_arg` for `EngineSession::launch`, accept `&Path` rather than `&PathBuf` and update only proven call sites. Path resolution and command construction must remain equivalent in meaning.
6. Prefer removing the cause to adding an allow. No new `#[allow(...)]`, `#[expect(...)]`, underscore-renaming, dummy read, unreachable use, or warning suppression is permitted.
7. Add or update focused tests only when needed to prove unchanged timeout or path behaviour. Do not add timing-sensitive sleeps merely to consume the field.

## Tool trial requirements

### rust-analyzer / OpenCode LSP

The worker must actually use language-aware definition/reference queries before editing. Text search may supplement but not replace this evidence.

If the current OpenCode process does not expose the LSP tool, launch a fresh process through `scripts/start-opencode-lsp.ps1` using `OPENCODE_BIN` or an explicit `-OpenCodePath`, then continue there.

### Nextest

Use Nextest during the edit loop with the repository config and zero retries. Run the narrowest available engine-session tests first, then the complete Nextest graph before completion.

Nextest remains single-threaded on this Windows baseline. Do not alter the configuration in M01C1.

### Cargo

Ordinary Cargo test remains final authority. Clippy output is the warning measurement authority.

### cargo-deny and cargo-machete

Run both locked Cargo-deny gates and cargo-machete. No dependency or policy change is authorised. Machete findings are evidence only.

## Warning accounting

Capture before and after:

- total Clippy warning count;
- warnings grouped by lint code;
- warnings whose primary span is `src/engine_stdio.rs`;
- warnings outside the target file.

Acceptance requires:

- zero warnings with a primary span in `src/engine_stdio.rs`;
- no new warning code or warning instance outside the target file;
- total warning count lower than the baseline;
- no suppression attribute added.

Do not repair warnings in `application.rs`, `child_process.rs`, `event_queue.rs`, `result_anchor.rs`, tests, or another module during this job. Record them for later M01C slices.

## Behavioural invariants

- MCP protocol version remains `2025-11-25`.
- Initialize request and initialized notification remain unchanged.
- Validation uses `tethers.validate` and evaluation uses `tethers.evaluate`.
- Default engine read timeout remains ten seconds.
- Error strings, response validation, request IDs, shutdown behaviour, and stderr-tail behaviour remain unchanged unless formatting alone moves code.
- No new retry, fallback, concurrency, parallelism, or process lifecycle path.
- No public CLI or Tethers language change.

## Permitted files

Only these files may change:

- `tethers-0.1/host-rust/src/engine_stdio.rs`;
- a direct call site in `tethers-0.1/host-rust/src/check_command.rs` only if a `&PathBuf` to `&Path` signature cleanup requires it;
- a direct call site in `tethers-0.1/host-rust/src/host_execution.rs` only for the same signature cleanup;
- focused existing tests colocated in those files when necessary;
- `docs/CURRENT_CLINE_TASK.md` for control state and checkpoint;
- `docs/worker-notes/2026-08-04-m01c1-engine-session-warning-pilot.md`.

## Forbidden work

Do not modify:

- Cargo.toml, Cargo.lock, dependencies, features, edition, rust-version, or publish metadata;
- rust-toolchain.toml or any agent-tool version/configuration;
- `.config/nextest.toml`, deny.toml, justfile, opencode.json, or tooling scripts;
- OCaml source, opam, Dune, locks, fixtures, or generated evidence;
- production modules outside the permitted list;
- event-queue Send semantics or its misleading compile-time-comment test;
- Plug installation, J24J, provider policy, runtime permissions, Trail, Anchor, package, release, tag, or publication work.

Do not add warning suppressions, dependencies, retries, sleeps, fake uses, or large refactors.

## Acceptance evidence

M01C1 is complete only when:

1. LSP definition/reference evidence is recorded.
2. Before/after machine-readable warning inventories are recorded.
3. No warning has a primary span in `src/engine_stdio.rs` after the repair.
4. No warning outside the target file is added or changed unexpectedly.
5. The effective default read timeout remains exactly ten seconds and is used by initialization, validation, and evaluation reads as designed.
6. Focused engine-session tests pass under Nextest with zero retries.
7. All 926 ordinary Cargo tests pass.
8. All 1133 Nextest tests pass.
9. Cargo-deny licence, ban, source, and advisory gates pass.
10. Cargo-machete reports its exact result without modification.
11. Cargo.lock hash remains `D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB`.
12. Rustfmt, Clippy, packet checker, `just verify-agent`, and `git diff --check` pass.
13. No dependency, OCaml, protocol, CLI, Tethers behaviour, lifecycle, retry, or concurrency change occurs.

## Editing recovery discipline

After an exact `oldString` replacement failure:

1. do not retry the identical edit;
2. reread the current file;
3. use a fresh smaller patch against the latest content;
4. stop after two materially different failures rather than rewriting a file wholesale.
