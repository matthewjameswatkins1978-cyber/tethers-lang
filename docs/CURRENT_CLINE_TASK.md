# Tethers 0.6 preparation: cold-agent proof and verification gate

Control contract: `1`

Status: `IN_PROGRESS`

Task colour: `Amber`

Owner: `Codex`

Route: `Fresh branch from current origin/main; prove the cold-agent plan-to-execution workflow, repair the strict verification gate, align the public product release line to 0.6, then publish only after final evidence.`

Base commit: `a8762952a4f92697ab5349e93698699a5955c477`

Implementation checkpoint: `NOT SET`

Worker note: `docs/worker-notes/2026-09-10-tethers-0.6-preparation.md`

Suggested branch:

`codex/tethers-0.6-preparation-20260910`

Source branch:

`origin/main`

Updated: 2026-09-10

## Objective

Turn the reconciled Tethers main line into an evidence-backed 0.6 preparation
line: prove that an unfamiliar agent can discover, plan, obtain authority,
execute a bounded operation, and inspect durable evidence; make the strict Rust
verification gate honest and green; and update the public product release
presentation and packaging workflow to 0.6 without changing the frozen Tethers
0.1 language, native host 0.2.2 compatibility identity, Portable Workbench
0.2.2 identity, or existing authority/replay/Trail semantics.

## Relevant background and existing behaviour

The reconciled main line already contains the side-effect-free machine-facing
`tethers plan` command and a real OCaml-engine fixture proving that planning
does not invoke providers or write execution Trail entries. Existing 0.5 cold-
agent evidence covers discovery and harmless inspection, while the J14 local
scenarios and agent-oriented Plugs provide the bounded execution seams needed
for a real workflow proof. The public product line still says 0.5, the release
packager defaults to `0.5.0`, and the tag workflow is named and filtered for
0.5. The native host and Portable Workbench intentionally remain versioned
0.2.2 for compatibility.

## Required behaviour

1. A fresh-workspace cold-agent pilot must exercise discovery, side-effect-free
   `plan`, explicit authority/execution, and post-execution Trail or receipt
   inspection with structured evidence. Planning must still prove zero provider
   calls and zero execution Trail entries.
2. The repository's strict Rust check must pass without suppressing warnings or
   weakening tests. Existing warnings must be removed or repaired at their
   actual ownership boundary.
3. The complete Rust test command must be run in its normal parallel form. Any
   barrier or scope interference must be repaired if it is a demonstrated
   repository defect; serial evidence may supplement but may not replace the
   normal command.
4. Public product release references, release documentation, packaging default,
   and tag-triggered workflow must consistently describe Tethers 0.6 and
   `tethers-v0.6.*` assets. Historical 0.5/tagged evidence remains immutable.
5. A reproducible 0.6 release package or package-build proof must include exact
   SHA-256 evidence and preserve the separate compatibility version axes.

## Frozen decisions and invariants

- A Plan remains a request, not permission.
- Core remains in OCaml; Rust remains the host and execution boundary.
- No provider, policy, replay, or Trail semantic redesign is permitted.
- Existing host and Portable Workbench package versions remain `0.2.2` unless
  a separate compatibility decision explicitly changes them.
- `tethers-v0.5.8` and all historical 0.5 evidence remain reachable and
  unchanged.
- No force push, destructive reset, blanket cleanup, or dependency addition.
- Release publication must use normal Git operations and exact remote proof.

## Relevant components

- `tethers-0.1/host-rust/`
- `tethers-0.1/portable-rust/`
- `scripts/package-tethers-release.ps1`
- `.github/workflows/tethers-v0.5-release.yml`
- `README.md`, `QUICKSTART.md`, `docs/AGENT_QUICKSTART.md`,
  `docs/CURRENT_GOAL.md`, `docs/PROJECT_DASHBOARD.md`, and release docs
- `tethers-0.1/scripts/test-j14a-complete-scenario.ps1`
- `tethers-0.1/scripts/test-j14c-real-file-move.ps1`
- `.github/scripts/check-tethers-task-packet.ps1`

## Acceptance criteria

1. The cold-agent pilot has a committed evidence note with exact commands,
   structured plan result, explicit execution result, receipt/Trail evidence,
   and negative proof that planning alone caused no provider or Trail effect.
2. `just check` and the normal complete locked Rust test command pass; focused
   plan/Core tests remain green.
3. The explicit OCaml build and test commands remain green after the changes.
4. Product-facing release metadata and packaging workflow consistently identify
   0.6, while compatibility axes and historical 0.5 records remain explicit.
5. A release package/build proof and SHA-256 manifest are recorded. No release
   claim is made for an asset or platform that was not actually built.
6. The implementation checkpoint, worker note, task checker, branch, remote
   branch, final remote SHA, and clean worktree all agree.

## Required verification

- toolchain and task-packet gates;
- Rust formatter, strict check, Clippy, normal parallel complete tests,
  focused plan/Core tests, and release builds;
- explicit OCaml switch build/tests and the selected J14 scenario checks;
- cold-agent plan/no-effect and execution/Trail evidence;
- version-reference search, package-build output, SHA-256 manifest, diff check,
  remote ancestry, tag, and final clean-worktree proof.

## Stop conditions

Stop and report if the pilot requires a new authority or capability semantic,
the normal parallel suite exposes an unresolved concurrency defect after two
bounded repair attempts, the strict warning gate requires broad speculative
refactoring, the release version axes cannot be separated honestly, or required
tooling/platform coverage is unavailable.

## Forbidden changes

- no Tethers 0.1 syntax or semantic change;
- no new provider, permission, replay, Trail, or Core boundary;
- no host/portable compatibility-version rewrite disguised as product release
  labelling;
- no force push, reset, rebase of published history, or destructive cleanup;
- no invented CI, release, platform, or live-provider evidence.

## Expected pre-existing changes

None. The branch starts at current `origin/main`; all changes belong to this
0.6 preparation task.
