# Current Implementation Task

Control contract: `1`
Task: `J20-ENV-P1 - Execution Environment Handshake`
Owner: `Codex`
Status: `COMPLETE`
Task colour: `Red`
Route: `Codex host-bound implementation under Matthew's explicit authority; replayed onto M5 baseline 2026-08-02`
Base branch: `main`
Base commit: `777026be2945895c86e36ce997ba8e15d4f8b0f6`
Original base (pre-replay): `e57bf536fe3d7fb074c00ddac867b5720a15116e`
Replayed final SHA: `4da7c0e853392075ea4e3bdf43b7792e49827dc5`
Worker note: `docs/worker-notes/2026-08-02-j20-execution-environment-handshake.md`

## Objective

Add one host-owned, one-shot execution-environment handshake for Tethers
development tasks without changing the OCaml language Core or starting M5/M6.

## Relevant background and existing behaviour

The Rust host owns process supervision through `child_process::SupervisedChild`.
The existing PowerShell diagnostics prove workstation tools and toolchains, but
do not issue task-specific immutable contracts. `CURRENT_CLINE_TASK.md` is the
task-control authority.

## Required behaviour

1. Define one shared Windows workbench profile and only small current-worker role overlays; require an explicit Matthew-selected worker assignment.
2. Define request, host-observation, and immutable contract evidence with task/session/repository binding, three JCS SHA-256 digests, exact command arrays/cwds, version policy, scoped permissions, and required/preferred/replaceable/optional outcomes.
3. Implement the Rust host issuer and permit boundary so a command must match the frozen absolute program, arguments and working directory, and launches through the existing supervised process-tree owner.
4. Add host probes, schemas, examples, documentation, and focused tests; record unavailability truthfully without installation or fallback invention.

## Relevant components

- `tethers-0.1/host-rust/src/child_process.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `scripts/check-dev-tools.ps1`
- `.github/scripts/check-tethers-toolchains.ps1`
- `docs/architecture/`

## Frozen decisions and invariants

- Matthew alone selects or replaces workers; agents recommend but never self-appoint or inherit ownership.
- One request, one host observation, one frozen contract; changed facts require a new handshake.
- No automatic installation, global toolchain/configuration change, shell switching, invented substitute, force push, or language-semantic change.
- A PowerShell permit is a reviewed `-File` command, never arbitrary `-Command` or `-EncodedCommand`.
- Job Object supervision owns process-tree launch/cleanup but is not a filesystem, network, or hostile-code sandbox.

## Acceptance criteria

1. One profile plus four small overlays document the current team without duplicated tool profiles, and the Rust issuer refuses a non-Matthew assignment.
2. The schema/example/documentation name all request, observation, contract, digest, command, version, permission, degraded, and blocked semantics.
3. Focused Rust tests prove immutable digests, required/preferred/optional outcomes, exact command/cwd refusal, PowerShell refusal, and supervised-child configuration.
4. Host probe output is JSON, uses bounded real commands, and reports the current offline toolchain limitation without changing the machine.

## Required verification

```powershell
pwsh -NoProfile -File scripts/check-dev-tools.ps1
pwsh -NoProfile -File scripts/check-tethers-environment.ps1 -Profile rust-host
rustfmt +1.89.0 --check tethers-0.1/host-rust/src/execution_environment.rs
Push-Location tethers-0.1/host-rust; cargo +1.89.0 test execution_environment --locked
Push-Location tethers-0.1/host-rust; cargo +1.89.0 check --all-targets --all-features --locked
Push-Location tethers-0.1/host-rust; cargo +1.89.0 test --all-targets --all-features --locked
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
git diff --check
```

The explicit existing OCaml switch is
`D:\The Next Thing\Tethers Lang - J16 Clean\tethers-0.1\engine-ocaml`; use it
only with `opam exec --switch=<absolute path>` to build the current worktree's
engine if the full Rust suite needs that binary. Do not create, copy, move, or
select a switch globally.

## Forbidden changes

No Tether syntax/semantic change, OCaml Core change, M5/M6 work, release/tag
movement, force push, automatic install, dependency update, global config
change, shell/WSL/Docker fallback, or provider/Plug redesign.

## Stop conditions

Stop after two materially different evidence-based attempts if the requested
host boundary requires a second execution supervisor, a new security model,
an unsupported semantic change, unavailable authorised toolchain, or a change
outside the exact file scope. Do not turn any of these into an invented fallback.

## Expected pre-existing changes

None.
