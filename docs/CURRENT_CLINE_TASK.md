# Current Implementation Task

Control contract: `1`

Task: `J13C - strict public trail command`

Owner: `Goose`

Status: `COMPLETE`

Task colour: `Amber`

Route: `Goose Medium - bounded Amber public inspection route`

Base commit: `3020e7ea3c68ac2bdec5e50a91a0232fedd503f0`

Branch: `goose/j13c-trail-command`

Worker note: `docs/worker-notes/2026-07-30-j13c-trail-command.md`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

## Objective

Add one strict public command: `tethers-reference-host trail --trail <ABSOLUTE_PATH> --execution-id <exec_UUID>`.  The command reads one existing append-only JSONL Trail file, selects the entries belonging to exactly one execution identity, preserves their file order, and emits one stable `tethers.cli/1` JSON envelope.  This is a read-only inspection route.

## Relevant background and existing behaviour

J13A (`check`) validates Tether source, engine, and provider availability.  J13B (`run`) submits one explicit Anchor and Facts through the real execution slice.  J13C completes J13 by providing read-only Trail inspection.  J14 depends on J13 being complete.

The existing `replay::ExecutionId::parse` validates the `exec_<UUID>` format and remains authoritative.  Trail files are append-only JSONL with a top-level `execution_id` field on relevant entries.  Unrelated audit entries (event_admitted, etc.) have no `execution_id` and must be skipped.

## Required behaviour

1. Add `Trail` variant to CLI enum with `--trail` and `--execution-id` mandatory options.
2. Create `trail_command.rs` with read-only inspection coordinator.
3. Wire from `main.rs`.
4. Validate execution ID through `ExecutionId::parse` before opening the file.
5. Validate trail path: absolute, exists, regular file, read-only.
6. Read JSONL sequentially with 8 MiB line limit, UTF-8, LF/CRLF accepted.
7. Reject blank lines, malformed JSON, duplicate keys, non-object JSON, non-string execution_id.
8. Return matching entries in original file order.
9. Zero matches returns not_found/9 with EXECUTION_NOT_FOUND.
10. Malformed content returns audit_failed/8 with TRAIL_INVALID and safe line number.
11. Emit exactly one compact JSON document to stdout.
12. No mutation of Trail file, replay storage, or repository files.
13. Create focused Rust tests (24 cases).
14. Create public acceptance script (16 cases).
15. Update DECISIONS.md.

## Relevant components

- `tethers-0.1/host-rust/src/cli.rs` - strict argument parsing
- `tethers-0.1/host-rust/src/main.rs` - dispatch
- `tethers-0.1/host-rust/src/trail_command.rs` - new read-only inspector
- `tethers-0.1/scripts/test-j13c-trail.ps1` - new acceptance script
- `docs/CURRENT_CLINE_TASK.md` - this task packet
- `docs/DECISIONS.md` - decision record
- `docs/worker-notes/2026-07-30-j13c-trail-command.md` - evidence

## Frozen decisions and invariants

- `ExecutionId::parse` remains authoritative for execution-ID validation.
- Inspection is read-only; no engine, provider, replay, or mutation.
- One explicit Trail path only; no search, replay lookup, or repair.
- Matching top-level execution_id entries retain file order.
- Malformed Trail content fails closed as audit_failed.
- Zero matching entries is not_found.
- J13 is complete only after this command is accepted.

## Acceptance criteria

1. Branch started from exact base `3020e7ea3c68ac2bdec5e50a91a0232fedd503f0`.
2. Effective Goose reasoning confirmed MEDIUM before mutation.
3. CLI accepts `trail --trail --execution-id` in either order, rejects missing/duplicate/unknown options.
4. `ExecutionId::parse` reused; no second parser.
5. Trail path validated: absolute, exists, regular file.
6. Read-only: Trail SHA-256 unchanged after all inspections.
7. Matching entries returned in original file order.
8. Unrelated execution IDs omitted, audit entries skipped.
9. Zero matches: not_found/9 with EXECUTION_NOT_FOUND.
10. Malformed content: audit_failed/8 with TRAIL_INVALID and safe line number.
11. Exactly one compact JSON document on stdout, no timestamp.
12. No raw Trail data or OS diagnostics in public errors.
13. Cargo.lock byte-identical to d323870ea02f09391a5d0d9aa0e9a701cf686a5ac005b840ee7218e70edb5602.
14. All Rust, OCaml, and script regressions pass.
15. Packet checker and whitespace checks pass.
16. Only authorised files changed.

## Forbidden changes

No Cargo manifest, Cargo.lock, OCaml, configuration, Trail writer, replay backend, or production execution file.  No engine, provider, replay, or Trail mutation.  No timestamp in any envelope.  No -D warnings or warning-attribute changes.  No main merge or push.

## Stop conditions

Return BLOCKED when: origin/main mismatch, dirty worktree, branch exists with different history, reasoning not MEDIUM, toolchain preflight fails, ExecutionId::parse cannot be reused, Cargo.lock changes, any production file must change, public errors expose raw data, two similar failures.

## Expected pre-existing changes

None. Starting from clean `main` at `3020e7ea3c68ac2bdec5e50a91a0232fedd503f0`.

## Required verification

```powershell
# Rust (all proxied through rustup run 1.89.0, RUSTUP_AUTO_INSTALL=0)
rustup run 1.89.0 cargo fmt --manifest-path .\tethers-0.1\host-rust\Cargo.toml --check
rustup run 1.89.0 cargo check --manifest-path .\tethers-0.1\host-rust\Cargo.toml --locked
rustup run 1.89.0 cargo check --manifest-path .\tethers-0.1\host-rust\Cargo.toml --locked --tests
rustup run 1.89.0 cargo test --manifest-path .\tethers-0.1\host-rust\Cargo.toml --locked j13c_ -- --nocapture
rustup run 1.89.0 cargo test --manifest-path .\tethers-0.1\host-rust\Cargo.toml --locked
rustup run 1.89.0 cargo clippy --manifest-path .\tethers-0.1\host-rust\Cargo.toml --locked --all-targets --all-features
rustup run 1.89.0 cargo build --manifest-path .\tethers-0.1\host-rust\Cargo.toml --locked
rustup run 1.89.0 cargo build --manifest-path .\tethers-0.1\host-rust\Cargo.toml --locked --release

# Public acceptance
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-j13c-trail.ps1

# Regressions
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-j13a-check.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-j13b-run.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\check-fixtures.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-mcp-transcripts.ps1
pwsh -NoProfile -File .\tethers-0.1\scripts\test-engine.ps1
pwsh -NoProfile -File .\tethers-0.1\scripts\demo.ps1

# Packet checker
pwsh -NoProfile -ExecutionPolicy Bypass -File .\.github\scripts\check-tethers-task-packet.ps1

# Diff and status
git diff --check
git diff --stat
git diff
git status --short --branch
```
