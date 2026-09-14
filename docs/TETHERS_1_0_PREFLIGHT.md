# Tethers 1.0 Preflight

Date: 2026-09-14  
Scope: repository stabilisation and baseline evidence before Tethers 1.0 feature work  
Closeout verdict: **ACCEPTED TECHNICAL BASELINE; OWNER DECISION REMAINS**

This is a preflight record, not a 1.0 feature design or release approval. It
records what is verified, what was repaired, what remains unverified, and the
owner decisions required before the next feature packet.

## 1. Baseline and authority

The audit was performed in an isolated worktree so the occupied J16 checkout
and historical worktrees were not disturbed.

| Item | Verified value |
| --- | --- |
| Audit root | `D:\The Next Thing\Tethers Lang - 1.0 Preflight` |
| Audit branch | `codex/tethers-1.0-preflight` |
| Audit HEAD | `699f9c8f48359b8b3de0d3614b8827acecf977cf` |
| `origin/main` at audit | `699f9c8f48359b8b3de0d3614b8827acecf977cf` |
| Audit divergence | `0 ahead / 0 behind` |
| Canonical occupied checkout | `D:\The Next Thing\Tethers Lang - J16 Clean` |
| Occupied checkout branch | `opencode/j19-m5-durable-local-anchor` |
| Occupied checkout HEAD | `777026be2945895c86e36ce997ba8e15d4f8b0f6` |
| Canonical remote | `https://github.com/matthewjameswatkins1978-cyber/tethers-lang.git` |

The occupied J16 checkout is clean but is not current `main`: it is an
ancestor of `origin/main` and is 818 commits behind it. Local `main` is also an
ancestor of `origin/main` and is 307 commits behind. This is a topology fact,
not permission to move or delete either checkout.

Required project instructions were read before mutation, including the control,
workflow, current-task, language-standard, worktree/line-ending, Rust, and
OCaml guides. The repository tool diagnostic passed. The current historical
packet on `origin/main` is already `COMPLETE` (Tethers 0.7 release integration,
owner Codex); no new implementation packet was invented for this audit.

## 2. Git, branches, and worktrees

`git fetch origin --prune` succeeded. There are 38 local branches and 9 remote
branch refs. All remote branches are ancestors of `origin/main`. The following
classification is the safe inventory:

| Classification | Evidence / examples | Disposition |
| --- | --- | --- |
| CURRENT | `origin/main` at `699f9c8` | Authoritative baseline for this preflight |
| RELEASE-HISTORY | `release/tethers-v0.7.0`, tag `tethers-v0.7.0` | Preserve release history |
| MERGED | Old milestone branches and remote branches proven ancestors of `origin/main` | Historical refs; cleanup can be a separate explicit housekeeping task |
| RESEARCH-HISTORY | Rocket/C-B1 and prior milestone refs | Preserve until stale benchmark residue is explicitly quarantined/retired |
| UNKNOWN / ACTIVE | `feature/tethers-0.7-execution-boundary` (`+1/-307`, occupied, untracked `release/`), `opencode/j20-operational-handshake-gateway` (`+4/-809`, occupied), `opencode/j24c-plug-disable-cli` (`+1` unique), `opencode/j24d-plug-enable-scope-file` (`+1` unique) | Preserve; no safe deletion inference |

Other occupied worktrees include the 0.7 integration/release trees,
environment-handshake, J20, J21 PDF provider work, and this audit tree. No
branch or worktree was deleted. This preserves unexplained work, unique commits,
and the untracked `release/` directory.

There are no open GitHub pull requests at the time of audit. This closeout has
not yet pushed or merged the new closeout commits. No history rewrite or tag
mutation was performed.

## 3. Version and product truth

| Surface | Current truth |
| --- | --- |
| Product/native host | `0.7.0` (`VERSION`, Rust host package, published tag) |
| Language semantics | Tethers `0.1`, defined by `tethers-0.1/SPEC.md` |
| Core protocol | Protocol `0.1`; MCP transport revision `2025-11-25` |
| Portable product | `0.2.2`, a separate portable Rust surface |
| Plug package format | `1`; socket major `1`; Rust manifest format `1.0` |
| Canonical identity | V2 IR canonicalisation in production; no V1 fallback in the live adapter |
| Rocket | Exact V3 portfolio/refinement/success-path machinery and evidence |
| Trail | Host-owned evidence; Core proposes Plans and does not execute Actions |
| MCP | Current server exposes `tethers.evaluate` and `tethers.validate`; evaluation is Core-backed and non-executing |

The six top-level/current-facing documents audited for stale 0.6 release
claims were aligned to 0.7 and to the narrower Windows x64 full-runtime
release claim. They do not claim an unverified Linux 0.7 native asset.

## 4. Published 0.7 release

The public `tethers-v0.7.0` release is verified as a non-draft, non-prerelease
release published on 2026-09-11, targeting `main` and source commit
`51ef9fd53f353c0413c4900f337071cbca6f921a`.

Downloaded assets and SHA-256 evidence matched GitHub metadata:

