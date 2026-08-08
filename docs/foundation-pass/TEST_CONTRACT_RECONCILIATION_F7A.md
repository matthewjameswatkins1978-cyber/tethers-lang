# F7a Test Contract Reconciliation

Control task: F7a
Base: `2a2417f5d943a5c1ca27d6c646746cfaf7b93a86`
Audit checkpoint: `4946629f31c6156e66d187e432f64e55297c7233`
Packet fix commit: `31479d9`
Final HEAD: `532126810ad51dfbf6d75472854c9cb49d8d0811`

## 1. Exact Base SHA

`2a2417f5d943a5c1ca27d6c646746cfaf7b93a86`

## 2. Current Test Topology

### Rust (host-rust)

| Category | Count | Approx. lines |
|----------|-------|---------------|
| Dedicated `#[cfg(test)]` module files in `src/` | 15 | ~10,626 |
| Production files with inline `#[cfg(test)] mod tests` | 33 | ~83 blocks |
| Pub mod declarations in lib.rs | 39 | -- |
| Private mod declarations in lib.rs | 15 | -- |
| `#[cfg(test)] mod` declarations in lib.rs | 15 | -- |
| `pub(crate) mod` declarations | 0 | -- |
| Integration tests in `tests/` | 22 | ~11,461 |
| Doc tests | 0 | -- |
| **Total Rust tests** | | **1331 passed, 0 failed, 2 ignored** |

### OCaml (engine-ocaml)

| Module | .ml lines | .mli lines | Has .mli |
|--------|-----------|------------|----------|
| Tethers_error | 4 | 3 | Yes |
| Tether_parser | 178 | 34 | Yes |
| Tethers_protocol | 101 | None | No |
| Tethers_outcome | 89 | 39 | Yes |
| Tethers_evaluator | 249 | 1 | Yes |
| Tethers_mcp_server | 317 | None | No |
| Tethers_mcp_main | 29 | None | No |
| Main | 20 | None | No |
| **Total** | **995** | **77** | **4/8** |

**No OCaml-native tests exist.** No library stanza, no test stanza, no inline test DSL, no cram tests.

### External Test Scripts (PowerShell)

- `test-engine.ps1` — invokes compiled engine against fixture cases (29 tests)
- `test-mcp-transcripts.ps1` — MCP JSON-RPC transcript validation (15 tests)
- Plus ~15 additional test scripts for host scenarios

## 3. Current Command Results

| Command | Result | Detail |
|---------|--------|--------|
| `git status --short` | CLEAN | No dirty files |
| `cargo test --locked` | PASS | 1331 passed, 0 failed, 2 ignored |
| `cargo test --all-targets --all-features --locked` | PASS | 1331 passed, 0 failed, 2 ignored |
| `opam exec -- dune build` | PASS | Clean build |
| `opam exec -- dune runtest` | PASS | No tests found (none configured) |
| `test-engine.ps1` | PASS | 29 fixture cases matched |
| `test-mcp-transcripts.ps1` | PASS | 15 transcript cases passed |
| `tethers-0.1/scripts/check-fixtures.ps1` | PASS | 46 JSON + 30 JSONL valid |
| `cargo fmt --all -- --check` | KNOWN FAILURE | `replay_windows.rs:3277` formatting diff (pre-existing) |
| `git diff --check` | PASS | Clean whitespace |
| `check-tethers-task-packet.ps1` | PASS | control-v1/IN_PROGRESS |

## 4. Exact All-Features Failing Tests

**Zero failures.**

The F1-R1 evidence reported 6 failures under `cargo test --all-targets --all-features --locked` at both the historical baseline (`24428139`) and the current-F5 target (`ea7426dbeb1934cf336673d03ae2abf76146ea7d`). At the current base (`2a2417f5d943a5c1ca27d6c646746cfaf7b93a86`), all 1331 tests pass (0 failed, 2 ignored).

**Classification: CURRENTLY NOT REPRODUCED — PRIOR CAUSE UNVERIFIED.**

Git comparison `ea7426d..2a2417f5` contains documentation changes only. No production, test, fixture, build, or dependency change exists in that range that could causally explain the changed all-features result. The three commits from `ea7426d` to `2a2417f5` are:
- `227c579` — restructure closeout docs for packet checker compliance
- `28670e4` — add missing task section for packet checker compliance
- `5e31f1f` — closeout documentation (task packet + worker note)

Therefore no production/test change can account for the differing result. Environment, timing, feature state, or another cause has not been investigated. Cause remains UNVERIFIED. Do not claim the failures are fixed.

