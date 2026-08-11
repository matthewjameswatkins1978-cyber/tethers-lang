# Worker Note — TETHERS CORE-2A

Task: `TETHERS CORE-2A — Ambiguous Environment Fail-Closed Correction`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `ca7d013effef4bf1e697141651301561f573435c`

Implementation checkpoint: `47cb5469d758cd0d2c4239a95f3c7ebe02de26bb`

## Requested outcome

Correct two CORE-2 lowering ambiguity gaps without architecture or runtime
changes: duplicate Human Fact bindings must produce an explicit ambiguity
error rather than `Unknown_fact`, and two used capability bindings sharing one
`CapabilityId` with conflicting digests must never be silently deduplicated.

## Changes made

- `tethers-0.1/engine-ocaml/bin/tethers_core_lowerer.ml` — added
  `Duplicate_fact of string` and `Conflicting_capability_contract of
  capability_id` to `lowering_error`; changed `resolve_fact` 2+ match branch
  from `Unknown_fact` to `Duplicate_fact`; changed `capability_contracts`
  construction to a `result`-returning dedup that collapses only identical
  `(capability_id, contract_digest)` pairs and errors on conflicting digests.
- `tethers-0.1/engine-ocaml/bin/tethers_core_lowerer.mli` — mirrored the two
  new `lowering_error` constructors.
- `tethers-0.1/engine-ocaml/bin/tethers_core_lowerer_test.ml` — added four
  tests: duplicate Fact, same contract through two source names, conflicting
  contract rejection, and unused-conflict non-poisoning.
- `docs/CURRENT_CLINE_TASK.md` — replaced the CORE-2 packet with the CORE-2A
  packet; set to `COMPLETE` at closeout.
- `docs/worker-notes/2026-08-11-core-2a-ambiguous-environment-fail-closed.md`
  — this note.

No dune change was required (test executable already registered). No Core type
changes. No Rust changes. No evaluator/protocol/outcome changes.

## Decisions and assumptions

- `Duplicate_fact` carries the offending `source_name` string, matching the
  existing `Unknown_fact` shape, so absence and ambiguity are distinct but
  consistently diagnostic.
- `Conflicting_capability_contract` carries the `capability_id` (nominal Core
  type), the identity whose semantic pin is contradicted.
- The contract dedup now returns `result`: a repeated `(capability_id,
  contract_digest)` pair collapses into one entry; a repeated `capability_id`
  with a different digest produces `Conflicting_capability_contract` before
  `Ok program` is constructed. The first-seen digest is never silently
  preferred.
- Used-subset validation is preserved: the dedup runs only over contracts for
  capabilities actually referenced by the Tether, so unused conflicting
  environment bindings do not poison lowering. This is deterministic and
  documented in the test.
- Action Origin contract references are unchanged: each Action still carries
  the digest resolved from its own source-name binding; the contract table is
  the consistency check.

## Evidence

- Packet checker: `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` — PASS `control-v1/COMPLETE` (base `ca7d013`, HEAD `47cb546`).
- OCaml build: `opam exec --switch="D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" -- dune build` — PASS (exit 0).
- Lowerer tests: `opam exec --switch="D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" -- dune runtest` — PASS `all lowerer tests (49/49)`.
- Fixture suite: `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\check-fixtures.ps1` — PASS (64 JSON files, 32 JSONL files).
- Engine suite: `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-engine.ps1` — PASS (32 fixture cases plus determinism and line-ending checks).
- MCP transcript suite: `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-mcp-transcripts.ps1` — PASS (16 cases).
- Whitespace: `git diff --check` — PASS (no whitespace errors; only informational LF-to-CRLF working-copy warnings).
- Rust formatter (RUST_UNCHANGED): `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check` — PASS (exit 0).
- Diff inspection: only the three authorised files changed (`tethers_core_lowerer.ml`, `.mli`, test) plus closeout docs; zero Rust/dependency changes.
- Working tree implementation files match checkpoint `47cb5469d758cd0d2c4239a95f3c7ebe02de26bb` exactly (empty `git diff`).

## Publication evidence

Branch pushed: `origin/feature/core-2-human-to-core-lowering` (continued from
CORE-2). Full remote HEAD SHA resolved after the normal push:
`ae8d1224909fc930b761af7511fa602149150ab2`. Local `HEAD == remote HEAD`:
confirmed. Final `git status --short --branch`: clean.

## Discoveries

- The existing test executable already covers the lowerer module, so no dune
  stanza change was needed for CORE-2A.

## Remaining risks

- None known within packet scope. Deferred: canonical capability identity and
  full environment validation belong to later packets; CORE-2A deliberately
  validates only the used semantic subset.

## Smallest next action

CORE-3: initial Core validation (success-edge completeness, unique Origin IDs,
contract-table/Action-Origin coherence) or the dedicated Together lowering
packet.

## References

- `tethers-0.1/engine-ocaml/bin/tethers_core_lowerer.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_lowerer.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_lowerer_test.ml`
- `docs/CURRENT_CLINE_TASK.md`
- Implementation checkpoint commit `47cb5469d758cd0d2c4239a95f3c7ebe02de26bb`
- Base commit `ca7d013effef4bf1e697141651301561f573435c` (CORE-2 closeout)
