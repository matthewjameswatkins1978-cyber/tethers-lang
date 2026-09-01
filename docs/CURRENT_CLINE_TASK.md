# Tethers Agent Essentials — Phase B Workspace Essentials

Control contract: `1`

Status: `IN_PROGRESS`

Task colour: `Red`

Owner: `Lucy`

Route: `Codex direct implementation in dedicated review worktree; do not merge to main`

Base commit: `393fd5f06bf61b36bf270357a7f82e69f212ec26`

Implementation checkpoint: `WORKTREE`

Worker note: `docs/worker-notes/2026-09-01-agent-essentials-phase-b.md`

Updated: 2026-09-01

## Objective

Extend the existing host-owned File Tools provider seam into a bounded
workspace essentials surface. Add trusted, self-describing capabilities for
safe filesystem inspection, text search/replacement, explicit patching, and
SHA-256 integrity checks. This is Phase B only; preserve the existing Core,
Plug lifecycle, provider trust, policy, scope, Trail, replay, and Together
semantics.

## Relevant background and existing behaviour

The repository already contains the M4 `file_tools` provider, host-owned
operational scope validation, supervised MCP execution, deterministic package
inspection, and the exact `tethers.cli/1` envelope. Existing `file.metadata`,
`file.metadata` v2, and `file.move` behaviour is frozen and must remain
compatible. New capabilities must use the same trust/conformance/install/
enablement route and must not become direct host shortcuts.

## Required behaviour

1. Add bounded filesystem capabilities for read, list, and stat, with canonical
   scope containment, safe reparse/symlink handling, deterministic directory
   ordering, output limits, typed errors, and no implicit globbing.
2. Add text capabilities for literal/explicit-regex search, bounded range read,
   exact replacement with an expected match count, and deterministic compare.
3. Add an explicit unified-diff patch capability that rejects traversal,
   malformed or unrelated hunks, stale base content, and fuzzy reinterpretation.
4. Add hash capabilities for bounded file/string SHA-256 and deterministic
   directory manifests containing relative path, type, and digest.
5. Add complete trusted manifests, provider bindings, strict input/output
   schemas, conservative effects/scopes/reversibility/idempotency/confirmation/
   retry metadata, and deterministic package material for the new surface.
6. Attack the new operations with adversarial tests for traversal, symlink
   escape, overwrite ambiguity, invalid UTF-8, multiple matches, stale input,
   malformed patches, oversized output, and nondeterministic ordering.

## Relevant components

* `tethers-0.1/host-rust/src/file_tools.rs`
* `tethers-0.1/host-rust/src/bin/file_tools_provider.rs`
* `tethers-0.1/host-rust/src/host_execution.rs`
* `tethers-0.1/host-rust/src/package.rs`
* `tethers-0.1/host-rust/src/manifest.rs`
* `tethers-0.1/host-rust/tests/m4_file_tools.rs`
* `tethers-0.1/protocol/capability-manifests/`

## Frozen decisions and invariants

* Do not redesign language, Core, planner, policy, provider trust, Plug
  lifecycle, scope evidence, Trail, replay, or Together concurrency.
* Extend the existing File Tools seam; do not create a second provider
  framework, registry, policy engine, scheduler, daemon, or hidden authority.
* Every path operation is host-scope-bound at the execution boundary and fails
  closed on escape, reparse/symlink ambiguity, or unavailable paths.
* No silent overwrite, fuzzy patching, hidden globbing, shell interpolation,
  arbitrary command execution, or ambient credentials.
* Preserve all existing M4 operations, package identities, schemas, tests, and
  compatibility contracts.
* New package material must be additive and must not rewrite frozen release
  artifacts or their hashes.

## Acceptance criteria

1. New workspace operations execute only through the existing trusted Plug
   install/enable/provider path and return typed structured results.
2. All new manifests are complete enough for `capability inspect` alone to
   explain invocation, effects, scope, limits, and provider binding.
3. Path, byte, match-count, patch, hash, and ordering adversarial tests pass.
4. Existing M4, host, OCaml, and portable compatibility tests remain at their
   baseline status; no frozen fixture is weakened or rewritten.
5. New package/manifest material is deterministic and `git diff --check`
   passes.
6. The active packet and worker note record exact test commands, results,
   branch, and final commit evidence.

## Required verification

* `cargo fmt --all -- --check`
* `cargo check --all-targets --all-features --locked`
* focused Phase B provider and adversarial tests
* existing M4 file-tools tests
* `git diff --check`
* deterministic manifest/package smoke check

## Forbidden changes

* No Git/process/network/archive/SQLite/system Plug implementation in this
  Phase B packet.
* No plan/Trail query redesign, automatic trust, automatic enablement, broad
  default scope, shell escape hatch, or secret access.
* No merge to `main`, force-push, reset, rebase, or unrelated refactor.

## Stop conditions

Stop and report `BLOCKED` if a requested operation requires weakening the
existing scope/trust boundary, if safe patch semantics cannot be bounded, if
cross-platform behaviour cannot be stated precisely, or if unrelated changes
become necessary.

## Expected pre-existing changes

The baseline host full suite had 1509 passed and 41 failures in the fresh
worktree, including missing engine-fixture setup failures and concurrency
stress failures. The portable and OCaml suites passed. Preserve that evidence
and do not classify those baseline failures as Phase B regressions without a
new comparison.
