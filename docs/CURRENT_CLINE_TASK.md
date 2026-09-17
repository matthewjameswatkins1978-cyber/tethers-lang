# TETHERS L1 - Linux Parity Recovery

Task: `TETHERS L1 / Linux Parity Recovery`

Control contract: `1`

Status: `BLOCKED`

Task colour: `Amber`

Owner: `Codex`

Route: `Recover still-valid native Linux work from PR #25 onto current fetched main, prove native Core and Rust parity, document the remaining verification boundary, commit and push without merging.`

Base commit: `15c85c7ffffb56614b0667b7a39164529a1b7fc4`

Current implementation checkpoint: `3466f7e7956f3dd8fd50abe41f44e7c36263e1e7`

Worker note: `docs/worker-notes/2026-09-17-tethers-l1-linux-parity-recovery.md`

Suggested branch:

`codex/tethers-linux-parity-finish`

## Objective

Bring the valid native Linux runtime work from PR #25 forward onto the current
accepted main without reintroducing superseded Resolve architecture or changing
Tethers semantics. Establish native OCaml/Core, verified-engine provenance,
Rust host, process supervision, replay, installation, provider-fixture and
cross-language evidence.

## Preserved and bounded

- PR #25 remains preserved at `ebed49020ed5686b299d31d7563802b7ecb1e478`.
- Native Linux replay persistence and symlink refusal are carried forward.
- Unix process supervision and Unix installation-lock implementation are carried
  forward.
- Linux provider fixture boundaries are native and semantically equivalent to
  the Windows fixture.
- No OCaml/Core, Plan, syntax, replay authority, outcome taxonomy, Resolve
  transport, or Windows checkout changes are authorised.
- PR #25 packaging, PowerShell CI, and files conflicting with current main are
  deferred to the later packaging/support packet.

## Acceptance evidence

- Native switch: `bl-tethers-5.5.0`, OCaml `5.5.0`, Dune `3.24.0`, Yojson
  `2.2.2`.
- Current engine: `tethers-0.1/engine-ocaml/_build/default/bin/tethers_mcp_main.exe`
  (Linux ELF), source commit bound by the generated provenance manifest.
- Verified native runner: 1,519 passed, 0 failed, 2 ignored in both default
  and single-thread modes.
- Focused native replay and Unix process supervision tests pass.
- Provider/socket/check-command/cross-language focused tests pass.

## Blocking verification boundary

The repository `just verify` recipe remains PowerShell-only. In native WSL it
crosses into Windows PowerShell through interop and stops before verification:

`SecurityError: File .../scripts/verify-tethers.ps1 cannot be loaded. The file .../scripts/verify-tethers.ps1 is not digitally signed.`

The workshop `bl verify tethers` is a lightweight workshop check against the
canonical workshop checkout, not a substitute for the full task-worktree
`just verify`. L1 remains blocked until the authoritative verification route
has a native Linux entry point or an explicitly accepted external-tooling
classification.

## Stop conditions

- Do not weaken or cfg-out cross-platform product tests.
- Do not modify WSL, global Git, toolchains, authentication, or the Windows
  checkout.
- Do not begin L2 packaging/support proof while this L1 verification blocker
  remains unresolved.
- After two materially similar failed approaches, stop and report the exact
  remaining tooling boundary.

## Implementation scope

- `tethers-0.1/host-rust/src/replay_linux.rs`
- `tethers-0.1/host-rust/src/replay_store.rs`
- `tethers-0.1/host-rust/src/child_process.rs`
- `tethers-0.1/host-rust/src/installation_execution.rs`
- `tethers-0.1/host-rust/src/launch_profile.rs`
- `tethers-0.1/host-rust/src/application.rs`
- `tethers-0.1/host-rust/src/check_command.rs`
- `tethers-0.1/host-rust/src/socket.rs`
- `tethers-0.1/host-rust/src/host_execution.rs`
- `tethers-0.1/scripts/tethers-stdio-fixture.sh`
- `tethers-0.1/scripts/tethers-stdio-fixture.py`
- `verification/current-engine-provenance.json` (generated, ignored)
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-09-17-tethers-l1-linux-parity-recovery.md`