- `Tethers-0.7.0-windows-x64.zip`: `73dab27330cd...`, 4,202,126 bytes.
- `Tethers-0.7.0-windows-x64-manifest.json`: `aa3bfea...`, 604 bytes.
- `SHA256SUMS`: `5cc9cee...`, 165 bytes.
- Inner `tethers.exe`: `b19fdd3682c23ad2effe82f8e2c9638e4a511136184a923e917a04bdc4f49992`.
- Inner `tethers-engine.exe`: `98c3392d38a7e362e430abd50809a0ae1b6a49bd135e2b3f78cbde94909d6a32`.

The manifest source commit and archive hash match the downloaded release. The
archive contains the full Windows x64 runtime, examples, documentation, and
both executables. Linux native parity and a Linux 0.7 asset are not established.

## 5. Cold-start CLI discovery

The extracted release executable was exercised in a fresh temporary project.
The correct version form is `tethers --version`, which returned `tethers 0.7.0`;
`tethers version --json` is not a valid native command. `tethers --help` exposed
`init`, `doctor`, `describe`, capability, workspace, git, exec, threadmoth,
plug, check, run, preview, plan, and trail surfaces.

`describe --json` returned schema `tethers.cli/1`, version `0.7.0`, and
machine-readable discovery/features. Before initialisation, `doctor --json`
reported the expected not-configured/invalid-project state while confirming the
sibling engine and its SHA. After `tethers init`, `doctor --json` returned
`healthy: true`, valid configuration, writable host state, and a usable sibling
engine. Capability listing and a missing-capability inspection also returned
structured JSON and stable status codes.

## 6. Build and test evidence

The repository-owned tool and toolchain checks passed: native Windows tooling,
Rust 1.97.1/rustfmt/clippy, OCaml 5.5.0, Dune 3.24.0, Yojson 2.2.2, the exact
opam switch, rust-analyzer, nextest, cargo-deny, cargo-machete, OpenCode, and
configuration checks were available and valid.

The OCaml engine build and forced test suite passed. The reported groups were:

`lowerer 52/52`, `validator 51/51`, `Rocket model 214/214`, `plan 188/188`,
`Rocket portfolio 41/41`, `adapter 46/46`, `request adapter 89/89`, `wire`
all pass, `Rocket search 39/39`, `Rocket refine 4807/4807`, V2 reference and
production vectors pass, dense V2 corpus `5000/5000` with zero mismatches,
origin-walk `105/105`, and the former chain-11 counterexample has no repaired
exact-search mismatch.

Rust verification passed after a bounded test-isolation repair in
`tethers-0.1/host-rust/src/dispatch.rs` removed fixed temporary directory names
from two tests. The full locked all-target/all-feature suite completed with exit
0 and no failed tests. Two PDF-provider tests and three evil-bunny-provider
tests remain ignored because their optional external provider executables were
not configured. `just test-m3`, `just test-m4`, `just test-m5`, `just fmt`,
`just check`, and `just agent-tools` passed. Clippy exited 0 but emitted 86
warnings, predominantly existing complexity/style/test-portability warnings;
this is not a warning-free baseline.

`just verify` was not used as an aggregate because its packet gate is invalid
for the current completed packet after a later, already-present documentation
file was added. The packet checker itself was run and failed honestly with:
`Implementation changed after recorded evidence checkpoint` for
`docs/hackathons/CALLPERMIT_GARY_BUILD_2026-09-11.md`. A new checkpoint was not
fabricated and the historical packet was not rewritten.

## 7. Rocket scale evidence

The production success-path evidence covers action counts 10, 100, and 1000
(also 9, 11, 12, 99, and 999). The 1000-action result is exact and records
`path_size=1000`, `successor_slots=1003`, `candidate_targets=1003`,
`feasibility_checks=1003`, `rejected=3`, `committed=1000`, zero complete
permutations, and maximum partial components 1000. The refinement chain-1000
statistics record `relation_visits=6999`, `splitter_pops=1004`, `cell_splits=998`,
`max_worklist=6`, and `final_cells=1004`.

The first-class quick benchmark passed three Rocket portfolio/reference parity
cases. The new direct production-helper scale test passed `89/89` checks in
175.508 seconds, covering repeated canonicalisation of 10, 50, 100, and 1000
sequential actions. Direct counters were:

| Actions | Candidate targets | Feasibility checks | Rejected | Committed | Complete permutations | Max partial components |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10 | 19 | 19 | 9 | 10 | 0 | 10 |
| 50 | 53 | 53 | 3 | 50 | 0 | 50 |
| 100 | 101 | 101 | 1 | 100 | 0 | 100 |
| 1000 | 1003 | 1003 | 3 | 1000 | 0 | 1000 |

The same test checks repeated identical digest, Action ordering, raw-ID
invariance, and completion through `Tethers_core_rocket_v3_success_path`.
The production evaluation adapter remains V2-only; no legacy fallback is
reachable. This resolves the direct Rocket 50-action evidence gap.

