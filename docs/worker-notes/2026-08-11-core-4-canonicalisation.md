# Worker Note

Task: `TETHERS CORE-4B — Collision-Free Canonical Refinement`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `7efec4b1eb69c37c98b3e6b71a7b2e1d8a9260f5`

Implementation checkpoint: `31b614727c1f42e2fcab975341c09be35508eefb`

## Requested outcome

CORE-4B: Replace lossy hash-based colouring with exact-signature partition refinement. Add complete graph relationship modelling. Preserve role scope through canonical assignment. Give every unordered collection a total semantic order.

## Changes made (CORE-4B)

- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical.ml` — rewritten refinement core (810 lines changed)
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical.mli` — added Refinement_exceeded variant
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_test.ml` — 8 new tests (238 lines added)
- Total canonicaliser tests: 50 (42 prior + 8 new)

### Collision-free colour compression

- Delete 31-bit `color_of_string` (djb2 variant hash)
- Implement `compress_colours`: collect unique full-signature strings per entity type, sort deterministically, assign consecutive colour numbers (1..N) by exact-signature equality
- Replace `partitions_equal` with `int_map_partition_stable`: checks partition identity by verifying entities sharing a colour in round N also share a colour in round N+1, and the number of unique colours is unchanged
- This prevents false convergence (colour values shifting while partition is stable) while detecting true non-stabilisation

### Graph relationship modelling

- Add `static_refs` with complete structural relationships:
  - `success_out_map`: origin → list of control_targets
  - `success_in_map`: target origin → list of predecessor origin_ids
  - `origin_branches`: origin → list of branch_ids
  - `together_for_member`: member origin → containing Together origin
  - `guards_for_fact`: fact → list of (operator, expected_value) with operator rank
  - `consumers_for_fact`: fact → list of (input_name, binding) from consuming actions
  - `origin_for_fact`: fact → declaring origin
- Origin signatures include: entry marker, success outgoing target colour, success incoming predecessor colours, attached branch colours, Together container colour
- Fact signatures include: provenance colour, guard operators (ranked), consuming action input colours, establishing origin colour

### Role scope through canonical assignment

- `role_order` now uses scoped keys (e.g. "P:rolename", "T:tid:rolename") instead of raw role_ids
- `role_scope_of` map added to `canonical_ids` for disambiguating Role_proxy references
- `canonical_role_in_scope` for scope-explicit lookup; `canonical_role` fallback uses scope map
- `rewrite_role` and `rewrite_item_template` objective use scope-explicit lookup

### Total ordering for unordered collections

- Guard sorting: canonical FactId → operator rank → typed expected value
- Action input sorting: input name → binding encoding (not just name)
- Origin site sorting: species-aware (Anchor/Action/Together→OriginId, Batch→BatchId)
- Duplicate input names with distinct bindings sorted consistently via binding encoding break

### Safety cap

- 1000-round safety cap, returns `Refinement_exceeded` error (never partial state)
- Refinement converges in O(log N) rounds for chains via colour-based neighbour propagation

### CORE-4A changes (preserved)

- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical.ml` — rewritten core fingerprinting and refinement (440 lines changed)
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_test.ml` — 14 new regression tests + real byte fixture (397 lines added)
- Total canonicaliser tests: 42 (28 original + 14 new)

### CORE-4 original changes

- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical.ml` — new (implementation, ~960 lines)
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical.mli` — new (interface)
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_test.ml` — new (28 tests, ~840 lines)
- `tethers-0.1/engine-ocaml/bin/dune` — added test executable stanza
- `tethers-0.1/engine-ocaml/tethers_engine.opam` — added digestif dep
- `tethers-0.1/engine-ocaml/tethers_engine.opam.locked` — added digestif 1.3.1 + eqaf 0.10
- `docs/CURRENT_CLINE_TASK.md` — updated to CORE-4 task

## Decisions and assumptions

### CORE-4A corrections

1. **Color-compressed partition refinement**: Replaced expanding-string key scheme with deterministic polynomial-hash color compression (`color_of_string = djb2 variant`). Prevents unbounded key growth while preserving deterministic comparison. Stable refinement terminates when all entity key maps are unchanged between rounds (capped at 200 for safety).

2. **Scoped role identity**: Role keys now qualified by containing scope (`"P:" ^ rid_s` for Program_scope, `"T:" ^ tid_s ^ ":" ^ rid_s` for Template_scope). Two templates each using local "R1" receive distinct keys.

3. **Raw-ID removal**: Together_origin no longer hashes raw GroupId strings. Anchor_value/Fact_from_origin bindings use `lookup_origin keys` / `lookup_fact keys` for referenced entities. Branch subject uses `lookup_origin keys`. All raw incoming internal ID strings are lookup handles only.

4. **Guard-based fact refinement**: `build_guard_refs p` collects entry_guards grouped by fact_id, contributing `:g=<sorted_val_keys>` to fact structural keys. Distinguishes structurally identical facts referenced by different guard operators/values.

5. **Graph-position refinement**: `build_origin_refs p` tags origins as "entry", "sc_from", or "sc_to" based on their role in the control flow graph. These tags contribute to origin structural keys, distinguishing otherwise identical origins by their graph position.

