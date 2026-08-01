# Worker Note

Task: `J17A1 - set the product release identity to 0.2.0`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `Hy3`
Status: `COMPLETE`
Base commit: `160f4d4da1641b12380644227a273e1d3db1a8bb`
Implementation checkpoint: `160f4d4da1641b12380644227a273e1d3db1a8bb`
Starting branch: `hy3/j17a-product-version`
Starting SHA: `160f4d4da1641b12380644227a273e1d3db1a8bb`

## Requested outcome

Set the live Tethers product release identity from `0.1.0` to `0.2.0` on the
`hy3/j17a-product-version` branch, refresh J14C's exact `Cargo.lock` digest pin
to the reviewed post-bump hash, and leave frozen `0.1` language semantics, the
MCP wire `protocolVersion`, and fixture identity untouched.

## Changes made (authorised paths only)

Seven product identities changed `0.1.0` -> `0.2.0`:

- `tethers-0.1/host-rust/Cargo.toml` — root package version.
- `tethers-0.1/host-rust/Cargo.lock` — root package version only.
- `tethers-0.1/host-rust/src/cli.rs` — public CLI version.
- `tethers-0.1/host-rust/src/engine_stdio.rs` — host `clientInfo.version`.
- `tethers-0.1/host-rust/src/stdio_provider.rs` — host `clientInfo.version`.
- `tethers-0.1/engine-ocaml/bin/tethers_mcp_server.ml` — engine `serverInfo.version`.
- `tethers-0.1/providers/tethers-local-file-provider.ps1` — provider `serverInfo.version`.

Thirteen real-engine golden transcripts changed `serverInfo.version` `0.1.0` ->
`0.2.0`, with byte-preserving literal replacement (no JSON reserialisation):

- `tethers-0.1/protocol/mcp-transcripts/clean-eof-shutdown/stdout.jsonl`
- `tethers-0.1/protocol/mcp-transcripts/evaluate-correlated-tethers-error/stdout.jsonl`
- `tethers-0.1/protocol/mcp-transcripts/evaluate-matched/stdout.jsonl`
- `tethers-0.1/protocol/mcp-transcripts/evaluate-minimal-tethers-error/stdout.jsonl`
- `tethers-0.1/protocol/mcp-transcripts/evaluate-not-matched/stdout.jsonl`
- `tethers-0.1/protocol/mcp-transcripts/initialization-success-2025-06-18/stdout.jsonl`
- `tethers-0.1/protocol/mcp-transcripts/initialization-success/stdout.jsonl`
- `tethers-0.1/protocol/mcp-transcripts/malformed-tool-arguments/stdout.jsonl`
- `tethers-0.1/protocol/mcp-transcripts/tools-list/stdout.jsonl`
- `tethers-0.1/protocol/mcp-transcripts/unknown-tool/stdout.jsonl`
- `tethers-0.1/protocol/mcp-transcripts/validate-invalid/stdout.jsonl`
- `tethers-0.1/protocol/mcp-transcripts/validate-missing-source/stdout.jsonl`
- `tethers-0.1/protocol/mcp-transcripts/validate-valid/stdout.jsonl`

J14C digest pin refresh (one line):

- `tethers-0.1/scripts/test-j14c-real-file-move.ps1` —
  `$ExpectedCargoLockHash = "d323870ea02f09391a5d0d9aa0e9a701cf686a5ac005b840ee7218e70edb5602"`
  changed to
  `$ExpectedCargoLockHash = "894f2ce6692837fa4c449c0fc593a37ed5597577ea5b4093da0912e6ee2b14e3"`.

Packet and this note are the only doc changes.

## Decisions and assumptions

- The version bump is product-metadata only; language semantics, MCP wire
  `protocolVersion`, and fixture identity are deliberately out of scope.
- The J14C `Cargo.lock` whole-file SHA-256 pin is a release-reproducibility
  guard, not a behavioural contract; refreshing it to the reviewed post-bump
  hash preserves the guard (it remains exact) and does not weaken it.
- The strict CLI `invalid_cli_usage` envelope around `--version` (exit 2) is
  pre-existing host behaviour; recording it satisfies the version-evidence
  requirement without changing parsing or envelope logic. A conventional exit-0
  `--version` route is outside the 0.2 metadata task and the accepted J13
  strict-envelope contract.

## Evidence

- Cargo.lock SHA-256 before bump: `d323870ea02f09391a5d0d9aa0e9a701cf686a5ac005b840ee7218e70edb5602`.
- Cargo.lock SHA-256 after bump (verified): `894f2ce6692837fa4c449c0fc593a37ed5597577ea5b4093da0912e6ee2b14e3`.
- Cargo.lock diff: only the root package `tethers-reference-host` version line
  `0.1.0` -> `0.2.0`; `cargo metadata --no-deps` rewrote nothing else.
- Cargo metadata (root package): `"name":"tethers-reference-host","version":"0.2.0"`.
- Rust: `cargo fmt --check`, `cargo check --locked`, `cargo test --locked` all
  exit 0; test results 44 + 724 + 29 = **797 passed, 0 failed, 0 ignored**.
- OCaml: `opam exec --switch=… -- dune build` exit 0.
- `check-fixtures.ps1`: 46 JSON + 30 JSONL valid, exit 0.
- `test-mcp-transcripts.ps1`: **15/15 cases pass**, exit 0 (after engine rebuild).
- `test-j14c-real-file-move.ps1`: exit 0, **9 rows, 9 passed, 0 failed, 196
  assertions**; cleanup succeeded; no second move on replay.
- CLI version envelope (already observed, not rerun): process exit 2,
  `status: invalid_cli_usage`, embedded `exit_code: 2`, message contains
  `tethers-reference-host 0.2.0`.

## Discoveries

- The first J17A1 run blocked only because J14C pins a whole-file `Cargo.lock`
  digest; the legitimate version bump changes that digest, so the pin had to be
  refreshed. This is a known release-reproducibility pattern, not a defect.
- Two `mcp-transcripts/*/stdout.jsonl` files (`call-before-initialization`,
  `incompatible-mcp-protocol-version`) contain no `serverInfo` and were rightly
  left unchanged.

## Remaining risks

- None within packet scope. The strict CLI exit-2 `--version` route is
  pre-existing and out of scope for this metadata task.

## Smallest next action

- Release notes, J17 independent sign-off, main publication, and the `v0.2.0`
  tag remain deferred and must be authorised separately. Do not begin them here.

## References

- `docs/CURRENT_CLINE_TASK.md` (J17A1 packet)
- `tethers-0.1/host-rust/Cargo.toml`, `Cargo.lock`, `src/cli.rs`,
  `src/engine_stdio.rs`, `src/stdio_provider.rs`
- `tethers-0.1/engine-ocaml/bin/tethers_mcp_server.ml`
- `tethers-0.1/providers/tethers-local-file-provider.ps1`
- `tethers-0.1/protocol/mcp-transcripts/*/stdout.jsonl` (13 changed)
- `tethers-0.1/scripts/test-j14c-real-file-move.ps1`
- branch `hy3/j17a-product-version`, base `160f4d4da1641b12380644227a273e1d3db1a8bb`
