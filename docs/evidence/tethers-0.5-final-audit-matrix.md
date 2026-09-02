# Tethers 0.5 final-audit evidence matrix

Audit baseline: `31d5e39a1e3505880e9a98cd8c650b3cf112b16d`  
Audit branch: `release/tethers-v0.5-final-audit`  
Audit state: implementation closed; publication pending

This matrix records the repository and hosted evidence inspected before any
feature implementation in the final-audit worktree. `DONE` means the packet
requirement is already evidenced. `PARTIAL` means a bounded closure item is
needed. `MISSING` means no current shipped/evidence surface was found.
`DEFERRED-WITH-REASON` is used only where the packet's safety or architecture
boundary makes a new 0.5 subsystem the wrong release change.

| Packet item | Classification | Evidence and smallest closure decision |
| --- | --- | --- |
| 1. Exact fresh base/worktree | DONE | Fresh `release/tethers-v0.5-final-audit` worktree is clean at the exact packet base. |
| 2. Pre-implementation evidence matrix | DONE | This matrix is created before implementation. |
| 3. Permanent exhaustive Rocket reference | DONE | `tethers_core_canonical_v2_reference.ml/.mli` and `tethers_core_rocket_v3_portfolio.ml`; explicit reference mode is covered by the portfolio test. |
| 4. Portfolio/reference bounded identity checks | DONE | Portfolio test reports `41/41`; release evidence records bounded/random/metamorphic parity and zero mismatches. |
| 5. First-class Benchmarker surface | DONE | `tethers-0.1/engine-ocaml/bin/tethers_benchmarker.ml` and the public `tethers-bench` Dune target provide one Rocket-aware front door. |
| 6. Benchmarker JSON/human/compare/context output | DONE | `docs/TETHERS_BENCHMARKER.md` defines stable JSON/human/compare/context/resource output; the crucible produced schema `tethers.benchmarker/1`, five cases, and three comparison rows. |
| 7. Benchmark data excluded from semantics | DONE | The benchmarker checks portfolio/reference parity before timing and never feeds measurement data into the frozen encoder or authority path. |
| 8. Cold agent performs real bounded work | DONE | J14A now proves public discovery, inspection, preview, harmless fixture execution, Result Anchor, Trail inspection, and receipt. |
| 9. Cold agent needs no hidden shortcut | DONE | The J14A script uses public CLI surfaces and records the seven-case/121-assertion journey in the cold-agent evidence. |
| 10. Public side-effect-free plan/preview | DONE | Public `preview` uses the existing host/Core validation and planning boundary without authority, provider, replay, intent, or Trail writes. |
| 11. Preview distinctions and no execution | DONE | The preview envelope reports parsed/validated/evaluation/planner/authority/execution distinctions and has a real no-invocation assertion in J14A. |
| 12. Practical Trail query/receipt | DONE | `trail --receipt` is a bounded projection over the existing validated execution-filtered Trail entries. |
| 13. Receipt causal story | DONE | The receipt exposes bounded sequence/phase/kind/outcome and available causal/provider/result-anchor fields without creating a second store. |
| 14. Actual starter Tether Sets | DONE | `examples/tether-sets/` contains three real ordinary `.tether` examples and a runnable README. |
| 15. Starter set beyond ALLOW/ASK/DENY | DONE | The curated examples cover typed work, Together workflow, and result/follow-on semantics using the established syntax. |
| 16a. Workspace/filesystem/text/patch | DONE | `reference-plugs/tethers-agent-workspace` manifests and provider are present and previously verified. |
| 16b. Git | DONE | `reference-plugs/tethers-agent-coding` Git manifests/provider and focused tests are present. |
| 16c. Process/named verification | DONE | Coding Plug exposes argv-only `process.execute` and configured `verification.run`. |
| 16d. Structured data | DEFERRED-WITH-REASON | Final manifest/provider re-check found no existing bounded operation to expose. A typed JSON/data operation would require a new contract and validation surface; current text/hash surfaces remain the safe 0.5 boundary. |
| 16e. Hashes/integrity | DONE | Workspace Plug exposes SHA-256 and directory-manifest verification. |
| 16f. Archives | DEFERRED-WITH-REASON | No archive dependency or provider exists. Adding extraction/creation plus traversal-proof tests is a new package/provider surface, not required to close the current release asset path. |
| 16g. Bounded HTTP/network | DEFERRED-WITH-REASON | No network authority/scheme-host policy exists. Adding it would expand authority and retry/redirect semantics; the packet permits deferral where that boundary is genuine. |
| 16h. SQLite | DEFERRED-WITH-REASON | No database dependency or explicit database-root contract exists. Adding it would be a new storage subsystem; read-only Trail inspection does not require SQLite. |
| 16i. Read-only system orientation | DEFERRED-WITH-REASON | Final manifest/provider re-check found no existing orientation operation. A safe capability would need explicit secret exclusion and allow-list semantics, so it is not a trivial provider/manifests exposure. |
| 17. Preserve workspace/coding security | DONE | Existing focused pack/inspect/conform/provider tests and reviewed scopes cover the current providers. |
| 18. New slices use Plug boundary | DONE for retained scope | Existing providers use the normal trusted manifest/Plug path; deferred slices introduce no code. |
| 19. Negative safety tests for new slices | DEFERRED-WITH-REASON | Applies only if archive/network/SQLite/system slices are implemented; none are being introduced by this audit unless evidence changes the decision. |
| 20. No shell/credential/second policy engine | DONE | Existing provider contracts are argv-only/scope-bound; no new authority engine is present. |
| 21. Plug portability | PARTIAL | Windows Plug conformance remains Windows-only by contract; the release workflow now keeps that proof on Windows and runs the platform-neutral native/pack/author gates on Linux. Final status depends on the fresh tagged run. |
| 22. Version coherence | DONE | README/release docs now explain host 0.5, language 0.1, Plug 0.1.0, portable workbench 0.2.2, and the JSON version/doctor evidence boundary. |
| 23. Install/update/removal | DONE | Release docs now cover checksum verification, extraction, replacement/removal, and bundle manuals/entrypoints. |
| 24. Clean Windows/Linux bundle smoke | PARTIAL | Windows package/extraction smoke passed locally; the prior Linux failure was repaired at the workflow test-selection boundary. Fresh tagged Linux package evidence remains pending. |
| 25. Final release hashes/workflow evidence | PARTIAL | Implementation evidence and Windows hash are recorded; the fresh tagged workflow and final hosted hashes remain pending. |
| 26. Remote tag/release assets | MISSING | No new tag/release has been published yet; publish only after the repaired workflow passes. |
| 27. Front-door documentation agreement | DONE | README, QUICKSTART, agent quickstart, release docs, benchmarker manual, Plug guidance, and examples now agree on the retained 0.5 surfaces. |
| 28. Historical docs unchanged | DONE | Audit will add current-truth evidence and will not rewrite historical worker notes/roadmaps. |
| 29. Full regression/release gates | DONE locally; hosted pending | Fresh OCaml/Rust/focused gates, J14A, benchmark parity, package smoke, and `git diff --check` passed; the repaired tagged workflow is the remaining platform gate. |
| 30. Clean published closeout | PARTIAL | Implementation checkpoint and closeout evidence are present; worker note, packet terminal state, tag, hosted assets, hashes, and remote-equal proof remain to be completed. |

## Pre-implementation decision

The safe release-sized implementation slice is:

1. promote the existing Rocket benchmark machinery as one first-class
   `tethers-bench` surface with stable JSON, human output, comparisons, and
   parity guards;
2. add the smallest public preview/receipt ergonomics only where existing
   host/Core data already supports them;
3. add curated starter Tether Set examples using existing syntax and protocol
   fixtures;
4. complete current documentation, cold-agent evidence, packaging evidence,
   and publication closure;
5. leave structured-data, archive, HTTP, SQLite, and system-orientation
   providers explicitly deferred unless inspection proves an existing safe
   contract that can be exposed without a new subsystem.

No frozen semantic or authority redesign is authorised by this matrix.
