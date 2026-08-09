# Current Implementation Task

Control contract: `1`
Task packet: `F10 — Foundation Clean-Checkout Completion Proof`
Owner: `Codex`
Status: `IN_PROGRESS`
Task colour: `Amber`
Route: `Codex performs independent Windows clean-checkout evidence and sign-off`
Worker note: `docs/worker-notes/2026-08-09-f10-clean-checkout-proof.md`
Base branch: `foundation/pre-f10-gate-consistency`
Base commit: `f1fcf6c1af380bb8a787d725ac83d7faae5bc17c`
Implementation branch: `foundation/f10-clean-checkout-proof`
Implementation checkpoint: `PENDING`
OCaml switch path: `resolve from existing machine state only`
Rust toolchain: `repository-pinned`
Rust change class: `DOCS`

## Objective

Produce an independent, disposable Windows clean-checkout proof for the
accepted pre-F10 Foundation target. This is evidence and sign-off only; it must
not repair product, test, toolchain, fixture, dependency, or programme issues.

## Relevant background and existing behaviour

The accepted target is `f1fcf6c1af380bb8a787d725ac83d7faae5bc17c` on
`origin/foundation/pre-f10-gate-consistency`; Foundation begins at
`24428139807cac0adeb0b62264547e61ca809d16`. The accepted F1 fixture evidence
tip is `f295daa288f4d3dc48181888d6655df798675033`. A fresh worktree lacks
ignored OCaml outputs by design; an existing compatible switch may build them,
but no install, switch creation, or source repair is authorised.

## Required behaviour

1. Commit a control-start task-packet/dashboard checkpoint from the exact
   accepted target and prove non-control files are byte-identical.
2. Create and prove a cold disposable Windows worktree at that checkpoint.
3. Review Foundation ancestry and complete programme diff before expensive
   verification; stop on unauthorised semantic/dependency/fixture change.
4. Prove F1 fixture/manifest byte integrity, validate fixtures, recover the
   existing compatible cross-language environment, and run engine/MCP tests.
5. Run serial environment identity, advisory Clippy, and exactly one final
   `just verify-agent`; record actual outputs and final clean proof state.
6. Close out only if every mandatory command passed; push normally and remove
   only the purpose-created clean proof worktree after evidence is captured.

## Relevant components

### AUTHORISED PATHS
- `docs/CURRENT_CLINE_TASK.md`
- `docs/PROJECT_DASHBOARD.md`
- `docs/worker-notes/2026-08-09-f10-clean-checkout-proof.md`

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/PROJECT_DASHBOARD.md`
- `docs/worker-notes/2026-08-09-f10-clean-checkout-proof.md`

## Frozen decisions and invariants

- F10 proves Foundation; it does not self-declare Foundation accepted or merge
  to main. Lucy independently accepts or rejects the pushed evidence.
- No production/test/tooling/fixture/specification/dependency/toolchain/CI or
  Clippy cleanup change. No fixture regeneration or environment installation.
- The clean worktree begins cold; only its own ignored build artefacts may be
  created. The control-start commit is the sole verified checkpoint.

## Acceptance criteria

1. Remote target, control-start lineage, clean worktree state, programme
   ancestry/diff, and F1 byte integrity are all proven.
2. Existing OCaml 5.5.0 compatible switch and repository environment probe pass.
3. Fixture validator, engine tests, MCP transcripts, Clippy, and one complete
   verify-agent matrix pass with recorded actual results.
4. The proof checkout has no tracked modifications after verification.
5. COMPLETE-state checker passes, only authorised docs are committed/pushed,
   remote equals local, and the disposable worktree is safely removed.
6. Every required command is recorded exactly once with its actual PASS, FAIL,
   or NOT RUN result in the worker evidence.

## Required verification

1. Every command and pre/post-clean-checkout capture required by F10 packet.
2. Exact complete programme and F1 fixture Git comparisons.
3. Repository-owned environment/fixture/engine/MCP scripts, Clippy, and one
   serial `just verify-agent`.
4. COMPLETE packet checker, diff/status, remote equality, and cleanup proof.

## Forbidden changes

- No repair, product/test/tooling/fixture/dependency/spec/toolchain/CI change,
  warning cleanup, merge to main, tag, release, installation, or fixture refresh.

## Stop conditions

STOP and record BLOCKED if target/ref or clean state differs, required switch is
absent, programme/fixture integrity differs, a mandatory command fails or is
unrun, tracked proof files change, complete checker fails, or push cannot be
proven normally. Do not repair and continue.

## Expected pre-existing changes

None.