The 2 ignored tests (`#[ignore]`) are intentional and were also present in the F1-R1 baseline.

## 5. Failure Classifications

No failures to classify. The all-features test surface currently returns zero failures at the observed state. Prior F1-R1 failures are currently not reproduced; cause unverified.

The `cargo fmt --all -- --check` failure in `replay_windows.rs:3277` is **PRE-EXISTING** and **OUT OF F7 SCOPE**. It does not block test consolidation.

## 6. M7 — No OCaml-Native Tests: Reconciliation

**Current state:** CONFIRMED. Zero OCaml-native tests exist.

Analysis:
- The `dune build` file defines two executables (`main`, `tethers_mcp_main`) but no library and no test stanzas.
- 4 of 8 modules have `.mli` interfaces (Tether_parser, Tethers_error, Tethers_outcome, Tethers_evaluator).
- All OCaml logic is exercised exclusively through external process invocation by PowerShell test scripts and the Rust host's engine integration.
- Test coverage is via cross-process integration, not native unit tests.

**Classification per plausible property:**

| Property | Classification | Rationale |
|----------|---------------|-----------|
| Parser accepts valid Tether source | INDIRECT BOUNDARY TEST SUFFICIENT | test-engine.ps1 covers 29 fixture cases including happy-path, deterministic repeat, and all line ending variants. |
| Parser rejects invalid Tether syntax | INDIRECT BOUNDARY TEST SUFFICIENT | 13 distinct error cases (action-type-error, missing-fact, invalid-indentation, etc.) all tested via engine CLI. |
| Evaluator maps conditions to correct status | INDIRECT BOUNDARY TEST SUFFICIENT | Tested via MCP transcripts and engine fixtures (evaluate-matched, evaluate-not-matched, etc.). |
| Outcome projection preserves semantic distinction | INDIRECT BOUNDARY TEST SUFFICIENT | Matched/Not_matched/Error distinctions are verified through JSON response shape. |
| error_response produces a Request_error variant | DIRECT TEST JUSTIFIED | This is a simple constructor not tested in isolation; a trivial native test could confirm the response shape. |
| json_of_response round-trips all status variants | DIRECT TEST JUSTIFIED | Serialization of Contextual vs Request_error is exercised only indirectly through external JSON comparison. |
| Tether_parser.drop_prefix correctness | DIRECT TEST JUSTIFIED | Utility function with defined behaviour; not exercised through the engine surface. |
| parse_capability rejects invalid JSON shapes | DIRECT TEST JUSTIFIED | Tethers_protocol has no .mli and no native tests; capability parsing errors are not independently tested. |
| check_unique_capabilities detects duplicates | DIRECT TEST JUSTIFIED | Duplicate detection logic has no native test; currently only verified through external engine errors. |
| Line ending equivalence (LF/CRLF/mixed) | COMPATIBILITY TEST MUST REMAIN EXTERNAL | test-engine.ps1 validates-lf/validate-crlf/validate-mixed prove byte-level acceptance; external test is the correct boundary. |
| Deterministic repeat | COMPATIBILITY TEST MUST REMAIN EXTERNAL | test-engine.ps1 happy-path-deterministic-repeat proves repeatability; external cross-process test is strongest evidence. |
| MCP protocol compliance | COMPATIBILITY TEST MUST REMAIN EXTERNAL | test-mcp-transcripts.ps1 covers 15 MCP lifecycle cases; this is a protocol boundary best tested externally. |

## 7. M8 — Test Modules Inside Rust src/: Reconciliation

**Current state:** 15 dedicated test files under `src/` declared as `#[cfg(test)] mod` in `lib.rs`, plus 33 production files with inline `#[cfg(test)] mod tests` blocks.

**Zero `pub(crate) mod` declarations for test modules.**

### Classification per group

| Group | File(s) | Classification | Rationale |
|-------|---------|---------------|-----------|
| Dedicated `*_tests.rs` files (13 files) | installation_recovery_tests.rs, installation_execution_tests.rs, current_trust_tests.rs, etc. | CORRECT PRIVATE OWNERSHIP | These test private modules (`installation_recovery`, `installation_execution`, `current_trust`, etc.) whose internals are not publicly exported. Private unit tests at the private boundary are the correct ownership. |
| Dedicated `*_evidence.rs` files (2 files) | `f3c_installation_intent_publication_evidence.rs`, `f3d_bounded_persistence_stores_evidence.rs` | CORRECT PRIVATE OWNERSHIP | These test private module invariants (installation_publication_intent, m3_store) at their private boundaries. |
| Inline `#[cfg(test)] mod tests` in 33 production files | `application.rs`, `dispatch.rs`, `replay.rs`, `trust.rs`, `policy.rs`, `validation.rs`, etc. | CORRECT PRIVATE OWNERSHIP | Each tests private invariants of the module it lives in. Inline is idiomatic Rust for unit tests that need access to private items. |
| Integration tests in `tests/` (22 files) | `j13a_cli.rs`, `m3_lifecycle.rs`, `j24j_installation_reconciliation.rs`, etc. | NO ACTION | Already at the correct integration boundary. |

