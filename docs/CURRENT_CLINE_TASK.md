# Current Implementation Task

Control contract: `1`

Task: `TETHERS CORE-2 — Human AST → Core Lowering`

Owner: `OpenCode`

Status: `COMPLETE`

Task colour: `Amber`

Route: `OpenCode implementation + evidence → Lucy independent GitHub review`

Worker note: `docs/worker-notes/2026-08-11-core-2-human-to-core-lowering.md`

Base branch: `feature/core-2-human-to-core-lowering`

Base commit: `b5daea00accff8e7617727a02ee524bfb80cd823`

Implementation checkpoint: `52032e42f8c1d44a801e79735272327c12ee004c`

OCaml switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

Rust toolchain: read exact channel from `rust-toolchain.toml`; use plain Cargo (resolved by root pin); `--locked` mandatory

Toolchain preflight: `pwsh -NoProfile -File scripts/check-dev-tools.ps1` (run; all tools present)

Rust change class: `RUST_UNCHANGED`

## Objective

Implement the first real Tethers Core lowering pass:

```text
Human Tether AST
        +
explicit lowering environment
        ↓
Tethers Core program
```

CORE-2 translates the existing sequential Tethers 0.1 subset into the dormant
Core representation introduced by CORE-1 / 1A / 1B. It does NOT replace the
existing evaluator path yet.

The lowerer translates meaning. It MUST NOT invent semantic information
unavailable from its inputs. Capability contract identities, host-input Fact
declarations, Program identity, and Core version are supplied explicitly
through a lowering environment. No fake hashes, no guessed contracts, no
hidden defaults.

## Relevant background and existing behaviour

CORE-1 established dormant nominal Core types. CORE-1A added typed literal
values, input Facts, and Fact Guards. CORE-1B added named capability inputs
and explicit success continuation flow. The existing parser (`Tether_parser`)
produces typed AST values for the supported sequential Tether 0.1 subset
including `Together`. The existing evaluator (`Tethers_evaluator`) evaluates
directly from parsed AST to protocol responses; it does not consume Core
types. The Core types exist as a dormant vocabulary with no consumer.

## Required behaviour

1. Lower a single-action Tether into a Core program with Anchor Origin,
   Action Origin, entry origin, and Program_complete continuation.
2. Lower multiple sequential Actions into an explicit A1→A2→...→complete
   success-continuation chain.
3. Preserve typed literals (string, int, bool) in Core as exact typed values
   without stringification.
4. Lower `anchor.*` references into structural `Anchor_value(origin_id, path
   parts)` bindings.
5. Lower all four Condition operators (Is, Contains, Greater_than,
   Greater_than_or_equal) to Core comparison operators with order preserved.
6. Resolve known input Facts through the lowering environment; reject unknown
   Facts with a bounded error.
7. Resolve known Capabilities to exact CapabilityId and ContractDigest from
   the environment; reject unknown and duplicate capabilities.
8. Reject any Tether containing `Together` with an explicit
   `Unsupported_construct` error.
9. Produce structurally equal Core programs for the same inputs (determinism).

## Relevant components

- `tethers-0.1/engine-ocaml/bin/tethers_core.ml` / `.mli` — dormant Core types
  (CORE-1/1A/1B vocabulary).
- `tethers-0.1/engine-ocaml/bin/tether_parser.ml` / `.mli` — parsed Human AST
  types (`tether`, `condition`, `action`, `action_item`, `value`, `operator`).
- `tethers-0.1/engine-ocaml/bin/dune` — module graph for both executables.
- `tethers-0.1/engine-ocaml/bin/tethers_core_lowerer.ml` / `.mli` — new
  lowerer module.

## Frozen decisions and invariants

- ID assignment: deterministic pre-canonical static IDs (`O_anchor`,
  `O_action_1`, ...). CORE-4 will canonicalise later.
- Anchor name preserved directly as `event_name`.
- Typed literals: lossless, no stringification.
- Anchor references: `"anchor.x.y"` → `Anchor_value(O_anchor, ["x"; "y"])`.
  Non-anchor refs → `Missing_anchor_reference`.
- Named action inputs: argument names become `capability_input_name`.
- Capability resolution: exact match from environment; duplicates rejected.
- Input Fact resolution: exact match; only referenced facts in `input_facts`.
- Guard operators: `Is→Equals`, `Contains→Contains`, `Greater_than→
  Greater_than`, `Greater_than_or_equal→Greater_than_or_equal`.
- Sequential flow: `entry_origin` + `success_continuations` chain; storage
  order carries no execution meaning.
- Together: explicit `Unsupported_construct`.
- No Core type changes needed.
- No evaluator/protocol/outcome/Rust changes.
- Determinism: structural equality for same inputs.

## Acceptance criteria

1. Single action produces correct Anchor + Action + entry + continuations
2. Three sequential actions: A1→A2→A3→complete chain explicit
3. Typed literals remain typed (string, int, bool)
4. Anchor binding: structural path parts
5. All four operators lower correctly, order preserved
6. Known fact resolves; unknown fact fails closed
7. Known capability resolves to exact ID/digest; unknown fails; duplicate fails
8. Together returns explicit unsupported-construct error
9. Determinism: repeat lowering produces structural equality

## Required verification

1. Packet checker at closeout: `control-v1/COMPLETE`
2. OCaml build: `dune build`
3. Lowerer tests: `dune runtest` — 44/44 assertions
4. Fixture suite: `check-fixtures.ps1` — 64 JSON + 32 JSONL
5. Engine suite: `test-engine.ps1` — 32 cases
6. MCP transcript suite: `test-mcp-transcripts.ps1` — 16 cases
7. Whitespace check: `git diff --check`
8. Rust formatter: `cargo fmt --check` (exit 0)
9. Complete diff inspection: only authorised files
10. Git status: clean worktree

## Forbidden changes

Do NOT modify: `tethers_evaluator.ml/.mli`, `tethers_protocol.ml/.mli`,
`tethers_outcome.ml/.mli`. Do not modify Rust. Do not change existing
runtime output. Do not route production evaluation through Core yet.
Do not modify `tethers_core.ml/.mli` unless a genuinely unavoidable
representation defect blocks correct lowering.

## Stop conditions

Committed CORE-2. STOP. Do NOT wire into evaluator, begin Core validation,
serialize Core, or begin CORE-3.

## Expected pre-existing changes

None.
