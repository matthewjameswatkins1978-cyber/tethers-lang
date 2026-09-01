# Rocket-Only Runtime V2 Cutover

Control contract: `1`

Status: `IN_PROGRESS`

Task colour: `Red`

Owner: `Lucy`

Route: `Direct GitHub implementation with repository CI verification`

Base commit: `b32d1940a36ddd92ac7048f87787d7ca7ff7d63a`

Worker note: `docs/worker-notes/2026-09-01-rocket-only-runtime-v2.md`

Related issue: `#5 — BUG: Rocket V2 factorial search on simple sequential Action chains`

Updated: 2026-09-01

## Objective

Finish the pre-1.0 Canonical V2 migration so the live Tethers evaluation path uses Rocket V2 as the sole producer of new semantic ProgramDigest identity. V1 may remain only as historical/test evidence and must not stamp new production runtime, Trail, or replay identity.

## Relevant background and existing behaviour

The frozen Canonical V2 migration policy says V1 is known incorrect, no new V1 identities may be produced after V2 ships, bare `sha256:<hex>` ProgramDigest rendering is legacy, and normal runtime identity must use the full `tethers:v2:sha256:<hex>` value.

Rocket V2 is integrated and independently proved against the V2 oracle and exhaustive baseline, including the 5,000-case dense differential. However, the live evaluation adapter still calls `Tethers_core_canonical.canonicalize` and the planner/wire surface is typed around the legacy V1 canonicalized value.

This task finishes that cutover. It does not change Human Tether syntax or provider execution semantics.

## Required behaviour

1. The production Human-source evaluation path MUST call Rocket V2 and MUST NOT call the V1 canonicalizer.
2. Every newly produced production ProgramDigest MUST use the exact `tethers:v2:sha256:<64 lowercase hex>` rendering.
3. Runtime planning MUST remain deterministic without depending on V1 canonicalized Core or incidental `origin_sites` storage order.
4. Rocket budget/validation failure MUST fail closed as a canonicalization error with no Plan and no digest.
5. The Rust host MUST accept and test the V2 ProgramDigest contract and MUST reject treating bare V1 `sha256:<hex>` as the current production contract.
6. V1 canonicalization may remain only in isolated legacy/reference tests or historical benchmarking; no production adapter/wire/runtime code may depend on it.

## Relevant components

- `tethers-0.1/engine-ocaml/bin/tethers_core_evaluation_adapter.ml/.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_plan.ml/.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_wire.ml/.mli`
- focused OCaml tests and `bin/dune`
- `tethers-0.1/host-rust/src/engine_stdio.rs` and focused ProgramDigest expectations
- frozen Canonical V2 spec and Rocket V2 implementation as read-only semantic authority

## Frozen decisions and invariants

- Rocket V2 is the only current semantic identity engine.
- Canonical V2 payload/digest equations and frozen vectors do not change.
- The V2 oracle and exhaustive baseline remain independent correctness witnesses.
- A Plan remains a request, not permission.
- Runtime occurrence IDs and idempotency keys remain occurrence-derived, not ProgramDigest-derived.
- Human Tether syntax and source ordering semantics do not change.
- Together group semantics and host physical concurrency do not change.
- No new V1 identity may be written for current production work.
- No silent fallback from Rocket to V1 is permitted.
- Canonicalization failure returns no best-effort digest.

## Acceptance criteria

1. A real production `tethers.evaluate` request returns a top-level ProgramDigest beginning `tethers:v2:sha256:`, and repository search proves the production evaluation adapter no longer calls `Tethers_core_canonical.canonicalize`.
2. Exact V2 prefix/length/hex tests pass in OCaml wire tests and Rust real-engine tests.
3. Planning/evaluation tests prove semantically equivalent raw-ID/storage permutations retain the same semantic Plan where the supported runtime vocabulary requires invariance.
4. A forced Rocket budget failure returns an adapter/wire error and produces no Plan or ProgramDigest.
5. Rust cross-language rehearsal tests expect V2 identity and pass against the real OCaml engine.
6. Production executable/module graphs include Rocket V2; legacy V1 modules are not required by the live adapter/wire path.
7. Frozen V2 vectors, Rocket differential tests, OCaml build/tests, Rust formatting/check/tests, and task-packet consistency pass.

## Required verification

- OCaml 5.5.0 / Dune build of all engine targets.
- `dune runtest --force`.
- Rocket V2 frozen-vector and differential suite.
- Focused Core evaluation, request, wire, and planning tests.
- Rust `cargo fmt --all -- --check`.
- Rust `cargo check --all-targets --all-features --locked`.
- Rust focused real-engine tests for CORE-9B ProgramDigest behaviour.
- Full Rust test suite if CI budget permits; otherwise report exact unrun scope.
- `git diff --check`.
- task-packet checker in COMPLETE state after the implementation checkpoint.

## Forbidden changes

- No Human Tether grammar or 0.1 operator changes.
- No policy, approval, Plug trust, provider dispatch, replay-state ordering, Trail ordering, Together semantics, or concurrency redesign.
- No new dependency.
- No fallback from Rocket V2 to V1.
- No edits to Agent Essentials implementation files.
- Do not rewrite historical worker notes merely to make them sound current.

## Follow-up tracked outside this cutover

GitHub issue #5 permanently tracks the separate sequential `Action_origin` factorial-search weakness exposed by this migration. Do not close or bury that issue merely because the Rocket-only identity cutover lands. The follow-up must prove an exact control-flow reduction, preserve Canonical V2 correctness/fail-closed behaviour, and benchmark coherent 10/100/1,000-Action Tethers.

## Stop conditions

- Rocket V2 disagrees with the frozen V2 oracle/baseline on any accepted program.
- Removing V1 from live planning requires changing Human Tether semantics.
- Runtime Plan determinism cannot be preserved without inventing a new semantic ordering rule.
- A new dependency, new unsafe Rust, or second execution boundary becomes necessary.
- Two materially similar implementation/test failures occur without a new diagnosis.

## Expected pre-existing changes

None.
