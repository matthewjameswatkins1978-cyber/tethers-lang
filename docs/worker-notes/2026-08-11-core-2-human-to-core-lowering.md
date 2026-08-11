# Worker Note — TETHERS CORE-2

Task: `TETHERS CORE-2 — Human AST → Core Lowering`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `b5daea00accff8e7617727a02ee524bfb80cd823`

Implementation checkpoint: `52032e42f8c1d44a801e79735272327c12ee004c`

## Requested outcome

Implement the first real Tethers Core lowering pass: translate the supported
sequential Tethers 0.1 subset from the parsed Human Tether AST into the dormant
Core program representation. CORE-2 is a deterministic sidecar path that does
not replace the existing evaluator.

## Changes made

- `tethers-0.1/engine-ocaml/bin/tethers_core_lowerer.ml` (new, 243 lines) —
  implementation of the lowerer. Defines `capability_binding`,
  `input_fact_binding`, `lowering_environment`, `lowering_error`, and the
  central `lower` function.
- `tethers-0.1/engine-ocaml/bin/tethers_core_lowerer.mli` (new, 42 lines) —
  public interface with types and `val lower`.
- `tethers-0.1/engine-ocaml/bin/dune` (modified) — added `tethers_core_lowerer`
  to both executable module lists; added a `(test ...)` stanza for the lowerer
  test executable.
- `tethers-0.1/engine-ocaml/bin/tethers_core_lowerer_test.ml` (new, 574 lines) —
  14 focused tests proving 44 individual assertions.
- `docs/CURRENT_CLINE_TASK.md` — replaced CORE-1B packet with CORE-2 (COMPLETE).
- `docs/worker-notes/2026-08-11-core-2-human-to-core-lowering.md` — this note.

No existing OCaml module other than `dune` was modified. No Core type changes
were needed. No Rust changes.

## Decisions and assumptions

### Lowering environment

```ocaml
type capability_binding = {
  source_name : string;
  capability_id : Tethers_core.capability_id;
  contract_digest : Tethers_core.capability_contract_digest;
}

type input_fact_binding = {
  source_name : string;
  fact : Tethers_core.fact;
}

type lowering_environment = {
  program_id : Tethers_core.program_id;
  core_version : Tethers_core.core_version;
  capabilities : capability_binding list;
  input_facts : input_fact_binding list;
}
```

The environment carries all semantic information the lowerer cannot invent:
program identity, Core version, capability-to-ID/digest mappings, and
host-input Fact declarations. No hashes, contract digests, or Fact types are
synthesised.

### ID assignment strategy

- Anchor: `O_anchor`
- Actions: `O_action_1`, `O_action_2`, ...

These are deterministic, source-position-derived, pre-canonical static
identities. The `.mli` header and worker note document that CORE-4 will
canonicalise semantic identity later. These IDs are explicitly not stable
for hashing.

### Anchor lowering

The Human Tether anchor name (`file.received`) becomes the Anchor Origin's
`event_name` field directly.

### Typed literal lowering

- `Tether_parser.String_value s` → `Tethers_core.String_value s`
- `Tether_parser.Int_value i` → `Tethers_core.Integer_value i`
- `Tether_parser.Bool_value b` → `Tethers_core.Boolean_value b`

No stringification. `Reference` values are handled separately (see below).

### Anchor reference lowering

`Tether_parser.Reference "anchor.customer.id"` is lowered to
`Anchor_value (O_anchor, ["customer"; "id"])`. The `"anchor."` prefix is
stripped; the remaining path is split on `'.'`. Only the structural path
survives into Core.

Non-`anchor.` references produce `Missing_anchor_reference`.

### Named action input lowering

Each parser argument `(name, value)` becomes an `action_input` record:
`{ input_name = capability_input_name_of_string name; binding = ... }`.
The input name is preserved structurally in Core; semantic association is
by explicit name, not position.

### Capability resolution

Each Human capability name is resolved through the environment's
`capabilities` list. Unknown capability → `Unknown_capability`. Multiple
bindings for the same source name → `Duplicate_capability`. The resolved
`CapabilityId` and `CapabilityContractDigest` populate the Action Origin.

### Input Fact resolution

Each Condition fact name is resolved through the environment's
`input_facts` list. Missing → `Unknown_fact`. The resolved `fact_id`
populates the `fact_guard`. Only actually-referenced Facts appear in
the program's `input_facts`.

### Entry guard lowering

Operators map directly: `Is → Equals`, `Contains → Contains`,
`Greater_than → Greater_than`, `Greater_than_or_equal → Greater_than_or_equal`.
Guard order preserves source order.

### Sequential action flow

For N actions A₁...Aₙ:
- `entry_origin = Some O_action_1`
- `O_action_1 SUCCESS → O_action_2`, ..., `O_action_n SUCCESS → Program_complete`

Execution meaning is explicit in `entry_origin` and `success_continuations`,
not derived from `origin_sites` list order.

### Together refusal

