# Current Implementation Task

Control contract: `1`
Task: `J24J - Read-only installation reconciliation planner`
Owner: `OpenCode`
Status: `READY`
Task colour: `Amber`
Route: `OpenCode using DeepSeek Pro V4 for bounded semantic Rust planning logic; Lucy performs independent review and routine safe merge`
Base branch: `main`
Base commit: `7b41eae28e48872986393561b961267613fe8338`
Implementation branch: `opencode/j24j-installation-reconciliation-planner`
Worker note: `docs/worker-notes/2026-08-04-j24j-installation-reconciliation.md`
Implementation blueprint: `docs/architecture/J24J_READ_ONLY_INSTALLATION_RECONCILIATION_PLANNER.md`
Rust toolchain: exact `1.97.1`; plain Cargo; `--locked` mandatory
Agent tools: bounded `rg`, compiler diagnostics, rustfmt, focused Nextest, and ordinary Cargo through `just verify`; LSP is optional and never a gate
OCaml switch path: `N/A`
Implementation checkpoint: `TBD`

## Objective

Implement a pure, read-only planner that reconciles one exact J24G installation request against the accepted candidate, exact-trust, launch-profile, conformance, installation-approval, and installed-state authorities.

Return exactly one legitimate next action:

- create exact-candidate trust;
- run supervised conformance;
- create installation approval;
- publish disabled installation;
- complete.

Read `docs/architecture/J24J_READ_ONLY_INSTALLATION_RECONCILIATION_PLANNER.md` completely before editing. It is authoritative.

## Accepted foundation

J24G, J24H, and J24I are accepted on `main`.

Current planning line:

```text
J24G request contract
  -> J24H read-only evidence access
  -> J24I exact-candidate trust
  -> J24J read-only reconciliation planner
  -> J24K locked gated executor
  -> J24L thin public plug install CLI
```

Accepted engineering baseline:

```text
Rust             1.97.1
Cargo tests      926 passing minimum
Nextest retries  0
Cargo.lock       D8AF5D2D09D0FED307557856031BE8256A82441734BB00FB46FF92812F7818CB
```

## Required public seam

Add and export `src/installation_plan.rs` with the exact action enum, plan record, and `plan_installation` signature frozen in the blueprint.

The planner accepts already-opened authorities. It does not define host-data-root layout and does not create missing stores.

## Core behaviour

1. Revalidate the public typed request fields.
2. Load the validated candidate registry and select the exact requested candidate.
3. Find and validate exact-candidate trust.
4. Construct deterministic exact `PackageTrustEvidence`.
5. Find reusable current passed conformance only when its launch profile, trust, candidate, and current suite pins all match.
6. Select multiple current passed runs deterministically by greatest `ended_unix_ms`, then greatest `evidence_id`.
7. Validate any existing candidate installation approval against the selected current chain.
8. Validate any existing exact-candidate installed record against the approval and current chain.
9. Return the earliest missing legitimate action with only the evidence pins available at that stage.

Malformed or corrupt store evidence fails closed. Historical failed, interrupted, invalidated, or stale conformance may be ignored when planning a new supervised run.

## Frozen read-only boundary

Do not:

- create or modify a directory or file;
- acquire a lock;
- generate a timestamp;
- create trust;
- prepare or launch a provider;
- run conformance;
- create installation approval;
- copy payloads;
- publish installed state;
- inspect or alter enablement;
- add a CLI command;
- call `PackageTrustEvidence::revalidate_current`;
- change an accepted evidence schema;
- change dependencies or Cargo.lock.

## Permitted files

Only:

