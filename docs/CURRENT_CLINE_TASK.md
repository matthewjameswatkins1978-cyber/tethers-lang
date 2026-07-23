# Current Cline Task

Status: `COMPLETE`

Task colour: `Red`

Base branch: `main`

Base commit: `9ed81b8335c20e6287925b0341dc16da86780508`

## Objective

Correct Joint Runtime Slice item 3 so the opaque `manifest_digest` is carried
through the real planner-to-dispatch path defined by the capability-bridge
contract, then fail closed on stale plans.

## Relevant background and existing behaviour

- `project_capabilities()` exists in `resolver.rs` and currently projects a
  deterministic capability view from admitted manifests and live availability.
- The present host pin step (`pin_projected_digest()`) compares projection and
  resolution after planning, so it does not prove that planning consumed the
  projected digest input.
- A focused mismatch test currently proves failure by mixing projection from one
  store with resolution from another. That branch is artificial relative to the
  production path, which uses one current store for both values.
- Existing fixtures and behaviour for non-bridge capabilities must remain valid.

## Required behaviour

1. Build the approved capability projection before evaluation and supply it as
   deterministic planner input.
2. Ensure bridge-backed capability planner input includes opaque
   `manifest_digest`.
3. Ensure the planner copies `manifest_digest` from capability input into the
   proposed Action without inspecting or transforming digest contents.
4. Host dispatch must compare the Action-pinned digest with the currently
   verified manifest/provider binding before execution.
5. A stale plan created when digest = D1 must fail closed when current binding
   resolves to D2.
6. Preserve compatibility via explicitly additive design so existing non-bridge
   capability fixtures continue to pass without migration.
7. Resolve the manifest/planner version-representation difference explicitly
   (manifest `1` vs planner `"1.0.0"`) before implementation. Do not introduce
   implicit conversion.
8. Correct projection documentation/tests that label ordinary provider
   unavailability as "provider mismatch".
9. Remove or replace the artificial mismatch proof that uses different stores
   for projection vs resolution; replace it with production-path-consistent
   evidence.
10. Update `docs/CURRENT_GOAL.md` and `docs/TASK_QUEUE.md` only after the real
    planner-to-dispatch digest flow is proven by focused tests and full
    verification.

## Relevant components

- `tethers-0.1/host-rust/src/main.rs`
- `tethers-0.1/host-rust/src/resolver.rs`
- `tethers-0.1/host-rust/src/provider.rs`
- `tethers-0.1/host-rust/src/validation.rs`
- `tethers-0.1/protocol/cases/`
- `docs/CAPABILITY_BRIDGE.md`
- `docs/TASK_QUEUE.md`
- `docs/CURRENT_GOAL.md`

## Invariants

- Tethers Core remains deterministic and application-agnostic.
- Planner does not inspect full manifests.
- Planner copies opaque digest bytes/strings exactly as supplied.
- Host owns trust checks for current manifest/provider binding at dispatch.
- No hidden version coercion between manifest and planner representations.
- No change to existing non-bridge semantics unless explicitly additive.

## Acceptance criteria

1. Evaluation request path is fed by approved pre-evaluation projection for
   bridge-backed capabilities.
2. Planner output Action contains pinned `manifest_digest` copied from planner
   input for matched bridge-backed Actions.
3. Dispatch verifies pinned digest against current verified binding and fails
   closed on D1 != D2 before execution.
4. Focused mismatch proof uses production-path-consistent inputs (single live
   store path), not split-store artificial setup.
5. Version representation rule (`1` vs `"1.0.0"`) is explicit, documented in
   code/tests, and covered by focused checks.
6. Existing non-bridge fixture cases remain green unchanged.
7. Full regression suite remains green.

## Required verification

Run sequentially from `tethers-0.1`:

```powershell
Set-Location host-rust; cargo fmt --check; cargo test; Set-Location ..
pwsh -NoProfile -File scripts/check-fixtures.ps1
pwsh -NoProfile -File scripts/test-engine.ps1
pwsh -NoProfile -File scripts/test-mcp-transcripts.ps1
pwsh -NoProfile -File scripts/test-host-denial.ps1
pwsh -NoProfile -File scripts/test-host-execution-failure.ps1
pwsh -NoProfile -File scripts/demo.ps1
Set-Location engine-ocaml; opam exec -- dune build; Set-Location ..
git diff --check
git status --short
```

Focused evidence must include:

- bridge-backed planner-input digest pinning path;
- stale-plan D1/D2 fail-closed path with no executor call;
- explicit version-representation handling path;
- corrected provider-unavailability labeling path.

## Forbidden changes

- No commit, push, merge, amend, or tag.
- No protocol/schema mutation without explicit architectural approval.
- No implicit manifest-version to planner-version conversion.
- No removal or regression of existing non-bridge fixture behaviour.

## Stop conditions

Stop and report before implementation if:

- introducing planner input digest pinning requires unapproved protocol/
  semantic changes;
- explicit version-representation handling cannot be made deterministic and
  additive with existing fixtures;
- required fail-closed stale-plan proof conflicts with current dispatch
  boundary invariants.

## Expected pre-existing changes

None. The completed implementation is checkpointed at the Base commit above.

Planning-control file `docs/CURRENT_CLINE_TASK.md` is intentionally excluded by
the task-packet checker from the non-planning dirty-path comparison.

Do not stage or commit unrelated files.
