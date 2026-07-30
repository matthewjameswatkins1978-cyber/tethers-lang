# Worker Note

Task: `J13C-A — preserve Trail text and repair acceptance evidence`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `Goose`
Status: `COMPLETE`
Base commit: `3020e7ea3c68ac2bdec5e50a91a0232fedd503f0`

## Required Reading

- `docs/PROJECT_CONTROL.md`
- `docs/AGENT_WORKFLOW.md`
- `docs/CURRENT_CLINE_TASK.md` (J13C-A packet)
- `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md`
- `docs/GIT_WORKTREES_AND_LINE_ENDINGS_FOR_AGENTS.md`
- `docs/DECISIONS.md` (J12/J13/J14 boundaries)
- `docs/ROAD_TO_0_2.md` (J13/J14 sections)

## Commits on this branch

| Role | SHA |
|------|-----|
| Implementation | `180a2feec6f8fd889e955d6f42c141e95602e337` |
| Documentation checkpoint | `fdb6327bba8ce5abb784293a101e1d8029fcfbdd` |
| Rejected report head | `3d27b2c8d1aa5905bc55fae1b48430707ddab5f0` |
| Repair commit | `afca84106d61767a0468606616e3aedd68c170f1` |

The worker note previously contained the nonexistent SHA
`fdb6327d3e13e7fb14965d3e3fb6f5e24fa3d6e0` (incorrectly labelled as
implementation checkpoint). That has been removed.

## Reasoning Evidence

- Effective reasoning level: MEDIUM
- Required by packet: MEDIUM
- Configured via `.goose.json` in worktree root with `"thinking_effort": "medium"`
- Match: Yes

## Rejection of first COMPLETE report

The original J13C report at `3d27b2c8d1aa5905bc55fae1b48430707ddab5f0` was
rejected because:

1. **DECISIONS.md whole-file rewrite**: Line-ending conversion turned the diff
   into 890 additions / 879 deletions. `git diff --check` reported trailing
   whitespace on dozens of lines.

2. **UTF-8 reader boundary failure**: `read_line_limited` validated each
   `fill_buf()` chunk independently with `std::str::from_utf8`. A multibyte
   character split across internal `BufRead` buffer boundaries would be
   rejected.

3. **Entry text normalisation**: Matching entries were stored as
   `serde_json::Value` and re-serialised through `serde_json::to_string`,
   destroying original key order, internal whitespace, and number/string
   representation.

4. **Inconsistent checkpoint evidence**: The worker note mislabelled the
   documentation checkpoint as the implementation checkpoint and contained a
   nonexistent SHA.

## J13C-A repairs

### DECISIONS.md

- Restored exact base version with `git restore --source=3020e7ea`.
- Inserted one concise J13C decision entry immediately after `# Decisions`
  using byte-level array manipulation, preserving CRLF and all existing bytes.
- Result: 15 additions, 0 deletions in `git diff --numstat`.

### UTF-8 reader

- Replaced per-chunk UTF-8 validation with byte-level line accumulation in a
  `Vec<u8>` buffer.
- Each `fill_buf()` chunk is appended to the buffer without intermediate
  validation. Only after the complete physical line (terminated by LF) is
  accumulated is the buffer validated once with `std::str::from_utf8`.
- No `from_utf8_unchecked` remains.
- Test `j13c_utf8_split_across_buffer_boundary` uses `BufReader::with_capacity(1)`
  to force one-byte chunks; café (U+00E9 = 0xC3 0xA9) is split across
  boundaries and correctly accepted.

### Original entry text preservation

- `read_and_filter` now returns `Vec<String>` (raw validated JSON text) instead
  of `Vec<Value>`.
- The success envelope is constructed as a format string: `execution_id` and
  `trail_path` are escaped by `serde_json::to_string`; matching entries are
  joined directly as pre-validated JSON strings.
