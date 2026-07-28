# Current Implementation Task

Control contract: `1`

Task: `J12 packet 2 prepared runtime and scope closure`

Status: `COMPLETE`

Task colour: `Green`

Owner: `Goose`

Route: `Goose - J12 packet 2 in local worktree`

Worker note: `docs/worker-notes/2026-07-28-j12-runtime-preparation.md`

Base branch: `goose/j12-config-foundation`

Base commit: `d3dc4c112bf141ce4f96b0188f0ce65776026617`

Branch: `goose/j12-runtime-preparation`

## Expected pre-existing changes

None.

## Relevant background and existing behaviour

J12 Packet 1 completed strict local runtime configuration parsing, validation,
and materialisation at `d3dc4c11`. The `runtime_config` module provides
`parse_runtime_config`, `load_runtime_config`, `LoadedRuntimeConfig`, and
materialisation helpers. `manifest::verify_manifest` verifies manifests.
`provider::admit_provider_manifest` admits them through the trusted store.
`policy.rs` defines `ProposedAction`, `ScopeAssessment`, `CapabilityRequirement`,
and `HostLocalPolicy`. `stdio_provider.rs` defines `StdioProviderConfig`.

Packet 2 turns a `LoadedRuntimeConfig` into a `PreparedRuntime` that J13 can
use without manually rebuilding internal objects. It reads files from disk,
verifies manifests, admits them, validates scope bindings, and builds immutable
runtime state.

## Objective

Turn one LoadedRuntimeConfig into a complete PreparedRuntime that J13 can use
without manually rebuilding internal objects.

PreparedRuntime contains:

- the selected Tether Set identity and version;
- all Tether source files loaded in exact configured order;
- all reviewed manifests read, verified, pinned and admitted;
- one deterministic provider launch plan per configured provider;
- exact capability requirements;
- exact default-deny host policy;
- planner capability descriptors derived from verified manifests;
- an exact capability-to-provider binding;
- binding-owned live Action scope assessment.

This packet performs filesystem loading and trusted local preparation.

It performs no provider, engine, dispatch or Trail I/O.

## J12 Packet 2 owns

- loading configured Tether source files;
- loading and verifying reviewed manifests;
- exact digest and identity cross-checking;
- admission into an in-memory TrustedManifestStore;
- deterministic provider launch plans;
- deterministic planner capability descriptors;
- binding-specific path-prefix scope assessment;
- complete PreparedRuntime assembly.

## J13 owns

- public check, run and trail commands;
- provider launch and live availability snapshots;
- OCaml engine invocation;
- assembling one explicit Anchor and Facts input with PreparedRuntime;
- locating and printing Trail records.

## J14 owns

- proving an actual configured provider capability call;
- intent, dispatch, validated output and Result Anchor through the public route;
- the complete positive scenario and negative integration matrix.

## Required behaviour

1. Tighten RuntimeConfig validation: every exact `(name, version)` must appear
   under exactly one configured provider. Reject duplicates across providers
   whether or not scope_binding is present.
2. Create `configured_runtime.rs` with `PreparedRuntime`, `PreparedTether`,
   `PreparedProvider`, `PreparedCapability`, structured error types, and
   `prepare_runtime` function.
3. Implement asset confinement: resolve, canonicalise, require beneath config
   dir, require regular file, read as UTF-8, reject NUL.
4. Manifest preparation: read, verify, cross-check name/version/provider/digest,
   validate scope-binding compatibility, admit through provider boundary.
5. Implement binding-owned scope assessor with JSON Pointer extraction,
   segment-precise prefix matching, and fail-closed behaviour.
6. Implement deterministic planner capability descriptors from verified manifests.
7. Provide tether_material accessor preserving configuration order.
8. Build provider launch plans with literal command/args and config dir as
   working directory.
9. Add at least 35 focused tests with `j12_packet2_` prefix.
10. Document J12 Packet 2 decision in DECISIONS.md, correcting Packet 1 wording.

## Relevant components

- `tethers-0.1/host-rust/src/configured_runtime.rs` - new module (PreparedRuntime, preparation, scope assessment, tests)
- `tethers-0.1/host-rust/src/main.rs` - register `pub mod configured_runtime`
- `tethers-0.1/host-rust/src/runtime_config.rs` - global exact-identity uniqueness validation
- `docs/DECISIONS.md` - J12 Packet 2 decision
- `docs/CURRENT_CLINE_TASK.md` - this file
- `docs/worker-notes/2026-07-28-j12-runtime-preparation.md` - worker note

## Frozen decisions and invariants

- All exact capability identities are globally unique across providers.
- Assets are confined beneath the config directory.
- Reviewed manifests are verified, pinned and admitted.
- Tether order is preserved.
- Planner descriptors come from verified manifests.
- Providers receive launch plans but are not started.
- Path scope uses configured JSON Pointer extraction and manifest-owned prefixes.
- Unsafe or unsupported scopes fail closed.
- PreparedRuntime is immutable after construction.
- J12 ends at runtime preparation.
- J13 owns public check/run/trail and process launch.
- J14 owns actual provider dispatch proof.

## Acceptance criteria

1. All Rust tests pass.
2. `cargo fmt --check` reports no diffs.
3. `cargo clippy --all-targets --all-features` produces zero new warnings.
4. `cargo build` and `cargo build --release` succeed.
5. `check-fixtures.ps1` passes.
6. `test-engine.ps1` passes.
7. `test-mcp-transcripts.ps1` passes.
8. `test-host-denial.ps1` passes.
9. `test-host-execution-failure.ps1` passes.
10. `test-host-result-follow-up.ps1` passes.
11. `test-host-event-admission.ps1` passes.
12. `test-host-event-admission-trail.ps1` passes.
13. `demo.ps1` passes.
14. `check-tethers-task-packet.ps1` passes.
15. `opam exec -- dune build` succeeds.
16. Branch pushed to origin with matching local/remote SHA.

## Required verification

```powershell
cargo fmt --check
cargo check && cargo check --tests
cargo test j12_packet1_ -- --nocapture
cargo test j12_packet2_ -- --nocapture
cargo test j12_ -- --nocapture
cargo test
cargo clippy --all-targets --all-features
cargo build && cargo build --release
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
pwsh -NoProfile -File tethers-0.1/scripts/check-fixtures.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-engine.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-mcp-transcripts.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-host-denial.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-host-execution-failure.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-host-result-follow-up.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-host-event-admission.ps1
pwsh -NoProfile -File tethers-0.1/scripts/test-host-event-admission-trail.ps1
pwsh -NoProfile -File tethers-0.1/scripts/demo.ps1
opam exec -- dune build
```

## Forbidden changes

- `Cargo.toml`, `Cargo.lock` - not modified
- `manifest.rs`, `policy.rs`, `provider.rs`, `stdio_provider.rs` - not modified
- `resolver.rs`, `trusted_store.rs` - not modified
- `dispatch.rs`, `replay.rs`, `event_admission.rs` - not modified
- OCaml, protocol fixtures, existing scripts - not modified
- `ROAD_TO_0_2.md` - not modified
- J11 or Packet 1 worker notes - not modified
- Project-local evidence skill - not modified
- No new dependencies

## Stop conditions

- Task packet checker fails due to missing sections.
- Any mandatory script produces unexpected results.
- Git status is not clean after expected changes.
- Branch cannot be pushed or remote SHA does not match local.
- Base commit does not resolve or is not an ancestor of HEAD.