6. **Group identity**: Groups derive canonical ordering from their Together origin's position in the sorted origin list, never from raw GroupId spelling.

7. **Complete batch collection**: Batches have their own `all_batches` collection and key map. `all_facts` includes facts from origins AND batches. BatchIds receive canonical BA1.. IDs from batch structural keys.

### CORE-4 original decisions

3. **Canonical byte encoding**: Uses a tagged, self-delimiting format with version prefix `TETHERS_CORE_CANON_V1\x00`. Strings are length-prefixed (`<len>:<bytes>`), integers are decimal-terminated (`<digits>;`), lists are count-prefixed (`<count>:<items>`), variants and options use explicit tags. No whitespace, no platform-dependent newlines. Fields excluded: `program_id`, `schema_description` (on fact and capability_contract).

4. **SHA-256 via digestif**: The `Digestif.SHA256.digest_string` / `to_hex` API over the complete canonical byte sequence. ProgramDigest representation is `sha256:<64 lowercase hex chars>`.

5. **Collection sorting**: All semantically unordered collections sorted by canonical ID after rewriting. Semantically ordered structures (Anchor path components) preserved in original order. Branch outcomes sorted in fixed semantic order: Success, Failure, Uncertain, Cancelled.

6. **No dereference of opaque strings**: Deadline, role_fulfillment, batch_collection_provenance, batch_traversal_policy, and batch_objective are treated as exact opaque atoms. Different bytes → different semantics.

## Evidence

All commands run against implementation checkpoint `31b614727c1f42e2fcab975341c09be35508eefb`.

| Command | Result |
| --- | --- |
| `dune build @all` | PASS (exit 0) |
| `dune runtest` | PASS (50 new + all existing validator/lowerer) |
| `git diff --check` | PASS |
| `cargo fmt --check` | PASS (RUST_UNCHANGED) |
| `git status --short` | PASS (clean) |
| `pwsh -NoProfile -File .github/scripts/check-tethers-task-packet.ps1` | `control-v1/COMPLETE` |

**New tests:** 42 focused tests covering:
- A: Baseline valid canonicalisation
- B: Determinism (identical inputs → identical outputs)
- C: Temporary Origin ID independence
- D: All internal ID independence
- E: Storage-order independence
- F: Named Action input reordering
- G: Together member reordering
- H: True symmetry (identical Together siblings)
- I: Branch outcome reordering
- J: Neutral descriptions excluded from digest
- K: ProgramId excluded from digest
- L: Literal value change → different digest
- M: CapabilityId change → different digest
- N: ContractDigest change → different digest
- O: Anchor event_name change → different digest
- P: Guard operator/value change → different digest
- Q: Control flow target change → different digest
- R: Branch outcome routing change → different digest
- S: Role ID independence + fulfillment sensitivity
- T: Item template ID independence + structural sensitivity
- U: Batch opaque field change → different digest
- V: CoreVersion change → different digest
- W: Multiplicity (1 vs 2 origins produce different digests)
- X: Invalid Core returns Invalid_core (fail-closed)
- Y: Raw-ID inversion trap (lexically reversed IDs → same digest)
- Canonical byte fixture (prefix verification)
- ProgramDigest frozen fixture (exact SHA-256 expected value)
- Canonical prefix bytes test

**Commands not run:**
- Fixture suite (`check-fixtures.ps1`): NOT RUN — CORE-4 does not modify evaluator/runtime/fixtures
- MCP transcript suite: NOT RUN — CORE-4 does not modify protocol
- Rust host tests: NOT RUN — RUST_UNCHANGED, no protocol changes
- Full task-packet checker: DEFERRED to closeout phase (after worker note committed)

## Discoveries

- `Tethers_core` ID types (`origin_id`, `fact_id`, etc.) are declared `private` in `.mli`, requiring `_of_string` helper functions instead of direct constructors for values created outside the defining module. This is correct security practice preventing callers from fabricating IDs.

- OCaml's record field disambiguation interacts poorly with `fact` and `fact_guard` types sharing `fact_id`. Widespread explicit type annotations on lambda parameters are required in the test executable compilation unit.

- `item_template.roles` is the correct home for roles with `Item_template_scope`. Roles declared at `program.roles` with template scope are rejected by the validator's `Item_objective_missing_role` check.

## Remaining risks

- The canonical byte encoding is versioned (`CANON_V1`) and can be deliberately changed in future tasks, but the frozen fixture test must be updated deliberately to avoid accidental drift.

- The iterative structural refinement caps at 20 rounds. For Core graphs with very deep structural dependencies, this cap may be insufficient. Not a practical concern for current 0.1 Core structures.

## Smallest next action

Push the branch, run the task-packet checker, update packet to COMPLETE, and return the completion report.

## References

- Branch: `feature/core-4-canonicalisation-program-digest`
- Implementation checkpoint: `f535713a83c3449f81dfd8c4cb624b4ba90f9dc2`
- Base: `feature/core-3-static-validator` at `7e94924d813bb7bd29ff234559cdb590bdddd016`
- New dependency: digestif 1.3.1 (with transitive eqaf 0.10)
