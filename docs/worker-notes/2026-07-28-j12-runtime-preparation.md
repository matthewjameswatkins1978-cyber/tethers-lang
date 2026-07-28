# Worker Note: J12 Packet 2 - Prepared Runtime And Scope Closure

- **Task**: `J12 packet 2 prepared runtime and scope closure`
- **Task packet**: `docs/CURRENT_CLINE_TASK.md`
- **Owner**: `Goose`
- **Status**: `COMPLETE`
- **Base commit**: `d3dc4c112bf141ce4f96b0188f0ce65776026617`
- **Implementation checkpoint**: `129817bae363d23f0c69fa282ea934dbe0c74ca5`
- **Documentation checkpoint**: `PLACEHOLDER`

## Requested outcome

Implement J12 Packet 2: prepared local runtime, verified assets, deterministic
capability materialisation, and binding-owned path-scope assessment. Turn one
LoadedRuntimeConfig into a complete PreparedRuntime that J13 can use without
manually rebuilding internal objects. This packet closes J12.

## Changes made

### `tethers-0.1/host-rust/src/runtime_config.rs`

- Tightened global capability-binding uniqueness: every exact `(name, version)`
  identity must appear under exactly one configured provider. Removed the
  narrower scoped-only global rule. Duplicates are rejected whether or not
  `scope_binding` is present.
- Added test `j12_packet2_duplicate_unscoped_across_providers_rejected` for
  two unscoped cross-provider duplicates.
- Updated existing test assertions for the broader error message.

### `tethers-0.1/host-rust/src/main.rs`

- Registered `pub mod configured_runtime`.
- Fixed pre-existing `let mut` warning (unused mut).

### `tethers-0.1/host-rust/src/configured_runtime.rs` (new module)

Structures:
- `PreparedRuntime` with read-only accessors (config_path, config_dir,
  tether_set_id/version, tethers, requirements, providers, policy, trusted_store)
- `PreparedTether` (id, version, source_path, source)
- `PreparedProvider` (identity, display_name, working_directory, stdio_config, capabilities)
- `PreparedCapability` (name, version, manifest_path, verified_manifest, scope_binding)
- `RuntimePreparationError` with 18 structured error codes
- `RuntimePreparationErrorCode` enum

Functions:
- `prepare_runtime(loaded: &LoadedRuntimeConfig) -> Result<PreparedRuntime, RuntimePreparationError>`
- `confine_asset` - filesystem confinement with escape detection
- `read_utf8_asset` - UTF-8 reading with NUL rejection
- `validate_resource_path` - safe relative logical path validation
- `validate_allowed_prefixes` - manifest prefix validation
- `assess_action_scope` - pure scope assessment with JSON Pointer extraction
- `planner_capabilities` - deterministic pin-free planner descriptors
- `tether_material` - ordered Tether JSON projection
- `convert_input_schema` - scalar-type-only input conversion

Tests: 45 focused tests (`j12_packet2_` prefix) covering:
- Valid preparation, Tether order, source retention
- Missing/empty/directory/escape/NUL/whitespace tether source
- Missing/invalid manifest, name/version/provider/digest mismatches
- Store population, same-name-different-versions
- Scope binding compatibility (PathPrefix, Unrestricted, Repository)
- Scope assessment (WithinScope, ScopeViolation, ScopeNotEstablished)
- Planner descriptors (deterministic, pin-free, sorted, unsupported schema)
- Tether material, provider launch plan, working directory
- Accessors, prepared_capabilities, JSON Pointer escaping, input schema ordering

### Documentation

- `docs/CURRENT_CLINE_TASK.md` - updated to J12 Packet 2
- `docs/DECISIONS.md` - J12 Packet 2 decision appended, Packet 1 boundary corrected
- `docs/worker-notes/2026-07-28-j12-runtime-preparation.md` - this file

## Decisions and assumptions

1. **Global exact-identity uniqueness**: Every `(name, version)` must appear
   under exactly one provider. This is stricter than the Packet 1 scoped-only
   rule and prevents non-deterministic provider selection.
2. **Asset confinement**: Uses `Path::canonicalize()` with a pre-check for
   non-existent escape paths (Windows requires file existence for
   canonicalization). Symlink escape is not directly tested on Windows where
   symlink creation requires admin privilege.
3. **Per-capability ProviderConfig for admission**: When a provider exposes
   multiple capabilities with the same name at different versions,
   `admit_provider_manifest` receives a single-capability ProviderConfig to
   ensure exact version matching, as the existing admission function matches
   by name first.
4. **No direct TrustedManifestStore bypass**: All manifest admission goes
   through `provider::admit_provider_manifest`.
5. **Pure scope assessment**: `assess_action_scope` performs no I/O, no
   environment access, no provider lookup.
6. **J13/J14 boundaries preserved**: No provider launch, engine invocation,
   dispatch, or Trail writing in Packet 2.

## Evidence

### Rust tests

- Packet 1 tests: 35/35 PASS
- Packet 2 tests: 45/45 PASS
- Combined J12 tests: 81/81 PASS
- Full Rust suite: 602/602 PASS

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

PASS (all six authorised files)

### Git

- Implementation commit: `129817bae363d23f0c69fa282ea934dbe0c74ca5`
- Documentation commit: `PLACEHOLDER`
- Exactly six authorised files

## Discoveries

1. Windows `Path::canonicalize()` requires the file to exist, so path-escape
   detection for non-existent files needs a pre-check using `Path::starts_with`
   on the joined path.
2. `admit_provider_manifest` matches capabilities by name first, then checks
   version. For multi-version same-name providers, a per-capability
   ProviderConfig is needed during admission.
3. The existing `provider_materializations()` correctly maps all capabilities
   from each provider, preserving scope bindings.

## Remaining risks

- Symlink escape not directly tested (requires Windows admin privilege for
  symlink creation). Confinement is tested with `../` parent-directory escape.
- No live provider admission (J13 responsibility).

## Smallest next action

Lucy's J12 acceptance review, then J13 design.

## References

- `docs/CURRENT_CLINE_TASK.md`
- `docs/DECISIONS.md`
- `tethers-0.1/host-rust/src/configured_runtime.rs`
- `tethers-0.1/host-rust/src/runtime_config.rs`
- `tethers-0.1/host-rust/src/provider.rs`
- `tethers-0.1/host-rust/src/manifest.rs`
