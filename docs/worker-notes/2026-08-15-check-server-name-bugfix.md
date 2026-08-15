# Check Command Provider Server-Name Bugfix

Task: `Check Command Provider Server-Name Bugfix`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Check Provider Server-Name Bugfix Agent`

Status: `COMPLETE`

Base commit: `7c9f846cf5c7681a919f321faf42657c386d99ca`

Implementation checkpoint: `ed786efbd156bbb4850a5c95077cae226eac5dcb`

## Requested outcome

Fix the Tethers `check` command so MCP provider initialization validates the
provider against the trusted capability manifest binding's `server_name`,
never against the provider's configured identity. Add a regression test proving
provider identity and MCP server name may legitimately differ, while preserving
the fail-closed behaviour when a provider reports a non-matching server name.

## Changes made

- `tethers-0.1/host-rust/src/check_command.rs`
  - `check_providers` now derives `expected_server_name` from
    `provider.capabilities.first().verified_manifest.manifest().binding.server_name`,
    mirroring the normal host run path in `host_execution.rs`
    (`launch_and_initialize_provider`).
  - The `mcp.initialize(...)` call now passes `expected_server_name` instead of
    `&stdio.provider_config.identity`.
  - Added two tests: the positive regression
    (`j13a_check_provider_uses_manifest_server_name_not_identity`) and a narrow
    negative trust proof (`j13a_check_provider_rejects_wrong_reported_server_name`),
    backed by a shared `check_server_name_runtime` fixture builder.

No other files changed. No provider protocol, manifest schema, config schema,
source language, replay, Trail, Runtime Plan, concurrency, or host execution
semantics were modified.

## Decisions and assumptions

- Used the existing production derivation (`first()` capability's manifest
  `binding.server_name`, `unwrap_or("")`) from `host_execution.rs`, so `check`
  and the normal run path share one server-name rule rather than inventing a
  second one.
- The `first()`-capability rule is the existing deterministic host-run model;
  no new validation was introduced and none was removed.

## Evidence

Run against implementation checkpoint `ed786efbd156bbb4850a5c95077cae226eac5dcb`.

- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- j13a_check_provider --test-threads=1`
  → `2 passed; 0 failed` (positive and negative regression tests).
- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml -- --check` → exit 0.
- `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml` → exit 0.
- `cargo check --locked --manifest-path tethers-0.1/host-rust/Cargo.toml` → exit 0.
- `cargo check --manifest-path tethers-0.1/host-rust/Cargo.toml --all-targets --all-features` → exit 0.
- `cargo test --manifest-path tethers-0.1/host-rust/Cargo.toml -- --test-threads=1`
  → `1550 passed; 0 failed; 2 ignored` (finished in 295.43s).
- `git diff --check` → clean.
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` →
  `PASS task packet consistency (control-v1/COMPLETE)`.

## Publication evidence

Branch `fix/check-provider-server-name` pushed normally to `origin`. Local
`HEAD` equals the resolved remote HEAD; full remote SHA and final Git status
are recorded in the completion report.

## Discoveries

None. The bug was a single-line misuse of provider identity as the expected MCP
server name in `check_providers`; the normal run path already used the manifest
binding's `server_name`.

## Remaining risks

None known within packet scope. The fixture manifest used by the regression test
keeps `binding.server_name` distinct from the configured provider identity, so
the positive case exercises the fix directly.

## Smallest next action

Lucy review and acceptance of this bugfix against the pushed branch
`fix/check-provider-server-name`.

## References

- `tethers-0.1/host-rust/src/check_command.rs`
- `tethers-0.1/host-rust/src/host_execution.rs` (server-name derivation at
  `launch_and_initialize_provider`)
- `tethers-0.1/host-rust/src/stdio_provider.rs` (`ManagedProvider::initialize`
  server-name validation)
- `tethers-0.1/scripts/tethers-stdio-fixture.ps1`
- `tethers-0.1/protocol/capability-manifests/fixture-ping.json`
