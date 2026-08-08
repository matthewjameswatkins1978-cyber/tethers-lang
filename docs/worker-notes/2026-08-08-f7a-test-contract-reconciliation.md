# Worker Note

Task: `F7a — Current Test Contract Reconciliation`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `2a2417f5d943a5c1ca27d6c646746cfaf7b93a86`

Implementation checkpoint: `4946629f31c6156e66d187e432f64e55297c7233`

## Requested outcome

Reconcile the F1 test/debt inventory (M7, M8, M9) against the current accepted Foundation state at base `2a2417f5`. Characterise all-features test failures, build a duplicate-candidate table, catalogue protected evidence, and produce an F7 authorisation table. No production, test, fixture, build, or dependency changes permitted.

## Changes made

- `docs/CURRENT_CLINE_TASK.md` — compiled F7a task packet replacing the completed F1-R1 packet
- `docs/foundation-pass/TEST_CONTRACT_RECONCILIATION_F7A.md` — comprehensive reconciliation evidence document

No Rust, OCaml, build, test, fixture, protocol, script, dependency, or production files changed.

## Decisions and assumptions

1. **M9 classified as RESOLVED BY INTERVENING WORK.** Zero `pub(crate) mod` declarations exist for test modules; all tests use `#[cfg(test)] mod`. This matches the expected F5 outcome.

2. **M8 classified as CORRECT PRIVATE OWNERSHIP.** All 15 dedicated test files and 33 inline test modules in `src/` are properly conditioned with `#[cfg(test)]` and test private modules at their private boundary. Moving to `tests/` would require widening production visibility (forbidden) or lose private-invariant evidence (weakens proof).

3. **No duplicate candidates warranting removal found.** The 1331-test surface was reviewed for genuinely redundant properties at the same boundary. Tests that call the same function exercise different inputs, code paths, or evidence dimensions that each contribute unique proof.

4. **The 6 all-features failures from F1-R1 are RESOLVED.** At base `2a2417f5`, `cargo test --all-targets --all-features --locked` passes with 1331/1331 tests. This is a Foundation state improvement, not a test-suite regression.

5. **OCaml switch ambiguity.** `OcamlSwitchPath: N/A` in the task packet reflects that no single switch is designated for this worktree. The existing switch at `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml` was used with explicit `--switch` and `$env:OPAMSWITCH` for OCaml verification commands.

6. **cargo fmt failure is pre-existing and independent.** The `replay_windows.rs:3277` formatting diff predates F7 and is not actionable in this audit.

## Evidence

### Git state

- `git status --short`: clean working tree
- `git branch --show-current`: `foundation/f7a-test-contract-reconciliation`
- `HEAD`: `31479d9dfe611ef0c16d5e5924e48dfe42afd296`
- `base commit`: `2a2417f5d943a5c1ca27d6c646746cfaf7b93a86`
- `origin/main`: `40ec42eb2aac108901d428af3cbfe264d3edd6dc`
- `git diff --name-only -- tethers-0.1/host-rust/`: (empty)
- `git diff --name-only -- tethers-0.1/engine-ocaml/`: (empty)
- `git diff --name-only -- tethers-0.1/protocol/`: (empty)
- `git diff --name-only HEAD~1..HEAD`: `docs/CURRENT_CLINE_TASK.md` only

### Rust tests

- `cargo test --locked`: PASS — 1331 passed, 0 failed, 2 ignored (14.26s)
- `cargo test --all-targets --all-features --locked`: PASS — 1331 passed, 0 failed, 2 ignored (15.01s)
- `cargo fmt --all -- --check`: KNOWN FAILURE — `replay_windows.rs:3277` formatting diff (pre-existing, out of scope)

### OCaml

- `opam exec --switch "..." -- dune build`: PASS (clean)
- `opam exec --switch "..." -- dune runtest`: PASS (no tests configured)
- `pwsh -NoProfile -File tethers-0.1/scripts/test-engine.ps1`: PASS — 29 fixture cases all matched (with `$env:OPAMSWITCH` set)
- `pwsh -NoProfile -File tethers-0.1/scripts/test-mcp-transcripts.ps1`: PASS — 15 MCP transcript cases passed

