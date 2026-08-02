# Worker Note

Task: `J20-ENV-P1 - Execution Environment Handshake`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `777026be2945895c86e36ce997ba8e15d4f8b0f6`

Original base (pre-replay): `e57bf536fe3d7fb074c00ddac867b5720a15116e`

Implementation checkpoint: `ddb582d46049c93724928c03e40888e425c7517e`

Replayed final SHA: `4da7c0e853392075ea4e3bdf43b7792e49827dc5`

Correction final SHA: `436dca377466818f57d6e4e66999a31b80a6633b`

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

## Replay evidence (2026-08-02)

**Method:** `git rebase --onto origin/main e57bf53 HEAD`. Clean, unpublished branch;
rewriting is permitted by explicit task authority.

**Pre-replay state:** `HEAD` = `d9f4926` on `codex/execution-environment-handshake`,
3 commits ahead of old merge-base `e57bf53`. Worktree clean. Backup tag:
`backup/execution-environment-handshake-pre-rebase`.

**Target:** `origin/main` = `777026be2945895c86e36ce997ba8e15d4f8b0f6` (M5 durable
local anchor COMPLETE, 14 new commits since old merge-base).

**Conflicts:** One conflict in `docs/CURRENT_CLINE_TASK.md` on the third commit
(d9f4926). Main carried the M5 task packet; the branch carried the J20-ENV-P1
handshake task. Resolved by taking the branch's version (theirs), since the
handshake branch owns this task and M5 is preserved in main's history.
`tethers-0.1/host-rust/src/lib.rs` merged cleanly — both `execution_environment`
and `local_anchor` module declarations coexist in alphabetical order.

**Post-replay verification:**

| Check | Result |
|---|---|
| `git diff --check origin/main...HEAD` | PASS |
| `git diff --stat` (branch vs main) | 11 files, +1242/-293 |
| `merge-base HEAD origin/main` | `777026b` (exact) |
| `pwsh -NoProfile -File scripts/check-tethers-environment.ps1 -Profile rust-host` | PASS; `rust.check` offline fail is known `arbitrary v1.4.2` |
| `cargo +1.89.0 test execution_environment --locked` | PASS (6/6) |
| `cargo +1.89.0 check --all-targets --all-features --locked` | PASS |
| `cargo +1.89.0 test --all-targets --all-features --locked` | 818 passed, 1 FAIL |
| `opam exec ... dune build` | PASS |
| `opam exec ... dune runtest` | PASS |
| `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` | PASS |
| `git diff --check` | PASS |

**Known baseline failures (outside this packet):**
- M3 `immediate_startup_descendant` now passes after dependency cache resolution.
- Repository-wide `cargo fmt` and `cargo clippy` — pre-existing debt in `file_tools.rs` and `application.rs`.

**Backup tag:** `backup/execution-environment-handshake-pre-rebase` at `d9f4926` (pre-replay tip). Retained; remove after acceptance.

## Integrity correction (2026-08-02)

Seven review corrections applied:

### 1. Immutable contract integrity

- `ContractData` (formerly `ContractBody`) made private; all fields inaccessible outside the module.
- `ExecutionEnvironmentContract` holds `ContractData` privately; `issue()` is the only constructor.
- `from_stored()` deserialises, recomputes the digest, and verifies it matches before returning a contract.
- `permit()` recomputes the JCS SHA-256 contract digest on every call and rejects mismatch before command lookup.
- Seven tamper tests: status, program_path, arguments, cwd, request_digest, observation_digest — all rejected with `contract_integrity`.

### 2. Shared Windows workbench enforcement

- `issue()` validates `platform == "windows"` and `shell == "pwsh"`.
- All `program_path` and `cwd` values required to be absolute `X:\...` or `X:/...` paths.
- Duplicate capability IDs and command IDs are rejected before any other validation.

### 3. PowerShell enforcement

- PowerShell commands (ending with `pwsh.exe` or `powershell.exe`) must use `-File` as the first argument.
- Script path must be absolute and the file must exist on disk.
- Script SHA-256 is computed from the actual file during issuance; must match the supplied `script_digest`.
- `-Command` and `-EncodedCommand` are unconditionally refused.
- Non-PowerShell commands with a `script_digest` are rejected.
- `recipe_shell` renamed to `just_recipe_shell` in the workbench profile and probe script, clarifying the `just` recipe shell is not an agent execution permit.

### 4. Supervised development command usability

- `max_processes` replaced from hard-coded `1` to `HOST_MAX_SUPERVISED_PROCESSES` (16) recorded in the contract.
- `clear_environment = true` preserved; approved environment map from the contract is bound to `ChildConfig`.
- Native Windows integration test: issues a contract for `cmd.exe`, launches through `SupervisedChild`, verifies `clear_environment` / `max_processes` / env map, and proves clean shutdown.
- Altered command is refused by `permit()`.

### 5. Substitute semantics — deferred (Option B)

- `substitutes` field removed from `CapabilityRequirement`.
- Workbench profile retains host-named substitutes as advisory documentation only.
- Architecture document updated: "Capability substitution is explicitly deferred from executable v1."
- Test proves a preferred capability whose host probe fails is Degraded without substitute resolution.

### 6. Corrected evidence

- Matthew's authorised `cargo fetch` resolved `arbitrary v1.4.2` and `derive_arbitrary v1.4.2` from `Cargo.lock`.
- `Cargo.lock` unchanged.
- `cargo +1.89.0 metadata --locked --offline --format-version 1` now exits 0.
- `cargo_offline: true` reported by `check-tethers-environment.ps1`.
- Stale uncached-dependency limitation removed from evidence.
- M3 `immediate_startup_descendant` test now passes (was OS-code-5 transient).

### 7. Verification matrix

| Check | Result |
|---|---|
| `cargo +1.89.0 test execution_environment --locked` | 25/25 PASS (includes 7 new tamper, 2 new Windows integration, 3 new enforcement) |
| `cargo +1.89.0 check --all-targets --all-features --locked --offline` | PASS |
| `cargo +1.89.0 test --all-targets --all-features --locked` | 837/837 PASS |
| `opam exec ... dune build` | PASS |
| `opam exec ... dune runtest` | PASS |
| `check-tethers-task-packet.ps1` | PASS |
| `check-tethers-environment.ps1 -Profile rust-host` | PASS (all 5 probes green; `cargo_offline: true`) |
| `git diff --check` | PASS |

## References

- Original base: `e57bf536fe3d7fb074c00ddac867b5720a15116e`
- New M5 baseline: `777026be2945895c86e36ce997ba8e15d4f8b0f6`
- Architecture/probe commit: `a4a297c`
- Rust issuer commit: `bb30fd0`
- Evidence commit: `4da7c0e`
- Replayed final SHA: `4da7c0e853392075ea4e3bdf43b7792e49827dc5`
- Correction final SHA: `436dca377466818f57d6e4e66999a31b80a6633b`
- Branch: `codex/execution-environment-handshake`
- `docs/architecture/TETHERS_EXECUTION_ENVIRONMENT_HANDSHAKE_V1.md`
