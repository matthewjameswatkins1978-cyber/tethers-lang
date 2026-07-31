# Current Implementation Task

Control contract: `1`

Task: `J14B - negative public integration matrix`

Owner: `OpenCode`

Status: `COMPLETE`

Task colour: `Amber`

Correction: `J14B-R - strengthen negative matrix assertions and evidence`

Route: `OpenCode implementation - Lucy independent review`

Base commit: `8a06b0883f968f1561153bf8d54bfce3818fbde8`

Branch: `opencode/j14b-negative-public-matrix`

Worker note: `docs/worker-notes/2026-07-30-j14b-negative-public-matrix.md`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

## Objective

Complete J14 by proving the required negative matrix through reproducible native
Windows boundaries.

Create one J14B verification script that reports eleven named matrix rows:

1. malformed manifest;
2. unavailable provider;
3. Ask;
4. Deny;
5. stale pin;
6. post-admission durable intent failure;
7. executor failure;
8. invalid provider output;
9. uncertain timeout;
10. duplicate replay;
11. causal depth beyond eight.

Use the real public `check` or `run` commands wherever the failure can be induced
without weakening production. Use an existing accepted internal seam only where
public fault injection would itself be a product defect.

J14 remains incomplete until Lucy independently accepts this branch.

## Relevant background and existing behaviour

J14A is accepted and published at
`8a06b0883f968f1561153bf8d54bfce3818fbde8`. It proves the positive public
`check -> run -> trail -> replay` route and exact trusted execution identity.

The roadmap requires J14 negative cases for malformed manifest, unavailable
provider, Ask, Deny, stale pin, intent failure, executor failure, invalid output,
uncertain timeout, duplicate replay, and loop depth.

The public `check` and `run` commands can naturally prove nine of those rows.
Post-admission Trail-intent failure cannot be safely induced through the public
CLI without a production fault switch. Depth-nine rejection already has an
accepted compiled debug boundary in `event-admission-trail-probe`. J14B must use
those narrow seams rather than add a hidden production bypass.

The public `run` coordinator still owns exactly one selected external input. Do
not expand public follow-up evaluation, change queue semantics, or redesign
J10/J11 in this task.

## Required behaviour

1. Complete the mandatory `AGENTS.md` startup report and all pre-flight checks
   before mutation.
2. Add `tethers-0.1/scripts/test-j14b-negative-matrix.ps1` as the single J14B
   scenario entry point. It must print each matrix row separately and finish
   with honest case and assertion totals.
3. Create all generated configurations, inputs, Trails, replay state, markers,
   and temporary assets beneath one unique system temporary directory containing
   both a space and a non-ASCII character. Remove it in `finally` on success or
   failure.
4. Exercise rows 1-5 and 7-10 through the real public `check` or `run` commands,
   the real OCaml engine, the real prepared runtime, and the real stdio provider
   boundary. Do not use `__legacy` for those rows.
5. Prove row 6 with one focused `#[cfg(test)]` Rust test in `main.rs` using the
   accepted execution boundary and existing test doubles. Do not add a runtime
   environment variable, CLI flag, global state, or production fault-injection
   branch.
6. Prove row 11 through the existing debug-only
   `event-admission-trail-probe causal-depth` compiled boundary. Also prove the
   release binary does not expose that debug command.
7. Extend `tethers-stdio-fixture.ps1` only with the three deterministic run modes
   required by J14B: `run-explicit-error`, `run-invalid-output`, and
   `run-hang-call`. All three must advertise the same accepted input and output
   schemas as `run-success` and record initialize, tools/list, and tools/call in
   the marker file.
8. Preserve exact public envelope status, process exit code, machine code,
   execution-ID presence rule, Result Anchor presence rule, provider call count,
   Trail evidence, and no-retry rule for every public row.
9. For malformed manifest and stale pinned digest, prove failure occurs before
   provider launch and before Trail or replay mutation.
10. For Ask and Deny, prove zero effectful calls, no trusted execution ID, and no
    Result Anchor. Ask must durably record `approval_requested` and expose no
    public approval ID.
11. For executor failure, invalid output, and uncertain timeout, prove exactly
    one effectful call, a trusted execution ID, the correct standard Result
    Anchor kind, and no automatic retry.
12. For duplicate replay, prove the second public run returns the original
    execution ID, performs no second effectful call, and leaves the filtered
    execution Trail structurally unchanged.
13. Record the boundary decision in `docs/DECISIONS.md` and write the required
    worker note with exact results and honest exceptions.
14. Run the complete required verification, create one implementation commit and
    at most one documentation closeout commit, push only the feature branch, and
    stop without beginning J15.

## Relevant components

- `tethers-0.1/scripts/test-j14b-negative-matrix.ps1` - new matrix harness.
- `tethers-0.1/scripts/tethers-stdio-fixture.ps1` - three deterministic provider
  failure modes.
