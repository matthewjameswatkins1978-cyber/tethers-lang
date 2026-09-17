# TETHERS L1a - Native Cross-Platform Verifier

Task: `TETHERS L1a / Native Cross-Platform Verifier`

Control contract: `1`

Status: `IN_PROGRESS`

Task colour: `Amber`

Owner: `Codex`

Route: `Replace the PowerShell-only aggregate verifier with one repository-owned cross-platform authority, prove native Linux execution, document the contract, commit and push without merging.`

Base commit: `7054875527dcb4332f9742b83c8d19e683be77a4`

Worker note: `docs/worker-notes/2026-09-17-tethers-l1a-native-verifier.md`

Suggested branch:

`codex/tethers-l1a-native-verifier`

## Objective

Make `just verify` authoritative and native on both Windows and Linux. The
verification result must describe this checkout, its current first-party OCaml
engine, every required suite, and any explicit skip or failure. Linux must not
invoke Windows PowerShell through WSL interop or inherit a stale engine.

## Relevant background and existing behaviour

The native engine preparation and Rust runner already exist as Linux shell
entry points and bind the engine to the current source commit, tree and SHA-256.
The aggregate `just verify` recipe still invokes `scripts/verify-tethers.ps1`,
which causes WSL interop to select Windows PowerShell and fail before product
verification. The pure fixture, MCP and compatibility checks are currently
PowerShell-only. L1 product semantics and the Windows checkout are outside this
task.

## Required behaviour

1. Provide one cross-platform Python verification authority at `scripts/verify-tethers.py`.
2. Preserve the `tethers.verify/1` report schema and required evidence fields.
3. Run the required packet, formatting, engine, OCaml, Rust, warning, protocol, MCP and compatibility suites in a fixed order.
4. Use the existing native Linux engine preparation and Rust runner without PowerShell interop.
5. Preserve the existing Windows engine and Rust verification contract through a native Windows process boundary.
6. Report PASS, FAIL, SKIPPED WITH REASON and NOT APPLICABLE explicitly and fail on required failures.
7. Make pure helper checks available natively and retain PowerShell files only as compatibility wrappers where practical.
8. Add verifier self-tests covering subprocess outcomes, dirty state, first failure, release eligibility, report schema and paths containing spaces or quotes.
9. Make `just verify` invoke the same authority on both platforms and update verification documentation.
10. Prove the Linux verification process does not resolve or invoke `pwsh.exe` while still passing the authoritative route.

## Relevant components

- `justfile`
- `scripts/verify-tethers.py`
- `scripts/verify-tethers.ps1`
- `scripts/test-verify-tethers.py`
- `scripts/test-engine-provenance.py`
- `scripts/verification_support.py`
- `scripts/prepare-current-engine.sh` and `.ps1`
- `scripts/run-rust-tests.sh` and `.ps1`
- `scripts/check-warning-ratchet.py` and `.ps1`
- `scripts/check-compatibility-corpus.py` and `.ps1`
- `tethers-0.1/scripts/check-fixtures.py` and `.ps1`
- `tethers-0.1/scripts/test-mcp-transcripts.py` and `.ps1`
- `.github/scripts/check-tethers-task-packet.py` and `.ps1`
- `docs/VERIFICATION.md`

## Frozen decisions and invariants

- No Tethers product semantics, OCaml Core, Plan, Trail, policy, approval, replay, Resolve, provider, version or package changes.
- The current OCaml engine must be built from this checkout and verified by provenance before dependent tests run.
- Rust verification uses the repository lockfile and existing test protocol.
- Linux verification must not call `pwsh.exe`, `powershell.exe`, `cmd.exe` or Windows-side tools.
- The machine report remains safe, deterministic and free of credentials or arbitrary environment dumps.
- A verifier failure is not a product failure unless a required product suite actually runs and fails.

## Acceptance criteria

1. The Python authority exists and emits `tethers.verify/1` with source, toolchain, engine, suites, counts, first failure, verdict and release eligibility.
2. `just verify` uses the Python authority on Linux and Windows.
3. Native Linux verification builds and proves the current OCaml engine before host tests.
4. Missing, stale or invalid engine provenance fails closed and dependent suites are reported skipped with reason.
5. The native Linux route does not invoke `pwsh.exe` or Windows interop.
6. Pure fixture, transcript, compatibility, warning and task checks have native implementations with thin compatibility wrappers.
7. Verifier self-tests cover pass, failure, missing executable, skip, dirty/clean, first failure, release eligibility, schema and unusual paths.
8. Existing Windows engine semantics remain unchanged and the PowerShell wrapper delegates to the same authority where applicable.
9. `docs/VERIFICATION.md` describes the actual cross-platform route and report.
10. Formatting, focused tests, native `just verify`, whitespace and Git evidence are recorded; no product semantics change.

## Required verification

- Native Python verifier self-tests.
- `cargo fmt --all -- --check`.
- Rust static checks and the current engine/provenance route.
- Native Linux `just verify` with a sanitized process-local PATH proving no `pwsh.exe` is available.
- `git diff --check` and clean status before publication.
- Windows verification evidence if a clean Windows checkout can be used without touching the canonical checkout.

## Forbidden changes

- Tethers product semantics or Core syntax.
- Plan, Trail, policy, approval, replay, Resolve, provider or packaging behaviour.
- WSL configuration, global Git configuration, toolchains, authentication or PATH.
- The Windows canonical checkout.
- Weakening tests or treating an unverified engine as current.
- Direct main updates, force-pushes, merges or version changes.

## Stop conditions

- A real product test failure after the verifier is repaired; report it as a separate L1b defect.
- Current engine provenance or fail-closed behaviour regresses.
- Native verification requires Windows interop or a new semantic authority.
- A required toolchain is unavailable and cannot be diagnosed without workshop surgery.
- Two materially similar failed implementation attempts against the same approach.

## Expected pre-existing changes

- The L1 branch remains the source baseline and is not rewritten.
- The ignored generated engine and verification reports may be refreshed by tests.
- The Windows canonical checkout may contain unrelated work and must remain untouched.

## Implementation scope

- `justfile`
- `scripts/verify-tethers.py`
- `scripts/verification_support.py`
- `scripts/check-warning-ratchet.py`
- `scripts/check-warning-ratchet.ps1`
- `scripts/check-compatibility-corpus.py`
- `scripts/check-compatibility-corpus.ps1`
- `tethers-0.1/scripts/check-fixtures.py`
- `tethers-0.1/scripts/check-fixtures.ps1`
- `tethers-0.1/scripts/test-mcp-transcripts.py`
- `tethers-0.1/scripts/test-mcp-transcripts.ps1`
- `.github/scripts/check-tethers-task-packet.py`
- `.github/scripts/check-tethers-task-packet.ps1`
- `docs/VERIFICATION.md`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/worker-notes/2026-09-17-tethers-l1a-native-verifier.md`
