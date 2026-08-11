# Worker Note — TETHERS CORE-1B

Task: `TETHERS CORE-1B — Named Inputs & Explicit Success Flow`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `6295842688c7637172723ba46e43f128c3e86bc5`

Implementation checkpoint: `1011a644b3aa550c70643aaea33b7c2f301539b4`

## Requested outcome

Extend the dormant `Tethers_core` vocabulary so Action arguments retain their
capability input names structurally and sequential Origin execution flow is
explicitly stated rather than derived from hidden list ordering. CORE-1B fixes
two type-level omissions required before CORE-2 lowering can begin.

## Changes made

- `tethers-0.1/engine-ocaml/bin/tethers_core.ml` — added `capability_input_name`
  nominal type with `capability_input_name_of_string` / `string_of_capability_input_name`;
  added `action_input` record associating input name with binding; changed
  `action_origin.input_bindings : input_binding list` to `inputs : action_input list`;
  added `control_target = Origin_target of origin_id | Program_complete` and
  `success_continuation` record; added `entry_origin : origin_id option` and
  `success_continuations : success_continuation list` to `program`.
- `tethers-0.1/engine-ocaml/bin/tethers_core.mli` — mirror declarations with
  private `capability_input_name` constructor and conversion functions; added
  execution-semantics comment on `origin_sites` ordering; documented the
  CORE-1B extension in the module header.
- `docs/CURRENT_CLINE_TASK.md` — replaced the CORE-1A packet with the CORE-1B
  packet; set to `IN_PROGRESS`, then `COMPLETE` at closeout.
- `docs/worker-notes/2026-08-11-core-1b-named-inputs-success-flow.md` — this
  note.

No existing OCaml module other than `tethers_core.ml` and `tethers_core.mli`
was modified. `bin/dune` was not modified.

## Decisions and assumptions

- `capability_input_name` follows the Core ID discipline: distinct nominal type
  with private constructor and `of_string`/`to_string` conversion functions.
- `action_input = { input_name; binding }` makes the association between a
  capability argument name and its resolved binding explicit. The Action no
  longer depends on position to determine which argument a binding belongs to.
- `control_target` has two constructors: `Origin_target` names the successor;
  `Program_complete` means forward execution ends normally.
- `success_continuation` represents only the ordinary SUCCESS path. FAILURE,
  UNCERTAIN, and CANCELLED routing belongs to Branches (explicit alternative
  routing construct) and is not represented here.
- `program.entry_origin : origin_id option` names the first executable Origin
  after the Anchor matches and all entry guards pass. `None` means no deployable
  Origin exists (empty or guard-only program; a later validator may treat this
  as an error).
- Interface comments explicitly document that `origin_sites` list order carries
  no execution semantics; execution meaning comes from `entry_origin`, success
  continuations, Branch semantics, and composite scheduling.
- Together and Branch semantics are unmodified; CORE-1B does not replace Branch
  with generic outcome edges.

## Evidence

- Packet checker at start: `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` — PASS (`control-v1/IN_PROGRESS`, base `6295842`, HEAD `6295842`).
- OCaml build: `opam exec --switch="D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml" -- dune build` — PASS (no output, exit 0).
- Fixture suite: `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\check-fixtures.ps1` — PASS (64 JSON files, 32 JSONL files).
- Engine suite: `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-engine.ps1` — PASS (32 fixture cases plus happy-path, together, and line-ending determinism checks).
- MCP transcript suite: `pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-mcp-transcripts.ps1` — PASS (16 cases).
- Whitespace check: `git diff --check` — PASS (no whitespace errors; only informational LF-to-CRLF working-copy warnings).
- Rust formatter check (RUST_UNCHANGED, read-only): `cargo fmt --manifest-path tethers-0.1/host-rust/Cargo.toml --all -- --check` — PASS (exit 0).
- `git diff --stat` — only the two implementation files plus the packet; zero Rust/dependency changes.
- Working tree implementation files match checkpoint `1011a644b3aa550c70643aaea33b7c2f301539b4` exactly (empty `git diff` for the two files).
- Packet checker at closeout: `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` — PASS (`control-v1/COMPLETE`).

## Publication evidence

Branch pushed: `origin/feature/core-1-ocaml-core-types`. Full remote HEAD SHA:
`1011a644b3aa550c70643aaea33b7c2f301539b4` (resolved after the normal push).
Local `HEAD == remote HEAD`: confirmed. Final `git status --short --branch`:
clean.

## Discoveries

- None within packet scope.

## Remaining risks

- None known within packet scope. `capability_input_name` is a nominal type
  alongside existing IDs; no validation of name/capability-input consistency is
  performed (deferred to CORE-2 or later). Success-edge validity and Branch
  coherence are deferred to Core validation.

## Smallest next action

CORE-2: lower the Human Tether AST into Core `program`, reusing `core_value`,
`Evaluation_input`, `fact_guard`, `action_input`, `Anchor_value`,
`entry_origin`, and `success_continuation` without changing the existing
parser/evaluator runtime path.

## References

- `tethers-0.1/engine-ocaml/bin/tethers_core.ml`
- `tethers-0.1/engine-ocaml/bin/tethers_core.mli`
- `docs/CURRENT_CLINE_TASK.md`
- `docs/OCAML_GUIDE_FOR_AGENTS.md`
- `tethers-0.1/SPEC.md` sections 4–5 (action arguments, evaluation lifecycle)
- Implementation checkpoint commit `1011a644b3aa550c70643aaea33b7c2f301539b4`
- Base commit `6295842688c7637172723ba46e43f128c3e86bc5` (CORE-1A closeout)
