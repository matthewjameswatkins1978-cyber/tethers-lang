# Tethers 0.6 cold-agent proof

Date: 2026-09-10

This record covers the public cold-agent path on the reconciled Tethers 0.6
preparation branch. It uses a fresh temporary workspace and the checked-in
J14A fixture rather than a fabricated provider or an in-memory substitute.

## Plan is side-effect-free

Command:

```powershell
pwsh -NoProfile -File tethers-0.1/scripts/test-tethers-0.6-plan.ps1
```

Observed result:

```json
{"command":"plan","exit_code":0,"cli_schema":"tethers.cli/1","plan_schema":"tethers.plan/1","plan_action_count":1,"first_action_id":"action_1","execution_performed":false,"provider_invocations":0,"trail_execution_entries":0,"provider_marker_exists":false,"trail_exists":false}
```

This proves the structured Plan result is emitted without authority, provider
invocation, provider marker creation, or Trail mutation.

## Explicit execution, Trail, receipt, and replay

Command:

```powershell
pwsh -NoProfile -File tethers-0.1/scripts/test-j14a-complete-scenario.ps1
```

Observed result:

```text
TOTAL: 7 cases, 7 passed, 0 failed
ASSERTIONS: 121
FIRST EXECUTION ID: exec_227862f3-82a0-41aa-a44d-764407f4f0ab
```

The seven cases prove configuration/provider availability, side-effect-free
preview, one successful provider effect with a Result Anchor, ordered Trail
inspection, bounded `trail --receipt`, exact replay with the same execution
identity, and no mutation of committed scenario sources or `Cargo.lock`.
The replay case proves the provider `tools/call` count remains one.

## Release package

The local Windows x64 package was built with the 0.6 default:

```powershell
pwsh -NoProfile -File scripts/package-tethers-release.ps1 -Target windows-x64
```

Artifact: `tethers-0.6.0-windows-x64-final.zip`

SHA-256:

```text
5EB87A45027116E768DEE16BE8B2F8593EB279D5402C7177128EDD1D5D7EBE1C
```

The adjacent `.sha256` sidecar matches the independently recomputed archive
hash. The package contains the native host, Portable Workbench, 0.6 release
manual, security manual, benchmark manual, specification, examples, and the
generated `SHA256SUMS` manifest.

## Verification

- `just check`: passed with `RUSTFLAGS=-D warnings`.
- `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check`: passed.
- `opam exec --switch="D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml" -- dune build @all`: passed.
- `opam exec --switch="D:\\The Next Thing\\Tethers Lang\\tethers-0.1\\engine-ocaml" -- dune runtest --force`: passed; all wired OCaml checks reported pass.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File .\\tethers-0.1\\scripts\\check-fixtures.ps1`: passed (64 JSON and 32 JSONL fixtures valid).
- `test-engine.ps1`: fails on the existing `top-level` fixture because the fixture expects a pre-`core_environment` request while the current engine correctly requires `core_environment`.
- `test-mcp-transcripts.ps1`: fails on the existing initialization transcript because the expected tool description is older than the current engine description.
- Normal parallel full suite: 1,576 passed, 0 failed, 2 ignored in the main
  library target; all integration targets completed without failures. The
  barrier/concurrency harnesses now serialize their external-provider test
  lifetimes, and the Git scope fixture uses a 30-second test-only subprocess
  budget to remain stable under the full process-heavy suite. Production
  runtime limits are unchanged.
- Isolated serial C2-A3a: 16 passed, 0 failed.
- Isolated serial C3: 18 passed, 0 failed.
- Isolated serial C4: 8 passed, 0 failed.

The serial results provide focused confirmation for the repaired host-execution
groups; the normal parallel command is also green after the test-only
isolation repairs.