- `tethers-0.1/host-rust/src/installation_plan.rs`
- `tethers-0.1/host-rust/src/lib.rs`
- `tethers-0.1/host-rust/tests/j24j_installation_reconciliation.rs`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-04-j24j-installation-reconciliation.md`

Stop before changing another path.

## Startup procedure

1. Require a clean worktree:

   ```powershell
   git status --short
   ```

2. Fetch remote state:

   ```powershell
   git fetch origin
   ```

3. Verify the J24J blueprint is on `origin/main`:

   ```powershell
   git merge-base --is-ancestor 2bfb7d36b0ab7c877d6042e327328eca8acdef34 origin/main
   ```

4. Inspect the remote packet and require J24J, READY, OpenCode, and the required branch:

   ```powershell
   git show origin/main:docs/CURRENT_CLINE_TASK.md | Select-Object -First 24
   ```

5. Require the implementation branch not to exist locally or remotely:

   ```powershell
   git branch --list opencode/j24j-installation-reconciliation-planner
   git branch --remotes --list origin/opencode/j24j-installation-reconciliation-planner
   ```

6. Create it from current remote main:

   ```powershell
   git switch --create opencode/j24j-installation-reconciliation-planner origin/main
   ```

7. Update this packet's Base commit to the exact current `origin/main` before the implementation commit. Record the same base in the worker note.

8. Read completely before editing:

   - `AGENTS.md`
   - this packet
   - J24J blueprint
   - J24G, J24H, and J24I blueprints
   - J24I worker note
   - `src/installation_request.rs`
   - `src/candidate.rs`
   - `src/installation_trust.rs`
   - relevant public validation and store seams in `trust.rs`, `launch_profile.rs`, `conformance.rs`, and `installed.rs`
   - `src/lib.rs`
   - focused J24G, J24H, and J24I tests

9. Run:

   ```powershell
   pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1
   pwsh -NoProfile -File scripts/check-rust-agent-tools.ps1
   Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256
   ```

If child processes cannot resolve `pwsh.exe`, prepend `$PSHOME` to PATH for this process only. Do not modify user or machine PATH.

## Discovery discipline

Use bounded `rg` to confirm the exact accepted seams and method names.

OpenCode LSP is optional. It may be tried once only when it would genuinely save work. Empty, null, unavailable, or hanging output must be recorded and abandoned immediately. Continue with source inspection, `rg`, compiler diagnostics, and tests.

No optional tool has veto power over this task.

## Required implementation details

- Use existing `M3Error` and `Result`.
- Validate manually constructed request fields before loading evidence.
- Candidate absence uses the frozen planner error.
- Present mismatched trust is an error, not absence.
- Launch-profile authority exists only when pinned by reusable current conformance.
- Do not choose an arbitrary unpinned launch profile.
- Multiple current passed conformances use the blueprint's deterministic ordering.
- A stale existing approval or installed record fails closed; do not ignore it and create another.
- Plans populate only evidence pins proven at their stage. Future pins remain `None`.
- Do not serialize the plan or add public JSON in J24J.

## Focused test requirements

Add `tests/j24j_installation_reconciliation.rs` and cover every required blueprint path, including:

- all five plan actions;
- exact evidence pins at each stage;
- stale/failed conformance handling;
- deterministic selection between current runs;
- invalid manual request;
- missing candidate;
- mismatched trust;
- corrupt store evidence;
- stale approval;
- stale installed state;
- complete recursive no-mutation snapshots;
- no provider launch or new evidence.

Use direct Rust fixtures and accepted store APIs. Do not add production test-only constructors.

## Edit recovery

If an exact edit misses:

1. reread the latest file;
2. use a smaller stable anchor;
3. make one fresh materially different patch;
4. never repeat the identical failed replacement;
5. stop after two materially different failed attempts rather than rewriting a file wholesale.

## Required verification

Run:

```powershell
pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1

cargo fmt `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all -- --check

cargo nextest run `
  --config-file .config/nextest.toml `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --all-targets --all-features --locked `
  -E 'test(j24j_installation_reconciliation) | test(installation_plan)'

cargo test `
  --manifest-path tethers-0.1/host-rust/Cargo.toml `
  --test j24j_installation_reconciliation `
  --locked

just verify

Get-FileHash tethers-0.1/host-rust/Cargo.lock -Algorithm SHA256
git diff --check
git status --short
```

The Nextest filter may be adjusted once if Nextest reports the exact integration test name differently. Record the adjustment. Do not repeat a bad filter blindly.

Do not run full Nextest, cargo-deny, cargo-machete, `just verify-agent`, OCaml tests, packaging, release, or unrelated scripts.

## Acceptance criteria

1. J24J module and export match the frozen public seam.
2. All five plan actions are reachable through valid evidence states.
3. Every later action carries the complete valid earlier evidence chain.
4. Corrupt evidence is never treated as absence.
5. Historical non-current conformance does not block a new conformance action.
6. Approval and installed state fail closed when their pins drift.
7. Planning changes no byte or path and launches no process.
8. Focused Nextest passes with zero retries.
9. Focused ordinary Cargo integration tests pass.
10. `just verify` passes with at least 926 Cargo tests and zero failures, aside from an honestly documented pre-existing environmental flake that passes on one evidence-led rerun.
11. Cargo.lock remains byte-identical.
12. Final diff contains only permitted files.

## Stop conditions

Stop as BLOCKED only if:

- an accepted authority lacks a required read-only load or validation seam;
- a required exact pin is absent from accepted records;
- safe reconciliation would require weakening validation;
- implementation requires mutation, process launch, lock, CLI, dependency, schema, or out-of-scope changes;
- required verification still fails after one evidence-led correction.

Do not stop for failed LSP, an optional tool, or one failed exact replacement.

## Completion contract

After all acceptance criteria pass:

1. Create the worker note with:
   - Requested outcome
   - Changes made
   - Decisions and assumptions
   - Evidence-chain algorithm
   - All five action proofs
   - Read-only snapshot evidence
   - Focused Nextest and Cargo evidence
   - Full Cargo evidence
   - Tool usefulness and fallbacks
   - Cargo.lock and final-diff evidence
   - Remaining risks
   - Smallest next action
2. Change packet status to `COMPLETE`.
3. Set the real implementation checkpoint SHA.
4. Commit documentation normally.
5. Push the branch normally without force.
6. Return branch, base, implementation checkpoint, completion tip, changed files, focused counts, full Cargo count, Cargo.lock hash, and confirmation that only permitted files changed.
