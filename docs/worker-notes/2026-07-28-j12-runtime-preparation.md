# Worker Note: J12 Packet 2 - Prepared Runtime And Scope Closure

- **Task**: `J12 packet 2 prepared runtime and scope closure`
- **Task packet**: `docs/CURRENT_CLINE_TASK.md`
- **Owner**: `Goose`
- **Status**: `COMPLETE`
- **Base commit**: `d3dc4c112bf141ce4f96b0188f0ce65776026617`
- **Implementation checkpoint**: `129817bae363d23f0c69fa282ea934dbe0c74ca5`
- **Implementation correction**: `70dda0d41f721f87d3699e84f3ce28b8424645f1`
- **Documentation checkpoint**: `06a8f7c64e91126c2ffcf34583b11b6e59140633`
- **Documentation correction**: `c8e340711bf67be38828cb1cc4f1666244af1b51`

## Requested outcome

Implement J12 Packet 2: prepared local runtime, verified assets, deterministic
capability materialisation, and binding-owned path-scope assessment. Turn one
LoadedRuntimeConfig into a complete PreparedRuntime that J13 can use without
manually rebuilding internal objects. This packet closes J12.

Correction: implement strict filesystem escape classification, strict RFC 6901
JSON Pointer decoding, and tighter allowed-prefix validation.

## Changes made

### Initial implementation (`129817b`)

### `tethers-0.1/host-rust/src/runtime_config.rs`

- Tightened global capability-binding uniqueness: every exact `(name, version)`
  identity must appear under exactly one configured provider.
- Added `pub(crate) decode_strict_pointer_token` and `pub(crate) is_valid_array_index`.
- Updated `validate_json_pointer` with strict token-by-token validation.
- Added test `j12_packet2_duplicate_unscoped_across_providers_rejected`.

### `tethers-0.1/host-rust/src/main.rs`

- Registered `pub mod configured_runtime`.
- Fixed pre-existing `let mut` warning.

### `tethers-0.1/host-rust/src/configured_runtime.rs` (new module)

Structures:
- `PreparedRuntime` with 9 fields and read-only accessors
- `PreparedTether` (id, version, source_path, source)
- `PreparedProvider` (identity, display_name, working_directory, stdio_config, capabilities)
- `PreparedCapability` (name, version, manifest_path, verified_manifest, scope_binding)
- `RuntimePreparationError` with 18 structured error codes

Functions:
- `prepare_runtime`, `confine_asset`, `read_utf8_asset`
- `validate_resource_path`, `validate_allowed_prefixes`
- `assess_action_scope`, `planner_capabilities`, `tether_material`
- `convert_input_schema`, `check_relative_path_safe`

45 focused tests.

### Correction (`70dda0d`)

#### `tethers-0.1/host-rust/src/configured_runtime.rs`

1. **Filesystem escape classification**: Replaced `joined.starts_with(config_dir)`
   fallback with `check_relative_path_safe()` — a pure relative-path preflight
   that counts ParentDir components and rejects any `..` that would move above
   the root. Absolute paths and root-prefixed paths are rejected before join.
   The `confine_asset` function now uses this preflight, then falls back to
   `canonicalize()` for existing-file containment (symlink escapes, etc.).

2. **Strict RFC 6901 JSON Pointer**: Replaced permissive `replace("~1", "/")`
   with shared `crate::runtime_config::decode_strict_pointer_token`. Rules:
   `~0`→`~`, `~1`→`/`, any other `~` sequence malformed. Array-index tokens
   validated via `is_valid_array_index` (rejects `01`, `+1`, `-1`, whitespace).
   Malformed configured pointers rejected during config validation.
   `extract_json_pointer` uses strict decoding; malformed tokens at runtime
   return `ScopeNotEstablished` defensively.

3. **Tightened allowed-prefix validation**: Rewrote `validate_allowed_prefixes`
   to reject `/` (root), leading `/`, `\\`, NUL, `.` and `..` segments, empty
   interior segments, and repeated trailing slashes (`projects//`). Permits
   `projects` and `projects/`.

#### `tethers-0.1/host-rust/src/runtime_config.rs`