The legacy C-B1 alternative benchmark was also run. Its partition checks pass,
but ordering/canonical-equivalence checks fail at sizes 100, 250, 500, and
1000, with a detailed size-10 byte/digest mismatch. Its own worker note marks
it benchmark-only and says it should be removed before production work. The
benchmark executable, benchmark-only interface, and Dune target were retired;
the shared old canonical module was retained because existing V2 reference
tests still compile against it. This is classified **RESOLVED** as obsolete
research residue, not a current production regression. Historical C-B1 design
and worker notes remain as evidence and are not acceptance claims.

## 8. MCP and Plug state

`docs/MCP_PLAN.md` is current as an architecture/proposal document: it names
JSON-RPC 2.0, stdio/HTTP, MCP revision `2025-11-25`, and a non-executing Core
adapter. It does not prove a production HTTP MCP deployment. Plug authoring and
reference manifests agree on package format 1, socket major 1, and MCP
`2025-11-25` over stdio.

The MCP transcript fixtures were stale against the Core cutover. The bounded
repair updated the current tool descriptions and the four evaluate cases to
include `core_environment`, `program_digest`, and the current structured error
envelopes. The repository MCP transcript check now passes all 16 required cases.

## 9. Security reality check

The live boundary is consistent with the project invariants: the Core planner
does not execute Actions; current manifests and provider pins are host-side;
the production adapter uses V2 canonicalisation only; actions are serially
planned/dispatched; and no automatic retry claim was found. Rust coverage
includes supervised child lifecycle, Windows Job Object handling, bounded I/O,
flush and reparse checks, canonical roots, timeouts, redaction, junction, job,
and secret-handling negative tests.

This is supervised host execution, not a hostile-code sandbox. The existing
security wording was kept at that level. No general shell capability was added
by this preflight.

## 10. Licence, release automation, and checker classifications

No tracked `LICENSE`, `LICENCE`, `COPYING`, or `NOTICE` file was found. The OCaml
package currently declares `UNLICENSED`. This requires an owner decision before
calling the repository an openly distributable 1.0 baseline. Apache-2.0 is a
candidate only; it was not selected or added by this audit.

There is no dedicated 0.7 release workflow. Existing 0.5/0.6 workflows still
describe older packaging, while the published 0.7 release uses the newer
`scripts/package-0.7.ps1` Windows full-runtime path. Linux parity for the 0.7
native host is not proven. This is a **1.0 REQUIREMENT** for a supported Linux
release, separated as follows: Rust host code is substantially portable but
Windows-specific supervision/path branches need Linux verification (**CODE
PORTABILITY**); OCaml engine code is platform-neutral in the tested path but no
Linux toolchain build was run (**BUILD PORTABILITY**); no Linux native 0.7
archive, checksum, manifest, or clean-machine smoke test exists (**PACKAGE
PORTABILITY**); and no current 0.7 native Linux matrix/release workflow exists
(**CI/RELEASE PORTABILITY**). No major port was attempted.

The exact packet-checker command
`pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` fails
before product verification with `Implementation changed after recorded
evidence checkpoint` and identifies
`docs/hackathons/CALLPERMIT_GARY_BUILD_2026-09-11.md`. The file is a later
GARY/task evidence document, not a Tethers implementation defect; the completed
historical packet was not rewritten. This is classified **EXTERNAL TOOLING
ISSUE**. The checker needs a future task/checkpoint compatibility repair, but it
does not block Core correctness or this closeout merge.

## 11. Changes made in this audit

- Aligned current README, quickstart, overview, security, goal, and 0.7 release
  documents with the published 0.7 truth.
- Repaired two fixed-path Rust tests so parallel/repeated runs do not collide.
- Updated stale MCP descriptions and evaluate transcript fixtures to the live
  Core-backed `tethers.evaluate` contract.
- Added direct Rocket 10/50/100/1000 scale and identity evidence.
- Retired the obsolete C-B1 benchmark executable and benchmark-only interface.
- Added this preflight record.

No implementation semantics, 0.1 syntax, architecture boundary, historical
worker note, current task packet, public history, branch, tag, release, or
unexplained worktree was rewritten.

The audit-created release extraction and CLI test directories remain outside
the repository because the host rejected the recursive cleanup command. They
are disposable, contain no repository state, and are explicitly listed in the
handoff rather than silently treated as removed.

## 12. Start line for Tethers 1.0

The repository has a reproducible current `origin/main` baseline with a
published and hash-verified Windows 0.7 full-runtime release, passing OCaml and
Rust suites, passing current MCP transcripts, documented version truth, and
preserved historical Git state.

There is no remaining Core correctness blocker from this preflight. The
remaining **OWNER DECISION** is the licence. Linux native support and release
automation are **1.0 REQUIREMENTS**, and the packet-checker mismatch is
**EXTERNAL TOOLING ISSUE**. Rocket scale evidence and C-B1 residue are
resolved.

Recommended small local commits on this audit branch are: (1) test isolation,
(2) documentation and MCP fixture alignment, and (3) this preflight artifact.
They are intentionally not pushed or merged by this audit.
