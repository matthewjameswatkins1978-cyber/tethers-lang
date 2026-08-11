# Worker Note

Task: `TETHERS CORE-4 — Canonicalisation + ProgramDigest`

Task packet: `docs/CURRENT_CLINE_TASK.md`

Owner: `OpenCode`

Status: `COMPLETE`

Base commit: `7e94924d813bb7bd29ff234559cdb590bdddd016`

Implementation checkpoint: `f535713a83c3449f81dfd8c4cb624b4ba90f9dc2`

## Requested outcome

Implement the first authoritative canonical semantic identity layer for Tethers Core: semantic projection, structural fingerprinting, canonical ordering, internal ID assignment, reference rewriting, canonical byte encoding, SHA-256, and ProgramDigest. Same semantic meaning must produce same canonical bytes and ProgramDigest regardless of temporary IDs or storage order.

## Changes made

- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical.ml` — new (implementation, ~960 lines)
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical.mli` — new (interface)
- `tethers-0.1/engine-ocaml/bin/tethers_core_canonical_test.ml` — new (28 tests, ~840 lines)
- `tethers-0.1/engine-ocaml/bin/dune` — added test executable stanza
- `tethers-0.1/engine-ocaml/tethers_engine.opam` — added digestif dep
- `tethers-0.1/engine-ocaml/tethers_engine.opam.locked` — added digestif 1.3.1 + eqaf 0.10
- `docs/CURRENT_CLINE_TASK.md` — updated to CORE-4 task

## Decisions and assumptions

1. **Structural fingerprinting via iterative refinement**: Each entity gets a structural key computed by iterative Weisfeiler-Lehman-style refinement over the typed Core graph. Round 0 keys include only scalar semantic fields. Each subsequent round incorporates neighbor keys from the previous round. Refinement runs until fixpoint (converges in 2-3 iterations for the current Core structure, capped at 20).

2. **Canonical ID assignment**: Entities are sorted by their final structural key (string comparison) and assigned sequential canonical IDs: O1, O2... (Origins), F1, F2... (Facts), R1, R2... (Roles), B1, B2... (Branches), G1, G2... (Groups), BA1, BA2... (Batches), IT1, IT2... (ItemTemplates). External semantic IDs (CapabilityId, CapabilityContractDigest, HostSnapshotKey, CoreVersion, CapabilityInputName) are NOT renumbered.

3. **Canonical byte encoding**: Uses a tagged, self-delimiting format with version prefix `TETHERS_CORE_CANON_V1\x00`. Strings are length-prefixed (`<len>:<bytes>`), integers are decimal-terminated (`<digits>;`), lists are count-prefixed (`<count>:<items>`), variants and options use explicit tags. No whitespace, no platform-dependent newlines. Fields excluded: `program_id`, `schema_description` (on fact and capability_contract).

4. **SHA-256 via digestif**: The `Digestif.SHA256.digest_string` / `to_hex` API over the complete canonical byte sequence. ProgramDigest representation is `sha256:<64 lowercase hex chars>`.

5. **Collection sorting**: All semantically unordered collections sorted by canonical ID after rewriting. Semantically ordered structures (Anchor path components) preserved in original order. Branch outcomes sorted in fixed semantic order: Success, Failure, Uncertain, Cancelled.

6. **No dereference of opaque strings**: Deadline, role_fulfillment, batch_collection_provenance, batch_traversal_policy, and batch_objective are treated as exact opaque atoms. Different bytes → different semantics.

## Evidence

All commands run against implementation checkpoint `f535713a83c3449f81dfd8c4cb624b4ba90f9dc2`.

| Command | Result |
| --- | --- |
| `dune build @all` | PASS (no output, exit 0) |
| `dune runtest` | PASS (all tests, including CORE-3 validator and CORE-2 lowerer) |
| `git diff --check` | PASS (no trailing whitespace) |
| `cargo fmt --check` | PASS (exit 0, RUST_UNCHANGED) |
| `git status --short` | PASS (clean) |
| `git diff --stat` | 7 files changed, 1875 insertions, 61 deletions |

**New tests:** 28 focused tests covering:
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