- Added `pub(crate) fn decode_strict_pointer_token` and
  `pub(crate) fn is_valid_array_index` as shared helpers.
- Updated `validate_json_pointer` to validate every token through
  `decode_strict_pointer_token`.

15 new correction tests (46–60).

## Decisions and assumptions

1. **Global exact-identity uniqueness**: Every `(name, version)` must appear
   under exactly one provider. Stricter than Packet 1 scoped-only rule.
2. **Pure component-counting escape detection**: `check_relative_path_safe` uses
   `std::path::Component::ParentDir` counting only. Does not access the
   filesystem. Existing-file symlink containment still uses `canonicalize()`.
3. **Shared strict pointer decoding**: `decode_strict_pointer_token` and
   `is_valid_array_index` are `pub(crate)` in runtime_config.rs, shared by
   both configuration validation and runtime pointer extraction.
4. **Per-capability ProviderConfig for admission**: Same-name multi-version
   providers use single-capability ProviderConfigs during admission.
5. **No direct TrustedManifestStore bypass**: All admission through
   `provider::admit_provider_manifest`.
6. **Pure scope assessment**: No I/O in `assess_action_scope`.
7. **J13/J14 boundaries preserved**: No provider launch, engine invocation,
   dispatch, or Trail writing in Packet 2.

## Evidence

### Rust tests

- Packet 1 tests: 35/35 PASS
- Packet 2 tests (original): 45/45 PASS
- Packet 2 correction tests: 15/15 PASS
- Combined J12 tests: 96/96 PASS
  (35 Packet 1 + 1 Packet 2 in runtime_config + 45 original + 15 correction = 96)
- Full Rust suite: 617/617 PASS

### Build

- `cargo fmt --check`: PASS
- `cargo check`: PASS
- `cargo check --tests`: PASS
- `cargo build`: PASS
- `cargo build --release`: PASS
- `cargo clippy --all-targets --all-features`: zero new warnings

### Integration scripts

| Script | Result |
|---|---|
| `check-fixtures.ps1` | PASS (46 JSON, 30 JSONL) |
| `test-engine.ps1` | PASS (24/24) |
| `test-mcp-transcripts.ps1` | PASS (15/15) |
| `test-host-denial.ps1` | PASS |
| `test-host-execution-failure.ps1` | PASS |
| `test-host-result-follow-up.ps1` | PASS |
| `test-host-event-admission.ps1` | PASS |
| `test-host-event-admission-trail.ps1` | PASS |
| `demo.ps1` | PASS |
| `check-tethers-task-packet.ps1` | PASS |
| `opam exec -- dune build` | PASS |

### Control-character scan

PASS (all four authorised correction files)

### Git

- Implementation checkpoint: `129817bae363d23f0c69fa282ea934dbe0c74ca5`
- Implementation correction: `70dda0d41f721f87d3699e84f3ce28b8424645f1`
- Documentation checkpoint: `06a8f7c64e91126c2ffcf34583b11b6e59140633`
- Documentation correction: `c8e340711bf67be38828cb1cc4f1666244af1b51`

## Discoveries

1. `Path::starts_with()` does not normalise `..` components, so the original
   fallback could not reliably detect nonexistent `../` escapes. Replaced with
   pure `std::path::Component` counting in `check_relative_path_safe`.
2. `admit_provider_manifest` matches by name first, then version. For
   multi-version same-name providers, a per-capability ProviderConfig is needed.
3. The original `pointer_tokens` used `String::replace` which accepts any `~`
   sequence — not valid per RFC 6901. Replaced with strict character-by-character
   decoding.

## Remaining risks

- Symlink escape not directly tested on Windows (requires admin privilege).
  Confinement tested with `..` parent-directory escape.
- No live provider admission (J13 responsibility).

## Smallest next action

Lucy's J12 acceptance review, then J13 design.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/DECISIONS.md`
- `tethers-0.1/host-rust/src/configured_runtime.rs`
- `tethers-0.1/host-rust/src/runtime_config.rs`
- RFC 6901 — JavaScript Object Notation (JSON) Pointer