- `TrailResult` carries `json_output: String` instead of `envelope: CliEnvelope`.
- `main.rs` prints the output string directly instead of going through
  `emit_envelope_and_exit`.
- Error envelopes continue to use `CliEnvelope` and are serialised normally.

## New focused Rust tests (7 added, 28 total)

| Test | What it proves |
|------|---------------|
| `j13c_utf8_split_across_buffer_boundary` | Multibyte UTF-8 across 1-byte buffers succeeds |
| `j13c_preserves_non_alphabetical_key_order` | `{"z":1,"a":2}` keeps z before a |
| `j13c_preserves_internal_spaces` | `{  "x" :  1  }` spacing unchanged |
| `j13c_success_output_is_valid_json` | Output parses as JSON |
| `j13c_exact_original_text_in_entries` | Raw entry text appears verbatim in output |
| `j13c_crlf_preserves_internal_data` | CRLF strips only terminator, not data |
| `j13c_malformed_later_prevents_all_output` | Later malformed line fails the whole inspection |

## New public acceptance cases (3 added, 19 total)

| Case | What it proves |
|------|---------------|
| 17: Unicode value succeeds | café (U+00E9) in a Trail entry returns ok |
| 18: Key order/spacing preserved | `{ "z": 1, "execution_id": "...", "a": 2 }` appears with exact ordering and spacing |
| 19: One JSON document | stdout remains exactly one valid JSON document |

## Verification evidence (J13C-A repair)

### Rust

- `cargo fmt --check`: PASS
- `cargo check --locked`: PASS
- `cargo check --locked --tests`: PASS
- `cargo test --locked j13c_ -- --nocapture`: 35 passed (7 CLI + 28 trail), 0 failed
- `cargo test --locked`: 766 passed, 0 failed (40 lib + 697 bin + 29 integration)
- `cargo clippy --locked --all-targets --all-features`: PASS, 0 errors
- `cargo build --locked`: PASS
- `cargo build --locked --release`: PASS

### Public acceptance

- `test-j13c-trail.ps1`: 19 passed, 0 failed

### Regressions

- `test-j13a-check.ps1`: 25 passed, 0 failed
- `test-j13b-run.ps1`: 10 passed, 0 failed
- `check-fixtures.ps1`: 46 JSON + 30 JSONL valid
- `test-mcp-transcripts.ps1`: 15 cases PASS
- `test-engine.ps1`: PASS
- `demo.ps1`: PASS

### Integrity

- `check-tethers-task-packet.ps1`: PASS (control-v1/IN_PROGRESS)
- `Cargo.lock` SHA-256: `d323870ea02f09391a5d0d9aa0e9a701cf686a5ac005b840ee7218e70edb5602` (unchanged)
- `git diff --check` (working tree): exits 0 (LF→CRLF conversion warnings only, no trailing whitespace)

## Changed files (authorised paths only)

| File | Change |
|------|--------|
| `tethers-0.1/host-rust/src/main.rs` | Switched trail dispatch from `emit_envelope_and_exit` to direct `println!` + `exit` |
| `tethers-0.1/host-rust/src/trail_command.rs` | Byte-level line accumulation, raw text preservation, 28 focused tests |
| `tethers-0.1/scripts/test-j13c-trail.ps1` | 19 acceptance cases including Unicode and key-order preservation |
| `docs/CURRENT_CLINE_TASK.md` | Set to IN_PROGRESS, J13C-A corrected acceptance criteria |
| `docs/DECISIONS.md` | Restored from base + 15-line J13C decision insertion |
| `docs/worker-notes/2026-07-30-j13c-trail-command.md` | Corrected checkpoint evidence, rejection record, required reading |

## Remaining risks

None. The repair corrects three acceptance defects and one evidence defect
without changing CLI arguments, Trail writing, replay, engine, provider,
policy, or execution.

## Smallest next action

Lucy reviews pushed evidence and decides to accept or correct. J14 follows
after J13 is accepted.