**Key finding:** All test modules under `src/` are private (`#[cfg(test)]` only). No production visibility is widened for tests. The original M8 claim ("blur production/test boundary") is technically true (separate files in `src/`) but architecturally sound — these files only compile in test configuration and test private modules at their private boundary. This is idiomatic Rust practice.

**No movement to `tests/` is authorised.** Moving private tests to integration tests would either require widening production visibility (forbidden by F7 rule) or lose private-invariant testing (weakens evidence).

## 8. M9 — Test Infrastructure pub(crate) Visibility: Reconciliation

**Classification: RESOLVED BY INTERVENING WORK**

Evidence:
- Zero `pub(crate) mod` declarations exist anywhere in the Rust source tree.
- All test modules use `#[cfg(test)] mod` (dedicated files) or `#[cfg(test)] mod tests { ... }` (inline).
- The `lib.rs` file contains one `pub(crate) use` re-export block for `application.rs` items (lines 92-101), but this relates to production code sharing, not test infrastructure.

## 9. Protected Compatibility/Public Evidence

The following evidence categories are protected and amply evidenced:

| Category | Evidence count | Coverage |
|----------|---------------|----------|
| Literal Foundation compatibility fixtures | 29 engine + 15 MCP = 44 | Full |
| Public CLI/output/exit-code tests | 29 (`j13a_cli.rs`) | Full |
| Trail/replay compatibility proof | 85+ `trail_command`, `replay`, `replay_windows` | Full |
| Recovery failure-path tests | 50+ `installation_recovery_*` tests | Full |
| Trust/tamper/fail-closed tests | 17 `trust.rs`, 17 `trusted_store.rs`, 16 `policy.rs` fail-closed | Full |
| test-engine.ps1 behavioural fixture execution | 29 cases | Full |
| test-mcp-transcripts.ps1 behavioural transcript execution | 15 cases | Full |
| Fixture file integrity | 46 JSON + 30 JSONL valid (tethers-0.1/scripts/check-fixtures.ps1) | Full |

**No protected evidence is weak or missing.** All categories have strong, independent proof.

## 10. Duplicate-Candidate Table

Extensive review of the 1331-test surface identified zero genuine duplicate properties where two tests at the same boundary prove the same named contract with no additional evidence from one of them.

| Duplicate candidate | Property | Proof A | Proof B | Unique evidence per proof | Removal weakens? |
|---------------------|----------|---------|---------|--------------------------|-----------------|
| `resolve_admitted_available_capability` vs `resolve_write_capability` | Resolver resolves admitted, available capability | `resolver::tests` | `resolver::tests` | A tests read capability; B tests write capability. Different admission data, different scope. | YES |
| `declared_live_admitted_allow_policy_returns_allow` vs `resolved_capability_with_allow_policy_returns_allow` | Allow policy permits | `policy::tests` | `policy::tests` | A tests declared-with-allow; B tests resolved-with-allow. Different codepaths (declared vs resolved admission). | YES |
| `projection_is_read_only` vs `resolution_is_read_only` | Read-only guarantee | `resolver::tests` | `resolver::tests` | A tests projection read-only; B tests resolution read-only. Different internal paths. | YES |
| Replay ledger recovery tests (30+) | Replay recovery semantics | `replay_windows::tests` | None overlap | Each ledger_XX test proves a distinct recovery scenario. | YES |

**Conclusion: No duplicate candidates warranting removal were identified.** Tests that call the same function exercise different code paths, inputs, or evidence dimensions.

## 11. OCaml Direct-Test Candidate Table

