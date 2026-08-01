# Current Implementation Task

Control contract: `1`

Task: `J17A1 - set the product release identity to 0.2.0`
Owner: `Hy3`
Status: `COMPLETE`
Task colour: `Green`
Route: `Hy3 implementation - Green product-metadata update`
Branch: `hy3/j17a-product-version`
Base commit: `160f4d4da1641b12380644227a273e1d3db1a8bb`
Worker note: `docs/worker-notes/2026-08-01-j17a-product-version.md`

## Objective

Set the Tethers product release identity from `0.1.0` to `0.2.0` across every
live product surface (Rust host package, CLI, engine and provider `serverInfo`
identities, and the real-engine golden transcripts) without touching the frozen
`0.1` language semantics, the MCP wire `protocolVersion`, fixture identity, or
dependency versions.

## Relevant background and existing behaviour

J16D completed the clean native Windows verification gate and left the release
at `0.1.0` product identity. J17A1 is the first release-identity step for 0.2.0.
The version bump necessarily changes `Cargo.lock` (root package only), which
invalidated J14C's exact whole-file `Cargo.lock` digest pin; that pin was
refreshed to the new reviewed hash. The host's strict CLI wraps Clap's
`--version` output inside the `invalid_cli_usage` JSON envelope and exits 2;
that pre-existing behaviour is recorded but not changed.

## Required behaviour

1. Change the seven live product identities from `0.1.0` to `0.2.0`: the Rust
   package `Cargo.toml`, `Cargo.lock` root entry, `cli.rs`, `engine_stdio.rs`
   `clientInfo.version`, `stdio_provider.rs` `clientInfo.version`,
   `tethers_mcp_server.ml` `serverInfo.version`, and
   `tethers-local-file-provider.ps1` `serverInfo.version`.
2. Change `serverInfo.version` from `0.1.0` to `0.2.0` in the 13 real-engine
   `tethers-0.1/protocol/mcp-transcripts/*/stdout.jsonl` files, leaving the two
   error-only transcripts and every fixture/`protocolVersion` byte unchanged.
3. Refresh J14C's exact `Cargo.lock` digest pin to the new reviewed hash
   (`894f2ce6…`); the digest guard stays exact and was not weakened.
4. Preserve frozen `0.1` language semantics, MCP `protocolVersion` (`2025-11-25`)
   and fixture server identity.

## Relevant components

- `tethers-0.1/host-rust/Cargo.toml`, `Cargo.lock`, `src/cli.rs`,
  `src/engine_stdio.rs`, `src/stdio_provider.rs`
- `tethers-0.1/engine-ocaml/bin/tethers_mcp_server.ml`
- `tethers-0.1/providers/tethers-local-file-provider.ps1`
- `tethers-0.1/protocol/mcp-transcripts/*/stdout.jsonl` (13 real-engine files)
- `tethers-0.1/scripts/test-j14c-real-file-move.ps1` (pin refresh only)

## Frozen decisions and invariants

- Product release identity is now `0.2.0`; the frozen language semantics remain
  `0.1`.
- MCP wire `protocolVersion` (`2025-11-25`) and the supported-version list are
  unchanged.
- The fixture `tethers-stdio-fixture.ps1` `serverInfo.version` stays `0.1.0`.
- `Cargo.lock` changed only at the root package version; no dependency version,
  checksum, source, or unrelated lock entry differs.
- The J14C `Cargo.lock` digest guard remains exact (whole-file SHA-256), merely
  updated to the reviewed post-bump hash.
- The strict CLI envelope reporting `0.2.0` with exit 2 is pre-existing and not
  altered by this metadata task.
- Release notes, J17 sign-off, main publication and tagging are deferred.

## Acceptance criteria

1. All seven product identities read `0.2.0`; `cargo metadata` reports the root
   package version `0.2.0`.
2. `Cargo.lock` differs only at the root package version; its SHA-256 is
   `894f2ce6692837fa4c449c0fc593a37ed5597577ea5b4093da0912e6ee2b14e3`.
3. The 13 real-engine transcripts report `serverInfo.version` `0.2.0`; the two
   error-only transcripts and every `protocolVersion`/`fixture` byte are
   unchanged.
4. J14C passes `9 rows, 9 passed, 0 failed` with `196 assertions`, exit 0; the
   refreshed digest pin matches the reviewed hash.
5. `cargo fmt --check`, `cargo check --locked`, `cargo test --locked` (797
   passed, 0 failed, 0 ignored), `opam exec dune build`, `check-fixtures.ps1`
   and the 15-case MCP transcript suite all pass.
6. The packet checker passes for control-v1 consistency.
7. Only the updated authorised paths change.

## Required verification

- `cargo fmt --check`, `cargo metadata --locked --no-deps`, `cargo check
  --locked`, `cargo test --locked` (797/0/0) — all exit 0.
- `opam exec --switch=… -- dune build` — exit 0.
- `check-fixtures.ps1` — 46 JSON + 30 JSONL valid.
- `test-mcp-transcripts.ps1` — 15/15 cases pass.
- `test-j14c-real-file-move.ps1` — exit 0, 9/9, 196 assertions.
- `check-tethers-task-packet.ps1` — PASS.

## Forbidden changes

- `tethers-0.1/SPEC.md`, `tethers-0.1/README.md`, `tethers-0.1` directory name.
- `tethers-0.1/engine-ocaml/tethers_engine.opam` / `.opam.locked` wording.
- MCP `protocolVersion` values, manifest/capability versions, fixture versions,
  dependency versions, historical worker notes.
- CLI parsing or envelope behaviour.
- main, release notes, J17 sign-off, or any tag.

## Stop conditions

- A product identity left at `0.1.0`, an unintended `Cargo.lock` entry change,
  a transcript `protocolVersion` or fixture byte change, J14C failure, or any
  unauthorised path change stops the task.

## Expected pre-existing changes

The branch `hy3/j17a-product-version` was created at `160f4d4` (6 ahead of and
0 behind `origin/main`) with the 20 intended product/transcript modifications
already applied; this task added the authorised J14C pin refresh.

## Commit and publication boundary

Create exactly one commit: `chore: set product version to 0.2.0`.

Push only: `hy3/j17a-product-version`.

Do not push main. Do not create a tag. Do not begin release notes or J17
sign-off.

## Return contract

Return `COMPLETE` or `BLOCKED` and stop.

For `COMPLETE`, report the commit SHA, changed product identities, old and new
`Cargo.lock` hashes, J14C pin refresh, J14C result, exact 13 transcript paths,
Rust result, Cargo metadata result, CLI version-envelope result, preserved `0.1`
categories, changed paths, branch ahead/behind, and worktree cleanliness.

Stop after reporting. Do not begin J17 sign-off.
