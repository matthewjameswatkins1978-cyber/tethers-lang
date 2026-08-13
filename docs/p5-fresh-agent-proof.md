# P5 Fresh-Agent Plug Authoring Proof — Text Stats

Status: experiment log (awaiting Lucy review)

## Experiment

- **Model:** DeepSeek V4 Flash, Thinking ON, Effort High.
- **Prompt:** the P5 challenge prompt given to a fresh agent; it names
  `docs/PLUG_AUTHORING.md` as the only authoring guide, the public CLI path,
  the required package/provider/capability identities, the required Text Stats
  semantics, the destination directory, and the sources that may not be used.
- **Author sources made available:** `docs/PLUG_AUTHORING.md`; the public CLI
  binary `tethers-0.1/host-rust/target/debug/tethers-reference-host.exe` and its
  `--help` output; Cargo/Rust 1.97.1 and its official docs;
  `reference-plugs/pdf-tools/author/plug.json` and
  `reference-plugs/pdf-tools/author/manifests/pdf-inspect-v1.json` (opened ONLY
  because the manual names them as the concrete examples for sections 4, 5, 13).
- **Operating guidance also read (not an authoring source):**
  `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md`, because `AGENTS.md` makes it
  mandatory for any Rust task. It governs Rust implementation technique only and
  contains no Plug format, MCP, manifest, scope, or trust semantics.
- **Prohibited sources (not opened as authoring guides):** P1–P3 worker notes,
  P2/P3 host test implementations, `docs/CURRENT_CLINE_TASK.md`, internal host
  Rust source (`tethers-0.1/host-rust/src/`), the PDF provider implementation
  (`reference-plugs/pdf-tools/provider-rust/`), fixture builders, and
  architecture/Foundation Pass documents.

## Manual gaps and clarifications

- **None blocked authoring.** The manual was read in full (all 15 sections) and
  was sufficient to author, build, pack, inspect, and conform the Plug without
  consulting any prohibited source.
- **Non-blocking observation (protocol level):** the manual (section 6) defines
  the MCP stdio wire contract at the method level (`initialize`,
  `notifications/initialized`, `tools/list`, `tools/call`) but does not spell
  out the exact JSON-RPC/MCP response envelope shapes (e.g. `initialize` result
  with `serverInfo`, and `tools/call` result with
  `content`/`structuredContent`/`isError`). These were resolved from the MCP
  2025-11-25 protocol itself, which the manual names. This is protocol
  knowledge, not hidden Tethers internals.
- **Genuine narrow deficiency found during controller verification:** the fresh
  author advertised only `inputSchema` for `text_stats` in `tools/list`. That
  is enough for conformance (the conformance suite checks `inputSchema`), but
  the host's retained-session dispatch path also verifies the advertised
  `outputSchema` against the reviewed manifest. The manual's section 6 said
  only "the schemas the provider advertises in `tools/list` must match the
  reviewed manifest" without stating that BOTH `inputSchema` and `outputSchema`
  must be advertised.
- **Correction applied to `docs/PLUG_AUTHORING.md` (section 6):** replaced the
  single sentence with explicit wording: "the `tools/list` entry for each
  operation must advertise the operation name and both the `inputSchema` and
  `outputSchema` exactly as declared in the reviewed manifest. The host
  compares the advertised schemas against the manifest during discovery and
  dispatch, so advertising only `inputSchema` is incomplete."
- **Rerun after correction:** the Text Stats provider now advertises
  `outputSchema` (matching the manifest) and the `mcp_stdout_protocol` test
  asserts it. Provider tests re-ran (9/9 pass) and the full public journey was
  re-run with the corrected provider (results below).
- **Challenge-specific values** (package `tethers.text-stats`, capability
  `text.stats@1`, 8 MiB hard maximum) come from the challenge, not the manual;
  the manual's PDF example uses a 64 MiB maximum and different identities.

## What was built

`reference-plugs/text-stats-proof/` with `provider-rust/` (Cargo project,
MCP stdio provider), `author/plug.json`, `author/manifests/text-stats-v1.json`,
and `README.md`. Provider: `tethers-text-stats-provider` 1.0.0. Package:
`tethers.text-stats` 1.0.0. Capability `text.stats@1`, operation `text_stats`,
scope keys `query_root` + `max_bytes` (schema maximum 8388608 = 8 MiB).

Semantics implemented and tested: relative `path`; exact `size_bytes`;
`sha256:<64 hex>`; logical `line_count`; whitespace-separated `word_count`;
`character_count` after valid UTF-8; path must stay inside `query_root`
(absolute/rooted and `..` paths refused, canonicalised containment check);
regular file required; malformed UTF-8 fails cleanly; `max_bytes` honoured;
`max_bytes` hard maximum 8 MiB; read-only; no network; no writes beyond stderr
diagnostics; no hidden/test-only behaviour. The `tools/list` entry advertises
both `inputSchema` and `outputSchema` matching the reviewed manifest.

## Public journey results (exact commands and outcomes)

Build (manual section 8):

```
cargo build --manifest-path reference-plugs/text-stats-proof/provider-rust/Cargo.toml --locked
```
exit 0. Executable:
`reference-plugs/text-stats-proof/provider-rust/target/debug/tethers_text_stats_provider.exe`.

Pack (manual section 10), temp pack source with `plug.json`,
`manifests/text-stats-v1.json`, `provider/tethers_text_stats_provider.exe`:

