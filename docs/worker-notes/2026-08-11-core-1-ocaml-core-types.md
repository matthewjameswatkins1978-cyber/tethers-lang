Task: `TETHERS CORE-1 — OCaml Core Type Foundation`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `8bd975ae55c359ae09e30cfca3c905fdace0a01f`

Implementation checkpoint: `d03f8327a753c0b1f2380069b056db9e7cec7da7`

## Requested outcome

Introduce the first production representation of Tethers Core as a standalone OCaml semantic type module. The module defines nominal types only — no lowering, validation, serialization, or planning. It must compile alongside existing modules while leaving all existing behaviour byte-for-byte unchanged.

## Changes made

- `tethers-0.1/engine-ocaml/bin/tethers_core.mli` (new): Public interface exposing all Core types with private constructors on nominal IDs, semantic value types, and conversion functions.
- `tethers-0.1/engine-ocaml/bin/tethers_core.ml` (new): Implementation of all types and constructor/deconstructor functions.
- `tethers-0.1/engine-ocaml/bin/dune` (modified): Added `tethers_core` to both executable module lists (`main` and `tethers_mcp_main`).
- `docs/CURRENT_CLINE_TASK.md` (modified): Replaced C1C packet with CORE-1 packet.

No existing OCaml modules, Rust source, fixtures, dependencies, or toolchain configuration were modified.

## Decisions and assumptions

- Nominal ID types use the `private` OCaml keyword in the `.mli` to prevent accidental construction while allowing pattern matching. Explicit `of_string`/`to_string` conversion functions are provided for each type.
- `CapabilityContractDigest` and `CoreVersion` are modelled as distinct private types (not plain strings), enforcing the packet requirement that they not be confused with other string-carrying fields.
- `batch_collection_provenance`, `batch_traversal_policy`, `batch_objective`, and `role_fulfillment` use placeholder private string wrappers; their exact semantics belong to later packets.
- `fact_availability` is defined as a public variant (`Optional | Guaranteed`). The packet specifies that an unlisted Fact under an Outcome means unavailable, so a separate `Unavailable` constructor is intentionally omitted.
- Anchor Origin was given a minimal record (`anchor_origin_id`, `event_name`, `declared_facts`) sufficient for the species; its detailed schema belongs to the lowering packet.
- Literal values in input bindings use `string` as a placeholder; the full Core value representation belongs to a later packet.
- Execution Constraint `Deadline` carries a `string` representing the semantic duration/bound without implementing any timer logic.
- Branch `outcome_branches` uses an association list of `(terminal_outcome * branch_target)` pairs. `branch_target` distinguishes `Continue_to of origin_id` from `Stop`.
- `branch_subject` is typed as `origin_id`.

## Evidence

All tests and checks were run against the committed implementation checkpoint `d03f8327a753c0b1f2380069b056db9e7cec7da7`.

1. Packet checker at start: `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` — `PASS control-v1/IN_PROGRESS`

2. Rust formatter (read-only): `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check` — PASS (no output)

3. OCaml build: `opam exec --switch="D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" -- dune build` — PASS

4. Fixture suite: `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\check-fixtures.ps1` — PASS (64 JSON files, 32 JSONL files)

5. Engine suite: `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-engine.ps1` — PASS (32 cases + 4 line-ending validation + 2 determinism repeats)

6. MCP transcript suite: `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-mcp-transcripts.ps1` — PASS (16 cases)

7. Whitespace: `git diff --check` — PASS (no issues, only expected LF→CRLF advisory)

8. Complete diff inspection: `git diff --stat` shows only `docs/CURRENT_CLINE_TASK.md` modified (107 insertions, 291 deletions — the packet rewrite). No Rust files changed. `git diff --stat` of the implementation commit shows 3 files changed (dune, tethers_core.ml, tethers_core.mli), 436 insertions, 2 deletions.

9. Nominal ID type safety was verified by design: each ID is a distinct OCaml type with a private constructor. The types `origin_id`, `role_id`, `fact_id`, `batch_id`, etc. cannot be interchanged — doing so produces a compile-time type error.

10. Branch and Role are independent record types, not variants of `origin_site`. The `origin_site` closed variant has exactly four constructors: `Anchor_origin`, `Action_origin`, `Together_origin`, `Batch_site`.

11. `item_template` has three structurally distinct lists: `origin_sites : origin_site list`, `branches : branch list`, `roles : role list`. They are not collapsed.

## Publication evidence

Branch `feature/core-1-ocaml-core-types` was pushed normally to `origin`. Remote publication resolved below in the completion report. The local `HEAD` equals the pushed remote `HEAD` and final Git status is clean.

## Discoveries

- The engine test script (`test-engine.ps1`) builds via `opam exec -- dune build` without specifying a switch. In worktrees that don't own the `_opam` directory, `$env:OPAMSWITCH` must be set before invoking the script.
- `cargo fmt` requires `--manifest-path tethers-0.1/host-rust/Cargo.toml` since the workspace root has no `Cargo.toml`.

## Remaining risks

None known within packet scope. The module is dormant and introduces no behavioural change. The `batch_collection_provenance`, `batch_traversal_policy`, `batch_objective`, and `role_fulfillment` types use placeholder representations; later packets that flesh out these types will need to verify that the existing nominal type boundaries remain valid.

## Smallest next action

CORE-2: lower Human Tether AST into Core types, or whatever Lucy compiles as the next bounded packet.

## References

- Implementation: `tethers-0.1/engine-ocaml/bin/tethers_core.ml`, `tethers_core.mli`
- Build: `tethers-0.1/engine-ocaml/bin/dune`
- Packet: `docs/CURRENT_CLINE_TASK.md`
- Branch: `feature/core-1-ocaml-core-types`
- OCaml switch: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`