| # | Candidate property | Module | Justification | F7b authorised? |
|---|-------------------|--------|---------------|----------------|
| 1 | `Tethers_error.fail` raises `Tethers_error` with correct fields | Tethers_error | Single-pattern constructor not tested in isolation | NO (F7b not authorised) |
| 2 | `tethers_outcome.error_response` produces `Request_error` variant | Tethers_outcome | Response construction only indirectly tested | NO (F7b not authorised) |
| 3 | `tethers_outcome.json_of_response` round-trips all variants | Tethers_outcome | JSON serialization exercised through cross-process comparison only | NO (F7b not authorised) |
| 4 | `Tether_parser.drop_prefix` correctness | Tether_parser | Utility not exercised through engine boundary | NO (F7b not authorised) |
| 5 | Parser rejects specific malformed Tether shapes | Tether_parser | 13 error cases already tested externally; native test would be supplementary | NO (F7b not authorised) |
| 6 | Evaluator maps condition operators (Is, Contains, Gt, Gte) | Tethers_evaluator | Operator semantics tested indirectly through engine | NO (F7b not authorised) |
| 7 | `parse_capability` rejects invalid JSON shapes | Tethers_protocol | Capability parsing errors untested outside full integration | NO (F7b not authorised; Tethers_protocol has no .mli) |
| 8 | `check_unique_capabilities` detects duplicates | Tethers_protocol | Duplicate detection has no native test | NO (F7b not authorised; Tethers_protocol has no .mli) |

**F7b authorisation: NONE.** F7 completes as NO-OP. M7 is deferred.

## 12. Causal/Contract Limits

1. **OCaml tests are supplementary, not substitutive.** The external PowerShell test scripts already prove public behaviour at the CLI/MCP boundary. Native tests would strengthen internal property evidence but cannot replace compatibility fixtures.
2. **Private unit tests cannot move to integration without widening visibility.** The 15 dedicated test files and 33 inline test modules are correctly placed at their private boundaries.
3. **No test consolidation is authorised at this stage.** Duplicate candidates that warrant removal were not found. Moving tests would either weaken evidence or require forbidden visibility changes.
4. **The `cargo fmt` failure in `replay_windows.rs` is independently owned.** It predates F7 and has no bearing on test consolidation.
5. **The opam switch for OCaml builds requires an explicit switch path.** The task packet's `OcamlSwitchPath: N/A` reflects this project-level ambiguity; the existing switch at `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml` was used manually.

## 13. F7 Authorisation Table

| Item | Current reality | Evidence | F7 actionable? | Smallest next move |
|------|----------------|----------|---------------|-------------------|
| M7 — No OCaml-native tests | Confirmed: zero native tests | dune runtest passes with no tests; no test stanzas | NO — DEFER | Deferred — no native OCaml tests remain a maintainability observation, not a correctness defect. Current Dune topology has two executables sharing engine modules with no library seam; introducing native tests would require structural Dune/library work whose benefit is not justified by the proposed trivial direct properties. External engine/MCP compatibility evidence remains strong. |
| M8 — Test modules inside src/ | Confirmed: 15 cfg(test) files + 33 inline blocks | All properly conditioned; zero pub(crate) test modules | NO | No action — correct private ownership |
| M9 — pub(crate) test infrastructure | Resolved: zero pub(crate) mod declarations | Complete lib.rs and full-tree grep | NO | No action — resolved by intervening work |
| All-features test failures (F1-R1 report of 6) | Currently not reproduced: 0 failures | cargo test --all-targets --all-features --locked PASS at 2a2417f5; ea7426d..2a2417f5 is documentation-only | NO | No action — prior cause unverified |
| cargo fmt pre-existing failure | Unresolved but independent | replay_windows.rs:3277 formatting diff | NO | Owned by separate formatting decision |
| Test consolidation | No genuine duplicates found | Full property-level audit of 1331 tests | NO | No consolidation authorised |

## 14. Final F7 Verdict

**NO TEST CONSOLIDATION AUTHORISED.**

**NO F7b AUTHORISED.**

**F7 COMPLETES AS NO-OP after F7a-R1 acceptance.**

### M7 — DEFER

No native OCaml tests remain a maintainability observation, not a correctness defect. F7's authorised purpose is test-contract consolidation; no genuine duplicate tests were found. The current OCaml Dune topology has two executables sharing engine modules and no library seam; introducing clean native tests would require structural Dune/library work whose benefit is not justified by the proposed trivial direct properties. External engine/MCP compatibility evidence remains strong.

### M7 deferred to a future separately justified OCaml test/build architecture decision.

### M8 and M9 — NO ACTION

M8: All test modules correctly use `#[cfg(test)]` at private boundaries. M9: Already resolved by intervening Foundation work (zero `pub(crate) mod` declarations).

### No test consolidation is authorised. No F7b. No F8 from F7.

---

*Audit conducted against unchanged source/test tree at base `2a2417f5d943a5c1ca27d6c646746cfaf7b93a86`.*
*Audit checkpoint: `4946629f31c6156e66d187e432f64e55297c7233`*
*All verification commands run against the exact committed code state.*