Any `Together` action item returns `Unsupported_construct "together"`.
No flattening, no reinterpretation, no silent drop.

### Non-success behaviour

No Branches are generated. FAILURE, UNCERTAIN, and CANCELLED have no
normal continuation, matching the rule that unhandled non-success stops
forward execution.

### Capability contracts

Only capabilities actually used by the lowered Program appear in
`capability_contracts`. Contracts are deduplicated by `capability_id`.

### Determinism

Given the same parsed Tether and lowering environment, `lower` returns
structurally equal results. No clock, randomness, environment reads,
filesystem reads, or UUID generation.

## Evidence

- Packet checker at start: not run (packet supplied in chat as READY; branch
  created from base `b5daea0`, verified equal to HEAD).
- OCaml build: `dune build` — PASS.
- Lowerer tests: `dune runtest` — PASS (44/44 assertions across 14 tests).
- Fixture suite: `check-fixtures.ps1` — PASS (64 JSON + 32 JSONL).
- Engine suite: `test-engine.ps1` — PASS (32 cases + determinism/line-ending).
- MCP transcript suite: `test-mcp-transcripts.ps1` — PASS (16 cases).
- Whitespace check: `git diff --check` — PASS (no whitespace errors).
- Rust formatter (RUST_UNCHANGED): `cargo fmt --check` — PASS (exit 0).
- `git diff --stat` — only dune + 3 new files; zero Rust/dependency changes.
- Working tree implementation files match checkpoint `52032e4` exactly.
- Packet checker at closeout: `control-v1/COMPLETE` — PASS.

## Test coverage (44 assertions)

| Test | Assertions | Scenario |
|---|---|---|
| test_single_action | 6 | 1 action: entry, continuations, anchor |
| test_three_actions | 11 | 3 actions: A1→A2→A3→complete chain |
| test_typed_literals | 7 | string, int, bool preserved as typed Core values |
| test_anchor_binding | 3 | anchor.customer.id → Anchor_value with path parts |
| test_conditions | 9 | all 4 operators, order preserved |
| test_known_fact_resolves | 1 | known fact → Ok |
| test_unknown_fact_fails | 1 | unknown fact → Unknown_fact error |
| test_known_capability_resolves | 2 | exact CapabilityId and ContractDigest |
| test_unknown_capability_fails | 1 | → Unknown_capability error |
| test_duplicate_capability_fails | 1 | → Duplicate_capability error |
| test_together_refused | 1 | → Unsupported_construct "together" |
| test_determinism | 3 | r1=r2=r3 across 3 calls |
| test_non_anchor_reference_rejected | 1 | → Missing_anchor_reference |
| test_no_actions_handled | 1 | parser rejects pre-lowering |

### Negative tests proven

- Action list position NOT used as implicit execution (verified by explicit
  continuation assertions in test_three_actions)
- Integers and booleans NOT stringified (verified by typed literal assertions)
- "anchor." prefix NOT preserved as semantic path text (verified by
  test_anchor_binding)
- Capability digests NOT invented (supplied via environment, verified by
  test_known_capability_resolves)
- Missing Fact declarations NOT invented (verified by test_unknown_fact_fails)
- Together NOT flattened (verified by test_together_refused)
- Normal continuation NOT created after non-success (no Branches generated
  for sequential source)

## Publication evidence

Branch pushed: `origin/feature/core-2-human-to-core-lowering`. Full remote
HEAD SHA: `d3e9be70dd674088035f98be8c3f88a1d2871dd0` (closeout commit).
Local `HEAD == remote HEAD`: confirmed. Final Git status: clean.

## Discoveries

- OCaml record field name ambiguity required explicit type annotations in
  `resolve_capability`, `resolve_fact`, and the internal `dedup` function
  because `capability_binding`, `input_fact_binding`, and
  `capability_contract` share field names (`source_name`, `capability_id`).
- No Core type changes were needed; the CORE-1B vocabulary was sufficient
  for the supported sequential subset.

## Remaining risks

- Together lowering is explicitly unsupported; this is intentional per the
  task packet and will be addressed in a later dedicated packet.
- CORE-2 lowering IDs (`O_anchor`, `O_action_N`) are pre-canonical and must
  not be treated as stable for hashing.
- The lowerer does not validate success-edge completeness or Branch semantics;
  those belong to later Core validation.

## Smallest next action

CORE-3: initial Core validation (success-edge completeness, unique IDs, etc.)
or Together lowering as a dedicated packet.

## References

- `tethers-0.1/engine-ocaml/bin/tethers_core_lowerer.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core_lowerer.mli`
- `tethers-0.1/engine-ocaml/bin/tethers_core_lowerer_test.ml`
- `tethers-0.1/engine-ocaml/bin/dune`
- `docs/CURRENT_CLINE_TASK.md`
- Implementation checkpoint commit `52032e42f8c1d44a801e79735272327c12ee004c`
- Base commit `b5daea00accff8e7617727a02ee524bfb80cd823` (CORE-1B closeout)
