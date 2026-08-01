# Current Implementation Task

Control contract: `1`

Task: `J17B1 - refresh the remaining 0.2 Cargo.lock verification pins`
Owner: `Luna`
Status: `COMPLETE`
Task colour: `Amber`
Route: `Luna on OpenCode - narrowly authorised release-verification correction`
Branch: `luna/j17b-lock-pins`
Base commit: `ad44c519b435c52dd82347b1000f3aeda686d310`
Worker note: `docs/worker-notes/2026-08-01-j17b-lock-pins.md`

## Objective

Refresh only the stale J14A and J14B expected Cargo.lock SHA-256 pins to the
reviewed 0.2 digest, then run the consolidated matrix once.

## Relevant background and existing behaviour

J17-V2 exposed stale expected Cargo.lock digests in J14A and J14B. J14C already
expects the reviewed 0.2 digest. The lockfile itself is unchanged.

## Required behaviour

1. Change exactly one digest string in each authorised J14A/J14B script.
2. Confirm all three active J14 scripts expect the current lockfile digest.
3. Run the consolidated matrix exactly once and require the accepted totals.
4. Preserve the retained J17-V1 and J17-V2 evidence and defer J17-V3.

## Relevant components

- `tethers-0.1/scripts/test-j14a-complete-scenario.ps1`
- `tethers-0.1/scripts/test-j14b-negative-matrix.ps1`
- `tethers-0.1/scripts/test-j14c-real-file-move.ps1` (read-only comparison)
- `tethers-0.1/host-rust/Cargo.lock` (read-only hash confirmation)

## Frozen decisions and invariants

- Old digest: `d323870ea02f09391a5d0d9aa0e9a701cf686a5ac005b840ee7218e70edb5602`.
- New digest: `894f2ce6692837fa4c449c0fc593a37ed5597577ea5b4093da0912e6ee2b14e3`.
- No guard is weakened, relocated, or centralised.
- No runtime implementation, Cargo.toml, Cargo.lock, or product version changes.
- J17-V3 and J17 sign-off remain deferred.

## Acceptance criteria

1. The lockfile hash is exactly the new digest and Cargo.lock is unchanged.
2. Each J14A/J14B script has exactly one changed digest string and active pin
   audit shows the new digest once in J14A, J14B, and J14C.
3. The single consolidated run passes 6/6 suites, 79 cases/rows, and RESULT PASS.
4. Exactly the four authorised paths change and final checks pass.

## Required verification

- Run the lockfile hash confirmation and active pin audits.
- Run `verify-0.2.ps1` exactly once.
- Run the packet checker, `git diff --check`, changed-path, and status checks.

## Forbidden changes

Do not modify Cargo.toml, Cargo.lock, runtime implementation, tests, fixtures,
release notes, retained evidence, main, or tags. Do not rerun J17-V2 or begin
J17-V3.

## Stop conditions

Stop on any hash mismatch, failed matrix, unexpected path, evidence mutation, or
second materially similar failure. Do not rerun the matrix after failure.

## Expected pre-existing changes

None in the new branch before this task.
