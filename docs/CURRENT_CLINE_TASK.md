# Current Implementation Task

Control contract: `1`
Task: `F8-WORKFLOW — Worker Formatting and Publication Defaults`
Owner: `Codex`
Model: `GPT-5.6`
Status: `COMPLETE`
Task colour: `Green`
Route: `Codex updates documentation and task templates only`
Worker note: `docs/worker-notes/2026-08-09-f8-worker-formatting-publication-defaults.md`
Base branch: `foundation/f8a-r1-evidence-repair`
Base commit: `bfb47ced813d8ec227f8828bbf66c7ecd1110d2f`
Implementation branch: `foundation/f8-worker-lifecycle`
Implementation checkpoint: `30b26d1959138176dbf1481b267adc1791f0bc09`
OCaml switch path: `N/A`
Rust toolchain: `1.97.1`
Rust change class: `NON_RUST`

## Relevant background and existing behaviour

F8a-R1 is the accepted documentation-only base. The separate `F8-FMT` branch
owns the existing `replay_windows.rs` formatting repair and must not be changed
or incorporated by this task.

The project needs future packets and worker guidance to make three existing
workflow decisions automatic: Rust-changing work formats only its authorised
Rust files before its implementation checkpoint and stops on unrelated rustfmt
output; non-Rust/evidence-only work runs formatter check mode only and never
changes Rust source; every completed branch is pushed normally and its remote
head equality is reported.

## Objective

Update the task template and authoritative worker guidance with those formatting
and publication defaults. No production, test, fixture, build, protocol, script,
or dependency changes.

## Required behaviour

1. Template Rust-changing jobs to run Cargo formatting before the implementation
   checkpoint, inspect its immediate diff, and stop if rustfmt changes an
   unauthorised file.
2. Template non-Rust/evidence-only jobs to use `cargo fmt --all -- --check` only
   and prohibit Rust source changes.
3. Require every complete branch to be normally pushed to `origin` and reported
   with its full remote HEAD SHA, local-equals-remote confirmation, and clean
   Git status.
4. Preserve implementation-checkpoint then documentation-only closeout
   sequencing.

## Frozen decisions and invariants

- No production, Rust, OCaml, test, fixture, build, protocol, script, or
  dependency changes.
- Do not modify the F8-FMT branch or apply its formatting repair here.
- Do not clean warnings, run a mutating Cargo formatter, or activate F8 gates.
- Preserve existing Foundation checkpoint and closeout sequencing.
- A normal push is required for this branch; no force-push, merge, rebase,
  direct `main` update, or pull request is authorised.

## Acceptance criteria

1. Rust formatter pre-checkpoint and unrelated-file stop rule is present in the
   template and authoritative guidance — proven by diff.
2. Non-Rust formatter check-only rule and Rust-source prohibition is present in
   the template and authoritative guidance — proven by diff.
3. Completed-branch normal push and remote-SHA reporting rule is present in the
   template and authoritative guidance — proven by diff.
4. Foundation checkpoint/closeout sequencing remains intact — proven by diff.
5. Only documentation/template paths changed from base — proven by Git range.

## Required verification

- `cargo fmt --all -- --check`: record result; do not modify source
- `git diff --check`: PASS
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`: PASS
- `git diff --name-only bfb47ced813d8ec227f8828bbf66c7ecd1110d2f..HEAD`:
  authorised documentation/template paths only
- remote branch HEAD equals local HEAD after normal push
- `git status --short --branch`: clean

## Relevant components

### GUIDANCE AND TEMPLATES
- `AGENTS.md`
- `docs/PROJECT_CONTROL.md`
- `docs/AGENT_WORKFLOW.md`
- `docs/TASK_PACKET_TEMPLATE.md`
- `docs/WORKER_NOTE_TEMPLATE.md`
- `docs/CLINE_HANDOFF.md`
- `docs/working-guides/DEEPSEEK_PRO_OPENCODE_JOB_PLAYBOOK.md`

### CLOSEOUT
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-08-09-f8-worker-formatting-publication-defaults.md`

## Forbidden changes

- No changes outside the named documentation/template and closeout paths
- No Rust formatting repair or warning cleanup
- No F8-FMT implementation or branch changes

## Stop conditions

STOP if any change would require production, Rust, OCaml, test, fixture, build,
protocol, script, dependency, merge, rebase, or force-push work. STOP after two
materially similar failed attempts. Return exact evidence and one smallest
unresolved question.

## Expected pre-existing changes

None — this documentation-only task starts from the exact base commit
`bfb47ced813d8ec227f8828bbf66c7ecd1110d2f` with a clean tree.
