# Worker Note

Task: `J13C-A — preserve Trail text and repair acceptance evidence`
Task packet: `docs/CURRENT_CLINE_TASK.md`
Owner: `Goose`
Status: `COMPLETE`
Base commit: `3020e7ea3c68ac2bdec5e50a91a0232fedd503f0`
Implementation checkpoint: `0e8d56e592cdebf6dc66f38db738f31fff528348`

## Required Reading

- `docs/PROJECT_CONTROL.md`
- `docs/AGENT_WORKFLOW.md`
- `docs/CURRENT_CLINE_TASK.md` (J13C-A packet)
- `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md`
- `docs/GIT_WORKTREES_AND_LINE_ENDINGS_FOR_AGENTS.md`
- `docs/DECISIONS.md` (J12/J13/J14 boundaries)
- `docs/ROAD_TO_0_2.md` (J13/J14 sections)

## Requested outcome

Correct three acceptance defects in J13C and repair checkpoint evidence:

1. DECISIONS.md: restore base version and insert concise J13C decision with
   byte-preserving method — small additions-only diff, clean `git diff --check`.
2. UTF-8 reader: replace per-chunk validation with full-line byte accumulation,
   validate once with `std::str::from_utf8`, no `from_utf8_unchecked`.
3. Original entry text: store matching entries as raw validated strings, not
   `serde_json::Value`, preserving key order, internal whitespace, and lexical
   form.
4. Worker note: correct mislabelled checkpoints, remove nonexistent SHA, record
   rejection reasons, add required reading.

## Changes made

- `tethers-0.1/host-rust/src/main.rs`: Switched trail dispatch from
  `emit_envelope_and_exit` to direct `println!` + `std::process::exit`.
- `tethers-0.1/host-rust/src/trail_command.rs`: Byte-level line accumulation in
  `read_line_limited`, raw `Vec<String>` return from `read_and_filter`, manual
  JSON construction in `run_trail` for success envelopes. `TrailResult` now
  carries `json_output: String` + `exit_code: i32`. Added 7 focused tests (28
  total).
- `tethers-0.1/scripts/test-j13c-trail.ps1`: Added 3 cases (Unicode entry,
  key-order/spacing preservation, one-JSON-document verification). 19 cases
  total.
- `docs/CURRENT_CLINE_TASK.md`: Set to IN_PROGRESS, updated acceptance criteria,
  set to COMPLETE.
- `docs/DECISIONS.md`: Restored from base `3020e7ea` using `git restore`, then
  inserted 15-line J13C decision entry with byte-level array manipulation
  preserving CRLF and all existing bytes.
- `docs/worker-notes/2026-07-30-j13c-trail-command.md`: Corrected checkpoint
  evidence, recorded rejection reasons, added required reading.

## Decisions and assumptions

- `ExecutionId::parse` remains authoritative; no second parser.
- Byte-level line accumulation decouples UTF-8 validation from `BufRead` buffer
  boundaries. A multibyte character split across `fill_buf` chunks is correctly
  handled.
- Raw text preservation avoids `serde_json::Value` round-tripping for matching
  entries. The success envelope is built as a `format!` string with only
  `execution_id` and `trail_path` escaped by `serde_json::to_string`.
- Error envelopes continue to use `CliEnvelope` and are serialised normally.
- DECISIONS.md insertion detects the existing newline sequence (CRLF) and
  preserves every byte after the insertion point.
- The existing commits (180a2fe, fdb6327, 3d27b2c) are not rewritten. The
  repair commit undoes the defects at the working-tree level. The combined range
  diff `3020e7ea..HEAD` is clean.

## Commits on this branch

| Role | SHA |
|------|-----|
| Implementation | `180a2feec6f8fd889e955d6f42c141e95602e337` |
| Documentation checkpoint | `fdb6327bba8ce5abb784293a101e1d8029fcfbdd` |
| Rejected report head | `3d27b2c8d1aa5905bc55fae1b48430707ddab5f0` |
| Repair commit | `afca84106d61767a0468606616e3aedd68c170f1` |
| Evidence commit | `96e8629efa929bc83583d356f9d9eedf08f0cac1` |
| Packet heading fix | `1dde192b4c044f3b4694b48670d6b3ccb589df4c` |
| Implementation checkpoint | `0e8d56e592cdebf6dc66f38db738f31fff528348` |

The nonexistent SHA `fdb6327d3e13e7fb14965d3e3fb6f5e24fa3d6e0` (previously
mislabelled as implementation checkpoint) has been removed.

## Evidence

### Rust
- `cargo fmt --check`: PASS
- `cargo check --locked`: PASS
- `cargo check --locked --tests`: PASS
- `cargo test --locked j13c_ -- --nocapture`: 35 passed (7 CLI + 28 trail)
- `cargo test --locked`: 766 passed, 0 failed
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
- `check-tethers-task-packet.ps1`: PASS
- `Cargo.lock` SHA-256: `d323870ea02f09391a5d0d9aa0e9a701cf686a5ac005b840ee7218e70edb5602`
- `git diff --check 3020e7ea..HEAD`: exits 0, no output
- `git diff --numstat 3020e7ea..HEAD -- docs/DECISIONS.md`: 15 additions, 0 deletions
- Changed files: 7 (exactly the original authorised J13C paths)

### Focused test results of note

| Test | Result |
|------|--------|
| `j13c_utf8_split_across_buffer_boundary` | PASS — café (U+00E9) across 1-byte buffers |
| `j13c_preserves_non_alphabetical_key_order` | PASS — z before a |
| `j13c_preserves_internal_spaces` | PASS — exact spacing preserved |
| `j13c_exact_original_text_in_entries` | PASS — raw text in output |

## Discoveries

- The original `read_line_limited` validated each `fill_buf()` chunk with
  `std::str::from_utf8`, rejecting valid multibyte characters split across
  `BufRead` buffer boundaries. Fix: accumulate raw bytes for the complete
  physical line, validate once.
- `serde_json::Value` round-tripping destroys key order, internal whitespace,
  and number representation. Fix: store matching entries as validated raw
  strings, embed directly in a `format!`-built success envelope.
- The original DECISIONS.md diff was a whole-file line-ending conversion (890
  added / 879 deleted). Fix: `git restore` from base, byte-level insertion
  preserving CRLF.
- PowerShell here-strings with backticks require double-escaping for literal
  backtick characters.
- The packet checker requires specific worker-note section headings; section
  naming deviations cause failure.

## Remaining risks

None. The repair corrects three acceptance defects and one evidence defect
without changing CLI arguments, Trail writing, replay, engine, provider,
policy, or execution.

## Smallest next action

Lucy reviews pushed evidence and decides to accept or correct. J14 follows
after J13 is accepted.

## References

- Branch: `goose/j13c-trail-command`
- Base: `3020e7ea3c68ac2bdec5e50a91a0232fedd503f0`
- Final HEAD: `0e8d56e592cdebf6dc66f38db738f31fff528348`
- Remote: `origin/goose/j13c-trail-command`