```
tethers-reference-host.exe plug pack --source <pack-source> --output <text-stats-1.0.0.tetherplug>
```
exit 0, status `ok`:
`{"package_id":"tethers.text-stats","package_version":"1.0.0","provider_id":"tethers-text-stats-provider","capability_count":1,"semantic_package_digest":"sha256:58cb813d2993285cd5cc0fc3df3cfbdca59bb7c6ca0a51ddfc9b95914bac62e4","raw_archive_digest":"sha256:e0fcf54c8d219f420c9798e5150af6cbc88e58c72f1d98d4c4abc9b8acc819f7"}`

Inspect (manual section 11):

```
tethers-reference-host.exe plug inspect --package <text-stats-1.0.0.tetherplug>
```
exit 0, status `ok`. Identities preserved; capability `text.stats` @ 1,
operation `text_stats`, manifest digest `sha256:76a3816dd816d85a092b09510eb134bf5a1d7e62acabf67a65d0301078cea7e4`; generated payload evidence
(`manifests/text-stats-v1.json` size 1586, `provider/tethers_text_stats_provider.exe` size 740352, sha256 `b0bbf3f4…`); semantic digest
`sha256:58cb813d2993285cd5cc0fc3df3cfbdca59bb7c6ca0a51ddfc9b95914bac62e4` — identical to pack.

Conform, default (manual section 12):

```
tethers-reference-host.exe plug conform --package <text-stats-1.0.0.tetherplug>
```
exit 5, status `approval_required`,
error `conformance_execution_approval_required`
("conformance executes provider code under process supervision, not isolation;
pass --allow-non-isolated-supervised-execution to proceed"). Provider NOT
executed.

Conform, approved non-isolated:

```
tethers-reference-host.exe plug conform --package <text-stats-1.0.0.tetherplug> --allow-non-isolated-supervised-execution
```
exit 0, status `ok`, disposition `passed`, 6/6 cases passed
(`static_candidate_revalidation`, `exact_launch_clean_environment`,
`mcp_initialize_protocol_pin`, `provider_identity`,
`complete_discovery_exact_operations`, `bounded_shutdown_process_cleanup`),
suite `m3-generic-1`. Launch profile: `{"isolated":false,"label":"supervised","limitation":"process supervision only; not isolated or hostile-code-safe",...}`.
Semantic digest `sha256:58cb813d…` again identical.

## Provider semantic tests

```
cargo test --manifest-path reference-plugs/text-stats-proof/provider-rust/Cargo.toml --locked
```
`test result: ok. 8 passed; 0 failed` (unit) and `ok. 1 passed; 0 failed`
(integration).

Tests (all test the real production functions):
- `valid_utf8_file_returns_correct_stats`
- `path_traversal_and_absolute_paths_refuse`
- `oversized_file_refuses`
- `malformed_utf8_refuses`
- `directory_input_refuses_as_not_regular_file`
- `scope_above_hard_max_refuses`
- `scope_at_hard_max_parses_when_root_exists`
- `unknown_or_missing_arguments_refuse`
- `stdout_is_mcp_protocol_only` (spawns the real binary; every stdout line must
  be one JSON object)

## Digest continuity and immutability

- Semantic package digest identical across pack, inspect, and conform:
  `sha256:58cb813d2993285cd5cc0fc3df3cfbdca59bb7c6ca0a51ddfc9b95914bac62e4`.
- `.tetherplug` file SHA-256 (`sha256:e0fcf54c8d219f420c9798e5150af6cbc88e58c72f1d98d4c4abc9b8acc819f7`) equals the pack's
  `raw_archive_digest`; unchanged after inspect.
- Two independent packs from the same source produce byte-for-byte identical
  `.tetherplug` files (deterministic pack).
- Author source unchanged by pack: pre/post SHA-256 identical for
  `author/plug.json` (`7088342c…`), `author/manifests/text-stats-v1.json`
  (`f5d2aa3b…`), and the built provider exe (`b0bbf3f4…`); the packaged provider
  payload sha256 matches the source exe hash byte-for-byte.

## Mistakes encountered and fixes

- `StatsError` initially derived `PartialEq/Eq` while containing `io::Error`
  (not `Eq`); tests used `assert_eq!`. Fixed by dropping the derives and
  asserting with `matches!` patterns, keeping `io::Error` as the error source.
- JSON-RPC id partial-move borrow error; fixed by cloning `request.id`.
- `jsonrpc` field flagged as never read; fixed by validating it (`== "2.0"`,
  rejecting anything else) — protocol discipline, not decoration.
- First build required Cargo to generate `Cargo.lock`; subsequent builds and all
  tests used `--locked`.
- Controller verification finding: `tools/list` advertised only `inputSchema`;
  the retained-session dispatch path verifies `outputSchema` too. Fixed by
  advertising `outputSchema` matching the manifest, updating the
  `mcp_stdout_protocol` test to assert it, and clarifying `docs/PLUG_AUTHORING.md`
  section 6 (see "Manual gaps and clarifications"). The full public journey was
  re-run with the corrected provider; all results above are from that final run.

## Conclusion

A fresh author using `docs/PLUG_AUTHORING.md` as its only authoring guide built
a new non-PDF Plug from scratch and completed the full public journey — build,
pack, inspect, conform-refusal, and approved conform — with correct identities,
generated evidence, digest continuity, and passing semantic tests. The manual
was sufficient to author the Plug; no hidden Tethers knowledge was needed. The
proof surfaced one genuine narrow manual gap (the `tools/list` entry must
advertise both `inputSchema` and `outputSchema` matching the reviewed
manifest), which was fixed with explicit wording in `docs/PLUG_AUTHORING.md`
and validated by a corrected provider and a re-run journey.
