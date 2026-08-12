# Worker Note: CORE-9A Rust Semantic Environment Authority

Task: `TETHERS CORE-9A — Rust Semantic Environment Authority`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `81722867840c3adf03794cbaeff761f414a96301`

Implementation checkpoint: `c1a46c26815cfeb3999a97a6bd0e51e16cbdd87f`

## Requested outcome

Extend the Rust runtime configuration and preparation layer so each configured
Tether MAY carry the explicit semantic environment required by the CORE-8B
request boundary. Authority and preparation only; no production evaluation
injection yet.

## Changes made

### runtime_config.rs

- Added `CoreEnvironmentConfig`, `CoreCapabilityBindingConfig`,
  `CoreInputFactBindingConfig`, `CoreScalarType` types with
  `#[serde(deny_unknown_fields)]` and `#[serde(rename_all = "snake_case")]`
- Added `core_environment: Option<CoreEnvironmentConfig>` to `TetherRef`
  with `#[serde(default)]`
- Added `validate_core_environment()` function for structural host-owned
  invariants (9A.5): non-empty/non-whitespace for all semantic identity
  fields, scalar_type validated by enum
- Added runtime_name join validation to `validate_cross_references()` (9A.6):
  each core_environment capability binding's runtime_name must match exactly
  one provider capability by name; 0 matches = UnmatchedReference,
  2+ matches = DuplicateEntry
- Added config parsing tests: T1-T11 (14 tests total)

### configured_runtime.rs

- Added `PreparedCoreEnvironment`, `PreparedCoreCapabilityBinding`,
  `PreparedCoreInputFactBinding` types
- Added `core_environment: Option<PreparedCoreEnvironment>` to `PreparedTether`
- Carried core_environment through `prepare_runtime()` with exact string
  preservation
- Added `PreparedTether::core_environment_json()` pure serializer producing
  the exact CORE-8B JSON wire shape
- Added preparation and serialization tests: T1-T14 + adversarial test
  (16 tests total)

### host_execution.rs

- Added `core_environment: None` to three existing `PreparedTether` initializers

## Decisions and assumptions

- core_environment is optional via `#[serde(default)]` on TetherRef; absent
  means no Core semantic authority, existing configs remain valid
- scalar_type is a Rust enum with `#[serde(rename_all = "snake_case")]`;
  serde rejects invalid values at the type level
- Runtime name join validation (9A.6) runs at config parse time in
  `validate_cross_references()` since all provider capability names are
  available in the config
- The `core_environment_json()` method is a pure serializer on PreparedTether;
  it does not read filesystem, clock, or environment
- No derivation logic exists anywhere; every identity is a pass-through of
  the explicit config value

## Evidence

### Configuration parsing

- `cargo fmt --check` -- PASS (clean after cargo fmt)
- `cargo check` -- PASS
- `cargo test` -- PASS (1431 passed, 0 failed, 2 ignored)
- `git diff --check` -- PASS (LF/CRLF warnings only)

### Diff inspection

Changed files (3):
- `tethers-0.1/host-rust/src/runtime_config.rs` (+524 lines)
- `tethers-0.1/host-rust/src/configured_runtime.rs` (+577 lines)
- `tethers-0.1/host-rust/src/host_execution.rs` (+3 lines)

Unchanged:
- `tethers-0.1/engine-ocaml/` -- zero diff
- No production request construction modified
- No MCP schemas modified
- No policy changes
- No Trail changes

### Tests added

**runtime_config.rs** (config parsing, 14 tests):
- core9a_t1_existing_config_without_core_env_valid
- core9a_t2_explicit_environment_parses
- core9a_t3_program_id_not_derived
- core9a_t4_four_capability_identities_differ
- core9a_t5_manifest_digest_distinct_from_contract
- core9a_t6_explicit_fact_identities
- core9a_t7_scalar_types_all_accepted
- core9a_t8_invalid_scalar_type_rejected
- core9a_t9_missing_runtime_name_target
- core9a_t10_ambiguous_runtime_name
- core9a_t11_empty_program_id_rejected (+ 8 more T11 variants)

**configured_runtime.rs** (preparation + serialization, 16 tests):
- core9a_t1_prepare_no_core_env
- core9a_t2_prepare_explicit_environment
- core9a_t3_program_id_not_derived_prepare
- core9a_t4_capability_identities_prepare
- core9a_t5_contract_digest_not_manifest
- core9a_t6_fact_identities_prepare
- core9a_t7_all_scalar_types_prepare
- core9a_t12_exact_json_projection
- core9a_t13_no_env_no_json
- core9a_t14_serializer_preserves_order
- core9a_adversarial_no_derivation

## Publication evidence

Branch: `feature/core-9a-rust-semantic-environment`
(Push pending — task packet says STOP after implementation checkpoint.)

## Discoveries

- The existing `host_execution.rs` test fixtures construct `PreparedTether`
  directly, requiring `core_environment: None` additions
- `verified_digest()` returns `&str`, requiring `.as_str()` when comparing
  with `String` in tests

## Remaining risks

None known within packet scope. core_environment is dormant; production
request injection will be a separate task.

## Smallest next action

Push branch to origin and stop for independent Lucy review.

## References

- Task packet: `docs/CURRENT_CLINE_TASK.md`
- CORE-8B request boundary: `tethers-0.1/engine-ocaml/bin/tethers_core_request_adapter.ml`
- Runtime config types: `tethers-0.1/host-rust/src/runtime_config.rs`
- Prepared runtime: `tethers-0.1/host-rust/src/configured_runtime.rs`
- Host execution: `tethers-0.1/host-rust/src/host_execution.rs`
- Implementation checkpoint: `c1a46c26815cfeb3999a97a6bd0e51e16cbdd87f`
