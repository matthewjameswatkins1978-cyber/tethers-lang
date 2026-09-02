# Tethers 0.5 final-audit evidence matrix

Audit baseline: `31d5e39a1e3505880e9a98cd8c650b3cf112b16d`  
Audit branch: `release/tethers-v0.5-final-audit`  
Audit state: pre-implementation

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
| 5. First-class Benchmarker surface | MISSING | Existing `scripts/benchmark-tethers.ps1`, `tethers_benchmark_core.ml`, and evidence are harnesses, not a named shipped front door. Promote one existing Rocket-aware benchmark path; do not create a parallel framework. |
| 6. Benchmarker JSON/human/compare/context output | MISSING | Existing benchmark output is not a stable before/after agent contract. Add the smallest machine-readable wrapper around the existing Rocket machinery. |
| 7. Benchmark data excluded from semantics | DONE | Current Rocket portfolio derives identity from frozen V2 payload; benchmark evidence is documentation/runtime measurement only. Add an explicit regression assertion in the promoted tool. |
| 8. Cold agent performs real bounded work | PARTIAL | `docs/evidence/tethers-0.5-cold-agent-transcript.md` proves discovery, inspection, and conformance but no actual harmless provider execution. Extend the transcript with the existing public fixture journey. |
| 9. Cold agent needs no hidden shortcut | PARTIAL | Discovery is public and documented, but the execution step is not yet evidenced. Reuse public capability/Plug/run surfaces and record exact responses. |
| 10. Public side-effect-free plan/preview | MISSING | `docs/AGENT_QUICKSTART.md` explicitly says public `plan` is not available; CLI has no public plan/preview command. Implement a read-only preview over existing validation/planning boundary only if it can avoid authority/provider execution. |
| 11. Preview distinctions and no execution | MISSING | No preview envelope exists. Define explicit parsed/validated/proposed/unavailable/authority fields and test that provider/trail mutation is absent. |
| 12. Practical Trail query/receipt | PARTIAL | `trail_command.rs` filters a JSONL Trail by execution ID and preserves entries, but offers no bounded causal receipt/query projection. Add a read-only receipt view over the existing entries if the current causal fields support it. |
| 13. Receipt causal story | PARTIAL | Existing Trail records contain causal fields, but an AI must inspect raw entries. Add a stable projection without creating a second store. |
| 14. Actual starter Tether Sets | MISSING | Only prose and `tethers-0.1/examples/record-completed-task.tether` were found; no starter collection artifact exists. Add a small established-semantics example set. |
| 15. Starter set beyond ALLOW/ASK/DENY | PARTIAL | Core/protocol fixtures already cover Together and result-related semantics, but no curated starter set presents them together. Add typed, Together, and result/follow-on examples with a README. |
| 16a. Workspace/filesystem/text/patch | DONE | `reference-plugs/tethers-agent-workspace` manifests and provider are present and previously verified. |
| 16b. Git | DONE | `reference-plugs/tethers-agent-coding` Git manifests/provider and focused tests are present. |
| 16c. Process/named verification | DONE | Coding Plug exposes argv-only `process.execute` and configured `verification.run`. |
| 16d. Structured data | DEFERRED-WITH-REASON | No bounded manifest/provider exists. A general JSON manipulation capability would be a new operation contract; existing text/hash surfaces are safer for 0.5 and no current release blocker requires it. |
| 16e. Hashes/integrity | DONE | Workspace Plug exposes SHA-256 and directory-manifest verification. |
| 16f. Archives | DEFERRED-WITH-REASON | No archive dependency or provider exists. Adding extraction/creation plus traversal-proof tests is a new package/provider surface, not required to close the current release asset path. |
| 16g. Bounded HTTP/network | DEFERRED-WITH-REASON | No network authority/scheme-host policy exists. Adding it would expand authority and retry/redirect semantics; the packet permits deferral where that boundary is genuine. |
| 16h. SQLite | DEFERRED-WITH-REASON | No database dependency or explicit database-root contract exists. Adding it would be a new storage subsystem; read-only Trail inspection does not require SQLite. |
| 16i. Read-only system orientation | DEFERRED-WITH-REASON | No safe orientation capability is shipped. Adding one is optional and must define secret exclusion and tool allow-list semantics; it is not needed for the release's existing capability journey. |
| 17. Preserve workspace/coding security | DONE | Existing focused pack/inspect/conform/provider tests and reviewed scopes cover the current providers. |
| 18. New slices use Plug boundary | DONE for retained scope | Existing providers use the normal trusted manifest/Plug path; deferred slices introduce no code. |
| 19. Negative safety tests for new slices | DEFERRED-WITH-REASON | Applies only if archive/network/SQLite/system slices are implemented; none are being introduced by this audit unless evidence changes the decision. |
| 20. No shell/credential/second policy engine | DONE | Existing provider contracts are argv-only/scope-bound; no new authority engine is present. |
| 21. Plug portability | PARTIAL | Provider code is Rust, but both Agent Essentials manifests declare Windows/x64 and pack scripts are Windows-oriented. Add only a reproducible portability declaration/path if actual CI can prove it; otherwise record the exact platform boundary. |
| 22. Version coherence | PARTIAL | README has a version table, but the host `--version`, portable `version`, product 0.5, language 0.1, and Plug 0.1.0 story need one front-door explanation and JSON evidence. |
| 23. Install/update/removal | PARTIAL | Release docs cover download/checksum/extraction and `version`/`doctor`, but update/replace/removal and bundle-relative documentation discovery are incomplete. Complete the instructions. |
| 24. Clean Windows/Linux bundle smoke | PARTIAL | Windows package succeeded in the last hosted run; Linux package/test did not. Run a fresh local Windows smoke and repair/prove Linux CI without claiming local Linux. |
| 25. Final release hashes/workflow evidence | MISSING | `docs/TETHERS_0_5_RELEASE.md` still describes future completion; all existing `tethers-v0.5.*` workflows failed and no hosted release is listed. |
| 26. Remote tag/release assets | MISSING | Local tags exist, but hosted workflow runs `33634162259` through `33637303122` failed and no GitHub release is present. Publish only after a passing tagged run. |
| 27. Front-door documentation agreement | PARTIAL | README/QUICKSTART/release docs broadly agree on current surfaces, but they omit the final preview/receipt/benchmark/starter-set closure story. Update living docs together. |
| 28. Historical docs unchanged | DONE | Audit will add current-truth evidence and will not rewrite historical worker notes/roadmaps. |
| 29. Full regression/release gates | PARTIAL | Local OCaml/Rust/focused gates were previously green, but this fresh branch has not rerun them and hosted release has a known Linux failure. Rerun changed-surface and release gates. |
| 30. Clean published closeout | MISSING | No final-audit worker note, implementation checkpoint, passing tagged workflow, hosted assets, or clean remote-equal closeout exists yet. |

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
