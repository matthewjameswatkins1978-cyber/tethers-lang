# Worker Note

Task: `J20-ENV-P1 - Execution Environment Handshake`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `e57bf536fe3d7fb074c00ddac867b5720a15116e`

Implementation checkpoint: `ddb582d46049c93724928c03e40888e425c7517e`

## Requested outcome

Create a host-owned one-shot execution-environment handshake with one shared
Tethers workbench, small worker overlays, immutable task contracts, exact
command binding, and process-tree supervision. Keep it outside the OCaml Core
and do not begin M5 or M6.

## Changes made

- Added the accepted execution-environment architecture, shared workbench
  profile, small worker overlays, contract schema, and Rust-host request example.
- Added `check-tethers-environment.ps1`, which reuses the existing developer
  tool diagnostic and records profile-gated real command probes as JSON.
- Added the Rust issuer/permit boundary. It binds Matthew's worker selection,
  task/session/repository facts, request/observation/contract digests, version
  policy, permissions, exact program/argument/cwd tuples, and the existing
  supervised child launcher.
- Updated the current packet for this completed Red task and ignored local
  handshake artifacts.

## Decisions and assumptions

The reusable asset is one `tethers-development-workbench-v1` profile; the four
worker entries are role constraints only. A required absence blocks, preferred
absence degrades, replaceable absence is recorded, and optional absence does
not degrade. The Job Object launch path is used for process-tree ownership but
is explicitly not presented as a filesystem, network, or hostile-code sandbox.

## Evidence

- `pwsh -NoProfile -File scripts/check-dev-tools.ps1` — PASS; all repository
  developer-tool commands resolved.
- JSON parsing and PowerShell script parse for the profile, overlays, schema,
  example, and probe script — PASS.
- `rustfmt +1.89.0 --check src/execution_environment.rs` — PASS.
- `cargo +1.89.0 test execution_environment --locked` — PASS; 6 new focused
  issuer/permit tests.
- `cargo +1.89.0 check --all-targets --all-features --locked` — PASS.
- `opam exec --switch=D:\The Next Thing\Tethers Lang - J16 Clean\tethers-0.1\engine-ocaml -- dune build` from this worktree's engine directory — PASS.
- `cargo +1.89.0 test --all-targets --all-features --locked` — PASS; 811
  library tests, 29 CLI tests, 13 M3 lifecycle tests, and 4 M4 File Tools tests.
- `pwsh -NoProfile -File scripts/check-tethers-environment.ps1 -Profile rust-host` — expected BLOCKED report: offline `cargo +1.89.0 metadata --locked --offline --format-version 1` cannot resolve uncached `arbitrary v1.4.2`; no download or install was attempted.
- `just fmt` — NOT PASS: inherited `tethers-0.1/host-rust/src/file_tools.rs`
  is not rustfmt-clean. It was inspected, left unchanged, and is outside this packet.
- `cargo +1.89.0 clippy --all-targets --all-features --locked -- -D warnings` — NOT PASS: pre-existing dead-code and Clippy findings in application, child-process, candidate, package, and other baseline modules; no finding named the new module.

## Discoveries

The fresh worktree did not initially contain an engine binary, so full Rust
tests failed at five retained-engine tests before the packet-authorised existing
switch built this worktree's engine. The strict offline metadata probe correctly
reports a missing cached Cargo package even though the normal locked test matrix
can run from the local cache state.

## Remaining risks

The Rust issuer is a host library seam and permit launcher; a future task must
wire it into the agent/task orchestrator that collects request and observation
artifacts. Scoped fields are authority inputs, not a false claim of OS sandbox
enforcement. Repository-wide formatting and Clippy debt predate this task.

## Smallest next action

Route one real bounded implementation task through the probe script and Rust
issuer, then attach its contract digest to that task's worker note.

## References

- Base: `e57bf536fe3d7fb074c00ddac867b5720a15116e`
- Architecture/probe commit: `83e1130`
- Rust issuer commit: `ddb582d`
- Branch: `codex/execution-environment-handshake`
- `docs/architecture/TETHERS_EXECUTION_ENVIRONMENT_HANDSHAKE_V1.md`
