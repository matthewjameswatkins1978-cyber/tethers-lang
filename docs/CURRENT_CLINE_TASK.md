# Current Cline Task

Status: `PROPOSED`

Task colour: `Red`

Base branch: `main`

Base commit: `444d8a5e1235947588e30ae9381eeb8b87f99791`

## Objective

Derive the smallest host-owned live capability projection for one Tether Set
requirement from admitted verified manifests and explicit provider
availability, without changing planner semantics or transport behaviour.

## Relevant background and existing behaviour

- Joint Runtime Slice queue item 1 is complete: configured local provider
  binding plus real stdio MCP fixture admission is now verified.
- Admission currently proves one separately authored trusted manifest can be
  admitted only after discovery evidence matches host-owned binding checks.
- `ProviderAvailability` and `resolve_capability()` already enforce explicit
  host-supplied availability and exact capability name/version identity.
- The next unchecked queue item is deriving a live capability projection for
  one Tether Set with exact versions.

## Required behaviour

1. Add one host-side projection boundary that accepts:
   - one declared Tether Set requirement list (exact capability name/version),
   - the Trusted Manifest Store,
   - explicit `ProviderAvailability`.
2. Return a deterministic projection for planning that contains, per projected
   capability:
   - exact capability name,
   - exact capability version,
   - required effects,
   - opaque manifest digest.
3. Projection must fail closed per capability: missing admission, unavailable
   provider, provider mismatch, or version mismatch yields no projected entry.
4. Keep projection read-only and deterministic: no process launch, no dispatch,
   no policy decision, no planner I/O, and no protocol mutation.
5. Keep this increment bounded to one fixture-backed capability path already
   present in repository tests.

## Relevant components

- `tethers-0.1/host-rust/src/resolver.rs`
- `tethers-0.1/host-rust/src/policy.rs`
- `tethers-0.1/host-rust/src/provider.rs`
- `tethers-0.1/host-rust/src/stdio_provider.rs`
- `tethers-0.1/host-rust/src/main.rs`
- `docs/CURRENT_GOAL.md`
- `docs/TASK_QUEUE.md`

Follow existing structure. Stop before editing if the smallest correct change
requires planner protocol/schema changes outside this boundary.

## Invariants

- Tethers Core remains deterministic planner only.
- Provider discovery metadata remains untrusted evidence.
- Trusted manifest authority remains host-owned and pre-verified.
- Capability identity remains exact name + exact version.
- Projection carries opaque digest only; planner must not inspect full
  manifest content.
- Explicit host availability input remains authoritative; no implicit discovery.
- No trust-boundary relaxation, retries, automatic restart/reconnect,
  or application-specific branching.

## Acceptance criteria

1. One focused host test proves a declared requirement projects if and only if
   it is admitted, available, and exact-version resolved.
2. Focused host tests prove fail-closed omission for every declared mismatch
   branch: missing admission, unavailable provider, provider mismatch, and
   exact-version mismatch.
3. Projection output includes exact capability name/version, effects,
   and manifest digest.
4. Projection logic has no side effects and does not mutate store or
   availability snapshots.
5. Existing admission, resolver, policy, dispatch, denial, execution-failure,
   and demo behaviour remains unchanged.
6. `docs/CURRENT_GOAL.md` and `docs/TASK_QUEUE.md` are updated only after
   projection behaviour is fully verified.

## Required verification

Run sequentially from `tethers-0.1`:

```powershell
pwsh -NoProfile -File scripts/check-fixtures.ps1
pwsh -NoProfile -File scripts/test-mcp-transcripts.ps1
pwsh -NoProfile -File scripts/test-host-denial.ps1
pwsh -NoProfile -File scripts/test-host-execution-failure.ps1
pwsh -NoProfile -File scripts/demo.ps1
Set-Location engine-ocaml; opam exec -- dune build; Set-Location ..
Set-Location host-rust; cargo fmt --check; cargo test; Set-Location ..
git diff --check
git status --short
```

If a named script does not exist or the repository requires a different
working directory, inspect and use the canonical equivalent; report the
adjustment exactly.

## Forbidden changes

- No OCaml language or planner semantic changes.
- No `tethers-0.1/SPEC.md` change.
- No new dependency.
- No network service, FFI, database, or message broker.
- No retries, automatic discovery, or automatic provider restart/reconnect.
- No execution-path expansion (no `tools/call` work in this increment).
- No capability-projection-to-planner protocol mutation.
- No commit, push, merge, amend, tag, deletion, or installation unless this
  packet is explicitly approved for implementation.

## Stop conditions

Stop and report before implementation if:

- projection requires planner protocol/schema changes not already approved;
- provider identity ownership would move outside host configuration;
- projection requirements conflict with current resolver trust boundaries;
- unrelated working-tree changes overlap implementation files;
- two focused correction attempts fail to converge.

## Expected pre-existing changes

None. The working tree was clean when this packet was prepared.

Do not stage or commit unrelated files.
