# Tethers repository reconciliation and deterministic plan integration

Task: `Tethers repository reconciliation and deterministic plan integration`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `5cce71f8f93be26a0dfd1a0e50935f9419a5c284`

Implementation checkpoint: `d144d7f1acacd556daa4471e6cef6c93e0b64ed0`

## Requested outcome

Reconcile the Tethers 0.5 implementation ancestry into the live main line and
ship a deterministic machine-readable `tethers plan` surface whose planning
path cannot request authority, enter policy/execution, launch a provider, or
create a Trail execution entry.

## Changes made

- Merged the peeled `tethers-v0.5.8` commit into the recorded current main
  base, preserving both ancestry lines.
- Added CLI parsing, host planning service mapping, and the stable
  `tethers.cli/1` plus `tethers.plan/1` response contract.
- Added unit coverage for valid plans, no-actions, planner errors, invalid
  input, zero effects, and strict command options.
- Updated the stale program-digest assertions to the existing
  `tethers:v2:sha256:` contract exposed by the reconciled engine.
- Reconciled newer remote documentation and updated living current-truth
  documents and the control packet.

## Decisions and assumptions

The fetched live remote main was authoritative when it moved from the initial
base to `4f3e578`; it was merged normally rather than overwritten. The tag was
not rewritten. Existing `preview` remains compatible while `plan` is the
stable machine-facing surface.

## Evidence

- Integration merge: `73591c42bcd29cc655d67987ab27d2950d5cad12`.
- Plan implementation commit: `c46922e47b43d3f2de4f5a342f016dd7bdc55f19`.
- Remote-main reconciliation and final tip: `d144d7f1acacd556daa4471e6cef6c93e0b64ed0`.
- `origin/main` and `origin/integrate/tethers-main-plan-20260910` both resolve
  to `d144d7f1acacd556daa4471e6cef6c93e0b64ed0`; GitHub API confirmed the
  commit and `plan_command.rs`.
- Real `j14-complete` OCaml-engine fixture returned one `fixture.ping` action,
  `tethers.plan/1`, `provider_invocations: 0`, no provider marker, and no
  Trail file.
- Serial Rust verification reached 1574 passing tests before the two stale
  digest assertions were corrected; the corrected cross-language `core9`
  group passed 51/51 and the plan group passed 5/5. The default parallel run
  showed 39 pre-existing barrier/scope failures caused by cross-test
  interference; the serial concurrency cases passed.
- OCaml `dune build @all` and `dune runtest --force` passed, including the
  Rocket, validator, bridge, adapter, wire, refinement, and success-path
  suites.
- Release host build and invalid-input smoke passed; invalid plan returned
  structured `tethers.plan/1` data with exit code 3 and zero effects.

## Discoveries

The request’s stated remote-main SHA was stale. The first fetched main was
`5cce71f`; GitHub then advanced it to `4f3e578` during publication, so the
integration branch received a normal reconciliation merge before final
fast-forward publication. The repository’s strict `just verify` warning gate
still fails on pre-existing unused imports/dead code/unused-mut warnings; the
ordinary check, clippy, focused tests, and serial full suite evidence remain
available and the new plan code adds no warning class of its own.

## Remaining risks

The strict `RUSTFLAGS=-D warnings` repository gate remains blocked by existing
warning debt outside this bounded plan change. The default parallel Rust test
runner remains unsuitable for the repository’s Windows barrier tests; serial
execution is the reliable acceptance mode observed here.

## Smallest next action

Run the next product-workload acceptance against `tethers plan` and retain its
machine-readable response alongside the existing fixture evidence.

## References

- `README.md`
- `QUICKSTART.md`
- `docs/AGENT_QUICKSTART.md`
- `docs/CURRENT_GOAL.md`
- `docs/PROJECT_DASHBOARD.md`
- `tethers-0.1/host-rust/src/plan_command.rs`
- `tethers-0.1/host-rust/src/host_execution.rs`