- `tethers-0.1/host-rust/src/main.rs` - test-only post-admission intent-failure
  proof; production code must remain unchanged.
- `tethers-0.1/scripts/test-j14a-complete-scenario.ps1` - accepted positive
  scenario patterns and public envelope helpers.
- `tethers-0.1/scripts/test-j13a-check.ps1` - accepted public check patterns.
- `tethers-0.1/scripts/test-j13b-run.ps1` - accepted public run, Ask, Deny, and
  replay patterns.
- `tethers-0.1/scripts/test-host-event-admission-trail.ps1` - accepted durable
  depth rejection contract.
- `docs/DECISIONS.md` - boundary decision.
- `docs/worker-notes/2026-07-30-j14b-negative-public-matrix.md` - evidence.

## Expected pre-existing changes

None.

The branch contains one planning commit after the base and the live worktree must
be completely clean before implementation.

## Frozen decisions and invariants

- J14A at `8a06b0883f968f1561153bf8d54bfce3818fbde8` is the immutable base.
- Public status and exit-code vocabulary remains frozen.
- Trusted execution identity comes only from replay admission.
- Callers and planners cannot supply execution identity.
- Result Anchor schema remains unchanged and never contains `execution_id`.
- Exactly one provider invocation is the maximum for an attempted action.
- No automatic retry or compensation.
- Ask, Deny, unavailable, malformed configuration, and pre-admission failure
  produce no Result Anchor.
- Known executor failure and invalid output produce `capability.failed`.
- Timeout after possible dispatch produces `capability.uncertain`.
- Exact replay does not repeat an external effect.
- Event generation nine is rejected with maximum generation eight.
- No public fault-injection option, environment variable, magic path, or hidden
  mutable global may be added.
- No public follow-up coordinator expansion in J14B.
- J15 remains a later consolidation task and must not begin here.

## Acceptance criteria

1. Startup report and pre-flight prove the exact worktree, branch, base, remote
   refs, clean status, authorised paths, stop conditions, and two-failure rule.
2. The J14B script reports exactly 11 named matrix rows, all PASS, with exact
   assertion totals and no swallowed child-process failure.
3. Every temporary path is beneath one Unicode-plus-space temporary root, all
   temporary state is removed in `finally`, repository status is unchanged by
   the harness, and Cargo.lock remains byte-identical.
4. Rows 1-5 and 7-10 invoke only public `check` or `run` commands with the real
   engine and provider boundary.
5. The focused `j14b_` intent-failure test uses the accepted execution seam,
   establishes replay identity before the failing Trail intent, retains that
   exact identity, performs zero provider calls, emits no Result Anchor, and
   contains no production fault switch.
6. Depth row records exactly one external generation-zero admission and one
   generation-nine rejection with `causal_depth_exceeded` and
   `maximum_generation = 8`; no later sibling is recorded or evaluated. Release
   CLI rejects the debug command.
7. Fixture changes are limited to the three named modes, preserve all existing
   modes, preserve advertised schema for run modes, and add no unrelated output.
8. Every public row proves one JSON envelope, matching embedded/process exit
   codes, exact machine code, exact identity presence or absence, exact Result
   Anchor presence or absence, exact provider method counts, and no retry.
9. Malformed manifest and stale pinned digest both return `invalid_data`, exit 3,
   `RUNTIME_PREPARE_FAILED`, launch no provider, and create no Trail or replay
   state.
10. Unavailable provider returns `unavailable`, exit 4,
    `PROVIDER_CAPABILITY_UNAVAILABLE`, performs no tools/call, and preserves
    provider evidence.
11. Ask returns `approval_required`, exit 5, records `approval_requested`, exposes
    no public approval ID or execution ID, performs zero calls, and emits no
    Result Anchor. Deny returns `denied`, exit 0, with the same zero-effect and
    identity/anchor absence rules.
12. Executor error and invalid output return `failed`, exit 6, `ACTION_FAILED`,
    expose a parseable trusted execution ID, perform exactly one tools/call,
    emit `capability.failed`, and do not retry. Timeout returns `uncertain`, exit
    7, `ACTION_UNCERTAIN`, exactly one call, `capability.uncertain`, and bounded
    completion after the manifest deadline.
13. Duplicate replay returns `replay_blocked_completed_success`, the exact first
    execution ID, one total tools/call across both runs, and structurally
    identical filtered Trail entries.
14. Decision, worker note, packet, focused/full Rust checks, matrix script,
    regressions, toolchain checks, Cargo.lock hash, packet checker, whitespace,
    branch range, and clean worktrees all pass and are reported honestly.

## Required verification

### Mandatory reading

Read in full before editing:

