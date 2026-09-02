# Tethers 0.5 — Practical release finishing packet

Control contract: `1`

Status: `READY`

Task colour: `Red`

Owner: `Codex`

Route: `Codex direct implementation in the clean release worktree; bounded finishing packet with normal remote publication`

Base commit: `21bb7442fa9f8442db98e193eb4954096f356678`

Worker note: `docs/worker-notes/2026-09-02-tethers-v0.5-release.md`

Updated: 2026-09-02

## Objective

Finish Tethers 0.5 as a practical, installable release from current
`origin/main`. Preserve frozen Enc_V2, ProgramDigest V2, Core semantics, host
authority, Plug trust, policy, scope, Trail, replay, approval, Result Anchor,
Together, and protocol behaviour exactly. Make the accepted Rocket V3 work
usable as an exact solver portfolio while keeping the exhaustive implementation
as a permanent reference engine, then close the Agent Essentials/product,
documentation, evidence, packaging, and release gaps that are already supported
by repository history.

## Relevant background and existing behaviour

`origin/main` contains the accepted V2 production cutover, R3-1 semantic model,
R3-2 typed refinement, the portable 0.2.2 workbench, and the host Plug
lifecycle. Remote branches contain separately verified Rocket exact-search/path
work and Agent Essentials discovery/workspace/coding providers. The current
source checkout is dirty and is not this task's worktree; it must remain
untouched.

The R3-3A exhaustive search is correctness authority. R3-2 refinement is a
search aid only. Existing B2 success-path work is exact only where its own
certificate and differential tests prove it. No research-only ListIso or
complexity theorem may be promoted as a production theorem without parity
evidence.

## Required behaviour

1. Create one exact Rocket V3 portfolio entrypoint that preserves a named
   exhaustive/reference engine and routes only among exact backends: R3-2
   refinement/direct forcing, accepted B2 path solving, connected matching or
   symmetry collapse only where certified safe, bounded FPT completion where
   proved, exact memoised lexicographic branch-and-bound for harder cases, and
   exhaustive reference fallback.
2. Make routing and complexity escape valves runtime-only. Budgets, thresholds,
   backend selection, memoisation, and diagnostic counters must never alter the
   frozen canonical payload, digest, parent vector, or semantic ordering; an
   exhausted optimisation path must fall back exactly or fail closed.
3. Add bounded deterministic differential tests against the exhaustive
   reference over existing V2 cases plus generated, renamed, reordered,
   repeated-subtree, path/star/balanced/asymmetric, and metamorphic cases. Stop
   a release claim on any payload or digest mismatch.
4. Reconcile the already-complete Agent Essentials discovery, workspace, and
   coding-provider work onto the current release base without weakening the
   existing Plug, scope, trust, process, Git, or verification boundaries.
5. Provide a cold-agent path through public CLI discovery, trusted capability
   inspection, installed Plug inspection, harmless bounded workspace work, and
   evidence inspection, with stable machine-readable output and truthful
   unavailable/denied states.
6. Finish the small useful reference Plug/toolbelt surface already supported by
   the provider seam, including workspace/text/patch, Git, process/named
   verification, hashes, and clear authoring/build instructions. Do not create a
   second registry, policy engine, scheduler, server, daemon, database, or AI
   framework.
7. Bring the front-door manuals and product documents to 0.5 truth: ordinary
   Windows use, Linux installation/CI artifact use, AI-first discovery, useful
   Tethers/Tether Sets, the full host/Core/Plug relationship, and the distinction
   between the portable façade and the full platform.
8. Add reproducible benchmark/release evidence for common versus difficult
   Rocket shapes, exactness/parity, memory/branch/fallback counters, and cold
   agent usability. Keep research claims and unverified platform claims clearly
   labelled.
9. Add boring reproducible Windows/Linux packaging and release automation around
   the existing pinned toolchains and portable workflow, preserving existing
   portable artifact identities and checksums. Do not claim local Linux builds
   when only CI proves them.
10. Perform only safe repository cleanup, then create the 0.5 release commit,
    tag, and release assets using normal non-force Git/GitHub publication after
    all required checks pass; report exact hashes, URLs, and any external
    publication boundary.

## Relevant components

- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_partition.ml/.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_rocket_v3_refine.ml/.mli`
- the accepted Rocket exact-search, encoder, origin-walk, and success-path
  implementations on the fetched remote history
- `tethers-0.1/engine-ocaml/bin/dune`
- `tethers-0.1/host-rust/src/{cli,application,discovery,agent_workspace,agent_coding,plug_command}.rs`
- existing `manifest`, `installed`, `enablement`, `trail_command`, `plug_*`,
  package, provider, and host test modules
- `reference-plugs/`, `scripts/`, `.github/workflows/`, `README.md`,
  `QUICKSTART.md`, and current product/agent documentation

## Frozen decisions and invariants

- Enc_V2 and ProgramDigest V2 are byte-for-byte immutable.
- Exact canonical identity is the unsigned-byte minimum under the frozen
  encoder; no raw ID, storage order, heuristic rank, graph-library label, or
  partition-cell number is semantic authority.
- Every production Rocket backend must either prove the same frozen result or
  return control to an exact backend. Reference fallback remains available in
  tests/diagnostics and is not deleted or hidden.
- Core remains deterministic and application-agnostic. Plans request Actions;
  hosts authorise and execute them; Trails record evidence.
- Plug manifests are trusted stored data only after the existing validation,
  installation, binding, scope, and policy checks. Conformance is not trust or
  permission.
- No ambient credentials, arbitrary shell interpolation, remote Git mutation,
  force push, reset-hard, automatic effectful retry, hidden network access, or
  hostile-code sandbox claim is introduced.
- Existing portable 0.2.2 artifacts and hashes remain unchanged.

## Acceptance criteria

1. One portfolio API/command and a named reference engine exist, with exact
   differential parity on every bounded case used by the release tests.
2. Runtime escape valves and backend routing affect timing/counters only; forced
   fallback and bounded failure are fail-closed and never emit a different V2
   identity.
3. The differential/metamorphic corpus reports zero payload and digest
   mismatches, with deterministic counters and explicit hard-case coverage.
4. Agent Essentials discovery/workspace/coding history is integrated on top of
   current main and its focused, adversarial, pack, inspect, and conformance
   checks pass.
5. A fresh-client/cold-agent transcript proves discovery, trusted inspection,
   harmless bounded work, and evidence inspection without undocumented setup.
6. Reference Plug packages are reproducible and pass their existing schemas,
   conformance, and trust-boundary checks; no frozen package identity changes.
7. README, QUICKSTART, product dashboard/goal, and release/agent manuals
   describe only verified 0.5 behaviour and clearly mark deferred or CI-only
   claims.
8. Benchmark evidence includes exactness, common-case timing, difficult-case
   fallback, and resource/branch counters, with no wall clock in semantic output.
9. Windows packaging and Linux CI packaging are reproducible from the pinned
   toolchains, and the pre-existing portable 0.2.2 ZIP hashes remain identical.
10. The final release commit/tag/assets are published normally, the complete
    diff contains no unsafe cleanup or frozen-semantic edits, local/remote
    release heads and tags are recorded, and the release worktree is clean.

## Required verification

- Fetch and inspect `origin/main`, relevant Rocket branches, Agent Essentials,
  and portable release history before implementation.
- Run `pwsh -NoProfile -File scripts/check-dev-tools.ps1` and the task-packet
  checker in `READY`, `IN_PROGRESS`, and final `COMPLETE` states.
- Use the exact authorised OCaml switch
  `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml` with explicit
  `opam exec --switch=...`; run `dune build @all`, focused Rocket tests, and
  `dune runtest --force`.
- Run the existing fixture, MCP transcript, host, Rust, Plug, provider,
  discovery, workspace, coding, package, and conformance checks relevant to
  changed surfaces, plus `cargo fmt --all -- --check`, locked Rust checks,
  `git diff --check`, and release packaging smoke checks.
- Run bounded/random/metamorphic Rocket differential checks against the
  reference engine and record exact totals, hashes, fallbacks, and counters.
- Verify the existing portable 0.2.2 artifact SHA-256 before and after release
  packaging; verify Windows locally and Linux only through the repository's CI
  path if no local musl toolchain exists.
- Inspect the complete base-to-HEAD diff, exact authorised paths, full commit
  hashes, tag object, release asset hashes/URLs, remote equality, and clean
  status before reporting.

## Forbidden changes

- No redesign or semantic change to V2, Core, validator, evaluator, planner,
  wire, host authority, replay, Trail, approval, Result Anchor, Together,
  Plug trust/policy/scope, or portable 0.2.2 behaviour.
- No promotion of the blocked R3-3B3A/B3B/B3C research claims without an
  independent exact parity proof; no heuristic subtree ranking or raw-ID tie
  break.
- No deletion of the exhaustive reference engine, no generic graph library, no
  new dependency, no server/MCP/database/orchestration detour, no LLM judge,
  and no hidden semantic timeout.
- No mutation of the dirty source checkout or unrelated worktrees; no broad
  historical branch merge that overwrites current main; no force push, reset,
  destructive cleanup, or unrequested PR/review operation.
- No publication claim for an artifact, Linux build, release, or physical
  install that was not actually evidenced.

## Stop conditions

- Any frozen V2 payload/digest mismatch, nondeterministic semantic output, or
  reference/portfolio disagreement after two materially different diagnoses.
- A supposedly exact backend requires a heuristic tie-break, hidden raw identity,
  unsupported theorem, or cannot expose an admissible exact fallback.
- Agent Essentials integration changes an existing trust, scope, policy,
  process, Git, package, or protocol contract, or requires a second authority.
- Required Windows/Linux toolchain, credentials, CI, or release publication is
  unavailable after safe local/remote checks; classify the affected slice
  precisely instead of claiming completion.
- Any change would touch the protected dirty checkout, frozen portable hashes,
  unrelated branches, or files outside the packet without a new packet.

## Expected pre-existing changes

None
