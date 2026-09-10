# Tethers Agent Essentials — Phase A Worker Note

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `Lucy`

Base commit: `5cce71f8f93be26a0dfd1a0e50935f9419a5c284`

Branch: `feature/agent-essentials`

Worktree: `C:\Users\Matmus\Documents\Codex\2026-08-31\tethers-agent-essentials`

## Implemented

* Added the read-only `describe` discovery command.
* Added `capability list` with deterministic sorting and filters.
* Added `capability inspect` with exact-version selection and trusted
  manifest/provider/Plug/scope/conformance projection.
* Added `plug show` from installed state without requiring the source package.
* Added an additive `tethers` binary alias for the existing reference host.
* Added strict CLI and discovery safety tests.

Discovery re-verifies installed manifest identity and digest against lifecycle
evidence, rejects incomplete or inconsistent stores, distinguishes disabled
from enabled bindings, and does not contact providers.

## Verification

* `cargo fmt --all`: PASS.
* `cargo check --all-targets --all-features --locked`: PASS.
* CLI discovery parsing test: PASS.
* Deterministic, secret-free unconfigured describe test: PASS.
* Relative host-data-root rejection test: PASS.
* Manual `target/debug/tethers.exe describe --json`: PASS.
* Manual empty-store `capability list --json`: PASS.
* `git diff --check`: PASS.

The earlier baseline remains recorded separately: the host full suite had
1509 passed and 41 pre-existing/setup-sensitive failures in the fresh
worktree; the portable suite and OCaml Dune suite passed. No frozen fixture was
changed to hide those failures.
