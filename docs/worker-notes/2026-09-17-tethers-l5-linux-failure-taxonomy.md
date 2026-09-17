# TETHERS L5 - Linux Failure Taxonomy and Repair

Task: `TETHERS L5 / Linux Failure Taxonomy and Repair`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `ad5a728e753e1be77c4a4e1be40c34be3f827578`

Implementation checkpoint: `41db56f8700f59ef919c2a6ca528504920085b75`

## Requested outcome

Make the remaining Linux-facing failures truthful without widening L5 into a
general concurrency or platform cleanup task. The accepted L4 engine and
fail-closed provenance contract remain the verification authority.

## Changes made

- Added `tethers-0.1/scripts/tethers-stdio-fixture.sh`, a native POSIX
  provider fixture for Linux protocol and overlap tests. It covers the modes
  required by the existing Rust tests, including discovery failures,
  catalogue drift, marker output, and controlled overlap outcomes.
- Updated the Rust test-only fixture builders in
  `tethers-0.1/host-rust/src/stdio_provider.rs` and
  `tethers-0.1/host-rust/src/host_execution.rs` to select the PowerShell
  fixture on Windows and the shell fixture on non-Windows systems.
- Preserved the Windows `.ps1` route, production provider code, provider
  ordering, replay, outcome taxonomy, and L4 provenance scripts unchanged.
- Updated `docs/CURRENT_CLINE_TASK.md` to the L5 control-v1 packet and this
  worker note to preserve the accepted evidence.

## Decisions and assumptions

- Threadmoth was used for structural discovery across the provider fixture,
  stdio provider, engine-stdio, child-process, and host-execution files. The
  material structure was the shared test-only PowerShell construction in
  `stdio_provider::tests::fixture_config`,
  `host_execution::tests::catalogue_test_provider`, and the Core9c prepared
  runtime helper. Source inspection and tests, not structural matches alone,
  determined behaviour.
- The 18 comparable provider-fixture failures had one shared cause: Linux was
  attempting to launch `pwsh.exe` and a `.ps1` fixture. The semantically
  cross-platform provider protocol tests therefore needed a native fixture,
  not Windows-only gating.
- The three named unresolved cases were investigated but not repaired. The
  group-join case remains a concurrency outcome; the plug-pack and trail cases
  use Windows path literals whose meaning is not portable to Linux.
- No production semantics changed and no Windows behaviour changed.

## Evidence

### Baseline and taxonomy

| Failure family | Before | After |
| --- | ---: | ---: |
| Missing verified engine | 0 | 0 |
| Provider fixtures | 18 | 0 |
| Unresolved Linux behaviour | 3 | 3 |
| Concurrency cascades | 41 | 41 |
| Expanded-target platform/provider | 72 | 72 |
| Other/new | 0 | 0 |

Raw Linux totals changed from `1225 passed; 81 failed; 2 ignored` to
`1241 passed; 65 failed; 2 ignored`.

The verified runner totals in both default-threading and single-threading
modes were `1405 passed; 116 failed; 2 ignored`, including the expanded
integration targets. The host-library target was `1262 passed; 44 failed; 2
ignored`; its 44 remaining failures are the 41 concurrency cases plus the
three named unresolved cases. The verified runner exits non-zero because those
out-of-scope failures remain; this is recorded rather than hidden.

Focused evidence:

- `stdio_provider::tests` passed `21/21`, including all 13 previously failing
  provider protocol cases.
- The three in-scope host overlap/provider cases passed, including
  `c2_a3a_different_provider_tools_call_overlap_is_real`,
  `c2_a3a_same_provider_tools_call_overlap_is_real`, and
  `c4_same_provider_overlap_and_inverse_completion_preserves_semantic_order`.
- `core9c_t16_wrong_event_never_dispatches` and
  `core9c_t17_missing_core_environment_fails_closed_without_dispatch` passed.
- `cargo fmt --all -- --check`, `cargo check --workspace --locked`, and
  `cargo test --workspace --no-run --locked` passed.
- `bl verify tethers` passed.
- `git diff --check` passed before the implementation checkpoint.

### Provenance

The committed-checkpoint engine preparation produced:

- binary: `tethers-0.1/engine-ocaml/_build/default/bin/tethers_mcp_main.exe`
- source commit: `41db56f8700f59ef919c2a6ca528504920085b75`
- source tree: `5a6268456152852570dcbaf05860c3f9b42ae1ec`
- SHA-256: `4c89ff26024733218311086f0fff66c0a7fe2e652363a636a7efd358bd60e9af`
- OCaml: `5.5.0`
- Dune: `3.24.0`

The fail-closed runner rejected all four simulated invalid states:

- stale source commit: rejected before tests;
- tampered binary hash: rejected before tests;
- missing engine: rejected before tests;
- missing provenance manifest: rejected before tests.

The restored valid manifest passed verification and exposed the verified
engine/provenance paths. The manifest is ignored generated state and is not a
committed task artifact.

### Deferred tests

- `cargo test --lib c2a3a_group_join_after_all_terminals -- --nocapture`
  failed with `execute_group_concurrent returned: Uncertain` and the existing
  `ExecutionServiceResult::Completed` assertion. This remains in the
  concurrency family and was not changed.
- `cargo test --lib p2a_refuses_wrong_extension -- --nocapture` failed at
  `err.message.contains(".tetherplug")`. The test passes Windows literals
  `C:\\src` and `C:\\out.zip`; on Linux the source is not absolute, so the
  wrong-extension branch is not reached. It remains an out-of-scope path
  fixture repair.
- `cargo test --lib j13c_missing_trail_file_maps_to_not_found -- --nocapture`
  failed with `left: "invalid_data"`, `right: "not_found"`. The test uses a
  Windows-style missing path and remains an out-of-scope path/error fixture
  repair.

## Discoveries

All 13 stdio provider protocol tests shared the same Windows fixture builder.
The host-execution catalogue and overlap tests shared an equivalent builder,
and the Core9c dispatch fixtures independently encoded the same PowerShell
assumption. A native fixture had to reproduce not only initialize/list/call
protocol responses but also the barrier's controlled `failed` and `uncertain`
outcomes; otherwise C4 exposed a misleading second failure.

The three named unresolved cases did not share the provider fixture root cause.
The group-join failure is concurrency-specific. The remaining two are
Windows-path fixture assumptions in otherwise cross-platform tests and are
appropriate for a later narrow path-semantics packet.

## Remaining risks

- The 41 concurrency failures remain and need a separate bounded
  investigation.
- The 72 expanded-target platform/provider failures remain explicitly
  classified and were not swept into L5.
- The two path-literal tests need a later platform-correct fixture repair.
- Existing duplicate-target and Rust warning debt remains; no new warning
  baseline was introduced by this change.

Production semantics changed: `NO`.

Windows behaviour changed: `NO`.

## Smallest next action

Run a narrow L6 path-semantics repair for `p2a_refuses_wrong_extension` and
`j13c_missing_trail_file_maps_to_not_found`, using platform-correct absolute
fixtures and preserving their assertions. Keep
`c2a3a_group_join_after_all_terminals` for a separate concurrency packet.

## References

- L4 accepted baseline: `ad5a728e753e1be77c4a4e1be40c34be3f827578`
- L5 implementation checkpoint:
  `41db56f8700f59ef919c2a6ca528504920085b75`
- `docs/worker-notes/2026-09-17-tethers-l4-linux-verified-engine.md`
- `docs/worker-notes/2026-09-17-tethers-l3-platform-test-boundary.md`
- `scripts/prepare-current-engine.sh`
- `scripts/run-rust-tests.sh`