### Fixtures and checks

- `pwsh -NoProfile -File tethers-0.1/scripts/check-fixtures.ps1`: PASS — 46 JSON files, 30 JSONL files valid
- `git diff --check`: PASS
- `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1`: PASS — control-v1/IN_PROGRESS

### Test topology (read-only audit)

- Rust: 15 dedicated `#[cfg(test)]` module files in `src/`, 33 production files with inline `#[cfg(test)] mod tests`, 22 integration test files in `tests/`
- OCaml: 8 `.ml` files, 4 `.mli` files, 0 native tests
- PowerShell: `test-engine.ps1` (29 cases), `test-mcp-transcripts.ps1` (15 cases), plus ~15 additional host test scripts

## Discoveries

1. **F1-R1 all-features failures (6) are RESOLVED at `2a2417f5`.** The Foundation work between `ea7426d` (F5) and `2a2417f5` (current base) eliminated these failures.

2. **M9 is fully resolved.** No `pub(crate)` test module exposure exists anywhere. The `lib.rs` uses only `#[cfg(test)] mod` for test modules and one `pub(crate) use` block for production code sharing.

3. **No genuine duplicate tests found.** Every test identified as a potential duplicate during deep inspection contributes unique evidence (different failure paths, inputs, or evidence dimensions).

4. **The `scripts/check-fixtures.ps1` path in the task packet is incorrect.** The actual path is `tethers-0.1/scripts/check-fixtures.ps1`. The task packet lists `scripts/check-fixtures.ps1` which does not exist at the repository root.

## Remaining risks

- **No opam switch is designated for this worktree.** The existing switch from another worktree (`D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`) works but requires explicit invocation. This is a project-environment concern, not an F7a finding.
- **cargo fmt failure in replay_windows.rs** remains unresolved and independently owned.
- **LSP diagnostics in integration tests** report private module accessibility errors that are expected (private modules cannot be accessed by integration tests); these are design-level warnings, not build failures.

## Smallest next action

If Lucy accepts this audit, the next step is either:

1. **F7b (if authorised):** Add 1-2 narrow OCaml native test files for Tethers_error.fail, Tethers_outcome.error_response, and Tethers_outcome.json_of_response, with zero new dependencies. Scope is 2-4 named properties, Green risk.

OR

2. **F8 (if F7 consolidation is complete):** Proceed to the next Foundation pass as Lucy directs. F7a recommends NO-OP for Rust test consolidation.

DO NOT START either without explicit Lucy authorisation.

## References

- `docs/CURRENT_CLINE_TASK.md` — F7a task packet
- `docs/foundation-pass/TEST_CONTRACT_RECONCILIATION_F7A.md` — complete reconciliation evidence
- `tethers-0.1/host-rust/src/lib.rs` — Rust module declarations
- `tethers-0.1/engine-ocaml/bin/dune` — OCaml build configuration
- `tethers-0.1/engine-ocaml/bin/tether_parser.mli` — parser interface
- `tethers-0.1/engine-ocaml/bin/tethers_outcome.mli` — outcome interface
- `tethers-0.1/engine-ocaml/bin/tethers_evaluator.mli` — evaluator interface
- `tethers-0.1/engine-ocaml/bin/tethers_error.mli` — error interface
- `tethers-0.1/scripts/test-engine.ps1` — engine fixture tests
- `tethers-0.1/scripts/test-mcp-transcripts.ps1` — MCP transcript tests
- `tethers-0.1/scripts/check-fixtures.ps1` — fixture integrity check
- Base commit: `2a2417f5d943a5c1ca27d6c646746cfaf7b93a86`
- Audit checkpoint: `4946629f31c6156e66d187e432f64e55297c7233`
- Packet fix commit: `31479d9dfe611ef0c16d5e5924e48dfe42afd296`
- Branch: `foundation/f7a-test-contract-reconciliation`
