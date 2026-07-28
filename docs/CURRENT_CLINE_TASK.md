# Current Implementation Task

Control contract: `1`

Task: `J12 packet 1 strict minimal local runtime configuration foundation`

Status: `IN_PROGRESS`

Task colour: `Green`

Owner: `Goose`

Route: `Goose - J12 packet 1 in local worktree`

Worker note: `docs/worker-notes/2026-07-28-j12-config-foundation.md`

Base branch: `goose/j11-event-trail-final`

Base commit: `f0a76ee3782f5b7d2d7120e1b36100f5fa465acb`

Branch: `goose/j12-config-foundation`

## Expected pre-existing changes

None.

## Objective

Freeze and implement the smallest strict JSON configuration that selects
one identified and versioned Tether Set, its ordered source files, exact
capability requirements, explicit stdio provider bindings, reviewed manifest
files with pinned digests, scope bindings, and exact local policy rules.

This packet implements parsing, validation and materialisation only.

It must not yet launch a provider, admit a manifest, invoke the engine,
assess a live Action, dispatch, write a Trail, or create a J13 command.

## Relevant background and existing behaviour

J11 Packet 4 completed the event-admission Trail and finalised the J11
implementation cycle at `f0a76ee`. The host reference application currently
reads engine output and dispatches through a hardwired main.rs route with no
configuration-driven Tether Set selection or provider binding. Policy,
provider identity, and capability resolution are wired through CLI arguments
and in-memory structures.

The `manifest.rs` module provides `parse_value_no_dupes` (strict
duplicate-key-rejecting JSON parser), `TrustedManifest::parse` (serde-based
manifest parsing with `deny_unknown_fields`), and semantic validation.
`provider.rs` defines `ProviderConfig` and `AllowedCapability` with public
fields. `policy.rs` defines `CapabilityRequirement`, `HostLocalPolicy`, and
`PolicyRule` with public constructors. `stdio_provider.rs` defines
`StdioProviderConfig` with public fields.

## Required behaviour

1. Expose `manifest::parse_value_no_dupes` as `pub(crate)`.
2. Create `runtime_config.rs` with the frozen J12 JSON schema types,
   strict parsing (`parse_value_no_dupes` + serde with `deny_unknown_fields`
   + semantic validation), structured errors, and materialisation helpers.
3. Define `parse_runtime_config` and `load_runtime_config` public functions.
4. `LoadedRuntimeConfig` must retain the parsed config, absolute config file
   path, absolute parent directory, and resolve relative source/manifest paths
   against that parent.
5. Materialisation helpers must produce `Vec<CapabilityRequirement>`,
   `HostLocalPolicy`, and `ProviderMaterialization` intermediates carrying
   scope bindings alongside the data for `ProviderConfig` and
   `StdioProviderConfig`.
6. Where direct materialisation would require modifying an unauthorised
   module, return a clearly typed intermediate and document the Packet 2
   wiring seam.
7. Add exactly 22+ focused tests with `j12_packet1_` prefix covering the
   complete validation matrix.
8. Add a J12 decision to `docs/DECISIONS.md` freezing the exact schema.

## Relevant components

- `tethers-0.1/host-rust/src/manifest.rs` - expose `parse_value_no_dupes` as `pub(crate)`
- `tethers-0.1/host-rust/src/main.rs` - register `pub mod runtime_config`
- `tethers-0.1/host-rust/src/runtime_config.rs` - new module (parsing, validation, materialisation, tests)
- `docs/DECISIONS.md` - J12 decision
- `docs/CURRENT_CLINE_TASK.md` - this file
- `docs/worker-notes/2026-07-28-j12-config-foundation.md` - worker note

## Frozen decisions and invariants

- The JSON format is frozen per the task packet schema in `docs/DECISIONS.md`.
- Only `format_version` "0.1" is accepted.
- One configuration file selects one Tether Set with ordered source paths.
- Providers are explicitly configured; only `stdio` transport is supported.
- Every provider capability must pin an exact `sha256:` digest with 64
  lowercase hex characters.
- Policy default must be exactly `deny`; rules may be `allow`, `ask`, or `deny`.
- Scope bindings are `path_prefix` with a JSON Pointer argument extraction;
  the manifest's `allowed_prefixes` remain authoritative.
- Binding-owned pointer extraction; manifest-owned allowed prefixes; host-owned
  scope assessment.
- Relative paths are resolved against the configuration file's parent directory.
- No secrets, interpolation, package management, discovery, wildcards, or J13
  commands.
- Unsupported structured scope kinds fail closed (Packet 2).
- Packet 2 owns runtime wiring and live scope assessment.

## Acceptance criteria

1. All 554 Rust tests pass (522 existing + 32 new).
2. `cargo fmt --check` reports no diffs.
3. `cargo clippy --all-targets --all-features` produces zero new warnings.
4. `cargo build` and `cargo build --release` succeed.
5. `check-fixtures.ps1` passes.
6. `test-engine.ps1` passes (24/24).
7. `test-mcp-transcripts.ps1` passes (15/15).
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
- `policy.rs`, `provider.rs`, `stdio_provider.rs` - not modified
- `resolver.rs`, `trusted_store.rs` - not modified
- `dispatch.rs`, `replay.rs`, `event_admission.rs` - not modified
- `OCaml`, protocol fixtures, existing scripts - not modified
- `ROAD_TO_0_2.md` - not modified
- J11 worker note - not modified
- Project-local evidence skill - not modified
- No new dependencies

## Stop conditions

- Task packet checker fails due to missing sections.
- Any mandatory script produces unexpected results beyond documented pre-existing issues.
- Git status is not clean after expected changes.
- Branch cannot be pushed or remote SHA does not match local.
- Base commit does not resolve or is not an ancestor of HEAD.