- `AGENTS.md`
- `docs/PROJECT_CONTROL.md`
- `docs/AGENT_WORKFLOW.md`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/IMPLEMENTATION_LANGUAGE_STANDARD.md`
- `docs/GIT_WORKTREES_AND_LINE_ENDINGS_FOR_AGENTS.md`
- `docs/RUST_ENGINEERING_GUIDE_FOR_AGENTS.md`
- `docs/ROAD_TO_0_2.md` J14 and J15 sections
- `docs/DECISIONS.md` J14A decision
- `tethers-0.1/host-rust/src/main.rs` relevant execution tests and debug probes
- `tethers-0.1/scripts/tethers-stdio-fixture.ps1`
- `tethers-0.1/scripts/test-j13a-check.ps1`
- `tethers-0.1/scripts/test-j13b-run.ps1`
- `tethers-0.1/scripts/test-j14a-complete-scenario.ps1`
- `tethers-0.1/scripts/test-host-event-admission-trail.ps1`

### Pre-flight

Run:

```powershell
git rev-parse --show-toplevel
git branch --show-current
git status --porcelain=v1 --untracked-files=all
git fetch origin --prune
git rev-parse HEAD
git rev-parse origin/opencode/j14b-negative-public-matrix
git rev-parse origin/main
git merge-base HEAD origin/main
git rev-list --count origin/main..HEAD
git rev-list --count HEAD..origin/main
git worktree list --porcelain
```

Require:

- worktree `D:\The Next Thing\Tethers Lang - Goose Integration`;
- branch `opencode/j14b-negative-public-matrix`;
- completely clean status;
- local and remote branch identical;
- origin/main exactly `8a06b0883f968f1561153bf8d54bfce3818fbde8`;
- merge base exactly origin/main;
- branch exactly one planning commit ahead and zero behind;
- original worktree preserved on `cline/j10-result-event-queue` with only
  `M docs/TETHERS_LUCY_NOTES.md`.

Run the non-mutating toolchain preflight:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass `
  -File .\.github\scripts\check-tethers-toolchains.ps1 `
  -OcamlSwitchPath `
    "D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml"
```

### Matrix row contract

Implement these exact rows:

| ID | Boundary | Required result |
| --- | --- | --- |
| M01 | public `check` | malformed manifest -> `invalid_data` / 3 / `RUNTIME_PREPARE_FAILED`; no provider, Trail, or replay |
| M02 | public `check` | missing advertised tool -> `unavailable` / 4 / `PROVIDER_CAPABILITY_UNAVAILABLE`; zero tools/call |
| M03 | public `run` | Ask -> `approval_required` / 5; approval Trail; no public approval ID, execution ID, call, or Result Anchor |
| M04 | public `run` | Deny -> `denied` / 0; no execution ID, call, or Result Anchor |
| M05 | public `check` | one-nibble stale pinned digest -> `invalid_data` / 3 / `RUNTIME_PREPARE_FAILED`; no provider, Trail, or replay |
| M06 | focused Rust seam | fresh replay admission followed by durable Trail intent failure retains exact ID; denied; zero call; no Result Anchor |
| M07 | public `run` | explicit provider error -> `failed` / 6 / `ACTION_FAILED`; one call; ID; `capability.failed` |
| M08 | public `run` | schema-invalid returned value -> `failed` / 6 / `ACTION_FAILED`; one call; ID; `capability.failed` |
| M09 | public `run` | tools/call hangs past timeout -> `uncertain` / 7 / `ACTION_UNCERTAIN`; one call; ID; `capability.uncertain`; no retry |
| M10 | public `run` twice | exact replay -> same ID; one total effectful call; structurally unchanged filtered Trail |
| M11 | debug compiled boundary | generation 9 rejected; maximum 8; no later sibling; release command hidden |

### Rust and build checks

Use process-local `RUSTUP_AUTO_INSTALL=0`, restoring its previous value in
`finally`. Run:

```powershell
rustup run 1.89.0 cargo fmt --manifest-path .\tethers-0.1\host-rust\Cargo.toml --check
rustup run 1.89.0 cargo check --manifest-path .\tethers-0.1\host-rust\Cargo.toml --locked
rustup run 1.89.0 cargo check --manifest-path .\tethers-0.1\host-rust\Cargo.toml --locked --tests
rustup run 1.89.0 cargo test --manifest-path .\tethers-0.1\host-rust\Cargo.toml --locked j14b_ -- --nocapture
rustup run 1.89.0 cargo test --manifest-path .\tethers-0.1\host-rust\Cargo.toml --locked
rustup run 1.89.0 cargo clippy --manifest-path .\tethers-0.1\host-rust\Cargo.toml --locked --all-targets --all-features
rustup run 1.89.0 cargo build --manifest-path .\tethers-0.1\host-rust\Cargo.toml --locked
rustup run 1.89.0 cargo build --manifest-path .\tethers-0.1\host-rust\Cargo.toml --locked --release
```

Ordinary baseline warnings are acceptable. Add no allow/suppression and report
exact warning counts.

### Scenario and regressions

Run:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-j14b-negative-matrix.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-j14a-complete-scenario.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-j13a-check.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-j13b-run.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-j13c-trail.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-host-denial.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-host-execution-failure.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-host-result-follow-up.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-host-event-admission.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-host-event-admission-trail.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\check-fixtures.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-mcp-transcripts.ps1
```

