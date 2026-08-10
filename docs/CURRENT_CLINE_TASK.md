# Current Implementation Task

Control contract: `1`
Task: `TETHERS-0.3-P1-R1F — Migrate J23C2 to Generic Scope Conformance`
Owner: `OpenCode`
Status: `COMPLETE`
Task colour: `Amber`
Route: `OpenCode implements migration`
Worker note: `docs/worker-notes/2026-08-10-0.3-p1-r1f-j23c2-generic-scope-conformance.md`
Base branch: `feature/0.3-p1-r1e-synthetic-unrelated-plug`
Base commit: `36f1790550309b5695cfd1aa4f3e03223594dba8`
Implementation branch: `feature/0.3-p1-r1f-j23c2-generic-scope-conformance`
Implementation checkpoint: `08d0b81bca2fae8d1da5bfeb1d654c78db421a6f`
OCaml switch path: `not applicable`
Rust toolchain: `1.97.1`
Rust change class: `GREEN_AMBER_EVIDENCE_MIGRATION`

## Objective

Repair the stale J23C2 PDF conformance proof so it tests the current generic Operational Scope / conformance architecture rather than the retired PDF query-root placeholder design.

## Relevant background and existing behaviour

- The PDF reference package (`build_reference_package`) declares empty `launch.arguments` (no `--query-root` placeholder).
- The provider binary receives scope through `TETHERS_OPERATIONAL_SCOPE_JSON` in normal mode; conformance fallback (`TETHERS_CONFORMANCE=1`) uses `TEMP` as query root.
- `PreparedSupervisedLaunch::prepare()` creates conformance environment with `TETHERS_CONFORMANCE=1` and `TEMP` only (no operational scope delivery).
- The retired `--query-root __TETHERS_PDF_QUERY_ROOT__` placeholder model is gone from all production code.
- J23C2 still tested the retired placeholder model.

## Frozen decisions and invariants

1. No production code changes expected.
2. Test/fixture/documentation changes only.
3. If a tiny generic bug is found, STOP and report it; do not quietly repair architecture.
4. Overall P1 remains `completion repair in progress`.
5. The retired placeholder names must not appear in the repaired J23C2 file.

## Required behaviour

### Part A — Provider Startup Evidence

1. Normal mode with valid TETHERS_OPERATIONAL_SCOPE_JSON starts, proves MCP initialise + tools/list.
2. Normal mode with no operational scope refuses (non-zero exit, empty stdout, "configuration refused" on stderr).
3. TETHERS_CONFORMANCE=0 with no operational scope refuses (exact "1" gate).
4. Exact TETHERS_CONFORMANCE=1 with valid TEMP starts (no scope JSON, MCP initialise + tools/list works).
5. Exact conformance mode with missing TEMP refuses.
6. Exact conformance mode with invalid TEMP refuses (relative, absent/nonexistent).

### Part B — Real Package Host Conformance

- package_id = tethers.pdf-tools, provider_id = tethers-pdf-provider
- candidate.launch_arguments is EMPTY, prepared.evidence.arguments is EMPTY
- conformance disposition is Passed, retry_count=0, raw_stderr_persisted=false
- conformance environment contains TETHERS_CONFORMANCE + TEMP, excludes TETHERS_OPERATIONAL_SCOPE_JSON + TETHERS_OPERATIONAL_SCOPE_DIGEST
- All required individual case assertions retained
- No conformance_session failure
- Frozen pdf.inspect@1 manifest digest retained

## Acceptance criteria

1. package inspection still accepts tethers.pdf-tools
2. launch_arguments is empty (no retired placeholder)
3. conformance disposition is Passed
4. all required cases passed
5. no conformance_session failure
6. environment names checked for conformance machinery presence and operational scope absence

## Expected pre-existing changes

None

## Relevant components

### Authorised paths

- `tethers-0.1/host-rust/tests/j23c2_pdf_conformance.rs` (rewritten)
- `docs/CURRENT_CLINE_TASK.md` (this file)
- `docs/worker-notes/2026-08-10-0.3-p1-r1f-j23c2-generic-scope-conformance.md` (new)

## Required verification

1. J23C2 tests pass (7 tests)
2. pdf_tools_provider bin tests pass (9 tests)
3. J23B package tests pass (1 test)
4. `cargo check --all-targets --all-features --locked` clean
5. `cargo fmt --all -- --check` clean
6. `git diff --check` clean
7. Retired placeholder grep (no matches in j23c2_pdf_conformance.rs)

## Forbidden changes

- No production code changes
- No compatibility shim for retired `--query-root` placeholder
- No J23B or J23C3 changes
- No P2, migration tooling, concurrency, or unrelated cleanup
- No full verify-agent, engine fixture suite, MCP transcript suite, or final P1 gate

## Stop conditions

- Production code needs a change
- A required test or check has two materially similar failed attempts
- A tiny generic bug is found (report, don't fix)
