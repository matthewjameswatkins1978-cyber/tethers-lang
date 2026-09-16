# Tethers R0 Preservation Report

Status: `CURRENT AUDIT`

Audit date: 2026-09-16

This report records the preservation state observed from the fresh R0 worktree.
It replaces neither Git history nor the evidence in older audit packets. The
older packet reported 40 worktrees, 10 dirty worktrees and 21 unreachable
commits; the fetched repository currently exposes 13 worktrees, 1 dirty
worktree and 8 unreachable commits. R0 records the current facts and preserves
all eight currently unreachable commits.

## Remote and baseline

| Item | Evidence |
| --- | --- |
| Repository | `matthewjameswatkins1978-cyber/tethers-lang` |
| Current `origin/main` | `4493e563a81b7c193c0c815db4ac74b81d1c6e59` |
| R0 starting `HEAD` | `4493e563a81b7c193c0c815db4ac74b81d1c6e59` |
| Pre-P0 baseline recorded by P0 | `ddc5d0fccfc0a38ff0af3013c5e9ff6ba357e0b2` |
| P0 accepted merge | `66e8360a7247b60ccc8fb6cb447f796b922afb42` |
| Accepted main line | merged through PR #38 |

## Preservation refs

Each current unreachable commit is protected by a ref under
`refs/archive/tethers-r0/unreachable/`. The ref target is the complete commit
SHA; no commit content was rewritten.

| Commit | Subject | Classification |
| --- | --- | --- |
| `13057dedb97ce125f74b277085d592bb3f726f1a` | `feat: add 0.7 execution boundary agent core` | unique historical implementation candidate |
| `272cfc6d2461387e8c2ef4f36b669f2277945087` | `fix: correct script hash identity, powershell shape, canonical scope, descendants, and bypass closure` | unique historical repair |
| `5b38874f9223efe435f5ee50a1479bf1a355be79` | `docs: draft 0.2.0 release notes` | historical documentation |
| `76c853d2b1e04dc73abb44b3ecb263bbbba5bb53` | `WIP on opencode/j23c2-pdf-conformance` | dirty-worktree recovery commit |
| `fa4b3225afa083368577a4d9e33bbae9ca2f1bea` | `index on opencode/j23c2-pdf-conformance` | dirty-worktree recovery index |
| `d7e54243ce94ba4555064187b68259ac8bc2a429` | `docs: align current state for 0.2.0 sign-off` | historical documentation |
| `87e66b5adaf197cc2782ece554ed5e6496ecdf72` | `docs: close Tethers 1.0 preflight` | historical closeout documentation |
| `436dca377466818f57d6e4e66999a31b80a6633b` | `fix: enforce contract integrity, windows workbench, powershell policy, supervised usability, and substitute deferral` | historical implementation repair |

The refs are verification targets, not continuation branches. Future work starts
from accepted `origin/main`.

## Worktrees

Current Git worktree count is 13. Twelve are clean; one is dirty and remains
untouched. The complete current inventory is:

| Class | Path | Branch or state | HEAD | Status |
| --- | --- | --- | --- | --- |
| ACTIVE / occupied | `D:\The Next Thing\Tethers Lang - J16 Clean` | `codex/tethers-resolve-p3a-protocol-provenance` | `71c1fa9` | clean; occupied historical task checkout |
| HISTORICAL-CLEAN | `D:\HackathonDev\TethersAuthorityBridge` | `codex/lantern-authority-bridge` | `f100f5a` | clean |
| HISTORICAL-DIRTY / UNIQUE-UNPUBLISHED | `D:\The Next Thing\Tethers Lang - 0.7 Execution Boundary` | `feature/tethers-0.7-execution-boundary` | `dd66e85` | dirty; eight untracked release files |
| HISTORICAL-CLEAN | `D:\The Next Thing\Tethers Lang - 0.7 Integration` | `release/tethers-v0.7.0` | `0fe324a` | clean |
| HISTORICAL-CLEAN | `D:\The Next Thing\Tethers Lang - 0.7 Main Release` | detached | `51ef9fd` | clean |
| HISTORICAL-CLEAN | `D:\The Next Thing\Tethers Lang - Environment Handshake` | `codex/execution-environment-handshake` | `9d71d75` | clean |
| HISTORICAL-CLEAN | `D:\The Next Thing\Tethers Lang - J20 Operational Handshake` | `opencode/j20-operational-handshake-gateway` | `f987326` | clean |
| HISTORICAL-CLEAN | `D:\The Next Thing\Tethers Lang - J21 M6A PDF Provider` | `feature/0.3-p3-pdf-reference-plug` | `e23030a` | clean |
| ACTIVE / current R0 | `D:\The Next Thing\Tethers Lang - R0 Host Architecture Recovery` | `codex/tethers-r0-host-architecture-recovery` | `4493e56` | clean at audit start; R0 changes are isolated here |
| UNIQUE-UNPUBLISHED / HISTORICAL-CLEAN | `D:\The Next Thing\Tethers Lang - R3 Linux Native Parity` | `codex/tethers-0.8-linux-native-parity` | `ebed490` | clean; open PR #25 |
| HISTORICAL-CLEAN | `D:\The Next Thing\Tethers Lang - Resolve P0 Contract` | `codex/tethers-resolve-p0-contract` | `5e04595` | clean |
| HISTORICAL-CLEAN | `D:\The Next Thing\Tethers Lang - Resolve P3 Live Transport` | `codex/tethers-resolve-p3-live-transport-pre-p4a` | `dc19bcd` | clean |
| UNIQUE-UNPUBLISHED / HISTORICAL-CLEAN | `D:\The Next Thing\Tethers Lang - Trident P5` | `codex/trident-p5-real-workload` | `4493e56` | clean; retained for prior task continuity |

R0 changed no worktree outside the current R0 worktree. The dirty 0.7
worktree’s eight untracked release artifacts may be valuable evidence and are
not inspected by or imported into R0. No worktree was switched, reset, stashed,
cleaned or removed.

The dirty release artifacts may be valuable evidence and are not inspected by
or imported into R0. No worktree was switched, reset, stashed, cleaned or
removed.

## Branches and pull requests

No branch was deleted. No tag was deleted. Open PR #25 (Linux native parity)
and PR #29 (context-local AGENTS routing) remain open and held. Neither was
merged, rebased or modified by R0.

## Do-not-delete list

- every occupied worktree listed by `git worktree list --porcelain`;
- the dirty 0.7 execution-boundary worktree and its untracked release files;
- all eight archive-ref commits above;
- PR #25 and PR #29;
- all P0-P4b commits, files and worker notes;
- all existing branches and tags.

## R0 conclusion

The current repository is recoverable without destructive cleanup. The historical
counts are retained as historical claims, while the current count and exact
preservation refs above are the R0 evidence. Any later pruning or retirement
requires a separate explicit cleanup decision after checking these refs and the
worktree owners again.
