# TETHERS L1a / Native Cross-Platform Verifier

Task: `TETHERS L1a / Native Cross-Platform Verifier`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Codex`

Status: `COMPLETE`

Base commit: `7054875527dcb4332f9742b83c8d19e683be77a4`

Implementation checkpoint: `707760c514cd9021a067f0bbdfd1a4893c3fe482`

## Requested outcome

Replace the PowerShell-only aggregate verification route with one repository-
owned cross-platform authority. Native Linux verification must build and prove
the current OCaml engine, run the required Rust/OCaml/protocol evidence, and
fail closed without Windows PowerShell interop or stale engine discovery.

## Changes made

- Added `scripts/verify-tethers.py` as the aggregate authority.
- Added shared process, canonical JSON, digest and fixture helpers in
  `scripts/verification_support.py`.
- Added native Python task packet, warning ratchet, compatibility, fixture and
  MCP transcript checks. Existing PowerShell entry points now delegate to the
  corresponding Python checks.
- Changed `just verify` to select `/usr/bin/python3` on native Linux and `py -3`
  on Windows, with `/bin/sh` and `windows-shell` kept as distinct Just process
  boundaries.
- Corrected the Linux coordinator to invoke the existing Bash engine/test
  launchers with Bash rather than `sh`.
- Added verifier and provenance fail-closed self-tests.
- Updated `docs/VERIFICATION.md` and the L1a task packet.
- No product, OCaml/Core, Plan, Trail, policy, approval, replay, Resolve,
  provider, package or version semantics changed.

## Decisions and assumptions

- The current accepted L1 warning debt remains visible and non-blocking: the
  duplicate Cargo target warning and Linux `from_discovered` dead-code warning.
  Static compilation is still locked and all-targets/all-features; the warning
  ratchet owns the explicit baseline.
- The current native runner remains the authority for engine provenance and
  host test selection. The coordinator does not search PATH for an engine.
- The task packet checker is external project tooling, but it is implemented
  natively so Linux verification does not need PowerShell.

## Evidence

- Starting branch: `codex/tethers-linux-parity-finish` at
  `7054875527dcb4332f9742b83c8d19e683be77a4`.
- Native verifier self-tests pass: process pass/fail, missing executable,
  dependent skip, dirty/clean eligibility, first failure, schema and quoted
  path handling.
- Native fixture and compatibility checks pass.
- Native provenance checks reject stale source commit, tampered binary hash,
  missing binary path and missing provenance, then accept the restored valid
  manifest.
- Full native `just verify` passes all ten logical suites: task packet checker,
  formatting, current OCaml engine/provenance, OCaml tests, Rust static checks,
  warning ratchet, Rust/cross-language tests, protocol fixtures, MCP
  transcripts and compatibility corpus.
- Machine report: `verification/tethers-verification.json`, schema
  `tethers.verify/1`, platform `linux`, 10 PASS, 0 FAIL, 0 skipped, verdict
  `PASS`. The report is currently not release-eligible because this worktree is
  intentionally dirty with L1a changes.
- A separate clean Windows clone of the pushed branch passed the same ten-suite
  Python authority with the existing Windows OCaml switch. Its report verdict
  was `PASS` with current engine provenance and no dependent-suite skips.
- Rust and OCaml evidence used the prepared `bl-tethers-5.5.0` switch, OCaml
  `5.5.0`, Dune `3.24.0`, and the current engine SHA
  `4c89ff26024733218311086f0fff66c0a7fe2e652363a636a7efd358bd60e9af`.

## Discoveries

- The old aggregate invoked Windows PowerShell from WSL and failed with an
  unsigned-script `SecurityError` before product verification.
- The existing Linux `.sh` engine and Rust launchers are Bash scripts; invoking
  them through `sh` caused Bash-specific `$#`/`${BASH_SOURCE}` failures. The
  native coordinator now invokes them with Bash.
- Dune is correctly available through the explicit opam switch rather than
  ambient PATH. The machine report now checks Dune and OCaml through that
  switch.
- The full product matrix itself remained green once the verifier reached it;
  no L1b product failure was discovered.

## Remaining risks

- `bl doctor tethers` has the previously known rust-analyzer environment issue;
  no workshop configuration was changed.
- The accepted warning baseline remains technical debt and is intentionally not
  repaired in L1a.

## Smallest next action

Publish the completed branch normally. L1 should review and integrate it with
the normal no-force workflow before the next packet begins.

## References

- `docs/VERIFICATION.md`
- `scripts/verify-tethers.py`
- `scripts/prepare-current-engine.sh`
- `scripts/run-rust-tests.sh`
- `verification/current-engine-provenance.json` (ignored generated evidence)