Run `test-engine.ps1` and `demo.ps1` through the accepted safe process-local
`OPAMSWITCH` wrapper using the exact external switch path above. Restore the
previous environment value in `finally`.

Confirm Cargo.lock SHA-256 remains:

`d323870ea02f09391a5d0d9aa0e9a701cf686a5ac005b840ee7218e70edb5602`

Run the task packet checker after setting the packet COMPLETE:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass -File .\.github\scripts\check-tethers-task-packet.ps1
```

### Commit and push

Before the implementation commit, require only authorised paths changed.

Authorised paths:

1. `docs/CURRENT_CLINE_TASK.md`
2. `docs/DECISIONS.md`
3. `docs/worker-notes/2026-07-30-j14b-negative-public-matrix.md`
4. `tethers-0.1/host-rust/src/main.rs`
5. `tethers-0.1/scripts/tethers-stdio-fixture.ps1`
6. `tethers-0.1/scripts/test-j14b-negative-matrix.ps1`

In `main.rs`, only `#[cfg(test)]` test code may change. Prove the production
semantic diff is empty. Do not touch public command routing or runtime logic.

Create the implementation commit first from code/test paths only:

```powershell
git add -- `
  tethers-0.1/host-rust/src/main.rs `
  tethers-0.1/scripts/tethers-stdio-fixture.ps1 `
  tethers-0.1/scripts/test-j14b-negative-matrix.ps1

git commit -m "test: prove j14b negative matrix"
```

Record that exact implementation SHA in the worker note as
`Implementation checkpoint`.

Then update `docs/DECISIONS.md`, the worker note, and this packet to COMPLETE.
Create at most one documentation closeout commit:

```powershell
git add -- `
  docs/CURRENT_CLINE_TASK.md `
  docs/DECISIONS.md `
  docs/worker-notes/2026-07-30-j14b-negative-public-matrix.md

git commit -m "docs: complete j14b negative matrix"
```

Do not amend, squash, rebase, reset, merge, or force-push.

Run:

```powershell
git diff --check 8a06b0883f968f1561153bf8d54bfce3818fbde8..HEAD
git diff --name-status 8a06b0883f968f1561153bf8d54bfce3818fbde8..HEAD
git log --oneline 8a06b0883f968f1561153bf8d54bfce3818fbde8..HEAD
git status --short --branch --untracked-files=all
```

Push only:

```powershell
git push -u origin opencode/j14b-negative-public-matrix
```

Return the final local and remote SHA, implementation SHA, exact changed paths,
all eleven row results, public envelope and provider-call evidence, focused and
full Rust totals, regression results, Cargo.lock hashes, packet-checker and
whitespace results, clean worktree status, unchanged origin/main, and preserved
original worktree.

## Forbidden changes

- No production Rust behaviour change.
- No OCaml change.
- No CLI flag, environment fault switch, magic filename, hidden mutable global,
  or runtime bypass.
- No Result Anchor, Trail, replay, manifest, runtime-config, or public-envelope
  schema change.
- No modification to existing J13 or J14A scenario scripts.
- No new package or dependency.
- No Cargo.toml or Cargo.lock change.
- No roadmap, dashboard, task queue, constitution, SPEC, or engineering-guide
  update.
- No public follow-up coordinator integration.
- No automatic retry.
- No main push, merge, branch deletion, worktree deletion, or J15 work.

## Stop conditions

Return `BLOCKED` with exact evidence when:

- startup report is incomplete;
- repository root, branch, base, remote ref, or worktree differs;
- origin/main is not exactly `8a06b0883f968f1561153bf8d54bfce3818fbde8`;
- either worktree has unexpected state;
- toolchain preflight fails;
- a required row cannot be proved without production fault injection or a
  production semantic change;
- any public envelope differs from the frozen vocabulary;
- a case performs an unexpected provider call or retry;
- a trusted execution-ID rule or Result Anchor rule differs;
- a non-authorised path must change;
- Cargo.lock changes;
- two materially similar attempts fail.

After two materially similar failed attempts, stop. Do not broaden research,
reread unchanged files repeatedly, or continue exploring outside authorised
paths. Return the smallest unresolved question and exact evidence.
