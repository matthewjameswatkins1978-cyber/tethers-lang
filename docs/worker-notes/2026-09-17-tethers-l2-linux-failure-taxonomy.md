# Tethers L2 — Linux Failure Taxonomy

Status: COMPLETE

## Authority

- Repository: `/home/matmus/biscuit-linux/projects/tethers`
- Branch: `feature/tethers-l2-linux-failure-taxonomy`
- Starting SHA: `47571579315c278bdcdbbef0f0403520bd3b41ed`
- Starting point: accepted and pushed L1 branch `feature/tethers-l1-linux-portability`
- Scope: investigation and evidence only; no Tethers product/runtime changes

## Reproduction

The full native WSL run was executed once from `tethers-0.1/host-rust`:

```text
cargo test --workspace
```

Complete combined output is preserved at:

```text
docs/worker-notes/evidence/tethers-l2-linux-failures.log
```

Observed result:

```text
test result: FAILED. 1258 passed; 245 failed; 4 ignored; 0 measured; 0 filtered out
finished in 18.78s
```

The reproducible machine-readable extraction is at:

```text
docs/worker-notes/evidence/tethers-l2-linux-failures.json
```

The JSON contains one entry for every captured failure block, the immediate diagnostic, the probable platform dependency, confidence, and the first shared root-cause cluster.

## Failure taxonomy

| Cluster | Count | Earliest shared assumption | Confidence |
| --- | ---: | --- | --- |
| Windows launch environment | 144 | Current-trust and installation tests require `SystemRoot`; native WSL does not provide the Windows launch environment | High |
| Concurrency cascade | 41 | A shared test lock is poisoned after an earlier platform-sensitive setup failure; these are dependent failures, not 41 independent defects | High |
| Engine provenance | 21 | Cross-language tests require a current repository-built/provenance-verified OCaml engine, which was not available in this worktree | High |
| Windows path semantics | 16 | Fixtures use Windows-style absolute paths while Linux `Path` semantics reject or reinterpret them | High |
| Windows provider fixture | 16 | stdio fixtures do not complete because their Windows-side child-process boundary is unavailable | Medium |
| Windows process boundary | 4 | PowerShell/process-inspection tests require Windows executable/process tooling | High |
| Unresolved Linux behaviour | 3 | The captured assertion/outcome does not by itself prove a shared platform dependency | Low |

The counts sum to all 245 failed test blocks. The full per-test record is the JSON evidence file, not this summary table.

## Root-cause interpretation

The earliest dominant assumption is that the host runtime tests are executing in a Windows launch environment. The clearest direct error is:

```text
M3Error { code: "launch_environment", message: "SystemRoot is unavailable" }
```

That accounts for 144 failures. The 41 `PoisonError` failures are classified as a cascade because they report a poisoned shared concurrency-test lock after an earlier panic; they should not be treated as independent product failures until the platform prerequisite is isolated.

The second independent prerequisite is the current OCaml engine. Twenty-one tests stop at:

```text
verified engine missing; run just test-rust so the current OCaml engine is built and provenance-checked
```

This is a verification/environment failure. It is not evidence of 21 Core semantic defects.

The remaining high-confidence groups are Windows-specific path, process, and provider-fixture assumptions. Three cases remain deliberately unresolved:

- `host_execution::tests::c2a3a_group_join_after_all_terminals`
- `plug_pack::tests::p2a_refuses_wrong_extension`
- `trail_command::tests::j13c_missing_trail_file_maps_to_not_found`

L2 does not repair or reinterpret these cases. They are explicit L3 investigation subjects.

## Changes and boundaries

Created only investigation evidence:

- this worker note;
- `docs/worker-notes/evidence/tethers-l2-linux-failures.log`;
- `docs/worker-notes/evidence/tethers-l2-linux-failures.json`.

No Rust, OCaml, test expectation, protocol, runtime, configuration, or release implementation file was changed. No test was disabled or weakened. The Windows canonical checkout was not modified.

## Required follow-up verification

The final L2 control checks are run after the evidence files are recorded:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace --no-run`
- `bl verify tethers`
- `bl doctor tethers`
- `git diff --check`
- clean Git status

The full `cargo test --workspace` result is intentionally retained as the subject of this packet; L2 does not require that platform-mixed suite to become green.

## Recommended L3

Build a native Linux failure-isolation matrix from these clusters. Start with explicit platform classification and test gating for Windows-only launch/process/provider fixtures, then establish a current OCaml engine in WSL and rerun the cross-language subset. Re-run the three unresolved assertion-only cases independently after those prerequisites are controlled. Do not apply broad product changes based only on the aggregate 245-failure result.
